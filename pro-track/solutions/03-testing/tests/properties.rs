//! Property-based tests: proptest generates hundreds of random inputs for each
//! property, and shrinks any failure down to the simplest input that breaks it.

use ledger::{Entry, Ledger, Money};
use proptest::prelude::*;

proptest! {
    #[test]
    fn display_then_parse_gives_back_the_same_amount(cents in any::<i64>()) {
        let money = Money::from_cents(cents);
        prop_assert_eq!(money.to_string().parse::<Money>(), Ok(money));
    }

    // any::<i64>() almost never picks a small number, and small numbers are
    // where the formatting edge cases are.
    #[test]
    fn small_amounts_round_trip_too(cents in -10_000i64..10_000) {
        let money = Money::from_cents(cents);
        prop_assert_eq!(money.to_string().parse::<Money>(), Ok(money));
    }

    #[test]
    fn split_adds_up_and_is_fair(cents in -1_000_000_000i64..1_000_000_000, parts in 1usize..50) {
        let pieces = Money::from_cents(cents).split(parts);
        prop_assert_eq!(pieces.len(), parts);
        prop_assert_eq!(pieces.iter().map(|p| p.cents()).sum::<i64>(), cents);
        prop_assert!(pieces.windows(2).all(|w| w[0] >= w[1]), "larger parts first");
        prop_assert!(pieces[0].cents() - pieces[parts - 1].cents() <= 1);
    }

    #[test]
    fn percentage_is_within_half_a_cent(cents in -1_000_000_000i64..1_000_000_000, basis_points in 0u32..100_000) {
        let exact = i128::from(cents) * i128::from(basis_points);
        let got = Money::from_cents(cents).percentage(basis_points).unwrap();
        prop_assert!((i128::from(got.cents()) * 10_000 - exact).abs() <= 5_000);
    }

    #[test]
    fn the_balance_never_goes_negative_and_matches_the_history(
        operations in prop::collection::vec((0u8..3, -100i64..10_000), 0..40)
    ) {
        let mut ledger = Ledger::new();
        for (kind, cents) in operations {
            let amount = Money::from_cents(cents);
            let _ = match kind {
                0 => ledger.deposit(amount),
                1 => ledger.withdraw(amount),
                _ => ledger.apply_interest(cents.unsigned_abs() as u32 % 1_000).map(|_| ()),
            };
            prop_assert!(ledger.balance() >= Money::ZERO);
        }
        let from_history: i64 = ledger
            .recent(usize::MAX)
            .iter()
            .map(|entry| match entry {
                Entry::Deposit(m) | Entry::Interest(m) => m.cents(),
                Entry::Withdrawal(m) => -m.cents(),
            })
            .sum();
        prop_assert_eq!(from_history, ledger.balance().cents());
        prop_assert_eq!(ledger.recent(usize::MAX).len(), ledger.operations());
    }
}
