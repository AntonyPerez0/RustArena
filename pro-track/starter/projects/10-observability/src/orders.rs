//! The business logic: price each order, check it, charge the customer (retrying when the
//! payment gateway has a hiccup) and count the results.
//!
//! The logic works. The logging doesn't: it's println! debugging that mixes with the
//! program's output, has no levels, and can't be filtered or parsed. Replace it with
//! `tracing` spans and events (see the README for exactly which ones).

use std::fmt;
use std::thread;
use std::time::Duration;

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
        order(1, "ada", vec![line("KB-01", 1, 4999), line("MS-02", 2, 1999)]),
        order(2, "bob", vec![]),
        order(3, "carol", vec![line("SC-07", 5, 49_999)]),
        order(4, "flaky-co", vec![line("MS-02", 1, 1999)]),
        order(5, "broke-co", vec![line("KB-01", 1, 4999)]),
        order(6, "dan", vec![line("CB-11", 3, 799)]),
    ]
}

/// Processes every order and logs a summary at the end.
pub fn process_batch(orders: &[Order], config: &Config) -> Summary {
    println!("[{}] processing {} orders", config.service_name, orders.len());
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
    println!("done: {summary:?}");
    summary
}

/// Prices, checks and charges one order.
pub fn process_order(order: &Order, config: &Config) -> Outcome {
    println!("order {} from {}", order.id, order.customer);
    if order.lines.is_empty() {
        return reject("order has no lines".to_string());
    }

    let mut total_cents = 0;
    for line in &order.lines {
        let line_cents = line.unit_cents * u64::from(line.quantity);
        println!("  {} x{} = {}", line.sku, line.quantity, line_cents);
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
            println!("  OK, charged {total_cents}");
            Outcome::Accepted { total_cents }
        }
        Err(err) => {
            eprintln!("  PAYMENT FAILED: {err}");
            Outcome::Failed {
                reason: err.to_string(),
            }
        }
    }
}

fn reject(reason: String) -> Outcome {
    eprintln!("  rejected: {reason}");
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
        println!("  charging {amount_cents}, attempt {attempt}");
        let result = gateway_charge(customer, attempt);
        if let Err(err) = result {
            eprintln!("  attempt {attempt} failed: {err}");
        }
        match result {
            Ok(()) => return Ok(()),
            Err(err) if err.is_transient() && attempt < payment.max_attempts => {
                thread::sleep(Duration::from_millis(payment.backoff_ms * u64::from(attempt)));
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
