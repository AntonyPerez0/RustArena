// Writes a real HTML page for every route into dist/ (after `vite build`), so
// search engines and link previews see each lesson with its own title,
// description and text. The React app replaces the pre-rendered content when
// it starts. Also writes sitemap.xml, robots.txt and 404.html.
//
// Env: SITE_URL (default https://antonyperez0.github.io/RustArena), BASE_PATH (default /).
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { marked } from "marked";
import { loadContent } from "./load-content.mjs";

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const DIST = path.join(ROOT, "dist");
// Host names are case-insensitive, but canonical links should match exactly what
// GitHub Pages serves (a lowercase host), even when the owner name has capitals.
const siteUrl = new URL(process.env.SITE_URL ?? "https://antonyperez0.github.io/RustArena");
const SITE = `${siteUrl.protocol}//${siteUrl.host.toLowerCase()}${siteUrl.pathname}`.replace(/\/$/, "");
const BASE = process.env.BASE_PATH ?? "/";
const NAME = "RustArena";
// The logo, as in src/components/Brand.tsx, so the page looks the same before the app starts.
const BRAND_MARK = `<svg class="brand-mark" viewBox="0 0 32 32" aria-hidden="true"><defs><linearGradient id="bm" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#F98A1F"/><stop offset="0.5" stop-color="#E5531A"/><stop offset="1" stop-color="#B7410E"/></linearGradient></defs><rect width="32" height="32" rx="8" fill="url(#bm)"/><rect x="0.5" y="0.5" width="31" height="31" rx="7.5" fill="none" stroke="#fff" stroke-opacity="0.22"/><path d="M5.2 15.4c-1.7-1.3-1.9-4-.4-5.6 1-1 2.4-1.2 3.6-.7l-1.3 2.3 2.4.4c.3 1.6-.4 3.2-1.8 3.9-.8.4-1.7.3-2.5-.3z" fill="#fff"/><path d="M26.8 15.4c1.7-1.3 1.9-4 .4-5.6-1-1-2.4-1.2-3.6-.7l1.3 2.3-2.4.4c-.3 1.6.4 3.2 1.8 3.9.8.4 1.7.3 2.5-.3z" fill="#fff"/><path d="M8.6 15.2l2.4 2.2M23.4 15.2l-2.4 2.2" stroke="#fff" stroke-width="1.6" stroke-linecap="round"/><path d="M10.2 22.2l-2 2.6M12.6 23.2l-1.2 2.8M21.8 22.2l2 2.6M19.4 23.2l1.2 2.8" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/><path d="M8.6 20.4c0-3.9 3.3-7 7.4-7s7.4 3.1 7.4 7c0 1.9-3.3 3.3-7.4 3.3s-7.4-1.4-7.4-3.3zM9.6 16.6l.4-2.2 1.6 1M11.4 14.6l.9-2.1 1.3 1.4M14 13.4l1-1.9 1 1.9M17 13.4l1.3-1.4.9 2.1M19.4 14.6l1.6-1 .4 2.2" fill="#fff" stroke="#fff" stroke-width="1" stroke-linejoin="round"/><circle cx="13.3" cy="18.6" r="1.35" fill="#2b1206"/><circle cx="18.7" cy="18.6" r="1.35" fill="#2b1206"/></svg>`;
const content = loadContent();
const template = fs.readFileSync(path.join(DIST, "index.html"), "utf8");
const today = new Date().toISOString().slice(0, 10);

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const link = (p) => BASE + p.replace(/^\//, "");
const md = (s) => marked.parse(s, { async: false });
/** Markdown whose shallowest heading becomes <h{top}>, so the page outline has no gaps. */
function mdAt(s, top) {
  const html = md(s);
  const levels = [...html.matchAll(/<h([1-6])[\s>]/g)].map((m) => Number(m[1]));
  if (levels.length === 0) return html;
  const shift = top - Math.min(...levels);
  return html.replace(/<(\/?)h([1-6])([\s>])/g, (_, slash, n, after) => `<${slash}h${Math.min(6, Math.max(1, Number(n) + shift))}${after}`);
}
/** Plain-text summary of Markdown, cut at a word boundary. */
function summary(text, max = 155) {
  const plain = text
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/^\s*(?:[-*+]|\d+\.)\s+/gm, "")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/\*\*|__|\*|_|#+ /g, "")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\|/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  if (plain.length <= max) return plain;
  const cut = plain.slice(0, max - 1);
  return cut.slice(0, cut.lastIndexOf(" ")) + "…";
}

const stepPath = (m, s) => `/learn/${m.id}/${s.slug}`;
const firstStepPath = (id) => {
  const m = content.modules.find((x) => x.id === id);
  return m ? stepPath(m, m.steps[0]) : "/learn";
};
const moduleCount = content.modules.length;
const stepCount = content.modules.reduce((n, m) => n + m.steps.length, 0);

const pages = [];
function page(route, { title, description, body, index = true, type = "website", jsonld = null }) {
  pages.push({ route, title, description, body, index, type, jsonld });
}

// ---------------------------------------------------------------- pages
const courseLd = {
  "@context": "https://schema.org",
  "@type": "Course",
  name: "Learn Rust: from zero to job-ready",
  description: `A free interactive Rust course with ${moduleCount} modules and ${stepCount} lessons whose exercises are compiled and tested by the real Rust compiler, plus drills, projects and a professional track.`,
  url: SITE + "/",
  isAccessibleForFree: true,
  inLanguage: "en",
  educationalLevel: "Beginner to advanced",
  teaches: ["Rust programming", "Ownership and borrowing", "Traits and generics", "Error handling", "Concurrency and async with Tokio", "Testing, Cargo and professional Rust tooling", "Web services with axum"],
  provider: { "@type": "Organization", name: NAME, url: SITE + "/" },
  hasCourseInstance: { "@type": "CourseInstance", courseMode: "online", courseWorkload: "PT120H" },
  offers: { "@type": "Offer", price: 0, priceCurrency: "USD", category: "Free" },
};

page("/", {
  title: `${NAME}: learn Rust from zero to job-ready`,
  description: `Free interactive Rust course: ${stepCount} lessons checked by the real Rust compiler, from your first println! to ownership, traits, async, testing and web services, plus projects and a Pro Track.`,
  jsonld: [courseLd, { "@context": "https://schema.org", "@type": "WebSite", name: NAME, url: SITE + "/" }],
  body: `<h1>Learn Rust by writing real code</h1>
<p>Every exercise you write is compiled and tested by the real Rust compiler. Work through ${moduleCount} modules from <code>println!</code> to ownership, traits, async and web services, drill the fundamentals in endless Deathmatch reps, ship projects milestone by milestone, and finish with a professional track on a real machine.</p>
<ul>
<li><a href="${link("/learn")}">Learn: ${moduleCount} modules, ${stepCount} steps</a></li>
<li><a href="${link("/deathmatch")}">Deathmatch: ${content.drills.length} drills</a></li>
<li><a href="${link("/projects")}">Projects: ${content.projects.length} multi-milestone builds</a></li>
<li><a href="${link("/pro")}">Pro Track: ${content.pro.length} projects graded by GitHub Actions</a></li>
<li><a href="${link("/roadmap")}">The Rust roadmap, mapped to lessons</a></li>
<li><a href="${link("/topics")}">Rust topics explained</a></li>
<li><a href="${link("/daily")}">Daily challenge</a></li>
<li><a href="${link("/placement")}">Placement quiz</a></li>
</ul>`,
});

const phases = [];
for (const m of content.modules) {
  let ph = phases.find((p) => p.name === m.phase);
  if (!ph) phases.push((ph = { name: m.phase, modules: [] }));
  ph.modules.push(m);
}
page("/learn", {
  title: `Rust course: all ${moduleCount} modules | ${NAME}`,
  description: `The full Rust curriculum: ${moduleCount} modules and ${stepCount} interactive lessons, from basics through ownership, traits, error handling, concurrency, async, testing and production Rust.`,
  body:
    `<h1>Learn</h1>` +
    phases
      .map(
        (ph) =>
          `<h2>${esc(ph.name)}</h2><ul>` +
          ph.modules.map((m) => `<li><a href="${link(stepPath(m, m.steps[0]))}">${esc(m.title)}</a>: ${esc(m.summary)}</li>`).join("") +
          `</ul>`,
      )
      .join(""),
});

for (const m of content.modules) {
  m.steps.forEach((s, i) => {
    const prev = i > 0 ? `<a href="${link(stepPath(m, m.steps[i - 1]))}">Previous: ${esc(m.steps[i - 1].title)}</a>` : "";
    const next = i + 1 < m.steps.length ? `<a href="${link(stepPath(m, m.steps[i + 1]))}">Next: ${esc(m.steps[i + 1].title)}</a>` : "";
    page(stepPath(m, s), {
      title: `${s.title} · ${m.title} (Rust) | ${NAME}`,
      description: summary(s.text),
      type: "article",
      jsonld: {
        "@context": "https://schema.org",
        "@type": "LearningResource",
        name: s.title,
        description: summary(s.text),
        learningResourceType: "Exercise",
        interactivityType: "active",
        isAccessibleForFree: true,
        inLanguage: "en",
        programmingLanguage: "Rust",
        isPartOf: { "@type": "Course", name: courseLd.name, url: SITE + "/" },
        url: `${SITE}${stepPath(m, s)}/`,
      },
      body: `<nav aria-label="Breadcrumb"><a href="${link("/learn")}">Learn</a> › ${esc(m.phase)} › ${esc(m.title)}</nav>
<p>Step ${i + 1} of ${m.steps.length}</p>
<h1>${esc(s.title)}</h1>
${mdAt(s.text, 2)}
<h2>Your task</h2>
${mdAt(s.task ?? "", 3)}
${(s.more ?? []).map((c, k) => `<h3>Challenge ${k + 2}</h3>\n${mdAt(c.task, 4)}`).join("\n")}
<p>${prev} ${next}</p>`,
    });
  });
}

page("/deathmatch", {
  title: `Deathmatch: endless Rust drills | ${NAME}`,
  description: `${content.drills.length} quick drills: predict the output, fill the token, spot the bug, will it compile. One life, ranked from Silver I to The Global Elite.`,
  body: `<h1>Deathmatch</h1><p>Endless reps from everything you've unlocked: predict the output, fill the token, spot the bug, will it compile. Every 8th rep is a boss rep you compile for real.</p>`,
});

page("/projects", {
  title: `Rust projects for beginners to advanced | ${NAME}`,
  description: `${content.projects.length} Rust projects built milestone by milestone, each checked by the real Rust compiler.`,
  body: `<h1>Projects</h1><ul>` + content.projects.map((p) => `<li><a href="${link(`/projects/${p.id}`)}">${esc(p.title)}</a>: ${esc(p.summary)}</li>`).join("") + `</ul>`,
});
for (const p of content.projects) {
  page(`/projects/${p.id}`, {
    title: `${p.title}: a Rust project | ${NAME}`,
    description: summary(p.summary),
    body: `<h1>${esc(p.title)}</h1><p>${esc(p.summary)}</p><h2>Milestones</h2><ol>${p.milestones.map((ms) => `<li>${esc(ms.title)}</li>`).join("")}</ol>`,
  });
}

page("/pro", {
  title: `Pro Track: professional Rust projects | ${NAME}`,
  description: `${content.pro.length} projects on a real machine with Cargo, Git, clippy, tests, clap, Tokio, axum, SQLx, tracing and criterion, graded by GitHub Actions.`,
  body: `<h1>Pro Track</h1><ul>` + content.pro.map((p) => `<li><a href="${link(`/pro/${p.id}`)}">${esc(p.title)}</a>: ${esc(p.summary)}</li>`).join("") + `</ul>`,
});
for (const p of content.pro) {
  page(`/pro/${p.id}`, {
    title: `${p.title} | Pro Track | ${NAME}`,
    description: summary(p.summary),
    type: "article",
    body: `<h1>${esc(p.title)}</h1>${md(p.readme.replace(/^# .*\n+/, ""))}`,
  });
}

// ---------------------------------------------------------------- topics, visualizations, quizzes
const topics = content.topics ?? [];
const modById = new Map(content.modules.map((m) => [m.id, m]));
page("/topics", {
  title: `Rust topics explained (${topics.length} guides) | ${NAME}`,
  description: "Short, tested explanations of Rust: ownership, borrowing, lifetimes, traits, generics, Option and Result, iterators, smart pointers, async and more.",
  body: `<h1>Rust topics</h1><ul>` + topics.map((t) => `<li><a href="${link(`/topics/${t.slug}`)}">${esc(t.title)}</a>: ${esc(t.description)}</li>`).join("") + `</ul>`,
});
for (const t of topics) {
  const lang = "Rust";
  page(`/topics/${t.slug}`, {
    title: `${t.title} | ${NAME}`,
    description: t.description,
    type: "article",
    jsonld: {
      "@context": "https://schema.org",
      "@type": "TechArticle",
      headline: t.title,
      description: t.description,
      proficiencyLevel: "Beginner",
      programmingLanguage: lang,
      inLanguage: "en",
      isAccessibleForFree: true,
      url: `${SITE}/topics/${t.slug}/`,
      isPartOf: { "@type": "Course", name: courseLd.name, url: SITE + "/" },
    },
    body: `<nav aria-label="Breadcrumb"><a href="${link("/topics")}">Topics</a> › ${lang}</nav>
<h1>${esc(t.title)}</h1>
<p>${esc(t.description)}</p>
${md(t.body)}
<h2>Example</h2>
<pre><code>${esc(t.example)}</code></pre>
<p>Output:</p>
<pre><code>${esc(t.output)}</code></pre>
<h2>Practice it</h2><ul>${t.modules.map((m) => `<li><a href="${link(firstStepPath(m))}">Lesson: ${esc(modById.get(m)?.title ?? m)}</a></li>`).join("")}</ul>`,
  });
}

page("/daily", {
  title: `Daily Rust challenge | ${NAME}`,
  description: "One quick Rust problem a day, the same for everyone. Keep your streak going.",
  body: `<h1>Daily challenge</h1><p>One quick Rust problem a day, the same for everyone. Answer it to keep your streak going.</p>`,
});
page("/placement", {
  title: `Rust placement quiz: where should you start? | ${NAME}`,
  description: `${(content.placement ?? []).length} quick questions from variables to traits and async that show which Rust lessons you can skip.`,
  body: `<h1>Placement quiz</h1><p>Already know some Rust? Answer ${(content.placement ?? []).length} quick questions to find your starting point and skip what you know.</p>`,
});
page("/playground", {
  title: `Rust Playground: write and run Rust online | ${NAME}`,
  description: "Write, compile and run Rust programs online with the real Rust compiler. Input, command-line arguments, popular crates like serde and tokio, and share links.",
  body: `<h1>Playground</h1>
<p>Write any Rust program and run it here, compiled by the real Rust compiler (stable, edition 2024) on the Rust Playground.</p>
<ul>
<li>Give your program input and command-line arguments, and see its output, panics and compiler errors explained in plain English.</li>
<li>Popular crates are available: serde, serde_json, tokio, anyhow, thiserror, regex, itertools, rayon and chrono.</li>
<li>Your code is saved in your browser, and Share makes a link that carries it.</li>
</ul>
<p>New to Rust? <a href="${link("/learn")}">Start the course</a>.</p>`,
});
const roadmap = content.roadmap ?? [];
page("/roadmap", {
  title: `Rust roadmap: what to learn, in order | ${NAME}`,
  description: "Every topic on the community Rust developer roadmap, from basics to async and the ecosystem, mapped to interactive lessons and projects.",
  body:
    `<h1>Rust roadmap</h1><p>Every topic on the community Rust developer roadmap (roadmap.sh/rust), in the order this course teaches it.</p>` +
    roadmap
      .map(
        (sec, i) =>
          `<h2>${i + 1}. ${esc(sec.title)}</h2><p>${esc(sec.summary ?? "")}</p><ul>` +
          sec.items
            .map((it) => {
              const mods = (it.modules ?? []).map((id) => modById.get(id)).filter(Boolean);
              return `<li>${mods.length ? `<a href="${link(firstStepPath(mods[0].id))}">${esc(it.title)}</a>` : esc(it.title)}${mods.length ? ` (${mods.map((m) => esc(m.title)).join(", ")})` : ""}</li>`;
            })
            .join("") +
          `</ul>`,
      )
      .join(""),
});
page("/about", {
  title: `About RustArena | ${NAME}`,
  description: "What RustArena is, how code runs, your data, and credits: the Rust Playground, Ferris the crab (public domain) and the community Rust roadmap.",
  body: `<h1>About RustArena</h1><p>RustArena is a free, open-source course that takes you from never having programmed to writing the kind of Rust teams ship at work. RustArena is an independent learning site, not affiliated with or endorsed by the Rust Foundation or the Rust Project.</p>`,
});
page("/settings", { title: `Settings | ${NAME}`, description: "Theme, text size and where your code runs.", index: false, body: `<h1>Settings</h1>` });
page("/next", {
  title: `${content.next.title} | ${NAME}`,
  description: content.next.description,
  type: "article",
  body: `<h1>Where to go next</h1>${mdAt(content.next.body.replace(/\]\(\//g, "](" + BASE), 2)}`,
});
page("/certificate", { title: `Certificates | ${NAME}`, description: "Certificates for finishing the RustArena course and the Pro Track.", index: false, body: `<h1>Certificates</h1>` });

page("/profile", { title: `Your progress | ${NAME}`, description: "Your saved progress, ranks and settings.", index: false, body: `<h1>Profile</h1>` });

// ---------------------------------------------------------------- render
function render(p) {
  const url = p.route === "/" ? SITE + "/" : `${SITE}${p.route}/`;
  const head = [
    `<title>${esc(p.title)}</title>`,
    `<meta name="description" content="${esc(p.description)}" />`,
    p.index ? `<link rel="canonical" href="${url}" />` : `<meta name="robots" content="noindex" />`,
    `<meta property="og:site_name" content="${NAME}" />`,
    `<meta property="og:type" content="${p.type}" />`,
    `<meta property="og:title" content="${esc(p.title)}" />`,
    `<meta property="og:description" content="${esc(p.description)}" />`,
    `<meta property="og:url" content="${url}" />`,
    `<meta property="og:image" content="${SITE}/og-image.png" />`,
    `<meta property="og:image:alt" content="RustArena: learn Rust by writing real code" />`,
    `<meta name="twitter:card" content="summary_large_image" />`,
    ...(p.jsonld ? [].concat(p.jsonld).map((j) => `<script type="application/ld+json">${JSON.stringify(j).replace(/</g, "\\u003c")}</script>`) : []),
  ].join("\n    ");
  const nav = `<header class="topbar"><a class="brand" href="${BASE}" aria-label="${NAME} home">${BRAND_MARK}<span class="brand-name">Rust<b>Arena</b></span></a><nav class="nav" aria-label="Main">${[
    ["/roadmap", "Roadmap"],
    ["/learn", "Learn"],
    ["/deathmatch", "Deathmatch"],
    ["/daily", "Daily"],
    ["/projects", "Projects"],
    ["/playground", "Playground"],
    ["/pro", "Pro"],
    ["/profile", "Profile"],
  ]
    .map(([r, t]) => `<a href="${link(r)}">${t}</a>`)
    .join("")}</nav></header>`;
  return template
    .replace(/<title>[\s\S]*?<\/title>/, "")
    .replace(/<meta name="description"[^>]*>/, "")
    .replace("</head>", `  ${head}\n  </head>`)
    .replace('<div id="root"></div>', `<div id="root">${nav}<main id="main" class="main prerendered md">${p.body}</main></div>`);
}

let written = 0;
for (const p of pages) {
  const dir = p.route === "/" ? DIST : path.join(DIST, p.route);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, "index.html"), render(p));
  written++;
}

// GitHub Pages serves 404.html for unknown paths; the app then shows its "not found" page.
fs.writeFileSync(
  path.join(DIST, "404.html"),
  render({ route: "/404", title: `Page not found | ${NAME}`, description: "This page doesn't exist.", index: false, type: "website", body: `<h1>Page not found</h1><p><a href="${BASE}">Go to the home page</a></p>` }),
);

const indexed = pages.filter((p) => p.index);
fs.writeFileSync(
  path.join(DIST, "sitemap.xml"),
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n` +
    indexed.map((p) => `  <url><loc>${p.route === "/" ? SITE + "/" : `${SITE}${p.route}/`}</loc><lastmod>${today}</lastmod></url>`).join("\n") +
    `\n</urlset>\n`,
);
// Search engines only read robots.txt at the domain root, so this one only counts when
// the site has its own domain; on a github.io project page, submit the sitemap in
// Google Search Console instead.
fs.writeFileSync(path.join(DIST, "robots.txt"), `User-agent: *\nAllow: /\n\nSitemap: ${SITE}/sitemap.xml\n`);

// The service worker saves the app's built files when it installs (public/sw.js), so the
// site works offline after one visit. Its version follows the files and the home page.
const built = fs.readdirSync(path.join(DIST, "assets")).sort().map((f) => "assets/" + f);
const buildInfo = {
  version: crypto.createHash("sha256").update(built.join("\n")).update(fs.readFileSync(path.join(DIST, "index.html"))).digest("hex").slice(0, 12),
  files: built,
};
const swFile = path.join(DIST, "sw.js");
const placeholder = 'const BUILD = { version: "dev", files: [] };';
const sw = fs.readFileSync(swFile, "utf8");
if (!sw.includes(placeholder)) throw new Error("dist/sw.js has no BUILD placeholder to fill in");
fs.writeFileSync(swFile, sw.replace(placeholder, `const BUILD = ${JSON.stringify(buildInfo)};`));
console.log(`prerender: ${written} pages, ${indexed.length} in sitemap.xml, 404.html, robots.txt, ${built.length} files for offline use`);
