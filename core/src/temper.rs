//! Its temper (`docs/entity.md`, Provocation and temper): 0 to 1, starting at the preset's
//! resting value, moved by how the target treats it, read by everything that paces the stay.
//! The rules hold at every temper: an angry lodger does what it does more often and sooner,
//! never anything it would not do.

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::memory::{distance, Memory};
use craftkit::world::{BlockPos, Event, EventKind};

/// Where temper rests, by preset: calm 0.2, balanced 0.3, intense 0.4, relentless 0.5.
pub fn resting(config: &Config) -> f32 {
    match config.preset.as_str() {
        "calm" => 0.2,
        "intense" => 0.4,
        "relentless" => 0.5,
        _ => 0.3,
    }
}

/// The share of the way back to resting it drifts each day with nothing done.
const DRIFT_PER_DAY: f32 = 0.1;
/// How many times one cause moves it in one day: a mine of torches is one grievance, not sixty.
const MOST_A_DAY: u32 = 3;
/// The causes kept, for the situation's words.
const MOST_CAUSES: usize = 8;

/// One thing that moved it, in the words the situation uses.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cause {
    pub what: String,
    pub by: f32,
    pub day: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Temper {
    pub value: f32,
    /// The preset's resting value, as last settled.
    pub resting: f32,
    /// Whether `value` has been set from the preset yet.
    pub begun: bool,
    /// The last day it drifted.
    pub drifted_day: f32,
    pub causes: Vec<Cause>,
    /// Today's moves by cause, for the cap: apart from `causes`, which keeps eight for words
    /// and evicted today's, so the cap reset.
    pub today: Vec<(String, u32)>,
    pub today_day: i64,
}

impl Default for Temper {
    fn default() -> Self {
        Self {
            value: 0.3,
            resting: 0.3,
            begun: false,
            drifted_day: 0.0,
            causes: Vec::new(),
            today: Vec::new(),
            today_day: -1,
        }
    }
}

/// The moves, by cause: what the target did, and how far it moves.
pub const CHASED_FIGURE: (&str, f32) = ("chased the figure", 0.15);
pub const LIT_ITS_PLACE: (&str, f32) = ("lit its place", 0.12);
pub const HARMED_ITS_PLACE: (&str, f32) = ("put TNT, lava or water by its place", 0.15);
pub const BROKE_ITS_LIGHT: (&str, f32) = ("broke its lights", 0.08);
pub const BROKE_ITS_THINGS: (&str, f32) = ("broke what it made", 0.08);
pub const BROKE_ITS_HOUSE: (&str, f32) = ("broke its house", 0.1);
pub const OPENED_ITS_CHEST: (&str, f32) = ("opened its chest", 0.05);
pub const CURSED_IT: (&str, f32) = ("cursed it", 0.1);
pub const BUILT_AT_ITS_PLACE: (&str, f32) = ("built at its place", -0.03);
pub const LEFT_A_CHEST: (&str, f32) = ("left a chest at its place", -0.1);
pub const WATCHED_FROM_AFAR: (&str, f32) = ("watched the figure from afar", -0.05);
pub const SPOKE_KINDLY: (&str, f32) = ("spoke to it kindly", -0.03);

const CURSES: &[&str] = &[
    "go away",
    "leave",
    "get out",
    "hate",
    "stupid",
    "idiot",
    "die",
    "kill you",
    "shut up",
    "fuck",
    "shit",
    "damn",
    "screw you",
];
const KIND: &[&str] = &[
    "hello",
    "hi ",
    "what do you want",
    "thank",
    "friend",
    "sorry",
    "please",
];

impl Temper {
    /// Moved by a cause, at most three times a cause a day, within 0 to 1.
    pub fn moved(&mut self, (what, by): (&str, f32), day: f32) {
        if self.today_day != day.floor() as i64 {
            self.today_day = day.floor() as i64;
            self.today.clear();
        }
        let count = match self.today.iter_mut().find(|(w, _)| w == what) {
            Some((_, n)) => n,
            None => {
                self.today.push((what.to_string(), 0));
                &mut self.today.last_mut().unwrap().1
            }
        };
        if *count >= MOST_A_DAY {
            return;
        }
        *count += 1;
        self.value = (self.value + by).clamp(0.0, 1.0);
        if self.causes.len() >= MOST_CAUSES {
            self.causes.remove(0);
        }
        self.causes.push(Cause {
            what: what.to_string(),
            by,
            day,
        });
    }

    /// Set to the preset's resting value once, and drifted back toward it a tenth of the way
    /// for each day gone by.
    pub fn settle(&mut self, resting: f32, day: f32) {
        self.resting = resting;
        if !self.begun {
            self.value = resting;
            self.begun = true;
            self.drifted_day = day;
            return;
        }
        let days = (day - self.drifted_day).floor();
        if days >= 1.0 {
            for _ in 0..(days as u32).min(30) {
                self.value += (resting - self.value) * DRIFT_PER_DAY;
            }
            // Back at rest once near enough: drifting a tenth at a time it never quite got there,
            // and on `calm`, resting at 0.2 exactly, a hair under held its stages slow for good.
            if (self.value - resting).abs() < 0.01 {
                self.value = resting;
            }
            self.drifted_day += days;
        }
    }

    /// The day's events scaled: half again when provoked, half when appeased, one at rest.
    pub fn budget_factor(&self, resting: f32) -> f32 {
        ((0.5 + self.value) / (0.5 + resting)).clamp(0.5, 1.5)
    }

    /// A stage's least days halved above 0.7.
    pub fn least_factor(&self) -> f32 {
        if self.value > 0.7 {
            0.5
        } else {
            1.0
        }
    }

    /// A stage's most days stretched by half below 0.2.
    pub fn most_factor(&self) -> f32 {
        if self.value < 0.2 {
            1.5
        } else {
            1.0
        }
    }

    /// The bolder events a stage early above 0.6, a stage late below 0.2.
    pub fn stage_shift(&self) -> i8 {
        if self.value > 0.6 {
            -1
        } else if self.value < 0.2 {
            1
        } else {
            0
        }
    }

    /// The least distance it keeps, three quarters above 0.8.
    pub fn distance_factor(&self) -> f64 {
        if self.value > 0.8 {
            0.75
        } else {
            1.0
        }
    }

    /// How much fiercer the fight is: 0 at rest or below, 1 at the top.
    pub fn fierce(&self, resting: f32) -> f32 {
        ((self.value - resting) / (1.0 - resting).max(0.01)).clamp(0.0, 1.0)
    }

    /// The situation's line, when it is far from resting: what made it so, the last two.
    pub fn words(&self, resting: f32) -> Option<String> {
        let (mood, up) = if self.value >= resting + 0.2 {
            ("It is angry", true)
        } else if self.value <= resting - 0.15 {
            ("It is pleased", false)
        } else {
            return None;
        };
        let mut why: Vec<&str> = Vec::new();
        for cause in self.causes.iter().rev() {
            if (cause.by > 0.0) == up && !why.contains(&cause.what.as_str()) {
                why.push(&cause.what);
            }
            if why.len() == 2 {
                break;
            }
        }
        Some(if why.is_empty() {
            format!("{mood}.")
        } else {
            format!("{mood}: the player {}.", why.join(" and "))
        })
    }
}

/// The bolder events, which temper brings forward or holds back a stage.
pub const BOLD: &[&str] = &[
    "behind_you",
    "on_the_roof",
    "in_doorway",
    "in_your_bed",
    "at_the_chest",
    "refuse_rest",
    "bed_occupied",
    "wake",
    "woke_elsewhere",
    "knock",
    "lights_fail",
    "tunnel_toward",
];

/// Whether an event is offered at this stage, temper's shift for the bold ones applied.
pub fn offered_in(event: &crate::events::Event, stage: u8, temper: &Temper) -> bool {
    if event.id == "wait" || !BOLD.contains(&event.id) {
        return event.offered_in(stage);
    }
    let from = (event.from_stage as i8 + temper.stage_shift()).max(1) as u8;

    from <= stage && stage <= event.to_stage
}

/// The least distance it keeps from the target, temper applied.
pub fn min_distance(config: &Config, memory: &Memory) -> f64 {
    config.events.min_distance as f64 * memory.temper.distance_factor()
}

/// What its builds are made of (`building.rs`).
const IT_BUILDS_WITH: &[&str] = &[
    "minecraft:cobblestone",
    "minecraft:oak_planks",
    "minecraft:oak_log",
    "minecraft:glass_pane",
    "minecraft:oak_door",
    "minecraft:chest",
    "minecraft:wall_torch",
    "minecraft:torch",
    "minecraft:oak_fence",
];

/// The events that leave a thing of its own where they happen.
const IT_MAKES: &[&str] = &[
    "sign",
    "sign_deep",
    "sign_word",
    "cairn",
    "pillar",
    "grave",
    "grave_each",
    "scarecrow",
    "head_on_post",
    "snowman",
    "kill_sign",
];

fn is_light(id: &str) -> bool {
    id.ends_with("torch")
        || id.ends_with("lantern")
        || id.contains("campfire")
        || id.ends_with("lamp")
}

fn is_harm(id: &str) -> bool {
    // A campfire is a light, not a fire set.
    id.ends_with("tnt")
        || id.contains("lava")
        || id.ends_with("water")
        || (id.ends_with("fire") && !id.contains("campfire"))
}

impl Memory {
    /// What one observation's event does to its temper, the target's alone.
    pub fn temper_event(&mut self, event: &Event, day: f32) {
        if Some(event.player) != self.target {
            return;
        }
        let near = |a: Option<BlockPos>, b: Option<BlockPos>, within: f64| match (a, b) {
            (Some(a), Some(b)) => distance(a, b) <= within,
            _ => false,
        };
        match event.kind {
            EventKind::PlacedBlock if near(event.pos, self.place, 5.0) => {
                let id = event.detail.as_str();
                let cause = if is_harm(id) {
                    HARMED_ITS_PLACE
                } else if is_light(id) {
                    LIT_ITS_PLACE
                } else if id.ends_with("chest") || id.ends_with("barrel") {
                    LEFT_A_CHEST
                } else {
                    BUILT_AT_ITS_PLACE
                };
                self.temper.moved(cause, day);
            }
            EventKind::BrokeBlock => {
                let Some(pos) = event.pos else { return };
                // Only what it made: a block of their own, grass by its house, or their own door
                // where something once happened is not its.
                let theirs = self.placed.contains(&pos);
                if self.its_lights.contains(&pos) {
                    self.temper.moved(BROKE_ITS_LIGHT, day);
                } else if !theirs
                    && IT_BUILDS_WITH.contains(&event.detail.as_str())
                    && self
                        .builds
                        .iter()
                        .any(|b| b.laid > 0 && b.ground_known() && distance(b.site, pos) <= 4.0)
                {
                    self.temper.moved(BROKE_ITS_HOUSE, day);
                } else if !theirs
                    && self
                        .landmarks
                        .iter()
                        .any(|l| l.ok && l.pos == Some(pos) && IT_MAKES.contains(&l.event.as_str()))
                {
                    self.temper.moved(BROKE_ITS_THINGS, day);
                }
            }
            EventKind::OpenedContainer => {
                // Theirs by placing it; its own chest is remembered too once opened, and was
                // taken for theirs from the second opening on.
                let theirs = self
                    .containers
                    .iter()
                    .any(|c| Some(c.pos) == event.pos && c.theirs());
                let its = self
                    .builds
                    .iter()
                    .any(|b| b.laid > 0 && near(event.pos, Some(b.site), 4.0));
                if its && !theirs {
                    self.temper.moved(OPENED_ITS_CHEST, day);
                }
            }
            EventKind::FigureSeen => {
                let look = crate::memory::FigureLook::parse(&event.detail, day, event.pos);
                if look.chased() {
                    self.temper.moved(CHASED_FIGURE, day);
                } else if look.looked_long() {
                    self.temper.moved(WATCHED_FROM_AFAR, day);
                }
            }
            _ => {}
        }
    }

    /// A chat line about it: cursed, or spoken to kindly.
    pub fn temper_chat(&mut self, text: &str, day: f32) {
        // Whole words only: "whatever" holds no "hate", "studied" no "die".
        let words: String = text
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();
        let padded = format!(
            " {} ",
            words.split_whitespace().collect::<Vec<_>>().join(" ")
        );
        let has = |phrase: &str| padded.contains(&format!(" {} ", phrase.trim()));
        if CURSES.iter().any(|c| has(c)) {
            self.temper.moved(CURSED_IT, day);
        } else if KIND.iter().any(|k| has(k)) {
            self.temper.moved(SPOKE_KINDLY, day);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_rests_at_the_preset_s_value_and_drifts_back_a_tenth_a_day() {
        let mut temper = Temper::default();
        temper.settle(0.3, 0.0);
        assert_eq!(temper.value, 0.3);

        temper.moved(CHASED_FIGURE, 0.5);
        temper.moved(CHASED_FIGURE, 0.6);
        assert!((temper.value - 0.6).abs() < 1e-6);
        temper.settle(0.3, 5.2);
        assert!(temper.value < 0.6 && temper.value > 0.3, "{}", temper.value);
        let after_five = temper.value;
        temper.settle(0.3, 30.0);
        assert!((temper.value - 0.3).abs() < (after_five - 0.3).abs() / 2.0);
    }

    #[test]
    fn one_cause_moves_it_three_times_a_day_at_most() {
        let mut temper = Temper::default();
        temper.settle(0.3, 0.0);
        for _ in 0..60 {
            temper.moved(BROKE_ITS_LIGHT, 1.5);
        }
        assert!((temper.value - (0.3 + 3.0 * 0.08)).abs() < 1e-6);
        temper.moved(BROKE_ITS_LIGHT, 2.1);
        assert!(
            (temper.value - (0.3 + 4.0 * 0.08)).abs() < 1e-6,
            "a new day"
        );
    }

    #[test]
    fn provoked_it_paces_faster_and_brings_the_bold_events_forward_appeased_the_other_way() {
        let mut temper = Temper {
            value: 0.9,
            begun: true,
            ..Temper::default()
        };
        assert!(temper.budget_factor(0.3) > 1.4);
        assert_eq!(temper.least_factor(), 0.5);
        assert_eq!(temper.stage_shift(), -1);
        assert_eq!(temper.distance_factor(), 0.75);
        let knock = crate::events::find("knock").unwrap();
        assert!(offered_in(knock, knock.from_stage - 1, &temper));

        temper.value = 0.1;
        assert!(temper.budget_factor(0.3) < 0.8);
        assert_eq!(temper.most_factor(), 1.5);
        assert!(!offered_in(knock, knock.from_stage, &temper));
        assert_eq!(temper.fierce(0.3), 0.0);

        temper.value = 0.3;
        assert!(
            (temper.budget_factor(0.3) - 1.0).abs() < 1e-6,
            "at rest, as the preset says"
        );
    }

    #[test]
    fn a_calm_stay_at_its_angriest_is_still_quieter_than_a_balanced_one_at_rest() {
        let calm = crate::config::Preset::Calm.config();
        let balanced = Config::default();
        let furious = Temper {
            value: 1.0,
            begun: true,
            ..Temper::default()
        };
        for stage in 1..=4u8 {
            let mut provoked = crate::arc::Arc {
                stage,
                ..crate::arc::Arc::default()
            };
            provoked.pace.budget = furious.budget_factor(resting(&calm));
            let at_rest = crate::arc::Arc {
                stage,
                ..crate::arc::Arc::default()
            };
            assert!(
                provoked.budget(&calm) < at_rest.budget(&balanced),
                "stage {stage}: {} against {}",
                provoked.budget(&calm),
                at_rest.budget(&balanced)
            );
        }
    }

    #[test]
    fn its_words_say_why_when_it_is_far_from_resting() {
        let mut temper = Temper::default();
        temper.settle(0.3, 0.0);
        assert!(temper.words(0.3).is_none());
        temper.moved(CHASED_FIGURE, 1.0);
        temper.moved(BROKE_ITS_LIGHT, 1.0);
        assert_eq!(
            temper.words(0.3).unwrap(),
            "It is angry: the player broke its lights and chased the figure."
        );
    }

    #[test]
    fn words_are_matched_whole_a_campfire_is_a_light_and_the_cap_outlasts_the_words_list() {
        let mut memory = Memory::default();
        memory.temper.settle(0.3, 0.0);
        memory.temper_chat("whatever you are, thank you", 1.0);
        assert!(memory.temper.value < 0.3, "no hate in whatever: kind words");

        assert!(!is_harm("minecraft:campfire") && is_light("minecraft:soul_campfire"));
        assert!(is_harm("minecraft:fire"));

        let mut temper = Temper::default();
        temper.settle(0.3, 0.0);
        for cause in [LIT_ITS_PLACE, HARMED_ITS_PLACE, BROKE_ITS_LIGHT] {
            for _ in 0..3 {
                temper.moved(cause, 1.5);
            }
        }
        let before = temper.value;
        temper.moved(LIT_ITS_PLACE, 1.6);
        assert_eq!(
            temper.value, before,
            "the fourth light that day moves nothing"
        );
    }

    #[test]
    fn a_calm_stay_drifts_all_the_way_back_to_rest() {
        let mut temper = Temper::default();
        temper.settle(0.2, 0.0);
        temper.moved(WATCHED_FROM_AFAR, 0.5);
        temper.settle(0.2, 60.0);
        assert_eq!(temper.value, 0.2);
        assert_eq!(
            temper.most_factor(),
            1.0,
            "at rest, calm's stages as written"
        );
    }

    #[test]
    fn chat_that_curses_it_angers_it_and_kind_words_calm_it() {
        let mut memory = Memory::default();
        memory.temper.settle(0.3, 0.0);
        memory.temper_chat("just go away already", 1.0);
        assert!(memory.temper.value > 0.3);
        let angry = memory.temper.value;
        memory.temper_chat("hello, what do you want", 1.0);
        assert!(memory.temper.value < angry);
    }
}
