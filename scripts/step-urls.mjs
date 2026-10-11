// Step addresses: /learn/<module>/<slug>. Slugs come from step titles but are pinned in
// content/step-urls.yaml, so editing a title or inserting a step never moves a lesson's
// address (shared links and search results keep working).
import fs from "node:fs";
import YAML from "yaml";

export const STEP_URLS_FILE = "content/step-urls.yaml";

const HEADER = `# Lesson step addresses, /learn/<module>/<slug>. Generated and updated by
# \`npm run content\`; commit it. A new step gets an address from its title the first time
# the content is built. After that it never changes, even if the title does.
`;

export function slugify(title) {
  return (
    title
      .toLowerCase()
      .replace(/&/g, " and ")
      .replace(/['’`]/g, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "step"
  );
}

export function readStepUrls(root) {
  const file = `${root}/${STEP_URLS_FILE}`;
  if (!fs.existsSync(file)) return { slugs: {} };
  const data = YAML.parse(fs.readFileSync(file, "utf8")) ?? {};
  return { slugs: data.slugs ?? {} };
}

export function writeStepUrls(root, urls) {
  const doc = new YAML.Document({ slugs: urls.slugs });
  fs.writeFileSync(`${root}/${STEP_URLS_FILE}`, HEADER + doc.toString({ lineWidth: 0 }));
}

/** Gives every step its `slug`; new steps get pinned ones. Returns true if the file needs writing. */
export function applyStepUrls(modules, urls, errors, warnings, { partial = false } = {}) {
  let changed = false;
  const known = new Set();
  for (const m of modules) {
    const used = new Set();
    const pinned = new Set(m.steps.map((s) => urls.slugs[s.id]).filter(Boolean));
    for (const s of m.steps) {
      known.add(s.id);
      let slug = urls.slugs[s.id];
      if (!slug) {
        const base = slugify(s.title);
        slug = base;
        for (let k = 2; used.has(slug) || pinned.has(slug); k++) slug = `${base}-${k}`;
        urls.slugs[s.id] = slug;
        changed = true;
      }
      if (used.has(slug)) errors.push(`${m.id}: two steps have the address ${slug}`);
      used.add(slug);
      s.slug = slug;
    }
  }
  if (!partial) for (const id of Object.keys(urls.slugs)) if (!known.has(id)) warnings.push(`${STEP_URLS_FILE}: step ${id} no longer exists; its address now shows "not found"`);
  return changed;
}
