// CodeMirror colors for both themes, matching the site's own syntax colors (components/highlight.tsx
// and the --tk-* tokens in styles.css). Every token keeps at least 4.5:1 contrast, including on the
// active line.
import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

type Palette = { kw: string; type: string; str: string; num: string; com: string; macro: string; fn: string; esc: string; bad: string; life: string };

function highlight(p: Palette) {
  return HighlightStyle.define([
    { tag: [t.keyword, t.controlKeyword, t.modifier, t.operatorKeyword, t.definitionKeyword, t.moduleKeyword, t.bool, t.null, t.self], color: p.kw },
    { tag: [t.typeName, t.standard(t.typeName), t.className, t.namespace], color: p.type },
    { tag: [t.string, t.character, t.special(t.string)], color: p.str },
    { tag: [t.number, t.integer, t.float], color: p.num },
    { tag: [t.comment, t.lineComment, t.blockComment, t.docComment], color: p.com, fontStyle: "italic" },
    { tag: [t.processingInstruction, t.meta, t.macroName, t.attributeName], color: p.macro },
    { tag: [t.function(t.variableName), t.function(t.propertyName)], color: p.fn },
    { tag: [t.labelName, t.special(t.variableName)], color: p.life },
    { tag: [t.escape, t.regexp], color: p.esc },
    { tag: t.invalid, color: p.bad },
  ]);
}

export const lightEditorTheme = [
  EditorView.theme(
    {
      "&": { backgroundColor: "#fdf8f4", color: "#241811" },
      ".cm-content": { caretColor: "#241811" },
      ".cm-gutters": { backgroundColor: "#fdf8f4", color: "#6b5d53", borderRight: "1px solid #ecdfd4" },
      ".cm-activeLine": { backgroundColor: "#f7ece3" },
      ".cm-activeLineGutter": { backgroundColor: "#f2e4d8", color: "#241811" },
      "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": { backgroundColor: "#fcd9bd" },
      ".cm-matchingBracket": { backgroundColor: "#f6e2d2", outline: "1px solid #c9b3a3" },
    },
    { dark: false },
  ),
  syntaxHighlighting(highlight({ kw: "#b4320b", type: "#805300", str: "#2b6e1f", num: "#9a4a00", com: "#6b5d53", macro: "#0a6590", fn: "#2448a8", esc: "#8a3a00", bad: "#b8192c", life: "#7a2fae" })),
];

export const darkEditorTheme = [
  EditorView.theme(
    {
      "&": { backgroundColor: "#120d0a", color: "#ece3dc" },
      ".cm-content": { caretColor: "#FF9A5C" },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: "#FF9A5C" },
      ".cm-gutters": { backgroundColor: "#120d0a", color: "#9a8c82", borderRight: "1px solid #322620" },
      ".cm-activeLine": { backgroundColor: "#1c1511" },
      ".cm-activeLineGutter": { backgroundColor: "#241b16", color: "#ece3dc" },
      "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": { backgroundColor: "#4a2a17" },
      ".cm-matchingBracket": { backgroundColor: "#3a2a20", outline: "1px solid #6b4a36" },
      ".cm-tooltip": { backgroundColor: "#1f1814", border: "1px solid #4a3a31", color: "#ece3dc" },
      ".cm-tooltip-autocomplete ul li[aria-selected]": { backgroundColor: "#4a2a17", color: "#fbf6f2" },
    },
    { dark: true },
  ),
  syntaxHighlighting(highlight({ kw: "#ff8f6b", type: "#ffcb6b", str: "#c3e88d", num: "#f9a11b", com: "#9a8c82", macro: "#89ddff", fn: "#82aaff", esc: "#f78c6c", bad: "#ff6b6b", life: "#d7a8ff" })),
];
