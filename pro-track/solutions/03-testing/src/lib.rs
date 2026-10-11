//! Money handled safely: amounts are whole cents, never floating point, and an
//! account [`Ledger`] refuses anything that would make its balance wrong.
//!
//! The code under test is in `money.rs` and `account.rs`. Don't change those
//! files. Your tests go in `src/money/tests.rs`, `src/account/tests.rs`, in
//! `tests/`, and in this file's documentation.
//!
//! # Examples
//!
//! ```
//! use ledger::{Ledger, Money};
//!
//! let mut account = Ledger::new();
//! account.deposit("100.00".parse()?)?;
//! account.withdraw(Money::from_cents(2_550))?;
//! assert_eq!(account.balance().to_string(), "74.50");
//!
//! // Splitting a bill never loses a cent.
//! let shares = Money::from_cents(1_000).split(3);
//! assert_eq!(shares, [Money::from_cents(334), Money::from_cents(333), Money::from_cents(333)]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A refused operation changes nothing:
//!
//! ```
//! use ledger::{Ledger, LedgerError, Money};
//!
//! let mut account = Ledger::new();
//! let error = account.withdraw(Money::from_cents(1)).unwrap_err();
//! assert!(matches!(error, LedgerError::InsufficientFunds { .. }));
//! assert_eq!(account.operations(), 0);
//! ```

pub mod account;
pub mod money;

pub use account::{Entry, Ledger, LedgerError};
pub use money::{Money, ParseMoneyError};
