#!/usr/bin/env bash
# Grades project 07: the async chat server passes the provided integration tests
# (three runs in a row, to catch timing bugs) and uses Tokio the intended way.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"
SERVER="$HERE/src/server.rs"

# Runs a command with a time limit when the `timeout` tool exists (Linux, CI),
# so a hung test fails instead of freezing the job.
limited() {
  if command -v timeout >/dev/null 2>&1; then timeout "$@"; else shift; "$@"; fi
}

tests_pass_three_times() {
  local run
  (cd "$HERE" && cargo test --quiet --no-run) || return 1
  for run in 1 2 3; do
    echo "--- run $run of 3"
    (cd "$HERE" && limited 300 cargo test --quiet) || return 1
  done
}

# server_uses <what> <regex>: the regex appears in src/server.rs (comments ignored).
server_uses() {
  sed 's://.*$::' "$SERVER" | grep -Eq "$2" || { echo "src/server.rs should use $1"; return 1; }
}

no_blocking_calls() {
  if sed 's://.*$::' "$HERE"/src/*.rs | grep -En 'thread::sleep|std::io::stdin|block_on'; then
    echo "Blocking calls stop every task on that worker thread: use tokio::time::sleep and friends"
    return 1
  fi
}

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings" clippy_ok "$HERE"
check "provided tests are unchanged" unchanged "$HERE/tests/chat.rs" b2b7da4e7856839148784bdeb9c2c65952ce13c9bb6572879d19c234146854f9
check "all tests pass, 3 runs in a row" tests_pass_three_times
check "a task per connection (tokio::spawn or JoinSet)" server_uses "tokio::spawn or JoinSet::spawn" 'spawn\('
check "connections use select!" server_uses "tokio::select!" 'select!'
check "events go through a broadcast channel" server_uses "tokio::sync::broadcast" 'broadcast'
check "no blocking calls in async code" no_blocking_calls
finish
