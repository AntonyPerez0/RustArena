//! Integration tests: each test starts the server in-process on a free port and
//! talks to it over real TCP connections, like `nc` would.
//! Don't edit this file: the grader checks that it is unchanged.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;

/// How long to wait for any reply. Generous, so slow CI machines don't fail.
const WAIT: Duration = Duration::from_secs(5);

struct Server {
    addr: SocketAddr,
    stop: oneshot::Sender<()>,
    task: JoinHandle<std::io::Result<()>>,
}

impl Server {
    async fn start() -> Server {
        // Port 0: the OS picks a free port, so tests can run in parallel.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (stop, stopped) = oneshot::channel::<()>();
        let task = tokio::spawn(chat::run(listener, async move {
            let _ = stopped.await;
        }));
        Server { addr, stop, task }
    }

    async fn connect(&self) -> Client {
        let stream = timeout(WAIT, TcpStream::connect(self.addr))
            .await
            .expect("connecting took more than 5 s")
            .expect("could not connect to the server");
        let (reader, writer) = stream.into_split();
        Client {
            lines: BufReader::new(reader).lines(),
            writer,
        }
    }

    /// Connects and joins as `name`.
    async fn join(&self, name: &str) -> Client {
        let mut client = self.connect().await;
        client.send(&format!("NICK {name}")).await;
        client.expect(&format!("OK welcome {name}")).await;
        client.skip_until(&format!("JOIN {name}")).await;
        client
    }

    /// Triggers the shutdown and waits for `run` to return.
    async fn shutdown(self) {
        let _ = self.stop.send(());
        match timeout(WAIT, self.task).await {
            Ok(Ok(result)) => result.expect("run returned an error"),
            Ok(Err(err)) => panic!("run panicked: {err}"),
            Err(_) => panic!("run did not return within 5 s of the shutdown signal"),
        }
    }
}

struct Client {
    lines: Lines<BufReader<OwnedReadHalf>>,
    writer: OwnedWriteHalf,
}

impl Client {
    async fn send(&mut self, line: &str) {
        self.writer
            .write_all(format!("{line}\n").as_bytes())
            .await
            .expect("sending to the server failed");
    }

    async fn recv(&mut self) -> String {
        match timeout(WAIT, self.lines.next_line()).await {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => panic!("the server closed the connection"),
            Ok(Err(err)) => panic!("reading from the server failed: {err}"),
            Err(_) => panic!("no reply from the server within 5 s"),
        }
    }

    async fn expect(&mut self, expected: &str) {
        assert_eq!(self.recv().await, expected);
    }

    /// Skips other lines (events caused by other clients) until `expected` arrives.
    async fn skip_until(&mut self, expected: &str) {
        while self.recv().await != expected {}
    }

    async fn expect_closed(&mut self) {
        match timeout(WAIT, self.lines.next_line()).await {
            Ok(Ok(None)) | Ok(Err(_)) => {}
            Ok(Ok(Some(line))) => panic!("expected the connection to close, got {line:?}"),
            Err(_) => panic!("the server did not close the connection within 5 s"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_can_join() {
    let server = Server::start().await;
    let mut alice = server.connect().await;
    alice.send("NICK alice").await;
    alice.expect("OK welcome alice").await;
    alice.expect("JOIN alice").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn who_lists_the_names_in_use() {
    let server = Server::start().await;
    let mut lurker = server.connect().await;
    lurker.send("WHO").await;
    lurker.expect("USERS").await;
    let _bob = server.join("bob").await;
    let _alice = server.join("alice").await;
    lurker.send("WHO").await;
    lurker.expect("USERS alice bob").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn messages_reach_everyone_including_the_sender() {
    let server = Server::start().await;
    let mut alice = server.join("alice").await;
    let mut bob = server.join("bob").await;
    alice.expect("JOIN bob").await;
    alice.send("MSG hello, world").await;
    alice.expect("MSG alice hello, world").await;
    bob.expect("MSG alice hello, world").await;
    bob.send("MSG hi alice").await;
    bob.expect("MSG bob hi alice").await;
    alice.expect("MSG bob hi alice").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn messages_arrive_in_order() {
    let server = Server::start().await;
    let mut alice = server.join("alice").await;
    let mut bob = server.join("bob").await;
    for i in 0..50 {
        alice.send(&format!("MSG number {i}")).await;
    }
    for i in 0..50 {
        bob.expect(&format!("MSG alice number {i}")).await;
    }
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clients_see_nothing_until_they_join() {
    let server = Server::start().await;
    let mut lurker = server.connect().await;
    let mut alice = server.join("alice").await;
    alice.send("MSG anyone here?").await;
    alice.expect("MSG alice anyone here?").await;
    // No JOIN or MSG lines for the lurker: the next line is the WHO reply.
    lurker.send("WHO").await;
    lurker.expect("USERS alice").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn names_must_be_valid_and_unique() {
    let server = Server::start().await;
    let _alice = server.join("alice").await;
    let mut client = server.connect().await;
    for bad in [
        "NICK",
        "NICK two words",
        "NICK seventeen-chars-x",
        "NICK émile",
        "NICK a!",
    ] {
        client.send(bad).await;
        client.expect("ERR invalid name").await;
    }
    client.send("NICK alice").await;
    client.expect("ERR name taken").await;
    client.send("NICK Bob_2-x").await;
    client.expect("OK welcome Bob_2-x").await;
    client.skip_until("JOIN Bob_2-x").await;
    client.send("NICK carol").await;
    client.expect("ERR already joined").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bad_commands_get_an_error() {
    let server = Server::start().await;
    let mut client = server.connect().await;
    client.send("MSG hi").await;
    client.expect("ERR send NICK first").await;
    for unknown in ["DANCE", "msg hi", "", "WHO cares"] {
        client.send(unknown).await;
        client.expect("ERR unknown command").await;
    }
    client.send("NICK dave").await;
    client.expect("OK welcome dave").await;
    client.expect("JOIN dave").await;
    client.send("MSG").await;
    client.expect("ERR empty message").await;
    client.send("MSG    ").await;
    client.expect("ERR empty message").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn quit_says_bye_and_frees_the_name() {
    let server = Server::start().await;
    let mut alice = server.join("alice").await;
    let mut bob = server.join("bob").await;
    alice.expect("JOIN bob").await;
    bob.send("QUIT").await;
    bob.expect("BYE").await;
    bob.expect_closed().await;
    alice.expect("LEAVE bob").await;
    let _bob_again = server.join("bob").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disconnecting_frees_the_name() {
    let server = Server::start().await;
    let mut alice = server.join("alice").await;
    let bob = server.join("bob").await;
    alice.expect("JOIN bob").await;
    drop(bob); // closes the connection without QUIT, like a crashed client
    alice.expect("LEAVE bob").await;
    let _bob_again = server.join("bob").await;
    let mut lurker = server.connect().await;
    lurker.send("WHO").await;
    lurker.expect("USERS alice bob").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_idle_client_does_not_block_others() {
    let server = Server::start().await;
    let _idle = server.connect().await; // connected, never says a word
    let _alice = server.join("alice").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serves_many_clients_at_once() {
    let server = Server::start().await;
    let mut clients = Vec::new();
    for _ in 0..20 {
        clients.push(server.connect().await);
    }
    for (i, client) in clients.iter_mut().enumerate().rev() {
        client.send(&format!("NICK user{i}")).await;
        client.expect(&format!("OK welcome user{i}")).await;
    }
    clients[0].send("MSG hi everyone").await;
    for client in &mut clients {
        client.skip_until("MSG user0 hi everyone").await;
    }
    let mut lurker = server.connect().await;
    lurker.send("WHO").await;
    let reply = lurker.recv().await;
    assert_eq!(reply.split(' ').count(), 21, "{reply}");
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn long_lines_drop_only_that_client() {
    let server = Server::start().await;
    let mut alice = server.join("alice").await;
    let mut ok = server.connect().await;
    ok.send(&format!("MSG {}", "y".repeat(chat::MAX_LINE - 4)))
        .await;
    ok.expect("ERR send NICK first").await;
    let mut flooder = server.connect().await;
    flooder.send(&"x".repeat(chat::MAX_LINE + 1)).await;
    flooder.expect("ERR line too long").await;
    flooder.expect_closed().await;
    alice.send("MSG still here").await;
    alice.expect("MSG alice still here").await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_tells_every_client_and_run_returns() {
    let server = Server::start().await;
    let mut alice = server.join("alice").await;
    let mut bob = server.join("bob").await;
    let mut lurker = server.connect().await;
    lurker.send("WHO").await;
    lurker.expect("USERS alice bob").await;
    let addr = server.addr;
    server.shutdown().await;
    for client in [&mut alice, &mut bob, &mut lurker] {
        client.skip_until("SHUTDOWN").await;
        client.expect_closed().await;
    }
    assert!(
        TcpStream::connect(addr).await.is_err(),
        "the server should stop listening when run returns"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_with_no_clients() {
    let server = Server::start().await;
    server.shutdown().await;
}
