import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";

// Cloudflare Web Analytics: privacy-friendly visit counts with no cookies. Only
// added when CF_BEACON_TOKEN is set (the deploy workflow reads it from a
// repository variable), so local builds and forks send nothing.
function cloudflareAnalytics(): Plugin {
  const token = process.env.CF_BEACON_TOKEN?.trim();
  return {
    name: "cloudflare-analytics",
    transformIndexHtml(html) {
      if (!token) return html;
      if (!/^[0-9a-f]{32}$/i.test(token)) throw new Error("CF_BEACON_TOKEN should be the 32-character token from Cloudflare Web Analytics");
      const tag = `<script defer src="https://static.cloudflareinsights.com/beacon.min.js" data-cf-beacon='{"token":"${token}","spa":true}'></script>`;
      return html.replace("</head>", `  ${tag}\n  </head>`);
    },
  };
}

// Fetch the body font (src/styles.css) with the page instead of when text first needs it, so
// text doesn't reflow once it arrives. Every pre-rendered page gets the tag too. The code font
// is left to load on demand: preloading it too competes with the app's own code for bandwidth.
function preloadFonts(): Plugin {
  return {
    name: "preload-fonts",
    transformIndexHtml: {
      order: "post",
      handler(_html, ctx) {
        if (!ctx.bundle) return [];
        const base = process.env.BASE_PATH ?? "/";
        return Object.keys(ctx.bundle)
          .filter((f) => /^assets\/inter-.*\.woff2$/.test(f))
          .map((f) => ({ tag: "link", attrs: { rel: "preload", as: "font", type: "font/woff2", href: base + f, crossorigin: "" }, injectTo: "head" as const }));
      },
    },
  };
}

// Pages live at real paths (/learn/hello/your-first-program) so search engines can index each
// one, which needs an absolute base. The site is served from the root of its own
// domain; CI sets BASE_PATH to /<repo>/ when publishing to a github.io project page.
export default defineConfig({
  base: process.env.BASE_PATH ?? "/",
  plugins: [react(), cloudflareAnalytics(), preloadFonts()],
  // The site's public address (the deploy workflow sets SITE_URL), used in report links and certificates.
  define: { "import.meta.env.VITE_SITE_URL": JSON.stringify(process.env.SITE_URL ?? "") },
  build: {
    target: "es2022",
    chunkSizeWarningLimit: 1500,
  },
});
