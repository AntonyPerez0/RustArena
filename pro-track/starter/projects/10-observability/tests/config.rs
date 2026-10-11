//! Configuration loading, tested as a pure function: no files, no real environment.
//! Provided: don't change this file (the grader checks that). Make it pass.

use orderd::config::{Config, ConfigError, LogFormat, PaymentConfig, from_sources};

/// A fake environment holding exactly these variables.
fn env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name| {
        vars.iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
    }
}

const NO_ENV: &[(&str, &str)] = &[];

const FULL_FILE: &str = r#"
service_name = "orders-eu"
log_format = "json"
max_order_cents = 250000

[payment]
max_attempts = 5
backoff_ms = 20
"#;

#[test]
fn defaults_when_nothing_is_set() {
    let config = from_sources(None, env(NO_ENV)).unwrap();
    assert_eq!(config, Config::default());
    assert_eq!(config.service_name, "orderd");
    assert_eq!(config.log_format, LogFormat::Text);
    assert_eq!(config.max_order_cents, 100_000);
    assert_eq!(
        config.payment,
        PaymentConfig {
            max_attempts: 3,
            backoff_ms: 100
        }
    );
}

#[test]
fn the_file_sets_every_value() {
    let config = from_sources(Some(FULL_FILE), env(NO_ENV)).unwrap();
    assert_eq!(
        config,
        Config {
            service_name: "orders-eu".to_string(),
            log_format: LogFormat::Json,
            max_order_cents: 250_000,
            payment: PaymentConfig {
                max_attempts: 5,
                backoff_ms: 20
            },
        }
    );
}

#[test]
fn missing_keys_keep_their_defaults() {
    let config = from_sources(Some("[payment]\nmax_attempts = 7\n"), env(NO_ENV)).unwrap();
    assert_eq!(config.payment.max_attempts, 7);
    assert_eq!(config.payment.backoff_ms, 100);
    assert_eq!(config.service_name, "orderd");
    assert_eq!(
        from_sources(Some(""), env(NO_ENV)).unwrap(),
        Config::default()
    );
}

#[test]
fn the_environment_overrides_the_file() {
    let vars = [
        ("ORDERD_SERVICE_NAME", "orders-us"),
        ("ORDERD_LOG_FORMAT", "text"),
        ("ORDERD_MAX_ORDER_CENTS", "999"),
        ("ORDERD_PAYMENT_MAX_ATTEMPTS", "1"),
        ("ORDERD_PAYMENT_BACKOFF_MS", "0"),
        ("UNRELATED", "ignored"),
    ];
    let config = from_sources(Some(FULL_FILE), env(&vars)).unwrap();
    assert_eq!(
        config,
        Config {
            service_name: "orders-us".to_string(),
            log_format: LogFormat::Text,
            max_order_cents: 999,
            payment: PaymentConfig {
                max_attempts: 1,
                backoff_ms: 0
            },
        }
    );
}

#[test]
fn the_environment_works_without_a_file() {
    let config = from_sources(None, env(&[("ORDERD_LOG_FORMAT", "json")])).unwrap();
    assert_eq!(config.log_format, LogFormat::Json);
}

#[test]
fn a_typo_in_the_file_is_an_error_not_ignored() {
    let err = from_sources(Some("[payment]\nmax_atempts = 5\n"), env(NO_ENV)).unwrap_err();
    assert!(matches!(err, ConfigError::Parse(_)), "got {err:?}");
    assert!(
        err.to_string().contains("max_atempts"),
        "the message should name the key: {err}"
    );
}

#[test]
fn wrong_types_in_the_file_are_errors() {
    for text in [
        "max_order_cents = \"lots\"",
        "log_format = \"xml\"",
        "service_name = 42",
        "this is not toml",
    ] {
        let err = from_sources(Some(text), env(NO_ENV)).unwrap_err();
        assert!(
            matches!(err, ConfigError::Parse(_)),
            "{text:?}: got {err:?}"
        );
    }
}

#[test]
fn unparsable_environment_values_name_the_variable() {
    let cases = [
        ("ORDERD_PAYMENT_MAX_ATTEMPTS", "three"),
        ("ORDERD_MAX_ORDER_CENTS", "-5"),
        ("ORDERD_LOG_FORMAT", "xml"),
        ("ORDERD_PAYMENT_BACKOFF_MS", "1.5"),
    ];
    for (name, value) in cases {
        let vars = [(name, value)];
        let err = from_sources(None, env(&vars)).unwrap_err();
        match &err {
            ConfigError::Env {
                var, value: got, ..
            } => {
                assert_eq!(*var, name);
                assert_eq!(got, value);
            }
            other => panic!("{name}={value}: expected ConfigError::Env, got {other:?}"),
        }
        let message = err.to_string();
        assert!(
            message.contains(name) && message.contains(value),
            "unhelpful message: {message}"
        );
    }
}

#[test]
fn values_out_of_range_are_rejected_with_the_setting_name() {
    let cases: [(&str, &str); 6] = [
        ("max_order_cents = 0", "max_order_cents"),
        ("[payment]\nmax_attempts = 0", "payment.max_attempts"),
        ("[payment]\nmax_attempts = 11", "payment.max_attempts"),
        ("[payment]\nbackoff_ms = 10001", "payment.backoff_ms"),
        ("service_name = \"\"", "service_name"),
        ("service_name = \"   \"", "service_name"),
    ];
    for (text, setting) in cases {
        let err = from_sources(Some(text), env(NO_ENV)).unwrap_err();
        match &err {
            ConfigError::Invalid { field, .. } => assert_eq!(*field, setting, "for {text:?}"),
            other => panic!("{text:?}: expected ConfigError::Invalid, got {other:?}"),
        }
        assert!(
            err.to_string().contains(setting),
            "unhelpful message: {err}"
        );
    }
}

#[test]
fn the_environment_is_validated_too() {
    let err = from_sources(None, env(&[("ORDERD_PAYMENT_MAX_ATTEMPTS", "0")])).unwrap_err();
    assert!(
        matches!(
            err,
            ConfigError::Invalid {
                field: "payment.max_attempts",
                ..
            }
        ),
        "got {err:?}"
    );
    // The limits are inclusive.
    let vars = [
        ("ORDERD_PAYMENT_MAX_ATTEMPTS", "10"),
        ("ORDERD_PAYMENT_BACKOFF_MS", "10000"),
    ];
    let config = from_sources(None, env(&vars)).unwrap();
    assert_eq!(
        config.payment,
        PaymentConfig {
            max_attempts: 10,
            backoff_ms: 10_000
        }
    );
}
