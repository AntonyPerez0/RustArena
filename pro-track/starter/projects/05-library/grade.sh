#!/usr/bin/env bash
# Grades project 05: a documented, feature-gated library crate in a workspace,
# ready for crates.io (nothing is ever published).
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"
LIB="$HERE/semver-lite"

provided_tests_unchanged() {
  unchanged "$LIB/tests/api.rs" SHA_API &&
    unchanged "$LIB/tests/serde.rs" SHA_SERDE &&
    unchanged "$LIB/tests/metadata.rs" SHA_METADATA &&
    unchanged "$HERE/semver-cli/tests/cli.rs" SHA_CLI
}

typed_errors() {
  grep -q 'thiserror' "$LIB/src/error.rs" || { echo "Derive thiserror::Error for Error in semver-lite/src/error.rs"; return 1; }
  grep -q '#\[non_exhaustive\]' "$LIB/src/error.rs" || { echo "Mark the Error enum #[non_exhaustive]"; return 1; }
}

docs_required() {
  grep -Eq '^#!\[(deny|forbid)\(([a-z_:]+, *)*missing_docs' "$LIB/src/lib.rs" ||
    { echo "Add #![deny(missing_docs)] at the top of semver-lite/src/lib.rs"; return 1; }
}

enough_doc_examples() {
  local n
  n=$(cd "$HERE" && cargo test -p semver-lite --doc --all-features --quiet -- --list 2>/dev/null | grep -c ': test$')
  echo "$n documentation examples found"
  [ "$n" -ge 6 ] || { echo "Write at least 6 examples (\`\`\` blocks in /// or //! comments) in semver-lite"; return 1; }
}

docs_build_cleanly() {
  (cd "$HERE" && RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --quiet)
}

serde_is_optional() {
  if (cd "$HERE" && cargo tree -p semver-lite -e normal --prefix none --color never) | grep -q '^serde '; then
    echo "serde is compiled even without the serde feature: make it optional (optional = true)"
    return 1
  fi
}

package_ready() {
  local out
  out=$(cd "$HERE" && cargo package -p semver-lite --allow-dirty --color never 2>&1)
  local status=$?
  echo "$out"
  [ $status -eq 0 ] || return 1
  if grep -q '^warning' <<<"$out"; then
    echo "cargo package printed warnings: fix them (usually missing metadata in semver-lite/Cargo.toml)"
    return 1
  fi
  (cd "$HERE" && cargo package -p semver-lite --allow-dirty --list --color never 2>/dev/null) | grep -qx 'README.md' ||
    { echo "The package has no README.md: write one in semver-lite/ (crates.io shows it on the crate's page)"; return 1; }
}

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings, with every feature on" clippy_ok "$HERE" --workspace --all-features
check "provided tests are unchanged" provided_tests_unchanged
check "errors are typed with thiserror and #[non_exhaustive]" typed_errors
check "public items must be documented (#![deny(missing_docs)])" docs_required
check "all tests pass, default features" tests_ok "$HERE" --workspace
check "all tests pass with the serde feature" tests_ok "$HERE" --workspace --features semver-lite/serde
check "serde is optional (not compiled by default)" serde_is_optional
check "at least 6 documentation examples run as doc tests" enough_doc_examples
check "cargo doc builds with no warnings (broken links etc.)" docs_build_cleanly
check "cargo package builds the crate with no warnings (ready for crates.io)" package_ready
finish
