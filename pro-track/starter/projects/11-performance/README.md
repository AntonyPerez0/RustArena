# 11 · Benchmarks and profiling

The four functions in `src/lib.rs` give the right answers, and on a ten-item test they're instant. On 150,000 items each one takes many seconds. This is the most common performance story in industry: code that was fine at small scale meets production data.

Your job: **measure** where the time goes, fix it, and get every function under its time budget in an optimized build, without changing a single result.

**You'll practice:** release builds and profiles, criterion benchmarks, profilers and flame graphs, algorithmic complexity in real code, and avoiding needless allocation and copying.

## Background

### Rule 1: measure, don't guess

Programmers are famously bad at guessing where time goes, so always measure first, and always measure an **optimized** build. `cargo build` and `cargo test` make debug builds, which can be 10 to 100 times slower and point at the wrong culprit. `--release` turns the optimizer on.

The quickest complexity check: double the input and watch the time. O(n) doubles, O(n log n) a little more than doubles, O(n²) **quadruples**. A 10x faster constant factor never saves an O(n²) algorithm on a million items, so fix the big-O first.

### Benchmarks with criterion

Timing a function once with `Instant::now()` is noisy: caches warm up, the CPU changes speed, other programs interrupt. **criterion** runs the function many times, reports a confidence interval, flags outliers, and compares with your previous run.

```bash
cargo bench                                  # everything (benches/hotspots.rs)
cargo bench -- top_words                     # only benchmarks whose name contains top_words
cargo bench -- --save-baseline before        # remember these numbers as "before"
cargo bench -- --baseline before             # compare with them later
```

`std::hint::black_box(x)` stops the optimizer from noticing that a result is unused and deleting the work you're trying to measure. The HTML report with charts is in `target/criterion/report/index.html` at the root of the repository (all projects share one `target/` folder).

### Profilers and flame graphs

A benchmark tells you **how long**; a profiler tells you **where**. A sampling profiler interrupts the program thousands of times a second and records the call stack each time. In a **flame graph**, each box is a function, its width is the share of samples it appeared in, and callers sit below callees. Look for wide boxes: wide at the bottom is an expensive subtree, wide at the top is a hot spot.

Profilers need debug symbols to show function names and lines. `Cargo.toml` defines a `profiling` profile, a release build that keeps them, and `examples/workload.rs` runs one function on a big input:

```bash
cargo build --profile profiling --example workload
../../target/profiling/examples/workload top_words          # how long?

# Pick one profiler:
samply record ../../target/profiling/examples/workload top_words   # cargo install --locked samply
cargo flamegraph --profile profiling --example workload -- top_words   # cargo install flamegraph; writes flamegraph.svg
perf record -g ../../target/profiling/examples/workload top_words && perf report   # Linux
valgrind --tool=callgrind ../../target/profiling/examples/workload ranks           # works everywhere, slowly
```

samply, flamegraph and perf use the Linux `perf_event` interface. Inside some containers (Codespaces included) it's restricted. If they refuse, try `echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid`, or use callgrind (`sudo apt-get install valgrind`, then `callgrind_annotate callgrind.out.*`), which needs nothing special.

### The usual suspects

| Smell | Fix |
|---|---|
| `vec.contains(x)` or `iter().position(...)` inside a loop: O(n) per lookup | `HashSet` / `HashMap` (O(1) on average); the `entry` API for counting |
| sorting inside a loop | sort once at the end; `partition_point` (binary search) on sorted data |
| `s = format!("{s}...")` or `s = s.clone() + ...`: copies everything so far, every time | `push_str` or `write!` into one `String`, sized with `with_capacity` |
| `.clone()` and `.to_vec()` in a hot loop | borrow instead (`&str`, `&[T]`); clone only what you keep |
| a `Vec` or `String` growing one push at a time when you know the size | `with_capacity(n)` |

## Your tasks

1. Run the correctness tests: `cargo test`. They pass now, and must still pass after every change.
2. Benchmark the original code and save it: `cargo bench -- --save-baseline before`. Note the `unique_in_order` times for 1,000 and 4,000 items. What complexity does that suggest?
3. Profile at least one function with the `workload` example and find the hot spot. Don't skip this step: the point of the project is the habit.
4. Fix all four functions. Each has a different problem, and `top_words` has two. Run `cargo test` after each change, and `cargo bench -- --baseline before` to see the gain.
5. Add a criterion benchmark for `ranks` in `benches/hotspots.rs` (use `data::scores`) and register it in `criterion_group!`.
6. Check the time budgets, then grade:

```bash
cargo test --release --test budget -- --ignored --test-threads=1 --nocapture
bash ../../tools/grade.sh 11-performance
```

Each budget test generates a big input from `src/data.rs`, runs your function on another thread, and fails as soon as it takes longer than one second. A good solution takes a few milliseconds to a few tens of milliseconds, so the budget is generous even on a busy CI machine.

## What the grader checks

- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` (benches and examples too) are clean.
- `tests/correctness.rs`, `tests/budget.rs` and `src/data.rs` are unchanged.
- `cargo test` passes, and `cargo bench --no-run` compiles the benchmarks.
- `benches/hotspots.rs` benchmarks `ranks`.
- All four budget tests pass in a release build.

## Hints

- `unique_in_order`: a `HashSet<&str>` can borrow from the input, so you only clone the strings you return. `HashSet::insert` returns `false` when the item was already there.
- `top_words`: count everything first (`*counts.entry(word).or_insert(0) += 1`), then sort once. Keep the exact tie-breaking: count descending, then word ascending.
- `to_csv`: `use std::fmt::Write;` lets you `writeln!(csv, "{},{}", row.name, row.score)` straight into a `String`.
- `ranks`: in a list sorted highest first, the number of scores strictly higher than `s` is `sorted.partition_point(|&x| x > s)`.

## Going further

- Count allocations with `dhat` (the `dhat` crate as a global allocator) before and after.
- Try a faster hasher (`rustc-hash`'s `FxHashMap` or `ahash`) in `top_words`. Measure: is it worth a dependency?
- Avoid allocating for words that are already lowercase.
- Parallelize `top_words` with rayon (count chunks in parallel, then merge) and benchmark 1, 2 and 4 threads.
- Read the assembly of a hot loop with `cargo asm` (`cargo install cargo-show-asm`).
- For benchmarks that run in CI, look at instruction counts (`iai-callgrind`), which don't depend on how busy the machine is.
