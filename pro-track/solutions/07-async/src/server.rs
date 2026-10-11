use std::collections::HashSet;
use std::future::Future;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::protocol::{Command, MAX_LINE, valid_name};

/// State shared by every connection task.
struct Shared {
    /// The names in use. A std Mutex is right here: it is only held for a quick
    /// insert, remove or copy, never across an `.await`.
    names: Mutex<HashSet<String>>,
    /// JOIN, MSG and LEAVE events, already formatted as lines, for every joined client.
    events: broadcast::Sender<String>,
}

/// Serves chat clients on `listener` until `shutdown` completes, then tells
/// every client, waits for all connections to close, and returns.
pub async fn run(listener: TcpListener, shutdown: impl Future<Output = ()>) -> io::Result<()> {
    let (events, _) = broadcast::channel(1024);
    let shared = Arc::new(Shared {
        names: Mutex::default(),
        events,
    });
    // Every connection watches this; it flips to true on shutdown.
    let (stop_tx, stop_rx) = watch::channel(false);
    let mut connections = JoinSet::new();
    let mut shutdown = std::pin::pin!(shutdown);

    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((socket, _addr)) => {
                    connections.spawn(serve(socket, Arc::clone(&shared), stop_rx.clone()));
                }
                Err(err) => {
                    // For example "too many open files": keep serving the
                    // clients we have, and don't spin while the error lasts.
                    eprintln!("accept failed: {err}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
            // Collect finished tasks so the JoinSet doesn't grow forever.
            Some(finished) = connections.join_next(), if !connections.is_empty() => {
                if let Err(err) = finished {
                    eprintln!("a connection task failed: {err}");
                }
            }
            () = &mut shutdown => break,
        }
    }

    // Stop accepting, tell every connection, and wait for all of them.
    drop(listener);
    stop_tx.send_replace(true);
    while connections.join_next().await.is_some() {}
    Ok(())
}

/// One client's state, owned by its connection task.
#[derive(Default)]
struct Session {
    name: Option<String>,
    /// Subscribed when the client joins: before that it sees no events.
    events: Option<broadcast::Receiver<String>>,
}

/// What the connection should do after a command.
enum Outcome {
    Reply(String),
    Nothing,
    Close(&'static str),
}

/// Talks to one client until it leaves, misbehaves or the server shuts down.
async fn serve(socket: TcpStream, shared: Arc<Shared>, mut stop: watch::Receiver<bool>) {
    let (reader, mut writer) = socket.into_split();
    // next_line() is cancellation safe: if another select! branch wins, no
    // half-read line is lost.
    let mut lines = BufReader::new(reader).lines();
    let mut session = Session::default();

    loop {
        let out = tokio::select! {
            line = lines.next_line() => match line {
                Ok(Some(line)) if line.len() > MAX_LINE => Outcome::Close("ERR line too long\n"),
                Ok(Some(line)) => handle(&line, &mut session, &shared),
                // End of stream or a read error (reset, invalid UTF-8): just leave.
                Ok(None) | Err(_) => break,
            },
            event = next_event(&mut session.events) => match event {
                Ok(line) => Outcome::Reply(line),
                Err(RecvError::Lagged(missed)) => {
                    Outcome::Reply(format!("ERR too slow, missed {missed} messages\n"))
                }
                Err(RecvError::Closed) => break,
            },
            _ = stop.changed() => Outcome::Close("SHUTDOWN\n"),
        };
        let (text, close) = match out {
            Outcome::Reply(text) => (text, false),
            Outcome::Nothing => continue,
            Outcome::Close(text) => (text.to_string(), true),
        };
        if writer.write_all(text.as_bytes()).await.is_err() || close {
            break;
        }
    }

    // Free the name first, then announce, so the name is available to anyone
    // who reacts to the LEAVE.
    if let Some(name) = session.name {
        shared.names.lock().unwrap().remove(&name);
        let _ = shared.events.send(format!("LEAVE {name}\n"));
    }
}

/// The next event for a joined client; never completes for one that hasn't joined.
async fn next_event(events: &mut Option<broadcast::Receiver<String>>) -> Result<String, RecvError> {
    match events {
        Some(events) => events.recv().await,
        None => std::future::pending().await,
    }
}

/// Handles one command. Synchronous: no `.await` while the names lock is held.
fn handle(line: &str, session: &mut Session, shared: &Shared) -> Outcome {
    let reply = |text: &str| Outcome::Reply(format!("{text}\n"));
    match Command::parse(line) {
        Command::Nick(_) if session.name.is_some() => reply("ERR already joined"),
        Command::Nick(name) => {
            if !valid_name(&name) {
                return reply("ERR invalid name");
            }
            if !shared.names.lock().unwrap().insert(name.clone()) {
                return reply("ERR name taken");
            }
            // Subscribe before announcing, so this client sees its own JOIN.
            session.events = Some(shared.events.subscribe());
            let _ = shared.events.send(format!("JOIN {name}\n"));
            let welcome = reply(&format!("OK welcome {name}"));
            session.name = Some(name);
            welcome
        }
        Command::Msg(text) => {
            let Some(name) = &session.name else {
                return reply("ERR send NICK first");
            };
            if text.trim().is_empty() {
                return reply("ERR empty message");
            }
            let _ = shared.events.send(format!("MSG {name} {text}\n"));
            Outcome::Nothing
        }
        Command::Who => {
            let mut names: Vec<String> = shared.names.lock().unwrap().iter().cloned().collect();
            names.sort();
            let mut text = String::from("USERS");
            for name in names {
                text.push(' ');
                text.push_str(&name);
            }
            reply(&text)
        }
        Command::Quit => Outcome::Close("BYE\n"),
        Command::Unknown => reply("ERR unknown command"),
    }
}
