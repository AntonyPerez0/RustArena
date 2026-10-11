//! Unit tests for `money.rs`. This file is the `tests` module declared at the
//! bottom of `money.rs`, a child of `money`, so it can use private items too:
//! the `cents` field and the `all_digits` helper.

use super::*;

fn m(cents: i64) -> Money {
    Money::from_cents(cents)
}

#[test]
fn from_cents_keeps_the_amount() {
    assert_eq!(Money::from_cents(1234).cents(), 1234);
    assert_eq!(Money::from_cents(-7).cents, -7);
}

#[test]
fn all_digits_needs_at_least_one_digit_and_nothing_else() {
    assert!(all_digits("0123456789"));
    assert!(!all_digits(""));
    assert!(!all_digits("1a"));
    assert!(!all_digits("-1"));
}

#[test]
fn parses_units_and_cents() {
    assert_eq!("12".parse(), Ok(m(1200)));
    assert_eq!("12.34".parse(), Ok(m(1234)));
    assert_eq!("0.05".parse(), Ok(m(5)));
    assert_eq!("-0.05".parse(), Ok(m(-5)));
    assert_eq!("-3".parse(), Ok(m(-300)));
}

#[test]
fn parse_needs_exactly_two_decimals() {
    for s in ["1.5", "1.234", "1.", ".50", "-.50", "1.5a"] {
        assert_eq!(s.parse::<Money>(), Err(ParseMoneyError::Invalid), "{s:?}");
    }
}

#[test]
fn parse_rejects_signs_spaces_and_separators() {
    for s in ["+1.00", " 1.00", "1.00 ", "1,000.00", "1e3", "--1", "1.0.0"] {
        assert_eq!(s.parse::<Money>(), Err(ParseMoneyError::Invalid), "{s:?}");
    }
    assert_eq!("".parse::<Money>(), Err(ParseMoneyError::Empty));
    assert_eq!("-".parse::<Money>(), Err(ParseMoneyError::Empty));
}

#[test]
fn parse_reports_amounts_out_of_range() {
    assert_eq!(
        "92233720368547758.08".parse::<Money>(),
        Err(ParseMoneyError::Overflow)
    );
    assert_eq!("-92233720368547758.08".parse(), Ok(m(i64::MIN)));
    assert_eq!(
        "99999999999999999999999999999999999999999".parse::<Money>(),
        Err(ParseMoneyError::Overflow)
    );
}

#[test]
fn displays_exactly_two_decimals() {
    assert_eq!(m(1234).to_string(), "12.34");
    assert_eq!(m(100).to_string(), "1.00");
    assert_eq!(m(5).to_string(), "0.05");
    assert_eq!(m(0).to_string(), "0.00");
    assert_eq!(m(-5).to_string(), "-0.05");
    assert_eq!(m(-99).to_string(), "-0.99");
    assert_eq!(m(-1234).to_string(), "-12.34");
}

#[test]
fn percentage_rounds_to_the_nearest_cent() {
    assert_eq!(m(10_000).percentage(150), Some(m(150)));
    assert_eq!(m(100).percentage(149), Some(m(1)));
    assert_eq!(m(100).percentage(151), Some(m(2)));
    assert_eq!(m(0).percentage(500), Some(m(0)));
    assert_eq!(m(100).percentage(0), Some(m(0)));
}

#[test]
fn percentage_rounds_exact_halves_away_from_zero() {
    assert_eq!(m(100).percentage(50), Some(m(1)));
    assert_eq!(m(100).percentage(150), Some(m(2)));
    assert_eq!(m(-100).percentage(50), Some(m(-1)));
}

#[test]
fn percentage_out_of_range_is_none() {
    assert_eq!(m(i64::MAX).percentage(20_000), None);
}

#[test]
fn split_spreads_the_remainder_over_the_first_parts() {
    assert_eq!(m(100).split(3), [m(34), m(33), m(33)]);
    assert_eq!(m(2).split(4), [m(1), m(1), m(0), m(0)]);
    assert_eq!(m(5).split(1), [m(5)]);
}

#[test]
fn split_works_for_negative_amounts() {
    assert_eq!(m(-100).split(3), [m(-33), m(-33), m(-34)]);
    assert_eq!(m(-1).split(2), [m(0), m(-1)]);
}

#[test]
#[should_panic(expected = "zero parts")]
fn split_into_zero_parts_panics() {
    m(100).split(0);
}

#[test]
fn checked_arithmetic_detects_overflow() {
    assert_eq!(m(1).checked_add(m(2)), Some(m(3)));
    assert_eq!(m(i64::MAX).checked_add(m(1)), None);
    assert_eq!(m(1).checked_sub(m(3)), Some(m(-2)));
    assert_eq!(m(i64::MIN).checked_sub(m(1)), None);
}
