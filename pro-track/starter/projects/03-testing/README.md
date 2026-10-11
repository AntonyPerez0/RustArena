# 03 · Testing like a team

On the Arena site the tests were written for you. At work, **you** write them: every change ships with tests, and code review asks "where's the test?" In this project you write the whole test suite for a small money-handling library, using every kind of test Rust has, and the grader checks whether your tests would actually catch bugs.

**How it's graded: mutation testing.** Coverage only tells you a line *ran*, not that a test would notice if it were wrong. So the grader makes 8 **mutants**: copies of the library, each with one small, realistic bug planted (an off-by-one, a lost minus sign, the wrong rounding...). Your tests must pass on the real library and **fail on every mutant**. A mutant that survives means some rule in the documentation has no test.

**You'll practice:** unit tests, integration tests, doc tests, property-based tests with proptest, edge cases, and thinking like a tester.

## Background

### The library

`src/money.rs` defines `Money`, an amount stored as whole cents (never floating point, which can't represent 0.10 exactly), with parsing, formatting, percentages and fair splitting. `src/account.rs` defines `Ledger`, a balance with a history. **Don't change these two files**: the grader checks they're untouched. Read their documentation comments carefully: every sentence is a rule to test.

### Unit tests

Unit tests live inside the crate, so they can reach **private** items. Usually they sit at the bottom of the file in `#[cfg(test)] mod tests { ... }`. Here the library files end with `#[cfg(test)] mod tests;`, which loads the module from `src/money/tests.rs` and `src/account/tests.rs`: the same thing, kept in a separate file (big projects do this too). `#[cfg(test)]` means the code is only compiled for `cargo test`.

```rust
use super::*; // everything in the parent module, private items included

#[test]
fn withdraw_refuses_overdrafts() {
    let mut ledger = Ledger::new();                       // arrange
    ledger.deposit(Money::from_cents(300)).unwrap();
    let result = ledger.withdraw(Money::from_cents(301)); // act
    assert!(result.is_err());                             // assert
    assert_eq!(ledger.balance(), Money::from_cents(300));
}
```

Your tools: `assert!(cond)`, `assert_eq!(left, right)` and `assert_ne!` (both values are printed on failure), an optional message (`assert!(x > 0, "x was {x}")`), `matches!(value, Pattern { .. })`, and `#[should_panic(expected = "zero parts")]` for code that must panic.

### Integration tests

Every file in `tests/` is compiled as a **separate crate** that uses yours through its public API, exactly like a user would: `use ledger::{Ledger, Money};`. They catch a different class of mistakes: a missing `pub`, an awkward API, parts that work alone but not together.

### Doc tests

Code blocks in `///` and `//!` documentation are compiled and run by `cargo test`, so examples in the docs can never go stale. Lines starting with `# ` run but are hidden from the rendered docs:

```rust
//! ```
//! use ledger::Money;
//! let price: Money = "12.50".parse()?;
//! assert_eq!(price.cents(), 1250);
//! # Ok::<(), ledger::ParseMoneyError>(())
//! ```
```

### Property-based tests

An example test checks one input you thought of. A **property** is a rule that holds for *every* input, and proptest checks it against hundreds of random ones. When one fails, proptest **shrinks** it to the simplest input that still fails, and saves it in `proptest-regressions/` (commit that folder: the case is then retried on every run).

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn split_never_loses_a_cent(cents in -1_000_000i64..1_000_000, parts in 1usize..20) {
        let total: i64 = Money::from_cents(cents).split(parts).iter().map(|m| m.cents()).sum();
        prop_assert_eq!(total, cents);
    }
}
```

Good properties: round trips (`parse(format(x)) == x`), invariants (a balance is never negative), and comparisons with a simpler model. Choose ranges with care: `any::<i64>()` almost never produces a small number like -5, which is exactly where formatting bugs hide.

### What makes a good test

- **One behavior per test**, named after it: `withdraw_refuses_overdrafts`, not `test2`.
- **Check the side effects too.** A refused operation must change *nothing*: check the balance **and** the history afterwards.
- **Edge cases are where bugs live**: zero, negative, one, exactly the limit, one past it, rounding halves, empty.
- Read the docs one sentence at a time and ask: "which test fails if this sentence is violated?"

### Running tests

```bash
cargo test                       # everything: unit, integration, doc tests
cargo test split                 # only tests whose name contains "split"
cargo test --lib                 # only unit tests
cargo test --test properties     # only tests/properties.rs
cargo test --doc                 # only doc tests
cargo test -- --nocapture        # show println! output
```

## Your tasks

Work in `projects/03-testing`.

1. Read the documentation in `src/money.rs` and `src/account.rs`. List the rules.
2. **Unit tests** in `src/money/tests.rs` and `src/account/tests.rs`. Test the private helper `all_digits` directly, and read private fields like `ledger.history` to check that refused operations leave them alone.
3. **Integration tests** in `tests/` (add files as you like, e.g. `tests/ledger.rs`).
4. **A doc test**: add an `# Examples` section to the crate documentation at the top of `src/lib.rs`.
5. **Property tests** with proptest (already in `[dev-dependencies]`), e.g. in `tests/properties.rs`: round trips, `split` adding up, the balance never going negative.
6. Grade it: `bash ../../tools/grade.sh 03-testing`. Each surviving mutant tells you which rule is untested. Add tests until all 8 die, then commit and push.

## What the grader checks

- `src/money.rs`, `src/account.rs` and `mutants/` are unchanged,
- `cargo fmt --check` and clippy are clean, and every test passes on the real library,
- at least 15 tests, including at least 3 unit tests, 3 integration tests and 1 doc test in `src/lib.rs`,
- at least one `proptest!` block,
- each mutant in `mutants/` makes at least one of your tests fail. Property tests run with a fixed seed (`PROPTEST_RNG_SEED`) while grading, so results are the same every time.

## Hints

- "Rounded to the nearest cent, halves away from zero" is only observable when the exact result has a fraction. Pick numbers that make it exactly half a cent.
- "Newest first" needs at least two *different* entries to be observable.
- If a mutant survives and you can't see why, open its patch in `mutants/`: it shows the exact change, like a real mutation testing tool would. Then write the test that would have caught it.

## Going further

- Run a real mutation testing tool on your crate: `cargo install cargo-mutants && cargo mutants`. It invents mutants for you.
- Measure coverage with `cargo llvm-cov` and see that 100% line coverage still lets bugs through.
- Write a `transfer(&mut self, to: &mut Ledger, amount: Money)` test-first: tests, then the code.
