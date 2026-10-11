# 12 · Capstone: a production-ready web service

Time to put everything together. You'll build a **URL shortener**, a small service that turns `https://www.rust-lang.org/learn` into `https://sho.rt/Xy3k9Qa` and redirects visitors, and you'll build it the way it would ship at work: tested, configured from the environment, logging structured events, reporting its health, shutting down cleanly, packaged as a container image, and checked by CI.

Only the skeleton is provided: the public signatures the tests use, the database setup from project 09, and the tests. The design of everything in between is yours.

**You'll practice:** axum, SQLx and migrations, `tracing`, configuration, error handling, status codes, health checks, graceful shutdown, Docker multi-stage builds, and a CI pipeline.

## The service

| Request | Success | Errors |
|---|---|---|
| `GET /health` | `200 {"status":"ok"}` | `503` if the database doesn't answer |
| `POST /api/links` with `{"url": "...", "code": "optional"}` | `201`, `Location: /api/links/{code}`, the link as JSON | `422` invalid URL or code, `409` code taken |
| `GET /api/links/{code}` | `200` and the link as JSON | `404` |
| `DELETE /api/links/{code}` | `204` | `404` |
| `GET /{code}` | `307` with `Location:` the URL, and one more hit counted | `404` |

A link as JSON: `{"code": "Xy3k9Qa", "url": "https://www.rust-lang.org/learn", "short_url": "https://sho.rt/Xy3k9Qa", "hits": 0, "created_at": "2026-05-04T10:00:00Z"}`. Every error response is JSON too: `{"error": "the code \"rust\" is already taken"}`.

The rules:

- **URL**: an absolute `http` or `https` URL with a host, at most 2048 characters. Parse it with the `url` crate; store the parsed (normalized) form.
- **Custom code**: 4 to 32 ASCII letters, digits, `-` or `_`, and not `api` or `health` (they'd clash with the API's own paths).
- **Generated code**: 7 random letters and digits (`rand::distr::Alphanumeric`). The same URL shortened twice gets two codes.

Configuration comes from environment variables, each with a default that works on a laptop:

| Variable | Default | |
|---|---|---|
| `HOST` | `0.0.0.0` | IP address to listen on |
| `PORT` | `3000` | |
| `DATABASE_URL` | `sqlite:shortener.db` | |
| `PUBLIC_URL` | `http://localhost:{PORT}` | the address users see; short links start with it |
| `LOG_FORMAT` | `text` | or `json` |
| `RUST_LOG` | `info` | read by the `EnvFilter`, as in project 10 |

## Background

### Status codes are part of the API

Clients, browsers, proxies and monitoring all act on the status code, so pick precise ones:

- **201 Created** plus a `Location` header pointing at the new resource, not a plain 200.
- **307 Temporary Redirect** for short links, not **301 Moved Permanently**: browsers cache permanent redirects forever, so later visits would never reach you (no hit counted) and a deleted link would keep working.
- **409 Conflict** when the request is valid but clashes with existing state (a taken code). Let the database's `PRIMARY KEY` detect it, as in project 09: checking first and inserting after is a race.
- **422 Unprocessable Entity** for well-formed JSON with invalid values. axum's `Json` extractor already answers **400** for broken JSON, **415** without a JSON content type, and **422** for missing fields.
- **500** for your bugs and outages, with a generic message. Log the details (`tracing::error!`), but never send SQL errors or file paths to clients.

### Thin handlers

A handler's job is HTTP: extract the input, call the logic, map the result to a response. Keep validation and SQL in `links.rs`, and give the crate one error type in `error.rs` that implements `IntoResponse`, so every handler can use `?` and every error becomes the right status and JSON body in one place. Shared resources (the pool, the public URL) live in `AppState`, which axum clones into each request.

### Health checks and graceful shutdown

Load balancers and orchestrators (Kubernetes, ECS, Fly.io) call a health endpoint to decide whether to send traffic to an instance. A useful check proves the service can do its job, so `/health` runs `SELECT 1` and answers 503 when the database is gone.

When an instance is replaced, `docker stop` or Kubernetes sends **SIGTERM**, waits a grace period (10 seconds for Docker, 30 for Kubernetes), then kills it. A graceful shutdown stops accepting connections, lets requests in progress finish, closes the database, and exits with code 0. In axum that's `axum::serve(listener, app).with_graceful_shutdown(signal)`, where `signal` is a future that completes on Ctrl+C or SIGTERM. Without it, every deploy cuts off users mid-request.

### Logging requests

`tower_http::trace::TraceLayer` wraps every request in a span (method, path) and logs each response with its status and latency. Its default level is DEBUG; make the span and response events INFO so they show up in production logs:

```rust
TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
    .on_response(DefaultOnResponse::new().level(Level::INFO))
```

### Containers

A container image packages the program with everything it needs, so it runs the same on your laptop, in CI and in production. A **multi-stage** Dockerfile compiles in a big image that has the Rust toolchain (`FROM rust:1-bookworm AS builder`), then copies only the binary into a small runtime image (`FROM debian:bookworm-slim`, then `COPY --from=builder ...`). The result is a fraction of the size and has no compiler or source code inside.

- Copy `Cargo.toml` and `Cargo.lock` and build the dependencies before copying `src/`, so Docker caches that slow layer until the dependencies change. (`cargo-chef` does this more robustly.)
- Build with `--locked`, so the image uses exactly the versions in `Cargo.lock`.
- Copy `migrations/` before building: `sqlx::migrate!` reads it at compile time.
- Run as an unprivileged `USER`, keep the SQLite file in a `VOLUME` such as `/data`, and add a `.dockerignore` so `target/` and local databases stay out of the build context.

### CI for your service

This repository's Grade workflow tests your code here. Once the shortener moves into its own repository, it needs its own pipeline. Write it now as `ci.yml`: on every push and pull request, check formatting, run clippy with `-D warnings`, run the tests, then build the Docker image. Copied to `.github/workflows/ci.yml` in a new repository, it works as is.

## Your tasks

1. Read `src/lib.rs`, `src/config.rs`, `tests/api.rs` and `tests/config.rs`. Keep the public names the tests use.
2. Write `migrations/0001_create_links.sql`: `code` (primary key), `url`, `hits` (default 0) and `created_at` (default `strftime('%Y-%m-%dT%H:%M:%SZ', 'now')`).
3. Implement `Config::from_env_with`. Run `cargo test --test config`.
4. Write `AppError` in `src/error.rs`, then validation, code generation and the SQL in `src/links.rs` (an `UPDATE links SET hits = hits + 1 WHERE code = ? RETURNING url` counts a visit and finds the URL in one statement).
5. Write the handlers and the router in `src/routes.rs`, and make `app()` return it. Run `cargo test --test api`.
6. Implement `serve`, `shutdown_signal`, `telemetry::init` (reuse project 10) and `main`. Then try it for real:

```bash
cargo run
curl -i localhost:3000/health
curl -i -H 'Content-Type: application/json' -d '{"url":"https://www.rust-lang.org/learn"}' localhost:3000/api/links
curl -i localhost:3000/<the code you got>
curl -s localhost:3000/api/links/<the code>
LOG_FORMAT=json RUST_LOG=debug cargo run
```

   Press Ctrl+C: the service should log that it's shutting down and exit.

7. Write `Dockerfile` and `.dockerignore`. If you have Docker (Docker Desktop on your machine works; a Codespace may not have it), build and run the image:

```bash
docker build -t shortener .
docker run --rm -p 3000:3000 -v shortener-data:/data shortener
docker stop $(docker ps -q --filter ancestor=shortener)   # graceful: exit code 0
```

8. Write `ci.yml`, then grade: `bash ../../tools/grade.sh 12-capstone`.

## What the grader checks

- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean.
- `tests/api.rs` and `tests/config.rs` are unchanged, and all tests pass.
- The real binary, started with `HOST`, `PORT`, `DATABASE_URL` and `PUBLIC_URL`, answers `/health`, creates a link (201), redirects (307), answers 404 for an unknown code, exits with code 0 within 10 seconds of SIGTERM, and leaves the link with its hit in the database file.
- `Dockerfile` is a multi-stage build: two or more `FROM` lines, a named build stage, `cargo build --release`, `COPY --from=`, and a `CMD` or `ENTRYPOINT`; its final stage sets a non-root `USER`.
- `.dockerignore` excludes `target`.
- `ci.yml` has `on:` and `jobs:`, and runs `cargo fmt`, `cargo clippy`, `cargo test` and `docker build`.

## Hints

- Route paths in axum 0.8 use braces: `.route("/{code}", get(follow))`. Static routes like `/health` win over `/{code}`, which is why `health` is a reserved code.
- `Redirect::temporary(&url)` is a 307. For the 201, return a tuple: `(StatusCode::CREATED, [(header::LOCATION, location)], Json(body))`.
- A `#[derive(Deserialize)] struct CreateLink { url: String, code: Option<String> }` makes `code` optional.
- When a generated code collides (the 409 path), try again with a new one a few times before giving up.
- `tokio::select!` waits for whichever future finishes first; `tokio::signal::unix::signal(SignalKind::terminate())` listens for SIGTERM. Wrap the Unix part in `#[cfg(unix)]`.
- If the grader's service check fails, it prints everything your program logged. Run the same `curl` commands yourself.

## Going further

- Add request IDs (`tower_http::request_id`) and include them in every log line.
- Rate-limit link creation per client IP (`tower_governor`).
- Expose Prometheus metrics (`metrics` and `metrics-exporter-prometheus`): links created, redirects, latency.
- Generate an OpenAPI description with `utoipa` and serve Swagger UI.
- Make the image smaller: a static musl build in `FROM scratch` or a distroless image, with `cargo-chef` for caching.
- Switch to PostgreSQL with `sqlx::PgPool` and run it next to the service with Docker Compose.
- Deploy it (Fly.io, Render, Railway or Cloud Run all run a Dockerfile) and load-test it with `oha`.
- Protect `DELETE` with an API key read from the environment, compared in constant time.
