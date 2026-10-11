use crate::Index;

// Part 3: rayon, a data-parallelism library with a work-stealing thread pool.
//
// TODO: the same result in a few lines, with `use rayon::prelude::*;` and
// `docs.par_iter()`. Two ways to combine the per-document results:
// - .map(|doc| Index::of_document(doc)).reduce(Index::default, |a, b| ...)
// - .fold(Index::default, ...).reduce(Index::default, ...), which builds one
//   Index per batch of work instead of one per document.
pub fn index_rayon(docs: &[String]) -> Index {
    todo!("part 3: index {} documents with rayon", docs.len())
}
