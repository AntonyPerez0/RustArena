// Runs Rust code. The site has no server of its own: programs are compiled and run by the
// Rust Playground (play.rust-lang.org), the same service that runs the examples in The Rust
// Programming Language book. A school or company can point the site at their own copy of the
// Playground, or at `npm run runner` (scripts/local-runner.mjs) on their machine, in Settings.
import { cleanCompilerOutput, parseRuns, wrapProgram } from "../grader/assemble.js";
import { getState } from "../state/store";

export const DEFAULT_RUNNER = "https://play.rust-lang.org";

/** One run's input: stdin plus optional starter files and command-line arguments. */
export type RunCase = { stdin?: string; files?: Record<string, string>; args?: string[] };

export type RunnerStatus =
  | { state: "idle" }
  | { state: "busy" }
  | { state: "ok"; ms: number }
  | { state: "error"; message: string };

export type RunResult = {
  stdout: string;
  stderr: string;
  exitCode: number | null;
  /** Why the program stopped early, in a word: "panic", "stack-overflow", "killed"; null if it finished. */
  crash: string | null;
  timedOut: boolean;
  truncated: boolean;
  ms: number;
};

export type CompileRunResult = {
  compiled: boolean;
  /** The compiler's messages (errors and warnings), Cargo's progress lines removed. */
  diagnostics: string;
  /** Set when the code couldn't be run at all (network trouble, server down): not the learner's fault. */
  internalError?: string;
  compileMs: number;
  runs: RunResult[];
};

type PlaygroundResponse = { success: boolean; exitDetail?: string; stdout: string; stderr: string };

let status: RunnerStatus = { state: "idle" };
const listeners = new Set<() => void>();

function setStatus(s: RunnerStatus) {
  status = s;
  listeners.forEach((l) => l());
}

export function getRunnerStatus() {
  return status;
}

export function subscribeRunner(fn: () => void) {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

export function runnerUrl(): string {
  const u = getState().settings.runnerUrl?.trim();
  return (u || DEFAULT_RUNNER).replace(/\/+$/, "");
}

export function runnerName(): string {
  const u = runnerUrl();
  if (u === DEFAULT_RUNNER) return "Rust Playground";
  try {
    return new URL(u).host;
  } catch {
    return u;
  }
}

// The same program and input always gives the same result, so answers are kept for the session:
// checking unchanged code again is instant and costs the Playground nothing.
const cache = new Map<string, PlaygroundResponse>();
const CACHE_MAX = 60;

// One request at a time: the Playground is a shared, free service.
let chain: Promise<unknown> = Promise.resolve();

async function post(url: string, body: unknown, timeoutMs: number): Promise<Response> {
  const ctl = new AbortController();
  const t = setTimeout(() => ctl.abort(), timeoutMs);
  try {
    return await fetch(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body), signal: ctl.signal });
  } finally {
    clearTimeout(t);
  }
}

async function callPlayground(code: string, tests: boolean): Promise<PlaygroundResponse> {
  const base = runnerUrl();
  const req = { channel: "stable", mode: "debug", edition: "2024", crateType: "bin", tests, code, backtrace: false };
  const key = base + "\n" + JSON.stringify(req);
  const hit = cache.get(key);
  if (hit) return hit;
  let res: Response;
  try {
    res = await post(base + "/execute", req, 45000);
  } catch (e) {
    // Older Playground deployments only allow cross-site requests to /evaluate.json. It can't run
    // test binaries, but the site's own runner prints everything to stdout, so nothing is lost.
    if (tests) throw e;
    const legacy = await post(base + "/evaluate.json", { version: "stable", optimize: "0", edition: "2024", code }, 45000);
    if (!legacy.ok) throw new Error(`the server answered ${legacy.status}`);
    const j = (await legacy.json()) as { result?: string; error?: string | null };
    const out: PlaygroundResponse = j.error ? { success: false, stdout: "", stderr: j.error } : { success: true, stdout: j.result ?? "", stderr: "" };
    remember(key, out);
    return out;
  }
  if (res.status === 429) throw new Error("the Rust Playground is busy (too many requests). Wait a few seconds and try again");
  if (!res.ok) throw new Error(`the server answered ${res.status} ${res.statusText}`.trim());
  const out = (await res.json()) as PlaygroundResponse;
  if (typeof out.stdout !== "string" || typeof out.stderr !== "string") throw new Error("the server's answer wasn't a Playground response");
  remember(key, out);
  return out;
}

function remember(key: string, v: PlaygroundResponse) {
  cache.set(key, v);
  if (cache.size > CACHE_MAX) cache.delete(cache.keys().next().value as string);
}

function describeError(e: unknown): string {
  if (e instanceof DOMException && e.name === "AbortError") return "the request timed out (no answer in 45 seconds)";
  if (e instanceof TypeError) return navigator.onLine === false ? "you're offline. Running code needs an internet connection" : `couldn't reach ${runnerName()} (network error)`;
  return e instanceof Error ? e.message : String(e);
}

function toRun(r: ReturnType<typeof parseRuns>["runs"][number]): RunResult {
  let crash: string | null = null;
  if (r.timedOut) crash = null;
  else if (/has overflowed its stack/.test(r.stderr)) crash = "stack-overflow";
  else if (r.exitCode === 101 && /panicked at/.test(r.stderr)) crash = "panic";
  else if (r.exitCode == null) crash = r.signal === 9 ? "killed" : `signal ${r.signal}`;
  return { stdout: r.stdout, stderr: r.stderr, exitCode: r.exitCode, crash, timedOut: r.timedOut, truncated: r.truncated, ms: r.ms };
}

/**
 * Compile the learner's code once and run it on every case (stdout mode), or run every hidden
 * test function (when `harness` is given). Never throws: trouble reaching the runner comes back
 * as `internalError`.
 */
export function compileAndRun(code: string, opts: { cases?: RunCase[]; harness?: string } = {}): Promise<CompileRunResult> {
  const job = chain.then(async (): Promise<CompileRunResult> => {
    const t0 = performance.now();
    setStatus({ state: "busy" });
    const program = wrapProgram(code, { cases: opts.cases?.length ? opts.cases : [{}], harness: opts.harness });
    try {
      const res = await callPlayground(program, false);
      const ms = performance.now() - t0;
      setStatus({ state: "ok", ms });
      const diagnostics = cleanCompilerOutput(res.stderr);
      const parsed = parseRuns(res.stdout);
      if (!parsed.runs.length) {
        // No records: it didn't compile (or the runner itself failed).
        if (parsed.error) return { compiled: true, diagnostics, internalError: parsed.error, compileMs: ms, runs: [] };
        if (/^error(\[E\d+\])?:/m.test(diagnostics) || !res.success) return { compiled: false, diagnostics, compileMs: ms, runs: [] };
        return { compiled: true, diagnostics, internalError: "the program produced no results", compileMs: ms, runs: [] };
      }
      return { compiled: true, diagnostics, compileMs: ms, runs: Array.from(parsed.runs, (r) => (r ? toRun(r) : undefined)).filter((r): r is RunResult => !!r) };
    } catch (e) {
      const message = describeError(e);
      setStatus({ state: "error", message });
      return { compiled: false, diagnostics: "", internalError: message, compileMs: performance.now() - t0, runs: [] };
    }
  });
  chain = job.catch(() => {});
  return job;
}

/** Plain `cargo test` on the learner's code (for steps where they write the tests). */
export function cargoTest(code: string): Promise<{ compiled: boolean; diagnostics: string; output: string; internalError?: string; compileMs: number }> {
  const job = chain.then(async () => {
    const t0 = performance.now();
    setStatus({ state: "busy" });
    try {
      const res = await callPlayground(code, true);
      const ms = performance.now() - t0;
      setStatus({ state: "ok", ms });
      const diagnostics = cleanCompilerOutput(res.stderr);
      const compiled = /running \d+ tests?/.test(res.stdout);
      return { compiled, diagnostics, output: res.stdout, compileMs: ms };
    } catch (e) {
      const message = describeError(e);
      setStatus({ state: "error", message });
      return { compiled: false, diagnostics: "", output: "", internalError: message, compileMs: performance.now() - t0 };
    }
  });
  chain = job.catch(() => {});
  return job;
}
