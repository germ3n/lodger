//! The mind of the lodger.
//!
//! Everything the lodger knows and decides lives here, and nothing here knows what Minecraft is:
//! positions are integers, items are strings, players are indices. It is a craftkit mod
//! (`../sdk`): the host loads it as a shared library and talks to it once a tick in JSON. See
//! `../docs/architecture.md`.
//!
//! Phase 0 (see `../docs/todo.md`): the crate builds, `init` parses the config and answers the
//! effective one, and the ABI path is exercised once before anything rides on it.

pub mod arc;
pub mod brain;
pub mod config;
pub mod events;
pub mod memory;
pub mod model;
pub mod rules;
pub mod stay;

use config::{Config, ModelSize};
use craftkit::{
    Command, Init, InitReply, License, Mod, ModInfo, Now, Observations, Report, Resource, Status,
    WorldInfo,
};
use model::{Model, Settings};
use stay::Stay;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The mind's version, from `Cargo.toml`. `mod/gradle.properties` carries the same one and
/// `VersionsTest` keeps them agreeing.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The lodger as the host sees it: one per process, made by `init`. It owns the model worker,
/// which is one per process too (the model is 700 MB in memory), so a client that opens two
/// worlds in a row loads it once.
pub struct Lodger {
    pub config: Config,
    pub model: Model,
    /// The decision log, one per process, under the mod's own folder.
    pub log: Mutex<brain::log::Log>,
}

/// Which size the config asks for; `parse` already refused any other name.
fn model_size(config: &Config) -> ModelSize {
    ModelSize::named(&config.model.size).unwrap_or(ModelSize::Small)
}

/// `model.path` as the config has it, or the size's file under the mod's own folder; relative
/// to the game directory unless absolute.
fn model_path(config: &Config, init: &Init) -> PathBuf {
    let path = if config.model.path.is_empty() {
        init.own_dir()
            .join("models")
            .join(model_size(config).file_name())
    } else {
        PathBuf::from(&config.model.path)
    };

    if path.is_absolute() || init.game_dir.is_empty() {
        path
    } else {
        Path::new(&init.game_dir).join(path)
    }
}

/// `model.url` as the config has it, or Qwen's own file for the size.
fn model_url(config: &Config) -> String {
    if config.model.url.is_empty() {
        model_size(config).url().to_string()
    } else {
        config.model.url.clone()
    }
}

/// A seed that differs per start, so two starts do not pick the same way; logged so a run can
/// be replayed. The world's seed joins it when a world opens (phase 2).
fn start_seed(init: &Init) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    init.game_dir.hash(&mut hasher);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
        .hash(&mut hasher);

    hasher.finish()
}

impl Mod for Lodger {
    type World = Stay;

    fn info() -> ModInfo {
        ModInfo::new("lodger", "lodger", VERSION)
    }

    fn init(init: &Init) -> Result<(Self, InitReply), String> {
        let config = Config::parse(&init.config)?;
        let size = model_size(&config);
        let path = model_path(&config, init);
        let reply = InitReply {
            config: serde_json::to_value(&config).map_err(|err| err.to_string())?,
            unknown: config.unknown_keys(),
            resources: vec![Resource {
                id: "model".to_string(),
                title: format!(
                    "Downloading the lodger's model, Qwen2.5 {} ({} MB)",
                    size.name().to_uppercase(),
                    size.megabytes()
                ),
                url: model_url(&config),
                path: path.to_string_lossy().into_owned(),
                size_hint: size.size_hint(),
                magic: "GGUF".to_string(),
                sha256: String::new(),
                required: true,
                license: License {
                    name: "Apache License 2.0".to_string(),
                    url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/blob/main/LICENSE"
                        .to_string(),
                    text: MODEL_LICENSE_TEXT.to_string(),
                },
            }],
        };

        let model = Model::new(Settings {
            path,
            threads: config.model.threads,
            context: config.model.context,
            temperature: config.brain.temperature,
            seed: start_seed(init),
        });

        let log_path = if init.config_dir.is_empty() {
            None
        } else {
            Some(init.own_dir().join("decisions.log"))
        };
        let log = Mutex::new(brain::log::Log::new(
            log_path,
            config.log.keep_mb,
            config.log.decisions,
        ));

        Ok((Lodger { config, model, log }, reply))
    }

    fn warm(&self) -> Result<(), String> {
        self.model.warm()
    }

    fn status(&self) -> Status {
        Status {
            ready: self.model.loaded(),
            line: self.model.status_line(),
            loading: self.model.loading(),
            error: self.model.error(),
        }
    }

    fn open(&self, info: &WorldInfo, saved: &[u8]) -> Result<Stay, String> {
        // The world's seed joins the start's, so two worlds on one start draw differently.
        self.model.reseed(self.model.seed() ^ info.seed as u64);

        Stay::open(info, saved)
    }

    fn observe(&self, world: &mut Stay, observations: &Observations) {
        // The day is the stay's own; before its first tick it has none, and day zero serves.
        let day = world
            .first_day
            .map(|_| world.day(&Now::default()))
            .unwrap_or(0.0);
        world.observe(observations, day);
    }

    fn tick(&self, world: &mut Stay, now: &Now) -> Vec<Command> {
        let mut log = self.log.lock().unwrap_or_else(|err| err.into_inner());

        world.tick(now, &self.config, &mut log)
    }

    fn report(&self, world: &mut Stay, reports: &[Report]) {
        let mut log = self.log.lock().unwrap_or_else(|err| err.into_inner());
        world.report(reports, &mut log);
    }

    fn save(&self, world: &Stay) -> Vec<u8> {
        world.save()
    }
}

/// What the license screen shows before the model is fetched. Qwen2.5-0.5B-Instruct is
/// Apache-2.0, which allows this use; the screen is there so the player knows what is being
/// downloaded, from whom, and under what terms, before 491 MB move (owner, 2026-11-09).
const MODEL_LICENSE_TEXT: &str =
    "The lodger thinks with a Qwen2.5-Instruct language model made by Alibaba \
Cloud's Qwen team and released under the Apache License 2.0. It will be downloaded once from \
Hugging Face, from Qwen's own repository, and kept on this computer. Nothing is sent \
anywhere; the model runs here. Copyright 2024 Alibaba Cloud. Licensed under the Apache License, \
Version 2.0; the full text is at the link below. By continuing you accept those terms for your \
use of the model.";

craftkit::export_mod!(Lodger);

#[cfg(test)]
mod tests {
    use super::*;
    /// The exported functions share one static lodger, and llama.cpp's backend is one per
    /// process, so these tests take turns with the model's, as the host's calls do.
    fn turn() -> std::sync::MutexGuard<'static, ()> {
        model::TEST_TURN
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    fn harness() -> craftkit::harness::Harness {
        craftkit::harness!().unwrap()
    }

    #[test]
    fn info_names_the_lodger_and_its_version() {
        let _turn = turn();
        let info = harness().info();

        assert_eq!(info["id"], "lodger");
        assert_eq!(info["version"], VERSION);
    }

    #[test]
    fn init_with_an_empty_config_answers_the_defaults() {
        let _turn = turn();
        let reply = harness().init("");

        assert_eq!(reply["config"]["brain"]["queue"], 64);
        assert_eq!(reply["unknown"], serde_json::json!([]));
    }

    #[test]
    fn init_lists_unknown_keys_and_keeps_known_values() {
        let _turn = turn();
        let reply = harness().init(r#"{ "colour": 1, "model": { "threads": 4 } }"#);

        assert_eq!(reply["unknown"], serde_json::json!(["colour"]));
        assert_eq!(reply["config"]["model"]["threads"], 4);
    }

    #[test]
    fn init_declares_the_model_as_a_required_gguf_resource_under_the_game_directory() {
        let _turn = turn();
        let init = Init {
            config: String::new(),
            game_dir: "/game".to_string(),
            config_dir: "/game/config".to_string(),
            id: "lodger".to_string(),
        };
        let reply = harness().init_with(&init);
        let resource = &reply["resources"][0];

        assert_eq!(resource["id"], "model");
        assert_eq!(
            resource["path"],
            "/game/config/lodger/models/qwen2.5-0.5b-q4_k_m.gguf"
        );
        assert_eq!(resource["size_hint"], 491_400_032u64);
        assert!(resource["title"].as_str().unwrap().contains("491 MB"));
        assert_eq!(resource["magic"], "GGUF");
        assert_eq!(resource["required"], true);
        assert_eq!(resource["license"]["name"], "Apache License 2.0");
        assert!(resource["license"]["text"]
            .as_str()
            .unwrap()
            .contains("Apache License"));
        assert!(resource["url"]
            .as_str()
            .unwrap()
            .starts_with("https://huggingface.co/Qwen/"));
    }

    #[test]
    fn status_says_the_model_is_not_downloaded_until_it_is() {
        let _turn = turn();
        harness().init("");
        let status = harness().status();

        assert_eq!(status["ready"], false);
        assert_eq!(status["loading"], false);
        assert!(status["line"].as_str().unwrap().contains("not downloaded"));
    }

    #[test]
    fn a_stay_opens_ticks_and_saves_through_the_abi() {
        let _turn = turn();
        let h = harness();
        h.init("");
        let info = WorldInfo {
            id: "minecraft:overworld".to_string(),
            seed: 3,
            name: "w".to_string(),
        };
        let world = h.open_world(&info, &[]).unwrap();
        assert!(world.tick(&Now::default()).is_empty());
        let saved = world.save();
        assert_eq!(saved[0], stay::FORMAT);
        world.close();

        let again = h.open_world(&info, &saved).unwrap();
        again.tick(&Now::default());
        let twice = Stay::open(&info, &again.save()).unwrap();
        assert_eq!(twice.ticks, 2);
        again.close();
    }

    /// Needs the real model: `cargo test -p lodger-core -- --ignored`.
    #[test]
    #[ignore = "needs core/models/qwen.gguf, 491 MB, not in the repository"]
    fn with_the_real_model_warm_loads_it_and_status_says_so() {
        let _turn = turn();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/models/qwen.gguf");
        if !std::path::Path::new(path).is_file() {
            return;
        }

        let h = harness();
        let config = format!(r#"{{ "model": {{ "path": "{path}" }} }}"#);
        let reply = h.init(&config);
        assert!(reply.get("error").is_none(), "{reply}");
        assert_eq!(h.status()["ready"], false);

        assert_eq!(h.warm(), serde_json::json!({ "ok": true }));
        let status = h.status();
        assert_eq!(status["ready"], true, "{status}");
        assert_eq!(status["line"], "The model is loaded");
        h.shutdown();
    }

    #[test]
    fn the_model_path_is_under_the_game_directory_unless_absolute() {
        let init = Init {
            config: String::new(),
            game_dir: "/game".to_string(),
            config_dir: "/game/config".to_string(),
            id: "lodger".to_string(),
        };
        let by_size = Config::parse("").unwrap();
        assert_eq!(
            model_path(&by_size, &init),
            PathBuf::from("/game/config/lodger/models/qwen2.5-0.5b-q4_k_m.gguf")
        );
        assert!(model_url(&by_size).contains("0.5B"));

        let large = Config::parse(r#"{ "model": { "size": "3b" } }"#).unwrap();
        assert!(model_path(&large, &init).ends_with("qwen2.5-3b-q4_k_m.gguf"));
        assert!(model_url(&large).contains("3B"));

        let relative = Config::parse(r#"{ "model": { "path": "models/q.gguf" } }"#).unwrap();
        assert_eq!(
            model_path(&relative, &init),
            PathBuf::from("/game/models/q.gguf")
        );

        let absolute =
            Config::parse(r#"{ "model": { "path": "/models/q.gguf", "url": "https://x/q" } }"#)
                .unwrap();
        assert_eq!(
            model_path(&absolute, &init),
            PathBuf::from("/models/q.gguf")
        );
        assert_eq!(model_url(&absolute), "https://x/q");
    }

    #[test]
    fn init_with_bad_json_is_an_error_naming_the_config() {
        let _turn = turn();
        let reply = harness().init("{ nope");

        assert!(reply["error"].as_str().unwrap().starts_with("config: "));
    }
}
