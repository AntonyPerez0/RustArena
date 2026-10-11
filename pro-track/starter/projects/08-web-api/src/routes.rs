//! The router: which handler answers which method and path.

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::store::SharedStore;

/// The app with a fresh, empty store.
pub fn app() -> Router {
    router(SharedStore::default())
}

/// The app on a store you provide (handy for tests that prepare data).
///
/// TODO: add the routes (tasks 2 to 7). axum 0.8 writes path parameters in
/// braces: "/bookmarks/{id}".
///   GET    /bookmarks        list    (query: tag, q, limit, offset)
///   POST   /bookmarks        create  (201 Created + Location header)
///   GET    /bookmarks/{id}   read
///   PATCH  /bookmarks/{id}   update
///   DELETE /bookmarks/{id}   delete  (204 No Content)
///   anything else            404 with a JSON error (Router::fallback)
pub fn router(store: SharedStore) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(store)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
