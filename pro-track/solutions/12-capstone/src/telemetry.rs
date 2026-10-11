//! Logging setup, as in project 10.

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use crate::config::LogFormat;

/// Installs the global subscriber: levels from `RUST_LOG` (default `info`), text or JSON
/// lines on stderr.
pub fn init(format: LogFormat) {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let registry = tracing_subscriber::registry().with(filter);
    match format {
        LogFormat::Text => registry
            .with(
                fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_ansi(std::io::stderr().is_terminal()),
            )
            .init(),
        LogFormat::Json => registry
            .with(fmt::layer().json().with_writer(std::io::stderr))
            .init(),
    }
}
