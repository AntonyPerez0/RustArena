use std::sync::mpsc;
use std::thread;

use crate::Index;

/// Part 2: threads that share nothing and send their results over a channel.
///
/// Scoped threads borrow `docs` directly. Each one indexes a contiguous slice
/// and sends one Index; this thread merges them as they arrive.
pub fn index_channels(docs: &[String], threads: usize) -> Index {
    let per_thread = docs.len().div_ceil(threads.max(1)).max(1);
    let (tx, rx) = mpsc::channel();
    thread::scope(|scope| {
        for slice in docs.chunks(per_thread) {
            let tx = tx.clone();
            scope.spawn(move || {
                let mut index = Index::default();
                for doc in slice {
                    index.add_document(doc);
                }
                tx.send(index).expect("the receiver lives until every worker is done");
            });
        }
        // The loop below ends when every Sender is dropped. The workers drop
        // their clones when they finish; this one is ours.
        drop(tx);
        let mut total = Index::default();
        for part in rx {
            total.merge(part);
        }
        total
    })
}
