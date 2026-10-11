//! [`Ledger`]: an account balance and the history of what changed it.
//!
//! This is code under test. Don't change it: the grader checks that it is
//! untouched, then plants bugs in it to see whether your tests notice.

use std::fmt;

use crate::money::Money;

/// One successful operation, as recorded in the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Deposit(Money),
    Withdrawal(Money),
    Interest(Money),
}

/// Why an operation was refused. A refused operation changes nothing: not the
/// balance, not the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerError {
    /// Deposits and withdrawals must be more than zero.
    NotPositive(Money),
    /// A withdrawal of more than the balance. There are no overdrafts.
    InsufficientFunds { balance: Money, requested: Money },
    /// The new balance would be out of range.
    Overflow,
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::NotPositive(amount) => write!(f, "{amount} is not a positive amount"),
            LedgerError::InsufficientFunds { balance, requested } => {
                write!(f, "can't withdraw {requested}: the balance is {balance}")
            }
            LedgerError::Overflow => f.write_str("the balance would be out of range"),
        }
    }
}

impl std::error::Error for LedgerError {}

/// An account: a balance that starts at zero and never goes below zero, and
/// the history of every successful operation.
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    balance: Money,
    history: Vec<Entry>,
}

impl Ledger {
    /// An empty account: a zero balance and no history.
    pub fn new() -> Ledger {
        Ledger::default()
    }

    /// The current balance.
    pub fn balance(&self) -> Money {
        self.balance
    }

    /// Adds `amount`, which must be more than zero.
    pub fn deposit(&mut self, amount: Money) -> Result<(), LedgerError> {
        if !amount.is_positive() {
            return Err(LedgerError::NotPositive(amount));
        }
        self.balance = self
            .balance
            .checked_add(amount)
            .ok_or(LedgerError::Overflow)?;
        self.history.push(Entry::Deposit(amount));
        Ok(())
    }

    /// Takes out `amount`, which must be more than zero and at most the balance.
    pub fn withdraw(&mut self, amount: Money) -> Result<(), LedgerError> {
        if !amount.is_positive() {
            return Err(LedgerError::NotPositive(amount));
        }
        if amount > self.balance {
            return Err(LedgerError::InsufficientFunds {
                balance: self.balance,
                requested: amount,
            });
        }
        self.balance = Money::from_cents(self.balance.cents() - amount.cents());
        self.history.push(Entry::Withdrawal(amount));
        Ok(())
    }

    /// Adds interest of `basis_points` hundredths of a percent of the balance,
    /// rounded like [`Money::percentage`], and returns the interest added.
    /// Interest that rounds to zero is not recorded in the history.
    pub fn apply_interest(&mut self, basis_points: u32) -> Result<Money, LedgerError> {
        let interest = self
            .balance
            .percentage(basis_points)
            .ok_or(LedgerError::Overflow)?;
        self.balance = self
            .balance
            .checked_add(interest)
            .ok_or(LedgerError::Overflow)?;
        if interest != Money::ZERO {
            self.history.push(Entry::Interest(interest));
        }
        Ok(interest)
    }

    /// The last `n` entries of the history (fewer if there aren't `n`), newest
    /// first.
    pub fn recent(&self, n: usize) -> Vec<Entry> {
        self.history.iter().rev().take(n).copied().collect()
    }

    /// How many entries the history has.
    pub fn operations(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests;
