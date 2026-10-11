// Turns rustc's diagnostics into structured messages with beginner-friendly explanations.
// rustc's own messages are already good, so we keep them (with their code snippet) and add a
// sentence in plain English for the errors learners meet most.

export type Diagnostic = {
  line: number;
  col: number;
  severity: "error" | "warning" | "note";
  /** The error code, like E0382, when rustc gives one. */
  code?: string;
  message: string;
  /** rustc's full rendering of this diagnostic: the snippet with arrows and help. */
  rendered: string;
  friendly?: string;
  /** The problem is reported in code the site added (the hidden tests), not the learner's. */
  inTests?: boolean;
};

const BY_CODE: Record<string, string> = {
  E0382: "A value was used after it **moved** somewhere else. Assigning a `String`, `Vec` or other heap-owning value to another variable, or passing it to a function, hands over ownership. Borrow it with `&` instead, or `.clone()` it if you really need two copies.",
  E0499: "Two **mutable borrows** (`&mut`) of the same value are alive at once. Rust allows only one `&mut` at a time; finish using the first one before taking the second.",
  E0502: "The value is borrowed **mutably and immutably at the same time**. While a `&` reference is still in use, you can't change the value (or take `&mut`). Use the shared reference first, or copy what you need out of it.",
  E0506: "You're assigning to something that is still **borrowed**. Finish using the reference before changing the value.",
  E0505: "The value is **moved** while a reference to it is still in use. Use the reference first, or clone.",
  E0507: "You're trying to **move a value out of a reference** (or out of a collection). Borrow it (`&x`), clone it, or take it with a method like `std::mem::take`, `.remove()` or `Option::take()`.",
  E0384: "A variable is assigned twice, but it isn't `mut`. Variables are immutable by default: declare it with `let mut` if it has to change.",
  E0596: "You need a **mutable** borrow, but the variable isn't declared `mut` (or you only have a `&` reference). Use `let mut`, and `&mut` for the reference.",
  E0597: "A reference **outlives** the value it points to: the value is dropped at the end of its scope while something still refers to it. Keep the value alive longer, or store an owned copy.",
  E0716: "A temporary value is dropped while still borrowed. Store it in a `let` variable first so it lives long enough.",
  E0106: "A **lifetime** is missing. When a function returns a reference, Rust needs to know which input it borrows from: add a lifetime like `<'a>` and use it on the parameter and the return type (`&'a str`).",
  E0308: "**Mismatched types**: the value here isn't the type Rust expected. Read the `expected` and `found` lines. Common causes: a stray `;` that turns the last expression into `()`, a `&str` where a `String` is wanted (`.to_string()`), or mixing number types (convert with `as`).",
  E0277: "A type is missing a **trait** this code needs. For example `{}` printing needs `Display` (use `{:?}` and `#[derive(Debug)]` instead), `==` needs `PartialEq`, and adding an `i32` to an `f64` isn't defined.",
  E0425: "This **name isn't defined** here. Check the spelling (Rust is case-sensitive), and that the variable is declared before this line and inside the same block.",
  E0433: "Rust can't find this **path**. Check the spelling, and add a `use` for it (for example `use std::collections::HashMap;`).",
  E0412: "This **type isn't defined** (or isn't imported). Check the spelling and add the `use` line it needs.",
  E0599: "There's **no method** with this name for this type. Check the spelling, whether the type has it (look it up in the docs), and whether a trait providing it needs to be `use`d or derived.",
  E0609: "This struct has **no field** with that name. Check the spelling against the struct definition.",
  E0560: "The struct has no field with that name. Check the spelling against the struct definition.",
  E0063: "A struct literal is **missing fields**. Every field needs a value (or use `..Default::default()`).",
  E0061: "The function is called with the **wrong number of arguments**. Compare the call with the function's parameter list.",
  E0004: "The `match` is **not exhaustive**: some possible values have no arm. Add arms for the missing patterns, or a `_ =>` catch-all.",
  E0023: "The pattern has the wrong number of fields for this enum variant.",
  E0026: "The struct pattern names a field the struct doesn't have.",
  E0369: "This **operator** isn't defined for this type. Numbers support `+`; to join strings use `format!` or `push_str`; for your own types implement the trait (like `Add`) or derive `PartialEq` for `==`.",
  E0614: "You used `*` (dereference) on something that isn't a reference or pointer.",
  E0381: "The variable is used **before it has a value** on every path. Give it a value when you declare it, or in every branch.",
  E0428: "The same name is **defined twice** in this scope. Rename or remove one.",
  E0432: "This **`use` path doesn't exist**. Check the module and item names.",
  E0603: "This item is **private**. Mark it `pub` where it's defined to use it from another module.",
  E0616: "This field is **private** to its module. Make it `pub`, or add a method that returns it.",
  E0624: "This method is **private** to its module. Mark it `pub`.",
  E0040: "You can't call `drop` as a method. Use `drop(value)` (the free function) instead.",
  E0373: "A closure (often one passed to `thread::spawn`) **borrows** a variable that may not live long enough. Add `move` before the closure so it takes ownership.",
  E0282: "Rust **can't infer the type** here. Add a type annotation, like `let v: Vec<i32> = ...` or `.collect::<Vec<_>>()` or `.parse::<i32>()`.",
  E0283: "Rust can't decide which type you mean. Add a type annotation.",
  E0015: "Only `const fn` calls are allowed in a `const` or `static` initializer.",
  E0133: "This operation is **unsafe**: wrap it in an `unsafe { }` block, after making sure it's sound.",
  E0046: "The `impl` is **missing trait methods**. Add every method the trait requires (those without a default body).",
  E0407: "This method isn't part of the trait you're implementing. Check its name against the trait definition.",
  E0117: "You can only implement a trait for a type if your crate owns the trait or the type (the **orphan rule**). Wrap the type in a newtype struct to work around it.",
  E0038: "This trait can't be used as `dyn Trait` (it isn't **dyn-compatible**): methods that are generic or return `Self` prevent it.",
  E0107: "The wrong number of **generic arguments** (or lifetimes) is given for this type.",
  E0053: "The method's signature doesn't match the trait's definition. Copy the signature from the trait.",
  E0080: "Evaluating this constant failed (for example an overflow or a bad index at compile time).",
  E0423: "A struct or type name is used as if it were a value. For a struct, write `Name { field: value }`; for a tuple struct, `Name(value)`.",
  E0618: "You're calling something that isn't a function.",
  E0600: "This unary operator (like `!` or `-`) isn't defined for this type.",
  E0608: "You can't index into this type with `[ ]`. For a `HashMap` use `.get(&key)`; for a string use `.chars().nth(i)` or a slice `&s[a..b]`.",
};

type Rule = { re: RegExp; say: (m: RegExpMatchArray) => string };

/** Messages without an error code (or where the code alone says too little). */
const RULES: Rule[] = [
  { re: /expected `;`|expected one of .*`;`/, say: () => "A statement is missing its semicolon `;`. Look at the end of this line, or the line just before it." },
  { re: /this file contains an unclosed delimiter|unclosed delimiter/, say: () => "A `{`, `(` or `[` was never closed. Count your brackets: every opening one needs a matching closing one." },
  { re: /unexpected closing delimiter/, say: () => "There's an extra `}`, `)` or `]`. Check that the brackets pair up." },
  { re: /cannot find macro `(\w+)`/, say: (m) => `There's no macro called \`${m[1]}!\`. Check the spelling (it's \`println!\`, \`format!\`, \`vec!\`).` },
  { re: /main function not found|`main` function not found/, say: () => "Your program has no `fn main()`. Every Rust program starts running at `main`." },
  { re: /cannot find function `main`/, say: () => "Your program has no `fn main()`. Every Rust program starts running at `main`." },
  { re: /expected (?:item|expression), found/, say: () => "Rust found something it didn't expect here. Look for a stray symbol, a missing operator, or code outside a function." },
  { re: /unterminated double quote string/, say: () => "A string is missing its closing `\"` quote." },
  { re: /character literal may only contain one codepoint/, say: () => "Single quotes are for one `char` like `'a'`. Use double quotes for text: `\"hello\"`." },
  { re: /format argument must be a string literal/, say: () => "`println!` and `format!` need a string literal first, like `println!(\"{}\", x)`." },
  { re: /(\d+) positional arguments? in format string, but there (?:is|are) (\d+) arguments?/, say: (m) => `The format string has ${m[1]} \`{}\` placeholder(s) but you passed ${m[2]} value(s). They have to match.` },
  { re: /argument never used/, say: () => "You passed a value to `println!`/`format!` that the format string never uses. Add a `{}` for it." },
  { re: /unused variable: `(.+?)`/, say: (m) => `\`${m[1]}\` is never used. Not an error, just a sign you may have forgotten something. Prefix it with \`_\` if it's intentional.` },
  { re: /variable does not need to be mutable/, say: () => "This variable is `mut` but never changes. Remove `mut` (Rust prefers immutable variables)." },
  { re: /unused import/, say: () => "This `use` isn't needed. Not an error; you can delete it." },
  { re: /value assigned to `(.+?)` is never read/, say: (m) => `The value stored in \`${m[1]}\` is overwritten before it's ever used.` },
  { re: /unused `Result` that must be used/, say: () => "This returns a `Result` you ignored. Handle it with `?`, `match`, `.unwrap()`/`.expect()`, or `let _ =` if you really mean to ignore it." },
  { re: /function `(.+?)` is never used/, say: (m) => `\`${m[1]}\` is defined but never called. Not an error.` },
  { re: /mismatched types/, say: () => BY_CODE.E0308 },
  { re: /literal out of range for `(\w+)`/, say: (m) => `That number doesn't fit in a \`${m[1]}\`. Use a bigger type (like \`i64\` or \`u64\`).` },
  { re: /this arithmetic operation will overflow/, say: () => "This calculation overflows its integer type, so Rust rejects it. Use a bigger type, or `checked_`/`wrapping_` arithmetic." },
];

export function explain(message: string, code?: string): string | undefined {
  if (code && BY_CODE[code]) return BY_CODE[code];
  for (const r of RULES) {
    const m = message.match(r.re);
    if (m) return r.say(m);
  }
  return undefined;
}

/** Link to the official explanation of an error code. */
export function errorCodeUrl(code: string) {
  return `https://doc.rust-lang.org/error_codes/${code}.html`;
}

/**
 * Parse rustc's human-readable output. Each diagnostic starts with `error[E0382]: ...` or
 * `warning: ...` at the start of a line; its location is the ` --> src/main.rs:4:20` line under it.
 * `userLines`: how many lines the learner wrote (anything after is the site's hidden code).
 */
export function parseDiagnostics(raw: string, userLines: number): Diagnostic[] {
  const out: Diagnostic[] = [];
  const lines = raw.split("\n");
  let cur: { d: Diagnostic; body: string[] } | null = null;
  const flush = () => {
    if (!cur) return;
    cur.d.rendered = cur.body.join("\n").replace(/\n+$/, "");
    out.push(cur.d);
    cur = null;
  };
  for (const line of lines) {
    const head = line.match(/^(error|warning)(?:\[(E\d{4})\])?: (.*)$/);
    if (head) {
      flush();
      const msg = head[3];
      // Summary lines, not diagnostics.
      if (/^aborting due to|^\d+ warnings? emitted|^could not compile|^build failed/.test(msg) || /generated \d+ warnings?/.test(msg)) continue;
      cur = { d: { line: 0, col: 0, severity: head[1] as "error" | "warning", code: head[2], message: msg, rendered: "", friendly: explain(msg, head[2]) }, body: [line] };
      continue;
    }
    if (/^For more information about (this|an) error/.test(line) || /^Some errors have detailed explanations/.test(line)) {
      flush();
      continue;
    }
    if (!cur) continue;
    cur.body.push(line);
    const loc = line.match(/^\s*--> [^:]*?main\.rs:(\d+):(\d+)/);
    if (loc && !cur.d.line) {
      cur.d.line = +loc[1];
      cur.d.col = +loc[2];
      cur.d.inTests = +loc[1] > userLines;
    }
  }
  flush();
  return out;
}
