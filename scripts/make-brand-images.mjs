// Draws the brand images from one logo: public/favicon.svg, the app icons (192, 512 and a
// maskable 512 for Android home screens) and public/og-image.png, the 1200 x 630 picture
// shown when a link is shared. The share image names no counts, so it never goes stale.
// The logo matches src/components/Brand.tsx.
// Usage: node scripts/make-brand-images.mjs   (needs Playwright's Chromium, as for the e2e tests)
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { chromium } from "playwright";

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
// The crab: our own drawing (the official Rust logo is a Rust Foundation trademark). Matches src/components/Brand.tsx.
const GLYPH = `<path d="M5.2 15.4c-1.7-1.3-1.9-4-.4-5.6 1-1 2.4-1.2 3.6-.7l-1.3 2.3 2.4.4c.3 1.6-.4 3.2-1.8 3.9-.8.4-1.7.3-2.5-.3z" fill="#fff"/><path d="M26.8 15.4c1.7-1.3 1.9-4 .4-5.6-1-1-2.4-1.2-3.6-.7l1.3 2.3-2.4.4c-.3 1.6.4 3.2 1.8 3.9.8.4 1.7.3 2.5-.3z" fill="#fff"/><path d="M8.6 15.2l2.4 2.2M23.4 15.2l-2.4 2.2" stroke="#fff" stroke-width="1.6" stroke-linecap="round"/><path d="M10.2 22.2l-2 2.6M12.6 23.2l-1.2 2.8M21.8 22.2l2 2.6M19.4 23.2l1.2 2.8" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/><path d="M8.6 20.4c0-3.9 3.3-7 7.4-7s7.4 3.1 7.4 7c0 1.9-3.3 3.3-7.4 3.3s-7.4-1.4-7.4-3.3zM9.6 16.6l.4-2.2 1.6 1M11.4 14.6l.9-2.1 1.3 1.4M14 13.4l1-1.9 1 1.9M17 13.4l1.3-1.4.9 2.1M19.4 14.6l1.6-1 .4 2.2" fill="#fff" stroke="#fff" stroke-width="1" stroke-linejoin="round"/><circle cx="13.3" cy="18.6" r="1.35" fill="#2b1206"/><circle cx="18.7" cy="18.6" r="1.35" fill="#2b1206"/><circle cx="13.7" cy="18.1" r="0.45" fill="#fff"/><circle cx="19.1" cy="18.1" r="0.45" fill="#fff"/>`;
const GRADIENT = `<linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#F98A1F"/><stop offset="0.5" stop-color="#E5531A"/><stop offset="1" stop-color="#B7410E"/></linearGradient>`;
/** The rounded tile, as in the site header. */
const MARK = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><defs>${GRADIENT}</defs><rect width="32" height="32" rx="8" fill="url(#g)"/><rect x="0.5" y="0.5" width="31" height="31" rx="7.5" fill="none" stroke="#fff" stroke-opacity="0.22"/>${GLYPH}</svg>`;
/** Full-bleed square with the glyph inside the maskable safe zone (the middle 80%). */
const MASKABLE = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><defs>${GRADIENT}</defs><rect width="32" height="32" fill="url(#g)"/><g transform="translate(16 16) scale(0.74) translate(-16 -16)">${GLYPH}</g></svg>`;

fs.writeFileSync(path.join(ROOT, "public/favicon.svg"), MARK + "\n");

const font = (f) => "file://" + path.join(ROOT, "node_modules", f);
const og = `<!doctype html><html><head><meta charset="utf-8"><style>
  @font-face { font-family: Inter; font-weight: 100 900; src: url("${font("@fontsource-variable/inter/files/inter-latin-wght-normal.woff2")}"); }
  @font-face { font-family: Mono; font-weight: 100 800; src: url("${font("@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2")}"); }
  * { box-sizing: border-box; }
  body { margin: 0; width: 1200px; height: 630px; overflow: hidden; background: #0f0b09; color: #f7f1ec; font-family: Inter, sans-serif; position: relative; }
  .bg { position: absolute; inset: 0; background:
    radial-gradient(55% 60% at 85% 35%, rgb(247 76 0 / 0.22), transparent 70%),
    linear-gradient(rgb(255 255 255 / 0.04) 1px, transparent 1px) 0 0 / 48px 48px,
    linear-gradient(90deg, rgb(255 255 255 / 0.04) 1px, transparent 1px) 0 0 / 48px 48px; }
  .wrap { position: relative; display: flex; height: 100%; padding: 70px 80px; gap: 56px; align-items: center; }
  .left { flex: 1.15; }
  .brand { display: flex; align-items: center; gap: 18px; font-size: 34px; font-weight: 700; letter-spacing: -0.02em; margin-bottom: 48px; }
  .brand svg { width: 60px; height: 60px; }
  h1 { font-size: 68px; line-height: 1.02; letter-spacing: -0.045em; font-weight: 750; margin: 0 0 26px; }
  h1 span { background: linear-gradient(95deg, #FBBF4A, #FB8B2E 40%, #F74C00); -webkit-background-clip: text; color: transparent; }
  p { font-size: 26px; color: #b3a296; margin: 0; line-height: 1.4; }
  .card { flex: 1; background: #18120f; border: 1px solid #4a3a31; position: relative; border-radius: 22px; overflow: hidden; box-shadow: 0 40px 80px -30px #000; }
  .bar { display: flex; gap: 9px; padding: 16px 20px; background: #1f1814; border-bottom: 1px solid #322620; }
  .bar i { width: 13px; height: 13px; border-radius: 50%; background: #f87171; } .bar i:nth-child(2) { background: #fbbf24; } .bar i:nth-child(3) { background: #4ade80; }
  pre { margin: 0; padding: 22px 24px; font-family: Mono, monospace; font-size: 19px; line-height: 1.7; color: #ece3dc; }
  .k { color: #ff8f6b; } .f { color: #82aaff; } .s { color: #c3e88d; } .n { color: #f9a11b; } .p { color: #89ddff; } .t { color: #ffcb6b; }
  .ferris { position: absolute; right: -30px; bottom: -40px; width: 190px; transform: rotate(-8deg); filter: drop-shadow(0 12px 18px rgb(0 0 0 / 0.5)); }
  .ok { margin: 0 24px 22px; display: inline-flex; gap: 10px; align-items: center; font-size: 19px; font-weight: 600; color: #86efac; background: #0b1f14; border: 1px solid #1f5134; border-radius: 999px; padding: 7px 16px; }
</style></head><body><div class="bg"></div><div class="wrap">
  <div class="left">
    <div class="brand">${MARK}<span>RustArena</span></div>
    <h1>Learn Rust by <span>writing real code</span>.</h1>
    <p>From your first println! to job-ready Rust. Lessons checked by the real compiler, drills, projects and a Pro Track.</p>
  </div>
  <div class="card"><div class="bar"><i></i><i></i><i></i></div>
<pre><span class="k">fn</span> <span class="f">main</span>() {
    <span class="k">let</span> crabs = <span class="t">vec!</span>[<span class="s">"Ferris"</span>, <span class="s">"You"</span>];
    <span class="k">for</span> name <span class="k">in</span> &amp;crabs {
        <span class="f">println!</span>(<span class="s">"Hello, {name}!"</span>);
    }
}</pre>
    <div class="ok">✓ All tests passed</div>
    <img class="ferris" src="file://${path.join(ROOT, "public/ferris.png")}" alt="">
  </div>
</div></body></html>`;

const exe = process.env.CHROMIUM_PATH || (fs.existsSync("/opt/pw-browsers/chromium") ? "/opt/pw-browsers/chromium" : undefined);
const browser = await chromium.launch(exe ? { executablePath: exe } : { channel: "chromium" });
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "brand-"));
async function render(html, width, height, out, transparent = false) {
  const file = path.join(tmp, path.basename(out) + ".html");
  fs.writeFileSync(file, html);
  const page = await browser.newPage({ viewport: { width, height } });
  await page.goto("file://" + file);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: path.join(ROOT, out), omitBackground: transparent });
  await page.close();
  console.log("wrote " + out);
}
await new Promise((r) => setTimeout(r, 0));
const iconPage = (svg, size) => `<!doctype html><html><body style="margin:0;background:transparent">${svg.replace("<svg ", `<svg width="${size}" height="${size}" `)}</body></html>`;
await render(og, 1200, 630, "public/og-image.png");
await render(iconPage(MARK, 192), 192, 192, "public/icon-192.png", true);
await render(iconPage(MARK, 512), 512, 512, "public/icon-512.png", true);
await render(iconPage(MASKABLE, 512), 512, 512, "public/icon-maskable-512.png");
await browser.close();
fs.rmSync(tmp, { recursive: true });
