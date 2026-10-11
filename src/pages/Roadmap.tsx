import { Link } from "react-router-dom";
import { Check, CircleDashed, ExternalLink, Briefcase } from "lucide-react";
import { moduleById, proById, roadmap, stepPath, firstStepPath } from "../content";
import { useStore, type State } from "../state/store";
import { useTitle } from "../lib/title";

/** How much of a set of modules is done: steps finished / steps total. */
function progressOf(s: State, ids: string[]) {
  let done = 0;
  let total = 0;
  for (const id of ids) {
    const m = moduleById.get(id);
    if (!m) continue;
    total += m.steps.length;
    done += m.steps.filter((st) => s.steps[st.id]?.done).length;
  }
  return { done, total };
}

/** Where a roadmap topic's link goes: the first unfinished step of its modules, or their start. */
function targetOf(s: State, ids: string[]): string {
  for (const id of ids) {
    const m = moduleById.get(id);
    const st = m?.steps.find((x) => !s.steps[x.id]?.done);
    if (m && st) return stepPath(m, st);
  }
  return ids[0] ? firstStepPath(ids[0]) : "/learn";
}

function Status({ done, total }: { done: number; total: number }) {
  if (total === 0) return null;
  if (done >= total)
    return (
      <span className="rm-status rm-done" title="Done">
        <Check className="icon" aria-hidden="true" />
        <span className="visually-hidden">(done)</span>
      </span>
    );
  if (done > 0)
    return (
      <span className="rm-status rm-part" title={`${done} of ${total} lessons done`}>
        <span className="rm-pie" style={{ ["--p" as string]: `${Math.round((done / total) * 100)}%` }} aria-hidden="true" />
        <span className="visually-hidden">
          ({done} of {total} lessons done)
        </span>
      </span>
    );
  return (
    <span className="rm-status rm-todo" title="Not started">
      <CircleDashed className="icon" aria-hidden="true" />
      <span className="visually-hidden">(not started)</span>
    </span>
  );
}

export default function Roadmap() {
  useTitle("Rust roadmap: what to learn, in order");
  const s = useStore((x) => x);
  const all = [...new Set(roadmap.flatMap((sec) => sec.items.flatMap((it) => it.modules ?? [])))];
  const overall = progressOf(s, all);
  return (
    <div className="roadmap">
      <div className="page-head">
        <h1>Rust roadmap</h1>
        <p className="muted">
          Every topic on the community <a href="https://roadmap.sh/rust">Rust developer roadmap</a>, in the order this course teaches it, from installing Rust to shipping async web
          services. Pick any topic to practice it; the marks fill in as you finish lessons.
        </p>
        <div className="rm-overall">
          <div className="bar" aria-hidden="true">
            <div style={{ width: `${(overall.done / Math.max(overall.total, 1)) * 100}%` }} />
          </div>
          <span className="small muted">
            {overall.done} of {overall.total} lessons done
          </span>
        </div>
      </div>
      <ol className="rm">
        {roadmap.map((sec, i) => {
          const ids = [...new Set(sec.items.flatMap((it) => it.modules ?? []))];
          const p = progressOf(s, ids);
          const complete = p.total > 0 && p.done >= p.total;
          return (
            <li key={sec.title} className={"rm-section" + (complete ? " rm-section-done" : "")}>
              <div className="rm-node" aria-hidden="true">
                {complete ? <Check className="icon" /> : String(i + 1).padStart(2, "0")}
              </div>
              <section className="rm-card" aria-labelledby={`rm-${i}`}>
                <h2 id={`rm-${i}`} className="h3">
                  {sec.title}
                </h2>
                <p className="muted small">{sec.summary}</p>
                <ul className="rm-items">
                  {sec.items.map((it) => {
                    const mods = it.modules ?? [];
                    const ip = progressOf(s, mods);
                    return (
                      <li key={it.title} className="rm-item">
                        {mods.length > 0 ? (
                          <Link to={targetOf(s, mods)} className="rm-topic">
                            <Status {...ip} />
                            <span className="rm-topic-title">{it.title}</span>
                          </Link>
                        ) : (
                          <span className="rm-topic rm-topic-plain">
                            <span className="rm-status rm-ext" aria-hidden="true">
                              <ExternalLink className="icon" />
                            </span>
                            <span className="rm-topic-title">{it.title}</span>
                          </span>
                        )}
                        {(mods.length > 0 || it.note || it.links?.length || it.pro?.length) && (
                          <div className="rm-meta small">
                            {mods.length > 0 && (
                              <span className="muted">
                                {mods
                                  .map((id) => moduleById.get(id)?.title)
                                  .filter(Boolean)
                                  .join(", ")}
                              </span>
                            )}
                            {it.note && <span className="muted"> {it.note}</span>}
                            {it.pro?.map((id) => {
                              const pp = proById.get(id);
                              return pp ? (
                                <Link key={id} to={`/pro/${id}`} className="rm-pro">
                                  <Briefcase className="icon" aria-hidden="true" /> Pro: {pp.title}
                                </Link>
                              ) : null;
                            })}
                            {it.links?.map((l) => (
                              <a key={l.url} href={l.url} target="_blank" rel="noopener noreferrer" className="rm-link">
                                {l.label}
                                <span className="visually-hidden"> (opens in a new tab)</span>
                              </a>
                            ))}
                          </div>
                        )}
                      </li>
                    );
                  })}
                </ul>
              </section>
            </li>
          );
        })}
      </ol>
      <p className="muted small">
        This page follows the structure of <a href="https://roadmap.sh/rust">roadmap.sh/rust</a>, a community project RustArena isn't affiliated with. Topics marked with an outside link are
        specializations beyond this course; <Link to="/next">Where to go next</Link> has more.
      </p>
    </div>
  );
}
