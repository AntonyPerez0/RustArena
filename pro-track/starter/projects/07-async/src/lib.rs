//! A line-based chat server on Tokio. The protocol is described in the README.
//!
//! The library exposes [`run`], which serves clients on a listener you pass in.
//! The binary (`src/main.rs`) binds a port and stops on Ctrl+C; the tests bind
//! port 0 and stop the server with a oneshot channel.

mod protocol;
mod server;

pub use protocol::{Command, MAX_LINE, valid_name};
pub use server::run;
