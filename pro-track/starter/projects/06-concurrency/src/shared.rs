use std::sync::Arc;

use crate::Index;

// Part 1: threads that share state behind a lock.
//
// TODO: index `docs` with `threads` threads (0 means 1) and return the same
// Index that crate::index_serial would.
// - Start them with std::thread::spawn. Its closure must be 'static, because the
//   thread could outlive this function. That's why `docs` arrives in an Arc:
//   give each thread its own clone (a cheap pointer copy, not a copy of the logs).
// - Hand out work with a shared std::sync::atomic::AtomicUsize holding the index
//   of the next document: each thread loops on `fetch_add(1, ...)` until it runs
//   past the end. Fast threads simply take more documents.
// - Each thread indexes a document on its own (Index::of_document), then locks
//   an Arc<Mutex<Index>> just long enough to merge the result. Lock once per
//   document, never once per line, or the threads spend their time waiting.
// - join() every thread, then take the Index out:
//   Arc::try_unwrap(total) and then Mutex::into_inner.
pub fn index_shared(docs: Arc<Vec<String>>, threads: usize) -> Index {
    todo!("part 1: index {} documents with {threads} threads", docs.len())
}
