# 05 · Designing a library crate

Most Rust code at work lives in libraries that other code depends on. A library's public API is a promise: once someone builds on it, every name, signature and error variant is hard to change. So teams work to get it right: types that make mistakes impossible, errors callers can act on, documentation whose examples are tested, and version numbers that tell the truth.

You'll finish **semver-lite**, a library that parses, compares and matches [semantic versions](https://semver.org) (the `^1.2` rules Cargo itself uses), and a small `semver` command-line tool built on it, in one Cargo workspace.

**You'll practice:** workspaces, API design (newtypes, builders, `impl Trait`, `AsRef`), typed errors with `thiserror`, rustdoc and doc tests, cargo features, semver, and packaging for crates.io.

## The workspace

```
05-library/
  Cargo.toml        [workspace]: the members, plus settings and dependencies they share
  semver-lite/      the library (the product)
  semver-cli/       the `semver` binary: reads arguments, calls the library, prints
```

A workspace builds several packages with one `Cargo.lock` and one `target/` folder. Real projects (ripgrep, tokio, serde) split this way so the library can be tested and reused without the binary, and the binary stays thin. Shared settings go in `[workspace.package]` and `[workspace.dependencies]`, and members inherit them with `edition.workspace = true`.

```bash
cd projects/05-library
cargo build --workspace            # everything
cargo test -p semver-lite          # one member (-p = --package)
cargo run -p semver-cli -- sort 1.10.0 1.2.0 v1.2.0-rc.1
```

The CLI depends on the library with `path = "semver-lite", version = "0.1.0"`: the path is used here, and the version is what the dependency becomes if the CLI is ever published (crates.io doesn't allow path-only dependencies).

## Background

### API design: make the right thing easy

- **Re-export from `lib.rs`.** Users write `semver_lite::Version`, never `semver_lite::version::Version`. The module layout stays private, so you can reorganize it without breaking anyone.
- **Newtypes.** `Prerelease` wraps a `String` in a struct with a private field. The only way to make one is `Prerelease::new`, which validates, so every `Prerelease` in existence is valid, and its ordering rule lives with the data. A plain `String` would let anyone build `"rc..1"`.
- **Standard traits.** Users expect `"1.2.3".parse()` (`FromStr`), `println!("{v}")` (`Display`), `versions.sort()` (`Ord`), plus `Debug`, `Clone`, `PartialEq`, `Hash`. They can't add them from outside your crate.
- **Builders.** `Parser::new().allow_v_prefix(true).allow_partial(true)` reads well at the call site, unlike `parse(s, true, false)`, and adding an option later breaks nobody.
- **`impl Trait` and `AsRef`.** `parse_all(inputs: impl IntoIterator<Item = impl AsRef<str>>)` accepts `["1.0.0"]`, a `Vec<String>`, or `&args` without the caller converting anything. Returning `impl Iterator` (see `VersionReq::matching`) lets callers chain or collect without the crate committing to a collection type.

### Typed errors with thiserror

Applications report errors to people, so `anyhow` (project 04) is fine there. Libraries report errors to *code*, so they return an enum the caller can `match` on:

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid version number \"{part}\"")]
    InvalidNumber { part: String },
    #[error("invalid version requirement \"{req}\"")]
    InvalidReq { req: String, source: Box<Error> },   // a field named `source` is the cause
}
```

`thiserror` writes the `Display` and `std::error::Error` impls, including `source()`, so a program can print the whole chain (try `semver match "^1.x" 1.0.0`). `#[non_exhaustive]` makes users add a `_` arm when they match, so a new variant later isn't a breaking change.

### Documentation is part of the API

`///` documents the next item and `//!` the enclosing one (the crate, at the top of `lib.rs`). Code blocks in docs are **compiled and run** by `cargo test`, so examples can't silently rot. Inside them, lines starting with `# ` are hidden from the rendered page; `# Ok::<(), semver_lite::Error>(())` at the end lets an example use `?`. Conventional sections are `# Examples`, `# Errors` (when a function returns `Result`) and `# Panics`, and `` [`Version`] `` links to an item. `#![deny(missing_docs)]` at the top of `lib.rs` makes any undocumented public item a compile error. Read the result with `cargo doc --open`.

### Optional features

Not every user wants serde. An optional dependency plus a feature lets them choose:

```toml
[features]
serde = ["dep:serde"]

[dependencies]
serde = { version = "1", optional = true }
```

Code that needs it goes behind `#[cfg(feature = "serde")]` (a whole module is easiest). Features must be **additive**: turning one on may add API, never remove or change it, because Cargo unifies features across the whole dependency graph. Test both ways (`cargo test` and `cargo test --all-features`), as the grader does.

### Semver for your own crate

`MAJOR.MINOR.PATCH`: bump PATCH for fixes, MINOR for new API, MAJOR for breaking changes. Breaking includes removing or renaming a public item, changing a signature, adding a variant to an exhaustive enum, and adding a field to a struct with public fields (it breaks struct literals and patterns). Before 1.0, Cargo treats the first non-zero number as the major one: `^0.2.3` means `>=0.2.3, <0.3.0`. You are about to implement exactly that rule.

### Ready for crates.io

crates.io needs a `description`, a `license` (an SPDX expression like `MIT OR Apache-2.0`) and a link (`repository`). Good crates also set `readme`, `keywords`, `categories` and `rust-version` (the oldest Rust they support). `cargo package --list` shows exactly which files would be uploaded, and `cargo package` builds the `.crate` and compiles it in isolation, which catches files you forgot to include. **Don't run `cargo publish` for this project**: published versions are permanent.

## Your tasks

1. **Errors** (`semver-lite/src/error.rs`): derive `thiserror::Error`, write the messages listed in the comments, add `#[non_exhaustive]`.
2. **Prerelease** (`prerelease.rs`): `new` (validation) and `Ord` (precedence).
3. **Version and Parser** (`version.rs`, `parser.rs`): parsing, ordering, `Display`, bumps. Now `cargo test --workspace` should pass, CLI included.
4. **Documentation:** add `#![deny(missing_docs)]` and crate docs to `lib.rs`, then document every public item, including `req.rs` (written by a colleague, undocumented). At least 6 examples in total; `cargo test --doc -p semver-lite` runs them.
5. **The `serde` feature:** add it to `semver-lite/Cargo.toml` and implement `Serialize`/`Deserialize` for `Version` *as a string* in a `#[cfg(feature = "serde")]` module. Run `cargo test -p semver-lite --features serde`.
6. **Metadata:** fill in `[workspace.package]` and `semver-lite/Cargo.toml`, write `semver-lite/README.md` (what crates.io shows), then run `cargo package -p semver-lite --list` and `cargo package -p semver-lite --allow-dirty`.
7. Grade (`bash ../../tools/grade.sh 05-library`), then commit and push.

## What the grader checks

- `cargo fmt` and `cargo clippy --all-features` are clean; the provided tests are unchanged.
- `Error` uses thiserror and is `#[non_exhaustive]`; `lib.rs` has `#![deny(missing_docs)]`.
- Tests pass without features and with `serde`; serde isn't compiled when the feature is off.
- At least 6 doc tests; `cargo doc` passes with `RUSTDOCFLAGS="-D warnings"` (no broken links).
- `cargo package` succeeds with no warnings and includes `README.md`.

## Hints

- Since `Prerelease` knows its own ordering, `Version` can simply `#[derive(PartialOrd, Ord)]`: derived ordering compares fields in declaration order.
- `u64::from_str` accepts `"+2"`. Check that every byte is an ASCII digit first.
- Pre-release numbers have no leading zeros, so a longer number is a bigger number: compare lengths, then text. No overflow possible.
- For serde, `serializer.collect_str(self)` writes your `Display` output; for deserializing, read a `String` and `parse()` it, turning the error into `serde::de::Error::custom`.
- `#![warn(clippy::missing_errors_doc)]` reminds you to write `# Errors` sections.

## Going further

- Install [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks), make a breaking change (add a public field to `Version`), and watch it catch it.
- Use the README as the crate docs with `#![doc = include_str!("../README.md")]`, so its examples become doc tests too.
- Support build metadata (`1.0.0+build.5`), which is ignored when comparing versions.
- Run `cargo publish --dry-run -p semver-lite` to see the last checks crates.io would do, without uploading.
