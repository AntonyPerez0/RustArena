use crate::Index;

// Part 2: threads that share nothing and send their results over a channel.
//
// TODO: index `docs` with `threads` threads (0 means 1) and return the same
// Index that crate::index_serial would.
// - Use std::thread::scope: threads spawned in a scope may borrow `docs`
//   (no Arc needed), because the scope waits for all of them before it returns.
// - Give each thread one contiguous slice of the documents (`docs.chunks(n)`).
//   It builds one Index for its slice and sends it through a
//   std::sync::mpsc::Sender. Each thread needs its own clone of the Sender.
// - This thread receives and merges: `for part in rx { total.merge(part) }`.
//   That loop ends only when every Sender is gone, so drop the original one
//   after spawning, or this function deadlocks.
pub fn index_channels(docs: &[String], threads: usize) -> Index {
    todo!("part 2: index {} documents with {threads} threads", docs.len())
}
