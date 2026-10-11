//! The service's entry point. You write it:
//!
//! 1. Load the configuration with `Config::from_env()`. On error, print it and exit with
//!    code 2 (logging isn't set up yet).
//! 2. Install logging with `telemetry::init`.
//! 3. Connect to the database with `db::connect` (this runs the migrations).
//! 4. Bind a `tokio::net::TcpListener` to `config.addr` and log the address.
//! 5. `shortener::serve(listener, app, shortener::shutdown_signal())`, then close the pool.

#[tokio::main]
async fn main() {
    todo!("wire the service together")
}
