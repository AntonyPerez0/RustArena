//! Tests for the optional `serde` feature. Without the feature this whole file
//! compiles to nothing, so run it with: cargo test -p semver-lite --features serde
//! Don't edit this file: the grader checks that it is unchanged.
#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use semver_lite::Version;

#[test]
fn a_version_serializes_as_a_string() {
    let v = Version::parse("1.2.3-rc.1").unwrap();
    assert_eq!(serde_json::to_string(&v).unwrap(), r#""1.2.3-rc.1""#);
}

#[test]
fn a_version_deserializes_from_a_string() {
    let v: Version = serde_json::from_str(r#""2.0.0""#).unwrap();
    assert_eq!(v, Version::new(2, 0, 0));
}

#[test]
fn invalid_strings_are_rejected_with_the_parse_error() {
    let err = serde_json::from_str::<Version>(r#""1.02.0""#).unwrap_err();
    assert!(err.to_string().contains("leading zero"), "{err}");
    assert!(serde_json::from_str::<Version>("1").is_err(), "a number is not a version");
}

#[test]
fn works_inside_other_types() {
    let json = r#"{"serde":"1.0.228","tokio":"1.47.0"}"#;
    let deps: BTreeMap<String, Version> = serde_json::from_str(json).unwrap();
    assert_eq!(deps["tokio"], Version::new(1, 47, 0));
    assert_eq!(serde_json::to_string(&deps).unwrap(), json);
}
