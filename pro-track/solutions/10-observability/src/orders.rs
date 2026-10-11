//! The business logic: price each order, check it, charge the customer (retrying when the
//! payment gateway has a hiccup) and count the results.
//!
//! Instrumented with `tracing`: every order gets a span with its id, so each event below
//! is automatically tagged with the order it belongs to.

use std::fmt;
use std::thread;
use std::time::Duration;

use tracing::{debug, error, info, info_span, instrument, warn};

use crate::config::{Config, PaymentConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub sku: String,
    pub quantity: u32,
    pub unit_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: u64,
    pub customer: String,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Accepted { total_cents: u64 },
    Rejected { reason: String },
    Failed { reason: String },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Summary {
    pub accepted: usize,
    pub rejected: usize,
    pub failed: usize,
    pub revenue_cents: u64,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "processed {} orders: {} accepted, {} rejected, {} failed (revenue {}.{:02})",
            self.accepted + self.rejected + self.failed,
            self.accepted,
            self.rejected,
            self.failed,
            self.revenue_cents / 100,
            self.revenue_cents % 100
        )
    }
}

fn line(sku: &str, quantity: u32, unit_cents: u64) -> Line {
    Line {
        sku: sku.to_string(),
        quantity,
        unit_cents,
    }
}

/// The orders `cargo run` processes. Each one exercises a different path.
pub fn sample_orders() -> Vec<Order> {
    let order = |id, customer: &str, lines| Order {
        id,
        customer: customer.to_string(),
        lines,
    };
    vec![
        order(
            1,
            "ada",
            vec![line("KB-01", 1, 4999), line("MS-02", 2, 1999)],
        ),
        order(2, "bob", vec![]),
        order(3, "carol", vec![line("SC-07", 5, 49_999)]),
        order(4, "flaky-co", vec![line("MS-02", 1, 1999)]),
        order(5, "broke-co", vec![line("KB-01", 1, 4999)]),
        order(6, "dan", vec![line("CB-11", 3, 799)]),
    ]
}

/// Processes every order and logs a summary at the end.
#[instrument(skip_all, fields(service = %config.service_name, orders = orders.len()))]
pub fn process_batch(orders: &[Order], config: &Config) -> Summary {
    let mut summary = Summary::default();
    for order in orders {
        match process_order(order, config) {
            Outcome::Accepted { total_cents } => {
                summary.accepted += 1;
                summary.revenue_cents += total_cents;
            }
            Outcome::Rejected { .. } => summary.rejected += 1,
            Outcome::Failed { .. } => summary.failed += 1,
        }
    }
    info!(
        accepted = summary.accepted,
        rejected = summary.rejected,
        failed = summary.failed,
        revenue_cents = summary.revenue_cents,
        "batch finished"
    );
    summary
}

/// Prices, checks and charges one order.
#[instrument(skip_all, fields(order_id = order.id, customer = %order.customer))]
pub fn process_order(order: &Order, config: &Config) -> Outcome {
    if order.lines.is_empty() {
        return reject("order has no lines".to_string());
    }

    let mut total_cents = 0;
    for line in &order.lines {
        let line_cents = line.unit_cents * u64::from(line.quantity);
        debug!(sku = %line.sku, quantity = line.quantity, line_cents, "priced line");
        total_cents += line_cents;
    }

    if total_cents > config.max_order_cents {
        return reject(format!(
            "total {total_cents} cents is over the limit of {} cents",
            config.max_order_cents
        ));
    }

    match charge_with_retries(&order.customer, total_cents, &config.payment) {
        Ok(()) => {
            info!(total_cents, "order accepted");
            Outcome::Accepted { total_cents }
        }
        Err(err) => {
            error!(error = %err, "payment failed");
            Outcome::Failed {
                reason: err.to_string(),
            }
        }
    }
}

fn reject(reason: String) -> Outcome {
    warn!(%reason, "order rejected");
    Outcome::Rejected { reason }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PaymentError {
    #[error("payment gateway timed out")]
    Timeout,
    #[error("card declined")]
    Declined,
}

impl PaymentError {
    /// Worth retrying? A timeout may work next time; a declined card won't.
    pub fn is_transient(self) -> bool {
        self == PaymentError::Timeout
    }
}

fn charge_with_retries(
    customer: &str,
    amount_cents: u64,
    payment: &PaymentConfig,
) -> Result<(), PaymentError> {
    let mut attempt = 1;
    loop {
        // One span per attempt: its events carry the attempt number automatically.
        let result = info_span!("charge", attempt, amount_cents).in_scope(|| {
            let result = gateway_charge(customer, attempt);
            if let Err(err) = result {
                warn!(error = %err, "payment attempt failed");
            }
            result
        });
        match result {
            Ok(()) => return Ok(()),
            Err(err) if err.is_transient() && attempt < payment.max_attempts => {
                thread::sleep(Duration::from_millis(
                    payment.backoff_ms * u64::from(attempt),
                ));
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

/// A pretend payment gateway. It's deterministic, so the tests are too:
/// "broke-co" is always declined, and "flaky-co" times out on its first two attempts.
pub fn gateway_charge(customer: &str, attempt: u32) -> Result<(), PaymentError> {
    match customer {
        "broke-co" => Err(PaymentError::Declined),
        "flaky-co" if attempt < 3 => Err(PaymentError::Timeout),
        _ => Ok(()),
    }
}
