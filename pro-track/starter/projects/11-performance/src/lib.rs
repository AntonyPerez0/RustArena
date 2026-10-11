//! hotspots: four small functions that give the right answers, slowly.
//!
//! Each one is the kind of code that's fine on ten items and hopeless on a hundred
//! thousand. Measure them, find out why, and make them fast without changing a single
//! result. The tests in `tests/correctness.rs` pin down the exact behavior.

pub mod data;

pub use data::Row;

/// The distinct items, in the order they first appear.
/// `["b", "a", "b", "c", "a"]` gives `["b", "a", "c"]`.
pub fn unique_in_order(items: &[String]) -> Vec<String> {
    let mut unique: Vec<String> = Vec::new();
    for item in items {
        if !unique.contains(item) {
            unique.push(item.clone());
        }
    }
    unique
}

/// The `k` most common words in `text` with their counts: most common first, and words
/// with equal counts in alphabetical order. A word is a run of alphanumeric characters,
/// compared in lowercase ("Rust" and "rust" are the same word).
pub fn top_words(text: &str, k: usize) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        if word.is_empty() {
            continue;
        }
        let word = word.to_lowercase();
        match counts.iter().position(|(w, _)| *w == word) {
            Some(i) => counts[i].1 += 1,
            None => counts.push((word.clone(), 1)),
        }
        // Keep the list in order, so the answer is ready at the end.
        counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    }
    counts.truncate(k);
    counts
}

/// The rows as CSV: a `name,score` header line, then one `name,score` line per row.
pub fn to_csv(rows: &[Row]) -> String {
    let mut csv = String::from("name,score\n");
    for row in rows {
        let row = row.clone();
        csv = format!("{csv}{},{}\n", row.name, row.score);
    }
    csv
}

/// Competition ranking ("1224" ranking): each score's rank is 1 plus the number of
/// strictly higher scores, so equal scores share a rank. `[50, 80, 80, 20]` gives
/// `[3, 1, 1, 4]`.
pub fn ranks(scores: &[u32]) -> Vec<usize> {
    let mut ranks = Vec::new();
    for &score in scores {
        let mut sorted = scores.to_vec();
        sorted.sort_by(|a, b| b.cmp(a));
        let higher = sorted.iter().position(|&s| s == score).unwrap();
        ranks.push(higher + 1);
    }
    ranks
}
