// Compiles and runs Rust code on this machine, answering like the Rust Playground's
// /execute endpoint. Used by the content build (to verify every reference solution) and by
// the local runner (scripts/local-runner.mjs), which serves the same API to the site.
//
// Lesson code may use the crates listed in runner-deps/Cargo.toml (all of them are also on the
// Playground). They're built once with Cargo; each program is then compiled with plain rustc
// and linked against that build, which takes well under a second.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const DEPS = path.join(ROOT, "runner-deps");
export const EDITION = "2024";

export function run(cmd, args, { input = "", timeout = 20000, cwd, env } = {}) {
  return new Promise((resolve) => {
    const p = spawn(cmd, args, { stdio: ["pipe", "pipe", "pipe"], cwd, env: { ...process.env, ...env } });
    let out = "";
    let err = "";
    let timedOut = false;
    const t = setTimeout(() => {
      timedOut = true;
      p.kill("SIGKILL");
    }, timeout);
    p.stdout.on("data", (d) => (out += d));
    p.stderr.on("data", (d) => (err += d));
    p.on("error", (e) => {
      clearTimeout(t);
      resolve({ code: null, signal: null, out, err: err + String(e), timedOut });
    });
    p.on("close", (code, signal) => {
      clearTimeout(t);
      resolve({ code, signal, out, err, timedOut });
    });
    p.stdin.on("error", () => {});
    p.stdin.end(input);
  });
}

let externs = null;

/** Build runner-deps once and return the rustc flags that make its crates available. */
export function depsFlags() {
  if (externs) return externs;
  // The exact packages runner-deps depends on directly (a crate can also appear deeper in the tree in another version).
  const meta = spawnSync("cargo", ["metadata", "--format-version", "1"], { cwd: DEPS, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  if (meta.status !== 0) throw new Error("cargo metadata failed in runner-deps:\n" + meta.stderr);
  const md = JSON.parse(meta.stdout);
  const rootId = md.resolve.root;
  const direct = new Map(md.resolve.nodes.find((n) => n.id === rootId).deps.map((d) => [d.pkg, d.name]));
  const r = spawnSync("cargo", ["build", "--quiet", "--message-format=json"], { cwd: DEPS, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  if (r.status !== 0) throw new Error("Building runner-deps failed:\n" + r.stderr);
  const flags = ["-L", "dependency=" + path.join(DEPS, "target", "debug", "deps")];
  for (const line of r.stdout.split("\n")) {
    if (!line.startsWith("{")) continue;
    const m = JSON.parse(line);
    if (m.reason !== "compiler-artifact" || !direct.has(m.package_id)) continue;
    const rlib = m.filenames.find((f) => f.endsWith(".rlib"));
    if (rlib) flags.push("--extern", `${direct.get(m.package_id)}=${rlib}`);
  }
  externs = flags;
  return flags;
}

/**
 * Compile and run like the Playground: { success, exitDetail, stdout, stderr }.
 * `tests: true` builds with --test and runs the test harness (like `cargo test`).
 * stderr holds the compiler's messages followed by the program's stderr, as on the Playground.
 */
export async function execute({ code, tests = false, edition = EDITION, timeout = 15000 }) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "rustarena-run-"));
  try {
    fs.mkdirSync(path.join(dir, "src"));
    fs.writeFileSync(path.join(dir, "src", "main.rs"), code);
    const exe = path.join(dir, "playground");
    const args = ["--edition", edition, "--crate-name", "playground", "--crate-type", "bin", "-C", "opt-level=0", "-C", "debuginfo=0", "--color", "never", ...(tests ? ["--test"] : []), ...depsFlags(), "src/main.rs", "-o", exe];
    const c = await run("rustc", args, { cwd: dir, timeout: 120000 });
    const compileLog = c.err.trim() ? c.err.replace(/\s*$/, "\n") : "";
    if (c.code !== 0) {
      return { success: false, exitDetail: "", stdout: "", stderr: `   Compiling playground v0.0.1 (/playground)\n${compileLog}` };
    }
    const r = await run(exe, tests ? ["--test-threads=1"] : [], { cwd: dir, timeout, env: { RUST_BACKTRACE: "0" } });
    const exitDetail = r.timedOut ? "Killed (timeout)" : r.signal ? `Terminated by signal ${r.signal}` : `Exited with status ${r.code}`;
    const header = `   Compiling playground v0.0.1 (/playground)\n${compileLog}    Finished \`dev\` profile [unoptimized + debuginfo] target(s)\n     Running \`target/debug/playground\`\n`;
    return { success: !r.timedOut && r.code === 0, exitDetail, stdout: r.out, stderr: header + r.err };
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

/** Run clippy on a program (used to keep reference solutions idiomatic). Returns the warnings text. */
export async function clippy(code, edition = EDITION) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "rustarena-clippy-"));
  try {
    fs.mkdirSync(path.join(dir, "src"));
    fs.writeFileSync(path.join(dir, "src", "main.rs"), code);
    const r = await run("clippy-driver", ["--edition", edition, "--crate-type", "bin", "--crate-name", "playground", "--color", "never", "-A", "dead_code", ...depsFlags(), "src/main.rs", "-o", path.join(dir, "out")], { cwd: dir, timeout: 120000 });
    return r.err.trim();
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}
