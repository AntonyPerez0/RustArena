//! Serde support, compiled only with the `serde` feature.
//!
//! A version is written as its string form ("1.2.3-rc.1"), not as a struct with
//! four fields: that is how every config file and API already spells versions.
//! Written by hand instead of with `#[derive]`, so the crate does not need
//! serde's `derive` feature (and its proc-macro build time).

use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::Version;

impl Serialize for Version {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(de::Error::custom)
    }
}
