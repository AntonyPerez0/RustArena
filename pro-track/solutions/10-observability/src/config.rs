//! Configuration: defaults, then a TOML file, then environment variables, validated into
//! a typed [`Config`]. A bad setting stops the program at startup with a message that
//! names it, instead of failing strangely an hour later.
//!
//! The types, the defaults and `load` are provided. You write `from_sources`.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

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

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "text" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            _ => Err("expected \"text\" or \"json\"".to_string()),
        }
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LogFormat::Text => "text",
            LogFormat::Json => "json",
        })
    }
}

/// The TOML file as written: every key optional, unknown keys rejected (a typo like
/// `max_atempts` should be an error, not silently ignored).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    service_name: Option<String>,
    log_format: Option<LogFormat>,
    max_order_cents: Option<u64>,
    #[serde(default)]
    payment: FilePayment,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FilePayment {
    max_attempts: Option<u32>,
    backoff_ms: Option<u64>,
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
    let file: FileConfig = match file {
        Some(text) => toml::from_str(text)?,
        None => FileConfig::default(),
    };

    let defaults = Config::default();
    let mut config = Config {
        service_name: file.service_name.unwrap_or(defaults.service_name),
        log_format: file.log_format.unwrap_or(defaults.log_format),
        max_order_cents: file.max_order_cents.unwrap_or(defaults.max_order_cents),
        payment: PaymentConfig {
            max_attempts: file
                .payment
                .max_attempts
                .unwrap_or(defaults.payment.max_attempts),
            backoff_ms: file
                .payment
                .backoff_ms
                .unwrap_or(defaults.payment.backoff_ms),
        },
    };

    if let Some(name) = env("ORDERD_SERVICE_NAME") {
        config.service_name = name;
    }
    if let Some(format) = from_env(&env, "ORDERD_LOG_FORMAT")? {
        config.log_format = format;
    }
    if let Some(cents) = from_env(&env, "ORDERD_MAX_ORDER_CENTS")? {
        config.max_order_cents = cents;
    }
    if let Some(attempts) = from_env(&env, "ORDERD_PAYMENT_MAX_ATTEMPTS")? {
        config.payment.max_attempts = attempts;
    }
    if let Some(ms) = from_env(&env, "ORDERD_PAYMENT_BACKOFF_MS")? {
        config.payment.backoff_ms = ms;
    }

    validate(config)
}

/// Reads and parses one environment variable, if it's set.
fn from_env<T>(
    env: &impl Fn(&str) -> Option<String>,
    var: &'static str,
) -> Result<Option<T>, ConfigError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    let Some(value) = env(var) else {
        return Ok(None);
    };
    match value.trim().parse() {
        Ok(parsed) => Ok(Some(parsed)),
        Err(err) => Err(ConfigError::Env {
            var,
            value,
            reason: err.to_string(),
        }),
    }
}

fn invalid(field: &'static str, reason: impl Into<String>) -> ConfigError {
    ConfigError::Invalid {
        field,
        reason: reason.into(),
    }
}

fn validate(config: Config) -> Result<Config, ConfigError> {
    if config.service_name.trim().is_empty() {
        return Err(invalid("service_name", "must not be empty"));
    }
    if config.max_order_cents == 0 {
        return Err(invalid("max_order_cents", "must be greater than 0"));
    }
    if !(1..=10).contains(&config.payment.max_attempts) {
        return Err(invalid(
            "payment.max_attempts",
            format!(
                "must be between 1 and 10, got {}",
                config.payment.max_attempts
            ),
        ));
    }
    if config.payment.backoff_ms > 10_000 {
        return Err(invalid(
            "payment.backoff_ms",
            format!("must be at most 10000, got {}", config.payment.backoff_ms),
        ));
    }
    Ok(config)
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
