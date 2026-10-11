// Small regex-based Rust highlighter for read-only snippets (fill-in steps, drills, lesson code).
import type { ReactNode } from "react";

const KEYWORDS = new Set(
  (
    "as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while yield"
  ).split(" "),
);
const TYPES = new Set(
  (
    "i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64 bool char str String Vec Option Result Box Rc Arc RefCell Cell Mutex RwLock HashMap HashSet BTreeMap BTreeSet VecDeque BinaryHeap Some None Ok Err"
  ).split(" "),
);

// comment | string (raw, byte, normal) | char | lifetime | attribute | number | macro | word
const TOKEN =
  /(\/\/[^\n]*|\/\*[\s\S]*?\*\/)|(b?r(#*)"[\s\S]*?"\3|b?"(?:\\[\s\S]|[^"\\])*")|(b?'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])')|('[A-Za-z_]\w*)|(#!?\[[^\]\n]*\])|(\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?(?:[iu](?:8|16|32|64|128|size)|f32|f64)?\b|\b0x[0-9a-fA-F_]+\b|\b0b[01_]+\b)|([A-Za-z_]\w*!)|([A-Za-z_]\w*)/g;

function classOf(m: RegExpMatchArray, code: string): string {
  if (m[1]) return "tk-com";
  if (m[2]) return "tk-str";
  if (m[4]) return "tk-str";
  if (m[5]) return "tk-pre";
  if (m[6]) return "tk-pre";
  if (m[7]) return "tk-num";
  if (m[8]) return "tk-fn";
  const text = m[9];
  if (!text) return "";
  if (KEYWORDS.has(text)) return "tk-kw";
  if (TYPES.has(text) || /^[A-Z][A-Za-z0-9]*$/.test(text)) return "tk-type";
  if (/^\s*(\(|::<)/.test(code.slice(m.index! + text.length))) return "tk-fn";
  return "";
}

export function highlight(code: string, keyPrefix = ""): ReactNode[] {
  const out: ReactNode[] = [];
  let last = 0;
  let k = 0;
  for (const m of code.matchAll(TOKEN)) {
    if (m.index! > last) out.push(code.slice(last, m.index));
    const text = m[0];
    const cls = classOf(m, code);
    out.push(cls ? <span key={keyPrefix + k++} className={cls}>{text}</span> : text);
    last = m.index! + text.length;
  }
  if (last < code.length) out.push(code.slice(last));
  return out;
}

export function CodeView({ code, className = "" }: { code: string; className?: string }) {
  return <pre tabIndex={0} className={"codeview " + className}>{highlight(code)}</pre>;
}

const esc = (t: string) => t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/** The same highlighting as an HTML string, for code blocks inside lesson Markdown. */
export function highlightHtml(code: string): string {
  let out = "";
  let last = 0;
  for (const m of code.matchAll(TOKEN)) {
    if (m.index! > last) out += esc(code.slice(last, m.index));
    const text = m[0];
    const cls = classOf(m, code);
    out += cls ? `<span class="${cls}">${esc(text)}</span>` : esc(text);
    last = m.index! + text.length;
  }
  return out + esc(code.slice(last));
}
