//! Time budgets, measured in an optimized build:
//!
//!   cargo test --release --test budget -- --ignored --test-threads=1 --nocapture
//!
//! Provided: don't change this file (the grader checks that). The tests are `#[ignore]`d,
//! so a plain `cargo test` (an unoptimized debug build) skips them. One at a time
//! (`--test-threads=1`) keeps them from slowing each other down.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use hotspots::{data, ranks, to_csv, top_words, unique_in_order};

/// Generous on purpose: a good solution takes a few milliseconds to a few tens of
/// milliseconds, even on a busy CI machine. The original code takes many seconds.
const BUDGET: Duration = Duration::from_secs(1);

/// Runs `work` on another thread and fails as soon as it overruns the budget, instead of
/// waiting for slow code to finish.
fn within_budget<T: Send + 'static>(name: &str, work: impl FnOnce() -> T + Send + 'static) -> T {
    if cfg!(debug_assertions) {
        panic!(
            "time budgets only mean something in an optimized build. \
             Run: cargo test --release --test budget -- --ignored --test-threads=1"
        );
    }
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let start = Instant::now();
        let result = work();
        let _ = tx.send((result, start.elapsed()));
    });
    match rx.recv_timeout(BUDGET) {
        Ok((result, took)) => {
            println!("{name}: {took:.1?} (budget {BUDGET:?})");
            result
        }
        Err(_) => panic!("{name} was still running after its {BUDGET:?} budget. Profile it!"),
    }
}

/// A fingerprint of some bytes (64-bit FNV-1a), to compare big results cheaply.
fn fingerprint(bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[test]
#[ignore = "time budget: needs --release"]
fn unique_in_order_150k_names() {
    let names = data::names(150_000, 90_000, 1);
    let unique = within_budget("unique_in_order", move || unique_in_order(&names));
    assert_eq!(unique.len(), 72_797);
    assert_eq!(unique[..3], ["user52410", "user04487", "user18712"]);
    let lines = unique.iter().flat_map(|s| s.bytes().chain(*b"\n"));
    assert_eq!(
        fingerprint(lines),
        0x5172_80c0_5580_161b,
        "right length, wrong items or order"
    );
}

#[test]
#[ignore = "time budget: needs --release"]
fn top_words_150k_words() {
    let text = data::text(150_000, 20_000, 2);
    let top = within_budget("top_words", move || top_words(&text, 10));
    let expected = [
        ("a", 88),
        ("b", 84),
        ("c", 77),
        ("d", 75),
        ("j", 71),
        ("l", 62),
        ("q", 62),
        ("h", 61),
        ("ak", 60),
        ("g", 60),
    ];
    let expected: Vec<(String, usize)> =
        expected.iter().map(|&(w, n)| (w.to_string(), n)).collect();
    assert_eq!(top, expected);
}

#[test]
#[ignore = "time budget: needs --release"]
fn to_csv_80k_rows() {
    let rows = data::rows(80_000, 3);
    let csv = within_budget("to_csv", move || to_csv(&rows));
    assert!(
        csv.starts_with("name,score\navl0,756355\nfgw1,555655\n"),
        "{}",
        &csv[..40]
    );
    assert_eq!(csv.len(), 1_248_066);
    assert_eq!(
        fingerprint(csv.bytes()),
        0x1b37_176b_975d_b0ab,
        "right length, wrong content"
    );
}

#[test]
#[ignore = "time budget: needs --release"]
fn ranks_30k_scores() {
    let scores = data::scores(30_000, 1_000_000, 4);
    let ranks = within_budget("ranks", move || ranks(&scores));
    assert_eq!(ranks.len(), 30_000);
    assert_eq!(ranks[..5], [7206, 26190, 28365, 13530, 8369]);
    let bytes = ranks.iter().flat_map(|&r| (r as u64).to_le_bytes());
    assert_eq!(
        fingerprint(bytes),
        0x91fe_7d97_4235_350c,
        "right length, wrong ranks"
    );
}
