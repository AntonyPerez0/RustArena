//! Unit tests for `money.rs`. This file is the `tests` module declared at the
//! bottom of `money.rs`, a child of `money`, so it can use private items too:
//! the `cents` field and the `all_digits` helper.

use super::*;

#[test]
fn from_cents_keeps_the_amount() {
    assert_eq!(Money::from_cents(1234).cents(), 1234);
}

// TODO: test every rule in the documentation of money.rs (see the README).
