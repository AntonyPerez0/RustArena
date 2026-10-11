//! Runs the real `orderd` program the way an operator would, and checks its output.
//! Provided: don't change this file (the grader checks that). Make it pass.

use std::process::{Command, Output};

use serde_json::Value;

const SUMMARY: &str = "processed 6 orders: 3 accepted, 2 rejected, 1 failed (revenue 133.93)\n";

/// Runs orderd with exactly these settings (anything set in your shell is removed first).
fn orderd(vars: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_orderd"));
    for (name, _) in std::env::vars() {
        if name.starts_with("ORDERD_") || name == "RUST_LOG" || name == "NO_COLOR" {
            cmd.env_remove(name);
        }
    }
    cmd.env("ORDERD_PAYMENT_BACKOFF_MS", "0");
    cmd.envs(vars.iter().copied());
    cmd.output().expect("run orderd")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Every stderr line, parsed as JSON. Fails if any line isn't a JSON object.
fn json_lines(out: &Output) -> Vec<Value> {
    stderr(out)
        .lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("not a JSON log line ({e}): {line}"));
            assert!(value.is_object(), "not a JSON object: {line}");
            value
        })
        .collect()
}

fn levels(lines: &[Value]) -> Vec<&str> {
    lines
        .iter()
        .map(|l| l["level"].as_str().unwrap_or("?"))
        .collect()
}

fn messages(lines: &[Value]) -> Vec<&str> {
    lines
        .iter()
        .filter_map(|l| l["fields"]["message"].as_str())
        .collect()
}

#[test]
fn stdout_holds_only_the_summary() {
    let out = orderd(&[]);
    assert!(out.status.success(), "orderd failed:\n{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        SUMMARY,
        "logs belong on stderr; stdout is the program's output"
    );
    assert!(
        stderr(&out).contains("order accepted"),
        "expected logs on stderr"
    );
}

#[test]
fn json_logs_are_one_object_per_line() {
    let out = orderd(&[("ORDERD_LOG_FORMAT", "json"), ("RUST_LOG", "info")]);
    assert!(out.status.success(), "orderd failed:\n{}", stderr(&out));
    assert_eq!(stdout(&out), SUMMARY);
    let lines = json_lines(&out);
    let accepted: Vec<&Value> = lines
        .iter()
        .filter(|l| l["fields"]["message"] == "order accepted")
        .collect();
    assert_eq!(accepted.len(), 3, "expected 3 \"order accepted\" lines");
    let first = accepted[0];
    assert_eq!(first["level"], "INFO");
    assert_eq!(first["fields"]["total_cents"], 8997);
    // The current span and the list of enclosing spans are part of every line.
    assert_eq!(first["span"]["name"], "process_order");
    assert_eq!(first["span"]["order_id"], 1);
    let spans: Vec<&str> = first["spans"]
        .as_array()
        .expect("a \"spans\" list")
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert_eq!(spans, ["process_batch", "process_order"]);
}

#[test]
fn rust_log_sets_the_level() {
    let out = orderd(&[("ORDERD_LOG_FORMAT", "json"), ("RUST_LOG", "warn")]);
    let lines = json_lines(&out);
    let warn = levels(&lines);
    assert!(
        warn.iter().all(|l| *l == "WARN" || *l == "ERROR"),
        "RUST_LOG=warn let through {warn:?}"
    );
    assert!(
        warn.contains(&"WARN") && warn.contains(&"ERROR"),
        "got {warn:?}"
    );

    let out = orderd(&[("ORDERD_LOG_FORMAT", "json"), ("RUST_LOG", "debug")]);
    let lines = json_lines(&out);
    assert!(
        messages(&lines).contains(&"priced line"),
        "RUST_LOG=debug should show debug events"
    );

    // Without RUST_LOG the level is info: no debug lines.
    let out = orderd(&[("ORDERD_LOG_FORMAT", "json")]);
    let lines = json_lines(&out);
    assert!(levels(&lines).contains(&"INFO"));
    assert!(
        !levels(&lines).contains(&"DEBUG"),
        "the default level should be info"
    );
}

#[test]
fn rust_log_can_target_one_module() {
    let out = orderd(&[
        ("ORDERD_LOG_FORMAT", "json"),
        ("RUST_LOG", "error,orderd::orders=debug"),
    ]);
    let lines = json_lines(&out);
    let messages = messages(&lines);
    assert!(
        messages.contains(&"priced line"),
        "orderd::orders=debug should enable debug there"
    );
    assert!(
        !messages.contains(&"starting"),
        "main's info event should be filtered out by `error`"
    );
}

#[test]
fn text_logs_are_for_humans() {
    let out = orderd(&[("ORDERD_LOG_FORMAT", "text"), ("RUST_LOG", "info")]);
    assert!(out.status.success(), "orderd failed:\n{}", stderr(&out));
    let err = stderr(&out);
    let line = err
        .lines()
        .find(|l| l.contains("order accepted"))
        .expect("an \"order accepted\" line");
    assert!(
        !line.starts_with('{'),
        "text format shouldn't be JSON: {line}"
    );
    assert!(line.contains("INFO"), "{line}");
    assert!(
        line.contains("order_id=1"),
        "the span's fields should be shown: {line}"
    );
    assert!(line.contains("total_cents=8997"), "{line}");
    assert!(
        !err.contains('\u{1b}'),
        "no color codes when stderr isn't a terminal: {line}"
    );
}

#[test]
fn the_config_file_is_used_and_the_environment_wins() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("orderd.toml");
    std::fs::write(
        &path,
        "service_name = \"orders-test\"\nlog_format = \"json\"\nmax_order_cents = 5000\n",
    )
    .unwrap();
    let path = path.to_str().unwrap();

    let out = orderd(&[("ORDERD_CONFIG", path)]);
    assert!(out.status.success(), "orderd failed:\n{}", stderr(&out));
    // With a 50.00 limit, order 1 (89.97) is rejected too.
    assert_eq!(
        stdout(&out),
        "processed 6 orders: 2 accepted, 3 rejected, 1 failed (revenue 43.96)\n"
    );
    let lines = json_lines(&out);
    assert!(lines.iter().any(|l| l["span"]["service"] == "orders-test"));

    let out = orderd(&[
        ("ORDERD_CONFIG", path),
        ("ORDERD_MAX_ORDER_CENTS", "100000"),
    ]);
    assert_eq!(stdout(&out), SUMMARY);
}

#[test]
fn bad_configuration_stops_the_program_with_code_2() {
    let out = orderd(&[("ORDERD_PAYMENT_MAX_ATTEMPTS", "0")]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("payment.max_attempts"),
        "say what's wrong:\n{}",
        stderr(&out)
    );
    assert_eq!(stdout(&out), "");

    let out = orderd(&[("ORDERD_CONFIG", "/no/such/orderd.toml")]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("/no/such/orderd.toml"),
        "name the file:\n{}",
        stderr(&out)
    );
}

#[test]
fn fewer_attempts_means_flaky_payments_fail() {
    let out = orderd(&[("ORDERD_PAYMENT_MAX_ATTEMPTS", "2")]);
    assert_eq!(
        stdout(&out),
        "processed 6 orders: 2 accepted, 2 rejected, 2 failed (revenue 113.94)\n"
    );
}
