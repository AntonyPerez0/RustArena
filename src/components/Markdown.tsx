import { useMemo } from "react";
import { marked } from "marked";
import { highlightHtml } from "./highlight";

marked.setOptions({ gfm: true, breaks: false });
// Rust code blocks get the same colors as the rest of the site. ```rust-error marks an example that
// doesn't compile (on purpose), and ```compiler holds rustc's message about it.
marked.use({
  renderer: {
    code({ text, lang }) {
      if (lang === "rust-error") return `<div class="code-fail"><span class="code-fail-tag">does not compile</span><pre><code class="language-rust">${highlightHtml(text)}</code></pre></div>\n`;
      if (lang === "compiler") return `<pre class="compiler-msg"><code>${text.replace(/&/g, "&amp;").replace(/</g, "&lt;")}</code></pre>\n`;
      if (lang !== "rust" && lang !== "rs") return false;
      return `<pre><code class="language-rust">${highlightHtml(text)}</code></pre>\n`;
    },
  },
});

// Moves every heading so the shallowest one becomes <h{top}>, keeping the page outline in order.
function shiftHeadings(html: string, top: number): string {
  const levels = [...html.matchAll(/<h([1-6])[\s>]/g)].map((m) => Number(m[1]));
  if (levels.length === 0) return html;
  const shift = top - Math.min(...levels);
  if (shift === 0) return html;
  return html.replace(/<(\/?)h([1-6])([\s>])/g, (_, slash, n, after) => `<${slash}h${Math.min(6, Math.max(1, Number(n) + shift))}${after}`);
}

/** Renders trusted lesson markdown (content is authored in this repo). `top` sets the level of the text's first heading. */
export default function Markdown({ text, className = "", top }: { text: string; className?: string; top?: number }) {
  // Code blocks and tables can scroll sideways (tables do on phones), so they must be reachable with the keyboard.
  const html = useMemo(() => {
    const out = (marked.parse(text, { async: false }) as string).replace(/<(pre|table)(\s[^>]*)?>/g, '<$1$2 tabindex="0">');
    return top ? shiftHeadings(out, top) : out;
  }, [text, top]);
  return <div className={"md " + className} dangerouslySetInnerHTML={{ __html: html }} />;
}

export function InlineMd({ text }: { text: string }) {
  const html = useMemo(() => marked.parseInline(text, { async: false }) as string, [text]);
  return <span dangerouslySetInnerHTML={{ __html: html }} />;
}
