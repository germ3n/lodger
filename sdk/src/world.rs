//! What a mod sees and does, per world: the observations the host sends once a tick, the
//! commands the mod answers with, the reports on how they went, and the clock. Every shape is a
//! plain `serde` struct so the host's Gson and the mod's `serde_json` read the same bytes. New
//! fields take `#[serde(default)]`, so an older host and a newer mod, or the reverse, read each
//! other for as long as the ABI version allows. See `docs/sdk.md`.

use serde::{Deserialize, Serialize};

/// A block position.
pub type BlockPos = [i32; 3];
/// A position in the world.
pub type Pos = [f64; 3];

/// A world as the host opens it: which one, its seed, its name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldInfo {
    /// The dimension's id, `minecraft:overworld`.
    pub id: String,
    /// The world's seed, which a mod folds into its own randomness.
    pub seed: i64,
    /// The save's name, for logs.
    pub name: String,
}

/// The clock, so a mod never reads one itself.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Now {
    /// The server's tick count since the world opened.
    pub tick: u64,
    /// The world's time of day, 0 to 24000.
    pub day_time: u32,
    /// The in-game day, from the world's total time.
    pub day: u64,
    /// The real clock, milliseconds since the epoch.
    pub real_millis: u64,
}

/// What the host saw since the last tick. A quiet tick carries players' positions and nothing
/// else; the rest comes when it happens, on a schedule, or on request.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Observations {
    pub players: Vec<Player>,
    /// Entities within 64 blocks of any player, once a second.
    pub entities: Vec<Entity>,
    /// The world's weather and sky, once a second.
    pub world: Option<WorldState>,
    /// Things that happened: a chest opened, a block placed, sleep refused, a death.
    pub events: Vec<Event>,
    /// Chat lines players typed. Reading them is the mod's business; see `docs/good-to-know.md`.
    pub chat: Vec<ChatLine>,
    /// Answers to `Command::Scan`.
    pub scans: Vec<Scan>,
    /// The daily pass over the surroundings of a point the mod named.
    pub surroundings: Option<Surroundings>,
    /// The other mods loaded, once, on the first tick.
    pub mods: Vec<LoadedModInfo>,
}

/// A player, as the server knows them. `index` is stable for the session and is how events,
/// chat and commands name a player.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Player {
    pub index: u32,
    pub name: String,
    pub uuid: String,
    pub pos: Pos,
    pub yaw: f32,
    pub pitch: f32,
    pub velocity: Pos,
    pub sneaking: bool,
    pub sprinting: bool,
    pub swimming: bool,
    pub sleeping: bool,
    /// What they ride, by entity kind, or empty.
    pub riding: String,
    pub health: f32,
    pub hunger: u32,
    pub air: i32,
    pub level: u32,
    pub main_hand: ItemStack,
    pub off_hand: ItemStack,
    pub armor: Vec<ItemStack>,
    /// Effect ids, `minecraft:night_vision`.
    pub effects: Vec<String>,
    pub dimension: String,
    pub biome: String,
    /// The block light where they stand, 0 to 15, and the sky light.
    pub light: u8,
    pub sky_light: u8,
    /// What they look at within 8 blocks, if anything.
    pub looking_at: Option<Target>,
    /// How they have been moving, from the last seconds.
    pub movement: Movement,
}

/// How a player moves, read off their positions and facing over the last few seconds, so a mod
/// can read fear without being told (`docs/entity.md`, Senses).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Movement {
    /// Blocks a second over the last five seconds.
    pub pace: f32,
    /// Stops of a second or more in the last minute.
    pub stops: u32,
    /// Turns of more than 120 degrees in the last minute.
    pub turns: u32,
    /// A turn and back within two seconds, in the last minute.
    pub look_backs: u32,
    /// Seconds moving opposite to the facing in the last minute.
    pub backing_away: f32,
    /// Sprint starts in the last minute.
    pub sprint_bursts: u32,
    /// Seconds standing still right now.
    pub still_seconds: f32,
}

/// A block or an entity a player looks at.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    Block { pos: BlockPos, block: String },
    Entity { id: u32, entity: String },
}

/// An item and how many, and in a container's contents which slot it sits in.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemStack {
    /// `minecraft:iron_pickaxe`; empty for nothing.
    pub id: String,
    pub count: u32,
    /// The slot, where it matters (a container's contents); 0 otherwise.
    pub slot: u32,
}

/// An entity near a player.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Entity {
    /// The server's entity id, stable while it is loaded.
    pub id: u32,
    /// `minecraft:cow`.
    pub kind: String,
    pub pos: Pos,
    /// Its custom name, or empty.
    pub name: String,
    /// The uuid of its owner if tamed, or empty.
    pub owner: String,
    pub hostile: bool,
    /// The index of the player it faces within 10 degrees, if any.
    pub looking_at_player: Option<u32>,
}

/// The weather and the sky.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldState {
    pub raining: bool,
    pub thundering: bool,
    /// 0 to 7, 0 the full moon.
    pub moon_phase: u8,
}

/// Something a player did or had happen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Event {
    pub player: u32,
    pub kind: EventKind,
    pub pos: Option<BlockPos>,
    /// What it concerned: a block id, a dimension, an entity kind.
    pub detail: String,
    /// A container's contents, for `OpenedContainer`.
    pub contents: Vec<ItemStack>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    #[default]
    Joined,
    Left,
    OpenedContainer,
    ClosedContainer,
    PlacedBlock,
    BrokeBlock,
    SleptBegan,
    SleptEnded,
    SleepRefused,
    Ate,
    Fished,
    Mounted,
    Dismounted,
    Died,
    Respawned,
    ChangedDimension,
    /// Hit an entity; `detail` is its kind, `pos` where it stood.
    Hit,
}

/// A chat line a player typed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatLine {
    pub player: u32,
    pub text: String,
}

/// The answer to a `Command::Scan`: the blocks of the kinds asked for in a box.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scan {
    /// The command's id.
    pub id: u64,
    /// Counts by block id.
    pub counts: Vec<(String, u32)>,
    /// Positions of the kinds asked for by position.
    pub positions: Vec<(String, BlockPos)>,
}

/// The answer to `Command::Surroundings`: points of interest around a point, as features.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Surroundings {
    /// The command's id.
    pub id: u64,
    pub around: BlockPos,
    pub features: Vec<Feature>,
}

/// One thing found around: `water` with its extent, `village` with its center, `field`,
/// `pen` with its animals in `detail`, `tunnel` with its length, `cave`, `snow`, `sand`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Feature {
    pub kind: String,
    pub pos: BlockPos,
    /// Blocks across, or long.
    pub size: u32,
    pub detail: String,
}

/// Another mod loaded beside this one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadedModInfo {
    pub id: String,
    pub version: String,
}

/// What a mod wants done. Each carries an `id` the report names. The host checks each command's
/// preconditions as it runs it and reports `done` or why not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Command {
    /// A line in the host's log, prefixed with the mod's id.
    Log {
        id: u64,
        text: String,
    },
    /// Count and locate blocks of `kinds` in the box, answered as a `Scan` next tick.
    Scan {
        id: u64,
        min: BlockPos,
        max: BlockPos,
        kinds: Vec<String>,
    },
    /// A sound to one player at a position, by id, once.
    PlaySound {
        id: u64,
        player: u32,
        sound: String,
        pos: Pos,
        volume: f32,
        pitch: f32,
    },
    /// A client-side hint to one player: a kind the host's client knows, a place, a duration.
    Hint {
        id: u64,
        player: u32,
        hint: String,
        pos: Option<BlockPos>,
        seconds: f32,
        detail: String,
    },
    /// A system message to one player, as the server would send it.
    SystemMessage {
        id: u64,
        player: u32,
        text: String,
    },
    /// Open or close a door, trapdoor or gate, flip a lever, press a button.
    ToggleBlock {
        id: u64,
        pos: BlockPos,
    },
    /// Move one block to another place within reach, as the lodger moves a torch.
    MoveBlock {
        id: u64,
        from: BlockPos,
        to: BlockPos,
    },
    /// Set a block, with an optional state string the host parses.
    SetBlock {
        id: u64,
        pos: BlockPos,
        block: String,
        state: String,
    },
    /// Move items between two containers, or within one.
    MoveItem {
        id: u64,
        from: BlockPos,
        from_slot: u32,
        to: BlockPos,
        to_slot: u32,
        count: u32,
    },
    /// Take `count` of a container's slot away. The mod keeps what it took in its own memory;
    /// nothing in the world holds it until a `DropItem` or `MoveItem` puts it somewhere.
    TakeItem {
        id: u64,
        from: BlockPos,
        slot: u32,
        count: u32,
    },
    /// Put an item into a container: the slot given, or the first empty one when `slot` is
    /// absent. The mod gives back what it took this way.
    PutItem {
        id: u64,
        into: BlockPos,
        slot: Option<u32>,
        item: ItemStack,
    },
    /// Drop an item at a position, as an item entity that does not despawn.
    DropItem {
        id: u64,
        pos: Pos,
        item: ItemStack,
    },
    /// Write a sign's lines.
    SetSign {
        id: u64,
        pos: BlockPos,
        lines: Vec<String>,
    },
    /// Cancel one player's sleep with the vanilla message, or refuse their next attempt within
    /// `seconds` with it.
    CancelSleep {
        id: u64,
        player: u32,
        seconds: f32,
    },
    /// Turn an entity's head toward a position and keep it there for `seconds`.
    LookAt {
        id: u64,
        entity: u32,
        at: Pos,
        seconds: f32,
    },
    /// Look around `around` within `radius` blocks for the features of `Surroundings`, answered
    /// as `Observations.surroundings` over the next seconds, in slices, so it never shows in a
    /// tick profile.
    Surroundings {
        id: u64,
        around: BlockPos,
        radius: u32,
    },
    /// Spawn or move a figure wearing a player's skin, standing or walking a path.
    Figure {
        id: u64,
        figure: u32,
        skin_of: u32,
        pos: Pos,
        path: Vec<Pos>,
        held: ItemStack,
        name_shown: bool,
    },
    RemoveFigure {
        id: u64,
        figure: u32,
    },
}

impl Command {
    pub fn id(&self) -> u64 {
        match self {
            Command::Log { id, .. }
            | Command::Scan { id, .. }
            | Command::PlaySound { id, .. }
            | Command::Hint { id, .. }
            | Command::SystemMessage { id, .. }
            | Command::ToggleBlock { id, .. }
            | Command::MoveBlock { id, .. }
            | Command::SetBlock { id, .. }
            | Command::MoveItem { id, .. }
            | Command::TakeItem { id, .. }
            | Command::PutItem { id, .. }
            | Command::DropItem { id, .. }
            | Command::SetSign { id, .. }
            | Command::CancelSleep { id, .. }
            | Command::LookAt { id, .. }
            | Command::Surroundings { id, .. }
            | Command::Figure { id, .. }
            | Command::RemoveFigure { id, .. } => *id,
        }
    }
}

/// How a command went.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    pub id: u64,
    pub done: bool,
    /// Why not, when not.
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quiet_tick_is_a_small_message() {
        let quiet = Observations {
            players: vec![Player {
                index: 0,
                name: "razor".to_string(),
                pos: [1.0, 64.0, 2.0],
                ..Player::default()
            }],
            ..Observations::default()
        };
        let text = serde_json::to_string(&quiet).unwrap();

        assert!(text.len() < 1200, "{} bytes: {text}", text.len());
        let back: Observations = serde_json::from_str(&text).unwrap();
        assert_eq!(back, quiet);
    }

    #[test]
    fn an_older_message_without_new_fields_still_reads() {
        let text = r#"{ "players": [ { "index": 3, "name": "a" } ] }"#;
        let obs: Observations = serde_json::from_str(text).unwrap();

        assert_eq!(obs.players[0].index, 3);
        assert_eq!(obs.players[0].movement, Movement::default());
    }

    #[test]
    fn commands_are_tagged_by_kind_and_carry_their_id() {
        let command = Command::ToggleBlock {
            id: 9,
            pos: [1, 2, 3],
        };
        let text = serde_json::to_string(&command).unwrap();

        assert!(text.contains(r#""kind":"toggle_block""#), "{text}");
        assert_eq!(command.id(), 9);
        let back: Command = serde_json::from_str(&text).unwrap();
        assert_eq!(back, command);
    }

    #[test]
    fn a_target_is_a_block_or_an_entity() {
        let block: Target =
            serde_json::from_str(r#"{"kind":"block","pos":[0,1,2],"block":"minecraft:chest"}"#)
                .unwrap();
        assert_eq!(
            block,
            Target::Block {
                pos: [0, 1, 2],
                block: "minecraft:chest".to_string()
            }
        );
    }
}
