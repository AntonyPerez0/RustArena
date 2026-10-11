use std::cmp::Ordering;
use std::fmt;

use crate::Error;

/// The pre-release part of a version, like `alpha.1` in `1.0.0-alpha.1`.
///
/// You can only get one from [`Prerelease::new`], which checks the rules, so every
/// `Prerelease` is valid. It is ordered by semver precedence, which is not plain
/// string order:
///
/// ```
/// use semver_lite::Prerelease;
///
/// let beta2 = Prerelease::new("beta.2")?;
/// let beta11 = Prerelease::new("beta.11")?;
/// assert!(beta2 < beta11, "numbers compare as numbers");
/// assert!(beta11 < Prerelease::EMPTY, "a release comes after its pre-releases");
/// # Ok::<(), semver_lite::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Prerelease(String);

impl Prerelease {
    /// No pre-release: the version is a normal release.
    pub const EMPTY: Prerelease = Prerelease(String::new());

    /// Checks `text` and wraps it. An empty string gives [`Prerelease::EMPTY`].
    ///
    /// Otherwise `text` must be identifiers separated by dots, where each
    /// identifier is non-empty, uses only ASCII letters, digits and `-`, and is
    /// not a number with a leading zero.
    ///
    /// ```
    /// use semver_lite::Prerelease;
    ///
    /// assert_eq!(Prerelease::new("rc.1")?.as_str(), "rc.1");
    /// assert!(Prerelease::new("rc..1").is_err());
    /// assert!(Prerelease::new("01").is_err());
    /// # Ok::<(), semver_lite::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::InvalidPrerelease`] if `text` breaks any of those rules.
    pub fn new(text: &str) -> Result<Self, Error> {
        if text.is_empty() || text.split('.').all(valid_identifier) {
            Ok(Self(text.to_owned()))
        } else {
            Err(Error::InvalidPrerelease(text.to_owned()))
        }
    }

    /// The pre-release as text, without the leading `-`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True for [`Prerelease::EMPTY`], meaning a normal release.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn is_number(id: &str) -> bool {
    id.bytes().all(|b| b.is_ascii_digit())
}

fn valid_identifier(id: &str) -> bool {
    !id.is_empty()
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && !(is_number(id) && id.len() > 1 && id.starts_with('0'))
}

fn compare_identifiers(a: &str, b: &str) -> Ordering {
    match (is_number(a), is_number(b)) {
        // No leading zeros, so a longer number is a bigger one, even past u64::MAX.
        (true, true) => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => a.cmp(b),
    }
}

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

/// Semver precedence: [`Prerelease::EMPTY`] is the greatest; otherwise
/// identifiers are compared left to right (numbers numerically and before
/// non-numbers), and with equal shared identifiers, fewer identifiers is less.
impl Ord for Prerelease {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.is_empty(), other.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                for (a, b) in self.0.split('.').zip(other.0.split('.')) {
                    let order = compare_identifiers(a, b);
                    if order != Ordering::Equal {
                        return order;
                    }
                }
                self.0.split('.').count().cmp(&other.0.split('.').count())
            }
        }
    }
}

impl PartialOrd for Prerelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
