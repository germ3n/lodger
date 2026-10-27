//! The debug screen's half of the protocol: what the host's screen asks a mod about one world,
//! and what the mod answers. The host owns the screen (tabs, gauges, the event list with its
//! buttons, the key that opens it, the `/craftkit <id> ...` commands) and the senses' own
//! panel; a mod owns what is on its pages: its gauges, its lines, its events and their
//! availability, and the actions its buttons fire. A mod with nothing to show leaves
//! [`crate::Mod::debug`] at its default and the screen shows the host's panel alone.
//!
//! Everything here is plain `serde` data; see `docs/sdk.md`, The debug screen.

use serde::{Deserialize, Serialize};

/// What the screen asks, through `craftkit_world_debug`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DebugRequest {
    /// The whole panel, asked every half second while the screen is open.
    Snapshot,
    /// Do this event now, for this player, whatever the mod's own rules say. The mod answers
    /// with a message saying what it queued or why it could not.
    Force { event: String, player: u32 },
    /// Put the mod's progression at this stage.
    SetStage { stage: u8 },
    /// One of the mod's own buttons ([`DebugReply::actions`]).
    Action { id: String },
}

/// A mod's answer: for a snapshot, the panel; for anything else, a message and, if the mod
/// likes, the panel again.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DebugReply {
    /// What happened, for a request that did something; empty for a snapshot.
    pub message: String,
    /// Meters: stage, budget, temper, whatever the mod measures.
    pub gauges: Vec<Gauge>,
    /// Titled groups of lines: the memory as the mind reads it, the plan, the last decisions.
    pub sections: Vec<Section>,
    /// Every event the mod can do, with whether it could do it right now and why not.
    pub events: Vec<EventRow>,
    /// Buttons the screen shows, each firing a [`DebugRequest::Action`].
    pub actions: Vec<ActionRow>,
}

/// One meter. `value` over `max`, drawn as a bar, with `text` beside it (the value in words
/// where a number alone would not do: `stage 2 of 5`, `3 of 8 events today`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Gauge {
    pub label: String,
    pub value: f32,
    pub max: f32,
    pub text: String,
}

impl Gauge {
    pub fn new(label: &str, value: f32, max: f32, text: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            value,
            max,
            text: text.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Section {
    pub title: String,
    pub lines: Vec<String>,
    /// Rows with buttons of their own, after the lines: one per plan, say, each with Drop and
    /// Now. A button fires [`DebugRequest::Action`] with its id. ABI 7.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Item>,
}

/// A row of a [`Section`] with its own buttons.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Item {
    pub text: String,
    /// A second, quieter line: why it waits, when it is due.
    pub detail: String,
    pub actions: Vec<ActionRow>,
}

impl Section {
    pub fn new(title: &str, lines: Vec<String>) -> Self {
        Self {
            title: title.to_string(),
            lines,
            items: Vec::new(),
        }
    }
}

/// One of the mod's events as the screen lists it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EventRow {
    pub id: String,
    /// The event in words.
    pub words: String,
    /// Which stages offer it, as the mod words it: `2 to 5`.
    pub stages: String,
    /// Whether the mod would offer it right now.
    pub offered: bool,
    /// Why not, when not: the first need unmet, or the rule that refuses it.
    pub why: String,
    /// How many times it happened in this world.
    pub times: u32,
}

/// A button on the mod's page.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionRow {
    pub id: String,
    pub label: String,
}

impl ActionRow {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
        }
    }
}

impl DebugReply {
    /// A reply that only says something.
    pub fn message(text: impl Into<String>) -> Self {
        Self {
            message: text.into(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_section_s_items_carry_their_own_buttons_and_none_are_left_out() {
        let mut section = Section::new("Plans", vec![]);
        let plain = serde_json::to_string(&section).unwrap();
        assert!(!plain.contains("items"), "no items, no key: {plain}");
        section.items.push(Item {
            text: "#3 open_door".to_string(),
            detail: "in 20s".to_string(),
            actions: vec![ActionRow::new("plan_drop:3", "Drop")],
        });
        let text = serde_json::to_string(&section).unwrap();
        let back: Section = serde_json::from_str(&text).unwrap();
        assert_eq!(back, section);
        assert_eq!(back.items[0].actions[0].id, "plan_drop:3");
    }

    #[test]
    fn requests_are_tagged_by_kind() {
        let force: DebugRequest =
            serde_json::from_str(r#"{"kind":"force","event":"knock","player":0}"#).unwrap();
        assert_eq!(
            force,
            DebugRequest::Force {
                event: "knock".to_string(),
                player: 0
            }
        );

        let text = serde_json::to_string(&DebugRequest::Snapshot).unwrap();
        assert_eq!(text, r#"{"kind":"snapshot"}"#);
    }

    #[test]
    fn an_older_reply_without_new_fields_still_reads() {
        let reply: DebugReply = serde_json::from_str(r#"{"message":"ok"}"#).unwrap();
        assert_eq!(reply.message, "ok");
        assert!(reply.gauges.is_empty());

        let row: EventRow = serde_json::from_str(r#"{"id":"knock"}"#).unwrap();
        assert_eq!(row.id, "knock");
        assert!(!row.offered);
    }
}
