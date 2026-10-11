// Builds src/generated/content.json from content/**/*.yaml.
//
// Every reference solution, drill, project milestone and worked example is compiled and run
// with the real Rust compiler on this machine, through the same hidden runner the site sends
// to the Rust Playground. Expected outputs come from those real runs (or are checked against
// them when the author pinned them), so nothing the site grades against is hand-typed guesswork.
//
// Usage:
//   node scripts/build-content.mjs                 build everything and write src/generated/
//   node scripts/build-content.mjs --only 07-      check only content files whose path contains "07-" (writes nothing);
//                                                  several parts can be given, separated by commas: --only ownership,borrowing
//   node scripts/build-content.mjs --no-cache      ignore cached compiler results
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import YAML from "yaml";
import { assignDrillSteps } from "./drill-steps.mjs";
import { applyStepUrls, readStepUrls, writeStepUrls, STEP_URLS_FILE } from "./step-urls.mjs";
import { execute, EDITION } from "./rust-exec.mjs";
import {
  parseTemplate,
  fillFirstBlank,
  templateSolution,
  wrapProgram,
  parseRuns,
  parsePanic,
  parseLibtest,
  withHiddenTests,
  applyMutant,
  harnessTests,
  harnessTestInfo,
  testLabel,
  normalizeOutput,
  looseOutput,
  checkRules,
  drillProgram,
  drillDisplay,
  runCase,
  cleanCompilerOutput,
  hasMain,
} from "../src/grader/assemble.js";

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const CACHE_DIR = path.join(ROOT, "node_modules", ".cache", "rustarena");
const args = process.argv.slice(2);
const useCache = !args.includes("--no-cache");
const onlyAt = args.indexOf("--only");
const ONLY = onlyAt >= 0 ? args[onlyAt + 1] : null;
const ONLY_PARTS = ONLY ? ONLY.split(",").map((x) => x.trim()).filter(Boolean) : [];
const errors = [];
const warnings = [];
fs.mkdirSync(CACHE_DIR, { recursive: true });

// ---------------------------------------------------------------- running code
const MAX = Math.max(2, os.cpus().length);
let active = 0;
const queue = [];
function limit(fn) {
  return new Promise((resolve, reject) => {
    const go = () => {
      active++;
      fn()
        .then(resolve, reject)
        .finally(() => {
          active--;
          if (queue.length) queue.shift()();
        });
    };
    if (active < MAX) go();
    else queue.push(go);
  });
}

/** Compile and run a complete program (already wrapped), like one Playground request. Cached by content. */
async function play(code, { tests = false } = {}) {
  const key = crypto.createHash("sha1").update(JSON.stringify([code, tests, EDITION, "v1"])).digest("hex");
  const file = path.join(CACHE_DIR, key + ".json");
  if (useCache && fs.existsSync(file)) {
    try {
      return JSON.parse(fs.readFileSync(file, "utf8"));
    } catch {
      /* a half-written cache file: run again */
    }
  }
  const res = await limit(() => execute({ code, tests }));
  // Write then rename, so a parallel build never reads a half-written file.
  const tmp = file + "." + process.pid + ".tmp";
  fs.writeFileSync(tmp, JSON.stringify(res));
  fs.renameSync(tmp, file);
  return res;
}

const firstError = (diag) => diag.split("\n").find((l) => /^error/.test(l)) ?? diag.split("\n")[0] ?? "";

/** Warnings rustc reports in the learner-visible part of the code (the first `lines` lines). */
/** Whether one hidden test passed: it finished cleanly, or (for #[should_panic]) panicked with the expected text. */
function harnessPassed(r, info) {
  if (!r || r.timedOut) return false;
  if (!info?.shouldPanic) return r.exitCode === 0;
  return r.exitCode === 101 && (parsePanic(r.stderr)?.message ?? "").includes(info.expected ?? "");
}

function userWarnings(diag, lines) {
  const out = [];
  const blocks = diag.split(/\n(?=warning|error)/);
  for (const b of blocks) {
    if (!/^warning/.test(b) || /^warning: \d+ warnings? emitted/.test(b) || /generated \d+ warning/.test(b)) continue;
    const loc = b.match(/--> [^:]*main\.rs:(\d+):/);
    if (loc && +loc[1] <= lines) out.push(b.trim());
  }
  return out;
}

/** Run a program on several inputs: { compiled, diagnostics, runs[] }. */
async function runProgram(code, cases) {
  const res = await play(wrapProgram(code, { cases, limitMs: 5000 }));
  const diagnostics = cleanCompilerOutput(res.stderr);
  const { runs, error } = parseRuns(res.stdout);
  if (error) errors.push(`runner error: ${error}`);
  return { compiled: runs.length > 0, diagnostics, runs };
}

/** Run hidden #[test] functions against the code: { compiled, diagnostics, results[] }. */
async function runHarness(code, harness, c) {
  const res = await play(wrapProgram(code, { harness, cases: [c], limitMs: 5000 }));
  const diagnostics = cleanCompilerOutput(res.stderr);
  const { runs } = parseRuns(res.stdout);
  return { compiled: runs.length > 0, diagnostics, runs };
}

// ---------------------------------------------------------------- reading content
function readYamlDir(dir) {
  const full = path.join(ROOT, dir);
  if (!fs.existsSync(full)) return [];
  return fs
    .readdirSync(full)
    .filter((f) => f.endsWith(".yaml") || f.endsWith(".yml"))
    .sort()
    .filter((f) => !ONLY || ONLY_PARTS.some((part) => path.join(dir, f).includes(part)))
    .map((f) => {
      try {
        return { file: path.join(dir, f), data: YAML.parse(fs.readFileSync(path.join(full, f), "utf8")) };
      } catch (e) {
        errors.push(`${dir}/${f}: YAML error: ${e.message}`);
        return null;
      }
    })
    .filter(Boolean);
}

const ensureNl = (s) => (s == null ? s : String(s).replace(/\s*$/, "\n"));

// ---------------------------------------------------------------- exercises
/**
 * Normalise and verify one exercise (lesson step, challenge, project milestone or boss rep).
 * Returns the JSON shape the app consumes.
 */
async function buildExercise(where, raw, prevSolution) {
  const kind = raw.fill != null ? "fill" : "code";
  const mode = raw.mode === "test" ? "test" : raw.harness ? "harness" : "stdout";
  const seed = ensureNl(kind === "fill" ? raw.fill : raw.seed ?? prevSolution ?? "");
  const solution = ensureNl(kind === "fill" ? templateSolution(raw.fill) : raw.solution);
  if (!solution) {
    errors.push(`${where}: missing solution`);
    return null;
  }
  if (kind === "fill" && parseTemplate(raw.fill).blanks.length === 0) errors.push(`${where}: fill step has no [[blanks]]`);
  if (kind === "fill" && parseTemplate(raw.fill).blanks.some((b) => !b.answer.trim())) errors.push(`${where}: a blank has an empty answer`);
  const harness = raw.harness ? ensureNl(raw.harness) : undefined;
  const hints = raw.hints ?? [];
  if (hints.length === 0) warnings.push(`${where}: no hints`);
  const require = raw.require ?? [];
  const forbid = raw.forbid ?? [];
  for (const r of [...require, ...forbid]) {
    try {
      new RegExp(r.pattern, r.flags || "");
    } catch (e) {
      errors.push(`${where}: bad rule pattern ${r.pattern}: ${e.message}`);
    }
  }
  const ruleProblems = checkRules(solution, require, forbid);
  if (ruleProblems.length) errors.push(`${where}: solution breaks its own rules: ${ruleProblems.join("; ")}`);
  const lines = solution.split("\n").length;
  const out = { kind, mode, seed, solution, hints, require, forbid };
  if (harness) out.harness = harness;
  const testsRaw = raw.tests && raw.tests.length ? raw.tests : [{ stdin: "" }];

  if (mode === "test") {
    // The learner writes the tests. The reference solution's tests must pass, and fail on every mutant.
    const res = await play(withHiddenTests(solution, harness), { tests: true });
    const diag = cleanCompilerOutput(res.stderr);
    const parsed = parseLibtest(res.stdout);
    if (!parsed.ran) {
      errors.push(`${where}: reference solution does not compile for cargo test:\n${diag}`);
      return null;
    }
    for (const w of userWarnings(diag, lines)) errors.push(`${where}: reference solution has a compiler warning:\n${w}`);
    const failed = parsed.tests.filter((t) => t.status !== "ok");
    if (failed.length) errors.push(`${where}: solution's tests fail: ${failed.map((t) => t.name + "\n" + t.output).join("\n")}`);
    const own = parsed.tests.filter((t) => !t.name.startsWith("arena_hidden_tests::")).length;
    out.minTests = raw.minTests ?? 1;
    if (own < out.minTests) errors.push(`${where}: solution has ${own} tests but minTests is ${out.minTests}`);
    out.mutants = raw.mutants ?? [];
    for (const m of out.mutants) {
      const broken = applyMutant(solution, m);
      if (broken == null) {
        errors.push(`${where}: mutant "${m.name}": text not found in the solution`);
        continue;
      }
      if (applyMutant(seed, m) == null) errors.push(`${where}: mutant "${m.name}": text not found in the seed (learners keep the seed's code)`);
      const r = await play(broken, { tests: true });
      const p = parseLibtest(r.stdout);
      if (p.ran && !p.tests.some((t) => t.status === "failed")) errors.push(`${where}: mutant "${m.name}" isn't caught by the solution's tests`);
    }
    out.tests = [{ name: "tests", stdin: "", expect: "" }];
    // The seed must not already pass: a seed with no tests fails minTests.
    return out;
  }

  if (mode === "harness") {
    const names = harnessTests(harness);
    if (!names.length) errors.push(`${where}: harness has no #[test] functions`);
    if (new Set(names).size !== names.length) errors.push(`${where}: two hidden tests have the same name`);
    const c = runCase(testsRaw[0]);
    const res = await runHarness(solution, harness, c);
    if (!res.compiled) {
      errors.push(`${where}: reference solution does not compile with its tests:\n${res.diagnostics}`);
      return null;
    }
    for (const w of userWarnings(res.diagnostics, lines)) errors.push(`${where}: reference solution has a compiler warning:\n${w}`);
    const info = harnessTestInfo(harness);
    names.forEach((n, i) => {
      const r = res.runs[i];
      if (!harnessPassed(r, info[i])) {
        const p = r ? parsePanic(r.stderr) : null;
        const why = r?.timedOut ? "timeout" : info[i]?.shouldPanic && r?.exitCode === 0 ? "it should panic but finishes normally" : p ? p.message : r?.stderr ?? "did not run";
        errors.push(`${where}: solution fails hidden test ${n}: ${why}`);
      }
    });
    out.checks = names.length;
    out.checkNames = names.map(testLabel);
    if (info.some((t) => t.shouldPanic)) out.checkPanics = info.map((t) => (t.shouldPanic ? t.expected ?? "" : null));
    out.tests = [{ name: "tests", stdin: c.stdin ?? "", expect: "" }];
    if (c.files) out.tests[0].files = c.files;
    if (c.args) out.tests[0].args = c.args;
  } else {
    if (!hasMain(solution)) errors.push(`${where}: a stdout exercise needs fn main (or give it a harness)`);
    const cases = testsRaw.map(runCase);
    const res = await runProgram(solution, cases);
    if (!res.compiled) {
      errors.push(`${where}: reference solution does not compile:\n${res.diagnostics}`);
      return null;
    }
    for (const w of userWarnings(res.diagnostics, lines)) errors.push(`${where}: reference solution has a compiler warning:\n${w}`);
    out.tests = testsRaw.map((t, i) => {
      const r = res.runs[i];
      if (!r) {
        errors.push(`${where}: test ${i + 1} did not run`);
        return { name: `Test ${i + 1}`, stdin: "", expect: "" };
      }
      if (r.timedOut) errors.push(`${where}: test ${i + 1} timed out`);
      const wantExit = t.exit ?? 0;
      if (r.exitCode !== wantExit) errors.push(`${where}: test ${i + 1} exited with ${r.exitCode ?? "signal " + r.signal}, expected ${wantExit}\n${r.stderr}`);
      const actual = normalizeOutput(r.stdout);
      if (t.expect != null && normalizeOutput(String(t.expect)) !== actual) {
        errors.push(`${where}: test ${i + 1} expected\n${t.expect}\nbut solution printed\n${actual}`);
      }
      if (!actual && !t.allowEmpty) warnings.push(`${where}: test ${i + 1} expects empty output`);
      const test = { name: t.name ?? (testsRaw.length > 1 ? `Test ${i + 1}` : "Output"), stdin: t.stdin ?? "", expect: actual, hidden: !!t.hidden };
      if (t.files) test.files = t.files;
      if (t.args) test.args = t.args.map(String);
      if (t.exit != null) test.exit = t.exit;
      return test;
    });
  }
  // A code-kind seed should not already pass (otherwise the step is free).
  if (kind === "code" && seed && !raw.seedMayPass) {
    let passes = false;
    if (mode === "harness") {
      const sr = await runHarness(seed, harness, runCase(testsRaw[0]));
      const info = harnessTestInfo(harness);
      passes = sr.compiled && out.checks > 0 && sr.runs.length === out.checks && sr.runs.every((r, i) => harnessPassed(r, info[i]));
    } else {
      const sr = await runProgram(seed, testsRaw.map(runCase));
      passes = sr.compiled && out.tests.every((t, i) => sr.runs[i] && normalizeOutput(sr.runs[i].stdout) === t.expect && sr.runs[i].exitCode === (t.exit ?? 0));
    }
    if (passes && checkRules(seed, require, forbid).length === 0) errors.push(`${where}: the starter code already passes`);
  }
  return out;
}

// ---------------------------------------------------------------- worked examples in lesson text
/**
 * Lesson text can include complete example programs:
 *   ```rust run            compiled and run; if the next block is ```output, it must print exactly that
 *                          (an ```input block right after the example is what it reads)
 *   ```rust panics         compiled and run, and must panic (exit 101); ```output shows what it printed first
 *   ```rust compile_fail   must NOT compile; if the next block is ```error, its first line must appear
 *                          in rustc's output (so the error shown is the real one)
 * Plain ```rust blocks are shown as they are (fragments, code needing other files or the network).
 * Returns the text with the info strings the app renders.
 */
async function checkExamples(where, text) {
  if (!text || !/```rust (run|panics|compile_fail)/.test(text)) return text;
  // Fenced blocks may be indented (inside a list item); the indent is removed from their contents.
  const blocks = [...text.matchAll(/^([ \t]*)```([^\n]*)\n([\s\S]*?)^\1```[ \t]*$/gm)].map((b) => [b[0], b[2], b[3].replace(new RegExp("^" + b[1], "gm"), "")]);
  for (let i = 0; i < blocks.length; i++) {
    const m = blocks[i][1].trim().match(/^rust (run|panics|compile_fail)$/);
    if (!m) continue;
    const code = blocks[i][2];
    const hasInput = blocks[i + 1]?.[1].trim() === "input";
    const stdin = hasInput ? blocks[i + 1][2] : "";
    const next = blocks[i + (hasInput ? 2 : 1)];
    if (m[1] === "compile_fail") {
      const res = await runProgram(code, [{ stdin }]);
      if (res.compiled) {
        errors.push(`${where}: example ${i + 1} is marked compile_fail but it compiles`);
        continue;
      }
      if (next && next[1].trim() === "error") {
        const want = next[2].split("\n")[0].trim();
        if (!res.diagnostics.includes(want)) errors.push(`${where}: example ${i + 1} shows the error\n${want}\nbut rustc says\n${firstError(res.diagnostics)}`);
      }
      continue;
    }
    const nondeterministic = /HashMap|HashSet|thread|spawn|Instant|SystemTime|rand::/.test(code);
    const res = await runProgram(code, nondeterministic ? [{ stdin }, { stdin }, { stdin }] : [{ stdin }]);
    if (!res.compiled) {
      errors.push(`${where}: example ${i + 1} in the text does not compile:\n${res.diagnostics}`);
      continue;
    }
    for (const w of userWarnings(res.diagnostics, code.split("\n").length)) errors.push(`${where}: example ${i + 1} has a compiler warning:\n${w}`);
    const r = res.runs[0];
    if (m[1] === "panics" ? r.exitCode !== 101 : r.exitCode !== 0 || r.timedOut) errors.push(`${where}: example ${i + 1} exits with ${r.exitCode}${m[1] === "panics" ? " (expected a panic)" : ""}\n${r.stderr}`);
    if (res.runs.some((x) => normalizeOutput(x.stdout) !== normalizeOutput(r.stdout))) errors.push(`${where}: example ${i + 1} printed different output on different runs`);
    if (next && next[1].trim() === "output" && normalizeOutput(next[2]) !== normalizeOutput(r.stdout)) {
      errors.push(`${where}: example ${i + 1} in the text says it prints\n${next[2]}\nbut it prints\n${r.stdout}`);
    }
  }
  return text
    .replace(/^([ \t]*)```rust (run|panics)[ \t]*$/gm, "$1```rust")
    .replace(/^([ \t]*)```rust compile_fail[ \t]*$/gm, "$1```rust-error")
    .replace(/^([ \t]*)```(output|input)[ \t]*$/gm, "$1```text")
    .replace(/^([ \t]*)```error[ \t]*$/gm, "$1```compiler");
}

// ---------------------------------------------------------------- lessons
async function buildLessons() {
  const files = readYamlDir("content/lessons");
  const modules = [];
  const ids = new Set();
  for (const { file, data } of files) {
    const m = data;
    if (!m.id || !m.title || !m.steps) {
      errors.push(`${file}: module needs id, title, steps`);
      continue;
    }
    if (ids.has(m.id)) errors.push(`${file}: duplicate module id ${m.id}`);
    ids.add(m.id);
    const steps = await Promise.all(
      m.steps.map(async (s, i) => {
        const where = `${file} step ${i + 1} (${s.title})`;
        // Saved progress is keyed by step id, so ids must be explicit: a position-based id would shift when a step is inserted.
        if (!s.id) errors.push(`${where}: step needs an explicit id (progress is saved by id)`);
        // YAML reads bare words like true or 42 as non-strings, which would render as an empty heading.
        if (typeof s.title !== "string" || !s.title) errors.push(`${where}: title must be a non-empty string (quote it)`);
        const ex = await buildExercise(where, s, null);
        if (!s.text) errors.push(`${where}: missing text`);
        // The "**Your turn:**" paragraph (and anything after it) is the task: the app shows it in
        // its own card above the editor, so it's obvious what the exercise asks for.
        const full = s.text ?? "";
        const at = full.indexOf("**Your turn:**");
        if (at < 0) errors.push(`${where}: the text needs a "**Your turn:**" paragraph (the task)`);
        const text = await checkExamples(where, at < 0 ? full : full.slice(0, at).trimEnd() + "\n");
        const rest = full.slice(at + "**Your turn:**".length).trim();
        const task = at < 0 ? "" : await checkExamples(where + " task", rest.charAt(0).toUpperCase() + rest.slice(1) + "\n");
        // Extra challenges practice the same idea: each is a full exercise with its own task.
        const more = [];
        for (const [k, c] of (s.more ?? []).entries()) {
          const w = `${where} challenge ${k + 2}`;
          if (!c.task) errors.push(`${w}: needs a task`);
          if (c.fill == null && c.seed == null) errors.push(`${w}: needs fill or seed (starter code)`);
          const cx = await buildExercise(w, c, null);
          if (cx) more.push({ task: await checkExamples(w, String(c.task ?? "").trim() + "\n"), ...cx });
        }
        // The challenges are checked even when the main exercise failed, so one run reports every error.
        if (!ex) return null;
        const step = { id: s.id ?? `${m.id}-${i + 1}`, title: s.title, text, task, ...ex };
        if (more.length) step.more = more;
        return step;
      }),
    );
    modules.push({ id: m.id, title: m.title, phase: m.phase ?? "", summary: m.summary ?? "", ...(m.roadmap ? { roadmap: m.roadmap } : {}), steps: steps.filter(Boolean) });
  }
  return modules;
}

// ---------------------------------------------------------------- projects
async function buildProjects() {
  const files = readYamlDir("content/projects");
  const projects = [];
  for (const { file, data: p } of files) {
    let prev = p.seed ?? "";
    const milestones = [];
    for (let i = 0; i < (p.milestones ?? []).length; i++) {
      const ms = p.milestones[i];
      const where = `${file} milestone ${i + 1} (${ms.title})`;
      const ex = await buildExercise(where, { ...ms, seed: i === 0 ? p.seed : ms.seed, seedMayPass: i > 0 || ms.seedMayPass }, prev);
      if (!ex) continue;
      milestones.push({ title: ms.title, text: await checkExamples(where, ms.text ?? ""), ...ex });
      prev = ex.solution;
    }
    projects.push({ id: p.id, title: p.title, level: p.level, after: p.after ?? null, summary: p.summary ?? "", milestones });
  }
  return projects;
}

// ---------------------------------------------------------------- drills
async function buildDrills(moduleIds) {
  const files = readYamlDir("content/drills");
  const all = [];
  const seen = new Set();
  const jobs = [];
  for (const { file, data } of files) {
    const topic = data.topic;
    // "interview" is the interview prep set, not tied to a lesson module.
    if (moduleIds && !moduleIds.has(topic) && topic !== "interview") errors.push(`${file}: unknown topic ${topic}`);
    (data.drills ?? []).forEach((d, i) => {
      // Interview drills come from several files, so their ids include the file name.
      const id = d.id ?? (topic === "interview" ? `${path.basename(file, ".yaml")}-${i + 1}` : `${topic}-${i + 1}`);
      if (seen.has(id)) errors.push(`${file}: duplicate drill id ${id}`);
      seen.add(id);
      const where = `${file} drill ${i + 1} (${d.type})`;
      jobs.push(buildDrill(where, id, topic, d).then((x) => x && all.push(x)));
    });
  }
  await Promise.all(jobs);
  const order = [...seen];
  all.sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
  return all;
}

const srcOf = (d, pre, body) => ({ pre, body, stdin: d.stdin ?? "", ...(d.files ? { files: d.files } : {}), ...(d.args ? { args: d.args.map(String) } : {}) });

async function buildDrill(where, id, topic, d) {
  const base = { id, topic, type: d.type, why: d.why ?? "", ...(d.after ? { after: d.after } : {}) };
  if (!d.why && d.type !== "compiles") warnings.push(`${where}: no explanation (why)`);
  const pre = d.pre ? d.pre.replace(/\s*$/, "") : "";
  const body = d.body ? d.body.replace(/\s*$/, "") : "";
  const one = [runCase(d)];
  switch (d.type) {
    case "predict": {
      // Three runs: output that changes between runs (HashMap order, threads) makes a bad question.
      const res = await runProgram(drillProgram(pre, body), [one[0], one[0], one[0]]);
      if (!res.compiled) return void errors.push(`${where}: does not compile\n${res.diagnostics}`);
      const r = res.runs[0];
      if (r.exitCode !== 0 || r.timedOut) return void errors.push(`${where}: program failed (exit ${r.exitCode})\n${r.stderr}`);
      const answer = normalizeOutput(r.stdout);
      if (res.runs.some((x) => normalizeOutput(x.stdout) !== answer)) errors.push(`${where}: prints different output on different runs`);
      if (!answer) errors.push(`${where}: prints nothing`);
      if (answer.split("\n").length > 4) warnings.push(`${where}: long output (${answer.split("\n").length} lines)`);
      if (d.answer != null && looseOutput(String(d.answer)) !== looseOutput(answer)) errors.push(`${where}: author answer "${d.answer}" but program prints "${answer}"`);
      return { ...base, prompt: d.prompt ?? "What does this print?", display: drillDisplay(pre, body), answer, src: srcOf(d, pre, body) };
    }
    case "fill": {
      const tmpl = drillDisplay(pre, body);
      const { blanks } = parseTemplate(tmpl);
      if (blanks.length !== 1) return void errors.push(`${where}: fill drills need exactly one [[blank]]`);
      if (!blanks[0].answer.trim()) return void errors.push(`${where}: empty blank answer`);
      const res = await runProgram(drillProgram(templateSolution(pre), templateSolution(body)), one);
      if (!res.compiled) return void errors.push(`${where}: solution does not compile\n${res.diagnostics}`);
      const r = res.runs[0];
      if (r.exitCode !== 0 || r.timedOut) return void errors.push(`${where}: solution run failed (exit ${r.exitCode})`);
      const out = normalizeOutput(r.stdout);
      if (d.expect != null && normalizeOutput(String(d.expect)) !== out) errors.push(`${where}: expected "${d.expect}" but got "${out}"`);
      for (const alt of blanks[0].accept.slice(1)) {
        const fill = (s) => fillFirstBlank(s, alt);
        const ar = await runProgram(drillProgram(fill(pre), fill(body)), one);
        if (!ar.compiled || normalizeOutput(ar.runs[0]?.stdout ?? "") !== out) errors.push(`${where}: accepted alternative "${alt}" doesn't compile or prints something else`);
      }
      return { ...base, prompt: d.prompt ?? "Fill the blank so the program prints the output shown.", display: tmpl, answer: blanks[0].answer, accept: blanks[0].accept, output: out, src: srcOf(d, templateSolution(pre), templateSolution(body)) };
    }
    case "bug": {
      const fullDisplay = drillDisplay(pre, body);
      const lines = fullDisplay.split("\n");
      const bugIdx = lines.findIndex((l) => /\/\/\s*BUG\s*$/.test(l));
      if (bugIdx < 0 || !d.fix) return void errors.push(`${where}: mark the buggy line with // BUG and give fix`);
      if (/^\s*[{}]\s*;?\s*\/\/\s*BUG\s*$/.test(lines[bugIdx])) return void errors.push(`${where}: the bug line can't be a lone brace (not clickable)`);
      const clean = (s) => s.split("\n").map((l) => l.replace(/\s*\/\/\s*BUG\s*$/, "")).join("\n");
      const fixLine = (s) =>
        s
          .split("\n")
          .map((l) => (/\/\/\s*BUG\s*$/.test(l) ? l.match(/^\s*/)[0] + d.fix.trim() : l))
          .join("\n");
      const buggy = await runProgram(drillProgram(clean(pre), clean(body)), one);
      const fixed = await runProgram(drillProgram(fixLine(pre), fixLine(body)), one);
      if (!fixed.compiled) return void errors.push(`${where}: fixed version does not compile\n${fixed.diagnostics}`);
      const fr = fixed.runs[0];
      if (fr.exitCode !== 0 || fr.timedOut) return void errors.push(`${where}: fixed version fails at runtime\n${fr.stderr}`);
      const fixedOut = normalizeOutput(fr.stdout);
      if (buggy.compiled) {
        const br = buggy.runs[0];
        const same = !br.timedOut && br.exitCode === 0 && normalizeOutput(br.stdout) === fixedOut;
        if (same) errors.push(`${where}: buggy and fixed versions behave the same`);
      }
      return {
        ...base,
        prompt: d.prompt ?? `This should print: ${fixedOut.replace(/\n/g, " / ")}. Click the line with the bug.`,
        display: clean(fullDisplay),
        answer: String(bugIdx + 1),
        fix: d.fix.trim(),
        output: fixedOut,
        src: srcOf(d, fixLine(pre), fixLine(body)),
      };
    }
    case "compiles": {
      const res = await runProgram(drillProgram(pre, body), one);
      const answer = res.compiled ? "yes" : "no";
      if (d.answer != null && String(d.answer) !== answer) errors.push(`${where}: author says ${d.answer} but rustc says ${answer}\n${res.diagnostics}`);
      let why = d.why ?? "";
      let error;
      if (!res.compiled) {
        error = firstError(res.diagnostics);
        if (!why) why = "Compiler: " + error.replace(/^error(\[E\d+\])?:\s*/, "");
      }
      if (!why) warnings.push(`${where}: no explanation`);
      return { ...base, why, ...(error ? { error } : {}), prompt: d.prompt ?? "Does this compile?", display: drillDisplay(pre, body), answer, src: srcOf(d, pre, body) };
    }
    case "choice": {
      const choices = (d.choices ?? []).map(String);
      const answer = Number(d.answer);
      if (choices.length < 2 || choices.length > 4) return void errors.push(`${where}: choice drills need 2 to 4 choices`);
      if (new Set(choices).size !== choices.length) errors.push(`${where}: duplicate choices`);
      if (!(answer >= 1 && answer <= choices.length)) return void errors.push(`${where}: answer must be the number of the right choice (1 to ${choices.length})`);
      if (!d.prompt) errors.push(`${where}: choice drills need a prompt`);
      if (!d.why) errors.push(`${where}: choice drills need an explanation (why)`);
      const display = drillDisplay(pre, body);
      if (display) {
        // Code shown with a question must compile (unless the question is about a compile error),
        // and "verify: output" checks that the right choice really is what it prints.
        const res = await runProgram(drillProgram(pre, body || "// nothing"), one);
        if (d.compiles !== false && !res.compiled) return void errors.push(`${where}: code does not compile\n${res.diagnostics}`);
        if (d.compiles === false && res.compiled) errors.push(`${where}: marked compiles: false but it compiles`);
        if (d.verify === "output") {
          const r = res.runs[0];
          if (!r || r.exitCode !== 0) return void errors.push(`${where}: program failed`);
          const out = looseOutput(normalizeOutput(r.stdout));
          if (looseOutput(choices[answer - 1]) !== out) errors.push(`${where}: right choice "${choices[answer - 1]}" but the program prints "${normalizeOutput(r.stdout)}"`);
          choices.forEach((c, i) => i !== answer - 1 && looseOutput(c) === out && errors.push(`${where}: choice ${i + 1} is also correct`));
        }
      }
      return { ...base, prompt: d.prompt ?? "", display, answer: String(answer), choices };
    }
    case "boss": {
      const ex = await buildExercise(where, d, null);
      if (!ex) return null;
      return { ...base, prompt: d.prompt ?? "", display: "", answer: "", exercise: ex };
    }
    default:
      errors.push(`${where}: unknown drill type ${d.type}`);
      return null;
  }
}

// ---------------------------------------------------------------- placement quiz
async function buildPlacement(modules) {
  const file = path.join(ROOT, "content", "placement.yaml");
  if (!fs.existsSync(file)) return [];
  const data = YAML.parse(fs.readFileSync(file, "utf8"));
  const order = modules.map((m) => m.id);
  const out = [];
  let last = -1;
  for (const [i, q] of (data.questions ?? []).entries()) {
    const where = `content/placement.yaml question ${i + 1}`;
    const at = order.indexOf(q.module);
    if (at < 0) {
      errors.push(`${where}: unknown module ${q.module}`);
      continue;
    }
    if (at < last) errors.push(`${where}: questions must follow the curriculum order`);
    last = at;
    const d = await buildDrill(where, `placement-${i + 1}`, q.module, q);
    if (d) out.push({ ...d, module: q.module });
  }
  return out;
}

// ---------------------------------------------------------------- topic reference pages
async function buildTopics(modules) {
  const moduleIds = modules ? new Set(modules.map((m) => m.id)) : null;
  const out = [];
  const slugs = new Set();
  for (const { file, data } of readYamlDir("content/topics")) {
    for (const [i, t] of (data.topics ?? []).entries()) {
      const where = `${file} topic ${i + 1} (${t.slug})`;
      for (const k of ["slug", "title", "description", "body", "example"]) if (!t[k]) errors.push(`${where}: missing ${k}`);
      if (!/^[a-z0-9-]+$/.test(t.slug ?? "")) errors.push(`${where}: slug must be lowercase letters, digits and dashes`);
      if (slugs.has(t.slug)) errors.push(`${where}: duplicate slug`);
      slugs.add(t.slug);
      if ((t.description ?? "").length > 160) warnings.push(`${where}: description is over 160 characters`);
      if (moduleIds) for (const m of t.modules ?? []) if (!moduleIds.has(m)) errors.push(`${where}: unknown module ${m}`);
      const example = ensureNl(t.example);
      const nondeterministic = /HashMap|HashSet|thread|spawn/.test(example);
      const c = { stdin: t.stdin ?? "" };
      const res = await runProgram(example, nondeterministic ? [c, c, c] : [c]);
      if (!res.compiled) {
        errors.push(`${where}: example does not compile:\n${res.diagnostics}`);
        continue;
      }
      for (const w of userWarnings(res.diagnostics, example.split("\n").length)) errors.push(`${where}: example has a compiler warning:\n${w}`);
      const r = res.runs[0];
      if (r.exitCode !== 0 || r.timedOut) errors.push(`${where}: example exits with ${r.exitCode}\n${r.stderr}`);
      if (res.runs.some((x) => x.stdout !== r.stdout)) errors.push(`${where}: example printed different output on different runs`);
      out.push({ slug: t.slug, title: t.title, description: t.description, modules: t.modules ?? [], body: await checkExamples(where, t.body), example, stdin: t.stdin ?? "", output: r.stdout });
    }
  }
  return out;
}

// ---------------------------------------------------------------- pro track
function buildPro() {
  const file = path.join(ROOT, "content", "pro.yaml");
  if (!fs.existsSync(file)) return [];
  const data = YAML.parse(fs.readFileSync(file, "utf8"));
  const projDir = path.join(ROOT, "pro-track", "starter", "projects");
  const dirs = fs.existsSync(projDir) ? fs.readdirSync(projDir).sort() : [];
  const listed = data.projects.map((p) => p.dir);
  for (const d of dirs) if (!listed.includes(d)) errors.push(`content/pro.yaml: project folder ${d} is not listed`);
  return data.projects
    .map((p) => {
      const readme = path.join(projDir, p.dir, "README.md");
      if (!fs.existsSync(readme)) {
        errors.push(`content/pro.yaml: ${p.dir} has no README.md`);
        return null;
      }
      return {
        id: p.dir.replace(/^\d+-/, ""),
        dir: p.dir,
        number: parseInt(p.dir, 10),
        title: p.title,
        summary: p.summary,
        hours: p.hours,
        skills: p.skills ?? [],
        readme: fs.readFileSync(readme, "utf8"),
      };
    })
    .filter(Boolean);
}

// ---------------------------------------------------------------- roadmap
function buildRoadmap(modules, pro) {
  const file = path.join(ROOT, "content", "roadmap.yaml");
  if (!fs.existsSync(file)) return [];
  const data = YAML.parse(fs.readFileSync(file, "utf8"));
  const ids = new Set(modules.map((m) => m.id));
  const used = new Set();
  for (const s of data.sections ?? []) {
    if (!s.title || !s.items?.length) errors.push(`content/roadmap.yaml: section needs title and items`);
    for (const it of s.items ?? []) {
      for (const p of it.pro ?? []) if (!pro.some((x) => x.id === p)) errors.push(`content/roadmap.yaml: ${it.title}: unknown Pro Track project ${p}`);
      if (!it.modules?.length && !it.pro?.length && !it.links?.length) errors.push(`content/roadmap.yaml: ${it.title}: needs modules, pro or links`);
      for (const m of it.modules ?? []) {
        if (!ids.has(m)) errors.push(`content/roadmap.yaml: ${it.title}: unknown module ${m}`);
        used.add(m);
      }
    }
  }
  for (const m of modules) if (!used.has(m.id)) warnings.push(`content/roadmap.yaml: module ${m.id} isn't on the roadmap`);
  return data.sections ?? [];
}

// ---------------------------------------------------------------- main
const t0 = Date.now();
const modules = await buildLessons();
{
  // Stable step addresses (see scripts/step-urls.mjs). New steps get their address pinned here.
  const urls = readStepUrls(ROOT);
  if (applyStepUrls(modules, urls, errors, warnings, { partial: !!ONLY }) && !errors.length && !ONLY) {
    writeStepUrls(ROOT, urls);
    console.log(`${STEP_URLS_FILE}: pinned addresses for new steps (commit this file)`);
  }
}
const projects = await buildProjects();
// A partial check (--only) can't know every module id, so it skips the cross-file checks.
const allModuleIds = ONLY ? null : new Set(modules.map((m) => m.id));
const drills = await buildDrills(allModuleIds);
if (!ONLY) assignDrillSteps(modules, drills, errors);
const pro = ONLY ? [] : buildPro();
const placement = ONLY ? [] : await buildPlacement(modules);
const topics = await buildTopics(ONLY ? null : modules);
const roadmap = ONLY ? [] : buildRoadmap(modules, pro);
let next = null;
if (!ONLY) {
  // The "Where to go next" page (/next).
  next = YAML.parse(fs.readFileSync(path.join(ROOT, "content/next.yaml"), "utf8"));
  for (const k of ["title", "description", "body"]) if (!next?.[k]) errors.push(`content/next.yaml: missing ${k}`);
}
{
  const seenSteps = new Set();
  for (const m of modules)
    for (const st of m.steps) {
      if (seenSteps.has(st.id)) errors.push(`duplicate step id ${st.id}`);
      seenSteps.add(st.id);
    }
}
if (!ONLY) for (const p of projects) if (p.after && !modules.some((m) => m.id === p.after)) warnings.push(`project ${p.id}: unknown 'after' module ${p.after}`);

for (const w of warnings) console.warn("warn:", w);
if (errors.length) {
  for (const e of errors) console.error("\nERROR:", e);
  console.error(`\n${errors.length} content error(s).`);
  process.exit(1);
}
const steps = modules.reduce((a, m) => a + m.steps.length, 0);
const challengeCount = modules.reduce((a, m) => a + m.steps.reduce((b, s) => b + 1 + (s.more?.length ?? 0), 0), 0);
const ms = projects.reduce((a, p) => a + p.milestones.length, 0);
const summary = `${modules.length} modules, ${steps} steps (${challengeCount} exercises), ${drills.length} drills, ${projects.length} projects (${ms} milestones), ${pro.length} pro projects, ${placement.length} placement questions, ${topics.length} topic pages`;
if (ONLY) {
  console.log(`check ok (--only ${ONLY}, nothing written): ${summary} in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
  process.exit(0);
}
const content = { generatedAt: new Date().toISOString(), modules, projects, drills, pro, placement, topics, next, roadmap };
fs.mkdirSync(path.join(ROOT, "src/generated"), { recursive: true });
// Extra challenges are only needed on step pages, so they live in their own file that the app
// loads on demand. The main file keeps a count so progress works before they arrive.
const challenges = {};
for (const m of modules)
  for (const s of m.steps)
    if (s.more) {
      challenges[s.id] = s.more;
      s.moreCount = s.more.length;
      delete s.more;
    }
fs.writeFileSync(path.join(ROOT, "src/generated/content.json"), JSON.stringify(content));
fs.writeFileSync(path.join(ROOT, "src/generated/challenges.json"), JSON.stringify(challenges));
console.log(`content ok: ${summary} in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
