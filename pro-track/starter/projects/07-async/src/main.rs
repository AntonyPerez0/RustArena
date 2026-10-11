//! Runs the chat server until Ctrl+C.
//!
//!     cargo run                    # listens on 127.0.0.1:7878
//!     cargo run -- 0.0.0.0:9000    # or anywhere you like
//!
//! Then, in other terminals: nc 127.0.0.1 7878

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7878".to_string());
    let listener = TcpListener::bind(&addr).await?;
    println!(
        "chat server listening on {} (Ctrl+C to stop)",
        listener.local_addr()?
    );
    chat::run(listener, async {
        let _ = tokio::signal::ctrl_c().await;
        println!("shutting down...");
    })
    .await?;
    println!("bye");
    Ok(())
}
