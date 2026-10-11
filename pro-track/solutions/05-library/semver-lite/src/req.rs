use std::str::FromStr;

use crate::{Error, Parser, Version};

/// A version requirement, like the ones in `Cargo.toml`.
///
/// Comma-separated comparators that must all match: `^1.2`, `~1.2.3`,
/// `>=1.0.0, <2.0.0`, `=1.2.3`, or `*` for anything. A bare version (`1.2.3`)
/// means `^1.2.3`, as in Cargo.
///
/// `^` and `~` accept partial versions and allow these ranges (upper bound excluded):
///
/// | requirement | allows              | requirement | allows              |
/// |-------------|---------------------|-------------|---------------------|
/// | `^1.2.3`    | 1.2.3 up to 2.0.0   | `~1.2.3`    | 1.2.3 up to 1.3.0   |
/// | `^0.2.3`    | 0.2.3 up to 0.3.0   | `~1.2`      | 1.2.0 up to 1.3.0   |
/// | `^0.0.3`    | 0.0.3 up to 0.0.4   | `~1`        | 1.0.0 up to 2.0.0   |
///
/// `=`, `>`, `>=`, `<` and `<=` need a full version and use [`Version`]'s ordering.
/// The upper bounds of `^` and `~` ignore pre-releases: `^1.2` does not match
/// `2.0.0-alpha`.
///
/// ```
/// use semver_lite::{Version, VersionReq};
///
/// let req: VersionReq = ">=1.2.0, <1.5.0".parse()?;
/// assert!(req.matches(&Version::new(1, 4, 9)));
/// assert!(!req.matches(&Version::new(1, 5, 0)));
/// # Ok::<(), semver_lite::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VersionReq {
    comparators: Vec<Comparator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Comparator {
    op: Op,
    version: Version,
    // How many of MAJOR.MINOR.PATCH were written: ^1 and ^1.0.0 differ for 0.x.
    parts: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Exact,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Caret,
    Tilde,
}

impl VersionReq {
    /// Matches every version, like `*`.
    pub const STAR: VersionReq = VersionReq {
        comparators: Vec::new(),
    };

    /// Parses a requirement. Also available as `text.parse::<VersionReq>()`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidReq`], whose [`source()`](std::error::Error::source) is
    /// the error from the comparator that failed (an empty one gives [`Error::Empty`]).
    pub fn parse(text: &str) -> Result<Self, Error> {
        let text = text.trim();
        if text == "*" {
            return Ok(Self::STAR);
        }
        text.split(',')
            .map(|part| Comparator::parse(part.trim()))
            .collect::<Result<Vec<_>, _>>()
            .map(|comparators| Self { comparators })
            .map_err(|source| Error::InvalidReq {
                req: text.to_owned(),
                source: Box::new(source),
            })
    }

    /// True if `version` satisfies every comparator.
    pub fn matches(&self, version: &Version) -> bool {
        self.comparators.iter().all(|c| c.matches(version))
    }

    /// The highest version that matches, if any. Takes anything that yields
    /// `&Version`: a `&Vec<Version>`, a slice, a filtered iterator...
    ///
    /// ```
    /// use semver_lite::{Parser, VersionReq};
    ///
    /// let published = Parser::new().parse_all(["1.0.0", "1.4.2", "2.0.0"])?;
    /// let req: VersionReq = "^1".parse()?;
    /// assert_eq!(req.max_satisfying(&published), Some(&published[1]));
    /// # Ok::<(), semver_lite::Error>(())
    /// ```
    pub fn max_satisfying<'a>(
        &self,
        versions: impl IntoIterator<Item = &'a Version>,
    ) -> Option<&'a Version> {
        versions.into_iter().filter(|v| self.matches(v)).max()
    }

    /// The versions that match, in their original order.
    ///
    /// ```
    /// use semver_lite::{Parser, VersionReq};
    ///
    /// let versions = Parser::new().parse_all(["0.9.0", "1.1.0", "1.2.0"])?;
    /// let req: VersionReq = "~1.1".parse()?;
    /// assert_eq!(req.matching(&versions).count(), 1);
    /// # Ok::<(), semver_lite::Error>(())
    /// ```
    pub fn matching<'a>(&self, versions: &'a [Version]) -> impl Iterator<Item = &'a Version> {
        versions.iter().filter(move |v| self.matches(v))
    }
}

impl FromStr for VersionReq {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::parse(s)
    }
}

impl Comparator {
    fn parse(text: &str) -> Result<Self, Error> {
        let (op, rest) = [
            (">=", Op::GreaterEq),
            ("<=", Op::LessEq),
            (">", Op::Greater),
            ("<", Op::Less),
            ("=", Op::Exact),
            ("^", Op::Caret),
            ("~", Op::Tilde),
        ]
        .into_iter()
        .find_map(|(prefix, op)| text.strip_prefix(prefix).map(|rest| (op, rest)))
        .unwrap_or((Op::Caret, text));
        let rest = rest.trim();
        let partial = matches!(op, Op::Caret | Op::Tilde);
        let version = Parser::new().allow_partial(partial).parse(rest)?;
        let core = rest.split_once('-').map_or(rest, |(core, _)| core);
        let parts = core.split('.').count();
        Ok(Self { op, version, parts })
    }

    fn matches(&self, v: &Version) -> bool {
        let c = &self.version;
        match self.op {
            Op::Exact => v == c,
            Op::Greater => v > c,
            Op::GreaterEq => v >= c,
            Op::Less => v < c,
            Op::LessEq => v <= c,
            Op::Caret | Op::Tilde => v >= c && (v.major, v.minor, v.patch) < self.upper_bound(),
        }
    }

    // The exclusive upper bound of a ^ or ~ range, as (major, minor, patch).
    fn upper_bound(&self) -> (u64, u64, u64) {
        let (major, minor, patch) = (self.version.major, self.version.minor, self.version.patch);
        let next = |n: u64| n.saturating_add(1);
        match (self.op, self.parts) {
            (_, 1) => (next(major), 0, 0),
            (Op::Tilde, _) => (major, next(minor), 0),
            (_, _) if major > 0 => (next(major), 0, 0),
            (_, 2) => (0, next(minor), 0),
            (_, _) if minor > 0 => (0, next(minor), 0),
            _ => (0, 0, next(patch)),
        }
    }
}
