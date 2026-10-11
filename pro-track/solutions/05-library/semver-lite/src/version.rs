use std::fmt;
use std::str::FromStr;

use crate::{Error, Parser, Prerelease};

/// A semantic version: `MAJOR.MINOR.PATCH`, with an optional pre-release.
///
/// Versions are ordered by semver precedence: numbers compare as numbers, and
/// a pre-release comes before its release.
///
/// ```
/// use semver_lite::Version;
///
/// let v: Version = "1.2.3-rc.1".parse()?;
/// assert_eq!((v.major, v.minor, v.patch), (1, 2, 3));
/// assert_eq!(v.pre.as_str(), "rc.1");
/// assert!(v < Version::new(1, 2, 3));
/// assert!(Version::new(1, 10, 0) > Version::new(1, 9, 0));
/// # Ok::<(), semver_lite::Error>(())
/// ```
///
/// Build metadata (`+build.5`) is not supported.
// The derived Ord compares the fields in this order, using Prerelease's own Ord,
// which is exactly semver precedence. Don't reorder the fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version {
    /// Incremented for incompatible API changes.
    pub major: u64,
    /// Incremented for backward compatible new features.
    pub minor: u64,
    /// Incremented for backward compatible bug fixes.
    pub patch: u64,
    /// The pre-release, or [`Prerelease::EMPTY`] for a release.
    pub pre: Prerelease,
}

impl Version {
    /// A release version, with no pre-release. Usable in constants.
    ///
    /// ```
    /// use semver_lite::Version;
    ///
    /// const MSRV: Version = Version::new(1, 85, 0);
    /// assert_eq!(MSRV.to_string(), "1.85.0");
    /// ```
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Prerelease::EMPTY,
        }
    }

    /// Parses a version strictly: `MAJOR.MINOR.PATCH[-PRE]`, ignoring
    /// surrounding whitespace. The same as `Parser::new().parse(input)`; use a
    /// [`Parser`] to accept `v1.2.3` or `1.2`.
    ///
    /// ```
    /// use semver_lite::{Error, Version};
    ///
    /// assert_eq!(Version::parse("1.4.0")?, Version::new(1, 4, 0));
    /// assert_eq!(Version::parse("1.4"), Err(Error::BadShape { input: "1.4".into() }));
    /// # Ok::<(), semver_lite::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An [`Error`] saying which part of the input is wrong.
    pub fn parse(input: &str) -> Result<Self, Error> {
        Parser::new().parse(input)
    }

    /// True if the version has a pre-release, like `2.0.0-beta`.
    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }

    /// The next major version, for a breaking change: 1.2.3 becomes 2.0.0.
    /// The pre-release is dropped.
    ///
    /// ```
    /// use semver_lite::Version;
    ///
    /// let v = Version::new(1, 2, 3);
    /// assert_eq!(v.bump_major(), Version::new(2, 0, 0));
    /// assert_eq!(v.bump_minor(), Version::new(1, 3, 0));
    /// assert_eq!(v.bump_patch(), Version::new(1, 2, 4));
    /// ```
    ///
    /// # Panics
    ///
    /// If `major` is already `u64::MAX`.
    #[must_use]
    pub fn bump_major(&self) -> Self {
        Self::new(next(self.major), 0, 0)
    }

    /// The next minor version, for a new feature: 1.2.3 becomes 1.3.0.
    ///
    /// # Panics
    ///
    /// If `minor` is already `u64::MAX`.
    #[must_use]
    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, next(self.minor), 0)
    }

    /// The next patch version, for a bug fix: 1.2.3 becomes 1.2.4.
    ///
    /// # Panics
    ///
    /// If `patch` is already `u64::MAX`.
    #[must_use]
    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, next(self.patch))
    }
}

fn next(n: u64) -> u64 {
    n.checked_add(1).expect("version number overflow")
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if self.is_prerelease() {
            write!(f, "-{}", self.pre)?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::parse(s)
    }
}
