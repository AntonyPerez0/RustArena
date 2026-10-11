#!/usr/bin/env bash
# Grades project 04: formatting, clippy, the provided end-to-end tests, the
# clear subcommand (run for real), and your own tests.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"
TASKS=""

# taskr ARGS...: runs your taskr on the scratch tasks file $TASKS.
taskr() {
  (cd "$HERE" && cargo run --quiet -- --file "$TASKS" "$@")
}

clear_ok() {
  local tmp out left t
  tmp=$(mktemp -d)
  TASKS="$tmp/tasks.json"
  for t in one two three; do
    taskr add "$t" >/dev/null || { echo "taskr add $t failed"; rm -rf "$tmp"; return 1; }
  done
  { taskr done 1 && taskr done 3; } >/dev/null || { echo "taskr done failed"; rm -rf "$tmp"; return 1; }
  out=$(taskr clear) || { echo "taskr clear failed: is there a Clear subcommand?"; rm -rf "$tmp"; return 1; }
  left=$(taskr list --all --json)
  rm -rf "$tmp"
  echo "add one, two, three; done 1; done 3; clear  ->  $out"
  [ "$out" = "Removed 2 done tasks" ] || { echo "clear should print: Removed 2 done tasks"; return 1; }
  if ! echo "$left" | grep -q '"two"' || echo "$left" | grep -qE '"(one|three)"'; then
    echo "after clear, only task 2 should be left. list --all --json printed:"
    echo "$left"
    return 1
  fi
}

own_tests_ok() {
  local all provided n
  all=$(cd "$HERE" && cargo test --quiet --test '*' -- --list 2>/dev/null | grep -c ': test$')
  provided=$(cd "$HERE" && cargo test --quiet --test cli -- --list 2>/dev/null | grep -c ': test$')
  n=$((all - provided))
  echo "your own tests in tests/ (besides cli.rs): $n"
  [ "$n" -ge 2 ] || { echo "write at least 2 tests for taskr clear in a new file, such as tests/clear.rs"; return 1; }
}

check "the provided tests (tests/cli.rs) are unchanged" \
  unchanged "$HERE/tests/cli.rs" fc220fc1de211c546f73f1a92b39050d09f36230d9cb8887f1da579a31c75df4
check "the code is formatted (cargo fmt --check)" fmt_ok "$HERE"
check "no clippy warnings (cargo clippy -- -D warnings)" clippy_ok "$HERE"
check "every test passes (cargo test)" tests_ok "$HERE"
check "taskr clear deletes the done tasks" clear_ok
check "at least 2 tests of your own in tests/" own_tests_ok
finish
