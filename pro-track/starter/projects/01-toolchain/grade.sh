#!/usr/bin/env bash
# Grades project 01: Cargo.toml metadata and release profile, the dependency,
# formatting, clippy, the tests (provided and your own) and a release build.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"

# toml_has <section> <key> <regex>: in Cargo.toml's [section], key = value where the
# value (with spaces removed) matches the extended regex.
toml_has() {
  SEC="[$1]" KEY="$2" RE="$3" awk '
    BEGIN { sec = ENVIRON["SEC"]; key = ENVIRON["KEY"]; re = ENVIRON["RE"] }
    { line = $0; gsub(/[[:space:]]/, "", line) }
    substr(line, 1, 1) == "[" { in_sec = (line == sec); next }
    in_sec && index(line, key "=") == 1 && substr(line, length(key) + 2) ~ re { found = 1 }
    END { exit !found }
  ' "$HERE/Cargo.toml"
}

metadata_ok() {
  local ok=0
  toml_has package description '^"[^"]+"' || { echo 'add description = "..." to [package]'; ok=1; }
  toml_has package license '^"[^"]+"' || toml_has package license-file '^"[^"]+"' ||
    { echo 'add license = "MIT" (or another SPDX license) to [package]'; ok=1; }
  toml_has package rust-version '^"1\.[0-9]+(\.[0-9]+)?"' ||
    { echo 'add rust-version = "1.85" (the oldest Rust your crate supports) to [package]'; ok=1; }
  return $ok
}

dependency_ok() {
  toml_has dependencies unicode-segmentation '.' || grep -q '^\[dependencies\.unicode-segmentation\]' "$HERE/Cargo.toml" ||
    { echo "run: cargo add unicode-segmentation"; return 1; }
}

profile_ok() {
  local ok=0
  toml_has profile.release lto '^(true|"fat"|"thin")(#.*)?$' || { echo "set lto = true in [profile.release]"; ok=1; }
  toml_has profile.release strip '^(true|"symbols"|"debuginfo")(#.*)?$' || { echo "set strip = true in [profile.release]"; ok=1; }
  return $ok
}

unit_tests_ok() {
  local n
  n=$(cd "$HERE" && cargo test --quiet --lib --bins -- --list 2>/dev/null | grep -c ': test$')
  echo "unit tests in src/: $n"
  [ "$n" -ge 3 ] || { echo "write at least 3 #[test] functions in src/ (there's one to copy in src/lib.rs)"; return 1; }
}

release_ok() {
  local out
  out=$(cd "$HERE" && printf 'b a b\n' | cargo run --release --quiet -- --top 1) || return 1
  echo "printf 'b a b' | wordfreq --top 1  ->  $out"
  [ "$out" = "b 2" ]
}

check "Cargo.toml has a description, a license and a rust-version" metadata_ok
check "unicode-segmentation is a dependency" dependency_ok
check "the release profile turns on LTO and strips symbols" profile_ok
check "the provided tests (tests/cli.rs) are unchanged" \
  unchanged "$HERE/tests/cli.rs" 5b6e6b026ff578e0cf3142aa630891c80144c234f4a4e68c779542d562086494
check "the code is formatted (cargo fmt --check)" fmt_ok "$HERE"
check "no clippy warnings (cargo clippy -- -D warnings)" clippy_ok "$HERE"
check "every test passes (cargo test)" tests_ok "$HERE"
check "at least 3 unit tests in src/" unit_tests_ok
check "the release build works (cargo run --release)" release_ok
finish
