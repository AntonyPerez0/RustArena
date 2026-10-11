//! Unit tests for `account.rs`. Being a child module of `account`, this file
//! can read a `Ledger`'s private fields (`balance`, `history`) directly.

use super::*;

#[test]
fn deposit_increases_the_balance() {
    let mut ledger = Ledger::new();
    ledger.deposit(Money::from_cents(500)).unwrap();
    assert_eq!(ledger.balance(), Money::from_cents(500));
}

// TODO: test every rule in the documentation of account.rs (see the README).
