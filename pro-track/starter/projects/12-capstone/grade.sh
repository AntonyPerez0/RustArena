#!/usr/bin/env bash
# Grades the capstone like a release: formatting, lints, the provided tests (unchanged),
# the real binary serving requests and shutting down cleanly on SIGTERM, a multi-stage
# Dockerfile that runs as a non-root user, and a CI workflow.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"

tests_unchanged() {
  unchanged "$HERE/tests/api.rs" 8b9cf2e877d3dbb954a331d794f8eb7dfe525a45f6e7ee45144fefeef031e65d &&
    unchanged "$HERE/tests/config.rs" aac8d2b7ac295589d438981be65d2e9745c15a4e9983e0b658d868507a6cc12f
}

# Starts the real binary on a free-ish port with a database file in a temporary folder,
# talks to it with curl, then sends SIGTERM (what `docker stop` does) and expects a clean
# exit. The ( ) body runs in a subshell, so its variables and trap stay here.
service_runs() (
  local tmp target bin port base pid reply code
  tmp="$(mktemp -d)"
  (cd "$HERE" && cargo build --quiet) || return 1
  target="$(cd "$HERE" && cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
  bin="$target/debug/shortener"
  port=$((20000 + RANDOM % 20000))
  base="http://127.0.0.1:$port"
  HOST=127.0.0.1 PORT=$port DATABASE_URL="sqlite://$tmp/links.db" PUBLIC_URL="http://short.test" \
    RUST_LOG=info LOG_FORMAT=text "$bin" >"$tmp/log" 2>&1 &
  pid=$!
  trap 'kill -9 $pid 2>/dev/null; rm -rf "$tmp"' EXIT
  fail() {
    echo "$1"
    echo "--- the service's output:"
    cat "$tmp/log"
    return 1
  }

  for _ in $(seq 1 100); do
    curl -fs "$base/health" >/dev/null 2>&1 && break
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.1
  done
  reply="$(curl -fsS "$base/health" 2>&1)" || { fail "GET /health failed: $reply"; return 1; }
  echo "GET /health -> $reply"
  echo "$reply" | grep -qE '"status" *: *"ok"' || { fail 'expected {"status":"ok"}'; return 1; }

  code="$(curl -sS -o "$tmp/created" -w '%{http_code}' -H 'Content-Type: application/json' \
    -d '{"url":"https://www.rust-lang.org/learn","code":"grader"}' "$base/api/links")"
  echo "POST /api/links -> $code $(cat "$tmp/created")"
  [ "$code" = 201 ] || { fail "expected 201 Created"; return 1; }
  grep -q '"short_url" *: *"http://short.test/grader"' "$tmp/created" ||
    { fail 'expected "short_url":"http://short.test/grader" (built from PUBLIC_URL)'; return 1; }

  reply="$(curl -sS -o /dev/null -w '%{http_code} %{redirect_url}' "$base/grader")"
  echo "GET /grader -> $reply"
  [ "$reply" = "307 https://www.rust-lang.org/learn" ] || { fail "expected a 307 redirect to the URL"; return 1; }

  code="$(curl -sS -o /dev/null -w '%{http_code}' "$base/no-such-code")"
  echo "GET /no-such-code -> $code"
  [ "$code" = 404 ] || { fail "expected 404"; return 1; }

  kill -TERM "$pid"
  for _ in $(seq 1 100); do
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.1
  done
  if kill -0 "$pid" 2>/dev/null; then
    fail "still running 10 seconds after SIGTERM: handle it in shutdown_signal()"
    return 1
  fi
  wait "$pid"
  code=$?
  echo "SIGTERM -> exit code $code"
  [ "$code" = 0 ] || { fail "expected exit code 0 after SIGTERM (a graceful shutdown)"; return 1; }

  if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "The sqlite3 shell isn't installed. Run: bash tools/setup.sh"
    return 1
  fi
  reply="$(sqlite3 "$tmp/links.db" "SELECT url || ' ' || hits FROM links WHERE code = 'grader'")"
  echo "in $tmp/links.db: $reply"
  [ "$reply" = "https://www.rust-lang.org/learn 1" ] ||
    { fail "expected the link, with 1 hit, in the database file"; return 1; }
)

# Lines of the Dockerfile from the last FROM on: the final (runtime) stage.
final_stage() {
  awk 'toupper($1) == "FROM" { stage = "" } { stage = stage $0 "\n" } END { printf "%s", stage }' "$HERE/Dockerfile"
}

dockerfile_multi_stage() {
  local f="$HERE/Dockerfile" froms
  [ -f "$f" ] || { echo "Write projects/12-capstone/Dockerfile"; return 1; }
  froms=$(grep -ciE '^[[:space:]]*FROM[[:space:]]' "$f")
  [ "$froms" -ge 2 ] || { echo "A multi-stage build has at least two FROM lines (found $froms)"; return 1; }
  grep -qiE '^[[:space:]]*FROM[[:space:]]+[^[:space:]]+[[:space:]]+AS[[:space:]]+[^[:space:]]+' "$f" ||
    { echo "Name the build stage: FROM rust:1-bookworm AS builder"; return 1; }
  grep -qE 'cargo[[:space:]]+build.*--release' "$f" || { echo "Build with cargo build --release"; return 1; }
  grep -qiE '^[[:space:]]*COPY[[:space:]]+--from=' "$f" ||
    { echo "Copy the binary out of the build stage with COPY --from=..."; return 1; }
  final_stage | grep -qiE '^[[:space:]]*(CMD|ENTRYPOINT)[[:space:]]' || { echo "The final stage needs a CMD or ENTRYPOINT"; return 1; }
}

dockerfile_non_root() {
  [ -f "$HERE/Dockerfile" ] || { echo "Write projects/12-capstone/Dockerfile"; return 1; }
  local user
  user="$(final_stage | grep -iE '^[[:space:]]*USER[[:space:]]' | tail -n 1 | awk '{print $2}')"
  case "$user" in
    "") echo "Add a USER line to the final stage, so the service doesn't run as root"; return 1 ;;
    root | 0 | 0:*) echo "USER $user is root"; return 1 ;;
  esac
}

dockerignore_ok() {
  [ -f "$HERE/.dockerignore" ] && grep -qE '^/?target/?$' "$HERE/.dockerignore" ||
    { echo "Add a .dockerignore that excludes target"; return 1; }
}

ci_workflow_ok() {
  local f="$HERE/ci.yml" missing=""
  [ -f "$f" ] || { echo "Write projects/12-capstone/ci.yml"; return 1; }
  for needed in '^on:' '^jobs:' 'cargo fmt' 'cargo clippy' 'cargo test' 'docker build'; do
    grep -qE "$needed" "$f" || missing+=" '$needed'"
  done
  [ -z "$missing" ] || { echo "ci.yml is missing:$missing"; return 1; }
}

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings" clippy_ok "$HERE"
check "the provided tests are unchanged" tests_unchanged
check "all tests pass" tests_ok "$HERE"
check "the service runs: health, create, redirect, 404, and a clean exit on SIGTERM" service_runs
check "Dockerfile: a multi-stage build" dockerfile_multi_stage
check "Dockerfile: the image runs as a non-root user" dockerfile_non_root
check ".dockerignore keeps target/ out of the build" dockerignore_ok
check "ci.yml: formatting, lints, tests and a Docker build" ci_workflow_ok
finish
