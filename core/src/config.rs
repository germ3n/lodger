//! `config/lodger.json`, as the mind reads it.
//!
//! The body reads the file and hands the mind its text; the mind owns the schema, fills the
//! defaults, and hands back the effective config, so the keys live in one place (here and in
//! `docs/config.md`, which a test checks agree). An unknown key is reported, not refused: the
//! body logs it once. The hard rules in `docs/good-to-know.md` are not config and have no key.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The Hugging Face URL of Qwen's own `q4_k_m` GGUF, 491 MB, the file `~/engine` runs.
pub const DEFAULT_MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub log: Log,
    pub model: Model,
    pub brain: Brain,
    pub stages: Stages,
    pub events: Events,
    pub multiplayer: Multiplayer,
    /// Keys the schema does not know, kept so the body can log them. Never written back.
    #[serde(flatten, skip_serializing)]
    pub unknown: BTreeMap<String, serde_json::Value>,
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
    pub url: String,
    pub path: String,
    pub threads: u32,
    pub context: u32,
    pub idle_minutes: u32,
    pub gpu_layers: u32,
}

// Fractions are f64, not f32: a 0.4 written back to the player's file as 0.4000000059604645 is
// what an f32 looks like once serde_json prints it, and nothing here needs the smaller type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Brain {
    pub plan_every: u32,
    pub event_threshold: f64,
    pub react_threshold: f64,
    pub queue: u32,
    pub force_rules: bool,
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
    pub min_distance: u32,
    pub disabled: Vec<String>,
    pub offered: u32,
    pub cooldown_scale: f64,
    pub sounds: bool,
    pub figure: bool,
    pub glitches: bool,
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
            log: Log::default(),
            model: Model::default(),
            brain: Brain::default(),
            stages: Stages::default(),
            events: Events::default(),
            multiplayer: Multiplayer::default(),
            unknown: BTreeMap::new(),
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
            url: DEFAULT_MODEL_URL.to_string(),
            path: "config/lodger/models/qwen.gguf".to_string(),
            threads: 2,
            context: 512,
            idle_minutes: 10,
            gpu_layers: 0,
        }
    }
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            plan_every: 90,
            event_threshold: 0.4,
            react_threshold: 0.55,
            queue: 64,
            force_rules: false,
        }
    }
}

impl Default for Stages {
    fn default() -> Self {
        Self {
            least_days: [1, 2, 2, 3, 1],
            most_days: [3, 5, 5, 7, 3],
            events_per_day: [1.0, 2.0, 4.0, 6.0, 0.0],
            start_after_days: 3,
            ending: "difficulty".to_string(),
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Self {
            min_distance: 24,
            disabled: Vec::new(),
            offered: 8,
            cooldown_scale: 1.0,
            sounds: true,
            figure: true,
            glitches: true,
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

impl Config {
    /// Reads the file's text. An empty text is the defaults, as a missing file is.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.trim().is_empty() {
            return Ok(Self::default());
        }

        let config: Self = serde_json::from_str(text).map_err(|err| format!("config: {err}"))?;
        config.check()?;

        Ok(config)
    }

    /// The bounds a value must keep, with the reason beside each.
    fn check(&self) -> Result<(), String> {
        // A context too small cannot hold the frame and one question; llama.cpp wants a
        // multiple of 32 anyway.
        if self.model.context < 256 {
            return Err("model.context must be at least 256".to_string());
        }

        if self.model.threads == 0 {
            return Err("model.threads must be at least 1".to_string());
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

        // Zero or one offered leaves the model nothing to choose; the prompt grows past twelve.
        if !(2..=12).contains(&self.events.offered) {
            return Err("events.offered must be between 2 and 12".to_string());
        }

        if self.events.cooldown_scale <= 0.0 {
            return Err("events.cooldown_scale must be positive".to_string());
        }

        Ok(())
    }

    /// The keys the file had that the schema does not know, as `a.b` paths, one level deep: an
    /// unknown key inside a known section is found by that section's `unknown` map in a later
    /// phase; today the sections ignore theirs silently, which the test below pins so it is not
    /// forgotten.
    pub fn unknown_keys(&self) -> Vec<String> {
        self.unknown.keys().cloned().collect()
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
        assert_eq!(config.model.context, 512);
    }

    #[test]
    fn an_unknown_top_level_key_is_reported_and_not_written_back() {
        let config = Config::parse(r#"{ "enabled": true, "colour": "red" }"#).unwrap();

        assert_eq!(config.unknown_keys(), vec!["colour".to_string()]);
        assert!(!config.to_json().contains("colour"));
    }

    #[test]
    fn an_unknown_key_inside_a_section_is_ignored_today() {
        // Pinned so the behaviour is known: reporting these is a later phase's item.
        let config = Config::parse(r#"{ "brain": { "colour": "red" } }"#).unwrap();

        assert!(config.unknown_keys().is_empty());
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
            (
                r#"{ "brain": { "event_threshold": 1.5 } }"#,
                "brain.event_threshold",
            ),
            (r#"{ "brain": { "queue": 0 } }"#, "brain.queue"),
            (
                r#"{ "stages": { "least_days": [9, 2, 2, 3, 1] } }"#,
                "stages.least_days[0]",
            ),
            (r#"{ "stages": { "ending": "win" } }"#, "stages.ending"),
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
            .filter(|key| key.contains('.') || *key == "enabled")
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
}
