use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::Index;

/// Part 1: threads that share state behind a lock.
///
/// Work is handed out one document at a time through an atomic counter, so a
/// fast thread simply takes more documents. Each thread indexes a document on
/// its own and only locks the shared total to merge the result.
pub fn index_shared(docs: Arc<Vec<String>>, threads: usize) -> Index {
    let total = Arc::new(Mutex::new(Index::default()));
    let next = Arc::new(AtomicUsize::new(0));
    let workers: Vec<_> = (0..threads.max(1))
        .map(|_| {
            let docs = Arc::clone(&docs);
            let total = Arc::clone(&total);
            let next = Arc::clone(&next);
            thread::spawn(move || {
                // Relaxed is enough: the counter only has to hand out each
                // number once; the Mutex and join() order everything else.
                while let Some(doc) = docs.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let part = Index::of_document(doc);
                    // The guard is a temporary, so the lock is released at the
                    // end of this statement.
                    total.lock().expect("a worker panicked").merge(part);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("a worker panicked");
    }
    Arc::try_unwrap(total)
        .expect("every worker has finished, so this is the last Arc")
        .into_inner()
        .expect("a worker panicked while holding the lock")
}
