// TODO (task 4): crate-level documentation (`//!` comments with an example) and
// #![deny(missing_docs)], so the build fails if any public item is undocumented.

mod error;
mod parser;
mod prerelease;
mod req;
mod version;

// The public API is decided here, not by the module layout: users write
// `semver_lite::Version`, never `semver_lite::version::Version`. That leaves you
// free to move code between private modules without breaking anyone.
pub use error::Error;
pub use parser::Parser;
pub use prerelease::Prerelease;
pub use req::VersionReq;
pub use version::Version;
