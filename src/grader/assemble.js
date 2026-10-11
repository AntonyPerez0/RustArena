// Pure helpers shared by the browser grader and the Node content scripts.
//
// Every program the site runs goes through `wrapProgram`: the learner's code, with its
// `fn main` renamed, plus a small hidden runner. The runner starts the program once per
// test case as a child process (so each test gets its own stdin, arguments, files and
// time limit) and prints one `@@ARENA` record per run. That way one request to the Rust
// Playground grades every test, and the Playground's lack of stdin doesn't matter.

/**
 * Blank markers in fill-in templates look like [[answer]] or [[answer‖alt1‖alt2]] (U+2016 separates
 * accepted alternatives). Brackets inside an answer are balanced, so `[[v[0]]]` is a blank holding
 * `v[0]`, and in `arr[[[2]]]` the blank holds `2` between the array's own brackets.
 * Returns each blank's position and inner text.
 */
export function findBlanks(template) {
  const out = [];
  const t = String(template);
  let i = 0;
  while (i < t.length) {
    // A blank opens at "[[" that isn't followed by another "[" (in "[[[", it's the last two).
    if (t[i] === "[" && t[i + 1] === "[" && t[i + 2] !== "[") {
      let j = i + 2;
      let depth = 0;
      let end = -1;
      while (j < t.length && t[j] !== "\n") {
        if (t[j] === "[") depth++;
        else if (t[j] === "]") {
          if (depth === 0 && t[j + 1] === "]") {
            end = j;
            break;
          }
          depth = Math.max(0, depth - 1);
        }
        j++;
      }
      if (end >= 0) {
        out.push({ start: i, end: end + 2, inner: t.slice(i + 2, end) });
        i = end + 2;
        continue;
      }
    }
    i++;
  }
  return out;
}

/** Split a fill template into text parts and blank answers. */
export function parseTemplate(template) {
  const parts = [];
  const blanks = [];
  let last = 0;
  for (const b of findBlanks(template)) {
    parts.push(template.slice(last, b.start));
    const alts = b.inner.split("‖");
    blanks.push({ answer: alts[0], accept: alts });
    last = b.end;
  }
  parts.push(template.slice(last));
  return { parts, blanks };
}

/** The template with its first blank filled with `value` (other blanks untouched). */
export function fillFirstBlank(template, value) {
  const b = findBlanks(template)[0];
  return b ? template.slice(0, b.start) + value + template.slice(b.end) : template;
}

/** Rebuild source from a template and the learner's blank values. */
export function fillTemplate(template, values) {
  const { parts } = parseTemplate(template);
  let out = parts[0];
  for (let i = 1; i < parts.length; i++) out += (values[i - 1] ?? "") + parts[i];
  return out;
}

/** The reference solution of a fill template (first alternative of each blank). */
export function templateSolution(template) {
  const { blanks } = parseTemplate(template);
  return fillTemplate(template, blanks.map((b) => b.answer));
}

/** The name the learner's `fn main` gets inside the wrapped program. */
export const USER_MAIN = "__arena_user_main";

/** A top-level `fn main(` (optionally pub and/or async), at the start of a line. */
const MAIN_RE = /^((?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?)fn\s+main\s*(?=\()/m;

/** An `async fn main` without an attribute (such as #[tokio::main]) on the line before it. */
function bareAsyncMain(code) {
  const m = MAIN_RE.exec(code);
  if (!m || !/async/.test(m[1])) return false;
  const before = code.slice(0, m.index).replace(/\s+$/, "");
  return !/#\[[^\]]*\]$/.test(before);
}

export function hasMain(code) {
  return MAIN_RE.test(stripForRules(code));
}

/** A Rust string literal for any text. */
export function rustStr(s) {
  return '"' + String(s).replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n").replace(/\r/g, "\\r").replace(/\t/g, "\\t") + '"';
}

const RUNNER = String.raw`
// ---- RustArena test runner (added by the site; not part of your code) ----
#[allow(dead_code, unused_imports, unused_mut, unused_variables, clippy::all)]
mod __arena {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    pub struct Case {
        pub stdin: &'static str,
        pub args: &'static [&'static str],
        pub files: &'static [(&'static str, &'static str)],
        pub test: &'static str,
    }

    const LIMIT: Duration = Duration::from_millis(__LIMIT__);
    const MAX_OUT: usize = 64 * 1024;

    fn b64(data: &[u8]) -> String {
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut s = String::with_capacity(data.len() * 4 / 3 + 4);
        for c in data.chunks(3) {
            let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
            s.push(T[(n >> 18) as usize & 63] as char);
            s.push(T[(n >> 12) as usize & 63] as char);
            s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
            s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
        }
        if s.is_empty() { s.push('-'); }
        s
    }

    fn drain<R: Read + Send + 'static>(mut r: R) -> std::thread::JoinHandle<(Vec<u8>, bool)> {
        std::thread::spawn(move || {
            let mut kept = Vec::new();
            let mut buf = [0u8; 8192];
            let mut cut = false;
            loop {
                match r.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let room = MAX_OUT.saturating_sub(kept.len());
                        if n > room { cut = true; }
                        kept.extend_from_slice(&buf[..n.min(room)]);
                    }
                }
            }
            (kept, cut)
        })
    }

    pub fn run(cases: &[Case]) {
        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => { println!("\n@@ARENA-ERROR cannot find the program: {}", e); return; }
        };
        let base = std::env::temp_dir().join(format!("rustarena-{}", std::process::id()));
        for (i, case) in cases.iter().enumerate() {
            let dir = base.join(i.to_string());
            let _ = std::fs::create_dir_all(&dir);
            for (name, text) in case.files { let _ = std::fs::write(dir.join(name), text); }
            let started = Instant::now();
            let spawned = Command::new(&exe)
                .args(case.args)
                .current_dir(&dir)
                .env("RUSTARENA_CHILD", if case.test.is_empty() { "main" } else { case.test })
                .env("RUST_BACKTRACE", "0")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            let mut child = match spawned {
                Ok(c) => c,
                Err(e) => { println!("\n@@ARENA-ERROR cannot start the program: {}", e); return; }
            };
            let mut sin = child.stdin.take().unwrap();
            let input = case.stdin.as_bytes().to_vec();
            let writer = std::thread::spawn(move || { let _ = sin.write_all(&input); });
            let out = drain(child.stdout.take().unwrap());
            let err = drain(child.stderr.take().unwrap());
            let mut timed_out = false;
            let status = loop {
                match child.try_wait() {
                    Ok(Some(s)) => break Some(s),
                    Ok(None) if started.elapsed() > LIMIT => { timed_out = true; let _ = child.kill(); break child.wait().ok(); }
                    Ok(None) => std::thread::sleep(Duration::from_millis(2)),
                    Err(_) => break None,
                }
            };
            let _ = writer.join();
            let (stdout, cut1) = out.join().unwrap_or_default();
            let (stderr, cut2) = err.join().unwrap_or_default();
            let code = status.and_then(|s| s.code()).map(|c| c.to_string()).unwrap_or_else(|| "x".into());
            #[cfg(unix)]
            let signal = { use std::os::unix::process::ExitStatusExt; status.and_then(|s| s.signal()).unwrap_or(0) };
            #[cfg(not(unix))]
            let signal = 0;
            println!("\n@@ARENA {} {} {} {} {} {} {} {}", i, code, signal, timed_out as u8, (cut1 || cut2) as u8, started.elapsed().as_millis(), b64(&stdout), b64(&stderr));
            // An endless loop would only time out again: stop here.
            if timed_out { break; }
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
`;

function caseLiteral(c) {
  const args = (c.args ?? []).map(rustStr).join(", ");
  const files = Object.entries(c.files ?? {})
    .map(([n, t]) => `(${rustStr(n)}, ${rustStr(t)})`)
    .join(", ");
  return `__arena::Case { stdin: ${rustStr(c.stdin ?? "")}, args: &[${args}], files: &[${files}], test: ${rustStr(c.test ?? "")} }`;
}

/** The hidden tests of a harness exercise: their function names, in order. */
export function harnessTests(harness) {
  return harnessTestInfo(harness).map((t) => t.name);
}

/**
 * Each hidden test with whether it must panic: `#[should_panic]` (optionally with
 * `expected = "text"`) between `#[test]` and `fn` means the test passes only if it panics,
 * with a message containing the expected text.
 */
export function harnessTestInfo(harness) {
  return [...String(harness).matchAll(/#\[test\]((?:\s*#\[[^\]]*\]|\s*\/\/[^\n]*)*)\s*fn\s+([A-Za-z_]\w*)\s*\(/g)].map((m) => {
    const sp = m[1].match(/#\[should_panic(?:\s*\(\s*expected\s*=\s*"((?:[^"\\]|\\.)*)"\s*\))?\s*\]/);
    return { name: m[2], shouldPanic: !!sp, expected: sp?.[1] ? sp[1].replace(/\\(.)/g, "$1") : undefined };
  });
}

/** "adds_two_numbers" -> "adds two numbers": how a hidden test is named on screen. */
export function testLabel(fn) {
  return fn.replace(/_+/g, " ").trim();
}

/**
 * The complete program sent to the compiler.
 *
 * stdout mode: `cases` are the runs ({ stdin, args, files }); the learner's main runs once for each.
 * harness mode: `harness` is Rust code with `#[test] fn name() { ... }` functions that use the
 * learner's items. Each test function runs as its own child process; a panic (a failed assert)
 * fails that test only.
 *
 * The learner's code comes first and is unchanged apart from main's name, so compiler line
 * numbers match the editor.
 */
/**
 * @param {string} code
 * @param {{ cases?: { stdin?: string, args?: string[], files?: Record<string, string> }[], harness?: string, limitMs?: number }} [opts]
 * @returns {string}
 */
export function wrapProgram(code, { cases = [{}], harness = undefined, limitMs = 2000 } = {}) {
  const user = String(code).replace(/\s*$/, "\n");
  const runner = RUNNER.replace("__LIMIT__", String(limitMs));
  if (harness != null) {
    const tests = harnessTests(harness);
    const body = String(harness)
      .replace(/#\[should_panic[^\]]*\]/g, "")
      .replace(/#\[test\]\s*\n?(\s*)fn\s+/g, "$1pub fn ")
      .replace(/\s*$/, "")
      .split("\n")
      .map((l) => (l ? "    " + l : l))
      .join("\n");
    const dispatch = tests.map((t) => `        Ok(${rustStr(t)}) => { arena_tests::${t}(); std::process::ExitCode::SUCCESS }`).join("\n");
    const runCases = tests.map((t) => "            " + caseLiteral({ ...(cases[0] ?? {}), test: t }) + ",").join("\n");
    return (
      user.replace(MAIN_RE, `$1fn ${USER_MAIN}`) +
      `\n// ---- hidden tests ----\n#[allow(unused_imports, dead_code, clippy::all)]\nmod arena_tests {\n    use super::*;\n${body}\n}\n` +
      runner +
      `\nfn main() -> std::process::ExitCode {\n    match std::env::var("RUSTARENA_CHILD").as_deref() {\n${dispatch}\n        Ok(_) => std::process::ExitCode::FAILURE,\n        Err(_) => {\n            __arena::run(&[\n${runCases}\n            ]);\n            std::process::ExitCode::SUCCESS\n        }\n    }\n}\n`
    );
  }
  // `async fn main` with no runtime attribute (like #[tokio::main]) can't work: send the code as it is,
  // so rustc reports its own error (E0752) on the learner's line instead of in the hidden runner.
  if (bareAsyncMain(user)) return user;
  const runCases = (cases.length ? cases : [{}]).map((c) => "            " + caseLiteral({ ...c, test: "" }) + ",").join("\n");
  return (
    user.replace(MAIN_RE, `$1fn ${USER_MAIN}`) +
    runner +
    `\nfn main() -> std::process::ExitCode {\n    match std::env::var("RUSTARENA_CHILD").as_deref() {\n        Ok("main") => std::process::Termination::report(${USER_MAIN}()),\n        Ok(_) => std::process::ExitCode::FAILURE,\n        Err(_) => {\n            __arena::run(&[\n${runCases}\n            ]);\n            std::process::ExitCode::SUCCESS\n        }\n    }\n}\n`
  );
}

function fromB64(s) {
  if (s === "-" || !s) return "";
  if (typeof Buffer !== "undefined") return Buffer.from(s, "base64").toString("utf8");
  const bin = atob(s);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}

/**
 * Read the runner's records out of the wrapped program's stdout. Each run:
 * { stdout, stderr, exitCode (null if killed), signal, timedOut, truncated, ms }.
 */
export function parseRuns(stdout) {
  const runs = [];
  let error = null;
  for (const line of String(stdout).split("\n")) {
    const m = line.match(/^@@ARENA (\d+) (-?\d+|x) (\d+) ([01]) ([01]) (\d+) (\S+) (\S+)\s*$/);
    if (m) {
      runs[+m[1]] = {
        stdout: fromB64(m[7]),
        stderr: fromB64(m[8]),
        exitCode: m[2] === "x" ? null : +m[2],
        signal: +m[3],
        timedOut: m[4] === "1",
        truncated: m[5] === "1",
        ms: +m[6],
      };
      continue;
    }
    const e = line.match(/^@@ARENA-ERROR (.*)$/);
    if (e) error = e[1];
  }
  return { runs, error };
}

/**
 * What went wrong in a run, from its stderr: the panic message and where it happened.
 * Rust prints `thread 'main' (123) panicked at src/main.rs:4:5:\nmessage`.
 */
export function parsePanic(stderr) {
  const m = String(stderr).match(/thread '([^']*)'(?: \(\d+\))? panicked at ([^\n]*?):(\d+):(\d+):\n([\s\S]*?)(?:\nnote: run with `RUST_BACKTRACE|\nstack backtrace:|$)/);
  if (!m) return null;
  const message = m[5].replace(/\s+$/, "");
  const assert = message.match(/^assertion `left == right` failed(?::\s*([\s\S]*?))?\n\s*left: ([\s\S]*?)\n\s*right: ([\s\S]*)$/);
  const assertNe = message.match(/^assertion `left != right` failed(?::\s*([\s\S]*?))?\n\s*left: ([\s\S]*?)\n\s*right: ([\s\S]*)$/);
  return {
    file: m[2],
    line: +m[3],
    col: +m[4],
    message,
    ...(assert ? { label: assert[1]?.trim() || undefined, got: assert[2].trim(), expected: assert[3].trim() } : {}),
    ...(assertNe ? { label: assertNe[1]?.trim() || undefined, ne: true, got: assertNe[2].trim(), expected: assertNe[3].trim() } : {}),
  };
}

/** Normalise program output for comparison: unify newlines, drop trailing spaces and blank tail. */
export function normalizeOutput(s) {
  return String(s)
    .replace(/\r\n?/g, "\n")
    .split("\n")
    .map((l) => l.replace(/[ \t]+$/, ""))
    .join("\n")
    .replace(/\n+$/, "");
}

/** Looser comparison used by predict-the-output drills: all whitespace runs collapse. */
export function looseOutput(s) {
  return String(s).replace(/\s+/g, " ").trim();
}

/**
 * Strip comments and string/char literals so require/forbid rules only see code.
 * Handles raw strings (r"..", r#".."#), byte strings, and tells chars ('a') from lifetimes ('a).
 */
export function stripForRules(src, { keepStrings = false } = {}) {
  let out = "";
  const s = String(src);
  let i = 0;
  while (i < s.length) {
    const c = s[i];
    const n = s[i + 1];
    if (c === "/" && n === "/") {
      while (i < s.length && s[i] !== "\n") i++;
      out += " ";
      continue;
    }
    if (c === "/" && n === "*") {
      let depth = 1;
      i += 2;
      while (i < s.length && depth > 0) {
        if (s[i] === "/" && s[i + 1] === "*") (depth++, (i += 2));
        else if (s[i] === "*" && s[i + 1] === "/") (depth--, (i += 2));
        else i++;
      }
      out += " ";
      continue;
    }
    const raw = s.slice(i).match(/^b?r(#*)"/);
    if (raw && !/[A-Za-z0-9_]/.test(s[i - 1] ?? "")) {
      const close = '"' + raw[1];
      const end = s.indexOf(close, i + raw[0].length);
      const j = end < 0 ? s.length : end + close.length;
      out += keepStrings ? s.slice(i, j) : '""';
      i = j;
      continue;
    }
    if (c === '"' || (c === "b" && n === '"' && !/[A-Za-z0-9_]/.test(s[i - 1] ?? ""))) {
      const from = i;
      i += c === "b" ? 2 : 1;
      while (i < s.length && s[i] !== '"') i += s[i] === "\\" ? 2 : 1;
      i++;
      out += keepStrings ? s.slice(from, i) : '""';
      continue;
    }
    if (c === "'") {
      const ch = s.slice(i).match(/^'(?:\\(?:u\{[0-9a-fA-F]{1,6}\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'/u);
      if (ch) {
        i += ch[0].length;
        out += keepStrings ? ch[0] : "''";
        continue;
      }
    }
    out += c;
    i++;
  }
  return out;
}

/**
 * require/forbid rules are regexes tested against the code with comments removed. String and
 * char literals are blanked out too (so "for" in a message doesn't count as a loop), unless the
 * rule says `strings: true` (for rules about format strings, like "use {:?}"). `raw: true` tests the
 * code exactly as written, comments included (for rules about doc comments).
 */
export function checkRules(src, require = [], forbid = []) {
  const code = stripForRules(src);
  const withStrings = stripForRules(src, { keepStrings: true });
  const pick = (r) => (r.raw ? String(src) : r.strings ? withStrings : code);
  const problems = [];
  for (const r of require) if (!new RegExp(r.pattern, r.flags || "").test(pick(r))) problems.push(r.message);
  for (const r of forbid) if (new RegExp(r.pattern, r.flags || "").test(pick(r))) problems.push(r.message);
  return problems;
}

/**
 * Drills show only `display` (pre + body). The full program adds `fn main` around the body.
 * Lints that would only be noise in a three-line snippet are switched off.
 */
export const DRILL_HEAD = "#![allow(unused, clippy::all)]\n";

export function drillProgram(pre, body) {
  const p = pre ? pre.replace(/\s*$/, "\n") : "";
  if (body == null || body === "") return DRILL_HEAD + p;
  const indented = body
    .replace(/\s*$/, "")
    .split("\n")
    .map((l) => (l ? "    " + l : l))
    .join("\n");
  return DRILL_HEAD + p + (p ? "\n" : "") + "fn main() {\n" + indented + "\n}\n";
}

export function drillDisplay(pre, body) {
  const p = pre ? pre.replace(/\s*$/, "") : "";
  const b = body ? body.replace(/\s*$/, "") : "";
  if (p && b) return p + "\n\n// inside main:\n" + b;
  return p || b;
}

/** What one test run gets: { stdin, files, args }. */
/**
 * @param {{ stdin?: string, files?: Record<string, string>, args?: (string|number)[] } | undefined} t
 * @returns {{ stdin?: string, files?: Record<string, string>, args?: string[] }}
 */
export function runCase(t) {
  if (!t) return {};
  const c = { stdin: t.stdin ?? "" };
  if (t.files && Object.keys(t.files).length) c.files = t.files;
  if (t.args && t.args.length) c.args = t.args.map(String);
  return c;
}

/** Remove Cargo's progress lines from the Playground's stderr, leaving rustc's diagnostics. */
export function cleanCompilerOutput(stderr) {
  return String(stderr)
    .split("\n")
    .filter((l) => !/^\s*(Compiling|Finished|Running|Blocking|Updating|Downloaded|Downloading|Locking|Adding|Checking|Executable|Doc-tests)\b/.test(l))
    .join("\n")
    .replace(new RegExp(USER_MAIN, "g"), "main")
    .replace(/^\n+|\n+$/g, "");
}

/**
 * Parse `cargo test` (libtest) output: one entry per test with its status and, for failures,
 * what it printed (the panic message). Tests may finish in any order.
 */
export function parseLibtest(stdout) {
  const text = String(stdout);
  const tests = [];
  // `test name ... ok`, `test name - should panic ... ok`, `test name ... ignored, reason`
  for (const m of text.matchAll(/^test (\S+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)/gm)) tests.push({ name: m[1], status: m[2] === "ok" ? "ok" : m[2] === "FAILED" ? "failed" : "ignored", output: "" });
  for (const m of text.matchAll(/^---- (\S+) stdout ----\n([\s\S]*?)(?=^---- \S+ stdout ----|^failures:|^test result:)/gm)) {
    const t = tests.find((x) => x.name === m[1]);
    if (t) t.output = m[2].trim();
  }
  const summary = text.match(/^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed/m);
  return { tests, finished: !!summary, ran: /^running \d+ tests?/m.test(text) };
}

/** A learner's code with extra hidden tests for `cargo test` exercises. */
export function withHiddenTests(code, hidden) {
  if (!hidden) return code;
  const body = String(hidden)
    .replace(/\s*$/, "")
    .split("\n")
    .map((l) => (l ? "    " + l : l))
    .join("\n");
  return String(code).replace(/\s*$/, "\n") + `\n#[cfg(test)]\n#[allow(unused_imports)]\nmod arena_hidden_tests {\n    use super::*;\n${body}\n}\n`;
}

/** Apply one mutant (a broken copy of the code the learner's tests must catch). Null if the text isn't there. */
export function applyMutant(code, mutant) {
  const at = String(code).indexOf(mutant.find);
  if (at < 0) return null;
  return code.slice(0, at) + mutant.replace + code.slice(at + mutant.find.length);
}
