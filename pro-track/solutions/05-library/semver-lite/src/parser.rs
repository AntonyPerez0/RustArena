use crate::{Error, Prerelease, Version};

/// Parses versions: strictly by default, with options for messier input.
///
/// A builder: start from [`Parser::new`] and switch options on with chained calls.
/// `Parser` is `Copy`, so keep one around and reuse it.
///
/// ```
/// use semver_lite::{Parser, Version};
///
/// let tags = Parser::new().allow_v_prefix(true).allow_partial(true);
/// assert_eq!(tags.parse("v1.4")?, Version::new(1, 4, 0));
/// assert!(Parser::new().parse("v1.4").is_err());
/// # Ok::<(), semver_lite::Error>(())
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Parser {
    v_prefix: bool,
    partial: bool,
}

impl Parser {
    /// The strict parser: exactly `MAJOR.MINOR.PATCH[-PRE]`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            v_prefix: false,
            partial: false,
        }
    }

    /// Accept one leading `v` or `V`, as in Git tags: `v1.2.3`.
    #[must_use]
    pub const fn allow_v_prefix(self, allow: bool) -> Self {
        Self {
            v_prefix: allow,
            ..self
        }
    }

    /// Accept one or two parts and fill in zeros: `1` is 1.0.0, `1.2` is 1.2.0.
    #[must_use]
    pub const fn allow_partial(self, allow: bool) -> Self {
        Self {
            partial: allow,
            ..self
        }
    }

    /// Parses one version, ignoring surrounding whitespace.
    ///
    /// # Errors
    ///
    /// - [`Error::Empty`] for an empty input,
    /// - [`Error::BadShape`] for the wrong number of parts,
    /// - [`Error::InvalidNumber`] or [`Error::LeadingZero`] for a bad number,
    /// - [`Error::InvalidPrerelease`] for a bad pre-release.
    pub fn parse(&self, input: &str) -> Result<Version, Error> {
        let input = input.trim();
        if input.is_empty() {
            return Err(Error::Empty);
        }
        let text = match input.strip_prefix(['v', 'V']) {
            Some(rest) if self.v_prefix => rest,
            _ => input,
        };
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (text, None),
        };
        let parts: Vec<&str> = core.split('.').collect();
        let allowed = if self.partial { 1..=3 } else { 3..=3 };
        if !allowed.contains(&parts.len()) {
            return Err(Error::BadShape {
                input: input.to_owned(),
            });
        }
        let mut numbers = [0; 3];
        for (number, part) in numbers.iter_mut().zip(parts) {
            *number = parse_number(part)?;
        }
        let pre = match pre {
            None => Prerelease::EMPTY,
            Some("") => return Err(Error::InvalidPrerelease(String::new())),
            Some(pre) => Prerelease::new(pre)?,
        };
        let [major, minor, patch] = numbers;
        Ok(Version {
            major,
            minor,
            patch,
            pre,
        })
    }

    /// Parses every input, or returns the first error.
    ///
    /// Accepts anything that yields string-like items: an array of `&str`, a
    /// `Vec<String>`, a `&[String]` of command-line arguments...
    ///
    /// ```
    /// use semver_lite::Parser;
    ///
    /// let args: Vec<String> = vec!["v2.0.0".into(), "1.0.0".into()];
    /// let mut versions = Parser::new().allow_v_prefix(true).parse_all(&args)?;
    /// versions.sort();
    /// assert_eq!(versions[0].to_string(), "1.0.0");
    /// # Ok::<(), semver_lite::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The [`Error`] from the first input that does not parse.
    pub fn parse_all(
        &self,
        inputs: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Vec<Version>, Error> {
        inputs
            .into_iter()
            .map(|input| self.parse(input.as_ref()))
            .collect()
    }
}

fn parse_number(part: &str) -> Result<u64, Error> {
    let invalid = || Error::InvalidNumber {
        part: part.to_owned(),
    };
    // Check the digits ourselves: u64::from_str would also accept "+2".
    if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    if part.len() > 1 && part.starts_with('0') {
        return Err(Error::LeadingZero {
            part: part.to_owned(),
        });
    }
    part.parse().map_err(|_| invalid())
}
