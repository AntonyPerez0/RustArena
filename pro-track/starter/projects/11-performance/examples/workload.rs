//! A workload to run under a profiler: one function, on a big input, a few times.
//!
//!   cargo build --profile profiling --example workload
//!   perf record -g ../../target/profiling/examples/workload top_words
//!   perf report
//!
//! or, with samply (opens a flame graph in your browser):
//!
//!   samply record ../../target/profiling/examples/workload top_words
//!
//! The sizes are large enough to show up clearly in a profile, and small enough that the
//! original code finishes in a few seconds.

use std::hint::black_box;
use std::time::Instant;

use hotspots::{data, ranks, to_csv, top_words, unique_in_order};

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    let start = Instant::now();
    match which.as_str() {
        "unique_in_order" => {
            let names = data::names(40_000, 24_000, 1);
            black_box(unique_in_order(&names));
        }
        "top_words" => {
            let text = data::text(30_000, 20_000, 2);
            black_box(top_words(&text, 10));
        }
        "to_csv" => {
            let rows = data::rows(30_000, 3);
            black_box(to_csv(&rows));
        }
        "ranks" => {
            let scores = data::scores(8_000, 1_000_000, 4);
            black_box(ranks(&scores));
        }
        _ => {
            eprintln!("usage: workload unique_in_order|top_words|to_csv|ranks");
            std::process::exit(2);
        }
    }
    eprintln!("{which}: {:.2?}", start.elapsed());
}
