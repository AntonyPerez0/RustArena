//! Configuration: defaults, then a TOML file, then environment variables, validated into
//! a typed [`Config`]. A bad setting stops the program at startup with a message that
//! names it, instead of failing strangely an hour later.
//!
//! The types, the defaults and `load` are provided. You write `from_sources`.

use std::path::PathBuf;

use serde::Deserialize;

/// The environment variable holding the path of the TOML config file (optional).
pub const CONFIG_PATH_VAR: &str = "ORDERD_CONFIG";

/// The validated configuration. If you hold one, every value in it is usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub service_name: String,
    pub log_format: LogFormat,
    /// Orders whose total is above this are rejected.
    pub max_order_cents: u64,
    pub payment: PaymentConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaymentConfig {
    /// How many times to try a payment, counting the first try. 1 to 10.
    pub max_attempts: u32,
    /// Wait `backoff_ms * attempt` milliseconds before retrying. At most 10000.
    pub backoff_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable lines, for a terminal.
    Text,
    /// One JSON object per line, for log collectors.
    Json,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            service_name: "orderd".to_string(),
            log_format: LogFormat::Text,
            max_order_cents: 100_000,
            payment: PaymentConfig {
                max_attempts: 3,
                backoff_ms: 100,
            },
        }
    }
}

/// Everything that can be wrong with the configuration. Each message names the setting.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("can't read config file {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("config file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("environment variable {var}={value:?}: {reason}")]
    Env {
        var: &'static str,
        value: String,
        reason: String,
    },
    #[error("invalid setting {field}: {reason}")]
    Invalid { field: &'static str, reason: String },
}

/// Builds the configuration from the TOML file's text (if there is a file) and the
/// environment, then validates it. Precedence: environment, then file, then defaults.
///
/// `env` looks up an environment variable. Passing it in (instead of calling
/// `std::env::var` here) keeps this a pure function that tests can call with any
/// environment they like.
///
/// | File key                | Environment variable          | Rule              |
/// |-------------------------|-------------------------------|-------------------|
/// | `service_name`          | `ORDERD_SERVICE_NAME`         | not blank         |
/// | `log_format`            | `ORDERD_LOG_FORMAT`           | `text` or `json`  |
/// | `max_order_cents`       | `ORDERD_MAX_ORDER_CENTS`      | greater than 0    |
/// | `[payment] max_attempts`| `ORDERD_PAYMENT_MAX_ATTEMPTS` | 1 to 10           |
/// | `[payment] backoff_ms`  | `ORDERD_PAYMENT_BACKOFF_MS`   | 0 to 10000        |
///
/// Errors: bad TOML, a wrong type or an unknown key in the file is `Parse`; an
/// environment value that doesn't parse is `Env`; a value outside its rule is `Invalid`
/// with `field` set to the file key (`"payment.max_attempts"` for nested ones).
pub fn from_sources(
    file: Option<&str>,
    env: impl Fn(&str) -> Option<String>,
) -> Result<Config, ConfigError> {
    // TODO:
    // 1. Deserialize the file into a struct whose fields are all Option<...>, with
    //    #[serde(deny_unknown_fields)] so typos are errors.
    // 2. Start from Config::default() and apply the file's values, then the environment's.
    // 3. Validate.
    let _ = (file, env);
    todo!("from_sources")
}

/// Loads the configuration of the running program: the file named by `ORDERD_CONFIG`
/// (if set) and the process environment.
pub fn load() -> Result<Config, ConfigError> {
    let env = |name: &str| std::env::var(name).ok();
    let text = match env(CONFIG_PATH_VAR) {
        Some(path) => Some(
            std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
                path: path.into(),
                source,
            })?,
        ),
        None => None,
    };
    from_sources(text.as_deref(), env)
}
