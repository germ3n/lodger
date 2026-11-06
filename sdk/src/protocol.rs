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

/// What the host hands a mod at `init`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Init {
    /// The mod's config file's text, empty when there is none.
    pub config: String,
    /// The game directory: where `mods/`, `config/` and the worlds are. Files a mod keeps
    /// outside its config (a model, a log) go under `<config_dir>/<id>/`.
    pub game_dir: String,
    /// The config directory, `<game_dir>/config` on both loaders.
    pub config_dir: String,
    /// The mod's id, as the manifest names it, so a mod need not hardcode its own folder name.
    pub id: String,
    /// The config file's text before a change from the host's settings screen, empty at a
    /// start: a mod can tell what the player changed from what was only written. ABI 45.
    pub config_before: String,
}

impl Init {
    /// A bare `Init` with the config text alone, for tests and the harness.
    pub fn with_config(config: &str) -> Self {
        Self {
            config: config.to_string(),
            ..Self::default()
        }
    }

    /// `<config_dir>/<id>/`, where a mod keeps what is not its config file.
    pub fn own_dir(&self) -> std::path::PathBuf {
        std::path::Path::new(&self.config_dir).join(&self.id)
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
    /// Values the config file had of the wrong type, each mended or left at its default, in
    /// words for the host to log once (`brain.most_plans: invalid type ...; the default kept`).
    /// ABI 22.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Files the mod needs that are not in its jar. The host fetches the missing ones before the
    /// game is playable: on a client, a screen over the title screen with a progress bar; on a
    /// dedicated server, at startup with progress in the log. See `docs/sdk.md`.
    #[serde(default)]
    pub resources: Vec<Resource>,
    /// The settings the host's settings screen shows, a few of the config's keys made easy to
    /// change; the rest stay in the file. ABI 21.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<Setting>,
    /// Write `config` over the file, not only when it is missing: the mod changed what the file
    /// should say (a preset picked, its values in). ABI 45.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rewrite: bool,
}

/// One setting on the host's settings screen: a config key, how to change it, its value now.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Setting {
    /// The key, dotted for a nested one: `model.size`.
    pub key: String,
    pub label: String,
    /// The heading it sits under: `Model`, `How scary`.
    pub group: String,
    /// `choice` (one of `options`), `toggle` (true or false) or `number` (`min` to `max` by
    /// `step`).
    pub kind: String,
    pub options: Vec<String>,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub value: serde_json::Value,
}

/// A file a mod needs the host to fetch: the lodger's model, for one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Names the resource in the log, the screen and `status`.
    pub id: String,
    /// What the screen shows: `Downloading the lodger's model (491 MB)`.
    pub title: String,
    pub url: String,
    /// Where it goes. Absolute, or relative to the game directory.
    pub path: String,
    /// The size, if known, for the bar before the server says the length; 0 for unknown.
    #[serde(default)]
    pub size_hint: u64,
    /// Bytes the file must start with, as text (`GGUF`); empty for none. An HTML error page
    /// fails on its first bytes rather than after 491 MB.
    #[serde(default)]
    pub magic: String,
    /// The SHA-256 of the whole file, lowercase hex; empty to skip the check.
    #[serde(default)]
    pub sha256: String,
    /// Whether the game waits for it. A required resource gates the title screen; an optional
    /// one downloads in the background and the mod goes without until it lands.
    #[serde(default = "yes")]
    pub required: bool,
    /// The terms the file comes under, shown and accepted before the first download, once per
    /// resource (the host remembers). Empty for a file with no terms to show.
    #[serde(default)]
    pub license: License,
    /// Other forms of it the player may pick on the preparing screen (a smaller or a larger
    /// model): each a label and the change to the mod's config that picks it; the host writes
    /// the change and initialises the mod again. ABI 20.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<Choice>,
}

/// One form of a resource the player may pick.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Choice {
    /// What the screen shows: `Qwen2.5 1.5B, 1117 MB`.
    pub label: String,
    /// The change to the mod's config that picks it, merged into the file.
    pub config: serde_json::Value,
    /// Whether it is the one the config picks now.
    pub chosen: bool,
}

/// A resource's terms: a name, where to read them in full, and a short text the screen shows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct License {
    /// `Apache License 2.0`.
    pub name: String,
    /// The full text online.
    pub url: String,
    /// What the screen shows: who made the file, under what terms, in a few lines.
    pub text: String,
}

impl License {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.url.is_empty() && self.text.is_empty()
    }
}

fn yes() -> bool {
    true
}

/// What a mod says of itself when asked, for the host's screens and `/craftkit status`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Status {
    /// Whether the mod is ready to play: its resources present and whatever it loads loaded.
    pub ready: bool,
    /// One line for the screen: `Loading the model...`, `Ready`, or why not.
    pub line: String,
    /// Whether `warm` is still working.
    pub loading: bool,
    /// What is wrong, if anything, for the log.
    pub error: String,
}

/// An error crossing the ABI: `{"error": "..."}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub error: String,
}
