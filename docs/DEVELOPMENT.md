# Development

Running RustArena on your own computer, the npm scripts, and how the site is deployed. For how the site works, see [Architecture](ARCHITECTURE.md).

Paths written as code (such as `scripts/local-runner.mjs`) are relative to the root of the repository.

## Run locally

Needs Node 22 and Rust (stable, via [rustup](https://rustup.rs)).

```bash
npm install
npm run content    # compile and run every lesson, drill and project; writes src/generated/
npm run dev        # http://localhost:5173/
npm run runner     # optional: run code on this machine instead of the Playground (then pick it in Settings)
```

| Script | What it does |
|---|---|
| `npm run content` | Builds `src/generated/content.json` from `content/**/*.yaml`, compiling and running everything with your Rust toolchain. `node scripts/build-content.mjs --only <path part>` checks a few files without writing anything. |
| `npm run runner` | A local stand-in for the Rust Playground on `127.0.0.1:8787` |
| `npm run build` | Type-checks and builds `dist/`, packs the Pro Track starter, and pre-renders every page |
| `npm run test:e2e` | Serves `dist/` and drives the real UI in headless Chromium (code runs on the local runner), including axe accessibility scans (WCAG 2.2 AA) of every page type in both themes, phone widths, offline use and SEO checks |
| `npm run check:pro` | Checks every Pro Track grader: the starter must fail and the reference solution must pass |

## Deploy to GitHub Pages

The workflow in `.github/workflows/deploy.yml` runs on every push to `main`. It installs dependencies and stable Rust, compiles and runs every lesson solution, worked example, drill and project milestone with the real Rust compiler (the build fails if any expected output is wrong), builds the site, runs the browser tests (including the accessibility checks, with code run by the local runner), and publishes the result to the `gh-pages` branch. Under **Settings > Pages**, set **Source** to **Deploy from a branch**, **gh-pages**, **/ (root)**. The first publish usually sets this automatically.

The site is published at `https://rustarena.app`. Without a custom domain it would be at `https://<you>.github.io/<repo>/`. To set up a domain (bought and managed on Cloudflare), follow [docs/CUSTOM-DOMAIN.md](CUSTOM-DOMAIN.md). This site uses `rustarena.app`, set as `CUSTOM_DOMAIN` at the top of the workflow.
