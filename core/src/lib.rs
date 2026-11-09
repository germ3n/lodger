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
pub mod books;
pub mod brain;
pub mod building;
pub mod caves;
pub mod chains;
pub mod config;
pub mod debug;
pub mod elsewhere;
pub mod ending;
pub mod events;
pub mod farm;
pub mod habits;
pub mod lights;
pub mod memory;
pub mod model;
pub mod multiplayer;
pub mod people;
pub mod places;
pub mod rules;
pub mod stay;
pub mod temper;
pub mod tunnels;
pub mod village;
pub mod warped;
pub mod water;
pub mod workshop;

use config::{Config, ModelSize};
use craftkit::{
    Command, DebugReply, DebugRequest, Init, InitReply, License, Mod, ModInfo, Now, Observations,
    Report, Resource, Status, WorldInfo,
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
    /// What the overworld's stay and the other dimensions' tell each other (`elsewhere`).
    pub elsewhere: Mutex<elsewhere::Elsewhere>,
    /// When the target was last online, for `model.idle_minutes`.
    pub idle: Mutex<Idle>,
}

/// The model's idle clock: loaded with no target online for `model.idle_minutes`, it is
/// unloaded, and the next question loads it again. The key was read by nothing before, and the
/// model stayed loaded for as long as the game ran.
#[derive(Default)]
pub struct Idle {
    since: Option<std::time::Instant>,
}

impl Idle {
    /// Whether to unload now: `online` is the target's state at `at`.
    pub fn due(
        &mut self,
        online: bool,
        loaded: bool,
        at: std::time::Instant,
        minutes: u32,
    ) -> bool {
        if online || !loaded {
            self.since = None;
            return false;
        }
        let since = *self.since.get_or_insert(at);
        if at.duration_since(since).as_secs() >= minutes as u64 * 60 {
            self.since = None;
            return true;
        }

        false
    }
}

/// Which size the config asks for; `parse` already refused any other name.
/// The settings the title screen's settings page shows (owner: "the useful
/// ones, grouped"): the model, what it does, pacing, words. The rest stay in the file.
fn settings(config: &Config) -> Vec<craftkit::Setting> {
    use serde_json::json;
    let choice =
        |key: &str, label: &str, group: &str, options: &[&str], value: &str| craftkit::Setting {
            key: key.to_string(),
            label: label.to_string(),
            group: group.to_string(),
            kind: "choice".to_string(),
            options: options.iter().map(|o| o.to_string()).collect(),
            value: json!(value),
            ..Default::default()
        };
    let toggle = |key: &str, label: &str, group: &str, value: bool| craftkit::Setting {
        key: key.to_string(),
        label: label.to_string(),
        group: group.to_string(),
        kind: "toggle".to_string(),
        value: json!(value),
        ..Default::default()
    };
    let number =
        |key: &str, label: &str, group: &str, (min, max, step): (f64, f64, f64), value: f64| {
            craftkit::Setting {
                key: key.to_string(),
                label: label.to_string(),
                group: group.to_string(),
                kind: "number".to_string(),
                min,
                max,
                step,
                value: json!(value),
                ..Default::default()
            }
        };
    vec![
        toggle(
            "model.enabled",
            "Use the model",
            "Model",
            config.model.enabled,
        ),
        choice(
            "model.size",
            "Model size",
            "Model",
            &ModelSize::ALL.map(ModelSize::name),
            &config.model.size,
        ),
        number(
            "model.gpu_share",
            "GPU share",
            "Model",
            (0.0, 1.0, 0.1),
            config.model.gpu_share,
        ),
        number(
            "model.pause_ms",
            "Pause while thinking (ms)",
            "Model",
            (0.0, 100.0, 5.0),
            config.model.pause_ms as f64,
        ),
        choice(
            "preset",
            "Preset",
            "What it does",
            &["calm", "balanced", "intense", "relentless"],
            &config.preset,
        ),
        toggle(
            "events.sounds",
            "Sounds",
            "What it does",
            config.events.sounds,
        ),
        toggle(
            "events.glitches",
            "Screen glitches",
            "What it does",
            config.events.glitches,
        ),
        toggle(
            "events.chat",
            "Chat in your name",
            "What it does",
            config.events.chat,
        ),
        toggle(
            "events.figure",
            "The figure",
            "What it does",
            config.events.figure,
        ),
        number(
            "brain.most_plans",
            "Plans at once",
            "Pacing",
            (1.0, 6.0, 1.0),
            config.brain.most_plans as f64,
        ),
        number(
            "brain.plan_every",
            "Seconds between decisions",
            "Pacing",
            (15.0, 600.0, 15.0),
            config.brain.plan_every as f64,
        ),
        choice(
            "brain.text",
            "Words",
            "Words",
            &["generate", "bank"],
            &config.brain.text,
        ),
        number(
            "brain.text_temperature",
            "Word variety",
            "Words",
            (0.0, 1.5, 0.1),
            config.brain.text_temperature,
        ),
    ]
}

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

/// A seed that differs per start, so two starts do not pick the same way. The world's seed joins
/// it when a world opens.
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
        // A preset picked on the settings screen: the old preset's values out of the file, so
        // the new one's apply, and the file written again to say so.
        let rebased = crate::config::rebased(&init.config_before, &init.config);
        let config = Config::parse(rebased.as_deref().unwrap_or(&init.config))?;
        let size = model_size(&config);
        let path = model_path(&config, init);
        let reply = InitReply {
            config: serde_json::to_value(&config).map_err(|err| err.to_string())?,
            unknown: config.unknown_keys(),
            warnings: config.warnings.clone(),
            settings: settings(&config),
            rewrite: rebased.is_some(),
            resources: vec![Resource {
                id: "model".to_string(),
                title: format!(
                    "Downloading the lodger's model, {} ({} MB)",
                    size.title(),
                    size.megabytes()
                ),
                url: model_url(&config),
                path: path.to_string_lossy().into_owned(),
                size_hint: size.size_hint(),
                magic: "GGUF".to_string(),
                // Played without, it is not needed, and the rule brain decides.
                required: config.model.enabled,
                without: serde_json::json!({ "model": { "enabled": false } }),
                // A file from `model.url` is the player's own, unchecked.
                sha256: if config.model.url.is_empty() {
                    size.sha256().to_string()
                } else {
                    String::new()
                },
                license: License {
                    name: size.license().name.to_string(),
                    url: size.license().url.to_string(),
                    text: size.license().text.to_string(),
                },
                // The other sizes, to pick on the preparing screen before the download
                // (owner: "I want to be able to change the model on that screen").
                choices: ModelSize::ALL
                    .iter()
                    .map(|other| craftkit::Choice {
                        label: format!("{}, {} MB", other.title(), other.megabytes()),
                        config: serde_json::json!({ "model": { "size": other.name() } }),
                        chosen: *other == size,
                        note: other.note().to_string(),
                    })
                    .collect(),
            }],
        };

        let model = Model::new(Settings {
            path,
            threads: config.model.threads,
            context: config.model.context,
            temperature: config.brain.temperature,
            seed: start_seed(init),
            gpu_layers: (config.model.gpu_share.clamp(0.0, 1.0) * size.layers() as f64).round()
                as u32,
            pause_ms: config.model.pause_ms,
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

        Ok((
            Lodger {
                config,
                model,
                log,
                elsewhere: Mutex::new(elsewhere::Elsewhere::default()),
                idle: Mutex::new(Idle::default()),
            },
            reply,
        ))
    }

    fn warm(&self) -> Result<(), String> {
        if !self.config.model.enabled {
            return Ok(());
        }
        self.model.warm()
    }

    fn status(&self) -> Status {
        if !self.config.model.enabled {
            return Status {
                ready: true,
                line: "Playing without the model: the rules decide".to_string(),
                loading: false,
                error: String::new(),
            };
        }
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

        let mut stay = Stay::open(info, saved)?;
        // A new overworld: what the last world's said to the Nether and the End is not this
        // one's (`elsewhere`).
        if info.id == "minecraft:overworld" {
            *self.elsewhere.lock().unwrap_or_else(|err| err.into_inner()) =
                elsewhere::Elsewhere::default();
        }
        // In multiplayer with `multiplayer.pick` at `model`, the brain picks the target; with
        // `first`, the first player seen is it (the memory's default).
        stay.memory.defer_target = self.config.multiplayer.pick == "model";

        Ok(stay)
    }

    fn observe(&self, world: &mut Stay, observations: &Observations) {
        // The day is the stay's own, by the clock of its last tick (observations come just
        // before each tick); before its first tick it has none, and day zero serves. Until
        // now it read a default clock, day zero always: habits, adopted chests, how long
        // they were gone and every reaction's age never moved.
        let day = if world.first_day.is_none() {
            0.0
        } else if world.last_now == Now::default() {
            // Reopened, no tick yet: the day it was at.
            world.last_day
        } else {
            world.day(&world.last_now)
        };
        world.observe(observations, day);
    }

    fn tick(&self, world: &mut Stay, now: &Now) -> Vec<Command> {
        let mut log = self.log.lock().unwrap_or_else(|err| err.into_inner());
        let mut shared = self.elsewhere.lock().unwrap_or_else(|err| err.into_inner());
        if world.world != "minecraft:overworld" {
            if !self.config.enabled {
                return Vec::new();
            }
            return world.tick_elsewhere(now, &mut shared);
        }

        let model = self.config.model.enabled.then_some(&self.model);
        let commands = world.tick_with(now, &self.config, &mut log, model);
        world.share(now, &self.config, &mut shared);
        let idle = self.idle.lock().unwrap_or_else(|err| err.into_inner()).due(
            world.memory.last.online,
            self.model.loaded(),
            std::time::Instant::now(),
            self.config.model.idle_minutes,
        );
        if idle {
            log.note("no one online for a while: the model is unloaded");
            self.model.unload();
        }
        commands
    }

    fn report(&self, world: &mut Stay, reports: &[Report]) {
        let mut log = self.log.lock().unwrap_or_else(|err| err.into_inner());
        world.report(reports, &mut log);
    }

    fn save(&self, world: &Stay) -> Vec<u8> {
        world.save()
    }

    fn debug(&self, world: &mut Stay, request: &DebugRequest) -> DebugReply {
        let mut log = self.log.lock().unwrap_or_else(|err| err.into_inner());
        let now = world.last_now.clone();

        debug::answer(world, request, &self.config, &mut log, &now)
    }
}

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
    fn the_settings_page_gets_the_useful_keys_grouped_with_their_values() {
        let _turn = turn();
        let reply = harness().init_with(&Init {
            config: r#"{ "preset": "calm", "model": { "gpu_share": 0.3 } }"#.to_string(),
            game_dir: "/game".to_string(),
            config_dir: "/game/config".to_string(),
            id: "lodger".to_string(),
            ..Init::default()
        });
        let settings = reply["settings"].as_array().unwrap();
        let find = |key: &str| settings.iter().find(|s| s["key"] == key).unwrap().clone();
        assert_eq!(find("preset")["value"], "calm");
        assert_eq!(find("model.gpu_share")["value"], 0.3);
        assert_eq!(find("model.size")["options"].as_array().unwrap().len(), 4);
        assert_eq!(find("events.chat")["value"], false, "calm turns chat off");
        let groups: std::collections::BTreeSet<&str> = settings
            .iter()
            .map(|s| s["group"].as_str().unwrap())
            .collect();
        assert_eq!(groups.len(), 4, "{groups:?}");
        // Every key is one the config knows: a typo here would be a setting that does nothing.
        let config = serde_json::to_value(crate::config::Config::default()).unwrap();
        for setting in settings {
            let key = setting["key"].as_str().unwrap();
            let found = key.split('.').try_fold(&config, |at, part| at.get(part));
            assert!(found.is_some(), "{key} is not a config key");
        }
    }

    #[test]
    fn the_model_offers_every_size_to_pick_with_the_configured_one_chosen() {
        let _turn = turn();
        let init = |config: &str| Init {
            config: config.to_string(),
            game_dir: "/game".to_string(),
            config_dir: "/game/config".to_string(),
            id: "lodger".to_string(),
            ..Init::default()
        };
        let reply = harness().init_with(&init(""));
        let choices = reply["resources"][0]["choices"].as_array().unwrap().clone();
        assert_eq!(choices.len(), 4);
        let chosen: Vec<&serde_json::Value> =
            choices.iter().filter(|c| c["chosen"] == true).collect();
        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen[0]["config"]["model"]["size"], "1.5b", "the default");
        assert!(choices[2]["label"].as_str().unwrap().contains("3B"));
        assert!(
            choices
                .iter()
                .all(|c| c["note"].as_str().unwrap().contains("GB of VRAM")),
            "each says what it wants"
        );

        let reply = harness().init_with(&init(r#"{ "model": { "size": "0.5b" } }"#));
        assert_eq!(
            reply["resources"][0]["choices"][0]["chosen"], true,
            "the small one now"
        );
        assert!(reply["resources"][0]["path"]
            .as_str()
            .unwrap()
            .ends_with("qwen2.5-0.5b-q4_k_m.gguf"));
    }

    #[test]
    fn init_declares_the_model_as_a_required_gguf_resource_under_the_game_directory() {
        let _turn = turn();
        let init = Init {
            config: String::new(),
            game_dir: "/game".to_string(),
            config_dir: "/game/config".to_string(),
            id: "lodger".to_string(),
            ..Init::default()
        };
        let reply = harness().init_with(&init);
        let resource = &reply["resources"][0];

        assert_eq!(resource["id"], "model");
        assert_eq!(
            resource["path"],
            "/game/config/lodger/models/qwen2.5-1.5b-q4_k_m.gguf"
        );
        assert_eq!(resource["size_hint"], 1_117_320_736u64);
        assert!(resource["title"]
            .as_str()
            .unwrap()
            .contains("Qwen2.5 1.5B (1117 MB)"));
        assert_eq!(resource["sha256"], ModelSize::Medium.sha256());
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
        let labels: Vec<&str> = resource["choices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["label"].as_str().unwrap())
            .collect();
        assert_eq!(labels.len(), 4);
        assert_eq!(labels[3], "Qwen3 4B, 2497 MB");
    }

    #[test]
    fn the_4b_is_offered_with_its_own_license_and_sum_and_a_url_of_ones_own_is_unchecked() {
        let _turn = turn();
        let config = r#"{ "model": { "size": "4b" } }"#;
        let reply = harness().init(config);
        let resource = &reply["resources"][0];
        assert!(resource["path"]
            .as_str()
            .unwrap()
            .ends_with("qwen3-4b-instruct-2507-q4_k_m.gguf"));
        assert_eq!(resource["sha256"], ModelSize::Qwen3.sha256());
        assert!(resource["license"]["url"]
            .as_str()
            .unwrap()
            .contains("Qwen3-4B-Instruct-2507"));
        let size = reply["settings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["key"] == "model.size")
            .unwrap();
        let options = size["options"].as_array().unwrap();
        assert!(options.contains(&serde_json::json!("4b")));

        let own = harness().init(r#"{ "model": { "url": "https://example.org/mine.gguf" } }"#);
        assert_eq!(
            own["resources"][0]["sha256"], "",
            "a file of one's own is not checked"
        );
    }

    #[test]
    fn played_without_the_model_nothing_is_needed_and_it_is_ready() {
        let _turn = turn();
        let reply = harness().init(r#"{ "model": { "enabled": false } }"#);
        assert_eq!(reply["resources"][0]["required"], false);
        assert_eq!(reply["resources"][0]["without"]["model"]["enabled"], false);
        let status = harness().status();
        assert_eq!(status["ready"], true);
        assert!(status["line"]
            .as_str()
            .unwrap()
            .contains("without the model"));

        let reply = harness().init("");
        assert_eq!(reply["resources"][0]["required"], true, "on by default");
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
    #[ignore = "needs core/models/qwen-1.5b.gguf or qwen.gguf, not in the repository"]
    fn with_the_real_model_warm_loads_it_and_status_says_so() {
        let _turn = turn();
        let Some(path) = model::local_model_path() else {
            return;
        };

        let h = harness();
        let config = format!(r#"{{ "model": {{ "path": "{}" }} }}"#, path.display());
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
            ..Init::default()
        };
        let by_size = Config::parse("").unwrap();
        assert_eq!(
            model_path(&by_size, &init),
            PathBuf::from("/game/config/lodger/models/qwen2.5-1.5b-q4_k_m.gguf")
        );
        assert!(model_url(&by_size).contains("1.5B"));
        let small = Config::parse(r#"{ "model": { "size": "0.5b" } }"#).unwrap();
        assert!(model_path(&small, &init).ends_with("qwen2.5-0.5b-q4_k_m.gguf"));

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

    #[test]
    fn the_model_unloads_after_its_idle_minutes_with_no_one_online() {
        let start = std::time::Instant::now();
        let at = |s: u64| start + std::time::Duration::from_secs(s);
        let mut idle = Idle::default();

        assert!(!idle.due(true, true, at(0), 10), "someone is online");
        assert!(!idle.due(false, true, at(1), 10), "the clock starts");
        assert!(!idle.due(false, true, at(500), 10));
        assert!(idle.due(false, true, at(601), 10), "ten minutes and more");
        assert!(
            !idle.due(false, false, at(2000), 10),
            "nothing loaded: nothing to do"
        );
        assert!(!idle.due(true, true, at(2001), 10));
        assert!(
            !idle.due(false, true, at(2002), 10),
            "back online reset the clock"
        );
    }
}
