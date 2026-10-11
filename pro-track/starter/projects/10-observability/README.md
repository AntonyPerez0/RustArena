# 10 · Logging, tracing and configuration

It's 3 a.m., orders are failing in production, and all you have is what the service wrote down. Was it one customer or all of them? Which attempt failed? Code that's easy to operate answers those questions from its logs, and refuses to start with a bad setting instead of misbehaving for hours.

`orderd` processes a batch of orders: it prices them, rejects suspicious ones and charges customers, retrying when the payment gateway times out. The logic works. The logging is `println!` debugging, and there's no configuration at all. You'll fix both.

**You'll practice:** `tracing` events, spans and fields, `#[instrument]`, `tracing-subscriber` with `EnvFilter` and JSON output, layered configuration with serde and TOML, validation, and testing what your code logs.

## Background

### Events, spans and fields

`tracing` is the standard for logging in Rust (Tokio, axum, SQLx and most big crates use it). It has two building blocks:

- An **event** is something that happened, at a **level**: `info!(total_cents, "order accepted")`.
- A **span** is a period of time with context, like "processing order 4". Every event inside a span is tagged with the span's fields, so you write the order id once and every line about that order carries it.

Fields make logs **structured**: `key = value` pairs that a machine can filter on (`order_id=4`) instead of words buried in a sentence. `%x` records a value with `Display`, `?x` with `Debug`, and a bare `total_cents` uses the variable of that name.

```rust
#[instrument(skip_all, fields(order_id = order.id, customer = %order.customer))]
pub fn process_order(order: &Order, config: &Config) -> Outcome { ... }

let result = info_span!("charge", attempt).in_scope(|| gateway_charge(customer, attempt));
warn!(error = %err, "payment attempt failed");
```

`#[instrument]` wraps a function in a span named after it. By default it records every argument; `skip_all` plus explicit `fields(...)` keeps big or secret values (passwords, whole configs) out of your logs.

Pick levels the way teams do: **ERROR** means someone should look, **WARN** means unexpected but handled, **INFO** marks normal business milestones, **DEBUG** is detail for developers, and **TRACE** is everything.

### Subscribers: where logs go

`tracing` only emits; a **subscriber** decides what to keep and how to write it. With `tracing-subscriber` you stack layers on a registry:

- `EnvFilter` reads `RUST_LOG`: `warn`, `debug`, or per module like `error,orderd::orders=debug`. Operators change verbosity without a rebuild.
- `fmt::layer()` writes readable text; `fmt::layer().json()` writes one JSON object per line for log collectors (Loki, Elasticsearch, Datadog, CloudWatch), including the current span and the list of spans.

Logs go to **stderr**; **stdout** is for the program's real output (here, the summary line), so `orderd > summary.txt` works. Turn color codes off when stderr isn't a terminal, or your log files fill with `\x1b[32m`.

### Configuration

Settings come in layers, each overriding the one before: **defaults**, then a **TOML file** (`ORDERD_CONFIG` names it), then **environment variables** (how containers and CI pass settings). The result is checked once, at startup, and turned into a typed `Config`. If you hold a `Config`, every value in it is valid, and nobody re-checks it deep in the code.

- Deserialize the file into a struct whose fields are all `Option<T>`, with `#[serde(deny_unknown_fields)]`: a typo like `max_atempts` must be an error, not silently ignored.
- Errors name the setting and the bad value. `orderd` exits with code 2 (the convention for "started wrong") before doing any work.
- `from_sources` takes the environment as a function instead of calling `std::env::var`. In Rust 2024 `std::env::set_var` is `unsafe` (other threads may be reading the environment), so tests shouldn't change the real environment. Passing it in makes configuration a pure function you can test with any values.

### Testing logs

`tests/telemetry.rs` installs a small `Layer` that records every event with the fields of its spans, runs the batch, and checks the result. Read it: it's how you'd test logging in a real codebase.

## The logging contract

| What | Level | Fields | Where |
|---|---|---|---|
| span `process_batch` | | `service`, `orders` (count) | `process_batch` |
| span `process_order` | | `order_id`, `customer` | `process_order` |
| span `charge` | | `attempt` | one per payment attempt |
| event `"priced line"` | DEBUG | `sku`, `quantity`, `line_cents` | each line of an order |
| event `"order rejected"` | WARN | `reason` | empty or over-limit orders |
| event `"payment attempt failed"` | WARN | `error` | inside its `charge` span |
| event `"order accepted"` | INFO | `total_cents` | |
| event `"payment failed"` | ERROR | `error` | after the last attempt, or a permanent error |
| event `"batch finished"` | INFO | `accepted`, `rejected`, `failed`, `revenue_cents` | end of the batch |

## Your tasks

1. **Configuration.** Implement `from_sources` in `src/config.rs` (the doc comment lists every key, variable and rule). Run `cargo test --test config`.
2. **Subscriber.** Implement `telemetry::init` in `src/telemetry.rs`: an `EnvFilter` defaulting to `info`, then a text or JSON `fmt` layer writing to stderr.
3. **Instrumentation.** In `src/orders.rs`, replace every `println!`/`eprintln!` with the spans and events in the table. Run `cargo test --test telemetry`.
4. Run it every way an operator would:

```bash
cargo run
RUST_LOG=debug cargo run
RUST_LOG=warn ORDERD_LOG_FORMAT=json cargo run
RUST_LOG=info,orderd::orders=debug cargo run
ORDERD_CONFIG=config/orderd.toml ORDERD_PAYMENT_MAX_ATTEMPTS=2 cargo run
ORDERD_PAYMENT_MAX_ATTEMPTS=0 cargo run; echo "exit code $?"
cargo test
bash ../../tools/grade.sh 10-observability
```

## What the grader checks

- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean.
- The three test files are unchanged, and all their tests pass.
- `src/orders.rs` has no `println!`, `eprintln!` or `dbg!` left.
- `RUST_LOG=warn` with JSON output prints only WARN and ERROR lines, each a JSON object, with the summary alone on stdout.
- `RUST_LOG=debug` with `config/orderd.toml` prints readable DEBUG lines inside the order spans.

## Hints

- A generic helper keeps the environment parsing short: `fn from_env<T: FromStr>(env: &impl Fn(&str) -> Option<String>, var: &'static str) -> Result<Option<T>, ConfigError> where T::Err: Display`. Give `LogFormat` a `FromStr` impl to use it.
- `EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy()` gives you the `info` default.
- The two `fmt` layers have different types, so call `.init()` in each `match` arm rather than storing the layer in a variable.
- `std::io::stderr().is_terminal()` (from `std::io::IsTerminal`) tells you whether to use color.
- If a telemetry test fails, its message prints everything that was logged, with the spans each event was in.

## Going further

- Export spans to Jaeger or Grafana Tempo with `tracing-opentelemetry`, and follow one order across services.
- Write logs to a rotating file without slowing the program down using `tracing-appender`'s non-blocking writer.
- Add a `--check-config` flag that loads and prints the configuration, then exits: handy in deployment pipelines.
- Compare your loader with the `figment` or `config` crates. What do they add, and what do they hide?
- Reload the log level while the program runs with `tracing_subscriber::reload`.
