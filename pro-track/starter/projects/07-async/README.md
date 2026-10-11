# 07 · Async networking with Tokio

A network server spends nearly all its time waiting: for a client to send something, for a slow client to read. A thread per connection works until you have ten thousand connections and ten thousand mostly idle threads. **Async** Rust lets a handful of threads juggle any number of connections: a task that would wait gives its thread back to the runtime instead. **Tokio** is the runtime most Rust services are built on (axum, tonic, the AWS SDK...).

You'll build a chat server: many clients at once, shared state, messages broadcast to everyone, and a clean shutdown. The tests start it in-process and talk to it over real TCP connections.

**You'll practice:** `async`/`.await`, `tokio::spawn` and `JoinSet`, `Arc<Mutex>` in async code, broadcast and watch channels, `select!`, graceful shutdown, and per-connection error handling.

## The protocol

One command per line. Replies and events are lines too.

| Client sends | Server sends |
|---|---|
| `NICK <name>` | `OK welcome <name>`, then every joined client (you too) gets `JOIN <name>`. Or `ERR invalid name` (1 to 16 letters, digits, `_` or `-`), `ERR name taken`, `ERR already joined` |
| `MSG <text>` | every joined client (you too) gets `MSG <name> <text>`. Or `ERR send NICK first`, `ERR empty message` |
| `WHO` | `USERS alice bob` (sorted; works before joining) |
| `QUIT` | `BYE`, then the server closes the connection |
| anything else | `ERR unknown command` |

- Clients receive JOIN, MSG and LEAVE events only after they have joined.
- When a joined client leaves (QUIT or disconnect), its name is freed and everyone else gets `LEAVE <name>`.
- A line longer than 1024 bytes (`MAX_LINE`) gets `ERR line too long` and the connection is closed.
- On shutdown, every client gets `SHUTDOWN` and its connection is closed.

Try it: `cargo run` in one terminal, then `nc 127.0.0.1 7878` in two more. The starter answers the first client only. The second gets nothing until the first leaves.

## Background

### Futures, tasks and the runtime

An `async fn` returns a **future**: a value that does nothing until it's awaited. `#[tokio::main]` starts the runtime, which polls futures; at each `.await` that can't finish yet, the task steps aside and the thread runs another one. That only works if tasks actually step aside, so **never block in async code**: no `std::thread::sleep`, blocking file or network calls, or long CPU loops. Use `tokio::time::sleep`, `tokio::fs`, or `tokio::task::spawn_blocking`.

`tokio::spawn(future)` starts a **task**: a few hundred bytes, not a thread, so a task per connection is normal. The future must be `Send + 'static` because it may move between worker threads, so share data through an `Arc`. A `JoinSet` holds many tasks and lets you wait for all of them.

### Shared state

A `std::sync::Mutex` inside an `Arc` is fine in async code as long as you **never hold the guard across an `.await`**: lock, update, unlock. Holding it across an await could block a worker thread, and makes the future non-`Send`, so `tokio::spawn` refuses it (clippy's `await_holding_lock` warns too). The alternative is an **actor**: one task owns the state, and the others send it requests over an `mpsc` channel.

### Tokio's channels

| Channel | Use it for |
|---|---|
| `mpsc` | many senders, one receiver: a work queue |
| `oneshot` | one value, once: "here's your reply", "stop now" |
| `broadcast` | every receiver gets every message: chat events. A receiver that falls behind gets `RecvError::Lagged(n)` |
| `watch` | only the latest value: "we are shutting down" |

### select!

```rust
loop {
    tokio::select! {
        line = lines.next_line() => { /* the client sent something */ }
        event = events.recv() => { /* someone else did something */ }
        _ = stop.changed() => { /* the server is shutting down */ }
    }
}
```

`select!` waits for several futures and runs the branch of the first one to finish. The others are **dropped**, so each must be *cancellation safe*: dropping it mid-way must not lose data. `TcpListener::accept`, `Lines::next_line` and `broadcast::Receiver::recv` are; the docs of every Tokio method say so. `read_line` into a buffer you clear each time is not.

### Graceful shutdown

Deployments stop services by sending a signal and waiting a little (Kubernetes sends SIGTERM, then kills after 30 s). A clean stop is: stop accepting, tell every connection, let each finish what it's doing, wait for all tasks, return. Here `run` takes the trigger as a parameter, `shutdown: impl Future`: the binary passes `tokio::signal::ctrl_c()`, the tests pass a oneshot receiver. Taking the listener as a parameter lets tests bind port 0 (any free port) and run in parallel.

### Errors belong to one connection

A client that sends garbage, floods you or vanishes mid-write must cost you that connection, never the server. No `unwrap()` on network I/O: a failed write means the client is gone, so clean up and end the task.

## Your tasks

1. Run the starter with two `nc` clients and see the second hang.
2. **A task per connection** (`src/server.rs`): spawn `serve` into a `JoinSet` instead of awaiting it.
3. **Shared names:** an `Arc` holding a `Mutex<HashSet<String>>`. Implement `NICK` and `WHO`, and free the name when a client leaves.
4. **Events:** a `broadcast` channel for JOIN, MSG and LEAVE. Subscribe when a client joins, and `select!` between its next line and the next event.
5. **Shutdown:** `select!` between `accept()` and `shutdown` in `run`; on shutdown, drop the listener, signal the connections (a `watch` channel), and wait for the `JoinSet` to empty.
6. **Errors:** `ERR line too long`, failed writes, disconnects.
7. Grade (`bash ../../tools/grade.sh 07-async`), then commit and push.

## What the grader checks

- `cargo fmt` and `cargo clippy` are clean; `tests/chat.rs` is unchanged.
- All tests pass, three runs in a row (timing bugs often pass once).
- `src/server.rs` spawns tasks, uses `select!` and a `broadcast` channel; no blocking calls in `src/`.

## Hints

- `run` gets `shutdown` by value. To use it in a `select!` inside a loop, pin it first: `let mut shutdown = std::pin::pin!(shutdown);` then `() = &mut shutdown => break,`.
- Keep the command handling in a plain (non-async) function that returns the reply. Easy to test, and the lock can't be held across an `.await`.
- A client that hasn't joined has no event receiver. A helper that returns `std::future::pending()` when there's no receiver keeps the `select!` simple.
- Free the name *before* broadcasting LEAVE, so a client reacting to the LEAVE can take the name.
- Collect finished tasks in the accept loop (`connections.join_next()`), or the `JoinSet` grows forever.

## Going further

- Replace the `Mutex` with an actor task that owns the names and answers over `oneshot` channels.
- `next_line` buffers a whole line before you can check its length. `tokio_util::codec::LinesCodec::new_with_max_length` stops at the limit instead. Try it.
- Add rooms (`JOIN #rust`) and private messages (`MSG @bob hi`).
- Write a load generator that connects 1,000 clients and measures messages per second.
- Watch your tasks live with [tokio-console](https://github.com/tokio-rs/console).
