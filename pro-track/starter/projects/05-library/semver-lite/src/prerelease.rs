use std::cmp::Ordering;
use std::fmt;

use crate::Error;

// The pre-release part of a version, like `alpha.1` in `1.0.0-alpha.1`.
//
// A newtype: a struct with one private field. The only way to get a Prerelease is
// Prerelease::new, which checks the rules, so every Prerelease that exists is
// valid, and its ordering rule lives with the data instead of in every caller.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Prerelease(String);

impl Prerelease {
    // No pre-release: the version is a normal release.
    pub const EMPTY: Prerelease = Prerelease(String::new());

    // TODO (task 2): checks `text` and wraps it. "" gives Prerelease::EMPTY.
    // Otherwise `text` is one or more identifiers separated by dots, where each
    // identifier is non-empty, uses only ASCII letters, digits and '-', and is not
    // a number with a leading zero ("0" is fine, "01" is not).
    // Anything else: Err(Error::InvalidPrerelease(text)).
    pub fn new(text: &str) -> Result<Self, Error> {
        todo!("validate {text:?} (task 2)")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// AsRef<str> lets a Prerelease go anywhere a "string-like" value is accepted.
impl AsRef<str> for Prerelease {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Prerelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// TODO (task 2): pre-release precedence, from semver.org (item 11):
// - EMPTY (a release) is greater than every pre-release: 1.0.0-rc.1 < 1.0.0.
// - Otherwise compare identifier by identifier, left to right:
//   numbers compare as numbers (2 < 11), other identifiers compare as ASCII
//   text, and a number is always less than a non-number (1 < alpha).
// - If all shared identifiers are equal, the one with fewer identifiers is less:
//   alpha < alpha.1.
impl Ord for Prerelease {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!("compare {self:?} with {other:?} (task 2)")
    }
}

impl PartialOrd for Prerelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
