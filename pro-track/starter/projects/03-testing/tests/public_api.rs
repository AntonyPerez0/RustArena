//! Integration tests: each file in tests/ is compiled as a separate crate that
//! uses `ledger` like any other user would, through its public API only.

use ledger::{Ledger, Money};

#[test]
fn a_new_ledger_is_empty() {
    let ledger = Ledger::new();
    assert_eq!(ledger.balance(), Money::ZERO);
    assert_eq!(ledger.operations(), 0);
}

// TODO: more integration tests, and property tests with proptest (see the README).
