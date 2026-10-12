// End-to-end test in headless Chromium.
// Serves dist/ under a sub-path (like GitHub Pages) and drives the real UI: lessons pass and
// fail, compiler errors and panics are explained, Deathmatch runs, projects check, the
// playground runs, plus accessibility (axe) and SEO checks.
//
// The site sends code to the Rust Playground. Here every request to play.rust-lang.org is
// answered by the local runner (scripts/local-runner.mjs, the same API on this machine's
// rustc), so the tests use the production code path without needing the network.
//
// Usage: npm run build && node scripts/e2e.mjs [--shots dir]
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
import { chromium } from "playwright";
import AxeBuilder from "@axe-core/playwright";
import { loadContent } from "./load-content.mjs";
import { parseTemplate } from "../src/grader/assemble.js";

const DIST = path.resolve("dist");
const PORT = 4789;
const RUNNER_PORT = 8799;
const RUNNER = `http://127.0.0.1:${RUNNER_PORT}`;
const KEY = "rustarena-v1";
const BASE_PATH = process.env.BASE_PATH ?? "/";
const BASE = `http://localhost:${PORT}${BASE_PATH}`;
const shotsIdx = process.argv.indexOf("--shots");
const SHOTS = shotsIdx > 0 ? process.argv[shotsIdx + 1] : null;
if (SHOTS) fs.mkdirSync(SHOTS, { recursive: true });

// ---------------------------------------------------------------- servers
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".svg": "image/svg+xml", ".json": "application/json", ".png": "image/png", ".woff2": "font/woff2", ".webmanifest": "application/manifest+json", ".gz": "application/gzip" };
const server = http.createServer((req, res) => {
  let p = decodeURIComponent(new URL(req.url, "http://x").pathname);
  if (!p.startsWith(BASE_PATH)) {
    res.writeHead(404).end();
    return;
  }
  p = p.slice(BASE_PATH.length - 1);
  // Like GitHub Pages: a directory serves its index.html, and unknown paths get 404.html.
  let f = path.join(DIST, p);
  if (fs.existsSync(f) && fs.statSync(f).isDirectory()) f = path.join(f, "index.html");
  let status = 200;
  if (!f.startsWith(DIST) || !fs.existsSync(f)) {
    f = path.join(DIST, "404.html");
    status = 404;
  }
  res.writeHead(status, { "Content-Type": types[path.extname(f)] ?? "application/octet-stream" });
  fs.createReadStream(f).pipe(res);
});
await new Promise((r) => server.listen(PORT, r));

const runner = spawn(process.execPath, [path.resolve("scripts/local-runner.mjs")], { env: { ...process.env, PORT: String(RUNNER_PORT), PARALLEL: "3" }, stdio: ["ignore", "pipe", "inherit"] });
await new Promise((resolve, reject) => {
  const t = setTimeout(() => reject(new Error("local runner did not start")), 600000);
  runner.stdout.on("data", (d) => {
    if (String(d).includes("local runner on")) {
      clearTimeout(t);
      resolve();
    }
  });
  runner.on("exit", (c) => reject(new Error("local runner exited with " + c)));
});
let runnerCalls = 0;

/** Answer requests to the Rust Playground from the local runner, as the real one would (with CORS). */
async function routePlayground(ctx) {
  await ctx.route(/^https:\/\/play\.rust-lang\.org\/(execute|evaluate\.json)$/, async (route) => {
    const req = route.request();
    const cors = { "access-control-allow-origin": "*", "access-control-allow-headers": "Content-Type", "access-control-allow-methods": "GET, POST" };
    if (req.method() === "OPTIONS") return route.fulfill({ status: 204, headers: cors });
    runnerCalls++;
    const res = await fetch(RUNNER + new URL(req.url()).pathname, { method: "POST", headers: { "Content-Type": "application/json" }, body: req.postData() });
    await route.fulfill({ status: res.status, headers: { ...cors, "content-type": "application/json" }, body: await res.text() });
  });
}

// ---------------------------------------------------------------- browser
const exe = process.env.CHROMIUM_PATH || (fs.existsSync("/opt/pw-browsers/chromium") ? "/opt/pw-browsers/chromium" : undefined);
const browser = await chromium.launch(exe ? { executablePath: exe } : { channel: "chromium" });
// Deathmatch, the daily pick and choice order use Math.random. Seed it in every page so a run is
// repeatable: the same seed gives the same drills in the same order. E2E_SEED picks another sequence.
const SEED = Number(process.env.E2E_SEED ?? 1);
const newContext = browser.newContext.bind(browser);
browser.newContext = async (opts) => {
  const ctx = await newContext(opts);
  await ctx.addInitScript((seed) => {
    let a = seed >>> 0;
    Math.random = () => {
      // mulberry32
      a = (a + 0x6d2b79f5) >>> 0;
      let t = a;
      t = Math.imul(t ^ (t >>> 15), t | 1);
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }, SEED);
  await routePlayground(ctx);
  return ctx;
};
/** Seed saved progress once, the first time a page in this context loads. */
async function seed(ctx, state) {
  await ctx.addInitScript(
    ([k, st]) => {
      if (!localStorage.getItem(k)) localStorage.setItem(k, JSON.stringify({ version: 1, steps: {}, projects: {}, drills: {}, ...st }));
    },
    [KEY, state],
  );
}

// Service workers are tested on their own below; elsewhere they'd only add caching to reason about.
const mainCtx = await browser.newContext({ viewport: { width: 1360, height: 900 }, serviceWorkers: "block" });
const page = await mainCtx.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("console", (m) => m.type() === "error" && errors.push("console: " + m.text()));
let failures = 0;
const ok = (name) => console.log("  ✓", name);
const bad = (name, e) => {
  failures++;
  console.log("  ✗", name, "\n   ", String(e?.message ?? e).split("\n").slice(0, 8).join("\n    "));
};
async function test(name, fn) {
  try {
    await fn();
    ok(name);
  } catch (e) {
    bad(name, e);
    if (SHOTS) await page.screenshot({ path: path.join(SHOTS, "fail-" + name.replace(/\W+/g, "_") + ".png"), fullPage: true }).catch(() => {});
  }
}
const shot = async (name) => SHOTS && page.screenshot({ path: path.join(SHOTS, name + ".png"), fullPage: true });
const go = async (route) => {
  await page.goto("about:blank");
  await page.goto(BASE + route.replace(/^\//, ""));
  await page.waitForLoadState("domcontentloaded");
};
const setEditor = async (code, pg = page) => {
  await pg.locator(".cm-content").first().waitFor();
  await pg.evaluate((c) => {
    const view = window.__cmView;
    if (!view) throw new Error("no editor view");
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: c } });
  }, code);
};
const check = async (label = /^Check/, pg = page) => {
  await pg.getByRole("button", { name: label }).first().click();
  await pg.waitForSelector(".results", { timeout: 240000 });
  await pg.waitForFunction(() => !document.querySelector(".actions .btn-primary")?.hasAttribute("disabled"), null, { timeout: 240000 });
};
const passed = (pg = page) => pg.locator(".banner", { hasText: "All tests passed" }).first().waitFor({ timeout: 10000 });

// ---------------------------------------------------------------- content to test with
const content = loadContent();
const stepPath = (m, s) => `/learn/${m.id}/${s.slug}`;
/** The address of a module's nth step (1-based). */
const L = (moduleId, n) => {
  const m = content.modules.find((x) => x.id === moduleId);
  if (!m?.steps[n - 1]) throw new Error(`no step ${n} in ${moduleId}`);
  return stepPath(m, m.steps[n - 1]);
};
/** The first lesson step matching a test, with its module. */
function findStep(pred) {
  for (const m of content.modules) for (const s of m.steps) if (pred(s, m)) return { m, s, path: stepPath(m, s) };
  return null;
}
const hello = content.modules[0];
const fillStep = findStep((s) => s.kind === "fill" && s.mode === "stdout");
const codeStep = findStep((s) => s.kind === "code" && s.mode === "stdout" && s.tests.every((t) => !t.stdin && !t.args && !t.files));
const harnessStep = findStep((s) => s.mode === "harness" && (s.checks ?? 0) >= 2);
const stdinStep = findStep((s) => s.mode === "stdout" && s.kind === "code" && s.tests.some((t) => t.hidden) && s.tests.some((t) => t.stdin));
const argsStep = findStep((s) => s.mode === "stdout" && s.kind === "code" && s.tests.some((t) => t.args?.length || t.exit != null));
const filesStep = findStep((s) => s.mode === "stdout" && s.kind === "code" && s.tests.some((t) => t.files));
const testStep = findStep((s) => s.mode === "test");
const compileFail = findStep((s, m) => m.id === hello.id && s.kind === "code" && /printn!/.test(s.seed));

console.log(`e2e against ${BASE} (random seed ${SEED}); code runs on the local runner at ${RUNNER}`);

await test("home renders with the course in numbers", async () => {
  await go("/");
  await page.getByText("writing real code").waitFor();
  await page.locator(".fact-n").first().waitFor();
  await page.locator(".hero-ferris").waitFor();
  await shot("home");
});

await test("fill step passes with the right answer", async () => {
  await go(fillStep.path);
  const answers = parseTemplate(fillStep.s.seed).blanks.map((b) => b.answer);
  const blanks = page.locator("input.blank");
  for (let i = 0; i < answers.length; i++) await blanks.nth(i).fill(answers[i]);
  await check();
  await passed();
  await shot("step-pass");
});

await test("fill step fails with a wrong answer and shows expected vs got", async () => {
  await go(L(hello.id, 2));
  await page.locator("input.blank").first().fill("I am learning rust");
  await check();
  await page.locator(".t-fail").first().waitFor();
  await page.getByText("you printed / returned").first().waitFor();
  await shot("step-fail");
});

await test("a step's extra challenges: tracker, task card, and the step completes after the last one", async () => {
  await go(L(hello.id, 1));
  await page.getByRole("button", { name: /Challenge 2/ }).click();
  await page.locator(".task-card").waitFor();
  const step = hello.steps[0];
  for (let k = 0; k < step.more.length; k++) {
    await page.getByRole("button", { name: new RegExp(`Challenge ${k + 2}`) }).click();
    const c = step.more[k];
    if (c.kind === "fill") {
      const answers = parseTemplate(c.seed).blanks.map((b) => b.answer);
      for (let i = 0; i < answers.length; i++) await page.locator("input.blank").nth(i).fill(answers[i]);
    } else await setEditor(c.solution);
    await check();
    await passed();
  }
  await page.getByText(/Step complete|All challenges complete|Module complete/).first().waitFor();
});

await test("compile error shows rustc's message, a plain-English note and the error index link", async () => {
  await go(compileFail.path);
  await check();
  await page.locator(".banner", { hasText: "It didn't compile" }).waitFor();
  await page.locator(".diag-friendly").first().waitFor();
  await page.locator(".cm-lint-marker-error, .cm-lintRange-error").first().waitFor();
  await shot("compile-error");
});

await test("borrow-checker error links to the official explanation", async () => {
  await go(codeStep.path);
  await setEditor('fn main() {\n    let s = String::from("hi");\n    let t = s;\n    println!("{} {}", s, t);\n}\n');
  await check();
  await page.getByText("error E0382").first().waitFor();
  const href = await page.getByRole("link", { name: /Explain E0382/ }).getAttribute("href");
  if (href !== "https://doc.rust-lang.org/error_codes/E0382.html") throw new Error("explain link is " + href);
  await page.getByText(/moved/).first().waitFor();
});

await test("code step passes after writing the solution in the editor", async () => {
  await go(codeStep.path);
  await setEditor(codeStep.s.solution);
  await check();
  await passed();
});

await test("function step (hidden #[test]s): wrong code shows expected vs got, the solution passes", async () => {
  await go(harnessStep.path);
  await page.locator(".task-checks li").first().waitFor();
  await setEditor(harnessStep.s.seed);
  await check();
  await page.locator(".t-fail").first().waitFor();
  await setEditor(harnessStep.s.solution);
  await check();
  await passed();
});

await test("a program that reads stdin passes its visible and hidden tests", async () => {
  await go(stdinStep.path);
  await setEditor(stdinStep.s.solution);
  await check();
  await passed();
  const n = await page.locator(".tests li").count();
  if (n !== stdinStep.s.tests.length) throw new Error(`${n} test results for ${stdinStep.s.tests.length} tests`);
});

if (argsStep)
  await test(`command-line arguments and exit codes are graded (${argsStep.s.id})`, async () => {
    await go(argsStep.path);
    await setEditor(argsStep.s.solution);
    await check();
    await passed();
  });

if (filesStep)
  await test(`a step that reads files from the working folder passes (${filesStep.s.id})`, async () => {
    await go(filesStep.path);
    await setEditor(filesStep.s.solution);
    await check();
    await passed();
  });

if (testStep)
  await test(`writing your own tests: run with cargo test and graded against planted bugs (${testStep.s.id})`, async () => {
    await go(testStep.path);
    await setEditor(testStep.s.seed);
    await check();
    await page.locator(".t-fail, .rules li").first().waitFor();
    await setEditor(testStep.s.solution);
    await check();
    await passed();
  });

await test("an endless loop is stopped with a time-limit message", async () => {
  await go(codeStep.path);
  await setEditor("fn main() {\n    let mut n: u64 = 0;\n    loop {\n        n = n.wrapping_add(1);\n    }\n}\n");
  await check();
  await page.getByText(/Time limit exceeded/).first().waitFor();
});

await test("a panic is explained, pointing at the learner's line", async () => {
  await go(codeStep.path);
  await setEditor("fn main() {\n    let v = vec![1, 2, 3];\n    let i = v.len() + 2;\n    println!(\"{}\", v[i]);\n}\n");
  await check();
  await page.getByText(/index out of bounds/).first().waitFor();
  await page.getByText(/at line 4/).first().waitFor();
});

await test("when the runner can't be reached, the learner is told what to do", async () => {
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  await ctx.unroute(/^https:\/\/play\.rust-lang\.org\/(execute|evaluate\.json)$/);
  await ctx.route(/play\.rust-lang\.org/, (r) => r.abort("connectionrefused"));
  const pg = await ctx.newPage();
  await pg.goto(BASE + codeStep.path.slice(1));
  await setEditor(codeStep.s.solution, pg);
  await check(/^Check/, pg);
  await pg.locator(".runner-problem").waitFor();
  await pg.getByRole("link", { name: "use another runner" }).waitFor();
  await pg.locator(".pill-error").first().waitFor();
  await ctx.close();
});

await test("settings: switch to a runner on this computer and test the connection", async () => {
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const pg = await ctx.newPage();
  await pg.goto(BASE + "settings");
  await pg.getByRole("textbox", { name: "Server address" }).fill(RUNNER);
  await pg.getByRole("button", { name: "Use this server" }).click();
  await pg.getByRole("button", { name: "Test the connection" }).click();
  await pg.getByText(/It works: compiled and ran/).waitFor({ timeout: 120000 });
  await pg.getByLabel(/The Rust Playground/).check();
  await ctx.close();
});

// ---------------------------------------------------------------- Deathmatch
await test("deathmatch: play reps until death, respawn", async () => {
  await go("/deathmatch");
  const unlock = page.getByLabel("Unlock every topic");
  if (!(await unlock.isChecked())) await unlock.check();
  await page.getByRole("button", { name: /^Deathmatch/ }).click();
  let answered = 0;
  for (let k = 0; k < 12; k++) {
    await page.locator(".rep, .death").first().waitFor({ timeout: 10000 });
    if (await page.locator(".death").count()) break;
    // Answer every rep wrong on purpose, whatever its type, so the one life runs out.
    const id = await page.locator(".rep").first().getAttribute("data-drill");
    const d = content.drills.find((x) => x.id === id);
    if (!d) throw new Error("unknown drill id " + id);
    if (d.type === "compiles") await page.keyboard.press(d.answer === "yes" ? "n" : "y");
    else if (d.type === "predict") {
      await page.locator(".answer-input").fill("definitely wrong answer");
      await page.keyboard.press("Enter");
    } else if (d.type === "fill") {
      await page.locator("input.blank").fill("zzz");
      await page.keyboard.press("Enter");
    } else if (d.type === "bug") await page.locator("button.bugline").nth(+d.answer === 1 ? 1 : 0).click();
    else if (d.type === "choice") await page.locator(`.rep .choice[data-choice="${+d.answer === 1 ? 1 : 0}"]`).click();
    else if (d.type === "boss") await page.getByRole("button", { name: "Give up" }).click();
    else throw new Error("no wrong answer for drill type " + d.type);
    answered++;
    await page.waitForTimeout(150);
  }
  await page.locator(".death").waitFor({ timeout: 10000 });
  await page.getByText("ELIMINATED", { exact: true }).waitFor();
  await shot("deathmatch-dead");
  await page.keyboard.press("Enter");
  await page.locator(".rep").waitFor();
  console.log(`    answered ${answered} rep(s) before dying`);
});

/** Answer whatever rep is showing correctly. */
async function answerRep(pg, drillList = content.drills) {
  const rep = pg.locator(".rep").first();
  await rep.waitFor({ timeout: 15000 });
  const id = await rep.getAttribute("data-drill");
  const d = drillList.find((x) => x.id === id) ?? content.drills.find((x) => x.id === id);
  if (!d) throw new Error("unknown drill id " + id);
  if (d.type === "predict") {
    await pg.locator(".answer-input").fill(d.answer.replace(/\n/g, " "));
    await pg.keyboard.press("Enter");
  } else if (d.type === "fill") {
    await pg.locator(".rep input.blank").fill(d.answer);
    await pg.keyboard.press("Enter");
  } else if (d.type === "compiles") {
    await pg.getByRole("button", { name: d.answer === "yes" ? /^Compiles/ : /^Compile error/ }).click();
  } else if (d.type === "bug") {
    await pg.locator(".bugline").nth(+d.answer - 1).click();
  } else if (d.type === "choice") {
    await pg.locator(`.choice[data-choice="${+d.answer - 1}"]`).click();
  } else if (d.type === "boss") {
    await setEditor(d.exercise.solution, pg);
    await pg.getByRole("button", { name: /^Fire/ }).click();
    await pg.waitForFunction((prev) => document.querySelector(".rep")?.getAttribute("data-drill") !== prev, id, { timeout: 240000 });
  } else throw new Error("can't answer " + d.type);
  return d;
}

await test("deathmatch: every drill type answered correctly builds a streak (incl. boss reps)", async () => {
  await go("/deathmatch");
  const unlock = page.getByLabel("Unlock every topic");
  if (!(await unlock.isChecked())) await unlock.check();
  await page.getByRole("button", { name: /^Casual/ }).click();
  const seen = new Set();
  let kills = 0;
  for (let k = 0; k < 18; k++) {
    seen.add((await answerRep(page)).type);
    kills++;
    if (await page.locator(".death").count()) throw new Error("died on a correct answer");
    await page.waitForTimeout(80);
  }
  const n = await page.locator(".hud-n").innerText();
  if (+n !== kills) throw new Error(`streak shows ${n}, expected ${kills}`);
  if (!seen.has("boss")) throw new Error("no boss rep appeared");
  console.log(`    streak ${n}, types seen: ${[...seen].join(", ")}`);
  await shot("deathmatch-streak");
});

await test("interview prep: open to everyone, all question types answer correctly", async () => {
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const pg = await ctx.newPage();
  await pg.goto(BASE + "deathmatch");
  await pg.getByRole("button", { name: "Try interview prep" }).click();
  const seen = new Set();
  for (let k = 0; k < 14; k++) {
    await pg.locator(".rep-topic", { hasText: "Interview prep" }).waitFor();
    seen.add((await answerRep(pg)).type);
    if (await pg.locator(".death").count()) throw new Error("a correct answer was marked wrong");
    await pg.waitForTimeout(80);
  }
  if (!seen.has("choice")) throw new Error("no pick-one questions appeared: " + [...seen]);
  const streak = await pg.locator(".hud-n").innerText();
  if (+streak !== 14) throw new Error("streak is " + streak);
  await ctx.close();
});

// ---------------------------------------------------------------- projects, pro track, playground
if (content.projects.length)
await test("project milestone passes, and the next one starts from your code", async () => {
  const p = content.projects[0];
  await go(`/projects/${p.id}`);
  await setEditor(p.milestones[0].solution);
  await check(/^Check milestone/);
  await passed();
  await page.getByRole("button", { name: /Next milestone/ }).click();
  await page.getByText(/Milestone 2:/).waitFor();
  await shot("project");
});

await test("pro track pages render, and the starter pack downloads", async () => {
  await go("/pro");
  await page.getByText("Set up once").waitFor();
  const cmd = await page.locator(".copybox pre").innerText();
  if (!/curl -fsSL .*pro\/rustarena-pro\.tar\.gz \| tar -xz/.test(cmd)) throw new Error("unexpected import command: " + cmd);
  const res = await page.request.get(BASE + "pro/rustarena-pro.tar.gz");
  if (res.status() !== 200 || (await res.body()).length < 10000) throw new Error("starter pack missing: HTTP " + res.status());
  await page.locator(".project-card").first().click();
  await page.getByRole("heading", { name: content.pro[0].title, level: 1 }).waitFor();
  await page.locator(".pro-done input").check();
  await page.waitForTimeout(500); // progress is saved to localStorage after a short debounce
  await go("/pro");
  await page.getByText(`1/${content.pro.length} projects passing`).waitFor();
  await shot("pro");
});

await test("playground: runs with input and arguments, explains errors and panics, and share links carry the code", async () => {
  await go("/playground");
  await page.getByRole("heading", { name: "Playground", level: 1 }).waitFor();
  await setEditor('use std::io::Read;\n\nfn main() {\n    let mut s = String::new();\n    std::io::stdin().read_to_string(&mut s).unwrap();\n    let sum: i64 = s.split_whitespace().map(|x| x.parse::<i64>().unwrap()).sum();\n    let args: Vec<String> = std::env::args().skip(1).collect();\n    println!("sum {sum} args {args:?}");\n}\n');
  await page.getByLabel(/^Input/).fill("20 22");
  await page.getByLabel("Command-line arguments").fill('--fast "two words"');
  await page.getByRole("button", { name: /^Run/ }).click();
  await page.locator(".playground .console", { hasText: 'sum 42 args ["--fast", "two words"]' }).waitFor({ timeout: 60000 });
  await setEditor("fn main() {\n    println!(\"{}\", x);\n}\n");
  await page.getByRole("button", { name: /^Run/ }).click();
  await page.locator(".playground .banner", { hasText: "It didn't compile" }).waitFor({ timeout: 60000 });
  await setEditor('fn main() {\n    let v: Vec<u8> = Vec::new();\n    println!("{}", v[0]);\n}\n');
  await page.getByRole("button", { name: /^Run/ }).click();
  await page.getByText(/index out of bounds/).first().waitFor({ timeout: 60000 });
  // Share, then open the link in a fresh browser: the program and input come along.
  await setEditor('fn main() {\n    let mut w = String::new();\n    std::io::stdin().read_line(&mut w).unwrap();\n    println!("shared {}", w.trim());\n}\n');
  await page.getByLabel(/^Input/).fill("hello");
  await page.getByRole("button", { name: "Share" }).click();
  const link = await page.getByLabel("Share link").inputValue();
  if (!/\/playground#code=/.test(link)) throw new Error("unexpected share link " + link);
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const pg = await ctx.newPage();
  await pg.goto(link.replace(/^https?:\/\/[^/]+\//, `http://localhost:${PORT}/`));
  await pg.waitForFunction(() => document.querySelector("#pg-stdin")?.value === "hello", null, { timeout: 10000 });
  await pg.getByRole("button", { name: /^Run/ }).click();
  await pg.locator(".playground .console", { hasText: "shared hello" }).waitFor({ timeout: 120000 });
  await ctx.close();
});

await test("mobile layout renders the step page", async () => {
  await page.setViewportSize({ width: 390, height: 844 });
  await go(codeStep.path);
  await page.locator(".cm-content").waitFor();
  await shot("mobile-step");
  await go("/roadmap");
  await shot("mobile-roadmap");
  await page.setViewportSize({ width: 1360, height: 900 });
});

await test("progress persisted in localStorage", async () => {
  const s = await page.evaluate((k) => JSON.parse(localStorage.getItem(k)), KEY);
  if (!s.steps[hello.steps[0].id]?.done) throw new Error("the first step was not saved as done");
});

await test("roadmap: topics link to lessons and show progress", async () => {
  await go("/roadmap");
  await page.getByRole("heading", { name: "Rust roadmap", level: 1 }).waitFor();
  const sections = await page.locator(".rm-section").count();
  if (sections !== content.roadmap.length) throw new Error(`${sections} sections for ${content.roadmap.length} in the roadmap`);
  await page.locator(".rm-part, .rm-done").first().waitFor();
  await page.locator(".rm-topic").first().click();
  await page.waitForURL(/\/learn\//);
  await shot("roadmap");
});

// ---------------------------------------------------------------- features
await test("theme: the toggle switches theme and the choice is remembered", async () => {
  await go("/");
  const before = await page.evaluate(() => document.documentElement.dataset.theme);
  const after = before === "dark" ? "light" : "dark";
  await page.getByRole("button", { name: `Switch to ${after} theme` }).click();
  if ((await page.evaluate(() => document.documentElement.dataset.theme)) !== after) throw new Error("theme did not change");
  await page.waitForTimeout(400);
  await go("/learn");
  if ((await page.evaluate(() => document.documentElement.dataset.theme)) !== after) throw new Error("theme not remembered");
  await go("/settings");
  await page.getByLabel("Larger").check();
  const size = await page.evaluate(() => parseFloat(getComputedStyle(document.documentElement).fontSize));
  if (size < 17) throw new Error("text size did not grow: " + size);
  await page.getByLabel("Default").check();
  await page.getByLabel("Match my device").check();
  await page.waitForTimeout(400);
});

await test("symbol bar types into the editor and into blanks on a phone", async () => {
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, serviceWorkers: "block" });
  const ph = await ctx.newPage();
  await ph.goto(BASE + codeStep.path.slice(1));
  await ph.locator(".cm-content").click();
  await ph.keyboard.press("Control+End");
  await ph.getByRole("button", { name: "braces" }).tap();
  await ph.getByRole("button", { name: "semicolon" }).tap();
  const text = await ph.locator(".cm-content").innerText();
  if (!text.includes("{;}")) throw new Error("editor text doesn't end with {;}: " + JSON.stringify(text.slice(-20)));
  await ph.goto(BASE + fillStep.path.slice(1));
  await ph.locator("input.blank").first().fill("x");
  await ph.getByRole("button", { name: "double colon" }).tap();
  const v = await ph.locator("input.blank").first().inputValue();
  if (v !== "x::") throw new Error("blank holds " + v);
  await ctx.close();
});

await test("report a problem opens a pre-filled GitHub issue", async () => {
  await go(codeStep.path);
  const link = page.locator(".report-row").getByRole("link", { name: /Report a problem/ }).first();
  // The link is filled in when it's used (so it carries the latest code): focusing it does that.
  await link.focus();
  const href = await link.evaluate((a) => a.href);
  const u = new URL(href);
  if (!/github\.com\/.+\/issues\/new$/.test(u.origin + u.pathname)) throw new Error("not an issue link: " + href);
  if (!u.searchParams.get("body").includes(codeStep.s.id) || !u.searchParams.get("title").includes("Lesson step")) throw new Error("issue is not pre-filled");
});

if (content.placement.length)
  await test("placement quiz: all right skips every tested module", async () => {
    const ctx = await browser.newContext({ serviceWorkers: "block" });
    const pg = await ctx.newPage();
    await pg.goto(BASE + "placement");
    await pg.getByRole("button", { name: "Start the quiz" }).click();
    for (let i = 0; i < content.placement.length; i++) {
      await pg.getByRole("heading", { name: `Question ${i + 1} of ${content.placement.length}` }).waitFor();
      await answerRep(pg, content.placement);
    }
    await pg.getByRole("heading", { name: "Your starting point" }).waitFor();
    const last = content.modules.findIndex((m) => m.id === content.placement.at(-1).module);
    await pg.getByText(`You got ${content.placement.length} of ${content.placement.length}`).waitFor();
    await pg.getByRole("button", { name: /^Skip \d+ modules/ }).click();
    if (last + 1 < content.modules.length) {
      await pg.getByRole("link", { name: `Go to ${content.modules[last + 1].title}` }).click();
      await pg.waitForURL(new RegExp(`${L(content.modules[last + 1].id, 1)}$`));
    }
    await pg.waitForTimeout(300);
    const placed = await pg.evaluate((k) => JSON.parse(localStorage.getItem(k)).placed.length, KEY);
    if (placed !== last + 1) throw new Error(`placed ${placed} modules, expected ${last + 1}`);
    await pg.goto(BASE + "learn");
    await pg.getByText("skipped by placement").first().waitFor();
    await ctx.close();
  });

await test("daily challenge: answer it, see the streak, and it stays done", async () => {
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const pg = await ctx.newPage();
  await pg.goto(BASE + "daily");
  await answerRep(pg);
  await pg.getByText("Solved", { exact: true }).waitFor();
  await pg.waitForTimeout(300);
  await pg.reload();
  await pg.getByText("Solved", { exact: true }).waitFor();
  await ctx.close();
});

if (content.topics.length)
  await test("topic pages render with their tested example output", async () => {
    const t = content.topics[0];
    await go("/topics");
    await page.getByRole("link", { name: t.title }).click();
    await page.getByRole("heading", { name: t.title, level: 1 }).waitFor();
    await page.locator("pre.console", { hasText: t.output.trim().split("\n")[0] }).first().waitFor();
    await page.getByRole("button", { name: /Edit and run it in the playground/ }).click();
    await page.waitForURL(/\/playground$/);
    await page.getByRole("button", { name: /^Run/ }).click();
    await page.locator(".playground .console", { hasText: t.output.trim().split("\n")[0] }).waitFor({ timeout: 120000 });
  });

await test("sync: a transfer link moves progress to a fresh browser", async () => {
  await go("/profile");
  await page.getByRole("button", { name: "Make a transfer link" }).click();
  const link = await page.getByRole("textbox", { name: "Transfer link" }).inputValue();
  if (!link.includes("/profile#transfer=")) throw new Error("bad link " + link);
  await page.locator(".qr svg").waitFor();
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const pg = await ctx.newPage();
  await pg.goto(link);
  await pg.getByRole("button", { name: "Merge into this device" }).click();
  await pg.getByText(/Merged/).waitFor();
  await pg.waitForTimeout(300);
  const done = await pg.evaluate(([k, id]) => JSON.parse(localStorage.getItem(k)).steps[id]?.done, [KEY, hello.steps[0].id]);
  if (!done) throw new Error("progress did not arrive");
  await ctx.close();
});

await test("certificate: appears when every step is done and its share link opens", async () => {
  const ctx = await browser.newContext({ serviceWorkers: "block", permissions: ["clipboard-read", "clipboard-write"] });
  const steps = Object.fromEntries(content.modules.flatMap((m) => m.steps.map((s) => [s.id, { done: true, hintsUsed: 0, clean: true, doneAt: Date.UTC(2026, 8, 1) }])));
  await seed(ctx, { steps, settings: { certName: "Ferris Crab" } });
  const pg = await ctx.newPage();
  await pg.goto(BASE + "certificate");
  await pg.locator(".cert-name", { hasText: "Ferris Crab" }).waitFor();
  await pg.getByRole("button", { name: "Print or save as PDF" }).waitFor();
  const download = pg.waitForEvent("download");
  await pg.getByRole("button", { name: /Share image/ }).click();
  const file = await download;
  if (file.suggestedFilename() !== "rustarena-certificate-course.png") throw new Error("share image is " + file.suggestedFilename());
  const size = fs.statSync(await file.path()).size;
  if (size < 10000) throw new Error("share image is only " + size + " bytes");
  await pg.getByRole("button", { name: "Copy link" }).click();
  const url = await pg.evaluate(() => navigator.clipboard.readText());
  if (!url.includes("/certificate#view=")) throw new Error("bad share link " + url);
  const viewer = await browser.newContext({ serviceWorkers: "block" });
  const vp = await viewer.newPage();
  await vp.goto(url);
  await vp.locator(".cert-name", { hasText: "Ferris Crab" }).waitFor();
  await vp.getByText(/can't be independently verified/).waitFor();
  await viewer.close();
  await ctx.close();
});

await test("offline: after one visit, lessons open without a connection, and running code says why it can't", async () => {
  // A fresh browser with the service worker on, and a single visit (no reload), like a learner's first.
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const pg = await ctx.newPage();
  await pg.goto(BASE + "playground");
  await pg.evaluate(() => navigator.serviceWorker.ready);
  await pg.waitForTimeout(1000);
  await ctx.setOffline(true);
  // The test's stand-in for the Playground runs outside the browser, so cut it off too.
  await ctx.route(/play\.rust-lang\.org/, (r) => r.abort("internetdisconnected"));
  // Pages this browser never opened: the saved app shows them.
  await pg.goto(BASE + L(hello.id, 1).slice(1));
  await pg.getByRole("heading", { name: hello.steps[0].title }).waitFor();
  await pg.goto(BASE + "roadmap");
  await pg.getByRole("heading", { name: "Rust roadmap", level: 1 }).waitFor();
  await pg.goto(BASE + "playground");
  await pg.getByRole("button", { name: /^Run/ }).click();
  await pg.locator(".runner-problem").waitFor({ timeout: 60000 });
  await ctx.close();
});

// ---------------------------------------------------------------- accessibility
// axe-core checks WCAG 2.2 A/AA rules (plus best practices) on every kind of page,
// including interactive states: results shown, a drill in progress, the death screen.
const a11yCtx = await browser.newContext({ viewport: { width: 1360, height: 900 }, serviceWorkers: "block", colorScheme: "dark" });
await seed(a11yCtx, { settings: { sound: false, unlockAll: true, topics: null, boss: false, keys: true } });
const ap = await a11yCtx.newPage();
async function axe(label, pg = ap) {
  const r = await new AxeBuilder({ page: pg }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"]).analyze();
  if (r.violations.length) {
    throw new Error(
      `${label}: ` +
        r.violations.map((v) => `${v.id} (${v.impact}, ${v.nodes.length}x) at ${v.nodes[0].target.join(" ")}: ${v.nodes[0].failureSummary?.split("\n")[1] ?? ""}`).join("\n"),
    );
  }
}
const apGo = async (route) => {
  await ap.goto(BASE + route.replace(/^\//, ""));
  await ap.locator("#main h1, #main .hero h1").first().waitFor({ timeout: 15000 });
};
const someRoutes = () => {
  const mid = content.modules[Math.floor(content.modules.length / 2)];
  const last = content.modules.at(-1);
  return [
    "/",
    "/roadmap",
    "/learn",
    fillStep.path,
    harnessStep.path,
    L(mid.id, 1),
    L(last.id, 1),
    "/deathmatch",
    "/projects",
    ...(content.projects[0] ? [`/projects/${content.projects[0].id}`] : []),
    "/pro",
    ...(content.pro[0] ? [`/pro/${content.pro[0].id}`, `/pro/${content.pro.at(-1).id}`] : []),
    "/profile",
    "/settings",
    "/about",
    "/no-such-page",
    "/topics",
    ...(content.topics[0] ? [`/topics/${content.topics[0].slug}`] : []),
    "/daily",
    "/placement",
    "/certificate",
    "/next",
    "/playground",
  ];
};

await test("accessibility: every page type passes axe (WCAG 2.2 AA)", async () => {
  const problems = [];
  for (const r of someRoutes()) {
    await apGo(r);
    await ap.waitForTimeout(300);
    try {
      await axe(r);
    } catch (e) {
      problems.push(e.message);
    }
  }
  if (problems.length) throw new Error(problems.join("\n"));
});

await test("accessibility: results, compile errors and hints pass axe", async () => {
  await apGo(L(hello.id, 2));
  await ap.locator("input.blank").first().fill("wrong");
  await ap.getByRole("button", { name: /^Check/ }).click();
  await ap.locator(".t-fail").first().waitFor({ timeout: 240000 });
  await ap.getByRole("button", { name: /Hint/ }).click();
  await axe("failed fill step");
  const status = await ap.locator('[role="status"]').first().textContent();
  if (!/tests? failed/.test(status ?? "")) throw new Error("results were not announced: " + status);
  await apGo(compileFail.path);
  await ap.getByRole("button", { name: /^Check/ }).click();
  await ap.locator(".banner", { hasText: "It didn't compile" }).waitFor({ timeout: 60000 });
  await ap.locator(".diag-rendered summary").first().click();
  await axe("compile error");
});

await test("accessibility: deathmatch reps and the death screen pass axe", async () => {
  await apGo("/deathmatch");
  await axe("lobby");
  await ap.getByRole("button", { name: /^Casual/ }).click();
  const seen = new Set();
  for (let k = 0; k < 30 && seen.size < 4; k++) {
    await ap.locator(".rep, .death").first().waitFor({ timeout: 10000 });
    if (await ap.locator(".death").count()) {
      await ap.waitForTimeout(400);
      await axe("review screen");
      await ap.getByRole("button", { name: /Continue|Respawn/ }).click();
      continue;
    }
    const type = (await ap.locator(".rep").getAttribute("class")).match(/rep-(\w+)/)[1];
    if (!seen.has(type)) {
      seen.add(type);
      await ap.waitForTimeout(400); // let the rep's fade-in finish so contrast is measured at full opacity
      await axe("rep " + type);
    }
    if (type === "compiles") await ap.getByRole("button", { name: /Compiles/ }).click();
    else if (type === "predict") {
      await ap.getByRole("textbox", { name: /What does it print/ }).fill("x");
      await ap.keyboard.press("Enter");
    } else if (type === "fill") {
      await ap.locator("input.blank").fill("zzz");
      await ap.keyboard.press("Enter");
    } else if (type === "bug") await ap.getByRole("button", { name: /^Line \d+:/ }).first().click();
    else if (type === "choice") await ap.locator(".rep .choice").first().click();
    else break;
    await ap.waitForTimeout(150);
  }
  if (seen.size < 3) throw new Error("only saw rep types: " + [...seen].join(", "));
});

await test("phone width: pages fit the screen and pass axe", async () => {
  // At 360px wide, tables and code blocks scroll inside the page, never the page itself,
  // and whatever scrolls must be reachable with the keyboard.
  const phone = await browser.newContext({ viewport: { width: 360, height: 780 }, isMobile: true, hasTouch: true, serviceWorkers: "block", colorScheme: "dark" });
  await seed(phone, { settings: { sound: false, unlockAll: true, topics: null, boss: false, keys: true } });
  const pp = await phone.newPage();
  const problems = [];
  for (const r of someRoutes().filter((x) => x !== "/no-such-page")) {
    await pp.goto(BASE + r.replace(/^\//, ""));
    const h1 = pp.locator("#main h1").first();
    await h1.waitFor({ timeout: 15000 });
    if (!(await h1.textContent())?.trim()) problems.push(`${r}: empty h1`);
    await pp.waitForTimeout(300);
    const extra = await pp.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    if (extra > 1) problems.push(`${r}: page scrolls sideways by ${extra}px`);
    const res = await new AxeBuilder({ page: pp }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"]).analyze();
    for (const v of res.violations) problems.push(`${r}: ${v.id} (${v.nodes.length}x) at ${v.nodes[0].target.join(" ")}`);
  }
  await phone.close();
  if (problems.length) throw new Error(problems.join("\n"));
});

await test("accessibility: the light theme passes axe", async () => {
  await ap.evaluate((k) => {
    const s = JSON.parse(localStorage.getItem(k));
    s.settings.theme = "light";
    localStorage.setItem(k, JSON.stringify(s));
  }, KEY);
  const problems = [];
  for (const r of someRoutes()) {
    await apGo(r);
    await ap.waitForTimeout(400);
    if ((await ap.evaluate(() => document.documentElement.dataset.theme)) !== "light") problems.push(r + ": not in the light theme");
    try {
      await axe("light " + r);
    } catch (e) {
      problems.push(e.message);
    }
  }
  await apGo(L(hello.id, 2));
  await ap.locator("input.blank").first().fill("wrong");
  await ap.getByRole("button", { name: /^Check/ }).click();
  await ap.locator(".t-fail").first().waitFor({ timeout: 240000 });
  await ap.waitForTimeout(400);
  try {
    await axe("light failed step");
  } catch (e) {
    problems.push(e.message);
  }
  await apGo("/deathmatch");
  await ap.getByRole("button", { name: /^Interview prep/ }).click();
  await ap.locator(".rep").waitFor();
  await ap.waitForTimeout(400);
  try {
    await axe("light interview rep");
  } catch (e) {
    problems.push(e.message);
  }
  await ap.evaluate((k) => {
    const s = JSON.parse(localStorage.getItem(k));
    s.settings.theme = "dark";
    localStorage.setItem(k, JSON.stringify(s));
  }, KEY);
  if (problems.length) throw new Error(problems.join("\n"));
});

await test("keyboard: skip link, focus moves to the new page, editor can be left", async () => {
  await apGo("/learn");
  await ap.keyboard.press("Tab");
  const first = await ap.evaluate(() => document.activeElement?.textContent?.trim());
  if (first !== "Skip to content") throw new Error("first Tab stop is " + first);
  await ap.keyboard.press("Enter");
  const inMain = await ap.evaluate(() => !!document.activeElement?.closest("#main") || document.activeElement?.id === "main");
  if (!inMain) throw new Error("skip link did not move focus to the content");
  await ap.getByRole("link", { name: "Projects" }).first().click();
  await ap.getByRole("heading", { name: "Projects", level: 1 }).waitFor();
  const focused = await ap.evaluate(() => document.activeElement?.tagName + ":" + document.activeElement?.textContent);
  if (!/^H1:Projects/.test(focused)) throw new Error("after navigation focus is on " + focused);
  const title = await ap.title();
  if (!/^Projects \| RustArena/.test(title)) throw new Error("page title is " + title);
  await apGo(codeStep.path);
  await ap.locator(".cm-content").click();
  await ap.keyboard.press("Escape");
  await ap.keyboard.press("Tab");
  const left = await ap.evaluate(() => !document.activeElement?.closest(".cm-editor"));
  if (!left) throw new Error("Escape then Tab did not leave the code editor");
});

// ---------------------------------------------------------------- SEO
await test("SEO: real URLs, per-page metadata and a sitemap", async () => {
  const get = async (p) => (await page.request.get(BASE + p)).text();
  const s = codeStep.s;
  const html = await get(codeStep.path.slice(1) + "/");
  const escTitle = s.title.replace(/[.*+?^${}()|[\]\\]/g, "\\$&").replace(/'/g, "(?:'|&#39;)").replace(/"/g, "&quot;");
  for (const [re, what] of [
    [new RegExp(`<title>${escTitle} · .* \\(Rust\\) \\| RustArena</title>`), "title"],
    [/<meta name="description" content="[^"]{20,}/, "description"],
    [new RegExp(`<link rel="canonical" href="https://[^"]+${codeStep.path}/"`), "canonical"],
    [/<meta property="og:image"/, "og:image"],
    [/"@type":"LearningResource"/, "structured data"],
    [/"programmingLanguage":"Rust"/, "programming language"],
  ])
    if (!re.test(html)) throw new Error("step page is missing its " + what);
  const home = await get("");
  if (!/"@type":"Course"/.test(home)) throw new Error("home page has no Course structured data");
  const sitemap = await get("sitemap.xml");
  const urls = (sitemap.match(/<loc>/g) ?? []).length;
  const steps = content.modules.reduce((n, m) => n + m.steps.length, 0);
  if (urls < steps) throw new Error(`sitemap has only ${urls} URLs for ${steps} steps`);
  for (const p of ["/roadmap/", "/daily/", "/placement/", "/next/", "/playground/", "/about/"]) if (!sitemap.includes(p)) throw new Error("sitemap is missing " + p);
  if (content.topics[0]) {
    const t = content.topics[0];
    const topicHtml = await get(`topics/${t.slug}/`);
    if (!/"@type":"TechArticle"/.test(topicHtml)) throw new Error("topic page is not pre-rendered");
  }
  const manifest = await page.request.get(BASE + "manifest.webmanifest");
  if (manifest.status() !== 200 || !(await manifest.json()).icons?.length) throw new Error("web app manifest missing");
  if (/\/(profile|settings|certificate)\/<\/loc>/.test(sitemap)) throw new Error("a private page is in the sitemap");
  const titles = new Set();
  for (const m of content.modules.slice(0, 6)) titles.add((await get(L(m.id, 1).slice(1) + "/")).match(/<title>(.*?)<\/title>/)[1]);
  if (titles.size !== Math.min(6, content.modules.length)) throw new Error("pages share titles");
  // Old hash links (#/learn/...) land on the real address.
  await page.goto(BASE + "#" + L(hello.id, 2));
  await page.waitForURL(new RegExp(`${L(hello.id, 2)}$`));
  await page.getByRole("heading", { name: hello.steps[1].title }).waitFor();
  const res = await page.request.get(BASE + "no-such-page");
  if (res.status() !== 404) throw new Error("unknown pages should return 404, got " + res.status());
  await page.goto(BASE + "no-such-page");
  await page.getByRole("heading", { name: "Page not found" }).waitFor();
});
await a11yCtx.close();

const realErrors = errors.filter((e) => !/favicon|Failed to load resource/.test(e));
if (realErrors.length) {
  console.log("\nBrowser errors:\n  " + realErrors.join("\n  "));
}
console.log(`\n${runnerCalls} programs compiled and run by the local runner`);
await browser.close();
server.close();
runner.kill();
console.log(failures ? `\n${failures} e2e failure(s)` : "\nall e2e checks passed");
process.exit(failures ? 1 : 0);
