//! Money handled safely: amounts are whole cents, never floating point, and an
//! account [`Ledger`] refuses anything that would make its balance wrong.
//!
//! The code under test is in `money.rs` and `account.rs`. Don't change those
//! files. Your tests go in `src/money/tests.rs`, `src/account/tests.rs`, in
//! `tests/`, and in this file's documentation.
//
// TODO: add an "# Examples" section to the documentation above, with at least
// one example that `cargo test` runs as a doc test (see the README).

pub mod account;
pub mod money;

pub use account::{Entry, Ledger, LedgerError};
pub use money::{Money, ParseMoneyError};
