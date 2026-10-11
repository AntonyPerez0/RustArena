//! criterion benchmarks: `cargo bench` (or `cargo bench -- top_words` for one group).
//!
//! criterion runs each function many times, reports the time with a confidence interval,
//! and compares with the previous run ("change: -97.1%"). The HTML report is in
//! target/criterion/report/index.html (target/ is at the root of the repository).
//!
//! The inputs are small so the original, slow code finishes in a reasonable time.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hotspots::{data, to_csv, top_words, unique_in_order};

fn bench_unique(c: &mut Criterion) {
    let mut group = c.benchmark_group("unique_in_order");
    for n in [1_000, 4_000] {
        let names = data::names(n, n * 6 / 10, 1);
        // Doubling the input and watching the time is the quickest complexity check:
        // O(n) doubles, O(n^2) quadruples.
        group.bench_with_input(BenchmarkId::from_parameter(n), &names, |b, names| {
            b.iter(|| unique_in_order(black_box(names)))
        });
    }
    group.finish();
}

fn bench_top_words(c: &mut Criterion) {
    let text = data::text(4_000, 2_000, 2);
    c.bench_function("top_words/4000", |b| {
        b.iter(|| top_words(black_box(&text), 10))
    });
}

fn bench_to_csv(c: &mut Criterion) {
    let rows = data::rows(4_000, 3);
    c.bench_function("to_csv/4000", |b| b.iter(|| to_csv(black_box(&rows))));
}

// TODO: add a benchmark for `ranks` (on data::scores) and register it below.

criterion_group!(benches, bench_unique, bench_top_words, bench_to_csv);
criterion_main!(benches);
