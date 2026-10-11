# 08 · A REST API with axum

A large share of backend work is HTTP APIs that speak JSON. **axum**, from the Tokio team, is the most popular Rust web framework: routing and "extractors" on top of hyper (HTTP) and tower (middleware). You'll build a bookmarks API the way teams do: clear routes, validated input, the right status code for every outcome, errors as JSON, and tests that exercise every endpoint without opening a port.

**You'll practice:** routing, extractors (`State`, `Path`, `Query`, `Json`), serde, status codes, an error type that implements `IntoResponse`, shared state, and testing with tower's `oneshot`.

## The API

| Method and path | Success | Errors |
|---|---|---|
| `GET /health` | 200 `{"status":"ok"}` (done) | |
| `POST /bookmarks` | 201, `Location: /bookmarks/1`, the bookmark | 422 invalid, 409 duplicate url, 400/415 bad body |
| `GET /bookmarks/{id}` | 200, the bookmark | 404, 400 if the id isn't a number |
| `PATCH /bookmarks/{id}` | 200, the updated bookmark | 404, 422, 409 |
| `DELETE /bookmarks/{id}` | 204, no body | 404 |
| `GET /bookmarks?tag=&q=&limit=&offset=` | 200 `{"items": [...], "total": n}` | 400 bad parameters |
| anything else | | 404 `no route for GET /nope`, 405 wrong method |

A bookmark looks like `{"id": 1, "url": "https://www.rust-lang.org", "title": "Rust", "tags": ["lang", "rust"]}`. Every error body is `{"error": "a message"}`. The validation rules and exact messages are in `src/model.rs`; the store's behavior (ids, filters, pagination) is in `src/store.rs`.

```bash
cd projects/08-web-api
cargo run      # then, in another terminal:
curl -i -X POST localhost:3000/bookmarks -H 'content-type: application/json' \
     -d '{"url": "https://www.rust-lang.org", "title": "Rust", "tags": ["lang"]}'
curl -i 'localhost:3000/bookmarks?tag=lang'
```

## Background

### Routing

```rust
Router::new()
    .route("/bookmarks", get(list).post(create))
    .route("/bookmarks/{id}", get(read).patch(update).delete(remove))
    .fallback(no_route)
    .with_state(store)
```

axum 0.8 writes path parameters in braces, `/{id}`. Older tutorials use `/:id`, which axum 0.8 rejects with a panic when the router is built. A path that exists but with the wrong method gets 405 automatically; `fallback` handles paths that match nothing.

### Handlers and extractors

A handler is an `async fn` whose parameters are **extractors** and whose return value implements `IntoResponse`:

```rust
async fn read(State(store): State<SharedStore>, Path(id): Path<u64>) -> Result<Json<Bookmark>, AppError>
```

`State` gives the shared state, `Path` the `{id}`, `Query` the query string, `Json` the body (it must come **last**, because the body can only be read once). Each is deserialized with serde into your types: an `Option` field is optional, and `#[serde(default)]` fills in a default. When extraction fails, axum answers by itself with a **rejection**: plain text with 400, 415 or 422. To send your own JSON instead, take `Result<Json<T>, JsonRejection>` and convert the error with `?`.

### Status codes are part of the API

Clients, proxies and monitoring all act on the status code, so each one has to mean something: **201 Created** with a `Location` header for a new resource, **204 No Content** for a delete, **404** for a missing resource, **409 Conflict** when the request clashes with existing data, **422 Unprocessable Entity** for well-formed JSON that breaks a rule, **400** for a malformed request. A 500 means *your* bug; it should never reveal internal details.

### Errors as values

Handlers return `Result<T, AppError>`. `AppError` is an enum with a variant per kind of failure and one `IntoResponse` impl that maps each to a status code and a JSON body. Store methods return the same type, so a handler is a few lines of `?`:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self { /* one arm per variant */ };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### Shared state

`with_state(store)` hands every handler a clone of an `Arc<RwLock<Store>>`: many readers at once, or one writer. A `std::sync::RwLock` is fine because the store is plain synchronous code: a handler locks, calls one method, unlocks, and never holds the lock across an `.await`. Keeping the store free of HTTP details also means project 09 can swap it for a database.

### Testing without a port

A `Router` is a tower `Service`: give it a request, get a response. `app.oneshot(request).await` runs the whole stack (routing, extractors, handlers, errors) in-process. There's no port to collide with and no network to be flaky, so hundreds of tests run in parallel in milliseconds. Read the helpers at the top of `tests/api.rs`.

## Your tasks

1. **Errors** (`src/error.rs`): `IntoResponse` for `AppError`.
2. **Create and read** (`src/routes.rs`): the `/bookmarks` and `/bookmarks/{id}` routes with `create` and `read` handlers.
3. **Validation and storage:** `NewBookmark::validate` (`src/model.rs`), then `Store::insert` and `Store::get` (`src/store.rs`).
4. **Rejections:** `From<JsonRejection>`, `From<PathRejection>` and `From<QueryRejection>` for `AppError`; take `Result<Extractor, Rejection>` in the handlers.
5. **Update and delete:** `PATCH` and `DELETE`, with `Store::update` and `Store::delete`.
6. **List:** `GET /bookmarks` with `Store::list` (filters and pagination).
7. **Fallback:** a JSON 404 for unknown routes.
8. Grade (`bash ../../tools/grade.sh 08-web-api`), then commit and push.

Run one test while you work: `cargo test create_returns`.

## What the grader checks

- `cargo fmt` and `cargo clippy` are clean; `tests/api.rs` is unchanged.
- `AppError` implements `IntoResponse`, and the routes use axum 0.8's `/{id}` syntax.
- Every endpoint test passes: statuses, headers, bodies, validation messages, filters, pagination, and 50 concurrent requests.

## Hints

- A tuple is a response too: `(StatusCode::CREATED, [(header::LOCATION, location)], Json(bookmark))`.
- `PATCH` is "merge, then validate like a new bookmark": build a `NewBookmark` from the patch fields or the current values, and call `validate`.
- A url is a conflict only if *another* bookmark has it: patching a bookmark with its own url is fine.
- `Option::is_none_or` makes optional filters one line each: `params.tag.as_ref().is_none_or(|t| b.tags.contains(t))`.
- Lock the store in its own statement (`let page = store.read().unwrap().list(&params)?;`), so the guard is dropped right away.

## Going further

- Add `tower-http` middleware: `TraceLayer` for request logs, `TimeoutLayer`, `CorsLayer`, a request body size limit.
- Generate OpenAPI documentation with `utoipa`.
- Replace offset pagination with cursor pagination (`?after=25`), which stays correct while items are added.
- Persist the bookmarks in SQLite: that's project 09.
