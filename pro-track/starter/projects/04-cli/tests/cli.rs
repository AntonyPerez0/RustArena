//! End-to-end tests for taskr. Each test runs the real binary in a fresh
//! temporary folder, the way a user would, and checks its output, its exit code
//! and the tasks file it leaves behind.
//!
//! Don't change this file: the grader checks that it is untouched. Put your own
//! tests in another file in tests/.

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

/// A `taskr` command that runs inside `dir`, so the default tasks.json lands there.
fn taskr(dir: &TempDir) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_taskr"));
    cmd.current_dir(dir.path()).env_remove("TASKR_FILE");
    cmd
}

/// Runs `taskr args...` in `dir` and expects it to succeed.
fn ok(dir: &TempDir, args: &[&str]) {
    taskr(dir).args(args).assert().success();
}

/// The parsed contents of a JSON file in `dir`.
fn read_json(dir: &TempDir, name: &str) -> Value {
    let text = fs::read_to_string(dir.path().join(name)).expect("the tasks file exists");
    serde_json::from_str(&text).expect("the tasks file is valid JSON")
}

#[test]
fn add_prints_the_new_id_and_saves_pretty_json() {
    let dir = TempDir::new().unwrap();
    taskr(&dir)
        .args(["add", "Buy milk"])
        .assert()
        .success()
        .stdout("Added task 1: Buy milk\n")
        .stderr("");
    let tasks = read_json(&dir, "tasks.json");
    assert_eq!(
        tasks,
        serde_json::json!([{ "id": 1, "title": "Buy milk", "priority": "normal", "done": false }])
    );
    let text = fs::read_to_string(dir.path().join("tasks.json")).unwrap();
    assert!(
        text.contains("\"title\": \"Buy milk\""),
        "pretty-print the file:\n{text}"
    );
}

#[test]
fn ids_keep_increasing() {
    let dir = TempDir::new().unwrap();
    ok(&dir, &["add", "a"]);
    taskr(&dir)
        .args(["add", "b"])
        .assert()
        .stdout("Added task 2: b\n");
    ok(&dir, &["remove", "1"]);
    taskr(&dir)
        .args(["add", "c"])
        .assert()
        .stdout("Added task 3: c\n");
}

#[test]
fn list_shows_open_tasks_as_a_table() {
    let dir = TempDir::new().unwrap();
    ok(&dir, &["add", "Buy milk", "--priority", "high"]);
    ok(&dir, &["add", "Walk the dog"]);
    ok(&dir, &["add", "-p", "low", "Water the plants"]);
    taskr(&dir).arg("list").assert().success().stdout(
        "  1 [ ] high   Buy milk\n  2 [ ] normal Walk the dog\n  3 [ ] low    Water the plants\n",
    );
}

#[test]
fn list_without_a_file_has_nothing_to_do_and_creates_nothing() {
    let dir = TempDir::new().unwrap();
    taskr(&dir)
        .arg("list")
        .assert()
        .success()
        .stdout("No tasks.\n");
    assert!(
        !dir.path().join("tasks.json").exists(),
        "listing must not create the file"
    );
}

#[test]
fn done_hides_a_task_unless_all_is_given() {
    let dir = TempDir::new().unwrap();
    ok(&dir, &["add", "a"]);
    ok(&dir, &["add", "b"]);
    taskr(&dir)
        .args(["done", "1"])
        .assert()
        .success()
        .stdout("Completed task 1: a\n");
    taskr(&dir)
        .arg("list")
        .assert()
        .stdout("  2 [ ] normal b\n");
    taskr(&dir)
        .args(["list", "--all"])
        .assert()
        .stdout("  1 [x] normal a\n  2 [ ] normal b\n");
}

#[test]
fn remove_deletes_a_task() {
    let dir = TempDir::new().unwrap();
    ok(&dir, &["add", "a"]);
    taskr(&dir)
        .args(["remove", "1"])
        .assert()
        .success()
        .stdout("Removed task 1: a\n");
    assert_eq!(read_json(&dir, "tasks.json"), serde_json::json!([]));
}

#[test]
fn list_json_is_for_scripts() {
    let dir = TempDir::new().unwrap();
    ok(&dir, &["add", "a"]);
    ok(&dir, &["add", "b", "--priority", "low"]);
    ok(&dir, &["done", "1"]);
    let out = taskr(&dir).args(["list", "--json"]).output().unwrap();
    let open: Value = serde_json::from_slice(&out.stdout).expect("list --json prints JSON");
    assert_eq!(
        open,
        serde_json::json!([{ "id": 2, "title": "b", "priority": "low", "done": false }])
    );
    let out = taskr(&dir)
        .args(["list", "--all", "--json"])
        .output()
        .unwrap();
    let all: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(all.as_array().map(Vec::len), Some(2));
    assert_eq!(all[0]["done"], true);
}

#[test]
fn an_unknown_id_is_an_error_with_exit_code_1() {
    let dir = TempDir::new().unwrap();
    ok(&dir, &["add", "a"]);
    for command in ["done", "remove"] {
        taskr(&dir)
            .args([command, "42"])
            .assert()
            .code(1)
            .stdout("")
            .stderr(predicate::str::starts_with("error:"))
            .stderr(predicate::str::contains("no task with id 42"));
    }
}

#[test]
fn usage_errors_exit_with_code_2() {
    let dir = TempDir::new().unwrap();
    let bad: [&[&str]; 6] = [
        &[],
        &["frobnicate"],
        &["add"],
        &["add", ""],
        &["add", "x", "--priority", "urgent"],
        &["done", "abc"],
    ];
    for args in bad {
        taskr(&dir)
            .args(args)
            .assert()
            .code(2)
            .stdout("")
            .stderr(predicate::str::is_empty().not());
    }
    assert!(!dir.path().join("tasks.json").exists());
}

#[test]
fn help_lists_every_subcommand_and_version_comes_from_cargo_toml() {
    let dir = TempDir::new().unwrap();
    let mut help = taskr(&dir).arg("--help").assert().success();
    for name in ["add", "list", "done", "remove", "clear"] {
        help = help.stdout(predicate::str::contains(name));
    }
    taskr(&dir)
        .arg("--version")
        .assert()
        .success()
        .stdout(format!("taskr {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn file_option_chooses_the_tasks_file() {
    let dir = TempDir::new().unwrap();
    ok(&dir, &["--file", "work.json", "add", "a"]);
    ok(&dir, &["add", "b", "--file", "work.json"]);
    assert_eq!(
        read_json(&dir, "work.json").as_array().map(Vec::len),
        Some(2)
    );
    assert!(!dir.path().join("tasks.json").exists());
}

#[test]
fn taskr_file_variable_chooses_the_tasks_file_unless_file_is_given() {
    let dir = TempDir::new().unwrap();
    taskr(&dir)
        .env("TASKR_FILE", "env.json")
        .args(["add", "a"])
        .assert()
        .success();
    assert!(dir.path().join("env.json").exists());
    taskr(&dir)
        .env("TASKR_FILE", "env.json")
        .args(["--file", "flag.json", "add", "b"])
        .assert()
        .success();
    assert!(dir.path().join("flag.json").exists());
    assert_eq!(
        read_json(&dir, "env.json").as_array().map(Vec::len),
        Some(1)
    );
}

#[test]
fn a_corrupt_file_is_reported_by_name_and_left_alone() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("tasks.json"), "this is not JSON").unwrap();
    for args in [&["list"][..], &["add", "a"]] {
        taskr(&dir)
            .args(args)
            .assert()
            .code(1)
            .stdout("")
            .stderr(predicate::str::contains("could not parse tasks.json"));
    }
    let text = fs::read_to_string(dir.path().join("tasks.json")).unwrap();
    assert_eq!(text, "this is not JSON");
}

#[test]
fn a_file_that_cannot_be_written_is_reported() {
    let dir = TempDir::new().unwrap();
    taskr(&dir)
        .args(["--file", "no-such-folder/tasks.json", "add", "a"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains(
            "could not write no-such-folder/tasks.json",
        ));
}
