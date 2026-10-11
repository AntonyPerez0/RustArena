# Writing RustArena content

Everything a learner sees is YAML under `content/`. `npm run content` compiles and runs every
reference solution, worked example and drill with the real Rust compiler (edition 2024, debug
build, like the Rust Playground) and refuses to build if anything is off: a solution that doesn't
compile or has warnings, an example whose printed output isn't what the text claims, starter code
that already passes, a drill whose "buggy" line isn't actually buggy.

Check just your files while you write (nothing is written to `src/generated/`):

```bash
node scripts/build-content.mjs --only ownership     # any content path containing "ownership"
```

Fix every `ERROR`. Look at the `warn:` lines too (missing hints, missing `why`).

## Who we're writing for

Someone who has never programmed, all the way to someone who'll use Rust at work. The course
follows [roadmap.sh/rust](https://roadmap.sh/rust) and ends with workplace skills (serde, error
handling with anyhow/thiserror, async with Tokio, testing, CLIs, web services, code review).

Style, learned from the C/C++ Arena lessons this site is modeled on:

- Second person, plain words, short paragraphs. Explain **why**, not just what. Define every new
  term in **bold** the first time.
- One idea per step. Show a complete worked example (` ```rust run ` + ` ```output `), explain it
  piece by piece, then list **common mistakes** (with the compiler error learners will see).
- Rust's compiler errors are a teaching tool: show real ones with ` ```rust compile_fail ` +
  ` ```error ` blocks, especially for ownership and borrowing.
- Workplace notes where they matter: what idiomatic code looks like, what clippy would say, what
  teams do. Never hand-wave ("it's complicated").
- Only use features already taught in earlier modules (see the outline at the end) or in earlier
  steps of the same module. If you need something not yet taught, explain it briefly in place.
- Use realistic, varied scenarios (inventory, game scores, log lines, bank accounts, temperatures,
  orders, tickets). A little arena/crab flavor is welcome; don't overdo it.
- No em dashes; use colons, commas or parentheses. Avoid filler and hype.

## Lesson modules: `content/lessons/NN-name.yaml`

```yaml
id: ownership                 # module id: lowercase, used in URLs and drill files
title: Ownership
phase: "Ownership"            # the curriculum section (see the outline)
summary: One or two sentences shown on the curriculum page.
roadmap: [Ownership Rules & Memory Safety, "Deep Dive: Stack vs Heap"]   # roadmap.sh topics covered
steps:
  - title: Who owns this String?      # quote titles containing ':' or starting with a symbol
    id: ownership-1                   # explicit, unique, never reused: progress is saved by id
    text: |
      Markdown. Explanation, worked examples, common mistakes...

      **Your turn:** the task. Everything from "**Your turn:**" on is shown in a task card above
      the editor, so make it precise: exactly what to print or which function to write.
    seed: |          # starter code (for code exercises)
      ...
    solution: |      # reference solution (must compile with NO warnings)
      ...
    tests:
      - expect: "..."            # optional: pinned expected output (checked against the solution)
    hints:                       # 1-3 hints, revealed one at a time, the last one nearly gives it away
      - ...
    more:                        # extra challenges on the same idea (2 is typical; the step is done when all pass)
      - task: |
          ...
        seed: | ...
        solution: | ...
        hints: [...]
```

Aim for **5 to 7 steps per module** and **2 extra challenges per step** (so ~3 exercises per
step). Vary the extra challenges: one close variant, one that asks for more independence (write
it from a nearly empty starter, or combine with earlier ideas). Later modules can have bigger,
more realistic exercises.

### Exercise kinds

**Fill in the blanks** (early steps of a module, to introduce syntax): use `fill` instead of
`seed`/`solution`. Blanks are `[[answer]]`; accept alternatives with `‖`:
`[[&mut s‖&mut  s]]`. Answers are compared ignoring whitespace. Keep blanks short (a keyword,
a type, a method name, a small expression).

**Write a program (stdout mode)**: `seed` + `solution` with `fn main`. Each test runs the program
and compares what it prints (trailing spaces and trailing blank lines are ignored).

```yaml
tests:
  - stdin: "3\n4\n"          # what the program reads (optional)
  - stdin: "10\n20\n"
    hidden: true             # hidden tests stop hard-coding the visible answer
  - args: ["--verbose", "in.txt"]        # command-line arguments (std::env::args)
    files: { "in.txt": "line 1\nline 2\n" }   # files in the working directory
    exit: 2                  # expected exit code (default 0; use with std::process::exit)
```

The expected output is computed from the solution; you may pin it with `expect:` and the build
checks it. Every exercise that reads input should have at least one hidden test.

**Write functions (harness mode)**: give a `harness` with hidden `#[test]` functions. The learner's
code has no `fn main` (it may keep one; it's ignored). Each test runs on its own; a failed
`assert!` fails just that test. Name tests so their names read as sentences: they're shown to the
learner as "The tests check: adds two positive numbers".

```yaml
seed: |
  fn area(width: u32, height: u32) -> u32 {
      todo!()
  }
solution: |
  fn area(width: u32, height: u32) -> u32 {
      width * height
  }
harness: |
  #[test]
  fn multiplies_width_by_height() {
      assert_eq!(area(3, 4), 12, "area(3, 4)");
  }

  #[test]
  fn zero_width_gives_zero() {
      assert_eq!(area(0, 9), 0, "area(0, 9)");
  }
```

- Write `assert_eq!(actual, expected, "the call")`: the site shows "expected X, got Y" and the
  call. The values must implement `Debug` (derive it in the seed for the learner's own types).
- The harness can use everything in the learner's code (it's a child module with `use super::*`).
- `todo!()` in a seed is the idiomatic placeholder: it compiles and panics when called.
- Seeds that return `-> SomeType` need a body that compiles: `todo!()` works everywhere.
- Unused parameters in a seed are fine (seeds may have warnings); solutions may not.
- 3 to 6 tests per exercise, covering edge cases (empty input, zero, negatives, unicode).
- If the learner's code must also print, use stdout mode instead.

**Write tests (test mode)** (testing module and a few others): `mode: test`. The learner writes
`#[test]` functions; the site runs `cargo test`. `minTests` sets how many they must write.
`mutants` are broken copies of the code (find/replace on the seed's code) that a good test suite
must catch: each mutant must make at least one of the learner's tests fail. The `find` text must
appear in both the seed and the solution. An optional `harness` adds hidden tests (in test mode the
harness is plain `#[test]` functions too).

### Rules: `require` and `forbid`

Regexes tested against the learner's code with comments removed and string/char literals blanked
to `""`/`''` (so `"for"` in a message doesn't count as a `for` loop). Add `strings: true` to test
against the code with strings kept (for rules about format strings).

```yaml
require:
  - pattern: '\.iter\(\)'
    message: Use `.iter()` to walk the vector without taking ownership.
forbid:
  - pattern: '\.clone\(\)'
    message: Solve it without `.clone()`: borrow instead.
```

Use rules when the point of the step could be dodged (using `.clone()` in a borrowing lesson,
a `for` loop in an iterator lesson, hard-coding the output). The solution must pass its own rules.
YAML: put patterns in single quotes; inside single quotes a backslash is literal.

### Worked examples in the text

````markdown
```rust run
fn main() {
    println!("{}", 2 + 2);
}
```

```output
4
```
````

- ` ```rust run `: a complete program; compiled and run. If followed by ` ```output `, it must
  print exactly that. An ` ```input ` block right after it is what the program reads.
- ` ```rust panics `: must compile and then panic; ` ```output ` shows what it printed before.
- ` ```rust compile_fail `: must NOT compile. If followed by ` ```error `, that block's first line
  must appear in rustc's output, so copy it exactly, for example
  ``error[E0382]: borrow of moved value: `s` ``. You may add a few more lines of the real message
  below it (the arrows), but only the first line is checked.
- Plain ` ```rust `: shown as is, not compiled (fragments, `Cargo.toml` snippets go in ` ```toml `,
  shell sessions in ` ```text `).

Examples must have **no warnings** (unused variables etc.) and **deterministic output**.

### Determinism (important)

Tests compare exact output, and the build runs examples and drills several times:

- Never print a `HashMap`/`HashSet` directly or iterate one to print: the order changes between
  runs. Sort the keys first, or use `BTreeMap`/`BTreeSet`.
- Threads: join them and print results in a fixed order (collect results, then print), or print
  only from the main thread after joining.
- No real time, no random numbers in graded output (`Instant` durations, `SystemTime`, `rand`).
- Async (Tokio): `join!` results are in order; don't rely on which spawned task prints first.

### Crates

Lesson code may use these crates (all available on the Rust Playground; also in
`runner-deps/Cargo.toml`): `serde` (with `derive`), `serde_json`, `anyhow`, `thiserror`, `tokio`
(full), `regex`, `itertools`, `rayon`, `chrono`. Don't use any other crate in compiled code.
`chrono`: only fixed dates (no `now()`). Write `use serde::{Deserialize, Serialize};` as usual.

### The edition and the compiler

Code is compiled as edition **2024** in a debug build (integer overflow panics, like
`cargo run`). Inline format args (`println!("{name}")`) are fine and idiomatic; teach both forms.
In edition 2024, `std::env::set_var`/`remove_var` are `unsafe`; `gen` is a reserved keyword.

### What learners see when things go wrong

- Compile errors: rustc's message with its snippet, plus a plain-English note for common error
  codes, and a link to the error's page.
- Panics: the panic message and the line in their code.
- Time limit: 2 seconds per test run.

## Drills: `content/drills/<module-id>.yaml`

Drills are fast questions for Deathmatch (endless practice). Each unlocks when the learner
finishes the lesson step that teaches it (worked out automatically from the code features; set
`after: <step id>` to force it). Write **14 to 18 drills per module**, mixing types: about 5
predict, 3 fill, 2-3 bug, 3 compiles (great for ownership/borrowing/lifetimes/traits), 1-2
choice, 1 boss.

```yaml
topic: ownership            # the module id
drills:
  - type: predict           # "What does this print?" (answer computed by running it; keep output short, 1-3 lines)
    pre: |                  # optional items before main (structs, fns, use lines)
      fn takes(s: String) -> usize { s.len() }
    body: |                 # the body of fn main
      let s = String::from("crab");
      let n = takes(s.clone());
      println!("{} {}", s, n);
    why: One or two sentences explaining the answer. Always required.

  - type: fill              # exactly one [[blank]]; the program's output is shown as the goal
    prompt: Borrow the string instead of moving it.
    body: |
      let s = String::from("hi");
      let n = length([[&s]]);
      println!("{} {}", s, n);
    pre: |
      fn length(s: &String) -> usize { s.len() }
    why: ...

  - type: bug               # mark the one wrong line with // BUG and give the corrected line as `fix`
    body: |
      let mut v = vec![3, 1, 2];
      v.sort();
      println!("{}", v[3]); // BUG
    fix: 'println!("{}", v[2]);'
    prompt: This should print the largest number, 3. Click the line with the bug.   # optional; a default is generated from the fixed output
    why: ...

  - type: compiles          # "Does this compile?" answer computed by rustc; give `answer: no` or `yes` to state your intent (checked)
    answer: no
    body: |
      let s = String::from("hi");
      let t = s;
      println!("{}", s);
    why: Assigning `s` to `t` moves the String; `s` can't be used afterwards.

  - type: choice            # a conceptual question, 2-4 choices, answer is the 1-based index
    prompt: Where does a String keep its characters?
    choices: ["On the stack, inside the String", "On the heap; the String holds a pointer, length and capacity", "In the binary's read-only data"]
    answer: 2
    why: ...
    # Optional code shown with the question: pre/body (must compile unless `compiles: false`).
    # `verify: output` checks the right choice is exactly what the code prints.

  - type: boss              # a full mini exercise (harness or stdout), like a lesson step
    prompt: |
      Write `fn longest(words: &[String]) -> Option<&String>` ...
    seed: | ...
    solution: | ...
    harness: | ...
    hints: [...]
    why: ...
```

Drill code runs inside `fn main` with `#![allow(unused)]`, so unused variables are fine there.
`bug` drills: the buggy version must behave differently (wrong output, panic, or not compile) and
the fixed one must run cleanly. A lone `}` can't be the bug line.

## Curriculum outline (module ids, files, what each teaches)

Use this to know what's already been taught. Modules come in this order.

**Foundations**
- `01-hello` hello: fn main, println!/print!, string literals, escapes, `{}` placeholders, `{{}}`, comments, reading compiler errors, Cargo commands (new/run/build/check/test/fmt/clippy), rustup.
- `02-variables` variables: let, immutability by default, mut, shadowing, type annotations and inference, const, naming (snake_case, SCREAMING_CASE), `_` prefix, inline format args `{x}`, `{:?}` intro.
- `03-types` types: integer types and ranges, literals (1_000, 0xff, 0b1010), overflow (debug panic) and checked_/wrapping_/saturating_, floats (f64), integer division and %, casting with `as`, bool and logic operators, char (Unicode), `{:.2}` precision, i32::MAX.
- `04-functions` functions: parameters and types, return values, expressions vs statements, the last expression as return value, early `return`, blocks are expressions, the unit type `()`, calling functions from functions.
- `05-control` control: if/else if/else, if as an expression, comparison operators, loop (with break value), while, for over ranges (`0..n`, `0..=n`, `.rev()`, `.step_by()`), break/continue, loop labels.
- `06-compound` compound: tuples (types, `.0`, destructuring, returning several values), arrays `[T; N]`, `[0; 10]`, indexing and out-of-bounds panics, `.len()`, `for x in arr`, `{:?}` for arrays, 2D arrays, a first look at Vec (`vec![]`, push).
- `07-strings-intro` strings-intro: String vs &str (first look, not ownership yet), String::from, .to_string(), push_str/push, format!, + concatenation, .len() (bytes) vs .chars().count(), to_uppercase/to_lowercase, contains, replace, trim, comparing strings with ==.
- `08-input` input: std::io::stdin().read_line(&mut line), trim, parse::<T>() with .expect(), reading several lines, `lines()` over stdin.lock(), split_whitespace for several values per line, read_to_string for all input, a first look at handling bad input with match on parse (Ok/Err).

**Ownership**
- `09-ownership` ownership: stack vs heap, the ownership rules, moves (String, Vec), Copy types, clone, ownership through function calls (taking and giving back), drop at end of scope.
- `10-borrowing` borrowing: references &T, mutable references &mut T, the rules (many readers or one writer), borrows end at last use, dangling references prevented, passing &mut to functions, `*` to modify through &mut.
- `11-slices` slices: string slices `&s[a..b]`, &str parameters (prefer &str over &String), array/Vec slices &[T] and &mut [T], first_word, split_at, slices in functions, iterating slices.

**Structuring data**
- `12-structs` structs: defining, creating, field access, mutable structs, field init shorthand, struct update syntax, tuple structs (newtypes), unit structs, #[derive(Debug)], {:?} and {:#?}, dbg!.
- `13-methods` methods: impl blocks, &self / &mut self / self, associated functions (new), Self, getters, multiple impl blocks, method chaining.
- `14-enums` enums: enums, variants with data, match, Option<T> (Some/None, no null), exhaustiveness and `_`, if let, let...else, methods on enums, matches!.
- `15-patterns` patterns: literals, ranges, `|`, guards, @ bindings, destructuring structs/tuples/nested enums, `..` and `_`, while let, slice patterns, refutable vs irrefutable.
- `16-vectors` vectors: Vec::new/vec!, push/pop, indexing vs .get(), iterating (&v, &mut v, v), modifying elements, insert/remove/swap/truncate/clear, sort/sort_by/sort_by_key/sort_unstable/reverse/dedup/retain, contains/binary_search, Vec of structs, with_capacity.
- `17-hashmaps` hashmaps: HashMap (insert/get/contains_key/remove/entry().or_insert/and_modify), counting, iteration order is unspecified (sort keys or use BTreeMap), HashSet (insert returns bool, union/intersection/difference), BTreeMap/BTreeSet, keys need Hash + Eq.
- `18-strings` strings: UTF-8, bytes vs chars, no s[0] indexing, char_indices, slicing at char boundaries, split/split_whitespace/lines/splitn, trim variants, find/starts_with/ends_with, parse, building strings (with_capacity, push_str, collect), format! width/alignment/fill/precision/sign/hex/binary, Display vs Debug.

**Errors and abstraction**
- `19-errors` errors: panic! vs Result, Result<T, E>, match on Result, unwrap/expect (and when they're fine), the ? operator, main returning Result, Option/Result combinators (map, and_then, unwrap_or, unwrap_or_else, unwrap_or_default, ok_or, map_err, ok, filter), parse errors.
- `20-custom-errors` custom-errors: enum error types, impl Display, impl std::error::Error, From conversions so ? converts, Box<dyn Error>, source chains, panic vs Result guidelines, a first look at thiserror and anyhow.
- `21-generics` generics: generic functions and bounds (T: PartialOrd + Copy), generic structs and enums, impl<T>, impls for a concrete type, monomorphization (zero cost), const generics ([T; N]).
- `22-traits` traits: defining and implementing traits, default methods, trait bounds, impl Trait (arguments and return types), where clauses, multiple bounds, derive (Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default), implementing Display.
- `23-std-traits` std-traits: From/Into, TryFrom/TryInto, FromStr (parse into your own type), AsRef, Default, operator overloading (Add, Sub, Mul, Neg, AddAssign), Index, Ord/PartialOrd for sorting, Drop, Deref (briefly).
- `24-lifetimes` lifetimes: why lifetimes exist, the borrow checker and scopes, annotations (`fn longest<'a>`), structs holding references, the elision rules, 'static, when to return owned data instead, reading lifetime errors.
- `25-trait-objects` trait-objects: dyn Trait, &dyn and Box<dyn Trait>, Vec<Box<dyn Shape>>, static vs dynamic dispatch, dyn compatibility (object safety), choosing enums vs trait objects, downcasting with Any (briefly).

**Idiomatic Rust**
- `26-closures` closures: syntax and inference, capturing (borrow, mutable borrow, move), Fn/FnMut/FnOnce, passing closures to functions (generic F: Fn(i32) -> i32), returning closures (impl Fn, Box<dyn Fn>), closures with sort_by_key/retain/map.
- `27-iterators` iterators: the Iterator trait and next, iter/iter_mut/into_iter, adapters (map, filter, filter_map, enumerate, zip, chain, take, skip, take_while, rev, step_by, flat_map, flatten, peekable, windows, chunks), consumers (collect into Vec/String/HashMap, sum, product, count, min/max/min_by_key/max_by, fold, any/all, find, position), laziness, implementing Iterator.
- `28-modules` modules: mod blocks, pub and privacy, paths (crate::, super::, self::), use (braces, globs, as), re-exports (pub use), pub(crate), struct field privacy, packages vs crates, library vs binary crates, module files (src/foo.rs), Cargo.toml dependencies, semver, features, workspaces, publishing on crates.io.
- `29-smart-pointers` smart-pointers: Box<T> (heap, recursive types, trees), Deref and Drop, Rc<T>, RefCell<T> (interior mutability, runtime borrow checks), Rc<RefCell<T>>, Weak<T> (cycles, parent links), Cell<T>.
- `30-testing` testing: #[test], assert!/assert_eq!/assert_ne! with messages, #[should_panic], tests returning Result, #[cfg(test)] mod tests and use super::*, unit vs integration tests (tests/), doc tests, test organization, TDD. Uses `mode: test` exercises with mutants.

**Concurrency and async**
- `31-threads` threads: thread::spawn and join (returning values), move closures, thread::scope (borrowing local data), sleep, Send/Sync, panics in threads.
- `32-sync` sync: channels (mpsc: send/recv, multiple producers, iterating the receiver), Arc, Mutex (lock, guards), Arc<Mutex<T>>, RwLock, atomics (AtomicUsize, Ordering basics), deadlocks, OnceLock/LazyLock.
- `33-async` async: futures and laziness, async fn and .await, #[tokio::main], tokio::spawn and JoinHandle, join!/try_join!, tokio::time::sleep, tokio::sync::mpsc, select! (basics), async vs threads, spawn_blocking.

**Advanced Rust**
- `34-advanced-traits` advanced-traits: associated types, default generic parameters, supertraits, fully qualified syntax, blanket impls, the newtype pattern and the orphan rule, type aliases, PhantomData and typestate, the never type, Sized/?Sized.
- `35-macros` macros: macro_rules! (matchers, fragment specifiers, repetition, recursion, hygiene), writing a hashmap!/vec-like macro, small DSLs; procedural macros (derive, attribute, function-like) explained, how #[derive(Serialize)] works.
- `36-unsafe` unsafe: what unsafe allows, raw pointers, unsafe fn and blocks, building safe abstractions (split_at_mut), SAFETY comments, extern "C" and FFI (calling C's `abs`), static mut vs atomics, Miri, when unsafe is (rarely) right at work.
- `37-performance` performance: release builds, avoiding clones and allocations, &str/slice parameters, with_capacity, iterator chains are zero-cost, Cow, choosing collections, algorithmic complexity first, profiling and benchmarking tools (criterion, perf, flamegraph) explained.

**Rust at work**
- `38-serde` serde: derive Serialize/Deserialize, serde_json to_string/to_string_pretty/from_str, rename/rename_all, Option fields and default, skip_serializing_if, enum representations, serde_json::Value and json!, handling invalid JSON, other formats (TOML, YAML) mentioned.
- `39-app-errors` app-errors: thiserror for library errors (#[derive(Error)], #[error], #[from]), anyhow for applications (anyhow::Result, context/with_context, bail!, ensure!), main returning anyhow::Result, printing error chains, choosing between them.
- `40-cli` cli: std::env::args, parsing flags, exit codes (process::exit, ExitCode), eprintln! for errors, reading files with std::fs (tests provide files), reading stdin, a Config struct and run() -> Result, clap derive (shown, not compiled).
- `41-web` web: HTTP basics, request/response types, routing by (method, path), JSON in and out with serde, handler functions returning Result with status codes, shared state with Arc<Mutex<_>>, how this maps to axum (shown, not compiled), testing handlers.
- `42-patterns` patterns: builder, newtype, typestate, RAII guards, From/Into conversions, Default, returning iterators, AsRef parameters, enum state machines, making invalid states unrepresentable, clippy's common suggestions, a code-review refactor.
- `43-tooling` tooling: dbg!, eprintln!, reading panics and RUST_BACKTRACE, debuggers (rust-gdb/rust-lldb, VS Code + rust-analyzer), logging with log/tracing (shown), rustdoc (///, //!, doc tests), clippy and rustfmt configuration, CI basics.

**Algorithms and data structures**
- `44-collections-more` collections-more: stacks with Vec, VecDeque queues, BinaryHeap and Reverse (priority queues), BTreeMap/BTreeSet ranges, LinkedList (and why it's rare), choosing a collection.
- `45-algorithms` algorithms: Big-O, linear and binary search (binary_search, partition_point), sorting (and implementing insertion/merge sort), custom orderings, two pointers, sliding window, prefix sums.
- `46-recursion-dp` recursion-dp: recursion, memoization with HashMap, bottom-up DP tables, classic problems (fibonacci, coin change, longest common subsequence, knapsack, grid paths).
- `47-graphs` graphs: adjacency lists, BFS with VecDeque (shortest paths), DFS (recursive and with a stack), topological sort, Dijkstra with BinaryHeap, binary trees with Option<Box<Node>>, union-find.
- `48-capstone` capstone: mixed workplace-style tasks combining everything (log analyzer, inventory with traits and errors, refactoring for review).

## YAML pitfalls

- A plain value containing `: ` (colon space), or starting with a backtick, `*`, `&`, `!`, `{`,
  `[`, `@` or `%`, must be quoted. `python3 scripts/yaml-quote-fix.py content/lessons/NN-x.yaml content/drills/x.yaml`
  quotes `why:`/`prompt:`/`title:`/`message:`/hint lines for you. Multi-line `|` blocks are always safe.
- Inside double quotes, a backslash starts an escape (`"\\n"` is backslash-n). Inside single quotes
  nothing is escaped except `''` for a quote. For regexes, prefer single quotes.
- Hints containing code with quotes are easiest as `- "Use \`println!(\"{}\", x)\`"` or as a `|` block.
- `fix:` lines of bug drills: single-quote them if they contain `"`.
