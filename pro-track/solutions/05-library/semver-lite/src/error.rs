/// Everything that can go wrong in this crate.
///
/// Match on the variant to find out what happened; the [`Display`](std::fmt::Display)
/// message is for people. The enum is `#[non_exhaustive]`, so new variants can
/// be added in a minor release: when you `match` on it, add a `_` arm.
///
/// ```
/// use semver_lite::{Error, Version};
///
/// let err = Version::parse("1.02.3").unwrap_err();
/// assert_eq!(err, Error::LeadingZero { part: "02".into() });
/// assert_eq!(err.to_string(), r#"version number "02" has a leading zero"#);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The input was empty, or only whitespace.
    #[error("empty version string")]
    Empty,
    /// The input did not have three dot-separated parts
    /// (one to three with [`Parser::allow_partial`](crate::Parser::allow_partial)).
    #[error("invalid version \"{input}\": expected MAJOR.MINOR.PATCH")]
    BadShape {
        /// The whole input, trimmed.
        input: String,
    },
    /// A part was not a number: empty, not ASCII digits, or too big for a `u64`.
    #[error("invalid version number \"{part}\"")]
    InvalidNumber {
        /// The part that is not a number.
        part: String,
    },
    /// A number had a leading zero, like the `02` in `1.02.3`.
    #[error("version number \"{part}\" has a leading zero")]
    LeadingZero {
        /// The number with the leading zero.
        part: String,
    },
    /// The pre-release broke the rules described in [`Prerelease::new`](crate::Prerelease::new).
    #[error("invalid pre-release \"{0}\"")]
    InvalidPrerelease(String),
    /// A version requirement could not be parsed. [`source()`](std::error::Error::source)
    /// returns the error from the part that failed.
    ///
    /// ```
    /// use std::error::Error as _;
    /// use semver_lite::VersionReq;
    ///
    /// let err = "^1.x".parse::<VersionReq>().unwrap_err();
    /// assert_eq!(err.to_string(), r#"invalid version requirement "^1.x""#);
    /// assert_eq!(err.source().unwrap().to_string(), r#"invalid version number "x""#);
    /// ```
    #[error("invalid version requirement \"{req}\"")]
    InvalidReq {
        /// The whole requirement, trimmed.
        req: String,
        /// Why it failed.
        source: Box<Error>,
    },
}
