import { Link } from "react-router-dom";
import { REPO_URL } from "../lib/site";
import { useTitle } from "../lib/title";

const ext = <span className="visually-hidden"> (opens in a new tab)</span>;

function A({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <a href={href} target="_blank" rel="noopener noreferrer">
      {children}
      {ext}
    </a>
  );
}

export default function About() {
  useTitle("About RustArena");
  return (
    <article className="narrow about md">
      <h1>About RustArena</h1>
      <p className="lead">
        RustArena is a free, open-source course that takes you from never having programmed to writing the kind of Rust teams ship at work. You learn by writing code that has to
        compile and pass tests: lessons, endless Deathmatch drills, guided projects, and a Pro Track you complete on your own machine with the tools professionals use.
      </p>

      <h2>How your code runs</h2>
      <p>
        RustArena has no server and no accounts. When you press <b>Check</b> or <b>Run</b>, the site sends your code (with the exercise's hidden tests) to the{" "}
        <A href="https://play.rust-lang.org">Rust Playground</A>, the official service run by the Rust project that also powers the "Run" buttons in{" "}
        <A href="https://doc.rust-lang.org/book/">The Rust Programming Language</A>. It compiles with the current stable <code>rustc</code> (edition 2024, debug build) and returns the
        output. You can switch to your own runner in <Link to="/settings#runner">Settings</Link>, including one on your computer (<code>npm run runner</code>), which works offline.
      </p>
      <p>
        Every reference solution, worked example and drill on the site was compiled and run with the real Rust compiler when the site was built, so the expected outputs are real, not
        typed by hand.
      </p>

      <h2>Your data</h2>
      <p>
        Your progress is stored in your browser (localStorage) and never leaves it unless you choose to sync it (a transfer link, or a private GitHub Gist you control). The only thing sent
        anywhere is the code you run, to the runner you chose.
      </p>

      <h2>Credits and trademarks</h2>
      <ul>
        <li>
          <b>Rust</b> and the Rust logo are trademarks of the <A href="https://rustfoundation.org/">Rust Foundation</A>. RustArena is an independent learning site; it is not
          affiliated with or endorsed by the Rust Foundation or the Rust Project. We refer to Rust by name to say what this site teaches, and we don't use the Rust logo: the crab in our
          logo is our own drawing. See the <A href="https://www.rust-lang.org/policies/media-guide">Rust media guide and trademark policy</A>.
        </li>
        <li>
          <b>Ferris the crab</b>, the Rust community's unofficial mascot, was created by Karen Rustad Tölva and dedicated to the public domain (CC0). The Ferris artwork comes from{" "}
          <A href="https://rustacean.net/">rustacean.net</A>.
        </li>
        <li>
          The order of the curriculum follows the community <A href="https://roadmap.sh/rust">Rust developer roadmap</A> on roadmap.sh. RustArena isn't affiliated with roadmap.sh.
        </li>
        <li>
          Code runs on the <A href="https://github.com/rust-lang/rust-playground">Rust Playground</A>. Error explanations link to the official{" "}
          <A href="https://doc.rust-lang.org/error_codes/error-index.html">Rust error codes index</A>.
        </li>
        <li>
          Built with React, React Router, CodeMirror, Vite, marked, Lucide icons, and the Inter and JetBrains Mono fonts.
        </li>
        <li>
          RustArena is a sister site of <A href="https://cpparena.com">C/C++ Arena</A> and <A href="https://github.com/AntonyPerez0/Java-Arena">Java Arena</A>, and shares their design.
        </li>
      </ul>

      <h2>Help improve it</h2>
      <p>
        Found a mistake, a confusing hint, or a correct answer marked wrong? Every exercise has a <b>Report a problem</b> link, or open an issue on{" "}
        <A href={REPO_URL}>GitHub</A>. The lessons are YAML files; the repository's <code>docs/AUTHORING.md</code> explains how to write and check them.
      </p>
    </article>
  );
}
