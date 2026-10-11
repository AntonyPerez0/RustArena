//! Provided. Loads the configuration, sets up logging, processes the sample orders and
//! prints a one-line summary on stdout. Logs go to stderr.
//!
//!   cargo run
//!   RUST_LOG=debug cargo run
//!   ORDERD_LOG_FORMAT=json cargo run
//!   ORDERD_CONFIG=config/orderd.toml ORDERD_PAYMENT_MAX_ATTEMPTS=2 cargo run

use std::process::ExitCode;

use orderd::{config, orders, telemetry};

fn main() -> ExitCode {
    // Logging isn't set up until we know the log format, so a configuration error can only
    // be printed. Exit code 2 is the convention for "you started me wrong".
    let config = match config::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("orderd: configuration error: {err}");
            return ExitCode::from(2);
        }
    };
    telemetry::init(config.log_format);
    tracing::info!(service = %config.service_name, config = ?config, "starting");

    let summary = orders::process_batch(&orders::sample_orders(), &config);
    println!("{summary}");
    ExitCode::SUCCESS
}
