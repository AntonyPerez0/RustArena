#!/usr/bin/env bash
# Grades project 10: formatting, lints, the provided tests (unchanged), no println!
# logging left, and the real program's logs following RUST_LOG and the configuration.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"

SUMMARY="processed 6 orders: 3 accepted, 2 rejected, 1 failed (revenue 133.93)"

tests_unchanged() {
  unchanged "$HERE/tests/config.rs" b6fadb8ace26772475ee94d9dd01cde9fdd91475906a4d210dc542257c07c08d &&
    unchanged "$HERE/tests/telemetry.rs" ef20c72c8e75e6216b5ce953e30a56337ab48f2e53fba164eb6f559d66d8610e &&
    unchanged "$HERE/tests/cli.rs" 2191ff62b7d676b1d7b5b4be1b2bce9380da358efbeffae2f9ea873ffa895a61
}

no_print_logging() {
  if grep -nE '\b(println|eprintln|print|eprint|dbg)!' "$HERE/src/orders.rs"; then
    echo "Replace these with tracing events (info!, warn!, debug!, error!)"
    return 1
  fi
}

# Runs `cargo run` with only the given settings (the subshell keeps them out of other checks).
orderd() (
  unset ORDERD_CONFIG ORDERD_SERVICE_NAME ORDERD_LOG_FORMAT ORDERD_MAX_ORDER_CENTS \
    ORDERD_PAYMENT_MAX_ATTEMPTS ORDERD_PAYMENT_BACKOFF_MS RUST_LOG NO_COLOR
  export ORDERD_PAYMENT_BACKOFF_MS=0
  for setting in "$@"; do export "${setting?}"; done
  cd "$HERE" && cargo run --quiet
)

json_warnings_only() {
  local out err
  err="$(mktemp)"
  out="$(orderd RUST_LOG=warn ORDERD_LOG_FORMAT=json 2>"$err")" || { cat "$err"; return 1; }
  [ "$out" = "$SUMMARY" ] || { echo "stdout was: $out"; echo "expected:   $SUMMARY"; return 1; }
  if grep -vE '^\{.*\}$' "$err"; then echo "^ these stderr lines aren't JSON objects"; return 1; fi
  if grep -vE '"level":"(WARN|ERROR)"' "$err"; then echo "^ RUST_LOG=warn should hide these"; return 1; fi
  grep -q '"message":"payment failed"' "$err" || { echo "no \"payment failed\" error in:"; cat "$err"; return 1; }
  rm -f "$err"
}

text_debug_with_config_file() {
  local out err
  err="$(mktemp)"
  out="$(orderd RUST_LOG=debug ORDERD_CONFIG=config/orderd.toml 2>"$err")" || { cat "$err"; return 1; }
  [ "$out" = "$SUMMARY" ] || { echo "stdout was: $out"; echo "expected:   $SUMMARY"; return 1; }
  grep -E 'DEBUG.*order_id=1.*priced line' "$err" >/dev/null || {
    echo "expected a DEBUG \"priced line\" event inside the order 1 span, got:"
    cat "$err"
    return 1
  }
  rm -f "$err"
}

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings" clippy_ok "$HERE"
check "the provided tests are unchanged" tests_unchanged
check "src/orders.rs logs with tracing, not println!" no_print_logging
check "all tests pass" tests_ok "$HERE"
check "RUST_LOG=warn with JSON logs: only warnings and errors, one JSON object per line" json_warnings_only
check "RUST_LOG=debug with config/orderd.toml: readable debug events inside order spans" text_debug_with_config_file
finish
