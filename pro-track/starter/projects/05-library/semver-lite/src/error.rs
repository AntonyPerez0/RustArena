use std::fmt;

// The error type for the whole crate: one enum with a variant per way things can
// go wrong, so callers can `match` on what happened instead of parsing a message.
//
// TODO (task 1):
// - Replace the two hand-written impls at the bottom with #[derive(thiserror::Error)]
//   and an #[error("...")] attribute on every variant. The exact messages are in
//   the comments; `{input}` means "the value of the `input` field".
// - Mark the enum #[non_exhaustive], so adding a variant later is not a breaking change.
// - InvalidReq must report its cause through std::error::Error::source().
//   (thiserror treats a field named `source` as the cause automatically.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    // The input was empty, or only whitespace.
    // message: empty version string
    Empty,
    // The input did not have three dot-separated parts (one to three with allow_partial).
    // message: invalid version "{input}": expected MAJOR.MINOR.PATCH
    BadShape { input: String },
    // A part was not a number: empty, not digits, or too big for a u64.
    // message: invalid version number "{part}"
    InvalidNumber { part: String },
    // A number had a leading zero, like the 02 in 1.02.3.
    // message: version number "{part}" has a leading zero
    LeadingZero { part: String },
    // The pre-release part broke the rules (see Prerelease::new).
    // message: invalid pre-release "{0}"
    InvalidPrerelease(String),
    // A version requirement could not be parsed; `source` says why.
    // message: invalid version requirement "{req}"
    InvalidReq { req: String, source: Box<Error> },
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("error message for {self:?} (task 1)")
    }
}

impl std::error::Error for Error {}
