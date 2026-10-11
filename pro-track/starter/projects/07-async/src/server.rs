use std::future::Future;
use std::io;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::protocol::Command;

/// Serves chat clients on `listener` until `shutdown` completes, then tells
/// every client, waits for all connections to close, and returns.
///
/// This first version serves exactly one client at a time, never stops, and
/// only knows WHO and QUIT. The README walks you through the rest:
///
/// 1. Serve every client at once: a task per connection.
/// 2. Shared state: the names in use. NICK and WHO, and free a name when its
///    client leaves.
/// 3. A broadcast channel for JOIN, MSG and LEAVE: each connection `select!`s
///    between its client's next line and the next event.
/// 4. Graceful shutdown: stop accepting when `shutdown` completes, write
///    SHUTDOWN to every client, and wait for every connection task.
/// 5. Per-connection errors: lines longer than MAX_LINE, failed writes and
///    disconnects end that connection only, never the server.
pub async fn run(listener: TcpListener, shutdown: impl Future<Output = ()>) -> io::Result<()> {
    let _ = shutdown; // TODO (task 4): stop when this completes.
    loop {
        let (socket, _addr) = listener.accept().await?;
        // TODO (task 1): this waits for the client to leave before accepting
        // the next one, so a second client gets no answer at all.
        serve(socket).await;
    }
}

/// Talks to one client until it leaves.
async fn serve(socket: TcpStream) {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let reply = match Command::parse(&line) {
            // TODO (task 2): list the names in use.
            Command::Who => "USERS\n".to_string(),
            Command::Quit => {
                let _ = writer.write_all(b"BYE\n").await;
                return;
            }
            // TODO (tasks 2 and 3): NICK and MSG.
            _ => "ERR unknown command\n".to_string(),
        };
        if writer.write_all(reply.as_bytes()).await.is_err() {
            return;
        }
    }
}
