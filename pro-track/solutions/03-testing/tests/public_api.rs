//! Integration tests: each file in tests/ is compiled as a separate crate that
//! uses `ledger` like any other user would, through its public API only.

use ledger::{Entry, Ledger, LedgerError, Money};

fn m(cents: i64) -> Money {
    Money::from_cents(cents)
}

#[test]
fn a_new_ledger_is_empty() {
    let ledger = Ledger::new();
    assert_eq!(ledger.balance(), Money::ZERO);
    assert_eq!(ledger.operations(), 0);
    assert!(ledger.recent(10).is_empty());
}

#[test]
fn recent_is_newest_first() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(100)).unwrap();
    ledger.withdraw(m(30)).unwrap();
    ledger.deposit(m(5)).unwrap();
    assert_eq!(
        ledger.recent(2),
        [Entry::Deposit(m(5)), Entry::Withdrawal(m(30))]
    );
    assert_eq!(ledger.recent(10).len(), 3);
    assert!(ledger.recent(0).is_empty());
    assert_eq!(ledger.operations(), 3);
}

#[test]
fn interest_is_rounded_added_and_recorded() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(100)).unwrap();
    assert_eq!(ledger.apply_interest(50), Ok(m(1)));
    assert_eq!(ledger.balance(), m(101));
    assert_eq!(ledger.recent(1), [Entry::Interest(m(1))]);
}

#[test]
fn interest_that_rounds_to_zero_is_not_recorded() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(10)).unwrap();
    assert_eq!(ledger.apply_interest(1), Ok(Money::ZERO));
    assert_eq!(ledger.balance(), m(10));
    assert_eq!(ledger.operations(), 1);
}

#[test]
fn a_refused_withdrawal_leaves_the_history_alone() {
    let mut ledger = Ledger::new();
    ledger.deposit(m(50)).unwrap();
    assert!(ledger.withdraw(m(80)).is_err());
    assert!(ledger.withdraw(m(0)).is_err());
    assert_eq!(ledger.recent(10), [Entry::Deposit(m(50))]);
}

#[test]
fn errors_explain_themselves() {
    let error = LedgerError::InsufficientFunds {
        balance: m(300),
        requested: m(450),
    };
    assert_eq!(
        error.to_string(),
        "can't withdraw 4.50: the balance is 3.00"
    );
    assert_eq!(
        LedgerError::NotPositive(m(-5)).to_string(),
        "-0.05 is not a positive amount"
    );
}

#[test]
fn amounts_round_trip_through_text() {
    for s in ["0.00", "0.01", "-0.01", "-0.99", "12.34", "-1000000.00"] {
        assert_eq!(s.parse::<Money>().unwrap().to_string(), s);
    }
}
