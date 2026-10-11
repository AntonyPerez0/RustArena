//! [`Money`]: an amount stored as a whole number of cents.
//!
//! This is code under test. Don't change it: the grader checks that it is
//! untouched, then plants bugs in it to see whether your tests notice.

use std::fmt;
use std::str::FromStr;

/// An amount of money, stored as a whole number of cents so that arithmetic is
/// exact (in floating point, 0.1 + 0.2 is 0.30000000000000004).
///
/// Amounts can be negative: a refund, a debt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money {
    cents: i64,
}

/// Why a string is not a valid amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMoneyError {
    /// Nothing to parse (an empty string, or just `-`).
    Empty,
    /// Not of the form `123`, `123.45`, `-123` or `-123.45`.
    Invalid,
    /// Too large (or too small) to store.
    Overflow,
}

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ParseMoneyError::Empty => "no amount given",
            ParseMoneyError::Invalid => "expected an amount like 12.34",
            ParseMoneyError::Overflow => "amount out of range",
        })
    }
}

impl std::error::Error for ParseMoneyError {}

impl Money {
    /// No money at all.
    pub const ZERO: Money = Money { cents: 0 };

    /// An amount of `cents` cents: `Money::from_cents(1234)` is 12.34.
    pub const fn from_cents(cents: i64) -> Money {
        Money { cents }
    }

    /// This amount in cents.
    pub const fn cents(self) -> i64 {
        self.cents
    }

    /// True if this amount is more than zero.
    pub const fn is_positive(self) -> bool {
        self.cents > 0
    }

    /// `self + other`, or `None` if the result is out of range.
    pub fn checked_add(self, other: Money) -> Option<Money> {
        self.cents.checked_add(other.cents).map(Money::from_cents)
    }

    /// `self - other`, or `None` if the result is out of range.
    pub fn checked_sub(self, other: Money) -> Option<Money> {
        self.cents.checked_sub(other.cents).map(Money::from_cents)
    }

    /// `basis_points` hundredths of a percent of this amount (100 basis points
    /// are 1%), rounded to the nearest cent, with exact halves rounded away
    /// from zero. `None` if the result is out of range.
    pub fn percentage(self, basis_points: u32) -> Option<Money> {
        // In ten-thousandths of a cent. i128 can't overflow here.
        let exact = i128::from(self.cents) * i128::from(basis_points);
        let rounded = (exact.abs() + 5_000) / 10_000 * exact.signum();
        i64::try_from(rounded).ok().map(Money::from_cents)
    }

    /// Splits this amount into `parts` amounts that add up to exactly this
    /// amount and differ from each other by at most one cent, larger ones
    /// first: 1.00 split 3 ways is 0.34, 0.33, 0.33.
    ///
    /// # Panics
    ///
    /// If `parts` is zero.
    pub fn split(self, parts: usize) -> Vec<Money> {
        assert!(parts > 0, "can't split money into zero parts");
        let n = i64::try_from(parts).expect("too many parts");
        let base = self.cents.div_euclid(n);
        let extra = self.cents.rem_euclid(n) as usize;
        (0..parts)
            .map(|i| Money::from_cents(if i < extra { base + 1 } else { base }))
            .collect()
    }
}

/// Formats as units and exactly two decimals: `12.34`, `-0.05`, `0.00`.
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.cents < 0 { "-" } else { "" };
        let abs = self.cents.unsigned_abs();
        write!(f, "{sign}{}.{:02}", abs / 100, abs % 100)
    }
}

/// Parses `123`, `123.45`, `-123` or `-123.45`: digits, then optionally a
/// point and exactly two digits. No `+`, spaces or thousands separators.
impl FromStr for Money {
    type Err = ParseMoneyError;

    fn from_str(s: &str) -> Result<Money, ParseMoneyError> {
        let (negative, rest) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        if rest.is_empty() {
            return Err(ParseMoneyError::Empty);
        }
        let (whole, fraction) = rest.split_once('.').unwrap_or((rest, "00"));
        if !all_digits(whole) || !all_digits(fraction) || fraction.len() != 2 {
            return Err(ParseMoneyError::Invalid);
        }
        let whole: i128 = whole.parse().map_err(|_| ParseMoneyError::Overflow)?;
        let fraction: i128 = fraction.parse().map_err(|_| ParseMoneyError::Invalid)?;
        let cents = whole
            .checked_mul(100)
            .and_then(|c| c.checked_add(fraction))
            .ok_or(ParseMoneyError::Overflow)?;
        let cents = if negative { -cents } else { cents };
        i64::try_from(cents)
            .map(Money::from_cents)
            .map_err(|_| ParseMoneyError::Overflow)
    }
}

/// True if `s` is one or more ASCII digits.
fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests;
