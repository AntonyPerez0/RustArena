//! The router: which handler answers which method and path.

use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::{Method, StatusCode, Uri, header};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::AppError;
use crate::model::{Bookmark, BookmarkPatch, ListParams, NewBookmark, Page};
use crate::store::SharedStore;

/// The app with a fresh, empty store.
pub fn app() -> Router {
    router(SharedStore::default())
}

/// The app on a store you provide (handy for tests that prepare data).
pub fn router(store: SharedStore) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/bookmarks", get(list).post(create))
        .route("/bookmarks/{id}", get(read).patch(update).delete(remove))
        .fallback(no_route)
        .with_state(store)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// Extractors run in order, and the one that reads the body (Json) must be last.
// Taking `Result<Json<T>, JsonRejection>` instead of `Json<T>` lets us turn
// axum's plain-text rejections into our JSON errors with `?`.

async fn create(
    State(store): State<SharedStore>,
    body: Result<Json<NewBookmark>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    let Json(new) = body?;
    // The lock guard is a temporary: it is released at the end of this statement.
    let bookmark = store.write().unwrap().insert(new)?;
    let location = format!("/bookmarks/{}", bookmark.id);
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(bookmark),
    ))
}

async fn list(
    State(store): State<SharedStore>,
    params: Result<Query<ListParams>, QueryRejection>,
) -> Result<Json<Page>, AppError> {
    let Query(params) = params?;
    let page = store.read().unwrap().list(&params)?;
    Ok(Json(page))
}

async fn read(
    State(store): State<SharedStore>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<Json<Bookmark>, AppError> {
    let Path(id) = id?;
    let bookmark = store.read().unwrap().get(id)?;
    Ok(Json(bookmark))
}

async fn update(
    State(store): State<SharedStore>,
    id: Result<Path<u64>, PathRejection>,
    body: Result<Json<BookmarkPatch>, JsonRejection>,
) -> Result<Json<Bookmark>, AppError> {
    let Path(id) = id?;
    let Json(patch) = body?;
    let bookmark = store.write().unwrap().update(id, patch)?;
    Ok(Json(bookmark))
}

async fn remove(
    State(store): State<SharedStore>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<StatusCode, AppError> {
    let Path(id) = id?;
    store.write().unwrap().delete(id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn no_route(method: Method, uri: Uri) -> AppError {
    AppError::NotFound(format!("no route for {method} {}", uri.path()))
}
