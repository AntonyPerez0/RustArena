//! The service's error type, and the HTTP response for each kind of error.
//!
//! TODO: an `AppError` enum (invalid input, not found, code taken, database error) that
//! implements `axum::response::IntoResponse`: the right status code and a JSON body like
//! `{"error": "the code \"rust\" is already taken"}`. Database errors are logged with
//! `tracing::error!` and answered with a 500 and a generic message: internal details
//! (SQL, file paths) never go to clients.
