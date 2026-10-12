//! shortener: a URL shortener service.
//!
//! - [`config`]: settings from environment variables.
//! - [`telemetry`]: logging with `tracing`.
//! - [`db`]: the SQLite pool and migrations (provided).
//! - [`links`]: validation and the SQL for links.
//! - [`error`]: the error type, and how each error becomes an HTTP response.
//! - [`routes`]: the HTTP handlers and the router.
//!
//! The tests use the public items in this file and in `config`, so keep their names
//! and signatures. Everything else is yours to design.

pub mod config;
pub mod db;
pub mod error;
pub mod links;
pub mod routes;
pub mod telemetry;

use std::future::Future;
use std::sync::Arc;

use axum::Router;
use sqlx::SqlitePool;
use tokio::net::TcpListener;

/// What every handler can reach. Cloned for each request, so keep it cheap to clone:
/// the pool is a handle, and `Arc<str>` clones without copying the text.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    /// The service's public address without a trailing slash. A short link is
    /// `{public_url}/{code}`.
    pub public_url: Arc<str>,
}

impl AppState {
    pub fn new(pool: SqlitePool, public_url: impl Into<String>) -> Self {
        let public_url: String = public_url.into();
        AppState {
            pool,
            public_url: public_url.trim_end_matches('/').into(),
        }
    }
}

/// The whole HTTP API, ready to serve or to test with `tower::ServiceExt::oneshot`.
pub fn app(state: AppState) -> Router {
    routes::router(state)
}

/// Serves `app` on `listener` until `shutdown` completes. Then it stops accepting
/// connections, lets requests in progress finish, and returns.
pub async fn serve(
    listener: TcpListener,
    app: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

/// Completes when the process is asked to stop: Ctrl+C (SIGINT) in a terminal, or SIGTERM,
/// which is what `docker stop`, systemd and Kubernetes send.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("can't listen for Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("can't listen for SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutdown requested, finishing requests in progress");
}
