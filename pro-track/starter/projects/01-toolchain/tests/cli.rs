//! End-to-end tests for the `wordfreq` binary. Each test starts the real program
//! the way a user (or a shell script) would, feeds it text on standard input, and
//! checks what it prints and its exit code.
//!
//! Don't change this file: the grader checks that it is untouched.

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// Runs `wordfreq` with `args`, writes `input` to its standard input and waits for it.
fn wordfreq(args: &[&str], input: &[u8]) -> Output {
    // Cargo builds the binary before it builds integration tests, and tells them
    // where it is through the CARGO_BIN_EXE_<name> variable.
    let mut child = Command::new(env!("CARGO_BIN_EXE_wordfreq"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("could not start wordfreq");
    let mut stdin = child.stdin.take().expect("stdin is piped");
    // The program may exit without reading its input (after --help, say), so a
    // failed write is not an error here.
    let _ = stdin.write_all(input);
    drop(stdin); // closing stdin is how the program learns the input is over
    child.wait_with_output().expect("wordfreq did not finish")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Checks that `wordfreq args` prints exactly `expected` for `input`, and exits with 0.
fn assert_prints(args: &[&str], input: &str, expected: &str) {
    let out = wordfreq(args, input.as_bytes());
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), expected, "for input {input:?}");
}

#[test]
fn prints_the_most_frequent_words_first() {
    assert_prints(
        &[],
        "the cat sat on the mat and the cat slept\n",
        "the 3\ncat 2\nand 1\nmat 1\non 1\nsat 1\nslept 1\n",
    );
}

#[test]
fn ignores_case_and_punctuation() {
    assert_prints(
        &[],
        "Rust, rust! RUST? Ferris says: rust.\n",
        "rust 4\nferris 1\nsays 1\n",
    );
}

#[test]
fn keeps_apostrophes_and_decimal_points_inside_words() {
    assert_prints(
        &[],
        "Don't panic. don't! Pi is 3.14, not 3.\n",
        "don't 2\n3 1\n3.14 1\nis 1\nnot 1\npanic 1\npi 1\n",
    );
}

#[test]
fn handles_letters_beyond_ascii() {
    assert_prints(&[], "Café café CAFÉ crème\n", "café 3\ncrème 1\n");
}

#[test]
fn top_limits_how_many_words_are_printed() {
    assert_prints(&["--top", "2"], "a a a b b c\n", "a 3\nb 2\n");
    assert_prints(&["-n", "1"], "a a a b b c\n", "a 3\n");
}

#[test]
fn prints_ten_words_by_default() {
    let input = "one two three four five six seven eight nine ten eleven twelve twelve\n";
    let out = wordfreq(&[], input.as_bytes());
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr(&out));
    let lines: Vec<String> = stdout(&out).lines().map(String::from).collect();
    assert_eq!(lines.len(), 10, "got:\n{}", stdout(&out));
    assert_eq!(lines[0], "twelve 2");
    assert_eq!(lines[1], "eight 1");
}

#[test]
fn input_without_words_prints_nothing() {
    assert_prints(&[], "", "");
    assert_prints(&[], "  ... !!! --\n\n", "");
}

#[test]
fn rejects_a_bad_top_value_with_exit_code_2() {
    for args in [&["--top", "many"][..], &["--top", "0"], &["--top"]] {
        let out = wordfreq(args, b"a b c\n");
        assert_eq!(out.status.code(), Some(2), "for {args:?}");
        assert_eq!(stdout(&out), "", "errors must not go to standard output");
        assert!(
            stderr(&out).contains("usage: wordfreq"),
            "show the usage on standard error, got: {}",
            stderr(&out)
        );
    }
}

#[test]
fn rejects_unknown_arguments_with_exit_code_2() {
    let out = wordfreq(&["--frobnicate"], b"a b c\n");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stdout(&out), "");
    assert!(stderr(&out).contains("--frobnicate"), "{}", stderr(&out));
}

#[test]
fn help_goes_to_standard_output() {
    let out = wordfreq(&["--help"], b"");
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("usage: wordfreq"), "{}", stdout(&out));
    assert_eq!(stderr(&out), "");
}

#[test]
fn version_comes_from_cargo_toml() {
    let out = wordfreq(&["--version"], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        stdout(&out),
        format!("wordfreq {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn invalid_utf8_is_a_runtime_error_with_exit_code_1() {
    // "café" in Latin-1, which is not valid UTF-8.
    let out = wordfreq(&[], b"caf\xe9\n");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "");
    assert!(stderr(&out).starts_with("wordfreq:"), "{}", stderr(&out));
}
