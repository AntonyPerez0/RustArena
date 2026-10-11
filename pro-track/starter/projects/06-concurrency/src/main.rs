//! Benchmark: times every version on the same generated logs, best of N rounds,
//! and checks that they all give the same answer.
//!
//!     cargo run --release -- [--files N] [--lines N] [--threads N] [--rounds N]
//!
//! Always measure in release mode: debug builds can be 10-50 times slower and
//! distort the comparison. Don't edit this file: the grader uses it.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use logindex::{Index, generate, index_channels, index_rayon, index_serial, index_shared};

fn main() {
    let mut files = 160;
    let mut lines = 5_000;
    let mut threads = thread::available_parallelism().map_or(1, |n| n.get());
    let mut rounds = 3;
    let args: Vec<String> = std::env::args().skip(1).collect();
    for pair in args.chunks(2) {
        let value = pair.get(1).and_then(|v| v.parse().ok());
        match (pair[0].as_str(), value) {
            ("--files", Some(v)) => files = v,
            ("--lines", Some(v)) => lines = v,
            ("--threads", Some(v)) => threads = v,
            ("--rounds", Some(v)) => rounds = v,
            _ => {
                eprintln!("usage: logindex [--files N] [--lines N] [--threads N] [--rounds N]");
                std::process::exit(2);
            }
        }
    }

    let docs = Arc::new(generate(7, files, lines));
    let megabytes = docs.iter().map(String::len).sum::<usize>() as f64 / 1e6;
    println!(
        "{files} files x {lines} lines ({megabytes:.0} MB), {threads} threads, best of {rounds}"
    );

    let expected = index_serial(&docs);
    let serial = best_of(rounds, "serial", &expected, || index_serial(&docs));
    let shared = best_of(rounds, "shared", &expected, || {
        index_shared(Arc::clone(&docs), threads)
    });
    let channels = best_of(rounds, "channels", &expected, || {
        index_channels(&docs, threads)
    });
    let rayon = best_of(rounds, "rayon", &expected, || index_rayon(&docs));

    println!();
    println!("top words: {:?}", expected.top_words(5));
    let cpus = thread::available_parallelism().map_or(1, |n| n.get());
    println!(
        "RESULT cpus={cpus} serial_ms={:.1} shared_ms={:.1} channels_ms={:.1} rayon_ms={:.1}",
        ms(serial),
        ms(shared),
        ms(channels),
        ms(rayon)
    );
}

/// Runs `f` `rounds` times and returns the fastest time.
fn best_of(rounds: usize, name: &str, expected: &Index, f: impl Fn() -> Index) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..rounds.max(1) {
        let start = Instant::now();
        let index = f();
        best = best.min(start.elapsed());
        if index != *expected {
            eprintln!("{name}: wrong answer (different from index_serial)");
            std::process::exit(1);
        }
    }
    println!("{name:<9} {:>8.1} ms", ms(best));
    best
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}
