//! `config/lodger.json`, as the mind reads it.
//!
//! The body reads the file and hands the mind its text; the mind owns the schema, fills the
//! defaults, and hands back the effective config, so the keys live in one place (here and in
//! `docs/config.md`, which a test checks agree). An unknown key is reported, not refused: the
//! body logs it once. The hard rules in `docs/good-to-know.md` are not config and have no key.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Deep-merges `file` over `base`: objects key by key, anything else replaced by the file's.
fn merge_into(base: &mut serde_json::Value, file: &serde_json::Value) {
    match (base, file) {
        (serde_json::Value::Object(base), serde_json::Value::Object(file)) => {
            for (key, value) in file {
                match base.get_mut(key) {
                    Some(slot) if slot.is_object() && value.is_object() => merge_into(slot, value),
                    _ => {
                        base.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (base, file) => *base = file.clone(),
    }
}

/// The four presets: what each sets. Everything not listed is the `balanced` default, which is
/// `Config::default()`; `balanced` sets nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    /// Rare, quiet, nothing that touches the player's screen, chat or sleep, no death at the
    /// end. A stay that is a feeling more than a story.
    Calm,
    /// The design as the docs describe it.
    Balanced,
    /// Denser, bolder, closer, the figures earlier, and the hard ending on normal too.
    Intense,
    /// Something every evening from the first day, every group on, short cooldowns, a stay of
    /// a week, the death on every difficulty. For people who asked.
    Relentless,
}

impl Preset {
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "calm" => Some(Self::Calm),
            "balanced" => Some(Self::Balanced),
            "intense" => Some(Self::Intense),
            "relentless" => Some(Self::Relentless),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Calm => "calm",
            Self::Balanced => "balanced",
            Self::Intense => "intense",
            Self::Relentless => "relentless",
        }
    }

    /// The whole config this preset means, before the file's own keys are laid over it.
    pub fn config(self) -> Config {
        let mut config = Config {
            preset: self.name().to_string(),
            ..Config::default()
        };

        match self {
            Self::Balanced => {}
            Self::Calm => {
                config.stages.events_per_day = [0.5, 1.0, 1.5, 2.0, 1.0];
                config.stages.least_days = [2, 3, 3, 4, 1];
                config.stages.most_days = [5, 7, 7, 9, 3];
                config.events.cooldown_scale = 2.0;
                config.events.min_distance = 80;
                config.brain.most_plans = 2;
                config.events.chat = false;
                config.events.disabled = [
                    "refuse_rest",
                    "bed_occupied",
                    "wake",
                    "woke_elsewhere",
                    "other_house",
                    "in_doorway",
                    "at_the_chest",
                    "in_your_bed",
                    "lights_fail",
                    "red_rain",
                    "your_voice",
                    "knock",
                    "stare",
                    "sign_word",
                    "tunnel_toward",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect();
                config.brain.event_threshold = 0.5;
                config.stages.ending = "leave".to_string();
            }
            Self::Intense => {
                config.stages.events_per_day = [2.0, 3.0, 5.0, 8.0, 4.0];
                config.stages.least_days = [1, 1, 2, 2, 1];
                config.stages.most_days = [2, 4, 4, 6, 3];
                config.events.cooldown_scale = 0.75;
                config.events.min_distance = 48;
                config.brain.most_plans = 4;
                config.brain.event_threshold = 0.3;
                config.stages.ending = "take".to_string();
            }
            Self::Relentless => {
                config.stages.events_per_day = [4.0, 6.0, 8.0, 12.0, 6.0];
                config.stages.least_days = [1, 1, 1, 1, 1];
                config.stages.most_days = [1, 2, 2, 3, 3];
                config.stages.start_after_days = 0;
                config.events.cooldown_scale = 0.5;
                config.events.min_distance = 32;
                config.brain.most_plans = 5;
                config.events.offered = 12;
                config.brain.event_threshold = 0.25;
                config.brain.react_threshold = 0.4;
                config.brain.plan_every = 45;
                config.stages.ending = "take".to_string();
            }
        }

        config
    }
}

/// The models the lodger can run: three sizes of Qwen2.5-Instruct, each Qwen's own `q4_k_m`
/// GGUF, and Qwen3-4B-Instruct-2507 (owner: "how about we offer this too?"), whose
/// `q4_k_m` GGUF comes from unsloth's repository, Qwen publishing none of its own that a
/// download can reach. The 0.5B is the file `~/engine` runs. Sizes and SHA-256 sums are the
/// files' own as Hugging Face's servers give them (`x-linked-size`, `x-linked-etag`),
/// so the bar is right from the start and the host checks the file it fetched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelSize {
    Small,
    Medium,
    Large,
    /// Qwen3-4B-Instruct-2507: newer, a better reader, the biggest file.
    Qwen3,
}

/// A model's license as the preparing screen shows it before the download.
pub struct ModelLicense {
    pub name: &'static str,
    pub url: &'static str,
    pub text: &'static str,
}

const APACHE_TEXT: &str = "The lodger thinks with a Qwen2.5-Instruct language model made by \
Alibaba Cloud's Qwen team and released under the Apache License 2.0. It will be downloaded once \
from Hugging Face, from Qwen's own repository, and kept on this computer. Nothing is sent \
anywhere; the model runs here. Copyright 2024 Alibaba Cloud. Licensed under the Apache License, \
Version 2.0; the full text is at the link below. By continuing you accept those terms for your \
use of the model.";

const RESEARCH_TEXT: &str = "The lodger thinks with Qwen2.5-3B-Instruct, a language model made \
by Alibaba Cloud's Qwen team and released under the Qwen Research License Agreement, not the \
Apache License of the smaller sizes: it allows non-commercial use only, which it defines as \
research or evaluation, and commercial use needs a license from Alibaba Cloud. It will be \
downloaded once from Hugging Face, from Qwen's own repository, and kept on this computer. \
Nothing is sent anywhere; the model runs here. The full agreement is at the link below. By \
continuing you accept its terms for your use of the model.";

const QWEN3_TEXT: &str = "The lodger thinks with Qwen3-4B-Instruct-2507, a language model made \
by Alibaba Cloud's Qwen team and released under the Apache License 2.0. It will be downloaded \
once from Hugging Face, from unsloth's repository, which offers Qwen's model in the GGUF form \
the lodger runs, and kept on this computer. Nothing is sent anywhere; the model runs here. \
Copyright 2025 Alibaba Cloud. Licensed under the Apache License, Version 2.0; the full text is \
at the link below. By continuing you accept those terms for your use of the model.";

impl ModelSize {
    /// Every model, smallest first: the preparing screen's choices.
    pub const ALL: [ModelSize; 4] = [
        ModelSize::Small,
        ModelSize::Medium,
        ModelSize::Large,
        ModelSize::Qwen3,
    ];

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|size| size.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Small => "0.5b",
            Self::Medium => "1.5b",
            Self::Large => "3b",
            Self::Qwen3 => "4b",
        }
    }

    /// What the download screen says under the size: how it differs, and the VRAM it wants with
    /// the whole model on the GPU (the default), which is RAM instead without a GPU.
    pub fn note(self) -> &'static str {
        match self {
            Self::Small => "The smallest and quickest: plainer choices and plainer words. Wants about 0.6 GB of VRAM.",
            Self::Medium => "The default: sharper judgment and better words. Wants about 1.3 GB of VRAM.",
            Self::Large => "Better again, and slower; Qwen's research license, not for commercial use. Wants about 2.4 GB of VRAM.",
            Self::Qwen3 => "The best reader of the four, the newest. Wants about 2.9 GB of VRAM.",
        }
    }

    /// The model's name for the screen: `Qwen2.5 1.5B`, `Qwen3 4B`.
    pub fn title(self) -> &'static str {
        match self {
            Self::Small => "Qwen2.5 0.5B",
            Self::Medium => "Qwen2.5 1.5B",
            Self::Large => "Qwen2.5 3B",
            Self::Qwen3 => "Qwen3 4B",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            Self::Small => "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf",
            Self::Medium => "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf",
            Self::Large => "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf",
            Self::Qwen3 => "https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/Qwen3-4B-Instruct-2507-Q4_K_M.gguf",
        }
    }

    /// How many layers the model has: what `model.gpu_share` is a share of.
    pub fn layers(self) -> u32 {
        match self {
            Self::Small => 24,
            Self::Medium => 28,
            Self::Large | Self::Qwen3 => 36,
        }
    }

    /// The file name under `config/lodger/models/`.
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Small => "qwen2.5-0.5b-q4_k_m.gguf",
            Self::Medium => "qwen2.5-1.5b-q4_k_m.gguf",
            Self::Large => "qwen2.5-3b-q4_k_m.gguf",
            Self::Qwen3 => "qwen3-4b-instruct-2507-q4_k_m.gguf",
        }
    }

    /// The file's length in bytes, for the bar.
    pub fn size_hint(self) -> u64 {
        match self {
            Self::Small => 491_400_032,
            Self::Medium => 1_117_320_736,
            Self::Large => 2_104_932_768,
            Self::Qwen3 => 2_497_281_120,
        }
    }

    /// The file's SHA-256, which the host checks after the download.
    pub fn sha256(self) -> &'static str {
        match self {
            Self::Small => "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db",
            Self::Medium => "6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e",
            Self::Large => "626b4a6678b86442240e33df819e00132d3ba7dddfe1cdc4fbb18e0a9615c62d",
            Self::Qwen3 => "3605803b982cb64aead44f6c1b2ae36e3acdb41d8e46c8a94c6533bc4c67e597",
        }
    }

    /// Its license: Apache 2.0, but for the 3B, which is under Qwen's research license (the
    /// screen said Apache for every size until now).
    pub fn license(self) -> ModelLicense {
        match self {
            Self::Small => ModelLicense {
                name: "Apache License 2.0",
                url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/blob/main/LICENSE",
                text: APACHE_TEXT,
            },
            Self::Medium => ModelLicense {
                name: "Apache License 2.0",
                url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/blob/main/LICENSE",
                text: APACHE_TEXT,
            },
            Self::Large => ModelLicense {
                name: "Qwen Research License",
                url: "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/blob/main/LICENSE",
                text: RESEARCH_TEXT,
            },
            Self::Qwen3 => ModelLicense {
                name: "Apache License 2.0",
                url: "https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507/blob/main/LICENSE",
                text: QWEN3_TEXT,
            },
        }
    }

    /// For the screen: `Downloading the lodger's model (491 MB)`.
    pub fn megabytes(self) -> u64 {
        self.size_hint() / 1_000_000
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    /// `calm`, `balanced`, `intense` or `relentless`: a set of values for the keys below that
    /// decide how much happens, how bold it is and what is left out. A key the file sets
    /// explicitly wins over the preset; the file written back shows the preset's values filled
    /// in, so a player sees what `balanced` means. See `docs/config.md`.
    pub preset: String,
    pub log: Log,
    pub model: Model,
    pub brain: Brain,
    pub stages: Stages,
    pub events: Events,
    pub multiplayer: Multiplayer,
    /// Keys the schema does not know, kept so the body can log them. Never written back.
    #[serde(flatten, skip_serializing)]
    pub unknown: BTreeMap<String, serde_json::Value>,
    /// Values of the wrong type, each mended or left at the preset's (`Config::mend`), for
    /// the host to log. Never written back.
    #[serde(skip)]
    pub warnings: Vec<String>,
    /// Keys inside a known section that the schema does not know (`model.sise`), as paths.
    #[serde(skip)]
    pub unknown_inside: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Log {
    pub decisions: bool,
    pub keep_mb: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Model {
    /// Whether the model is used at all: off, nothing is downloaded and the rule brain decides
    /// (the download screen's "Play without it", or the settings; owner).
    pub enabled: bool,
    /// `0.5b`, `1.5b` or `3b`: which Qwen2.5-Instruct to run. See `docs/model.md`.
    pub size: String,
    /// Where to fetch from. Empty means Qwen's own file for `size`.
    pub url: String,
    /// Where the file lives, relative to the game directory unless absolute. Empty means
    /// `config/lodger/models/<the size's file name>`.
    pub path: String,
    pub threads: u32,
    pub context: u32,
    pub idle_minutes: u32,
    /// The share of the model's layers on the GPU: 0 is the CPU alone, 1 all of them, the default
    /// (owner: "by default the full model should run on the gpu"). It used to be half.
    pub gpu_share: f64,
    /// Milliseconds the model rests between each piece of its work while it thinks (a chunk
    /// of a question, a token): higher leaves more of the GPU and the CPU to the game at any
    /// moment, and its answers come slower. 1 by default, at most 100 (owner).
    pub pause_ms: u32,
}

// Fractions are f64, not f32: a 0.4 written back to the player's file as 0.4000000059604645 is
// what an f32 looks like once serde_json prints it, and nothing here needs the smaller type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Brain {
    pub plan_every: u32,
    /// How many plans it holds at once (`docs/brain.md`, Plans).
    pub most_plans: u32,
    /// How freely the model writes a line: 0 the likeliest words every time, 0.8 by default
    /// (`docs/brain.md`, Text).
    pub text_temperature: f64,
    pub event_threshold: f64,
    pub react_threshold: f64,
    pub queue: u32,
    pub force_rules: bool,
    /// `bank`: signs, books and chat lines are chosen from the phrase bank by scoring;
    /// `generate`: the model writes up to three words itself, constrained and filtered. An
    /// experiment; see `docs/brain.md`.
    pub text: String,
    /// How the answer is picked from the model's scores: 0 is always the best choice, which
    /// answers the same situation the same way every time; above 0 a seeded draw weighted by
    /// the scores, sharper near 0 and flatter near 1. See `docs/brain.md`.
    pub temperature: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stages {
    pub least_days: [u32; 5],
    pub most_days: [u32; 5],
    pub events_per_day: [f64; 5],
    pub start_after_days: u32,
    pub ending: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Events {
    /// Whether how they treat it moves its temper, and its temper the pace (`temper.rs`).
    pub temper: bool,
    pub min_distance: u32,
    pub disabled: Vec<String>,
    pub offered: u32,
    pub cooldown_scale: f64,
    pub sounds: bool,
    pub figure: bool,
    pub glitches: bool,
    /// Whether it may show in the player list, in join messages and in chat, as the target, to
    /// the target.
    pub chat: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Multiplayer {
    pub pick: String,
    pub retarget: bool,
    pub absent_days: u32,
    pub witness_traces: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: "balanced".to_string(),
            log: Log::default(),
            model: Model::default(),
            brain: Brain::default(),
            stages: Stages::default(),
            events: Events::default(),
            multiplayer: Multiplayer::default(),
            unknown: BTreeMap::new(),
            warnings: Vec::new(),
            unknown_inside: Vec::new(),
        }
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            decisions: true,
            keep_mb: 16,
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            enabled: true,
            size: "1.5b".to_string(),
            url: String::new(),
            path: String::new(),
            threads: 2,
            context: 1536,
            idle_minutes: 10,
            pause_ms: 1,
            gpu_share: 1.0,
        }
    }
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            plan_every: 90,
            most_plans: 3,
            text_temperature: 0.8,
            event_threshold: 0.4,
            react_threshold: 0.55,
            queue: 64,
            force_rules: false,
            text: "generate".to_string(),
            temperature: 0.7,
        }
    }
}

impl Default for Stages {
    fn default() -> Self {
        Self {
            least_days: [1, 2, 2, 3, 1],
            most_days: [3, 5, 5, 7, 3],
            events_per_day: [1.0, 2.0, 4.0, 6.0, 3.0],
            start_after_days: 3,
            ending: "difficulty".to_string(),
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Self {
            temper: true,
            min_distance: 64,
            disabled: Vec::new(),
            offered: 8,
            cooldown_scale: 1.0,
            sounds: true,
            figure: true,
            glitches: true,
            chat: true,
        }
    }
}

impl Default for Multiplayer {
    fn default() -> Self {
        Self {
            pick: "model".to_string(),
            retarget: true,
            absent_days: 7,
            witness_traces: true,
        }
    }
}

/// Numbers written whole with a point (`2.0`) read as whole numbers, so a count written by a
/// settings screen or by hand that way still loads: `2.0` for `brain.most_plans` refused the
/// whole config, and the mod with it (the dev client). A fraction keeps its point.
/// A config file whose preset was just changed, with the old preset's values taken out: every
/// value the file holds that is the old preset's own, so the new preset's apply. A file holds
/// every key, written at the first start, and picking `calm` on the settings screen used to
/// change the name alone, every value staying `balanced`'s. None when the preset is the same,
/// or either text is not a config.
pub fn rebased(before: &str, now: &str) -> Option<String> {
    let preset_of = |v: &serde_json::Value| {
        v.get("preset")
            .and_then(|p| p.as_str())
            .unwrap_or("balanced")
            .to_string()
    };
    let before: serde_json::Value = serde_json::from_str(before).ok()?;
    let mut now: serde_json::Value = serde_json::from_str(now).ok()?;
    let (old, new) = (preset_of(&before), preset_of(&now));
    if old == new {
        return None;
    }
    let mut base = serde_json::to_value(Preset::named(&old)?.config()).ok()?;
    whole_numbers(&mut base);
    whole_numbers(&mut now);
    strip_same(&mut now, &base);
    now.as_object_mut()?
        .insert("preset".to_string(), serde_json::Value::from(new));

    serde_json::to_string(&now).ok()
}

/// Takes out of `file` every value equal to `base`'s at the same keys; objects left empty go.
fn strip_same(file: &mut serde_json::Value, base: &serde_json::Value) {
    let (Some(file), Some(base)) = (file.as_object_mut(), base.as_object()) else {
        return;
    };
    file.retain(|key, value| match base.get(key) {
        Some(b) if value.is_object() && b.is_object() => {
            strip_same(value, b);
            value.as_object().is_some_and(|o| !o.is_empty())
        }
        Some(b) => value != b,
        None => true,
    });
}

fn whole_numbers(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Number(number) => {
            if let Some(float) = number.as_f64().filter(|_| number.is_f64()) {
                if float.fract() == 0.0 && float.abs() < 9.0e15 {
                    *value = serde_json::Value::from(float as i64);
                }
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(whole_numbers),
        serde_json::Value::Object(map) => map.values_mut().for_each(whole_numbers),
        _ => {}
    }
}

/// The paths in `file` under a known section that the schema has no place for.
fn unknown_inside(file: &serde_json::Value) -> Vec<String> {
    let Ok(schema) = serde_json::to_value(Config::default()) else {
        return Vec::new();
    };
    let mut leaves = Vec::new();
    collect_leaves(file, &mut Vec::new(), &mut leaves);
    let mut out: Vec<String> = Vec::new();
    for (path, _) in leaves {
        // A top-level key unknown is the flattened map's; only a known section's are here.
        if path.len() < 2 || schema.get(&path[0]).is_none() {
            continue;
        }
        let mut at = &schema;
        for (depth, key) in path.iter().enumerate() {
            match at.get(key) {
                Some(next) => at = next,
                None => {
                    let joined = path[..=depth].join(".");
                    if !out.contains(&joined) {
                        out.push(joined);
                    }
                    break;
                }
            }
            if !at.is_object() {
                break;
            }
        }
    }

    out
}

/// Every value in a config file with its path of keys; a list is one value.
fn collect_leaves(
    value: &serde_json::Value,
    path: &mut Vec<String>,
    out: &mut Vec<(Vec<String>, serde_json::Value)>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                path.push(key.clone());
                collect_leaves(value, path, out);
                path.pop();
            }
        }
        _ if !path.is_empty() => out.push((path.clone(), value.clone())),
        _ => {}
    }
}

/// `value` under its path of keys: `["brain", "most_plans"]` and `2` as `{"brain":{"most_plans":2}}`.
fn nested(path: &[String], value: serde_json::Value) -> serde_json::Value {
    path.iter().rev().fold(
        value,
        |inner, key| serde_json::json!({ key.clone(): inner }),
    )
}

/// Sets the value at a path, or takes it out with `None`.
fn set_at(root: &mut serde_json::Value, path: &[String], value: Option<serde_json::Value>) {
    let Some((last, parents)) = path.split_last() else {
        return;
    };
    let mut at = root;
    for key in parents {
        match at.get_mut(key) {
            Some(next) => at = next,
            None => return,
        }
    }
    if let Some(map) = at.as_object_mut() {
        match value {
            Some(value) => {
                map.insert(last.clone(), value);
            }
            None => {
                map.remove(last);
            }
        }
    }
}

/// What a value of the wrong type may have meant: text that reads as a number or as a yes or
/// no, a number or a yes or no as text. None when it means nothing else.
fn coerced(value: &serde_json::Value) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::String(text) => {
            let text = text.trim();
            match text.to_lowercase().as_str() {
                "true" | "yes" | "on" => return Some(serde_json::Value::Bool(true)),
                "false" | "no" | "off" => return Some(serde_json::Value::Bool(false)),
                _ => {}
            }
            if let Ok(whole) = text.parse::<i64>() {
                return Some(serde_json::Value::from(whole));
            }
            text.parse::<f64>()
                .ok()
                .filter(|f| f.is_finite())
                .map(serde_json::Value::from)
        }
        serde_json::Value::Number(number) => Some(serde_json::Value::String(number.to_string())),
        serde_json::Value::Bool(yes) => Some(serde_json::Value::String(yes.to_string())),
        _ => None,
    }
}

impl Config {
    /// Reads the file's text. An empty text is the defaults, as a missing file is. The preset's
    /// values are laid under the file's: a key the file names keeps its value, the rest take
    /// the preset's, so `{ "preset": "calm" }` alone is a whole calm config.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.trim().is_empty() {
            return Ok(Self::default());
        }

        let mut file: serde_json::Value =
            serde_json::from_str(text).map_err(|err| format!("config: {err}"))?;
        whole_numbers(&mut file);
        let preset_name = file
            .get("preset")
            .and_then(|p| p.as_str())
            .unwrap_or("balanced");
        let preset = Preset::named(preset_name)
            .ok_or_else(|| "preset must be calm, balanced, intense or relentless".to_string())?;
        let base = serde_json::to_value(preset.config()).map_err(|err| err.to_string())?;
        let mut merged = base.clone();
        merge_into(&mut merged, &file);

        let mut config: Self = match serde_json::from_value(merged) {
            Ok(config) => config,
            Err(_) => {
                let (kept, warnings) = Self::mend(&base, &file);
                let mut merged = base;
                merge_into(&mut merged, &kept);
                let mut config: Self =
                    serde_json::from_value(merged).map_err(|err| format!("config: {err}"))?;
                config.warnings = warnings;
                config
            }
        };
        config.check()?;
        config.warnings.sort();
        config.unknown_inside = unknown_inside(&file);

        Ok(config)
    }

    /// The file with each value of the wrong type mended or left out, one key at a time over
    /// the preset: text that reads as the number or the yes/no the key wants (`"2"`, `"true"`)
    /// is taken as that, anything else falls back to the preset's value, with a warning naming
    /// the key. One bad key no longer refuses the whole config, and the mod with it (owner: "make that more robust when it comes to parsing").
    fn mend(
        base: &serde_json::Value,
        file: &serde_json::Value,
    ) -> (serde_json::Value, Vec<String>) {
        let mut kept = file.clone();
        let mut warnings = Vec::new();
        let mut leaves = Vec::new();
        collect_leaves(file, &mut Vec::new(), &mut leaves);
        let reads = |path: &[String], value: &serde_json::Value| {
            let mut one = base.clone();
            merge_into(&mut one, &nested(path, value.clone()));
            serde_json::from_value::<Self>(one).err()
        };
        for (path, value) in leaves {
            let Some(err) = reads(&path, &value) else {
                continue;
            };
            let key = path.join(".");
            match coerced(&value).filter(|c| reads(&path, c).is_none()) {
                Some(mended) => {
                    warnings.push(format!("{key}: read {value} as {mended}"));
                    set_at(&mut kept, &path, Some(mended));
                }
                None => {
                    let reason = err.to_string();
                    let reason = reason.split(" at line").next().unwrap_or(&reason);
                    warnings.push(format!("{key}: {reason}; the default kept"));
                    set_at(&mut kept, &path, None);
                }
            }
        }

        (kept, warnings)
    }

    /// The bounds a value must keep, with the reason beside each.
    fn check(&self) -> Result<(), String> {
        // A context too small cannot hold the frame and one question; llama.cpp wants a
        // multiple of 32 anyway.
        if self.model.context < 256 {
            return Err("model.context must be at least 256".to_string());
        }

        if self.model.pause_ms > 100 {
            return Err("model.pause_ms must be 100 or less".to_string());
        }
        if self.model.threads == 0 {
            return Err("model.threads must be at least 1".to_string());
        }

        if ModelSize::named(&self.model.size).is_none() {
            return Err("model.size must be 0.5b, 1.5b, 3b or 4b".to_string());
        }

        if !(0.0..=2.0).contains(&self.brain.temperature) {
            return Err("brain.temperature must be between 0 and 2".to_string());
        }

        for (name, value) in [
            ("brain.event_threshold", self.brain.event_threshold),
            ("brain.react_threshold", self.brain.react_threshold),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(format!("{name} must be between 0 and 1"));
            }
        }

        if self.brain.queue == 0 {
            return Err("brain.queue must be at least 1".to_string());
        }
        // 0 retargeted the moment the target logged off, at the next stage.
        if self.multiplayer.absent_days == 0 {
            return Err("multiplayer.absent_days must be at least 1".to_string());
        }
        // 0 asked the model again the moment each answer came back, nearly without pause.
        if self.brain.plan_every < 15 {
            return Err("brain.plan_every must be at least 15".to_string());
        }

        for stage in 0..5 {
            if self.stages.least_days[stage] > self.stages.most_days[stage] {
                return Err(format!(
                    "stages.least_days[{stage}] is more than stages.most_days[{stage}]"
                ));
            }

            if self.stages.events_per_day[stage] < 0.0 {
                return Err(format!("stages.events_per_day[{stage}] is negative"));
            }
        }

        if !["difficulty", "leave", "take", "keep"].contains(&self.stages.ending.as_str()) {
            return Err("stages.ending must be difficulty, leave, take or keep".to_string());
        }

        if !["bank", "generate"].contains(&self.brain.text.as_str()) {
            return Err("brain.text must be bank or generate".to_string());
        }

        // Zero or one offered leaves the model nothing to choose; the prompt grows past twelve.
        if !(2..=12).contains(&self.events.offered) {
            return Err("events.offered must be between 2 and 12".to_string());
        }

        if self.events.cooldown_scale <= 0.0 {
            return Err("events.cooldown_scale must be positive".to_string());
        }

        Ok(())
    }

    /// The keys the file had that the schema does not know: at the top (`colour`) and inside
    /// a known section as a path (`model.sise`). Those inside used to pass silently.
    pub fn unknown_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.unknown.keys().cloned().collect();
        keys.extend(self.unknown_inside.iter().cloned());
        keys
    }

    /// The effective config, every key present, as the body writes it back to a missing file.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_is_the_defaults() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
        assert_eq!(Config::parse("  \n").unwrap(), Config::default());
    }

    #[test]
    fn a_missing_key_takes_its_default_and_a_given_one_is_kept() {
        let config = Config::parse(r#"{ "brain": { "plan_every": 30 } }"#).unwrap();

        assert_eq!(config.brain.plan_every, 30);
        assert_eq!(config.brain.queue, 64);
        assert_eq!(config.model.context, 1536);
    }

    #[test]
    fn an_unknown_top_level_key_is_reported_and_not_written_back() {
        let config = Config::parse(r#"{ "enabled": true, "colour": "red" }"#).unwrap();

        assert_eq!(config.unknown_keys(), vec!["colour".to_string()]);
        assert!(!config.to_json().contains("colour"));
    }

    #[test]
    fn an_unknown_key_inside_a_section_is_reported_too() {
        let config =
            Config::parse(r#"{ "brain": { "colour": "red" }, "model": { "sise": "3b" } }"#)
                .unwrap();

        assert_eq!(
            config.unknown_keys(),
            vec!["brain.colour".to_string(), "model.sise".to_string()]
        );
        assert!(
            Config::parse(&Config::default().to_json())
                .unwrap()
                .unknown_keys()
                .is_empty(),
            "the file as written has none"
        );
    }

    #[test]
    fn a_whole_number_written_with_a_point_still_loads() {
        let config = Config::parse(
            r#"{ "brain": { "most_plans": 2.0, "text_temperature": 0.5 }, "model": { "gpu_share": 1.0 } }"#,
        )
        .unwrap();

        assert_eq!(config.brain.most_plans, 2);
        assert_eq!(config.model.gpu_share, 1.0);
        assert_eq!(
            config.brain.text_temperature, 0.5,
            "a fraction keeps its point"
        );
    }

    #[test]
    fn a_value_of_the_wrong_type_falls_back_alone_with_a_warning() {
        let config = Config::parse(
            r#"{ "preset": "calm", "brain": { "most_plans": "lots", "plan_every": 30 },
                 "events": { "chat": 3 } }"#,
        )
        .unwrap();
        let calm = Config::parse(r#"{ "preset": "calm" }"#).unwrap();

        assert_eq!(
            config.brain.most_plans, calm.brain.most_plans,
            "the preset's"
        );
        assert_eq!(config.events.chat, calm.events.chat);
        assert_eq!(
            config.brain.plan_every, 30,
            "the good key beside it is kept"
        );
        assert_eq!(config.warnings.len(), 2, "{:?}", config.warnings);
        assert!(
            config.warnings[0].starts_with("brain.most_plans: invalid type"),
            "{:?}",
            config.warnings
        );
        assert!(
            config.warnings[1].starts_with("events.chat: "),
            "{:?}",
            config.warnings
        );
        assert!(!config.to_json().contains("warnings"), "never written back");
    }

    #[test]
    fn text_that_reads_as_the_type_is_taken_as_it() {
        let config = Config::parse(
            r#"{ "brain": { "most_plans": " 2 " }, "events": { "chat": "off" }, "model": { "gpu_share": "0.25" } }"#,
        )
        .unwrap();

        assert_eq!(config.brain.most_plans, 2);
        assert!(!config.events.chat);
        assert_eq!(config.model.gpu_share, 0.25);
        assert_eq!(
            config.warnings.len(),
            3,
            "each mended key is named: {:?}",
            config.warnings
        );
    }

    #[test]
    fn a_good_file_has_no_warnings() {
        let config = Config::parse(r#"{ "brain": { "most_plans": 2 } }"#).unwrap();

        assert!(config.warnings.is_empty());
    }

    #[test]
    fn bad_json_is_an_error_naming_the_config() {
        let err = Config::parse("{ nope").unwrap_err();

        assert!(err.starts_with("config: "), "{err}");
    }

    #[test]
    fn values_out_of_bounds_are_refused_by_name() {
        for (text, name) in [
            (r#"{ "model": { "context": 64 } }"#, "model.context"),
            (r#"{ "model": { "threads": 0 } }"#, "model.threads"),
            (r#"{ "model": { "pause_ms": 500 } }"#, "model.pause_ms"),
            (r#"{ "model": { "size": "7b" } }"#, "model.size"),
            (r#"{ "brain": { "temperature": 3 } }"#, "brain.temperature"),
            (
                r#"{ "brain": { "event_threshold": 1.5 } }"#,
                "brain.event_threshold",
            ),
            (r#"{ "brain": { "queue": 0 } }"#, "brain.queue"),
            (r#"{ "brain": { "plan_every": 0 } }"#, "brain.plan_every"),
            (
                r#"{ "stages": { "least_days": [9, 2, 2, 3, 1] } }"#,
                "stages.least_days[0]",
            ),
            (r#"{ "stages": { "ending": "win" } }"#, "stages.ending"),
            (r#"{ "brain": { "text": "prose" } }"#, "brain.text"),
            (r#"{ "events": { "offered": 1 } }"#, "events.offered"),
            (
                r#"{ "events": { "cooldown_scale": 0 } }"#,
                "events.cooldown_scale",
            ),
        ] {
            let err = Config::parse(text).unwrap_err();
            assert!(err.contains(name), "{text}: {err}");
        }
    }

    #[test]
    fn fractions_are_written_back_as_the_player_typed_them() {
        let text = Config::default().to_json();

        assert!(text.contains("\"event_threshold\": 0.4,"), "{text}");
        assert!(text.contains("\"react_threshold\": 0.55,"), "{text}");
        assert!(text.contains("\"cooldown_scale\": 1.0,"), "{text}");
    }

    #[test]
    fn a_preset_alone_is_a_whole_config_and_a_named_key_wins_over_it() {
        let calm = Config::parse(r#"{ "preset": "calm" }"#).unwrap();
        assert_eq!(calm.preset, "calm");
        assert_eq!(calm.stages.events_per_day, [0.5, 1.0, 1.5, 2.0, 1.0]);
        assert!(!calm.events.chat);
        assert!(calm.events.disabled.contains(&"in_your_bed".to_string()));

        let mixed = Config::parse(r#"{ "preset": "calm", "events": { "chat": true } }"#).unwrap();
        assert!(mixed.events.chat, "the file's key wins");
        assert_eq!(mixed.events.cooldown_scale, 2.0, "the rest is the preset's");

        assert_eq!(
            Config::parse(r#"{ "preset": "balanced" }"#).unwrap(),
            Config::default()
        );
        assert!(Config::parse(r#"{ "preset": "cosy" }"#)
            .unwrap_err()
            .contains("preset"));
    }

    #[test]
    fn every_preset_passes_its_own_checks_and_balanced_is_the_default() {
        for preset in [
            Preset::Calm,
            Preset::Balanced,
            Preset::Intense,
            Preset::Relentless,
        ] {
            let text = preset.config().to_json();
            let parsed =
                Config::parse(&text).unwrap_or_else(|err| panic!("{}: {err}", preset.name()));
            assert_eq!(parsed.preset, preset.name());
        }

        assert_eq!(Preset::Balanced.config(), Config::default());
        assert!(
            Preset::Relentless.config().stages.events_per_day[0]
                > Preset::Intense.config().stages.events_per_day[0]
        );
        assert!(
            Preset::Intense.config().stages.events_per_day[0]
                > Preset::Calm.config().stages.events_per_day[0]
        );
    }

    #[test]
    fn the_model_sizes_point_at_qwen_s_own_files() {
        for (name, size) in [
            ("0.5b", ModelSize::Small),
            ("1.5b", ModelSize::Medium),
            ("3b", ModelSize::Large),
        ] {
            assert_eq!(ModelSize::named(name), Some(size));
            assert_eq!(size.name(), name);
            assert!(
                size.url()
                    .starts_with("https://huggingface.co/Qwen/Qwen2.5-"),
                "{}",
                size.url()
            );
            assert!(size.url().ends_with("-instruct-q4_k_m.gguf"));
            assert!(size.file_name().ends_with(".gguf"));
        }
        assert_eq!(ModelSize::named("4b"), Some(ModelSize::Qwen3));
        assert!(ModelSize::Qwen3
            .url()
            .ends_with("Qwen3-4B-Instruct-2507-Q4_K_M.gguf"));
        assert_eq!(ModelSize::named("7b"), None);

        for pair in ModelSize::ALL.windows(2) {
            assert!(pair[0].size_hint() < pair[1].size_hint(), "smallest first");
        }
        for size in ModelSize::ALL {
            assert_eq!(size.sha256().len(), 64, "{}", size.name());
            assert!(size.sha256().chars().all(|c| c.is_ascii_hexdigit()));
            let names: Vec<&str> = ModelSize::ALL
                .iter()
                .filter(|o| o.file_name() == size.file_name())
                .map(|o| o.name())
                .collect();
            assert_eq!(names, [size.name()], "each its own file");
        }
        assert_eq!(ModelSize::Small.megabytes(), 491);
        assert_eq!(ModelSize::Large.license().name, "Qwen Research License");
        assert!(ModelSize::Large.license().text.contains("non-commercial"));
        assert_eq!(ModelSize::Qwen3.license().name, "Apache License 2.0");
        assert!(ModelSize::Qwen3.license().text.contains("unsloth"));
    }

    #[test]
    fn the_defaults_round_trip_through_json() {
        let text = Config::default().to_json();

        assert_eq!(Config::parse(&text).unwrap(), Config::default());
    }

    /// Every key in docs/config.md is in the schema and every key in the schema is in the docs,
    /// as `tests/toolchains.rs` in ~/chainv4 keeps its pinned versions agreeing.
    #[test]
    fn the_docs_and_the_schema_name_the_same_keys() {
        let docs =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/config.md"))
                .expect("docs/config.md beside the crate");
        let mut documented: Vec<String> = docs
            .lines()
            .filter(|line| line.starts_with("| `"))
            .filter_map(|line| line.split('`').nth(1))
            .filter(|key| key.contains('.') || *key == "enabled" || *key == "preset")
            .map(str::to_string)
            .collect();
        documented.sort();

        let json: serde_json::Value = serde_json::from_str(&Config::default().to_json()).unwrap();
        let mut schema = Vec::new();
        for (section, value) in json.as_object().unwrap() {
            match value.as_object() {
                Some(fields) => schema.extend(fields.keys().map(|key| format!("{section}.{key}"))),
                None => schema.push(section.clone()),
            }
        }
        schema.sort();

        assert_eq!(documented, schema);
    }

    #[test]
    fn a_preset_picked_over_a_file_of_every_key_takes_effect_and_the_player_s_own_stay() {
        // The file as the first start writes it: every key, balanced's values, and one the
        // player set themselves.
        let mut written = serde_json::to_value(Config::default()).unwrap();
        written["model"]["threads"] = serde_json::json!(6);
        let before = written.to_string();
        written["preset"] = serde_json::json!("calm");
        let now = written.to_string();

        let text = rebased(&before, &now).expect("the preset changed");
        let config = Config::parse(&text).unwrap();

        assert_eq!(config.preset, "calm");
        assert!(!config.events.chat, "calm's own value, not balanced's");
        assert_eq!(config.events.min_distance, 80);
        assert_eq!(config.model.threads, 6, "what the player set is kept");
        assert!(
            rebased(&now, &now).is_none(),
            "the same preset: nothing to do"
        );
    }
}
