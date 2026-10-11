//! Word counting for the `wordfreq` command-line tool.
//!
//! `main.rs` only deals with arguments, input and output. The real work lives
//! here, in a library, so it can be unit tested without starting a process.

use std::collections::HashMap;

/// Counts how many times each word appears in `text`.
///
/// A word is what the `unicode-segmentation` crate's `unicode_words()` returns
/// (so `don't` and `3.14` are single words and punctuation is skipped),
/// converted to lowercase, so `Rust`, `rust` and `RUST` are the same word.
pub fn count_words(text: &str) -> HashMap<String, usize> {
    // TODO 1: add the unicode-segmentation crate (`cargo add unicode-segmentation`),
    // then split `text` with `unicode_words()` and count each lowercased word.
    todo!("count the words in {} bytes of text", text.len())
}

/// The `n` most frequent words in `counts`, with their counts: the most
/// frequent first, and words with the same count in alphabetical order.
pub fn top_words(counts: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    // TODO 2: copy the pairs into a Vec, sort them, and keep the first n.
    todo!("pick the top {n} of {} words", counts.len())
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

    // TODO 3: add at least two more unit tests, for example for top_words.
}
