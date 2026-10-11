//! Logging setup: one place that decides where logs go and what they look like.

use crate::config::LogFormat;

/// Installs the global `tracing` subscriber. Call it once, at startup.
///
/// - Which events are kept comes from `RUST_LOG` (for example `warn`, `debug` or
///   `info,orderd::orders=debug`), and is `info` when it isn't set.
/// - `LogFormat::Text` writes readable lines (no color codes unless stderr is a
///   terminal); `LogFormat::Json` writes one JSON object per line, including the current
///   span and the list of spans it's in.
/// - Logs go to stderr, so stdout stays free for the program's real output.
pub fn init(format: LogFormat) {
    // TODO: tracing_subscriber::registry() with an EnvFilter and a fmt layer.
    let _ = format;
    todo!("telemetry::init")
}
