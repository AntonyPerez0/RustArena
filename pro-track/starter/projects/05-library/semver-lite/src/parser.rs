use crate::{Error, Version};

// Parses versions: strictly by default, with options for messier real-world input.
//
// A builder: start from Parser::new() and switch options on with chained calls,
//     let parser = Parser::new().allow_v_prefix(true).allow_partial(true);
// instead of a function with a row of bool arguments nobody can read at the call
// site (`parse(s, true, false)`), or a breaking change every time an option is added.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Parser {
    v_prefix: bool,
    partial: bool,
}

impl Parser {
    // The strict parser: exactly MAJOR.MINOR.PATCH[-PRE].
    pub const fn new() -> Self {
        Self {
            v_prefix: false,
            partial: false,
        }
    }

    // Accept one leading `v` or `V`, as in Git tags: v1.2.3.
    pub const fn allow_v_prefix(self, allow: bool) -> Self {
        Self {
            v_prefix: allow,
            ..self
        }
    }

    // Accept 1 or 2 parts and fill in zeros: "1" is 1.0.0, "1.2" is 1.2.0.
    pub const fn allow_partial(self, allow: bool) -> Self {
        Self {
            partial: allow,
            ..self
        }
    }

    // TODO (task 3): parse one version. The rules, in order:
    // 1. Trim surrounding whitespace. Nothing left: Error::Empty.
    // 2. With allow_v_prefix, strip one leading 'v' or 'V'.
    // 3. Split at the first '-': before it is the core, after it the pre-release.
    // 4. Split the core at '.': exactly 3 parts (1 to 3 with allow_partial), else
    //    Error::BadShape { input } where input is the trimmed input.
    // 5. Each part must be ASCII digits only and fit in a u64, else
    //    Error::InvalidNumber { part }; "0" is fine but "01" is Error::LeadingZero { part }.
    // 6. A '-' with nothing after it is Error::InvalidPrerelease(""). Otherwise the
    //    pre-release goes through Prerelease::new.
    pub fn parse(&self, input: &str) -> Result<Version, Error> {
        todo!("parse {input:?} with {self:?} (task 3)")
    }

    // TODO (task 3): parse every input, or return the first error.
    // `impl IntoIterator<Item = impl AsRef<str>>` accepts an array of &str, a
    // Vec<String>, a &[String] from std::env::args, an iterator... anything
    // that yields string-like items, with no copying and no conversions by the caller.
    pub fn parse_all(
        &self,
        inputs: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Vec<Version>, Error> {
        let _ = inputs.into_iter();
        todo!("parse every input (task 3)")
    }
}
