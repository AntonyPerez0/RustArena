//! Configuration from environment variables, the way containers and cloud platforms
//! pass it. Everything has a default that works on a laptop.

use std::net::SocketAddr;

/// The service's settings, validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// `HOST` (default `0.0.0.0`) and `PORT` (default `3000`).
    pub addr: SocketAddr,
    /// `DATABASE_URL` (default `sqlite:shortener.db`).
    pub database_url: String,
    /// `PUBLIC_URL` (default `http://localhost:{PORT}`): the address users see, used to
    /// build short links. Must be an http(s) URL. Stored without a trailing slash.
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

impl Config {
    /// Reads the real environment.
    pub fn from_env() -> Result<Config, ConfigError> {
        Config::from_env_with(|name| std::env::var(name).ok())
    }

    /// Builds the configuration from `env`, a function that looks variables up. Tests pass
    /// a fake environment; `from_env` passes the real one.
    pub fn from_env_with(env: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
        // TODO: read and validate HOST, PORT, DATABASE_URL, PUBLIC_URL and LOG_FORMAT.
        let _ = env;
        todo!("Config::from_env_with")
    }
}
