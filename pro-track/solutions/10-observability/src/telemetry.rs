//! Logging setup: one place that decides where logs go and what they look like.

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use crate::config::LogFormat;

/// Installs the global `tracing` subscriber. Call it once, at startup.
///
/// - Which events are kept comes from `RUST_LOG` (for example `warn`, `debug` or
///   `info,orderd::orders=debug`), and is `info` when it isn't set.
/// - `LogFormat::Text` writes readable lines; `LogFormat::Json` writes one JSON object per
///   line, including the current span and the list of spans it's in.
/// - Logs go to stderr, so stdout stays free for the program's real output.
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
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_writer(std::io::stderr),
            )
            .init(),
    }
}
