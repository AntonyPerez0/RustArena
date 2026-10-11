//! The service's error type, and the HTTP response for each kind of error.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The request is well-formed JSON but asks for something invalid: 422.
    #[error("{0}")]
    Invalid(String),
    /// No link with this code: 404.
    #[error("no link with code {0:?}")]
    NotFound(String),
    /// The custom code is already in use: 409.
    #[error("the code {0:?} is already taken")]
    CodeTaken(String),
    /// Something broke on our side: 500. The details go to the logs, not to the client.
    #[error("internal error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::CodeTaken(_) => StatusCode::CONFLICT,
            AppError::Database(err) => {
                tracing::error!(error = %err, "database error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
