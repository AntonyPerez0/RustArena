//! A JSON REST API for bookmarks, built with axum. The endpoints are listed in
//! the README.
//!
//! - `model`: the JSON shapes and the validation rules
//! - `store`: the in-memory data, behind `Arc<RwLock<..>>`
//! - `error`: `AppError`, which turns into a status code and a JSON body
//! - `routes`: the router and the handlers

pub mod error;
pub mod model;
pub mod routes;
pub mod store;

pub use error::AppError;
pub use routes::{app, router};
pub use store::{SharedStore, Store};
