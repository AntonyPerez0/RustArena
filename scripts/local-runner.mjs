// A stand-in for the Rust Playground that compiles and runs code on this machine with your own
// Rust toolchain. Use it to work offline, in a classroom with its own server, or in the browser
// tests. In the site, open Settings and set "Where code runs" to http://localhost:8787.
//
//   npm run runner                  # listens on 127.0.0.1:8787
//   PORT=9000 npm run runner
//
// It runs whatever code it is sent, so it only listens on this computer (127.0.0.1) unless
// HOST is set. Don't expose it to a network you don't trust.
import http from "node:http";
import { execute, depsFlags } from "./rust-exec.mjs";

const PORT = Number(process.env.PORT ?? 8787);
const HOST = process.env.HOST ?? "127.0.0.1";
const MAX_BODY = 512 * 1024;
const PARALLEL = Number(process.env.PARALLEL ?? 2);

let active = 0;
const waiting = [];
function limit(fn) {
  return new Promise((resolve, reject) => {
    const go = () => {
      active++;
      fn()
        .then(resolve, reject)
        .finally(() => {
          active--;
          waiting.shift()?.();
        });
    };
    if (active < PARALLEL) go();
    else waiting.push(go);
  });
}

function send(res, status, body) {
  res.writeHead(status, {
    "Content-Type": "application/json",
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
    "Access-Control-Max-Age": "3600",
  });
  res.end(JSON.stringify(body));
}

const server = http.createServer((req, res) => {
  if (req.method === "OPTIONS") return send(res, 204, {});
  if (req.method === "GET" && req.url === "/") return send(res, 200, { ok: true, name: "RustArena local runner" });
  if (req.method !== "POST" || !["/execute", "/evaluate.json"].includes(req.url ?? "")) return send(res, 404, { error: "not found" });
  let body = "";
  req.on("data", (d) => {
    body += d;
    if (body.length > MAX_BODY) req.destroy();
  });
  req.on("end", async () => {
    let j;
    try {
      j = JSON.parse(body);
    } catch {
      return send(res, 400, { error: "bad JSON" });
    }
    if (typeof j.code !== "string") return send(res, 400, { error: "code missing" });
    try {
      const out = await limit(() => execute({ code: j.code, tests: !!j.tests, edition: String(j.edition ?? "2024") }));
      if (req.url === "/evaluate.json") return send(res, 200, out.success ? { result: out.stdout, error: null } : { result: out.stderr + out.stdout, error: out.stderr + out.stdout });
      send(res, 200, out);
    } catch (e) {
      send(res, 500, { error: String(e?.message ?? e) });
    }
  });
});

console.log("Building the crates lessons can use (first time only)...");
depsFlags();
server.listen(PORT, HOST, () => console.log(`RustArena local runner on http://${HOST}:${PORT} (set it under Settings in the site)`));
