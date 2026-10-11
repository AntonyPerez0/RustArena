//! Parse, compare and match [semantic versions](https://semver.org).
//!
//! `semver-lite` is a small, dependency-light take on the version rules Cargo
//! uses: [`Version`] parses and orders versions like `1.4.2` and `2.0.0-rc.1`,
//! and [`VersionReq`] checks them against requirements like `^1.2` or
//! `>=1.0.0, <2.0.0`.
//!
//! ```
//! use semver_lite::{Version, VersionReq};
//!
//! let req: VersionReq = "^1.2".parse()?;
//! assert!(req.matches(&Version::parse("1.9.0")?));
//! assert!(!req.matches(&Version::parse("2.0.0")?));
//!
//! let mut versions = vec![Version::parse("1.10.0")?, Version::parse("1.2.0")?];
//! versions.sort();
//! assert_eq!(versions[0].to_string(), "1.2.0");
//! # Ok::<(), semver_lite::Error>(())
//! ```
//!
//! For messier input, such as Git tags, build a lenient [`Parser`].
//!
//! # Features
//!
//! - `serde`: implements `Serialize` and `Deserialize` for [`Version`], as a string.
//!
//! # Not supported
//!
//! Build metadata (`1.0.0+build.5`) and Cargo's special rule for matching
//! pre-releases against requirements.
#![deny(missing_docs)]
#![warn(clippy::missing_errors_doc)]

mod error;
mod parser;
mod prerelease;
mod req;
#[cfg(feature = "serde")]
mod serde_impl;
mod version;

pub use error::Error;
pub use parser::Parser;
pub use prerelease::Prerelease;
pub use req::VersionReq;
pub use version::Version;
