//! Configuration from environment variables, tested with a fake environment.
//! Provided: don't change this file (the grader checks that). Make it pass.

use std::net::SocketAddr;

use shortener::config::{Config, LogFormat};

fn config(vars: &[(&str, &str)]) -> Result<Config, String> {
    Config::from_env_with(|name| {
        vars.iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
    })
    .map_err(|err| err.to_string())
}

#[test]
fn defaults_work_on_a_laptop() {
    let config = config(&[]).unwrap();
    assert_eq!(config.addr, "0.0.0.0:3000".parse::<SocketAddr>().unwrap());
    assert_eq!(config.database_url, "sqlite:shortener.db");
    assert_eq!(config.public_url, "http://localhost:3000");
    assert_eq!(config.log_format, LogFormat::Text);
}

#[test]
fn every_setting_comes_from_the_environment() {
    let config = config(&[
        ("HOST", "127.0.0.1"),
        ("PORT", "8080"),
        ("DATABASE_URL", "sqlite:///data/links.db"),
        ("PUBLIC_URL", "https://sho.rt/"),
        ("LOG_FORMAT", "json"),
    ])
    .unwrap();
    assert_eq!(config.addr, "127.0.0.1:8080".parse::<SocketAddr>().unwrap());
    assert_eq!(config.database_url, "sqlite:///data/links.db");
    assert_eq!(
        config.public_url, "https://sho.rt",
        "the trailing slash is removed"
    );
    assert_eq!(config.log_format, LogFormat::Json);
}

#[test]
fn the_default_public_url_follows_the_port() {
    assert_eq!(
        config(&[("PORT", "8080")]).unwrap().public_url,
        "http://localhost:8080"
    );
}

#[test]
fn invalid_settings_name_the_variable() {
    let cases = [
        ("PORT", "http"),
        ("PORT", "70000"),
        ("HOST", "localhost:3000"),
        ("PUBLIC_URL", "sho.rt"),
        ("PUBLIC_URL", "ftp://sho.rt"),
        ("LOG_FORMAT", "xml"),
    ];
    for (name, value) in cases {
        let err =
            config(&[(name, value)]).expect_err(&format!("{name}={value} should be rejected"));
        assert!(
            err.contains(name) && err.contains(value),
            "unhelpful message: {err}"
        );
    }
}
