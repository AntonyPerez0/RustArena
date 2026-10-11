import { useState } from "react";
import { Link } from "react-router-dom";
import { patchSettings, useStore, type Theme } from "../state/store";
import { TEXT_SIZES } from "../lib/appearance";
import { useTitle } from "../lib/title";
import RunnerBadge from "../components/RunnerBadge";
import { DEFAULT_RUNNER, compileAndRun, runnerName } from "../runner/client";

const LOCAL_RUNNER = "http://localhost:8787";

/** Where code runs: the Rust Playground, a runner on this computer, or any server speaking its API. */
function RunnerSettings() {
  const s = useStore((x) => x.settings);
  const current = s.runnerUrl.trim();
  const choice = !current ? "playground" : current === LOCAL_RUNNER ? "local" : "custom";
  const [custom, setCustom] = useState(choice === "custom" ? current : "");
  const [test, setTest] = useState<string>("");
  const [testing, setTesting] = useState(false);

  const check = async () => {
    setTesting(true);
    setTest(`Compiling a test program on ${runnerName()}…`);
    const r = await compileAndRun(`fn main() {\n    println!("{}", 6 * 7);\n}\n`);
    setTesting(false);
    if (r.internalError) setTest(`It didn't work: ${r.internalError}.`);
    else if (!r.compiled) setTest("The server answered, but the test program didn't compile there. Is it a Rust Playground (or `npm run runner`)?");
    else if (r.runs[0]?.stdout.trim() === "42") setTest(`It works: compiled and ran in ${(r.compileMs / 1000).toFixed(1)}s.`);
    else setTest("The server answered, but the program's output was unexpected.");
  };

  return (
    <section className="card" id="runner" aria-labelledby="runner-h">
      <h2 className="h3" id="runner-h">
        Where your code runs
      </h2>
      <p className="muted small">
        RustArena has no server of its own. When you press Check or Run, your code is sent to a Rust compiler service, compiled with the real <code>rustc</code> (stable, edition 2024, debug
        build) and run there. Nothing else about you is sent.
      </p>
      <fieldset className="radio-stack">
        <legend className="visually-hidden">Runner</legend>
        <label>
          <input type="radio" name="runner" checked={choice === "playground"} onChange={() => patchSettings({ runnerUrl: "" })} /> <b>The Rust Playground</b> (play.rust-lang.org, the
          official service from the Rust project). Recommended.
        </label>
        <label>
          <input type="radio" name="runner" checked={choice === "local"} onChange={() => patchSettings({ runnerUrl: LOCAL_RUNNER })} /> <b>A runner on this computer</b> ({LOCAL_RUNNER}). Uses
          your own Rust installation: run <code>npm run runner</code> in a copy of the RustArena repository. Works offline.
        </label>
        <label>
          <input
            type="radio"
            name="runner"
            checked={choice === "custom"}
            onChange={() => {
              if (custom.trim()) patchSettings({ runnerUrl: custom.trim() });
            }}
          />{" "}
          <b>Another server</b>, for example a school's or company's own copy of the Rust Playground:
        </label>
        <div className="row runner-custom">
          <label className="visually-hidden" htmlFor="runner-url">
            Server address
          </label>
          <input id="runner-url" className="answer-input" placeholder="https://playground.example.com" value={custom} onChange={(e) => setCustom(e.target.value)} spellCheck={false} />
          <button className="btn" disabled={!/^https?:\/\/\S+$/.test(custom.trim())} onClick={() => patchSettings({ runnerUrl: custom.trim() })}>
            Use this server
          </button>
        </div>
      </fieldset>
      <div className="actions">
        <RunnerBadge />
        <button className="btn" onClick={check} disabled={testing}>
          Test the connection
        </button>
      </div>
      <p role="status" aria-live="polite" className="small">
        {test}
      </p>
      <p className="muted small">
        Running code needs an internet connection (unless you use a runner on this computer). The Playground is a free, shared service: please don't use it for heavy work. Identical code is only
        sent once per visit. {current && current !== DEFAULT_RUNNER && <>You're using {runnerName()}. </>}
        See <Link to="/about">About</Link> for details.
      </p>
    </section>
  );
}

export default function Settings() {
  useTitle("Settings");
  const s = useStore((x) => x);
  return (
    <div className="narrow settings">
      <div className="page-head">
        <h1>Settings</h1>
        <p className="muted">Saved in this browser.</p>
      </div>
      <section className="card">
        <h2 className="h3">Appearance</h2>
        <fieldset className="radio-row">
          <legend>Theme</legend>
          {(
            [
              ["system", "Match my device"],
              ["dark", "Dark"],
              ["light", "Light"],
            ] as [Theme, string][]
          ).map(([v, label]) => (
            <label key={v}>
              <input type="radio" name="theme" value={v} checked={s.settings.theme === v} onChange={() => patchSettings({ theme: v })} /> {label}
            </label>
          ))}
        </fieldset>
        <fieldset className="radio-row">
          <legend>Text size</legend>
          {TEXT_SIZES.map((t) => (
            <label key={t.value}>
              <input type="radio" name="textsize" value={t.value} checked={s.settings.textScale === t.value} onChange={() => patchSettings({ textScale: t.value })} /> {t.label}
            </label>
          ))}
        </fieldset>
      </section>
      <RunnerSettings />
      <section className="card">
        <h2 className="h3">Progress</h2>
        <p className="muted small">
          Export, import, sync between devices or reset your progress on <Link to="/profile">your profile</Link>.
        </p>
      </section>
    </div>
  );
}
