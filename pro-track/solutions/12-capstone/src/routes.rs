//! HTTP handlers and the router.

use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::AppState;
use crate::error::AppError;
use crate::links::{self, Link};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/links", post(create_link))
        .route("/api/links/{code}", get(link_details).delete(delete_link))
        .route("/{code}", get(follow))
        // One span per request (method, path), and one event per response (status, latency).
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct CreateLink {
    url: String,
    code: Option<String>,
}

#[derive(Debug, Serialize)]
struct LinkBody {
    code: String,
    url: String,
    short_url: String,
    hits: i64,
    created_at: String,
}

impl LinkBody {
    fn new(state: &AppState, link: Link) -> Self {
        LinkBody {
            short_url: format!("{}/{}", state.public_url, link.code),
            code: link.code,
            url: link.url,
            hits: link.hits,
            created_at: link.created_at,
        }
    }
}

/// Liveness and readiness in one: the process answers and the database works.
async fn health(State(state): State<AppState>) -> Response {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => Json(json!({ "status": "ok" })).into_response(),
        Err(err) => {
            tracing::error!(error = %err, "health check failed");
            let body = Json(json!({ "status": "unavailable" }));
            (StatusCode::SERVICE_UNAVAILABLE, body).into_response()
        }
    }
}

async fn create_link(
    State(state): State<AppState>,
    Json(input): Json<CreateLink>,
) -> Result<Response, AppError> {
    let url = links::validate_url(&input.url)?;
    let link = match input.code {
        Some(code) => {
            links::validate_code(&code)?;
            links::insert(&state.pool, &code, &url).await?
        }
        None => links::insert_with_random_code(&state.pool, &url).await?,
    };
    tracing::info!(code = %link.code, url = %link.url, "link created");
    let location = format!("/api/links/{}", link.code);
    let body = Json(LinkBody::new(&state, link));
    Ok((StatusCode::CREATED, [(header::LOCATION, location)], body).into_response())
}

async fn link_details(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<LinkBody>, AppError> {
    match links::find(&state.pool, &code).await? {
        Some(link) => Ok(Json(LinkBody::new(&state, link))),
        None => Err(AppError::NotFound(code)),
    }
}

async fn delete_link(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    if links::delete(&state.pool, &code).await? {
        tracing::info!(%code, "link deleted");
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(code))
    }
}

/// The short link itself: count the visit and redirect. 307 (temporary) rather than 301
/// (permanent), because browsers cache permanent redirects forever: later visits would
/// skip us, so they wouldn't be counted, and a deleted link would keep working.
async fn follow(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Redirect, AppError> {
    match links::record_hit(&state.pool, &code).await? {
        Some(url) => Ok(Redirect::temporary(&url)),
        None => Err(AppError::NotFound(code)),
    }
}
