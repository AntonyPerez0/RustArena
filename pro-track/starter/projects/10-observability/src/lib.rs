//! orderd: a batch order processor you can operate.
//!
//! - [`config`]: typed, validated configuration from a TOML file and environment variables.
//! - [`telemetry`]: installs the `tracing` subscriber (levels from `RUST_LOG`, text or JSON).
//! - [`orders`]: the business logic, instrumented with spans and events.

pub mod config;
pub mod orders;
pub mod telemetry;
