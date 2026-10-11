//! Tests for `taskr clear`.

use assert_cmd::Command;
use tempfile::TempDir;

fn taskr(dir: &TempDir) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_taskr"));
    cmd.current_dir(dir.path()).env_remove("TASKR_FILE");
    cmd
}

#[test]
fn clear_removes_only_the_done_tasks() {
    let dir = TempDir::new().unwrap();
    for title in ["a", "b", "c"] {
        taskr(&dir).args(["add", title]).assert().success();
    }
    taskr(&dir).args(["done", "1"]).assert().success();
    taskr(&dir).args(["done", "3"]).assert().success();
    taskr(&dir)
        .arg("clear")
        .assert()
        .success()
        .stdout("Removed 2 done tasks\n");
    taskr(&dir)
        .args(["list", "--all"])
        .assert()
        .stdout("  2 [ ] normal b\n");
}

#[test]
fn clear_with_nothing_done_removes_nothing() {
    let dir = TempDir::new().unwrap();
    taskr(&dir).args(["add", "a"]).assert().success();
    taskr(&dir)
        .arg("clear")
        .assert()
        .success()
        .stdout("Removed 0 done tasks\n");
    taskr(&dir)
        .arg("list")
        .assert()
        .stdout("  1 [ ] normal a\n");
}
