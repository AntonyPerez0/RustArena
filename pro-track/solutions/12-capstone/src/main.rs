//! The service's entry point: configuration, logging, database, then serve until asked
//! to stop.

use std::process::ExitCode;

use shortener::config::Config;
use shortener::{AppState, db, telemetry};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("shortener: configuration error: {err}");
            return ExitCode::from(2);
        }
    };
    telemetry::init(config.log_format);

    match run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!(error = %err, "shortener stopped with an error");
            ExitCode::FAILURE
        }
    }
}

async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect(&config.database_url).await?;
    let app = shortener::app(AppState::new(pool.clone(), config.public_url.clone()));

    let listener = TcpListener::bind(config.addr).await?;
    tracing::info!(
        addr = %listener.local_addr()?,
        public_url = %config.public_url,
        "listening"
    );
    shortener::serve(listener, app, shortener::shutdown_signal()).await?;

    // Requests are done: close the database cleanly (this also checkpoints the WAL).
    pool.close().await;
    tracing::info!("stopped");
    Ok(())
}
