//! The public API, tested from outside the crate exactly as a user would call it.
//! Don't edit this file: the grader checks that it is unchanged.

use std::collections::HashSet;
use std::error::Error as _;

use semver_lite::{Error, Parser, Prerelease, Version, VersionReq};

fn v(text: &str) -> Version {
    Version::parse(text).unwrap_or_else(|e| panic!("{text:?} should parse, got: {e:?}"))
}

fn req(text: &str) -> VersionReq {
    text.parse()
        .unwrap_or_else(|e| panic!("{text:?} should parse, got: {e:?}"))
}

// ---------------------------------------------------------------- parsing

#[test]
fn parses_a_release() {
    let x = v("1.2.3");
    assert_eq!((x.major, x.minor, x.patch), (1, 2, 3));
    assert!(x.pre.is_empty());
    assert!(!x.is_prerelease());
    assert_eq!(x, Version::new(1, 2, 3));
}

#[test]
fn parses_a_prerelease() {
    let x = v("1.0.0-rc.1");
    assert_eq!(x.pre.as_str(), "rc.1");
    assert!(x.is_prerelease());
    assert_eq!(v("0.0.0-x-y-z.0").pre.as_str(), "x-y-z.0");
}

#[test]
fn trims_surrounding_whitespace() {
    assert_eq!(v("  2.0.0\n"), Version::new(2, 0, 0));
}

#[test]
fn handles_the_largest_numbers() {
    let max = u64::MAX;
    assert_eq!(v(&format!("{max}.0.0")).major, max);
    assert_eq!(
        Version::parse("18446744073709551616.0.0"),
        Err(Error::InvalidNumber {
            part: "18446744073709551616".into()
        })
    );
}

#[test]
fn from_str_and_display_round_trip() {
    for text in [
        "0.0.0",
        "1.2.3",
        "10.20.30",
        "1.0.0-alpha.1",
        "2.0.0-rc.1.x-y",
    ] {
        let parsed: Version = text.parse().unwrap();
        assert_eq!(parsed.to_string(), text);
    }
}

#[test]
fn rejects_bad_versions_with_typed_errors() {
    assert_eq!(Version::parse(""), Err(Error::Empty));
    assert_eq!(Version::parse("   "), Err(Error::Empty));
    assert_eq!(
        Version::parse("1.2"),
        Err(Error::BadShape {
            input: "1.2".into()
        })
    );
    assert_eq!(
        Version::parse(" 1.2.3.4 "),
        Err(Error::BadShape {
            input: "1.2.3.4".into()
        })
    );
    assert_eq!(
        Version::parse("1.x.3"),
        Err(Error::InvalidNumber { part: "x".into() })
    );
    assert_eq!(
        Version::parse("1..3"),
        Err(Error::InvalidNumber { part: "".into() })
    );
    assert_eq!(
        Version::parse("1.+2.3"),
        Err(Error::InvalidNumber { part: "+2".into() })
    );
    assert_eq!(
        Version::parse("1.02.3"),
        Err(Error::LeadingZero { part: "02".into() })
    );
    assert_eq!(
        Version::parse("1.2.3-"),
        Err(Error::InvalidPrerelease("".into()))
    );
    assert_eq!(
        Version::parse("1.2.3-rc..1"),
        Err(Error::InvalidPrerelease("rc..1".into()))
    );
    assert_eq!(
        Version::parse("1.2.3-01"),
        Err(Error::InvalidPrerelease("01".into()))
    );
    // The strict parser does not accept a v prefix.
    assert_eq!(
        Version::parse("v1.2.3"),
        Err(Error::InvalidNumber { part: "v1".into() })
    );
}

#[test]
fn error_messages() {
    let msg = |text: &str| Version::parse(text).unwrap_err().to_string();
    assert_eq!(msg(""), "empty version string");
    assert_eq!(
        msg("1.2"),
        r#"invalid version "1.2": expected MAJOR.MINOR.PATCH"#
    );
    assert_eq!(msg("1.x.3"), r#"invalid version number "x""#);
    assert_eq!(msg("1.02.3"), r#"version number "02" has a leading zero"#);
    assert_eq!(msg("1.2.3-rc..1"), r#"invalid pre-release "rc..1""#);
}

#[test]
fn errors_are_real_errors_that_can_cross_threads() {
    fn assert_error<E: std::error::Error + Send + Sync + 'static>() {}
    assert_error::<Error>();
    // So callers can use `?` in functions returning Box<dyn Error> or anyhow::Result.
    let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(Error::Empty);
    assert_eq!(boxed.to_string(), "empty version string");
}

// ---------------------------------------------------------------- Prerelease

#[test]
fn prerelease_newtype_checks_the_rules() {
    assert_eq!(Prerelease::new("").unwrap(), Prerelease::EMPTY);
    assert!(Prerelease::EMPTY.is_empty());
    for ok in ["alpha", "alpha.1", "0", "x-y-z", "rc.0.beta", "--"] {
        assert!(Prerelease::new(ok).is_ok(), "{ok:?} is a valid pre-release");
    }
    for bad in ["a..b", ".a", "a.", "01", "a b", "été", "rc_1"] {
        assert_eq!(
            Prerelease::new(bad),
            Err(Error::InvalidPrerelease(bad.into())),
            "{bad:?} is not a valid pre-release"
        );
    }
}

#[test]
fn prerelease_is_string_like() {
    let p = Prerelease::new("beta.2").unwrap();
    let s: &str = p.as_ref();
    assert_eq!(s, "beta.2");
    assert_eq!(p.to_string(), "beta.2");
}

// ---------------------------------------------------------------- ordering

#[test]
fn orders_by_semver_precedence() {
    // The example from semver.org, item 11, plus a few more.
    let ordered = [
        "0.9.9",
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
        "1.0.1",
        "1.1.0",
        "1.9.0",
        "1.10.0",
        "2.0.0",
    ];
    for pair in ordered.windows(2) {
        assert!(v(pair[0]) < v(pair[1]), "{} < {}", pair[0], pair[1]);
        assert!(v(pair[1]) > v(pair[0]), "{} > {}", pair[1], pair[0]);
    }
    let mut shuffled: Vec<Version> = ordered.iter().rev().map(|s| v(s)).collect();
    shuffled.swap(0, 5);
    shuffled.sort();
    let sorted: Vec<String> = shuffled.iter().map(Version::to_string).collect();
    assert_eq!(sorted, ordered);
}

#[test]
fn prerelease_numbers_compare_as_numbers_even_when_huge() {
    assert!(v("1.0.0-99999999999999999999") < v("1.0.0-100000000000000000000"));
    assert!(v("1.0.0-99999999999999999999") < v("1.0.0-a"));
}

#[test]
fn equal_versions_are_equal_everywhere() {
    assert_eq!(
        v("1.2.3-rc.1").cmp(&v("1.2.3-rc.1")),
        std::cmp::Ordering::Equal
    );
    let set: HashSet<Version> = ["1.0.0", "1.0.0", "1.0.0-rc.1"]
        .iter()
        .map(|s| v(s))
        .collect();
    assert_eq!(set.len(), 2);
}

#[test]
fn bumps_return_a_new_version() {
    let x = v("1.2.3");
    assert_eq!(x.bump_major(), v("2.0.0"));
    assert_eq!(x.bump_minor(), v("1.3.0"));
    assert_eq!(x.bump_patch(), v("1.2.4"));
    assert_eq!(x, v("1.2.3"), "bumping must not change the original");
    assert_eq!(v("1.2.3-rc.1").bump_patch(), v("1.2.4"));
    assert_eq!(v("0.9.0-alpha").bump_major(), v("1.0.0"));
}

// ---------------------------------------------------------------- Parser builder

#[test]
fn the_default_parser_is_strict() {
    assert_eq!(Parser::new().parse("1.2.3"), Ok(Version::new(1, 2, 3)));
    assert_eq!(Parser::default(), Parser::new());
    assert!(Parser::new().parse("v1.2.3").is_err());
    assert!(Parser::new().parse("1.2").is_err());
}

#[test]
fn allow_v_prefix() {
    let p = Parser::new().allow_v_prefix(true);
    assert_eq!(p.parse("v1.2.3"), Ok(v("1.2.3")));
    assert_eq!(p.parse("V1.2.3-rc.1"), Ok(v("1.2.3-rc.1")));
    assert_eq!(p.parse("1.2.3"), Ok(v("1.2.3")));
    assert!(p.parse("vv1.2.3").is_err());
    assert!(p.parse("v1.2").is_err());
}

#[test]
fn allow_partial() {
    let p = Parser::new().allow_partial(true);
    assert_eq!(p.parse("1"), Ok(v("1.0.0")));
    assert_eq!(p.parse("1.2"), Ok(v("1.2.0")));
    assert_eq!(p.parse("1.2.3"), Ok(v("1.2.3")));
    assert_eq!(p.parse(""), Err(Error::Empty));
    assert_eq!(
        p.parse("1.2.3.4"),
        Err(Error::BadShape {
            input: "1.2.3.4".into()
        })
    );
}

#[test]
fn builder_options_combine_and_can_be_switched_off() {
    let lenient = Parser::new().allow_v_prefix(true).allow_partial(true);
    assert_eq!(lenient.parse("v2"), Ok(v("2.0.0")));
    let strict_again = lenient.allow_v_prefix(false).allow_partial(false);
    assert_eq!(strict_again, Parser::new());
}

#[test]
fn parse_all_accepts_anything_string_like() {
    let p = Parser::new();
    assert_eq!(
        p.parse_all(["1.0.0", "2.0.0"]),
        Ok(vec![v("1.0.0"), v("2.0.0")])
    );
    let owned: Vec<String> = vec!["3.0.0".to_string()];
    assert_eq!(p.parse_all(&owned), Ok(vec![v("3.0.0")]));
    assert_eq!(p.parse_all(owned), Ok(vec![v("3.0.0")]));
    assert_eq!(p.parse_all(Vec::<&str>::new()), Ok(vec![]));
    assert_eq!(
        p.parse_all(["1.0.0", "nope", "1.x.0"]),
        Err(Error::BadShape {
            input: "nope".into()
        }),
        "parse_all returns the first error"
    );
}

// ---------------------------------------------------------------- VersionReq

fn check(req_text: &str, yes: &[&str], no: &[&str]) {
    let r = req(req_text);
    for s in yes {
        assert!(r.matches(&v(s)), "{req_text:?} should match {s}");
    }
    for s in no {
        assert!(!r.matches(&v(s)), "{req_text:?} should not match {s}");
    }
}

#[test]
fn caret_requirements() {
    check(
        "^1.2.3",
        &["1.2.3", "1.2.4", "1.9.0"],
        &["1.2.2", "2.0.0", "0.9.0"],
    );
    check("1.2.3", &["1.2.3", "1.9.0"], &["2.0.0", "1.2.2"]);
    check("^1.2", &["1.2.0", "1.99.0"], &["1.1.9", "2.0.0"]);
    check("^1", &["1.0.0", "1.99.99"], &["0.9.0", "2.0.0"]);
    check("^0.2.3", &["0.2.3", "0.2.9"], &["0.2.2", "0.3.0", "1.0.0"]);
    check("^0.2", &["0.2.0", "0.2.9"], &["0.3.0"]);
    check("^0.0.3", &["0.0.3"], &["0.0.4", "0.1.0"]);
    check("^0", &["0.0.1", "0.9.9"], &["1.0.0"]);
}

#[test]
fn tilde_requirements() {
    check("~1.2.3", &["1.2.3", "1.2.9"], &["1.2.2", "1.3.0"]);
    check("~1.2", &["1.2.0", "1.2.9"], &["1.3.0", "1.1.0"]);
    check("~1", &["1.0.0", "1.9.9"], &["2.0.0"]);
}

#[test]
fn comparison_requirements() {
    check(">=1.0.0, <2.0.0", &["1.0.0", "1.5.0"], &["0.9.9", "2.0.0"]);
    check("=1.2.3", &["1.2.3"], &["1.2.4", "1.2.3-rc.1"]);
    check(">1.2.3", &["1.2.4", "2.0.0"], &["1.2.3"]);
    check("<=1.2.3", &["1.2.3", "0.1.0", "1.2.3-rc.1"], &["1.2.4"]);
    check(">= 1.2.3", &["1.2.3"], &["1.2.2"]);
}

#[test]
fn star_matches_everything() {
    check("*", &["0.0.1", "1.0.0-alpha", "99.0.0"], &[]);
    assert_eq!(req("*"), VersionReq::STAR);
}

#[test]
fn prereleases_and_ranges() {
    // 1.2.0-rc.1 comes before 1.2.0, so it is below the range.
    check("^1.2", &["1.2.0"], &["1.2.0-rc.1"]);
    check(">=1.2.0-rc.1", &["1.2.0-rc.2", "1.2.0"], &["1.2.0-beta"]);
    // Upper bounds ignore pre-releases.
    check("^1.2", &["1.9.9"], &["2.0.0-alpha"]);
}

#[test]
fn invalid_requirements_keep_their_cause() {
    let err = "^1.x".parse::<VersionReq>().unwrap_err();
    assert!(matches!(err, Error::InvalidReq { .. }), "{err:?}");
    assert_eq!(err.to_string(), r#"invalid version requirement "^1.x""#);
    let cause = err
        .source()
        .expect("InvalidReq should return its cause from source()");
    assert_eq!(cause.to_string(), r#"invalid version number "x""#);

    // `>=` needs a full version.
    let err = ">=1.2".parse::<VersionReq>().unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        r#"invalid version "1.2": expected MAJOR.MINOR.PATCH"#
    );

    let err = "^1.2,".parse::<VersionReq>().unwrap_err();
    assert_eq!(
        err,
        Error::InvalidReq {
            req: "^1.2,".into(),
            source: Box::new(Error::Empty)
        }
    );
}

#[test]
fn max_satisfying_and_matching() {
    let versions = Parser::new()
        .parse_all(["1.0.0", "1.9.0", "1.2.5", "2.0.0", "1.9.1-rc.1", "0.8.0"])
        .unwrap();
    let r = req("^1.2");
    assert_eq!(r.max_satisfying(&versions), Some(&v("1.9.1-rc.1")));
    assert_eq!(
        req(">=1.0.0, <1.9.1-rc.1").max_satisfying(&versions),
        Some(&v("1.9.0"))
    );
    assert_eq!(req("^3").max_satisfying(&versions), None);
    assert_eq!(req("^3").max_satisfying(Vec::new().iter()), None);
    let matching: Vec<String> = r.matching(&versions).map(Version::to_string).collect();
    assert_eq!(matching, ["1.9.0", "1.2.5", "1.9.1-rc.1"]);
    assert_eq!(req("*").matching(&versions).count(), versions.len());
}
