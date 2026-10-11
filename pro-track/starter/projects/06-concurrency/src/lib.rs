//! A log indexer, built four ways: single-threaded, with threads sharing a
//! `Mutex`, with threads sending results over a channel, and with rayon.
//! All four must give exactly the same [`Index`].
//!
//! Rust checks thread safety at compile time. An `Rc` (a reference count that
//! is not atomic) can't be sent to another thread, so this doesn't compile:
//!
//! ```compile_fail
//! let shared = std::rc::Rc::new(vec![1, 2, 3]);
//! std::thread::spawn(move || println!("{shared:?}"));
//! ```
//!
//! With `Arc` (an atomic reference count) it does:
//!
//! ```
//! let shared = std::sync::Arc::new(vec![1, 2, 3]);
//! std::thread::spawn(move || println!("{shared:?}")).join().unwrap();
//! ```
//!
//! Don't edit this file: the grader checks that it is unchanged.

mod channels;
mod generate;
mod index;
mod shared;
mod with_rayon;

pub use channels::index_channels;
pub use generate::generate;
pub use index::Index;
pub use shared::index_shared;
pub use with_rayon::index_rayon;

/// The single-threaded reference: every parallel version must return exactly this.
pub fn index_serial(docs: &[String]) -> Index {
    let mut index = Index::default();
    for doc in docs {
        index.add_document(doc);
    }
    index
}
