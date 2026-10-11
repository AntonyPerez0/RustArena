use rayon::prelude::*;

use crate::Index;

/// Part 3: rayon. Its pool has one thread per core; idle threads steal work
/// from busy ones, so uneven documents balance themselves.
///
/// `fold` builds one Index per batch of documents a thread processes, and
/// `reduce` merges those batches pairwise.
pub fn index_rayon(docs: &[String]) -> Index {
    docs.par_iter()
        .fold(Index::default, |mut index, doc| {
            index.add_document(doc);
            index
        })
        .reduce(Index::default, |mut a, b| {
            a.merge(b);
            a
        })
}
