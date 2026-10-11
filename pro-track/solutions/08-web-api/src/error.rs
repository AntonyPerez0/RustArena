//! One error type for the whole API. Handlers return `Result<_, AppError>`, so
//! `?` works everywhere, and every error becomes a proper status code with a
//! JSON body like `{"error": "bookmark 7 not found"}`.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    /// 404 Not Found.
    NotFound(String),
    /// 400 Bad Request: the request itself is malformed (a bad query parameter...).
    BadRequest(String),
    /// 409 Conflict: the request clashes with existing data (a duplicate url).
    Conflict(String),
    /// 422 Unprocessable Entity: well-formed JSON that breaks a rule.
    Validation(String),
    /// An axum extractor rejected the request (invalid JSON, a missing
    /// content-type, an id that isn't a number...). Keeps axum's status code.
    Rejected(StatusCode, String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::Validation(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            AppError::Rejected(status, message) => (status, message),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// These let `?` turn axum's rejections into AppError, keeping axum's status
// code and message but sending them as JSON.

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::Rejected(rejection.status(), rejection.body_text())
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::Rejected(rejection.status(), rejection.body_text())
    }
}

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        AppError::Rejected(rejection.status(), rejection.body_text())
    }
}
