//! hotspots: four small functions that give the right answers, slowly.
//!
//! Each one is the kind of code that's fine on ten items and hopeless on a hundred
//! thousand. Measure them, find out why, and make them fast without changing a single
//! result. The tests in `tests/correctness.rs` pin down the exact behavior.

pub mod data;

use std::collections::{HashMap, HashSet};
use std::fmt::Write;

pub use data::Row;

/// The distinct items, in the order they first appear.
/// `["b", "a", "b", "c", "a"]` gives `["b", "a", "c"]`.
pub fn unique_in_order(items: &[String]) -> Vec<String> {
    // Was: Vec::contains, a linear scan per item, so O(n * unique).
    // A HashSet answers "seen it?" in O(1). It borrows the strings: no copies until the end.
    let mut seen: HashSet<&str> = HashSet::with_capacity(items.len());
    items
        .iter()
        .filter(|item| seen.insert(item.as_str()))
        .cloned()
        .collect()
}

/// The `k` most common words in `text` with their counts: most common first, and words
/// with equal counts in alphabetical order. A word is a run of alphanumeric characters,
/// compared in lowercase ("Rust" and "rust" are the same word).
pub fn top_words(text: &str, k: usize) -> Vec<(String, usize)> {
    // Was: a linear search in a Vec for every word, and a full sort after every word.
    // Count in a HashMap, then sort once at the end.
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        if word.is_empty() {
            continue;
        }
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    let mut counts: Vec<(String, usize)> = counts.into_iter().collect();
    counts.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    counts.truncate(k);
    counts
}

/// The rows as CSV: a `name,score` header line, then one `name,score` line per row.
pub fn to_csv(rows: &[Row]) -> String {
    // Was: format!("{csv}...") copied everything written so far for every row: O(n^2)
    // bytes. Append in place instead, into a buffer sized up front, without cloning rows.
    let mut csv = String::with_capacity(16 + rows.len() * 20);
    csv.push_str("name,score\n");
    for row in rows {
        writeln!(csv, "{},{}", row.name, row.score).expect("writing to a String can't fail");
    }
    csv
}

/// Competition ranking ("1224" ranking): each score's rank is 1 plus the number of
/// strictly higher scores, so equal scores share a rank. `[50, 80, 80, 20]` gives
/// `[3, 1, 1, 4]`.
pub fn ranks(scores: &[u32]) -> Vec<usize> {
    // Was: a copy and a full sort for every score, O(n^2 log n).
    // Sort once (highest first), then binary-search each score: O(n log n).
    let mut sorted = scores.to_vec();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    scores
        .iter()
        .map(|&score| sorted.partition_point(|&s| s > score) + 1)
        .collect()
}
