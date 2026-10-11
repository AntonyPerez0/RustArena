//! Configuration from environment variables, the way containers and cloud platforms
//! pass it. Everything has a default that works on a laptop.

use std::net::{IpAddr, SocketAddr};

/// The service's settings, validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// `HOST` (default `0.0.0.0`) and `PORT` (default `3000`).
    pub addr: SocketAddr,
    /// `DATABASE_URL` (default `sqlite:shortener.db`).
    pub database_url: String,
    /// `PUBLIC_URL` (default `http://localhost:{PORT}`): the address users see, used to
    /// build short links. Stored without a trailing slash.
    pub public_url: String,
    /// `LOG_FORMAT`: `text` (default) or `json`.
    pub log_format: LogFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    Json,
}

/// A setting that isn't valid. The message names the variable and its value.
#[derive(Debug, thiserror::Error)]
#[error("{var}={value:?}: {reason}")]
pub struct ConfigError {
    pub var: &'static str,
    pub value: String,
    pub reason: String,
}

fn bad(var: &'static str, value: &str, reason: &str) -> ConfigError {
    ConfigError {
        var,
        value: value.to_string(),
        reason: reason.to_string(),
    }
}

impl Config {
    /// Reads the real environment.
    pub fn from_env() -> Result<Config, ConfigError> {
        Config::from_env_with(|name| std::env::var(name).ok())
    }

    /// Builds the configuration from `env`, a function that looks variables up. Tests pass
    /// a fake environment; `from_env` passes the real one.
    pub fn from_env_with(env: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
        let host: IpAddr = match env("HOST") {
            Some(value) => value
                .parse()
                .map_err(|_| bad("HOST", &value, "expected an IP address like 0.0.0.0"))?,
            None => IpAddr::from([0, 0, 0, 0]),
        };
        let port: u16 = match env("PORT") {
            Some(value) => value
                .parse()
                .map_err(|_| bad("PORT", &value, "expected a port number (0 to 65535)"))?,
            None => 3000,
        };
        let database_url = env("DATABASE_URL").unwrap_or_else(|| "sqlite:shortener.db".to_string());
        let public_url = match env("PUBLIC_URL") {
            Some(value) => {
                let parsed = url::Url::parse(&value)
                    .map_err(|_| bad("PUBLIC_URL", &value, "expected a URL like https://sho.rt"))?;
                if !matches!(parsed.scheme(), "http" | "https") {
                    return Err(bad(
                        "PUBLIC_URL",
                        &value,
                        "must start with http:// or https://",
                    ));
                }
                value.trim_end_matches('/').to_string()
            }
            None => format!("http://localhost:{port}"),
        };
        let log_format = match env("LOG_FORMAT").as_deref() {
            None | Some("text") => LogFormat::Text,
            Some("json") => LogFormat::Json,
            Some(other) => return Err(bad("LOG_FORMAT", other, "expected text or json")),
        };
        Ok(Config {
            addr: SocketAddr::new(host, port),
            database_url,
            public_url,
            log_format,
        })
    }
}
