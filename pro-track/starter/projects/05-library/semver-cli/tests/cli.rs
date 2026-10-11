//! End-to-end tests: run the real `semver` binary like a user would.
//! Don't edit this file: the grader checks that it is unchanged.

use std::process::{Command, Output};

fn semver(args: &[&str]) -> Output {
    // Cargo builds the binary before the tests and tells us where it is.
    Command::new(env!("CARGO_BIN_EXE_semver"))
        .args(args)
        .output()
        .expect("failed to run the semver binary")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn parse_shows_the_parts() {
    let out = semver(&["parse", "v1.2.3-rc.1"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "major: 1\nminor: 2\npatch: 3\npre-release: rc.1\n"
    );
}

#[test]
fn sort_orders_by_precedence() {
    let out = semver(&["sort", "1.10.0", "v1.2.0", "1.2.0-rc.1", "1.2.0-alpha", "0.9.9"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "0.9.9\n1.2.0-alpha\n1.2.0-rc.1\n1.2.0\n1.10.0\n"
    );
}

#[test]
fn bump_minor_resets_patch() {
    let out = semver(&["bump", "minor", "1.4.7"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "1.5.0\n");
}

#[test]
fn match_prints_matching_versions_in_order() {
    let out = semver(&["match", "^1.2", "1.1.0", "1.9.0", "1.2.5", "2.0.0"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "1.9.0\n1.2.5\n");
}

#[test]
fn max_prints_the_highest_match() {
    let out = semver(&["max", "~1.2", "1.2.0", "1.2.9", "1.3.0"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "1.2.9\n");
}

#[test]
fn nothing_matching_exits_with_1() {
    let out = semver(&["match", "^3", "1.0.0", "2.0.0"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "");
}

#[test]
fn bad_versions_exit_with_2_and_say_why() {
    let out = semver(&["sort", "1.0.0", "1.02.0"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        stderr(&out),
        "error: version number \"02\" has a leading zero\n"
    );
}

#[test]
fn bad_requirements_print_the_whole_error_chain() {
    let out = semver(&["match", "^1.x", "1.0.0"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        stderr(&out),
        "error: invalid version requirement \"^1.x\"\n  caused by: invalid version number \"x\"\n"
    );
}

#[test]
fn no_arguments_prints_usage() {
    let out = semver(&[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("usage:"), "{}", stderr(&out));
}
