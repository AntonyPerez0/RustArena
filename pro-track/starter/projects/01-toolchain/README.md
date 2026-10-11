# 01 · rustup, Cargo and continuous integration

On the RustArena site, one click compiled and ran your code. At work you'll run the same toolchain yourself: **rustup** installs Rust, **Cargo** builds, tests and manages dependencies, **rustfmt** and **clippy** keep code consistent, and **CI** runs all of it on every push so nobody merges broken code. In this project you finish a small command-line tool, `wordfreq`, and get it through the checks a Rust team's CI would run.

**You'll practice:** rustup and toolchains, Cargo commands, `Cargo.toml` and dependencies, debug and release profiles, rustfmt, clippy, unit and integration tests, and reading CI results.

## Background

### rustup: one installer for every Rust

A **toolchain** is a compiler (`rustc`), Cargo and the standard library for one **channel**: `stable` (a new release every six weeks), `beta` or `nightly`, or a pinned version like `1.85.0`. rustup installs and switches them. **Components** are optional tools for a toolchain.

```bash
rustup show                          # installed toolchains and which one is active
rustup update                        # upgrade stable (do it every few weeks)
rustup component add rustfmt clippy  # the formatter and the linter
rustup toolchain install 1.85        # an older compiler, to test your minimum version
cargo +1.85 test                     # run one command with that toolchain
```

Teams often commit a `rust-toolchain.toml` so every laptop and CI machine uses the same channel. rustup reads it automatically in that folder:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

Versions are not **editions** (2018, 2021, 2024): an edition is a set of language rules each crate picks in `Cargo.toml`, and crates on different editions work together.

### Cargo, the tool you'll type all day

| Command | What it does |
|---|---|
| `cargo new name` | create a package (`--lib` for a library) |
| `cargo check` | type-check without producing a binary: the fastest feedback |
| `cargo build` / `cargo run -- ARGS` | compile (and run) a debug build; `--` separates your program's arguments from Cargo's |
| `cargo test` | build and run unit, integration and doc tests |
| `cargo doc --open` | generate HTML docs for your crate and every dependency |
| `cargo add name` / `cargo tree` | add a dependency / show the dependency graph |
| `cargo fmt` / `cargo clippy` | format the code / lint it |

This package has **two crates**: a library (`src/lib.rs`, where the logic lives so it can be unit tested) and a binary (`src/main.rs`, which handles arguments and I/O and calls `wordfreq::count_words`). That split is how most Rust command-line tools are organized.

### Cargo.toml and dependencies

```toml
[package]
name = "wordfreq"
version = "0.1.0"          # semantic versioning: MAJOR.MINOR.PATCH
edition = "2024"
rust-version = "1.85"      # the oldest compiler you support (your MSRV)
description = "Prints the most frequent words in a text"
license = "MIT"            # an SPDX identifier; crates.io requires it

[dependencies]
unicode-segmentation = "1"
```

`"1"` is a **caret requirement**: any version from 1.0.0 up to (not including) 2.0.0, because semver promises no breaking changes within a major version. The exact versions Cargo picked go in **`Cargo.lock`**. Commit it: it makes every build, yours and CI's, use the same code. `cargo update` moves to newer compatible versions on purpose, as its own reviewable commit.

### Profiles: debug and release builds

`cargo build` uses the **dev** profile: quick to compile, unoptimized, with debug info, and integer overflow **panics** so you find bugs. `cargo build --release` uses the **release** profile: optimized (often 10 to 100 times faster), and overflow wraps silently. Builds land in `target/debug/` and `target/release/` (here, one `target/` at the repository root shared by every project, set in `.cargo/config.toml`). Profiles are tunable:

```toml
[profile.release]
lto = true      # link-time optimization: optimize across crates, smaller and faster binaries
strip = true    # remove symbols from the binary: much smaller files
```

Never benchmark or ship a debug build.

### rustfmt and clippy

**rustfmt** formats code one standard way. Teams run it on save and check it in CI, so code review is about logic, not spacing. `cargo fmt` rewrites files; `cargo fmt --check` only reports. **clippy** is a linter with hundreds of lints for bugs, slow code and unidiomatic code; each warning links to an explanation. CI runs it with `-D warnings` so every warning is an error. When a lint is wrong for one spot, silence it there with `#[allow(clippy::lint_name)]` and a comment saying why.

### Continuous integration

Open `.github/workflows/grade.yml` at the root of this repository. On every push, GitHub Actions starts a fresh Ubuntu machine, installs stable Rust with rustfmt and clippy, restores a cache of compiled dependencies, and runs `bash tools/grade.sh <project>`, which runs `cargo fmt --check`, `cargo clippy`, `cargo test` and project checks. A fresh machine catches "works on my laptop" problems (a file you forgot to commit), and a green check is what lets teammates trust and merge a change.

## Your tasks

Work in `projects/01-toolchain` (`cd projects/01-toolchain`).

1. **Check your tools.** Run `rustup show`, `cargo --version` and `bash ../../tools/doctor.sh`. Then `cargo run -- --help` builds and runs the program, and `cargo test` fails with `not yet implemented`: that's what you'll fix.
2. **Fill in the metadata.** In `Cargo.toml`, add `description`, `license` and `rust-version = "1.85"` (edition 2024 needs at least 1.85).
3. **Add a dependency.** Splitting text into words correctly (apostrophes, decimal numbers, accents) is harder than it looks, and the Unicode standard already defines it. Run `cargo add unicode-segmentation`, then look at what changed in `Cargo.toml` and `Cargo.lock`, and run `cargo tree`. Read its docs with `cargo doc --open`.
4. **Write the library.** Complete TODO 1 and TODO 2 in `src/lib.rs`:
   - `count_words`: `use unicode_segmentation::UnicodeSegmentation;` adds `unicode_words()` to `&str`. Lowercase each word with `to_lowercase()` and count with `*counts.entry(word).or_insert(0) += 1`.
   - `top_words`: collect the pairs into a `Vec`, sort with `sort_by` comparing counts in reverse and then words (`Ordering::then_with`), and `truncate(n)`.
5. **Unit tests.** Do TODO 3: at least 3 `#[test]` functions in `src/`, for example how `top_words` breaks ties.
6. **Format and lint.** `cargo fmt`, then `cargo clippy --all-targets`. `main.rs` was written in a hurry: fix every warning clippy reports (read the explanation link each time).
7. **Release build.** Add the `[profile.release]` section above. Run `cargo build` and `cargo build --release`, then compare `ls -lh ../../target/debug/wordfreq ../../target/release/wordfreq`. Try it: `cargo run --release -- --top 5 < README.md`.
8. **Grade it** like CI does: `bash ../../tools/grade.sh 01-toolchain`. Then commit, push, and watch the **Actions** tab turn green.

## What the grader checks

- `Cargo.toml` has a description, a license and a rust-version, depends on `unicode-segmentation`, and its release profile sets `lto` and `strip`,
- `tests/cli.rs` is unchanged: these integration tests run the real binary through `CARGO_BIN_EXE_wordfreq`, checking output, exit codes (0 success, 1 runtime error, 2 usage error) and that errors go to standard error,
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean,
- every test passes, and `src/` has at least 3 unit tests,
- the release build runs and prints the right answer.

## Hints

- `cargo test name_part` runs only the tests whose names contain `name_part`. Add `-- --nocapture` to see `println!` output.
- Reproduce a failing integration test by hand: `printf 'a b a' | cargo run -- --top 1`.
- `cargo clippy --fix --allow-dirty` applies the machine-fixable suggestions. Read the diff before committing.

## Going further

- Prove your `rust-version` is honest: `rustup toolchain install 1.85 && cargo +1.85 test`.
- Run `cargo run --release < big.txt | head -n 1` on a large file: `println!` panics when `head` closes the pipe. Write through `std::io::stdout().lock()` with `writeln!` and stop quietly on `ErrorKind::BrokenPipe`.
- Install your tool like any Rust user would: `cargo install --path .`, then run `wordfreq` from anywhere.
