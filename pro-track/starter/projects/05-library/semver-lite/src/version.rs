use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use crate::{Error, Parser, Prerelease};

// A semantic version: MAJOR.MINOR.PATCH with an optional pre-release,
// like 1.4.2 or 2.0.0-rc.1. (Build metadata, the `+build.5` suffix, is not
// supported by this lite crate.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Prerelease,
}

impl Version {
    // A release version (no pre-release). `const`, so it works in constants too.
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Prerelease::EMPTY,
        }
    }

    // Strict parsing; the same as Parser::new().parse(input).
    pub fn parse(input: &str) -> Result<Self, Error> {
        Parser::new().parse(input)
    }

    // TODO (task 3): true if the version has a pre-release.
    pub fn is_prerelease(&self) -> bool {
        todo!("is {self:?} a pre-release? (task 3)")
    }

    // TODO (task 3): the next major version: 1.2.3 -> 2.0.0, and 1.2.3-rc.1 -> 2.0.0.
    // The bump methods take &self and return a new Version; `self` is unchanged.
    pub fn bump_major(&self) -> Self {
        todo!("bump {self:?} (task 3)")
    }

    // TODO (task 3): the next minor version: 1.2.3 -> 1.3.0.
    pub fn bump_minor(&self) -> Self {
        todo!("bump {self:?} (task 3)")
    }

    // TODO (task 3): the next patch version: 1.2.3 -> 1.2.4, and 1.2.3-rc.1 -> 1.2.4.
    pub fn bump_patch(&self) -> Self {
        todo!("bump {self:?} (task 3)")
    }
}

// TODO (task 3): order by major, then minor, then patch (as numbers), then pre.
// Tip: a derived Ord compares fields in the order they are declared, using each
// field's own Ord. Since Prerelease already knows its rules, you could delete
// these two impls and add PartialOrd and Ord to the #[derive] above instead.
impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!("compare {self:?} with {other:?} (task 3)")
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// TODO (task 3): "1.2.3", or "1.2.3-rc.1" when there is a pre-release.
impl fmt::Display for Version {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("format {self:?} (task 3)")
    }
}

// FromStr makes `"1.2.3".parse::<Version>()` work. Strict, like Version::parse.
impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::parse(s)
    }
}
