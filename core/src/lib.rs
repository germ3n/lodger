//! The mind of the lodger.
//!
//! Everything the lodger knows and decides lives here, and nothing here knows what Minecraft is:
//! positions are integers, items are strings, players are indices. It is a craftkit mod
//! (`../sdk`): the host loads it as a shared library and talks to it once a tick in JSON. See
//! `../docs/architecture.md`.
//!
//! Phase 0 (see `../docs/todo.md`): the crate builds, `init` parses the config and answers the
//! effective one, and the ABI path is exercised once before anything rides on it.

pub mod config;

use config::Config;
use craftkit::{InitReply, Mod, ModInfo};

/// The mind's version, from `Cargo.toml`. `mod/gradle.properties` carries the same one and
/// `VersionsTest` keeps them agreeing.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The lodger as the host sees it: one per process, made by `init`.
pub struct Lodger {
    pub config: Config,
}

impl Mod for Lodger {
    fn info() -> ModInfo {
        ModInfo::new("lodger", "lodger", VERSION)
    }

    fn init(text: &str) -> Result<(Self, InitReply), String> {
        let config = Config::parse(text)?;
        let reply = InitReply {
            config: serde_json::to_value(&config).map_err(|err| err.to_string())?,
            unknown: config.unknown_keys(),
        };

        Ok((Lodger { config }, reply))
    }
}

craftkit::export_mod!(Lodger);

#[cfg(test)]
mod tests {
    use super::*;

    fn harness() -> craftkit::harness::Harness {
        craftkit::harness!().unwrap()
    }

    #[test]
    fn info_names_the_lodger_and_its_version() {
        let info = harness().info();

        assert_eq!(info["id"], "lodger");
        assert_eq!(info["version"], VERSION);
    }

    #[test]
    fn init_with_an_empty_config_answers_the_defaults() {
        let reply = harness().init("");

        assert_eq!(reply["config"]["brain"]["queue"], 64);
        assert_eq!(reply["unknown"], serde_json::json!([]));
    }

    #[test]
    fn init_lists_unknown_keys_and_keeps_known_values() {
        let reply = harness().init(r#"{ "colour": 1, "model": { "threads": 4 } }"#);

        assert_eq!(reply["unknown"], serde_json::json!(["colour"]));
        assert_eq!(reply["config"]["model"]["threads"], 4);
    }

    #[test]
    fn init_with_bad_json_is_an_error_naming_the_config() {
        let reply = harness().init("{ nope");

        assert!(reply["error"].as_str().unwrap().starts_with("config: "));
    }
}
