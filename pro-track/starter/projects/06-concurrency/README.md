# 06 · Concurrency with threads, channels and rayon

Every machine you'll deploy to has several cores, and a program that uses one of them leaves most of the hardware idle. Rust is famous for "fearless concurrency": a **data race** (two threads touching the same memory, at least one writing, without synchronization) is a compile error, not a 3 a.m. bug. What the compiler can't do is choose a design for you, stop **deadlocks**, or make parallel code fast. That's your job here.

You'll build a log indexer that counts lines, levels, services and words across many log files, three ways: threads sharing a lock, threads sending messages, and rayon. Then you'll prove they all give the serial answer, and measure which is faster.

**You'll practice:** `std::thread::spawn` and `thread::scope`, `Arc`, `Mutex` and atomics, `mpsc` channels, rayon, `Send` and `Sync`, deadlocks, and measuring speedups in release mode.

## The project

| File | What |
|---|---|
| `src/index.rs` | `Index`: the counts, with `add_document` and `merge` (provided) |
| `src/lib.rs` | `index_serial`, the single-threaded reference (provided) |
| `src/shared.rs`, `src/channels.rs`, `src/with_rayon.rs` | **your three versions** |
| `src/main.rs` | the benchmark: times all four on 58 MB of generated logs |

```bash
cd projects/06-concurrency
cargo test                           # do they agree with index_serial?
cargo run --release                  # how fast is each one?
cargo run --release -- --threads 2   # try 1, 2, 4, 8...
```

## Background

### Threads, `'static` and scopes

`thread::spawn(closure)` starts an OS thread and returns a `JoinHandle`; `join()` waits for it and returns `Err` if it panicked. The closure must be `'static` (own everything it uses, usually via `move`), because the new thread may outlive the function that started it. To share read-only data, wrap it in an `Arc` (an atomically reference-counted pointer) and give each thread a clone: a pointer copy, not a data copy.

`thread::scope(|s| { s.spawn(...); })` waits for every thread spawned in it before returning, so those threads may **borrow** local data. No `Arc` needed.

### Send and Sync

Two marker traits, implemented automatically by the compiler:

- `Send`: a value may **move** to another thread. `Rc` isn't `Send`: its reference count isn't atomic, so two threads cloning it could corrupt it.
- `Sync`: a value may be **shared** (`&T`) between threads. `RefCell` isn't `Sync`: its borrow flag isn't atomic. A `Mutex<T>` is `Sync` because the lock serializes access.

`thread::spawn` requires its closure to be `Send`, so passing an `Rc` to a thread fails to compile (`src/lib.rs` has an example that the doc tests prove does not compile). That's how safe Rust rules out data races. It does not rule out *logic* races, like two threads reading a balance and both deciding there's enough money.

### Shared state: `Arc<Mutex<T>>`

```rust
let total = Arc::new(Mutex::new(Index::default()));
// in each thread, with its own clone of the Arc:
total.lock().unwrap().merge(part);   // the guard unlocks at the end of the statement
```

The lock is held as long as the guard lives. Every moment a thread holds it, the others wait, so keep critical sections short: do the work on local data and lock only to combine. `lock()` returns `Err` if another thread panicked while holding it ("poisoning"). For a single counter, an atomic (`AtomicUsize::fetch_add`) needs no lock at all.

### Message passing: channels

"Don't communicate by sharing memory; share memory by communicating." `mpsc::channel()` gives a `Sender` (clone it for each producer) and a `Receiver` (one consumer). Each worker owns its data, and the only thing that crosses threads is the finished result. `for msg in rx` ends when **every** `Sender` has been dropped. `mpsc::sync_channel(n)` is bounded: `send` blocks when it's full, which is how you stop fast producers from using all your memory (backpressure).

### rayon and work stealing

```rust
use rayon::prelude::*;
let total: u64 = docs.par_iter().map(|d| d.len() as u64).sum();
```

`par_iter()` runs the iterator on rayon's global pool (one thread per core). Work is split in halves recursively; each thread keeps a queue of pieces, and an idle thread **steals** from a busy one's queue, so uneven work balances itself with no tuning. `fold` gives each piece its own accumulator, and `reduce` combines them pairwise, so the combining operation must be associative (merging indexes is).

### Deadlocks

The compiler can't see these, and a deadlocked program just stops:

- Two threads lock mutexes A and B in opposite orders, and each waits for the other forever. Fix: always lock in one fixed order, or hold one lock at a time.
- A thread locks a `Mutex` it already holds (Rust's `Mutex` is not re-entrant).
- A loop over a `Receiver` while a `Sender` is still alive somewhere, even your own original.
- Holding a lock while blocking: sending into a full `sync_channel`, or `join()`ing a thread that needs that lock.

The tests run every version with a 60-second limit, so a deadlock fails with a hint instead of hanging CI.

### Measuring

Always benchmark with `--release`; debug builds are many times slower. Take the best of several rounds to filter noise. The serial part of a job limits the speedup (Amdahl's law): here that's merging, so merge whole indexes, not single words. More threads than cores rarely helps.

## Your tasks

1. Read `src/index.rs` and `src/lib.rs`, then run `cargo run --release` to see the serial time (the next version panics with `todo!`).
2. **Part 1** (`src/shared.rs`): `index_shared`, with `thread::spawn`, an atomic work counter and `Arc<Mutex<Index>>`. The comments walk you through it.
3. **Part 2** (`src/channels.rs`): `index_channels`, with `thread::scope`, one slice of documents per thread, and an `mpsc` channel.
4. **Part 3** (`src/with_rayon.rs`): `index_rayon`, with `par_iter`.
5. `cargo test`, then `cargo run --release` with different `--threads` values. Which version wins on your machine, and why?
6. Grade (`bash ../../tools/grade.sh 06-concurrency`), then commit and push.

## What the grader checks

- `cargo fmt` and `cargo clippy` are clean; the tests, `lib.rs` and `main.rs` are unchanged.
- All tests pass: every version, with 0 to 64 threads, empty and uneven inputs, matches `index_serial`.
- Each part uses its tool (`thread::spawn` + `Arc` + `Mutex`; `thread::scope` + a channel; rayon's `par_iter`) and doesn't call `index_serial`.
- The release benchmark: no parallel version may be more than 25% slower than serial. That's a deliberately low bar for noisy shared CI machines; on 4 cores you should see 2 to 4 times faster.

## Hints

- `threads` can be 0 (treat it as 1) or larger than the number of documents (some threads get nothing, which must still work).
- `while let Some(doc) = docs.get(next.fetch_add(1, Ordering::Relaxed))` is a whole work queue in one line.
- With `docs.chunks(n)`, use `n = docs.len().div_ceil(threads)`, but never 0: `chunks(0)` panics.
- If a test times out, look for a `Sender` you didn't `drop`.

## Going further

- In `index_shared`, lock once per *line* instead of once per document and measure the difference. That's lock contention.
- Write a deliberate deadlock with two mutexes locked in opposite orders, watch it hang, then fix it.
- Replace the `mpsc` channel with a bounded `sync_channel(1)`. Does it still work if you receive *after* the scope ends? Why not?
- Implement a parallel merge sort with `rayon::join`.
- Profile with `perf` or `cargo flamegraph` (project 11) to see where the serial time goes.
