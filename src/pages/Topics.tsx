import { Link, useNavigate, useParams } from "react-router-dom";
import { ChevronRight, TerminalSquare } from "lucide-react";
import { firstStepPath, moduleById, modules, phases, topicBySlug, topics } from "../content";
import Markdown from "../components/Markdown";
import { CodeView } from "../components/highlight";
import { useTitle } from "../lib/title";

/** The curriculum stage a topic belongs to: the phase of the first module that teaches it. */
function phaseOf(t: (typeof topics)[number]): string {
  const m = t.modules.map((id) => moduleById.get(id)).find(Boolean);
  return m?.phase ?? "More";
}
const order = (t: (typeof topics)[number]) => Math.min(...t.modules.map((id) => modules.findIndex((m) => m.id === id)).filter((i) => i >= 0), 999);

const GROUPS: [string, (t: (typeof topics)[number]) => boolean][] = [...phases.map((p) => p.name), "More"].map((name) => [name, (t: (typeof topics)[number]) => phaseOf(t) === name]);

export function TopicIndex() {
  useTitle("Rust topics explained");
  return (
    <div>
      <div className="page-head">
        <h1>Rust topics</h1>
        <p className="muted">Short explanations of the core ideas of Rust, each with a tested example and a link to the lesson where you practice it.</p>
      </div>
      {GROUPS.filter(([, test]) => topics.some(test)).map(([name, test]) => (
        <section key={name} className="phase">
          <h2 className="phase-title">{name}</h2>
          <ul className="topic-list">
            {topics
              .filter(test)
              .sort((a, b) => order(a) - order(b))
              .map((t) => (
              <li key={t.slug}>
                <Link to={`/topics/${t.slug}`} className="topic-title">
                  {t.title}
                </Link>
                <p className="muted small">{t.description}</p>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

/** Opens the example in the playground (it replaces what was there, so ask first if that was edited). */
function TryIt({ code, stdin }: { code: string; stdin: string }) {
  const nav = useNavigate();
  return (
    <p>
      <button
        className="btn"
        onClick={() => {
          try {
            localStorage.setItem("rustarena-playground", JSON.stringify({ code, stdin, args: "" }));
          } catch {
            /* storage blocked: the playground opens with its default program */
          }
          nav("/playground");
        }}
      >
        <TerminalSquare className="icon" aria-hidden="true" /> Edit and run it in the playground
      </button>
    </p>
  );
}

export function TopicPage() {
  const { slug = "" } = useParams();
  const t = topicBySlug.get(slug);
  useTitle(t ? t.title : "Topic not found");
  if (!t) {
    return (
      <div className="page-head">
        <h1>Topic not found</h1>
        <Link to="/topics">See all topics</Link>
      </div>
    );
  }
  const related = topics.filter((x) => x.slug !== t.slug && x.modules.some((m) => t.modules.includes(m))).slice(0, 4);
  return (
    <article className="topic-page narrow">
      <div className="crumbs">
        <Link to="/topics">Topics</Link> <ChevronRight className="icon" aria-hidden="true" /> <span>{phaseOf(t)}</span>
      </div>
      <h1>{t.title}</h1>
      <p className="lead">{t.description}</p>
      <Markdown text={t.body} />
      <h2 className="h3">Example</h2>
      <CodeView code={t.example} />
      {t.stdin && (
        <>
          <div className="lbl">input</div>
          <pre className="console" tabIndex={0}>
            {t.stdin}
          </pre>
        </>
      )}
      <div className="lbl">output</div>
      <pre className="console" tabIndex={0}>
        {t.output}
      </pre>
      <TryIt code={t.example} stdin={t.stdin} />
      {t.modules.length > 0 && (
        <section className="card topic-practice">
          <h2 className="h3">Practice it</h2>
          <ul>
            {t.modules.map((m) => (
              <li key={m}>
                <Link to={firstStepPath(m)}>Lesson: {moduleById.get(m)?.title}</Link>
              </li>
            ))}
          </ul>
        </section>
      )}
      {related.length > 0 && (
        <nav aria-label="Related topics" className="topic-related">
          <h2 className="h3">Related</h2>
          <ul>
            {related.map((x) => (
              <li key={x.slug}>
                <Link to={`/topics/${x.slug}`}>{x.title}</Link>
              </li>
            ))}
          </ul>
        </nav>
      )}
    </article>
  );
}
