//! Every parallel version must give exactly the same answer as the serial one,
//! for any number of threads, and must finish (no deadlocks).
//! Don't edit this file: the grader checks that it is unchanged.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use logindex::{Index, generate, index_channels, index_rayon, index_serial, index_shared};

const THREAD_COUNTS: [usize; 7] = [0, 1, 2, 3, 4, 8, 64];

/// 40 small log files and their serial index, built once for all tests.
fn fixture() -> &'static (Arc<Vec<String>>, Index) {
    static FIXTURE: OnceLock<(Arc<Vec<String>>, Index)> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let docs = generate(42, 40, 300);
        let expected = index_serial(&docs);
        (Arc::new(docs), expected)
    })
}

/// Runs `f` on another thread and fails the test if it takes more than 60 s:
/// a hung test should fail with a clue, not freeze CI.
fn finishes<T: Send + 'static>(what: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(Duration::from_secs(60)) {
        Ok(value) => value,
        Err(RecvTimeoutError::Timeout) => panic!(
            "{what} did not finish within 60 s. A deadlock? Is a Sender still alive, \
             or a lock never released? (See Deadlocks in the README.)"
        ),
        Err(RecvTimeoutError::Disconnected) => panic!("{what} panicked (see the message above)"),
    }
}

#[test]
fn the_generator_is_deterministic() {
    assert_eq!(generate(1, 3, 50), generate(1, 3, 50));
    assert_ne!(generate(1, 3, 50), generate(2, 3, 50));
    let (docs, expected) = fixture();
    assert_eq!(docs.len(), 40);
    assert_eq!(expected.lines, 40 * 300);
}

#[test]
fn shared_state_version_matches_serial() {
    let (docs, expected) = fixture();
    for threads in THREAD_COUNTS {
        let docs = Arc::clone(docs);
        let got = finishes(&format!("index_shared(docs, {threads})"), move || {
            index_shared(docs, threads)
        });
        assert!(got == *expected, "index_shared with {threads} threads gave a different index");
    }
}

#[test]
fn channel_version_matches_serial() {
    let (docs, expected) = fixture();
    for threads in THREAD_COUNTS {
        let docs = Arc::clone(docs);
        let got = finishes(&format!("index_channels(&docs, {threads})"), move || {
            index_channels(&docs, threads)
        });
        assert!(got == *expected, "index_channels with {threads} threads gave a different index");
    }
}

#[test]
fn rayon_version_matches_serial() {
    let (docs, expected) = fixture();
    let docs = Arc::clone(docs);
    let got = finishes("index_rayon(&docs)", move || index_rayon(&docs));
    assert!(got == *expected, "index_rayon gave a different index");
}

#[test]
fn rayon_version_works_in_a_small_pool() {
    let (docs, expected) = fixture();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let got = pool.install(|| index_rayon(docs));
    assert!(got == *expected, "index_rayon in a 2-thread pool gave a different index");
}

#[test]
fn empty_input_gives_an_empty_index() {
    let empty: Vec<String> = Vec::new();
    assert_eq!(index_serial(&empty), Index::default());
    for threads in [1, 4] {
        assert_eq!(
            finishes("index_shared on no documents", move || index_shared(Arc::new(Vec::new()), threads)),
            Index::default()
        );
        assert_eq!(
            finishes("index_channels on no documents", move || index_channels(&[], threads)),
            Index::default()
        );
    }
    assert_eq!(index_rayon(&empty), Index::default());
}

#[test]
fn uneven_documents() {
    // Documents of very different sizes, including empty ones.
    let mut docs = generate(9, 12, 1);
    docs.extend(generate(10, 3, 2_000));
    docs.insert(5, String::new());
    docs.push(String::new());
    let expected = index_serial(&docs);
    let docs = Arc::new(docs);
    for threads in [2, 5, 16] {
        let d = Arc::clone(&docs);
        assert!(finishes("index_shared", move || index_shared(d, threads)) == expected);
        let d = Arc::clone(&docs);
        assert!(finishes("index_channels", move || index_channels(&d, threads)) == expected);
    }
    assert!(index_rayon(&docs) == expected);
}

#[test]
fn an_index_can_be_sent_and_shared_between_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Index>();
    assert_send_sync::<Arc<Vec<String>>>();
}
