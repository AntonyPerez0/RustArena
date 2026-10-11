#!/usr/bin/env bash
# Grades project 03 with mutation testing: your tests must pass on the real
# library and FAIL on each of 8 copies of it with one planted bug ("mutants",
# the patches in mutants/).
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"
# Property tests draw random inputs. A fixed seed makes every grading run identical.
export PROPTEST_RNG_SEED="${PROPTEST_RNG_SEED:-20261011}"
# Each mutant is built in a scratch copy of this project, inside target/ (ignored by Git).
MUT="$ARENA_ROOT/target/mutants/03-testing"
REAL_OK=0

frozen_ok() {
  local h
  h=$(cd "$HERE" && cat src/money.rs src/account.rs mutants/*.patch | sha256sum | cut -d" " -f1)
  [ "$h" = "7e22bad0788360a585769e3efce6b64b944ed3744c407722c18f0718af8b2a8e" ] || {
    echo "src/money.rs, src/account.rs or mutants/ changed. Restore them with:"
    echo "  git checkout -- projects/03-testing/src/money.rs projects/03-testing/src/account.rs projects/03-testing/mutants"
    return 1
  }
}

real_tests_ok() {
  tests_ok "$HERE" && REAL_OK=1
}

# count <cargo test args>: how many tests those targets contain.
count() {
  (cd "$HERE" && cargo test --quiet "$@" -- --list 2>/dev/null) | grep -c ': test$'
}

enough_tests() {
  local unit integration doc total ok=0
  unit=$(count --lib)
  integration=$(count --test '*')
  doc=$( (cd "$HERE" && cargo test --quiet --doc -- --list 2>/dev/null) | grep -c '^src/lib\.rs .*: test$')
  total=$((unit + integration + doc))
  echo "unit tests: $unit, integration tests: $integration, doc tests in src/lib.rs: $doc, total: $total"
  [ "$unit" -ge 3 ] || { echo "write at least 3 unit tests (src/money/tests.rs, src/account/tests.rs)"; ok=1; }
  [ "$integration" -ge 3 ] || { echo "write at least 3 integration tests (files in tests/)"; ok=1; }
  [ "$doc" -ge 1 ] || { echo "add at least 1 doc test: an example in the documentation in src/lib.rs"; ok=1; }
  [ "$total" -ge 15 ] || { echo "write at least 15 tests in total"; ok=1; }
  return $ok
}

proptest_ok() {
  local files
  files=$(cd "$HERE" && grep -rlE 'proptest!' src tests 2>/dev/null)
  [ -n "$files" ] || { echo "no proptest! { ... } block in src/ or tests/"; return 1; }
  echo "property tests in: $files"
}

mutant_killed() {
  local patch="$HERE/mutants/$1.patch"
  if [ "$REAL_OK" -ne 1 ]; then
    echo "skipped: first make every test pass on the real library"
    return 1
  fi
  rm -rf "$MUT" && mkdir -p "$MUT" && cp -R "$HERE/." "$MUT/" || return 1
  # GIT_CEILING_DIRECTORIES makes git apply patch this folder like patch(1) would,
  # instead of treating paths as relative to your repository's root.
  (cd "$MUT" && GIT_CEILING_DIRECTORIES="$(dirname "$MUT")" git apply "$patch") ||
    { echo "could not plant the bug: are src/money.rs and src/account.rs unchanged?"; return 1; }
  if ! (cd "$MUT" && cargo test --quiet --no-run >/dev/null 2>&1); then
    echo "your tests don't compile against this mutant:"
    (cd "$MUT" && cargo test --quiet --no-run 2>&1 | tail -n 30)
    return 1
  fi
  if (cd "$MUT" && cargo test --quiet >/dev/null 2>&1); then
    echo "every test still PASSES with this bug planted, so no test checks this rule."
    echo "The bug (mutants/$1.patch):"
    grep -E '^[-+][^-+]' "$patch"
    return 1
  fi
  echo "killed: at least one test failed"
}

check "the library and the mutants are unchanged" frozen_ok
check "the code is formatted (cargo fmt --check)" fmt_ok "$HERE"
check "no clippy warnings (cargo clippy -- -D warnings)" clippy_ok "$HERE"
check "every test passes on the real library" real_tests_ok
check "at least 15 tests: 3+ unit, 3+ integration, 1+ doc test" enough_tests
check "property tests with proptest" proptest_ok
check "mutant 1 killed (parse accepts one decimal: 1.5 becomes 1.05)" mutant_killed 1-parse-accepts-one-decimal
check "mutant 2 killed (display drops the minus sign of -0.05)" mutant_killed 2-display-drops-the-minus-sign
check "mutant 3 killed (split loses cents of negative amounts)" mutant_killed 3-split-loses-cents-of-negative-amounts
check "mutant 4 killed (percentage truncates instead of rounding)" mutant_killed 4-percentage-truncates
check "mutant 5 killed (withdraw refuses the whole balance)" mutant_killed 5-withdraw-refuses-the-whole-balance
check "mutant 6 killed (deposit accepts zero)" mutant_killed 6-deposit-accepts-zero
check "mutant 7 killed (a refused withdrawal is recorded in the history)" mutant_killed 7-refused-withdrawal-is-recorded
check "mutant 8 killed (recent() is oldest first)" mutant_killed 8-recent-is-oldest-first
rm -rf "$MUT"
finish
