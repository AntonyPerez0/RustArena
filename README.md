# RustArena

Learn Rust from zero to job-ready, freeCodeCamp style, with **every exercise compiled and tested by the real Rust compiler**.

RustArena is the Rust sister site of [C/C++ Arena](https://github.com/AntonyPerez0/cpp-arena) and [Java Arena](https://github.com/AntonyPerez0/Java-Arena), with the same design, and a curriculum that follows the community [Rust developer roadmap](https://roadmap.sh/rust).

- **Learn**: 48 modules from `println!` to professional Rust. Foundations (variables, types, functions, control flow, compound types, strings, input), then the heart of Rust (ownership, borrowing, slices), structuring data (structs, methods, enums, pattern matching, vectors, hash maps, strings in depth), errors and abstraction (Result and `?`, custom errors, generics, traits, standard traits, lifetimes, trait objects), idiomatic Rust (closures, iterators, modules and Cargo, smart pointers, testing), concurrency and async (threads, channels and shared state, async/await with Tokio), advanced Rust (advanced traits and type-level programming, macros, unsafe and FFI, performance), Rust at work (serde, anyhow and thiserror, command-line tools, web services, idiomatic patterns and code review, debugging and tooling), and algorithms for interviews, ending in a capstone. Early steps are fill-in-the-blank; later steps have you write whole programs, functions checked by hidden `#[test]`s, and your own test suites graded against planted bugs. Every step has hints you reveal one at a time and a "show solution" escape hatch.
- **Roadmap** (`/roadmap`): every topic on roadmap.sh/rust, in order, linked to the lessons and projects that teach it, with your progress.
- **Deathmatch**: endless reps drawn from what you've learned: predict the output, fill the token, spot the bug, and "does this compile?" against the borrow checker. Each drill unlocks when you finish the lesson step that teaches it. Every 8th rep is a boss rep you compile for real. One life (ranked), three lives (casual), a spaced-review warm-up, or interview prep.
- **Projects**: guided multi-milestone builds (a to-do CLI, a bank ledger, a tokenizer and calculator, a JSON inventory, a log analyzer, a task scheduler and more). Your code carries forward between milestones.
- **Pro Track**: 12 projects done on a real machine (GitHub Codespaces or your own computer) with the tools Rust teams use: rustup and Cargo, Git and pull requests, rustfmt and clippy, unit, integration and property tests (graded by mutation testing), clap, a library crate with typed errors and docs, threads and rayon, Tokio networking, an axum REST API, SQLx and SQLite, tracing and configuration, criterion benchmarks and profiling, and a production-ready capstone service with Docker and CI. Each project is graded by GitHub Actions in the learner's own repository. See [`pro-track/`](pro-track/README.md).
- **Playground** (`/playground`): run any Rust program with input and command-line arguments, with serde, tokio, anyhow, thiserror, regex, itertools, rayon and chrono available, compiler errors explained, panics described, and share links that carry the code.
- **Topics**: short reference pages (`/topics/...`) on the core ideas (ownership, the borrow checker's most common errors, lifetimes, traits, Option and Result, iterators, smart pointers, async...), each with a compiled and tested example. A "Where to go next" page (`/next`) collects books, practice sites, projects and job advice.
- **Daily challenge, placement quiz and certificates**: one problem a day with a streak, a quiz that lets experienced learners skip ahead, and printable certificates for finishing the course and the Pro Track.
- **Comfort**: light and dark themes in Rust's colors, adjustable text size, a symbol bar for phone keyboards (`&`, `::`, `=>`, `'a`...), "Report a problem" on every exercise, progress sync (transfer link or QR code, or a private GitHub Gist), and offline reading once visited.

## How code runs

RustArena is a static site with no server of its own. When a learner presses **Check** or **Run**, the site sends the code to the [Rust Playground](https://play.rust-lang.org) (the official service that runs the examples in The Rust Programming Language book), which compiles it with the current stable `rustc` (edition 2024, debug build) and runs it.

- **One request per check.** The site wraps the learner's code with a small hidden runner (`src/grader/assemble.js`): the learner's `fn main` is renamed, and a new `main` starts the program once per test case as a child process, each with its own stdin, command-line arguments, files in a private working folder, a 2 second time limit and a 64 KB output cap. It prints one record per run (exit code, signal, timing, and base64 stdout/stderr), so one Playground request grades every test, and stdin works even though the Playground has no stdin.
- **Function exercises** append hidden `#[test]`-style functions; each runs as its own child process, so a failed `assert_eq!` fails only that check, and the site shows "expected X, got Y" from the panic message.
- **"Write the tests" exercises** run real `cargo test` on the Playground, then run again with each planted bug (mutant) to check the learner's tests catch it.
- **Errors**: rustc's messages are shown with their snippet, a plain-English note for common error codes (E0382 moved value, E0502 borrow conflict, E0106 missing lifetime...), the line marked in the editor, and a link to the official error index. Panics are explained with the learner's line.
- **Caching and courtesy**: identical requests are answered from memory for the session, and requests go one at a time.
- **Your own runner**: `npm run runner` starts `scripts/local-runner.mjs`, which serves the same API using your own Rust installation (and the crates in `runner-deps/`). In the site, **Settings > Where your code runs** switches to it (or to any self-hosted Playground). It works offline and suits classrooms.

## Deploy to GitHub Pages

The workflow in `.github/workflows/deploy.yml` runs on every push to `main`. It installs dependencies and stable Rust, compiles and runs every lesson solution, worked example, drill and project milestone with the real Rust compiler (the build fails if any expected output is wrong), builds the site, runs the browser tests (including the accessibility checks, with code run by the local runner), and publishes the result to the `gh-pages` branch. Under **Settings > Pages**, set **Source** to **Deploy from a branch**, **gh-pages**, **/ (root)**. The first publish usually sets this automatically.

The site is published at `https://rustarena.app`. Without a custom domain it would be at `https://<you>.github.io/<repo>/`. To set up a domain (bought and managed on Cloudflare), follow [docs/CUSTOM-DOMAIN.md](docs/CUSTOM-DOMAIN.md). This site uses `rustarena.app`, set as `CUSTOM_DOMAIN` at the top of the workflow.

## Run locally

Needs Node 22 and Rust (stable, via [rustup](https://rustup.rs)).

```bash
npm install
npm run content    # compile and run every lesson, drill and project; writes src/generated/
npm run dev        # http://localhost:5173/
npm run runner     # optional: run code on this machine instead of the Playground (then pick it in Settings)
```

| Script | What it does |
|---|---|
| `npm run content` | Builds `src/generated/content.json` from `content/**/*.yaml`, compiling and running everything with your Rust toolchain. `node scripts/build-content.mjs --only <path part>` checks a few files without writing anything. |
| `npm run runner` | A local stand-in for the Rust Playground on `127.0.0.1:8787` |
| `npm run build` | Type-checks and builds `dist/`, packs the Pro Track starter, and pre-renders every page |
| `npm run test:e2e` | Serves `dist/` and drives the real UI in headless Chromium (code runs on the local runner), including axe accessibility scans (WCAG 2.2 AA) of every page type in both themes, phone widths, offline use and SEO checks |
| `npm run check:pro` | Checks every Pro Track grader: the starter must fail and the reference solution must pass |

## Branding and credits

- **Colors** come from Rust and Ferris: Ferris orange `#F74C00`, amber `#F9A11B` and rust red `#B7410E`, on warm darks and creams, with every text color at WCAG AA contrast.
- **The logo** is our own drawing of a crab (`src/components/Brand.tsx`; `scripts/make-brand-images.mjs` renders the favicon, app icons and share image). The official **Rust logo** is a trademark of the Rust Foundation, and its trademark policy restricts using it in ways that suggest endorsement, so RustArena doesn't use it. RustArena uses the word "Rust" only to say what it teaches, and states that it isn't affiliated with or endorsed by the Rust Foundation or the Rust Project.
- **Ferris the crab** (`public/ferris.png`), the community's unofficial mascot by Karen Rustad Tölva, is in the public domain (CC0), from [rustacean.net](https://rustacean.net/).
- The curriculum order follows [roadmap.sh/rust](https://roadmap.sh/rust) (not affiliated).

## How it works

- **Content** (`content/`): YAML lessons, drills, projects, topics, the roadmap, placement quiz and Pro Track list. `scripts/build-content.mjs` compiles and runs every reference solution through the same wrapper the browser uses, so expected outputs come from real runs; checks that starter code doesn't already pass, that worked examples print what the text says (and that "doesn't compile" examples really fail with the error shown), that solutions have no compiler warnings, and that output is deterministic across runs. Results are cached by program text in `node_modules/.cache/rustarena`.
- **Crates**: `runner-deps/Cargo.toml` lists the crates lesson code may use (all available on the Playground). The build and local runner compile them once with Cargo and link each program against that build with plain `rustc`.
- **Progress**: stored in the browser's localStorage. The Profile page moves it between devices with a transfer link (also as a QR code), a private GitHub Gist, or a file. Imports merge rather than overwrite.
- **Offline**: `public/sw.js` saves the app when it installs, so lessons and pages open offline after one visit. Running code needs the runner (online, unless you run your own).
- **Accessibility** targets WCAG 2.2 AA: a skip link, focus moved to each new page's heading, visible focus rings, labeled editor and blanks, results announced to screen readers, keyboard access to every drill, 24px tap targets, sufficient contrast (including syntax colors), and reduced-motion support.
- **SEO**: every page has a real URL (`/learn/ownership/<step>`). `scripts/prerender.mjs` writes a static HTML file per route with its own title, description, canonical link, Open Graph tags and structured data (`Course`, `LearningResource`, `TechArticle`), plus `sitemap.xml`, `robots.txt` and `404.html`.

## Writing content

See [`docs/AUTHORING.md`](docs/AUTHORING.md): the YAML formats for lessons, exercises (fill-in, programs with stdin/args/files/exit codes, functions with hidden tests, learner-written tests with mutants), worked examples (`rust run`, `rust panics`, `rust compile_fail`), drills, determinism rules, and the full curriculum outline.

## Known limits

- Running code needs a connection to the Rust Playground (or a runner you host). The Playground is a free, shared service; if it's busy or down, the site says so and suggests the alternatives.
- Programs can't read input interactively; each test supplies stdin in advance. Use "Run with my input" to try your own.
- Only the crates in `runner-deps/Cargo.toml` are used in browser lessons. Frameworks that need a network, a database or several files (axum, sqlx, clap derive in a real crate) are taught hands-on in the Pro Track.
