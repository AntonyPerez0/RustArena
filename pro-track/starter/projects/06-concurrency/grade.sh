#!/usr/bin/env bash
# Grades project 06: three parallel versions of the log indexer give the same
# answer as the serial one, use the intended tools, and are not slower.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"
SRC="$HERE/src"

provided_files_unchanged() {
  unchanged "$HERE/tests/parallel.rs" SHA_TESTS &&
    unchanged "$SRC/main.rs" SHA_MAIN &&
    unchanged "$SRC/lib.rs" SHA_LIB
}

# uses <file> <what> <regex>...: every regex must appear in the file.
uses() {
  local file="$1" what="$2"
  shift 2
  for pattern in "$@"; do
    grep -Eq "$pattern" "$SRC/$file" || { echo "src/$file should use $what (looked for: $pattern)"; return 1; }
  done
  if grep -q 'index_serial' "$SRC/$file"; then
    echo "src/$file calls index_serial: do the work in parallel instead"
    return 1
  fi
}

# Runs the benchmark (release build) and compares each parallel version with the
# serial one. Shared CI machines are noisy, so the bar is low: a parallel version
# may be up to 25% slower than serial (it should be 2-4 times faster), and a
# failed measurement is repeated once.
not_slower() {
  local attempt out result
  (cd "$HERE" && cargo build --release --quiet) || return 1
  for attempt in 1 2; do
    out=$(cd "$HERE" && cargo run --release --quiet -- --rounds 5 2>&1) || {
      echo "$out"
      echo "The benchmark failed. Run it yourself: cargo run --release"
      return 1
    }
    echo "$out"
    result=$(grep '^RESULT ' <<<"$out")
    if awk -v line="$result" '
      BEGIN {
        n = split(line, fields, " ")
        for (i = 2; i <= n; i++) { split(fields[i], kv, "="); v[kv[1]] = kv[2] + 0 }
        if (v["cpus"] < 2) { print "Only 1 CPU: the timing comparison is skipped."; exit 0 }
        ok = 1
        split("shared channels rayon", names, " ")
        for (i = 1; i <= 3; i++) {
          t = v[names[i] "_ms"]
          printf "%-9s %.2fx the speed of serial\n", names[i], v["serial_ms"] / t
          if (t > v["serial_ms"] * 1.25) {
            printf "  too slow: %.1f ms vs %.1f ms serial. Look for a lock held too long or too often.\n", t, v["serial_ms"]
            ok = 0
          }
        }
        exit !ok
      }'; then
      return 0
    fi
    [ "$attempt" -eq 1 ] && echo "Measuring once more, in case the machine was busy..."
  done
  return 1
}

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings" clippy_ok "$HERE"
check "provided tests, lib.rs and benchmark are unchanged" provided_files_unchanged
check "all tests pass" tests_ok "$HERE"
check "part 1 uses std::thread::spawn with an Arc<Mutex>" uses shared.rs "threads, Arc and Mutex" 'thread::spawn|[^.]spawn\(' 'Arc' 'Mutex'
check "part 2 uses scoped threads and an mpsc channel" uses channels.rs "thread::scope and a channel" 'scope' 'mpsc|channel'
check "part 3 uses rayon's parallel iterators" uses with_rayon.rs "rayon" 'par_iter|into_par_iter|par_chunks|par_bridge'
check "parallel versions are not slower than serial (release build)" not_slower
finish
