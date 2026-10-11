import { useCallback, useEffect, useRef, useState } from "react";
import { CircleX, ExternalLink, Play } from "lucide-react";
import CodeEditor from "../components/CodeEditor";
import SymbolBar from "../components/SymbolBar";
import { DiagnosticList, RunnerProblem } from "../components/Results";
import { runOnly } from "../grader/grade";
import { packText, unpackText } from "../lib/pack";
import { useTitle } from "../lib/title";
import { runnerName } from "../runner/client";

type Saved = { code: string; stdin: string; args: string };

const EXAMPLES: { name: string; code: string; stdin?: string; args?: string }[] = [
  {
    name: "Hello, input",
    stdin: "Ferris",
    code: `use std::io;

fn main() {
    println!("What's your name?");
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("failed to read a line");
    println!("Hello, {}!", name.trim());
}
`,
  },
  {
    name: "Structs, traits and iterators",
    code: `use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct Player {
    name: String,
    score: u32,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:<8}{:>5}", self.name, self.score)
    }
}

fn main() {
    let mut players = vec![
        Player { name: "ferris".into(), score: 1200 },
        Player { name: "corro".into(), score: 950 },
        Player { name: "bob".into(), score: 1310 },
    ];
    players.sort_by(|a, b| b.score.cmp(&a.score));
    for (rank, p) in players.iter().enumerate() {
        println!("#{} {p}", rank + 1);
    }
    let total: u32 = players.iter().map(|p| p.score).sum();
    println!("total: {total}");
}
`,
  },
  {
    name: "Error handling with ?",
    stdin: "12\n7\nnot a number\n",
    code: `use std::io::{self, BufRead};
use std::num::ParseIntError;

fn parse_line(line: &str) -> Result<i64, ParseIntError> {
    let n: i64 = line.trim().parse()?;
    Ok(n * 2)
}

fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("read error");
        match parse_line(&line) {
            Ok(v) => println!("{line} doubled is {v}"),
            Err(e) => println!("skipping {line:?}: {e}"),
        }
    }
}
`,
  },
  {
    name: "JSON with serde",
    code: `use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    id: u32,
    item: String,
    quantity: u32,
}

fn main() -> Result<(), serde_json::Error> {
    let json = r#"{ "id": 7, "item": "crab plushie", "quantity": 2 }"#;
    let mut order: Order = serde_json::from_str(json)?;
    order.quantity += 1;
    println!("{order:?}");
    println!("{}", serde_json::to_string_pretty(&order)?);
    Ok(())
}
`,
  },
  {
    name: "Async with Tokio",
    code: `use tokio::time::{sleep, Duration};

async fn fetch(id: u32) -> String {
    sleep(Duration::from_millis(10 * id as u64)).await;
    format!("response {id}")
}

#[tokio::main]
async fn main() {
    // Both requests run at the same time; join! waits for both.
    let (a, b) = tokio::join!(fetch(2), fetch(1));
    println!("{a}, {b}");
}
`,
  },
  {
    name: "Threads",
    code: `use std::thread;

fn main() {
    let data: Vec<u64> = (1..=1_000).collect();
    // Scoped threads can borrow data from the stack safely.
    let total: u64 = thread::scope(|s| {
        let handles: Vec<_> = data
            .chunks(250)
            .map(|chunk| s.spawn(move || chunk.iter().sum::<u64>()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    });
    println!("sum = {total}");
}
`,
  },
  {
    name: "Command-line arguments",
    args: "--shout hello world",
    code: `use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: echo [--shout] words...");
        return ExitCode::from(2);
    }
    let shout = args.iter().any(|a| a == "--shout");
    let words: Vec<&str> = args.iter().filter(|a| !a.starts_with("--")).map(|s| s.as_str()).collect();
    let line = words.join(" ");
    println!("{}", if shout { line.to_uppercase() } else { line });
    ExitCode::SUCCESS
}
`,
  },
];

const KEY = "rustarena-playground";

function load(): Saved {
  const fresh: Saved = { code: EXAMPLES[0].code, stdin: EXAMPLES[0].stdin ?? "", args: "" };
  try {
    const s = JSON.parse(localStorage.getItem(KEY) ?? "null");
    return s && typeof s.code === "string" ? { ...fresh, ...s } : fresh;
  } catch {
    return fresh;
  }
}

function save(s: Saved) {
  try {
    localStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    /* private mode or full: the code just isn't remembered */
  }
}

/** Split an argument line like a shell would for simple cases: spaces separate, quotes group. */
function splitArgs(line: string): string[] {
  const out: string[] = [];
  for (const m of line.matchAll(/"([^"]*)"|'([^']*)'|(\S+)/g)) out.push(m[1] ?? m[2] ?? m[3]);
  return out;
}

type Result = Awaited<ReturnType<typeof runOnly>>;

export default function Playground() {
  useTitle("Rust Playground: write and run Rust online");
  const [state, setState] = useState<Saved>(load);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Result | null>(null);
  const [status, setStatus] = useState("");
  const [shared, setShared] = useState<string | null>(null);
  const busyRef = useRef(false);
  const boxRef = useRef<HTMLDivElement>(null);

  const update = (patch: Partial<Saved>) =>
    setState((s) => {
      const next = { ...s, ...patch };
      save(next);
      return next;
    });

  // A shared link carries the program in the address: #code=<packed JSON>.
  useEffect(() => {
    const m = /^#code=(.+)$/.exec(location.hash);
    if (!m) return;
    unpackText(m[1])
      .then((json) => {
        const p = JSON.parse(json);
        update({ code: String(p.code ?? ""), stdin: String(p.stdin ?? ""), args: String(p.args ?? "") });
        setResult(null);
        setStatus("Loaded the shared program.");
      })
      .catch(() => setStatus("That share link is damaged, so the program couldn't be loaded."));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const run = useCallback(async () => {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(true);
    setStatus("Running your program.");
    try {
      const r = await runOnly(state.code, { stdin: state.stdin, args: splitArgs(state.args) });
      setResult(r);
      setStatus(r.internalError ? "Your code couldn't be run." : !r.compiled ? "It didn't compile." : r.note ? "The program stopped with a problem." : "The program finished.");
    } finally {
      busyRef.current = false;
      setBusy(false);
    }
  }, [state]);

  const share = async () => {
    const url = `${location.origin}${import.meta.env.BASE_URL}playground#code=${await packText(JSON.stringify({ code: state.code, stdin: state.stdin, args: state.args }))}`;
    setShared(url);
    try {
      if (navigator.share) {
        await navigator.share({ title: "My Rust program on RustArena", url });
        setStatus("Shared.");
        return;
      }
    } catch {
      /* cancelled, or not allowed: fall back to copying */
    }
    try {
      await navigator.clipboard.writeText(url);
      setStatus("Link copied. Anyone who opens it sees this program.");
    } catch {
      setStatus("Copy the link below to share this program.");
    }
  };

  // A stable handler for the editor's Ctrl+Enter, so typing doesn't rebuild the editor's extensions.
  const runRef = useRef(run);
  runRef.current = run;
  const runFromEditor = useCallback(() => runRef.current(), []);

  const warnings = result?.compiled ? result.diagnostics.filter((d) => d.severity === "warning") : [];
  const out = result?.run ? result.run.stdout : "";
  const err = result?.run?.stderr.trim() ?? "";
  const officialUrl = `https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&code=${encodeURIComponent(state.code)}`;

  return (
    <div className="playground">
      <div className="page-head">
        <h1>Playground</h1>
        <p className="muted">
          Write any Rust program and run it, compiled by the real Rust compiler (stable, edition 2024) on {runnerName()}. Popular crates like <code>serde</code>, <code>serde_json</code>, <code>tokio</code>,{" "}
          <code>anyhow</code>, <code>thiserror</code>, <code>regex</code>, <code>itertools</code>, <code>rayon</code> and <code>chrono</code> are available. Your code stays in this browser; <b>Share</b>{" "}
          makes a link that carries it.
        </p>
      </div>

      <div className="row pg-examples">
        <label className="lbl" htmlFor="pg-example">
          Start from an example
        </label>
        <select
          id="pg-example"
          className="select"
          value=""
          onChange={(e) => {
            const ex = EXAMPLES[Number(e.target.value)];
            if (!ex) return;
            if (state.code === ex.code || confirm(`Replace your program with the "${ex.name}" example?`)) {
              update({ code: ex.code, stdin: ex.stdin ?? "", args: ex.args ?? "" });
              setResult(null);
            }
          }}
        >
          <option value="">Choose…</option>
          {EXAMPLES.map((ex, i) => (
            <option key={ex.name} value={i}>
              {ex.name}
            </option>
          ))}
        </select>
      </div>

      <div className="workbench" ref={boxRef}>
        <CodeEditor value={state.code} onChange={(v) => update({ code: v })} onRun={runFromEditor} diagnostics={result?.diagnostics} minHeight="320px" label="Your Rust program" />
        <SymbolBar container={boxRef} />
        <div className="pg-io">
          <div>
            <label className="lbl" htmlFor="pg-stdin">
              Input (what the program reads from stdin)
            </label>
            <textarea id="pg-stdin" className="stdin" rows={3} value={state.stdin} onChange={(e) => update({ stdin: e.target.value })} spellCheck={false} />
          </div>
          <div>
            <label className="lbl" htmlFor="pg-args">
              Command-line arguments
            </label>
            <input id="pg-args" className="stdin" value={state.args} onChange={(e) => update({ args: e.target.value })} spellCheck={false} placeholder="--verbose input.txt" />
          </div>
        </div>
        <div className="actions">
          <button className="btn btn-primary" onClick={run} disabled={busy} aria-keyshortcuts="Control+Enter Meta+Enter">
            {busy ? (
              "Compiling and running…"
            ) : (
              <>
                <Play className="icon" aria-hidden="true" /> Run <kbd aria-hidden="true">⌃↵</kbd>
              </>
            )}
          </button>
          <button className="btn" onClick={share}>
            Share
          </button>
          <a className="btn btn-ghost" href={officialUrl} target="_blank" rel="noopener noreferrer">
            Open in play.rust-lang.org <ExternalLink className="icon" aria-hidden="true" />
            <span className="visually-hidden"> (opens in a new tab)</span>
          </a>
        </div>
        {shared && (
          <p className="small">
            <label className="lbl" htmlFor="pg-link">
              Share link
            </label>
            <input id="pg-link" className="share-link" readOnly value={shared} onFocus={(e) => e.currentTarget.select()} />
          </p>
        )}
        <p className="visually-hidden" role="status" aria-live="polite">
          {status}
        </p>
        {result && (
          <section className="results" aria-label="Result">
            {result.internalError ? (
              <RunnerProblem message={result.internalError} />
            ) : !result.compiled ? (
              <>
                <div className="banner banner-fail">
                  <span>
                    <CircleX className="icon" aria-hidden="true" /> It didn't compile
                  </span>
                </div>
                <DiagnosticList diagnostics={result.diagnostics} raw={result.rawDiagnostics} />
              </>
            ) : (
              <>
                <h2 className="h3">Output</h2>
                <pre tabIndex={0} className="console">
                  {out || "(no output)"}
                </pre>
                {err && (
                  <>
                    <h2 className="h3">Standard error</h2>
                    <pre tabIndex={0} className="console console-err">
                      {err}
                    </pre>
                  </>
                )}
                {result.run?.exitCode != null && result.run.exitCode !== 0 && !result.note && <div className="t-note">Exit code {result.run.exitCode}.</div>}
                {result.run?.truncated && <div className="t-note">Output was cut off after 64 KB.</div>}
                {result.note && <div className="t-note">{result.note}</div>}
                {warnings.length > 0 && (
                  <>
                    <h2 className="h3">Warnings</h2>
                    <DiagnosticList diagnostics={warnings} raw="" />
                  </>
                )}
              </>
            )}
          </section>
        )}
      </div>
    </div>
  );
}
