# 04 · A command-line tool with clap

Command-line tools are everywhere at work: internal tools, deploy scripts, developer tooling. Rust is a favorite for them (ripgrep, fd, bat and Cargo itself are Rust CLIs) because the result is one fast binary with no runtime to install. In this project you finish `taskr`, a to-do manager that stores its tasks in a JSON file, built the way production CLIs are: **clap** for arguments, **anyhow** for errors, **serde** for data, and end-to-end tests with **assert_cmd**.

**You'll practice:** clap's derive API, subcommands and options, environment variables, error context with anyhow, JSON with serde, exit codes, stdout vs. stderr, and testing a binary like a user would.

## Background

### What users expect from a CLI

```
taskr [OPTIONS] <COMMAND>
taskr add "Buy milk" --priority high      # a subcommand, a positional argument, an option
taskr list --all                          # a flag
taskr --file work.json list               # an option that applies to every subcommand
```

- `--help` and `--version` work, and every argument is documented in the help.
- **Standard output is for results**: users pipe it into other programs (`taskr list --json | jq`) or files. **Errors go to standard error**, so they still reach the user's screen and never corrupt the data in a pipe.
- **The exit code** tells scripts what happened: `0` success, `1` the command failed (a missing task, an unreadable file), `2` it was used wrongly (unknown option, missing argument). Check it with `echo $?`.
- Settings can come from **environment variables** (`TASKR_FILE`), so users don't retype them. A flag on the command line beats the variable, which beats the default.

### clap's derive API

You describe the command line as Rust types and clap generates the parser, the help text and the error messages. Doc comments become help text.

```rust
#[derive(Parser)]
#[command(version, about)]                // --version reads Cargo.toml
struct Cli {
    /// The tasks file
    #[arg(long, short, global = true, default_value = "tasks.json", env = "TASKR_FILE")]
    file: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Mark a task as done
    Done {
        /// The task's id, as shown by `taskr list`
        id: u32,                         // positional; "abc" is rejected with exit code 2
    },
    /// Delete every task that is done
    Clear,
}
```

`Cli::parse()` handles `--help`, `--version` and usage errors itself, and exits with code 2 on errors. Some attributes need a Cargo **feature** of clap: `env = ...` only compiles with `features = ["derive", "env"]` in `Cargo.toml`. Features let crates keep rarely used code out of your build.

### Errors with anyhow

In an application, most errors end the same way: tell the user, exit with 1. `anyhow::Result<T>` holds any error type, `?` converts errors for you, and **context** says what you were doing when it failed:

```rust
use anyhow::{Context, Result};

let text = fs::read_to_string(path)
    .with_context(|| format!("could not read {}", path.display()))?;
let task = tasks.iter_mut().find(|t| t.id == id)
    .with_context(|| format!("no task with id {id}"))?;      // works on Option too
```

`main` prints errors with `{:#}`, which shows the whole chain on one line: `error: could not parse tasks.json: expected value at line 1 column 1`. Compare that with a bare `No such file or directory (os error 2)`: which file? Use anyhow in applications; libraries define their own error types with thiserror (project 05).

### Data with serde

`#[derive(Serialize, Deserialize)]` lets `serde_json::from_str` and `serde_json::to_string_pretty` convert your structs to and from JSON; `#[serde(rename_all = "lowercase")]` makes `Priority::High` the string `"high"`. Two habits of careful tools: a **missing** file means "no tasks yet", but a **corrupt** file is an error and is never overwritten, because it's the user's data. And save **atomically**: write a temporary file, then `fs::rename` it over the real one. A rename is all-or-nothing, so a crash or a full disk mid-write can't leave half a file behind.

### Testing a CLI end to end

`tests/cli.rs` runs the real binary in a fresh temporary folder for every test:

```rust
let dir = tempfile::TempDir::new().unwrap();      // deleted when dropped
assert_cmd::Command::new(env!("CARGO_BIN_EXE_taskr"))
    .current_dir(dir.path())
    .args(["done", "42"])
    .assert()
    .code(1)
    .stdout("")
    .stderr(predicates::str::contains("no task with id 42"));
```

`assert()` runs the command; `success()`, `code(n)`, `stdout(...)` and `stderr(...)` check the result, with exact strings or **predicates** (`contains`, `starts_with`, `is_empty().not()`). On failure you see the command, its exit code and both outputs. A temporary folder per test keeps tests independent, so Cargo can run them in parallel.

## Your tasks

Work in `projects/04-cli`. `tests/cli.rs` is the specification: read it first.

1. **Explore.** `cargo run -- --help`, `cargo run -- add --help`. Note the `--` between Cargo's arguments and taskr's.
2. **Load and save** (TODO 1 and 2 in `src/task.rs`), with the error messages the comments give: `could not read <file>`, `could not parse <file>`, `could not write <file>`.
3. **New subcommands** (TODO 3 in `src/main.rs`): `Done { id: u32 }`, `Remove { id: u32 }` and `Clear`, each with a doc comment.
4. **Handle them** (TODO 4):
   - `done ID` marks the task done and prints `Completed task 1: Buy milk`,
   - `remove ID` deletes it and prints `Removed task 1: Buy milk`,
   - an unknown id is an error: `no task with id 42`,
   - `clear` deletes every task that is done and prints `Removed 2 done tasks` (with the real count).
5. **The environment variable** (TODO 5): enable clap's `env` feature in `Cargo.toml` and add `env = "TASKR_FILE"` to `--file`.
6. **Your own tests**: create `tests/clear.rs` with at least 2 tests for `clear`, in the style of `tests/cli.rs`.
7. Run `cargo fmt`, `cargo clippy --all-targets` and `bash ../../tools/grade.sh 04-cli`. Commit and push.
8. Use it for real: `cargo install --path .`, then `export TASKR_FILE=~/tasks.json` and `taskr add "Finish project 05"`.

## What the grader checks

- `tests/cli.rs` is unchanged and every test passes: output, exit codes, files, `--file` and `TASKR_FILE`, error messages on standard error,
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean,
- it runs your `taskr` itself: add three tasks, complete tasks 1 and 3, then `clear` must print `Removed 2 done tasks` and leave only task 2,
- `tests/` has at least 2 tests of your own besides `cli.rs`.

## Hints

- One test at a time: `cargo test --test cli remove`. Add `-- --nocapture` to see what you print.
- `iter_mut().find(...)` gives the task to change; `iter().position(...)` gives an index for `Vec::remove`; `Vec::retain` keeps the elements you choose.
- `error[E0599]: no method named env` means the `env` feature isn't enabled yet.
- Save the file *before* printing the confirmation, so you never claim a change that wasn't saved.

## Going further

- Generate shell completions with the `clap_complete` crate (a hidden `completions` subcommand).
- Default to the user's data folder instead of the current directory, with the `directories` crate.
- Add `taskr edit ID [--title ...] [--priority ...]`, with tests first.
- Color the list only when standard output is a terminal (`std::io::IsTerminal`) and `NO_COLOR` isn't set.
