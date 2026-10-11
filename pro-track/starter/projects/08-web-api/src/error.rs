//! One error type for the whole API. Handlers return `Result<_, AppError>`, so
//! `?` works everywhere, and every error becomes a proper status code with a
//! JSON body like `{"error": "bookmark 7 not found"}`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

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

// TODO (task 1): turn every AppError into its status code and a JSON body
// {"error": "<the message>"}. `(status, Json(value)).into_response()` builds one.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        todo!("respond to {self:?} (task 1)")
    }
}

// TODO (task 4): `From` impls so `?` converts axum's rejections into
// AppError::Rejected(rejection.status(), rejection.body_text()):
// axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection}.
