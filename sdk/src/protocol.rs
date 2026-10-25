//! The JSON shapes that cross the ABI. Every one is a plain `serde` struct, so the host's Gson
//! and the mod's `serde_json` read the same bytes, and a shape change is a diff here, a line in
//! `docs/sdk.md` and a bump of [`crate::ABI_VERSION`].

use serde::{Deserialize, Serialize};

/// What a mod says about itself, for the host's log and `/craftkit status`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModInfo {
    /// The mod id, as the content jar's manifests name it: lowercase, `[a-z0-9_]`.
    pub id: String,
    pub name: String,
    pub version: String,
    /// The SDK the library was built with ([`crate::VERSION`]).
    pub sdk: String,
}

impl ModInfo {
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            sdk: crate::VERSION.to_string(),
        }
    }
}

/// The answer to `init`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InitReply {
    /// The mod's config as it read it, every key present with its default, for the host to write
    /// to `config/<id>.json` when that file was missing. A mod without config answers `{}`.
    pub config: serde_json::Value,
    /// Keys the config file had that the mod does not know, for the host to log once.
    #[serde(default)]
    pub unknown: Vec<String>,
}

/// An error crossing the ABI: `{"error": "..."}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub error: String,
}
