use std::collections::HashMap;

/// Counts from a set of log lines like
/// `2026-10-11T08:15:02Z WARN  billing: payment 1234 declined by bank`:
/// how many lines, how many per level and per service, and how often each word
/// of the message appears (lowercased; tokens containing digits are skipped).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Index {
    pub lines: u64,
    pub levels: HashMap<String, u64>,
    pub services: HashMap<String, u64>,
    pub words: HashMap<String, u64>,
}

impl Index {
    /// An index of one document (many lines).
    pub fn of_document(doc: &str) -> Index {
        let mut index = Index::default();
        index.add_document(doc);
        index
    }

    pub fn add_document(&mut self, doc: &str) {
        for line in doc.lines() {
            self.add_line(line);
        }
    }

    /// Adds one line. Blank lines are ignored.
    pub fn add_line(&mut self, line: &str) {
        let mut tokens = line.split_whitespace();
        let Some(_timestamp) = tokens.next() else {
            return;
        };
        self.lines += 1;
        if let Some(level) = tokens.next() {
            bump(&mut self.levels, level);
        }
        if let Some(service) = tokens.next() {
            bump(&mut self.services, service.trim_end_matches(':'));
        }
        for token in tokens {
            let word = token.trim_matches(|c: char| !c.is_alphanumeric());
            if word.is_empty() || word.chars().any(|c| c.is_ascii_digit()) {
                continue;
            }
            if word.chars().any(char::is_uppercase) {
                bump(&mut self.words, &word.to_lowercase());
            } else {
                bump(&mut self.words, word);
            }
        }
    }

    /// Adds every count from `other` into `self`.
    pub fn merge(&mut self, mut other: Index) {
        // Merging is cheaper when the bigger index is the one kept.
        if self.words.len() < other.words.len() {
            std::mem::swap(self, &mut other);
        }
        self.lines += other.lines;
        for (map, from) in [
            (&mut self.levels, other.levels),
            (&mut self.services, other.services),
            (&mut self.words, other.words),
        ] {
            for (key, count) in from {
                *map.entry(key).or_insert(0) += count;
            }
        }
    }

    /// The `n` most frequent words, most frequent first (ties: alphabetical).
    pub fn top_words(&self, n: usize) -> Vec<(&str, u64)> {
        let mut words: Vec<(&str, u64)> =
            self.words.iter().map(|(w, c)| (w.as_str(), *c)).collect();
        words.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        words.truncate(n);
        words
    }
}

/// Adds 1 to `key`, allocating a String only the first time the key is seen.
fn bump(map: &mut HashMap<String, u64>, key: &str) {
    match map.get_mut(key) {
        Some(count) => *count += 1,
        None => {
            map.insert(key.to_owned(), 1);
        }
    }
}
