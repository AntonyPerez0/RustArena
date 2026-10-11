import { Link } from "react-router-dom";
import type { ReactNode } from "react";
import {
  ArrowRight,
  BookOpen,
  Briefcase,
  CalendarCheck,
  Check,
  CircleCheck,
  Code2,
  Crosshair,
  GraduationCap,
  Hammer,
  Map as MapIcon,
  ShieldCheck,
  TerminalSquare,
} from "lucide-react";
import { useStore } from "../state/store";
import { nextStep, rankFor, totals, dailyStreak, localDay } from "../state/derived";
import { drills, modules, projects, pro, placement, phases, stepPath, topics } from "../content";
import RunnerBadge from "../components/RunnerBadge";
import { highlight } from "../components/highlight";
import { useTitle } from "../lib/title";

const DEMO = `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn main() {
    let words = ["borrow", "checker", "crab"];
    let mut best = "";
    for w in words {
        best = longest(best, w);
    }
    println!("longest: {best}");
}`;

/** A still picture of the lesson workspace: code on top, the checked result below. */
function ProductShot() {
  return (
    <figure className="shot" aria-label="Example: a Rust exercise that passes its tests">
      <div className="shot-bar">
        <span className="shot-dots" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className="shot-tab">main.rs</span>
        <span className="shot-lang lang-tag lang-rs">Rust 2024</span>
      </div>
      <pre className="shot-code">
        <code>{highlight(DEMO, "demo")}</code>
      </pre>
      <div className="shot-result">
        <div className="shot-pass">
          <CircleCheck className="icon" aria-hidden="true" /> All tests passed
        </div>
        <ul>
          <li>
            <Check className="icon ic-good" aria-hidden="true" /> prints <code>longest: checker</code>
          </li>
          <li>
            <Check className="icon ic-good" aria-hidden="true" /> the borrow checker is happy
          </li>
        </ul>
        <div className="shot-foot">Compiled and tested by the real Rust compiler</div>
      </div>
    </figure>
  );
}

function Feature({ icon, title, children, to, span }: { icon: ReactNode; title: string; children: ReactNode; to: string; span?: "wide" | "wide-md" }) {
  return (
    <Link to={to} className={span ? `feature feature-${span}` : "feature"}>
      <span className="feature-icon" aria-hidden="true">
        {icon}
      </span>
      <h3>{title}</h3>
      <p>{children}</p>
      <span className="feature-more">
        Explore <ArrowRight className="icon" aria-hidden="true" />
      </span>
    </Link>
  );
}

export default function Home() {
  useTitle(null);
  const s = useStore((x) => x);
  const t = totals(s);
  const next = nextStep(s);
  const r = rankFor(s.dm.best.deathmatch);
  const streak = dailyStreak(s.dm.days);
  const projDone = projects.filter((p) => (s.projects[p.id]?.completed.length ?? 0) >= p.milestones.length).length;
  const started = t.done > 0 || s.dm.kills > 0 || s.placed.length > 0;
  const interview = drills.filter((d) => d.id.startsWith("interview")).length;
  const startTo = next ? stepPath(next.module, next.step) : "/deathmatch";
  const startLabel = !next ? "All lessons done. Go get reps" : t.done === 0 ? "Start learning free" : `Continue: ${next.step.title}`;

  return (
    <div className="landing">
      <section className="hero">
        <div className="hero-bg" aria-hidden="true" />
        <div className="container hero-grid">
          <div className="hero-copy">
            <p className="eyebrow">
              <ShieldCheck className="icon" aria-hidden="true" /> Every exercise checked by the real Rust compiler
            </p>
            <h1>
              Learn Rust by <span className="hero-accent">writing real code</span>.
            </h1>
            <p className="hero-lead">
              Go from your first <code>println!</code> to the Rust teams ship at work: ownership and borrowing, traits, error handling, iterators, async, testing, serde and
              web services. Short lessons, endless Deathmatch drills, guided projects, and a Pro Track on your own machine.
            </p>
            <div className="hero-actions">
              <Link className="btn btn-brand btn-lg" to={startTo}>
                {startLabel} <ArrowRight className="icon" aria-hidden="true" />
              </Link>
              <Link className="btn btn-lg" to="/roadmap">
                <MapIcon className="icon" aria-hidden="true" /> See the roadmap
              </Link>
            </div>
            <ul className="hero-points">
              <li>
                <Check className="icon" aria-hidden="true" /> Free and open source
              </li>
              <li>
                <Check className="icon" aria-hidden="true" /> No install, no account
              </li>
              <li>
                <Check className="icon" aria-hidden="true" /> Beginner to job-ready
              </li>
            </ul>
            {t.done === 0 && s.placed.length === 0 && (
              <p className="small muted hero-note">
                Already know some Rust? <Link to="/placement">Take the 5-minute placement quiz</Link> and skip what you know.
              </p>
            )}
          </div>
          <div className="hero-visual">
            <ProductShot />
            {/* Ferris the crab, the Rust community's unofficial mascot (public domain, rustacean.net). */}
            <img className="hero-ferris" src={`${import.meta.env.BASE_URL}ferris.png`} alt="Ferris the crab, the Rust mascot" width="230" height="153" />
          </div>
        </div>
      </section>

      <section className="facts" aria-label="The course in numbers">
        <div className="container facts-grid">
          {(
            [
              [modules.length, "modules"],
              [t.total, "lessons"],
              [drills.length, "practice drills"],
              [projects.length, "guided projects"],
              [pro.length, "Pro Track projects"],
              [topics.length, "topic guides"],
            ] as [number, string][]
          ).map(([n, label]) => (
            <div key={label} className="fact">
              <div className="fact-n">{n}</div>
              <div className="fact-l">{label}</div>
            </div>
          ))}
        </div>
      </section>

      {started && (
        <section className="section section-tight" aria-labelledby="progress-h">
          <div className="container">
            <div className="section-head section-head-row">
              <div>
                <p className="eyebrow">Your progress</p>
                <h2 id="progress-h">Welcome back</h2>
              </div>
              {next && (
                <Link className="btn btn-primary" to={startTo}>
                  Continue: {next.step.title} <ArrowRight className="icon" aria-hidden="true" />
                </Link>
              )}
            </div>
            <div className="stat-row">
              <div className="stat">
                <div className="stat-n">
                  {t.done}
                  <span className="muted">/{t.total}</span>
                </div>
                <div className="stat-l">lesson steps</div>
                <div className="bar" aria-hidden="true">
                  <div style={{ width: `${(t.done / Math.max(t.total, 1)) * 100}%` }} />
                </div>
              </div>
              <div className="stat">
                <div className="stat-n">{s.dm.best.deathmatch}</div>
                <div className="stat-l">best deathmatch streak</div>
                <div className="rank-chip">{r.rank.name}</div>
              </div>
              <div className="stat">
                <div className="stat-n">{s.dm.kills}</div>
                <div className="stat-l">total reps landed</div>
                <div className="muted small">{streak > 0 ? `${streak}-day streak` : "20 reps in a day starts a streak"}</div>
              </div>
              <div className="stat">
                <div className="stat-n">
                  {projDone}
                  <span className="muted">/{projects.length}</span>
                </div>
                <div className="stat-l">projects shipped</div>
              </div>
            </div>
          </div>
        </section>
      )}

      <section className="section" aria-labelledby="how-h">
        <div className="container">
          <div className="section-head">
            <p className="eyebrow">How it works</p>
            <h2 id="how-h">Read a little, write a lot, get real feedback</h2>
            <p className="section-lead">Each lesson is one idea and a few exercises. You learn by typing code that has to compile and pass tests, not by watching videos.</p>
          </div>
          <ol className="how">
            <li>
              <span className="how-icon" aria-hidden="true">
                <BookOpen className="icon" />
              </span>
              <h3>Learn one idea</h3>
              <p>A short explanation with worked examples, the compiler errors you'll meet and how to fix them. Early steps are fill-in-the-blank; later you write it all.</p>
            </li>
            <li>
              <span className="how-icon" aria-hidden="true">
                <Code2 className="icon" />
              </span>
              <h3>Write the code</h3>
              <p>A real editor with Rust syntax highlighting, hints you reveal one at a time, and a symbol bar for typing <code>&amp;</code>, <code>::</code> and <code>{"=>"}</code> on a phone.</p>
            </li>
            <li>
              <span className="how-icon" aria-hidden="true">
                <CircleCheck className="icon" />
              </span>
              <h3>Compile and check it</h3>
              <p>
                rustc compiles your program and tests check what it does. Borrow-checker errors come with rustc's own message, a plain-English note, and a link to the
                official explanation.
              </p>
            </li>
          </ol>
        </div>
      </section>

      <section className="section section-alt" aria-labelledby="features-h">
        <div className="container">
          <div className="section-head">
            <p className="eyebrow">Everything in one place</p>
            <h2 id="features-h">From your first println! to production Rust</h2>
          </div>
          <div className="features">
            <Feature span="wide" to="/learn" icon={<BookOpen className="icon" />} title={`${modules.length} modules, zero to professional`}>
              Following the Rust roadmap: the basics, then ownership and borrowing, structs and enums, error handling, traits, generics and lifetimes, iterators and
              closures, smart pointers, concurrency and async, macros and unsafe, then workplace Rust (serde, anyhow and thiserror, CLIs, web services, testing) and
              algorithms for interviews.
            </Feature>
            <Feature to="/deathmatch" icon={<Crosshair className="icon" />} title="Deathmatch drills">
              {drills.length} quick reps: predict the output, fill the token, spot the bug, and "does this compile?" against the borrow checker. Every 8th rep is a boss
              you compile for real.
            </Feature>
            <Feature to="/projects" icon={<Hammer className="icon" />} title={`${projects.length} guided projects`}>
              Build real programs milestone by milestone: a todo CLI, a bank ledger, a tokenizer and interpreter, a JSON-backed inventory, a log analyzer and more.
            </Feature>
            <Feature to="/roadmap" icon={<MapIcon className="icon" />} title="The Rust roadmap">
              Every topic of the community Rust roadmap, from syntax to async and the ecosystem, mapped to the lessons and projects that teach it.
            </Feature>
            <Feature to="/playground" icon={<TerminalSquare className="icon" />} title="Playground">
              Run any Rust program with input and arguments, with serde, tokio, anyhow and more available, and share it as a link.
            </Feature>
            <Feature to="/deathmatch" icon={<GraduationCap className="icon" />} title="Interview prep">
              {interview} classic interview questions on ownership, lifetimes, traits, smart pointers, concurrency and data structures.
            </Feature>
            <Feature to="/daily" icon={<CalendarCheck className="icon" />} title="Daily challenge">
              {localDay() in s.daily ? "Done for today. Come back tomorrow." : "One quick problem a day, the same for everyone. Keep your streak going."}
            </Feature>
            <Feature span="wide-md" to="/pro" icon={<Briefcase className="icon" />} title={`Pro Track: ${pro.length} projects on a real machine`}>
              Cargo workspaces, Git and pull requests, clippy and rustfmt, tests and CI, a CLI with clap, an async web API with axum, a database with sqlx, tracing and
              benchmarks, graded by GitHub Actions in your own repository, like a team's CI.
            </Feature>
          </div>
        </div>
      </section>

      <section className="section" aria-labelledby="curriculum-h">
        <div className="container">
          <div className="section-head section-head-row">
            <div>
              <p className="eyebrow">Curriculum</p>
              <h2 id="curriculum-h">A path, not a pile of tutorials</h2>
              <p className="section-lead">
                {t.total} lessons in {phases.length} stages, in the order that makes each idea easy. Drills unlock as you finish the lessons that teach them.
              </p>
            </div>
            <Link className="btn" to="/learn">
              See the full curriculum <ArrowRight className="icon" aria-hidden="true" />
            </Link>
          </div>
          <ol className="tracks">
            {phases.map((ph, i) => (
              <li key={ph.name}>
                <Link to="/learn" className="track">
                  <span className="track-n" aria-hidden="true">
                    {String(i + 1).padStart(2, "0")}
                  </span>
                  <span className="track-body">
                    <span className="track-name">{ph.name}</span>
                    <span className="track-meta">
                      {ph.modules.length} {ph.modules.length === 1 ? "module" : "modules"} · {ph.modules.reduce((a, m) => a + m.steps.length, 0)} lessons
                    </span>
                  </span>
                </Link>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="section section-alt" aria-labelledby="faq-h">
        <div className="container faq-wrap">
          <div className="section-head">
            <p className="eyebrow">Questions</p>
            <h2 id="faq-h">Frequently asked questions</h2>
          </div>
          <div className="faq">
            <details>
              <summary>Is it really free?</summary>
              <p>Yes. Every lesson, drill and project, and the whole Pro Track, is free. The site is open source, and the code is on GitHub.</p>
            </details>
            <details>
              <summary>Do I need to install anything?</summary>
              <p>
                No. When you press Check or Run, your code is compiled and run by the real Rust compiler on the <a href="https://play.rust-lang.org">Rust Playground</a>,
                the same service that runs the examples in the official Rust book. You'll install Rust yourself (with <a href="https://rustup.rs">rustup</a>) when you
                reach the Pro Track, because that's how you'll work on a job.
              </p>
            </details>
            <details>
              <summary>Do I need an account?</summary>
              <p>
                No. Your progress is saved in your browser. To move it to another device, use a transfer link or a private GitHub Gist from <Link to="/profile">your
                profile</Link>.
              </p>
            </details>
            <details>
              <summary>Which Rust version does it use?</summary>
              <p>
                The current stable Rust with the 2024 edition, in a debug build (so integer overflow is caught, like <code>cargo run</code>). Popular crates are
                available too: <code>serde</code>, <code>serde_json</code>, <code>tokio</code>, <code>anyhow</code>, <code>thiserror</code>, <code>regex</code>,{" "}
                <code>itertools</code>, <code>rayon</code> and <code>chrono</code>.
              </p>
            </details>
            <details>
              <summary>Will this get me ready to use Rust at work?</summary>
              <p>
                That's the goal. The later modules teach what Rust teams actually use (Result-based error handling with anyhow and thiserror, serde, async with Tokio,
                testing, clippy, modules and Cargo), and the <Link to="/pro">Pro Track</Link> has you build and ship real projects (a CLI, a web API, a database-backed
                service) with Git, pull requests and CI, the way a team works.
              </p>
            </details>
            <details>
              <summary>Can I learn on my phone?</summary>
              <p>Yes. Every page fits a phone screen, and a symbol bar puts code characters one tap away. Lessons you've opened also stay readable offline.</p>
            </details>
            <details>
              <summary>I already know some Rust. Where should I start?</summary>
              <p>
                Take the <Link to="/placement">placement quiz</Link>: {placement.length} quick questions that mark the modules you already know, so you can skip them.
              </p>
            </details>
          </div>
        </div>
      </section>

      <section className="cta-band" aria-labelledby="cta-h">
        <div className="container cta-inner">
          <h2 id="cta-h">Write your first Rust program today.</h2>
          <p>No download, no account. Just open a lesson and start typing.</p>
          <div className="hero-actions cta-actions">
            <Link className="btn btn-brand btn-lg" to={startTo}>
              {startLabel} <ArrowRight className="icon" aria-hidden="true" />
            </Link>
          </div>
          <div className="cta-compiler">
            <RunnerBadge />
            <span className="muted small">Code is compiled and run by the official Rust Playground. You can point the site at your own runner in Settings.</span>
          </div>
        </div>
      </section>
    </div>
  );
}
