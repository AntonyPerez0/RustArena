//! Runs the bookmarks API until Ctrl+C.
//!
//!     cargo run
//!     curl -i localhost:3000/health
//!     curl -i -X POST localhost:3000/bookmarks -H 'content-type: application/json' \
//!          -d '{"url": "https://www.rust-lang.org", "title": "Rust", "tags": ["lang"]}'
//!     curl -i 'localhost:3000/bookmarks?tag=lang'

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3000".to_string());
    let listener = TcpListener::bind(&addr).await?;
    println!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, bookmarks::app())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
}
