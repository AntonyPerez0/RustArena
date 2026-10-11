import { fillFirstBlank } from "../src/grader/assemble.js";

// Works out which lesson step teaches each drill, so Deathmatch can unlock a drill
// only after the learner has done that step (not as soon as they start the module).
//
// A drill "uses" the language and library features in its code: keywords, paths like
// HashMap::new, method calls like .iter(), macros like vec!, and a few pieces of syntax
// (?, &mut, lifetimes, closures). Each feature is "introduced" by the first step of the
// drill's module that uses it, in its text or code. The drill belongs to the latest of those
// steps. Features the module never mentions (things from earlier modules, or the drill's own
// names) don't count. A drill can name its step explicitly with `after: <step id>` in its YAML.

const KEYWORDS = new Set(
  (
    "as async await break const continue crate dyn else enum extern fn for if impl in let loop match mod move mut pub ref return " +
    "self Self static struct super trait type unsafe use where while"
  ).split(" "),
);
// Everywhere from the first lesson on; they carry no information about which step a drill needs.
const COMMON = new Set("fn main let println! print! format! String i32 u32 usize f64 bool str if else for return mut".split(" "));

/** The features a piece of code (or `inline code` in prose) uses. */
export function features(code) {
  const out = new Set();
  const src = code.replace(/\/\/[^\n]*|\/\*[\s\S]*?\*\//g, " ").replace(/r#*"[\s\S]*?"#*|"(?:[^"\\]|\\.)*"|'(?:[^'\\\n]|\\.)'/g, '""');
  // Names the code defines itself (types, functions, macros) aren't features it learned.
  const own = new Set();
  for (const m of src.matchAll(/\b(?:struct|enum|trait|type|fn|mod|union|macro_rules!)\s+([A-Za-z_]\w*)/g)) own.add(m[1]);
  for (const m of src.matchAll(/(?:[A-Za-z_]\w*::)+[A-Za-z_]\w*!?|[A-Za-z_]\w*!?/g)) {
    const t = m[0];
    const i = m.index;
    const after = src.slice(i + t.length).match(/^\s*(\(|<|\{)?/)[1];
    const before = src.slice(Math.max(0, i - 1), i);
    if (t.endsWith("!")) out.add(t);
    else if (t.includes("::")) {
      const parts = t.split("::");
      if (!own.has(parts[0])) out.add(t.replace(/^std::/, ""));
      // The type in a path (HashMap in HashMap::new) is a feature on its own too.
      for (const p of parts) if (/^[A-Z]/.test(p) && !own.has(p)) out.add(p);
    } else if (KEYWORDS.has(t)) out.add(t);
    else if (before === "." && after === "(" && !own.has(t)) out.add("." + t);
    else if (/^[A-Z][A-Za-z0-9]+$/.test(t) && !own.has(t)) out.add(t);
    else if (after === "(" && !own.has(t) && /^[a-z_]\w*$/.test(t)) out.add(t + "()");
  }
  // A few ideas that are syntax rather than names.
  if (/\)\s*\?|\w\?[;.)]/.test(src)) out.add("? operator");
  if (/&\s*mut\b/.test(src)) out.add("&mut");
  if (/&\s*[a-z_(\[]/.test(src)) out.add("&");
  if (/'[a-z_]\w*\b(?!')/.test(src)) out.add("lifetime");
  if (/\|[^|\n]*\|\s*[^|=]/.test(src) && /(?:\(|,|=|move)\s*\|/.test(src)) out.add("closure");
  if (/\bif\s+let\b/.test(src)) out.add("if let");
  if (/\bwhile\s+let\b/.test(src)) out.add("while let");
  if (/\blet\b[^;=]*=[^;]*\belse\s*\{/.test(src)) out.add("let else");
  if (/=>/.test(src)) out.add("match arm");
  if (/\[[^\]\n;]+;\s*[\w.]+\s*\]/.test(src)) out.add("array repeat");
  if (/:\s*\[\s*[\w<>&']+\s*;\s*\d+\s*\]/.test(src)) out.add("array type");
  if (/[A-Za-z_)\]]\.\d+\b/.test(src)) out.add("tuple index");
  if (/\blet\s*(?:mut\s+)?\(/.test(src)) out.add("tuple destructuring");
  if (/\.\.=?/.test(src)) out.add("range");
  if (/#\[derive\(/.test(src)) out.add("derive");
  for (const m of src.matchAll(/#\[derive\(([^)]*)\)\]/g)) for (const d of m[1].split(",")) out.add("derive " + d.trim());
  for (const t of COMMON) out.delete(t), out.delete(t + "()");
  return out;
}

/** Code blocks and `inline code` of a Markdown text, joined. */
function codeOf(md) {
  const parts = [];
  for (const m of md.matchAll(/```[^\n]*\n([\s\S]*?)```/g)) parts.push(m[1]);
  for (const m of md.replace(/```[\s\S]*?```/g, "").matchAll(/`([^`\n]+)`/g)) parts.push(m[1]);
  return parts.join("\n");
}

const STOP = new Set(
  (
    "about above after again also always another answer anything around because before being below between both cannot could does doesn't " +
    "during each either every example first from gets give given have here into it's just keep know later least less like line lines " +
    "make makes many might more most much must need never next nothing only other same should since some something still such than that " +
    "their them then there these they thing things this those though through under until using value values very what when where which " +
    "while whole will with without would write your you'll you're print prints right wrong code program step steps true false"
  ).split(" "),
);

/** Lowercase prose words that might name a concept ("deadlock", "collapsing"), crudely de-pluralized. */
function words(text) {
  const out = new Set();
  for (const m of text.toLowerCase().matchAll(/[a-z][a-z'-]{3,}/g)) {
    const w = m[0].replace(/'s$/, "").replace(/(?<=[a-z]{3})(es|s)$/, "");
    if (!STOP.has(m[0]) && !STOP.has(w)) out.add(w);
  }
  return out;
}

/**
 * Sets `step` (a step id) on every drill of a lesson topic. `errors` collects bad
 * `after:` overrides. Interview drills have no module, so they get no step.
 */
export function assignDrillSteps(modules, drills, errors) {
  // Where each feature first appears in the whole course. A feature counts for a drill only
  // if its module is where the course introduces it: printf in a pointers drill was taught
  // long before, even if the pointers module first shows it in its last step.
  const introduced = new Map(); // feature -> { module, step }
  const byModule = new Map();
  for (const m of modules) {
    const first = new Map();
    const stepWords = m.steps.map((s) => words(s.title + " " + (s.text ?? "")));
    const df = new Map();
    for (const ws of stepWords) for (const w of ws) df.set(w, (df.get(w) ?? 0) + 1);
    m.steps.forEach((s, i) => {
      const f = features([codeOf(s.text ?? ""), s.seed ?? "", s.solution ?? ""].join("\n"));
      for (const t of f) if (!introduced.has(t)) introduced.set(t, { module: m.id, step: i });
    });
    for (const [t, where] of introduced) if (where.module === m.id) first.set(t, where.step);
    byModule.set(m.id, { m, first, stepWords, df });
  }
  for (const d of drills) {
    const mod = byModule.get(d.topic);
    if (!mod) continue;
    if (d.after) {
      if (!mod.m.steps.some((s) => s.id === d.after)) errors.push(`drill ${d.id}: after: ${d.after} is not a step of ${d.topic}`);
      d.step = d.after;
      delete d.after;
      continue;
    }
    // Code: the latest step that introduces a feature the drill uses. A fill drill's blank counts as filled in.
    const shown = d.type === "fill" ? fillFirstBlank(d.display ?? "", d.answer) : d.display ?? "";
    const code = [shown, d.fix ?? "", d.exercise?.solution ?? "", codeOf(d.prompt ?? ""), ...(d.choices ?? []).map(codeOf)].join("\n");
    let at = -1;
    for (const t of features(code)) if (mod.first.has(t)) at = Math.max(at, mod.first.get(t));
    if (at < 0) {
      // No code features: a question in words. Pick the step whose text shares the most
      // informative words with the question, its choices and its explanation.
      const n = mod.m.steps.length;
      const dw = words([d.prompt ?? "", ...(d.choices ?? []), d.why ?? ""].join(" "));
      let best = 0;
      let bestScore = 0;
      mod.stepWords.forEach((ws, i) => {
        let score = 0;
        for (const w of dw) if (ws.has(w)) score += Math.log(n / mod.df.get(w));
        if (score > bestScore + 1e-9) (best = i), (bestScore = score);
      });
      // A question with no code has nothing better to go on. A drill with code whose features
      // all come from earlier modules (pointer syntax, printf) moves only on a clear match:
      // weaker scores come from words shared by chance, so it stays with the first step.
      const hasCode = !!(d.display ?? "").trim();
      at = !hasCode || bestScore >= 4 ? best : 0;
    }
    d.step = mod.m.steps[at].id;
  }
}
