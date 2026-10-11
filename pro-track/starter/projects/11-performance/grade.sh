#!/usr/bin/env bash
# Grades project 11: formatting, lints, unchanged tests and data, correct results, the
# benchmarks (including your own for `ranks`), and every function within its time
# budget in an optimized build.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"

provided_files_unchanged() {
  unchanged "$HERE/tests/correctness.rs" b46720aecc5ba09df1f1f86af909b3a4fdeb0648d646db93713c6ed386a50fdb &&
    unchanged "$HERE/tests/budget.rs" 733aeee425040fd1ce6045ec4236900d288c53d670b554689d019585eb15555e &&
    unchanged "$HERE/src/data.rs" 5faf0f28e7671641daeab7344aafa84cf26317c8937c68a47f2e24edc5cd06ab
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
