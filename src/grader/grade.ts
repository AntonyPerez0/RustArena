import { cargoTest, compileAndRun, type RunCase, type RunResult } from "../runner/client";
import type { Exercise } from "../content/types";
import { applyMutant, checkRules, hasMain, normalizeOutput, parseLibtest, parsePanic, runCase, testLabel, withHiddenTests } from "./assemble.js";
import { parseDiagnostics, type Diagnostic } from "./friendly";

export type TestResult = {
  name: string;
  pass: boolean;
  hidden?: boolean;
  stdin?: string;
  files?: Record<string, string>;
  args?: string[];
  expected?: string;
  got?: string;
  note?: string;
};

export type GradeResult = {
  status: "pass" | "fail" | "compile-error" | "internal-error";
  diagnostics: Diagnostic[];
  rawDiagnostics: string;
  tests: TestResult[];
  ruleProblems: string[];
  output: string;
  compileMs: number;
  runNote?: string;
};

/** A panic message, pointing at the learner's line when the panic happened in their code. */
function panicNote(stderr: string, userLines: number): string | undefined {
  const p = parsePanic(stderr);
  if (!p) return undefined;
  const where = p.line <= userLines ? ` at line ${p.line}` : "";
  const msg = p.message.split("\n")[0];
  if (/attempt to (add|subtract|multiply|negate|shift).* with overflow/.test(msg))
    return `Panic${where}: ${msg}. The result doesn't fit in the integer type (debug builds check this). Use a bigger type, or checked_/wrapping_/saturating_ arithmetic.`;
  if (/index out of bounds/.test(msg)) return `Panic${where}: ${msg}. Valid indexes run from 0 to len - 1; use .get(i) to get an Option instead of panicking.`;
  if (/called `Option::unwrap\(\)` on a `None` value/.test(msg)) return `Panic${where}: ${msg}. Something you unwrapped was None. Handle the None case with match, if let, or unwrap_or.`;
  if (/called `Result::unwrap\(\)` on an `Err` value/.test(msg)) return `Panic${where}: ${msg}. Something you unwrapped was an Err. Handle the error with match or ?, or check the input.`;
  if (/attempt to divide by zero/.test(msg)) return `Panic${where}: attempt to divide by zero.`;
  if (/already (mutably )?borrowed/.test(msg)) return `Panic${where}: ${msg}. A RefCell was borrowed mutably while another borrow was still alive.`;
  return `The program panicked${where}: ${p.message}`;
}

export function describeRun(r: RunResult | undefined, userLines = Infinity): string | undefined {
  if (!r) return "The program did not run (a previous test ran out of time, so the rest were skipped).";
  if (r.timedOut) return "Time limit exceeded: the program ran for over 2 seconds. Look for a loop that never ends, or one that waits for input that never comes.";
  if (r.truncated) return "Output limit hit (over 64 KB printed). Probably a loop that never stops printing.";
  if (r.crash === "stack-overflow") return "Crash: stack overflow. Recursion that never reaches its base case?";
  if (r.crash === "panic") return panicNote(r.stderr, userLines);
  if (r.crash === "killed") return "The program was stopped (it may have used too much memory).";
  if (r.crash) return "Crash: " + r.crash;
  if (r.exitCode && r.exitCode !== 0) {
    const err = r.stderr.trim();
    return `The program exited with code ${r.exitCode}.` + (err ? ` It printed this error: ${err.split("\n").slice(0, 3).join(" ")}` : "");
  }
  return undefined;
}

function fail(status: "compile-error" | "internal-error", diagnostics: Diagnostic[], raw: string, ruleProblems: string[], compileMs: number): GradeResult {
  return { status, diagnostics, rawDiagnostics: raw, tests: [], ruleProblems, output: "", compileMs };
}

export async function grade(ex: Exercise, code: string): Promise<GradeResult> {
  const userLines = code.split("\n").length;
  const ruleProblems = checkRules(code, ex.require, ex.forbid);
  if (ex.mode === "test") return gradeTests(ex, code, ruleProblems);
  const harness = ex.mode === "harness";
  if (!harness && !hasMain(code)) ruleProblems.push("Your program needs a `fn main()`: that's where every Rust program starts running.");
  const res = await compileAndRun(code, harness ? { harness: ex.harness, cases: [runCase(ex.tests[0])] } : { cases: ex.tests.map(runCase) });
  const diagnostics = parseDiagnostics(res.diagnostics, userLines);
  if (res.internalError) return fail("internal-error", diagnostics, res.internalError, ruleProblems, res.compileMs);
  if (!res.compiled) return fail("compile-error", diagnostics, res.diagnostics, ruleProblems, res.compileMs);
  let tests: TestResult[];
  let output = "";
  let runNote: string | undefined;
  if (harness) {
    const names = ex.checkNames ?? [];
    tests = names.map((name, i) => {
      const r = res.runs[i];
      const pass = !!r && !r.timedOut && !r.crash && r.exitCode === 0;
      if (pass) return { name, pass };
      const p = r ? parsePanic(r.stderr) : null;
      if (p && p.expected !== undefined && p.line > userLines) {
        return { name, pass, expected: p.ne ? `anything but ${p.expected}` : p.expected, got: p.got, note: p.label ? `Checked: ${p.label}` : undefined };
      }
      if (p && p.line > userLines) return { name, pass, note: p.message.split("\n")[0] };
      return { name, pass, note: describeRun(r, userLines) };
    });
    output = res.runs.map((r) => r?.stdout ?? "").join("").replace(/^\n+|\n+$/g, "");
  } else {
    output = res.runs[0]?.stdout ?? "";
    tests = ex.tests.map((t, i) => {
      const r = res.runs[i];
      const got = r ? normalizeOutput(r.stdout) : "";
      const note = describeRun(r, userLines);
      const exitOk = t.exit == null ? (r?.exitCode ?? 0) === 0 : r?.exitCode === t.exit;
      const pass = !!r && !r.timedOut && (t.exit != null || !r.crash) && got === t.expect && exitOk;
      const exitNote = r && !exitOk && t.exit != null ? `The program exited with code ${r.exitCode ?? "(killed)"}, but this test expects exit code ${t.exit}.` : undefined;
      return { name: t.name, pass, hidden: t.hidden, stdin: t.stdin, files: t.files, args: t.args, expected: t.expect, got, note: pass ? undefined : exitNote ?? note };
    });
    const first = res.runs[0];
    runNote = first && ex.tests[0]?.exit != null && first.exitCode === ex.tests[0].exit ? undefined : describeRun(first, userLines);
  }
  const allPass = tests.length > 0 && tests.every((t) => t.pass) && ruleProblems.length === 0;
  return { status: allPass ? "pass" : "fail", diagnostics, rawDiagnostics: res.diagnostics, tests, ruleProblems, output, compileMs: res.compileMs, runNote };
}

/** `cargo test` exercises: the learner's own tests must pass, hidden tests too, and they must catch each broken version. */
async function gradeTests(ex: Exercise, code: string, ruleProblems: string[]): Promise<GradeResult> {
  const userLines = code.split("\n").length;
  const res = await cargoTest(withHiddenTests(code, ex.harness));
  const diagnostics = parseDiagnostics(res.diagnostics, userLines);
  if (res.internalError) return fail("internal-error", diagnostics, res.internalError, ruleProblems, res.compileMs);
  if (!res.compiled) return fail("compile-error", diagnostics, res.diagnostics, ruleProblems, res.compileMs);
  const parsed = parseLibtest(res.output);
  const tests: TestResult[] = parsed.tests.map((t: { name: string; status: string; output: string }) => {
    const hidden = t.name.startsWith("arena_hidden_tests::");
    const name = hidden ? `Hidden check: ${testLabel(t.name.replace(/^.*::/, ""))}` : `Your test ${t.name.replace(/^tests::/, "")}`;
    const p = parsePanic(t.output);
    const note = t.status === "ok" ? undefined : t.status === "ignored" ? "This test is #[ignore]d, so it didn't run." : p ? p.message.split("\n").slice(0, 4).join("\n") : t.output.split("\n").slice(0, 4).join("\n");
    return { name, pass: t.status === "ok", note };
  });
  const own = parsed.tests.filter((t: { name: string }) => !t.name.startsWith("arena_hidden_tests::")).length;
  if (own < (ex.minTests ?? 1)) ruleProblems.push(`Write at least ${ex.minTests ?? 1} \`#[test]\` function${(ex.minTests ?? 1) > 1 ? "s" : ""} (found ${own}).`);
  if (!parsed.finished) tests.push({ name: "All tests finish", pass: false, note: "The test run didn't finish: a test may loop forever." });
  // Each mutant is the code with a bug planted; a good test suite fails on it.
  const mutants = ex.mutants ?? [];
  if (tests.every((t) => t.pass) && mutants.length) {
    for (const m of mutants) {
      const broken = applyMutant(code, m);
      if (broken == null) {
        tests.push({ name: `Catches the bug: ${m.name}`, pass: false, note: "The function under test was changed, so the planted bug couldn't be inserted. Keep the given function as it is and only add tests." });
        continue;
      }
      const r = await cargoTest(broken);
      const caught = !r.internalError && (!r.compiled || parseLibtest(r.output).tests.some((t: { status: string }) => t.status === "failed"));
      tests.push({ name: `Catches the bug: ${m.name}`, pass: caught, note: caught ? undefined : "With this bug planted in the code, all your tests still pass. Add a test that would notice it." });
    }
  }
  const allPass = tests.length > 0 && tests.every((t) => t.pass) && ruleProblems.length === 0;
  return { status: allPass ? "pass" : "fail", diagnostics, rawDiagnostics: res.diagnostics, tests, ruleProblems, output: "", compileMs: res.compileMs };
}

/** Free-run: compile and run with custom stdin, no grading. */
export async function runOnly(code: string, input: RunCase) {
  const res = await compileAndRun(code, { cases: [input] });
  const lines = code.split("\n").length;
  return {
    compiled: res.compiled,
    /** Set when the code couldn't be run at all (it isn't the program's fault). */
    internalError: res.internalError,
    diagnostics: parseDiagnostics(res.diagnostics, lines),
    rawDiagnostics: res.internalError ?? res.diagnostics,
    run: res.runs[0],
    note: res.compiled ? describeRun(res.runs[0], lines) : undefined,
  };
}
