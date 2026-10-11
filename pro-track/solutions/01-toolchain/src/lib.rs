//! Word counting for the `wordfreq` command-line tool.
//!
//! `main.rs` only deals with arguments, input and output. The real work lives
//! here, in a library, so it can be unit tested without starting a process.

use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

/// Counts how many times each word appears in `text`.
///
/// A word is what the `unicode-segmentation` crate's `unicode_words()` returns
/// (so `don't` and `3.14` are single words and punctuation is skipped),
/// converted to lowercase, so `Rust`, `rust` and `RUST` are the same word.
pub fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.unicode_words() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// The `n` most frequent words in `counts`, with their counts: the most
/// frequent first, and words with the same count in alphabetical order.
pub fn top_words(counts: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut pairs: Vec<(String, usize)> = counts
        .iter()
        .map(|(word, &count)| (word.clone(), count))
        .collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pairs.truncate(n);
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_repeated_words() {
        let counts = count_words("the cat and the hat");
        assert_eq!(counts["the"], 2);
        assert_eq!(counts["cat"], 1);
    }

    #[test]
    fn counting_ignores_case() {
        let counts = count_words("Rust rust RUST");
        assert_eq!(counts.len(), 1);
        assert_eq!(counts["rust"], 3);
    }

    #[test]
    fn top_words_sorts_by_count_then_alphabetically() {
        let counts = count_words("b a c b a d b");
        assert_eq!(
            top_words(&counts, 3),
            vec![
                ("b".to_string(), 3),
                ("a".to_string(), 2),
                ("c".to_string(), 1)
            ]
        );
    }

    #[test]
    fn top_words_returns_everything_when_n_is_large() {
        let counts = count_words("x y");
        assert_eq!(top_words(&counts, 100).len(), 2);
    }
}
