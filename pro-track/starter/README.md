# RustArena · Pro Track

This repository is your workspace for the **Pro Track**: twelve projects that take you from "I know Rust" to "I can work in a professional Rust codebase". You build real software with the tools Rust teams use every day: rustup and Cargo, Git and pull requests, rustfmt and clippy, unit, integration and property tests, clap, Tokio, axum, SQLx, tracing, criterion, Docker and CI.

Every project is graded automatically by **GitHub Actions** each time you push, exactly like a team's CI.

## Getting started

1. Use **GitHub Codespaces** (a full Linux development environment in your browser) or your own machine (Linux, macOS, or Windows with WSL).
   - Codespaces: on this repository's page, **Code > Codespaces > Create codespace on main**. Rust and the tools install themselves (a few minutes the first time).
   - Your own machine: `bash tools/setup.sh` (installs Rust with rustup if you don't have it).
2. Check your tools: `bash tools/doctor.sh`
3. Open `projects/01-toolchain/README.md` and start.

## The projects

| # | Project | You'll learn |
|---|---|---|
| 01 | [rustup, Cargo and CI](projects/01-toolchain/README.md) | rustup, Cargo.toml, profiles, rustfmt, clippy, CI |
| 02 | [Git and pull requests](projects/02-git/README.md) | branches, PRs, merge conflicts, revert, tags |
| 03 | [Testing like a team](projects/03-testing/README.md) | unit, integration and doc tests, proptest, mutation testing |
| 04 | [A command-line tool with clap](projects/04-cli/README.md) | clap derive, subcommands, anyhow, exit codes, assert_cmd |
| 05 | [Designing a library crate](projects/05-library/README.md) | workspaces, API design, thiserror, rustdoc, features, semver |
| 06 | [Concurrency](projects/06-concurrency/README.md) | threads, channels, Arc and Mutex, rayon |
| 07 | [Async networking with Tokio](projects/07-async/README.md) | TCP servers, tasks, select!, graceful shutdown |
| 08 | [A REST API with axum](projects/08-web-api/README.md) | routing, extractors, JSON, error responses, tower tests |
| 09 | [Persistence with SQLx](projects/09-database/README.md) | migrations, queries, transactions, repositories |
| 10 | [Logging, tracing and configuration](projects/10-observability/README.md) | tracing, spans, EnvFilter, configuration |
| 11 | [Benchmarks and profiling](projects/11-performance/README.md) | criterion, flamegraphs, allocations |
| 12 | [Capstone: a production-ready service](projects/12-capstone/README.md) | everything, plus Docker and CI |

Do them in order: later projects assume earlier skills.

## How grading works

- Grade one project locally, exactly as CI does: `bash tools/grade.sh 04-cli`
- On every push, the **Grade** workflow grades each project you've changed since importing the starter. Open the **Actions** tab: each project gets its own green or red check, and the log shows which step failed.
- To grade every project: **Actions > Grade > Run workflow**, with "all" ticked.
- Every grader checks formatting (`cargo fmt --check`) and lints (`cargo clippy -- -D warnings`) as well as tests, because teams' CI does.

Keep your repository **public** if you can. GitHub Actions minutes are free and unlimited for public repositories, and a public repo doubles as a portfolio for job applications.

## Layout

```
projects/NN-name/     one folder per project: README.md, a Cargo package, tests, grade.sh
tools/                setup, doctor and grading scripts
.cargo/config.toml    every project shares one target/ folder, so dependencies build once
.github/workflows/    the grading workflow
```
