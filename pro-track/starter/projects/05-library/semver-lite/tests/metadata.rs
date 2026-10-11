//! crates.io needs to know what a crate is, who may use it and where its code
//! lives. Cargo passes these Cargo.toml fields to the compiler as environment
//! variables (empty when the field is missing), so a test can check them.
//! Don't edit this file: the grader checks that it is unchanged.

fn require(field: &str, value: &str) {
    assert!(
        !value.is_empty(),
        "`{field}` is missing from [package] in semver-lite/Cargo.toml (see task 6 in the README)"
    );
}

#[test]
fn has_a_description() {
    require("description", env!("CARGO_PKG_DESCRIPTION"));
}

#[test]
fn has_a_license() {
    require("license", env!("CARGO_PKG_LICENSE"));
}

#[test]
fn has_a_repository() {
    require("repository", env!("CARGO_PKG_REPOSITORY"));
}

#[test]
fn declares_its_minimum_supported_rust_version() {
    require("rust-version", env!("CARGO_PKG_RUST_VERSION"));
}

#[test]
fn has_a_readme() {
    require(
        "readme (or a README.md next to Cargo.toml)",
        env!("CARGO_PKG_README"),
    );
}

#[test]
fn its_own_version_is_valid_semver() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        semver_lite::Version::parse(version).is_ok(),
        "{version} is not valid semver"
    );
}
