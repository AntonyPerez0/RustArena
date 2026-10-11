use std::fmt::Write;

const LEVELS: [&str; 5] = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
// Out of 100 lines: 3 TRACE, 20 DEBUG, 60 INFO, 12 WARN, 5 ERROR.
const LEVEL_WEIGHTS: [u64; 5] = [3, 20, 60, 12, 5];
const SERVICES: [&str; 8] = [
    "auth",
    "billing",
    "search",
    "gateway",
    "storage",
    "mailer",
    "scheduler",
    "metrics",
];
// `#` is replaced by a random number.
const MESSAGES: [&str; 16] = [
    "User # logged in from 10.0.#.#",
    "user # failed to log in: wrong password",
    "request # completed in # ms",
    "cache miss for key session:#",
    "Payment # declined by bank, customer notified",
    "connection to database lost; retrying in # s",
    "slow query took # ms on table orders",
    "disk usage at # percent on volume data",
    "sent email # to customer",
    "job # finished with status OK",
    "rate limit exceeded for client #",
    "upstream timeout after # ms (attempt #)",
    "index rebuilt with # documents",
    "invalid token rejected for user #",
    "Config reloaded from disk",
    "health check passed",
];

/// Generates `files` log files of `lines` lines each, the same every time for
/// the same `seed`. Each file is one String.
pub fn generate(seed: u64, files: usize, lines: usize) -> Vec<String> {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    (0..files)
        .map(|file| {
            let mut doc = String::with_capacity(lines * 72);
            let mut second = file as u64 * 7;
            for _ in 0..lines {
                second += rng.below(3);
                let (h, m, s) = (second / 3600 % 24, second / 60 % 60, second % 60);
                let level = rng.weighted(&LEVEL_WEIGHTS);
                let service = SERVICES[rng.below(SERVICES.len() as u64) as usize];
                let _ = write!(
                    doc,
                    "2026-10-11T{h:02}:{m:02}:{s:02}Z {:<5} {service}: ",
                    LEVELS[level]
                );
                let message = MESSAGES[rng.below(MESSAGES.len() as u64) as usize];
                for (i, piece) in message.split('#').enumerate() {
                    if i > 0 {
                        let _ = write!(doc, "{}", rng.below(10_000));
                    }
                    doc.push_str(piece);
                }
                doc.push('\n');
            }
            doc
        })
        .collect()
}

/// A tiny xorshift random number generator: fast, deterministic, no dependencies.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn weighted(&mut self, weights: &[u64]) -> usize {
        let mut roll = self.below(weights.iter().sum());
        for (i, w) in weights.iter().enumerate() {
            if roll < *w {
                return i;
            }
            roll -= w;
        }
        weights.len() - 1
    }
}
