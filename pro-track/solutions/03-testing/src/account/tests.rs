//! Unit tests for `account.rs`. Being a child module of `account`, this file
//! can read a `Ledger`'s private fields (`balance`, `history`) directly.

use super::*;

fn m(cents: i64) -> Money {
    Money::from_cents(cents)
}

#[test]
fn deposit_increases_the_balance() {
    let mut ledger = Ledger::new();
    ledger.deposit(Money::from_cents(500)).unwrap();
    assert_eq!(ledger.balance(), Money::from_cents(500));
    assert_eq!(ledger.history, [Entry::Deposit(m(500))]);
}

#[test]
fn deposit_refuses_zero_and_negative_amounts() {
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.deposit(Money::ZERO),
        Err(LedgerError::NotPositive(Money::ZERO))
    );
    assert_eq!(ledger.deposit(m(-5)), Err(LedgerError::NotPositive(m(-5))));
    assert_eq!(ledger.balance, Money::ZERO);
    assert!(ledger.history.is_empty());
}

#[test]
fn deposit_refuses_overflow_and_changes_nothing() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(i64::MAX)).unwrap();
    assert_eq!(ledger.deposit(m(1)), Err(LedgerError::Overflow));
    assert_eq!(ledger.balance, m(i64::MAX));
    assert_eq!(ledger.history.len(), 1);
}

#[test]
fn withdraw_can_take_the_whole_balance() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(300)).unwrap();
    assert_eq!(ledger.withdraw(m(300)), Ok(()));
    assert_eq!(ledger.balance, Money::ZERO);
    assert_eq!(ledger.history.last(), Some(&Entry::Withdrawal(m(300))));
}

#[test]
fn withdraw_refuses_overdrafts_and_changes_nothing() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(300)).unwrap();
    assert_eq!(
        ledger.withdraw(m(301)),
        Err(LedgerError::InsufficientFunds {
            balance: m(300),
            requested: m(301)
        })
    );
    assert_eq!(ledger.balance, m(300));
    assert_eq!(ledger.history, [Entry::Deposit(m(300))]);
}

#[test]
fn withdraw_refuses_zero_and_negative_amounts() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(100)).unwrap();
    assert_eq!(
        ledger.withdraw(Money::ZERO),
        Err(LedgerError::NotPositive(Money::ZERO))
    );
    assert_eq!(
        ledger.withdraw(m(-50)),
        Err(LedgerError::NotPositive(m(-50)))
    );
    assert_eq!(ledger.balance, m(100));
    assert_eq!(ledger.history.len(), 1);
}
