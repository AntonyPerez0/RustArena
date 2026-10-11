//! Checks that processing orders emits the right spans and events, by installing a
//! small capturing `tracing` layer (written below) while the code runs.
//! Provided: don't change this file (the grader checks that). Make it pass.

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use orderd::config::{Config, PaymentConfig};
use orderd::orders::{Summary, process_batch, sample_orders};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

// ---------------------------------------------------------------------------------------
// A layer that records every event, with the fields of the spans it happened in.
// This is how you'd test logging in a real project, too.

/// Field values as text. Strings recorded with `%` or as `&str` appear as-is.
#[derive(Debug, Clone, Default)]
struct Fields(BTreeMap<String, String>);

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        let text = format!("{value:?}");
        let text = text.trim_matches('"').to_string();
        self.0.insert(field.name().to_string(), text);
    }
}

#[derive(Debug, Clone)]
struct SpanInfo {
    name: String,
    fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct Captured {
    level: Level,
    message: String,
    fields: BTreeMap<String, String>,
    /// The spans the event happened in, outermost first.
    spans: Vec<SpanInfo>,
}

impl Captured {
    fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }
    fn innermost_span(&self) -> Option<&SpanInfo> {
        self.spans.last()
    }
    fn span_named(&self, name: &str) -> Option<&SpanInfo> {
        self.spans.iter().find(|s| s.name == name)
    }
    /// The `order_id` field of the `process_order` span this event happened in.
    fn order_id(&self) -> Option<&str> {
        self.span_named("process_order")?.fields.get("order_id").map(String::as_str)
    }
}

struct Capture(Arc<Mutex<Vec<Captured>>>);

impl<S> Layer<S> for Capture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        ctx.span(id).unwrap().extensions_mut().insert(fields);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();
        if let Some(fields) = extensions.get_mut::<Fields>() {
            values.record(fields);
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let message = fields.0.remove("message").unwrap_or_default();
        let spans = ctx
            .event_scope(event)
            .map(|scope| {
                scope
                    .from_root()
                    .map(|span| SpanInfo {
                        name: span.name().to_string(),
                        fields: span.extensions().get::<Fields>().cloned().unwrap_or_default().0,
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.0.lock().unwrap().push(Captured {
            level: *event.metadata().level(),
            message,
            fields: fields.0,
            spans,
        });
    }
}

/// Processes the sample orders (no retry delays) and returns what was logged.
fn run_and_capture() -> (Summary, Vec<Captured>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Capture(events.clone()));
    let config = Config {
        payment: PaymentConfig { max_attempts: 3, backoff_ms: 0 },
        ..Config::default()
    };
    let summary = tracing::subscriber::with_default(subscriber, || {
        process_batch(&sample_orders(), &config)
    });
    let events = events.lock().unwrap().clone();
    (summary, events)
}

fn find<'a>(events: &'a [Captured], message: &str) -> Vec<&'a Captured> {
    events.iter().filter(|e| e.message == message).collect()
}

fn dump(events: &[Captured]) -> String {
    events
        .iter()
        .map(|e| {
            let spans: Vec<&str> = e.spans.iter().map(|s| s.name.as_str()).collect();
            format!("  {:5} {:?} {:?} in {:?}\n", e.level, e.message, e.fields, spans)
        })
        .collect()
}

// ---------------------------------------------------------------------------------------

#[test]
fn the_business_logic_still_works() {
    let (summary, _) = run_and_capture();
    assert_eq!(
        summary,
        Summary { accepted: 3, rejected: 2, failed: 1, revenue_cents: 13_393 }
    );
    assert_eq!(
        summary.to_string(),
        "processed 6 orders: 3 accepted, 2 rejected, 1 failed (revenue 133.93)"
    );
}

#[test]
fn every_event_about_an_order_is_inside_its_span() {
    let (_, events) = run_and_capture();
    let mut ids: Vec<&str> = events.iter().filter_map(Captured::order_id).collect();
    ids.dedup();
    assert_eq!(
        ids,
        ["1", "2", "3", "4", "5", "6"],
        "expected events inside a `process_order` span with an `order_id` field, got:\n{}",
        dump(&events)
    );
    for event in &events {
        if let Some(span) = event.span_named("process_order") {
            assert!(span.fields.contains_key("customer"), "the span should record `customer`");
        }
    }
}

#[test]
fn accepted_orders_are_info_events_with_their_total() {
    let (_, events) = run_and_capture();
    let accepted = find(&events, "order accepted");
    let got: Vec<(Option<&str>, Option<&str>)> =
        accepted.iter().map(|e| (e.order_id(), e.field("total_cents"))).collect();
    assert_eq!(
        got,
        [(Some("1"), Some("8997")), (Some("4"), Some("1999")), (Some("6"), Some("2397"))],
        "\"order accepted\" events (order_id, total_cents). Everything logged:\n{}",
        dump(&events)
    );
    assert!(accepted.iter().all(|e| e.level == Level::INFO), "use info! for accepted orders");
}

#[test]
fn rejected_orders_are_warnings_with_a_reason() {
    let (_, events) = run_and_capture();
    let rejected = find(&events, "order rejected");
    let ids: Vec<Option<&str>> = rejected.iter().map(|e| e.order_id()).collect();
    assert_eq!(ids, [Some("2"), Some("3")], "Everything logged:\n{}", dump(&events));
    for event in rejected {
        assert_eq!(event.level, Level::WARN, "use warn! for rejected orders");
        let reason = event.field("reason").unwrap_or_default();
        assert!(!reason.is_empty(), "\"order rejected\" needs a `reason` field: {event:?}");
    }
}

#[test]
fn line_pricing_is_logged_at_debug_level() {
    let (_, events) = run_and_capture();
    let priced = find(&events, "priced line");
    // Order 2 has no lines; the other five have six lines between them.
    assert_eq!(priced.len(), 6, "Everything logged:\n{}", dump(&events));
    for event in priced {
        assert_eq!(event.level, Level::DEBUG);
        for field in ["sku", "quantity", "line_cents"] {
            assert!(event.field(field).is_some(), "\"priced line\" needs `{field}`: {event:?}");
        }
    }
}

#[test]
fn each_payment_attempt_has_its_own_span() {
    let (_, events) = run_and_capture();
    let failed_attempts: Vec<(Option<&str>, Option<&str>)> = find(&events, "payment attempt failed")
        .iter()
        .map(|e| {
            let attempt = e.innermost_span().filter(|s| s.name == "charge");
            (e.order_id(), attempt.and_then(|s| s.fields.get("attempt")).map(String::as_str))
        })
        .collect();
    // flaky-co (order 4) times out twice before paying; broke-co (order 5) is declined once.
    assert_eq!(
        failed_attempts,
        [(Some("4"), Some("1")), (Some("4"), Some("2")), (Some("5"), Some("1"))],
        "\"payment attempt failed\" events (order_id, attempt of the `charge` span). Everything logged:\n{}",
        dump(&events)
    );
    for event in find(&events, "payment attempt failed") {
        assert_eq!(event.level, Level::WARN);
        assert!(event.field("error").is_some(), "include the `error`: {event:?}");
    }
}

#[test]
fn a_failed_payment_is_an_error_event() {
    let (_, events) = run_and_capture();
    let failed = find(&events, "payment failed");
    assert_eq!(failed.len(), 1, "Everything logged:\n{}", dump(&events));
    assert_eq!(failed[0].level, Level::ERROR);
    assert_eq!(failed[0].order_id(), Some("5"));
    assert_eq!(failed[0].field("error"), Some("card declined"));
}

#[test]
fn the_batch_ends_with_a_summary_event() {
    let (_, events) = run_and_capture();
    let finished = find(&events, "batch finished");
    assert_eq!(finished.len(), 1, "Everything logged:\n{}", dump(&events));
    let event = finished[0];
    assert_eq!(event.level, Level::INFO);
    for (field, value) in [("accepted", "3"), ("rejected", "2"), ("failed", "1"), ("revenue_cents", "13393")]
    {
        assert_eq!(event.field(field), Some(value), "field `{field}` of {event:?}");
    }
    let batch = event.span_named("process_batch").expect("inside the `process_batch` span");
    assert_eq!(batch.fields.get("orders").map(String::as_str), Some("6"));
    assert_eq!(batch.fields.get("service").map(String::as_str), Some("orderd"));
}
