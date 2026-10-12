# Helpers shared by every projects/*/grade.sh. Source it, don't run it.
# shellcheck shell=bash

ARENA_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARENA_FAILED=0
export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"

# check "description" command [args...]: runs a check and records the result.
check() {
  local what="$1"
  shift
  echo
  echo "::group::$what"
  if "$@"; then
    echo "::endgroup::"
    echo "PASS  $what"
  else
    echo "::endgroup::"
    echo "FAIL  $what"
    ARENA_FAILED=1
  fi
}

# fmt_ok <crate dir>: the code is formatted the way `cargo fmt` would format it.
fmt_ok() {
  (cd "$1" && cargo fmt --all -- --check) || { echo "Run: cargo fmt   (in $1)"; return 1; }
}

# clippy_ok <crate dir> [extra args]: no clippy warnings, in any target (tests and examples too).
clippy_ok() {
  local dir="$1"
  shift
  (cd "$dir" && cargo clippy --all-targets --quiet "$@" -- -D warnings)
}

# tests_ok <crate dir> [cargo test args]: every test passes.
tests_ok() {
  local dir="$1"
  shift
  (cd "$dir" && cargo test --quiet "$@")
}

# sha256_hex: the SHA-256 of standard input, as hex (sha256sum on Linux, shasum on macOS).
sha256_hex() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | cut -d" " -f1
  else
    shasum -a 256 | cut -d" " -f1
  fi
}

# unchanged <file> <sha256>: a file the grader relies on (usually a provided test) is untouched.
unchanged() {
  local got
  got=$(sha256_hex <"$1")
  [ "$got" = "$2" ] || { echo "$1 was changed. Restore it with: git checkout -- $1"; return 1; }
}

# finish: exit with the overall result. Call it last in grade.sh.
finish() {
  echo
  if [ "$ARENA_FAILED" -eq 0 ]; then
    echo "RESULT: PASSED"
  else
    echo "RESULT: NOT YET. Scroll up to the first FAIL line for details."
  fi
  exit "$ARENA_FAILED"
}
