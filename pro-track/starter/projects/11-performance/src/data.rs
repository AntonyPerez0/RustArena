//! Deterministic input data for the tests, benchmarks and the profiling workload.
//!
//! Provided, and checked by the grader: the time budgets are measured on exactly this
//! data, so don't change this file.

/// One row of a score table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub score: u32,
}

/// A small, fast pseudo-random number generator (xorshift64*). The same seed gives the
/// same numbers on every machine, which keeps tests and benchmarks repeatable.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in `0..n`.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// `n` user names like `user01234`, drawn from `distinct` possible names.
pub fn names(n: usize, distinct: usize, seed: u64) -> Vec<String> {
    let mut rng = Rng::new(seed);
    (0..n)
        .map(|_| format!("user{:05}", rng.below(distinct as u64)))
        .collect()
}

/// A made-up word for each index: 0 is "a", 1 is "b", 26 is "aa"...
fn word(mut index: usize) -> String {
    let mut letters = Vec::new();
    loop {
        letters.push(b'a' + (index % 26) as u8);
        index /= 26;
        if index == 0 {
            break;
        }
        index -= 1;
    }
    letters.reverse();
    String::from_utf8(letters).unwrap()
}

/// Text of `words` words from a vocabulary of `vocabulary` words. Like real text, a few
/// words are very common and most are rare. Some are Capitalized, with punctuation and
/// line breaks between them.
pub fn text(words: usize, vocabulary: usize, seed: u64) -> String {
    let mut rng = Rng::new(seed);
    let vocab: Vec<String> = (0..vocabulary).map(word).collect();
    let v = vocabulary as u64;
    let mut out = String::with_capacity(words * 8);
    for _ in 0..words {
        let index = (rng.below(v) * rng.below(v) / v) as usize;
        let w = &vocab[index];
        if rng.below(8) == 0 {
            out.push_str(&w[..1].to_uppercase());
            out.push_str(&w[1..]);
        } else {
            out.push_str(w);
        }
        out.push_str(match rng.below(16) {
            0 => ", ",
            1 => ". ",
            2 => "\n",
            3 => " - ",
            _ => " ",
        });
    }
    out
}

/// `n` rows with random names and scores.
pub fn rows(n: usize, seed: u64) -> Vec<Row> {
    let mut rng = Rng::new(seed);
    (0..n)
        .map(|i| Row {
            name: format!("{}{i}", word(rng.below(5000) as usize)),
            score: rng.below(1_000_000) as u32,
        })
        .collect()
}

/// `n` scores between 0 and `max`, inclusive (so there are ties when `n` is large).
pub fn scores(n: usize, max: u32, seed: u64) -> Vec<u32> {
    let mut rng = Rng::new(seed);
    (0..n)
        .map(|_| rng.below(u64::from(max) + 1) as u32)
        .collect()
}
