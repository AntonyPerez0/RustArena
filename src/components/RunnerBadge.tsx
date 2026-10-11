import { useSyncExternalStore } from "react";
import { Link } from "react-router-dom";
import { AlertTriangle } from "lucide-react";
import { getRunnerStatus, runnerName, subscribeRunner } from "../runner/client";
import { useStore } from "../state/store";

export function useRunnerStatus() {
  return useSyncExternalStore(subscribeRunner, getRunnerStatus, getRunnerStatus);
}

/** Where code runs, and whether the last request worked. Links to the setting that changes it. */
export default function RunnerBadge({ compact }: { compact?: boolean }) {
  const s = useRunnerStatus();
  useStore((x) => x.settings.runnerUrl);
  const name = runnerName();
  if (s.state === "error")
    return (
      <Link to="/settings#runner" className="pill pill-error" title={`Couldn't run code: ${s.message}`}>
        <AlertTriangle className="icon" aria-hidden="true" /> {compact ? "Runner offline" : `${name} unreachable`}
      </Link>
    );
  if (s.state === "busy")
    return (
      <span className="pill pill-loading" title={`Compiling and running on ${name}`}>
        <span className="pill-bar pill-bar-indeterminate" />
        <span className="pill-text">{compact ? "Running…" : `Running on ${name}…`}</span>
      </span>
    );
  return (
    <Link to="/settings#runner" className="pill pill-ready" title={`Code is compiled with the real Rust compiler on ${name}${s.state === "ok" ? ` (last run took ${(s.ms / 1000).toFixed(1)}s)` : ""}`}>
      <span className="live-dot" aria-hidden="true" /> {compact ? "rustc" : `rustc on ${name}`}
    </Link>
  );
}
