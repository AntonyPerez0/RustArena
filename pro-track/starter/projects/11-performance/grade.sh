#!/usr/bin/env bash
# Grades project 11: formatting, lints, unchanged tests and data, correct results, the
# benchmarks (including your own for `ranks`), and every function within its time
# budget in an optimized build.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"

provided_files_unchanged() {
  unchanged "$HERE/tests/correctness.rs" 085b1538cba45bccf5aee7fc7f0ea4b155fcc0d63730be0a82360deb6dcc469b &&
    unchanged "$HERE/tests/budget.rs" 5ca1f48603e368e82d5f237b6f1b4c83f0d17cbe60b753d41fc4580c9afc8cb3 &&
    unchanged "$HERE/src/data.rs" a1ac32d20bc3f4e986765f54738596cf45ea74c846fb37bf14eec349bfa8ce85
}

benches_compile() {
  (cd "$HERE" && cargo bench --quiet --no-run)
}

ranks_has_a_benchmark() {
  grep -qE '\branks\(' "$HERE"/benches/*.rs || {
    echo "Add a criterion benchmark that calls ranks(...) in benches/hotspots.rs"
    return 1
  }
}

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings (library, tests, benches, examples)" clippy_ok "$HERE"
check "the provided tests and src/data.rs are unchanged" provided_files_unchanged
check "results are correct (cargo test)" tests_ok "$HERE"
check "the benchmarks compile (cargo bench --no-run)" benches_compile
check "benches/hotspots.rs has a benchmark for ranks" ranks_has_a_benchmark
check "every function is within its time budget (release build)" \
  tests_ok "$HERE" --release --test budget -- --ignored --test-threads=1 --nocapture
finish
