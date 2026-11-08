//! What the lodger knows about its target, bounded, saved with the stay, and described as the
//! lines the brain reads. See `docs/memory.md`; this is the first cut, the pieces the first
//! events need. Positions are block positions; times are the stay's in-game day as a fraction.

use craftkit::world::{BlockPos, EventKind, Feature, ItemStack, Observations, Player};
use serde::{Deserialize, Serialize};

/// How far from the home's center the home reaches.
pub const HOME_RADIUS: i32 = 16;
/// Within this many blocks of the target, nothing happens while in view.
pub const CELL: i32 = 4;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Home {
    /// Where they sleep, if they have.
    pub bed: Option<BlockPos>,
    /// The middle of where they spend their time standing still.
    pub center: BlockPos,
    /// The floor's height, from the bed or the lowest block they placed.
    pub floor_y: i32,
    pub doors: Vec<BlockPos>,
    pub lights: Vec<BlockPos>,
}

/// A house the target is in that is not their home, as the mind's scan found it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Here {
    pub doors: Vec<BlockPos>,
    pub containers: Vec<BlockPos>,
    pub panes: Vec<BlockPos>,
    pub day: f32,
}

/// What the home read looks for: doors, lights, containers, beds, windows, stairs, carpets,
/// ladders, pots and the work blocks, whoever placed them. A `*` is a suffix.
pub const HOME_KINDS: &[&str] = &[
    "*_door",
    "*torch",
    "*lantern",
    "*lamp",
    // Every mod's chests and barrels, not the game's alone (owner: "use their
    // storage"); an ender chest is never theirs to the memory (`read_home`).
    "*chest",
    "*barrel",
    "minecraft:furnace",
    "minecraft:smoker",
    "*_bed",
    "*glass_pane",
    "minecraft:glass",
    "*_stairs",
    "*_carpet",
    "minecraft:ladder",
    "minecraft:flower_pot",
    "minecraft:crafting_table",
    "minecraft:lectern",
    "minecraft:jukebox",
    "minecraft:dandelion",
    "minecraft:poppy",
    "minecraft:blue_orchid",
    "minecraft:allium",
    "minecraft:azure_bluet",
    "*_tulip",
    "minecraft:oxeye_daisy",
    "minecraft:cornflower",
    "minecraft:lily_of_the_valley",
    "minecraft:pumpkin",
    "*_sapling",
    "minecraft:beehive",
    "minecraft:bee_nest",
    "minecraft:composter",
    "minecraft:bell",
    "minecraft:redstone_lamp",
    "minecraft:brewing_stand",
    "minecraft:grindstone",
];

/// Whether a block id is a flower the home read keeps (`flowers_potted`).
pub fn is_flower(id: &str) -> bool {
    let name = id.rsplit(':').next().unwrap_or(id);
    name.ends_with("_tulip")
        || matches!(
            name,
            "dandelion"
                | "poppy"
                | "blue_orchid"
                | "allium"
                | "azure_bluet"
                | "oxeye_daisy"
                | "cornflower"
                | "lily_of_the_valley"
        )
}

/// The fixture kinds the home read covers: inside its box, what it does not find is gone.
const READ_FIXTURES: &[&str] = &[
    "beehive",
    "bee_nest",
    "glass_pane",
    "glass",
    "stairs",
    "carpet",
    "flower_pot",
    "ladder",
    "crafting_table",
    "lectern",
    "jukebox",
    "composter",
    "bell",
    "redstone_lamp",
    "brewing_stand",
    "grindstone",
];

/// A door is two blocks; its lower half stands for it.
fn lower_halves(doors: &[BlockPos]) -> Vec<BlockPos> {
    doors
        .iter()
        .filter(|d| !doors.contains(&[d[0], d[1] - 1, d[2]]))
        .copied()
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Container {
    pub pos: BlockPos,
    /// The block id: `minecraft:chest`, `minecraft:furnace`, `minecraft:barrel`.
    pub kind: String,
    pub opens: u32,
    pub last_opened_day: f32,
    /// As the player left it: read when it is opened and again when it is closed.
    pub contents: Vec<ItemStack>,
    /// The target placed it.
    pub placed: bool,
    /// On how many different days the target opened it.
    pub days: u32,
    /// It stands in the house they sleep in (the surroundings pass's house read): the home's,
    /// whoever placed it (owner).
    pub in_home: bool,
}

impl Container {
    /// Opened on this many different days, a container the target did not place is theirs too:
    /// a village house moved into. One opened once on the way past is not.
    pub const ADOPTED_DAYS: u32 = 3;

    /// Whether it is the target's: placed by them, or used on [`Self::ADOPTED_DAYS`] days. Only
    /// these are touched by the lodger and described to the model.
    pub fn theirs(&self) -> bool {
        self.placed || self.in_home || self.days >= Self::ADOPTED_DAYS
    }

    pub fn is_furnace(&self) -> bool {
        self.kind.ends_with("furnace") || self.kind.ends_with("smoker")
    }

    /// The slots that hold nothing, for a chest-sized container (27) or a furnace (3).
    pub fn empty_slots(&self) -> Vec<u32> {
        let size = if self.is_furnace() { 3 } else { 27 };
        let used: Vec<u32> = self.contents.iter().map(|s| s.slot).collect();

        (0..size).filter(|slot| !used.contains(slot)).collect()
    }
}

/// A block the target placed that an event can use: a fence gate, a window pane, a lever, a
/// campfire, a cauldron, a crafting table, a note block, a bell, a jukebox, a lectern, an anvil,
/// a carpet, a sign. `kind` is the plain word, not the block id, so `oak_fence_gate` and
/// `spruce_fence_gate` are both `fence_gate`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fixture {
    pub kind: String,
    pub pos: BlockPos,
}

/// A hanging or standing thing near the home: an item frame, a painting, an armour stand. Seen
/// in the entity pass; the id is the server's and lasts while the world is loaded.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Decor {
    pub kind: String,
    pub id: u32,
    pub pos: BlockPos,
}

/// What the target has done to one kind of mob: how often they hit it and how many they killed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tally {
    /// `cow`, `zombie`: the kind without its namespace.
    pub kind: String,
    pub hits: u32,
    pub kills: u32,
}

/// Something that happened to the target that the lodger never forgets: where they died, the
/// first night they slept, the first of a kind they killed, a dimension crossed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Milestone {
    /// `died`, `first_sleep`, `first_kill`, `crossed`.
    pub kind: String,
    pub day: f32,
    pub pos: Option<BlockPos>,
    /// What it concerned: the mob kind, the dimension.
    pub detail: String,
}

/// A path the target walks: points two or more blocks apart, cut where they stand still for a
/// minute, sleep or leave. Kept when walked twice alike (the way to the mine, the way home).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Route {
    pub points: Vec<BlockPos>,
    /// How many times a trail began and ended where this one does.
    pub walked: u32,
    pub first_day: f32,
}

impl Route {
    pub fn start(&self) -> BlockPos {
        self.points.first().copied().unwrap_or_default()
    }

    pub fn end(&self) -> BlockPos {
        self.points.last().copied().unwrap_or_default()
    }

    pub fn length(&self) -> f64 {
        self.points.windows(2).map(|w| distance(w[0], w[1])).sum()
    }
}

/// One session of the target's: when they joined and left, in real time, and the game's time
/// of day when they joined.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    pub began_real: u64,
    /// Zero while the session is open.
    pub ended_real: u64,
    pub began_day_time: u32,
}

/// What the target did in the day after an event landed at a place.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reaction {
    pub event: String,
    pub day: f32,
    pub pos: Option<BlockPos>,
    /// `lit` (a light placed within 16 blocks), `walled` (closed in on all four sides, see
    /// [`encloses`]),
    /// `watched` (stood facing the spot a minute), `left` (left the game within a minute),
    /// `ignored` (a day passed), or empty while it is still being watched.
    pub kind: String,
    /// What the event does to them (`Event::effect`): `sounds`, `words`, `things`, `house`,
    /// `seen`, `sky`, `night`.
    pub effect: String,
    /// Where it was, or where the target stood when it had no place: what running is
    /// measured from.
    pub anchor: BlockPos,
    /// The turns and look-backs the target had made in the last minute when it landed.
    pub turns_at: u32,
    pub looks_at: u32,
    /// They turned or looked back more within ten seconds.
    pub startled: bool,
    /// Ticks facing the spot within 8 blocks, standing; three seconds is a look.
    pub looked_ticks: u32,
    pub looked: bool,
    /// Minutes until they broke or placed a block where it changed one.
    pub fixed_minutes: Option<u32>,
    /// The nearest they were to it in the first minute.
    pub nearest: f64,
    /// They ran from it in the first minute, or hit something then (`seen` only).
    pub ran: bool,
    pub fought: bool,
}

impl Reaction {
    /// Whether they took any notice: anything they did about it.
    pub fn noticed(&self) -> bool {
        matches!(self.kind.as_str(), "lit" | "walled" | "watched" | "left")
            || self.startled
            || self.looked
            || self.fixed_minutes.is_some()
            || self.ran
            || self.fought
    }
}

/// Ticks as a fraction of a day.
const fn ticks(n: u32) -> f32 {
    n as f32 / 24_000.0
}

/// A reaction is watched this long; then it is settled.
const REACTION_DAYS: f32 = 1.0;
/// At most this many reactions open at once, the oldest closed first.
const MOST_OPEN_REACTIONS: usize = 8;

/// What is around the home, from the host's daily pass: water and how wide, snow, sand, trees,
/// crops, a village or a mineshaft, and where each is. Rewritten each day.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Surround {
    pub day: f32,
    pub around: BlockPos,
    pub features: Vec<Feature>,
}

/// An animal or villager near the target, from the last entity pass.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Creature {
    pub id: u32,
    /// `cow`, `sheep`, `villager`, `iron_golem`, `wolf`: the id without its namespace.
    pub kind: String,
    pub pos: [f64; 3],
    pub named: bool,
    /// Tamed by the target.
    pub pet: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Held {
    pub item: ItemStack,
    pub from: BlockPos,
    pub slot: u32,
    pub taken_day: f32,
}

/// A figure gone, and how they looked at it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FigureLook {
    pub day: f32,
    pub pos: Option<BlockPos>,
    /// Ticks facing it within ten degrees, in looks of half a second or more.
    pub looked_ticks: u32,
    pub looks: u32,
    /// Blocks they came toward it from where they first stood.
    pub toward: f32,
    /// `approached`, `looked_away`, `timed` or `gone`.
    pub end: String,
}

impl FigureLook {
    /// From the host's detail: `looked=40 looks=1 longest=40 toward=6.5 end=approached`.
    pub fn parse(detail: &str, day: f32, pos: Option<BlockPos>) -> Self {
        let mut look = Self {
            day,
            pos,
            ..Self::default()
        };
        for part in detail.split_whitespace() {
            match part.split_once('=') {
                Some(("looked", v)) => look.looked_ticks = v.parse().unwrap_or(0),
                Some(("looks", v)) => look.looks = v.parse().unwrap_or(0),
                Some(("toward", v)) => look.toward = v.parse().unwrap_or(0.0),
                Some(("end", v)) => look.end = v.to_string(),
                _ => {}
            }
        }
        look
    }

    /// Looked at for a while: two seconds or more in all.
    pub fn looked_long(&self) -> bool {
        self.looked_ticks >= 40
    }

    /// Went at it: walked eight blocks or more toward it, or came close enough that it went.
    pub fn chased(&self) -> bool {
        self.toward >= 8.0 || self.end == "approached"
    }
}

/// The most figure looks kept.
const MOST_FIGURE_LOOKS: usize = 16;

/// An event that landed, or did not.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Done {
    pub event: String,
    pub pos: Option<BlockPos>,
    pub day: f32,
    pub ok: bool,
    pub reason: String,
}

/// The target as last seen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshot {
    pub pos: [f64; 3],
    pub yaw: f32,
    pub online: bool,
    pub indoors: bool,
    pub sleeping: bool,
    pub walking: bool,
    pub main_hand: String,
    /// Animals and villagers within 32 blocks, from the last entity pass.
    pub creatures_near: Vec<Creature>,
    /// Hostile mobs within 48 blocks, from the last entity pass: zombies, skeletons, spiders.
    pub hostiles_near: Vec<Creature>,
    /// Something of another horror mod's within 64 blocks of the target in the last seconds:
    /// the lodger holds off while it is (owner: "never at the same moment").
    pub other_horror_near: bool,
    /// The ids of the animals near that are tied by a lead to a post (`horse_moved`).
    pub leashed: Vec<u32>,
    pub look_backs: u32,
    /// Turns of more than 120 degrees in the last minute.
    pub turns: u32,
    pub raining: bool,
    /// Under the ground: no sky light and below the surface.
    pub underground: bool,
    /// The day they last ate, for the sound that answers it.
    pub ate_day: f32,
    /// Crouched and still ten seconds or more (`breath`).
    pub crouched_still: bool,
    /// Crouching now.
    pub sneaking: bool,
    /// The lodger's reading of how they feel, from the model's `How does the player feel?`:
    /// `calm`, `curious`, `uneasy`, `afraid`, `angry`, or empty before the first reading.
    pub mood: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Memory {
    pub target: Option<u32>,
    pub target_name: String,
    pub target_uuid: String,
    pub home: Option<Home>,
    pub containers: Vec<Container>,
    /// Item ids and how often they were in the target's main hand, most first.
    pub favourites: Vec<(String, u32)>,
    pub held: Option<Held>,
    pub done: Vec<Done>,
    pub last: Snapshot,
    /// Coarse cells the target stood still in, and for how many ticks: the home's center comes
    /// from the densest.
    pub still_cells: Vec<([i32; 3], u32)>,
    /// The last chat lines about the lodger, newest last.
    pub chat: Vec<String>,
    /// The lodger's place, where it leaves things, once chosen.
    pub place: Option<BlockPos>,
    /// Blocks the target placed that the lodger must never break: everything they placed.
    pub placed: Vec<BlockPos>,
    /// Placed blocks an event can use, by plain kind.
    pub fixtures: Vec<Fixture>,
    /// Frames, paintings and stands near the home, from the last entity pass.
    pub decor: Vec<Decor>,
    /// The last thing the target said in chat, whatever it was about.
    pub last_said: String,
    /// The kind the target just killed, taken by the brain on its next tick (`count_up`). Not
    /// saved.
    #[serde(skip)]
    pub killed_now: Option<String>,
    /// A bell the target just rang, taken by the brain on its next tick (`bell_twice`). Not
    /// saved.
    #[serde(skip)]
    pub rang_bell_now: Option<BlockPos>,
    /// A door the target just knocked on, taken by the brain on its next tick
    /// (`answer_knock`). Not saved.
    #[serde(skip)]
    pub knocked_now: Option<BlockPos>,
    /// How each figure shown them was looked at as it went, the last sixteen (the host's
    /// `figure_seen`): for the situation, temper and the chains.
    pub figure_looks: Vec<FigureLook>,
    /// Its temper, moved by how they treat it (`temper.rs`).
    pub temper: crate::temper::Temper,
    /// The day its diary of them went on sale (0: never), and whether they bought it
    /// (`books.rs`).
    pub diary_offered: f32,
    pub diary_bought: bool,
    /// When the target was last seen, seconds since the Unix epoch by the real clock; and
    /// whether it has changed target this stay (`multiplayer.rs`).
    pub target_seen_unix: u64,
    pub retargeted: bool,
    /// The other players here now, by index, and where. Not saved.
    #[serde(skip)]
    pub others_online: Vec<u32>,
    #[serde(skip)]
    pub others_at: Vec<BlockPos>,
    /// The chains under way, and those run this stay (`chains.rs`).
    pub chains: Vec<crate::chains::Open>,
    pub chains_done: Vec<String>,
    /// Of those, the ones walked to their last step.
    pub chains_walked: Vec<String>,
    /// The kind its kill sign counts, as it wrote it (`count_up` counts that kind, not
    /// whatever is killed most now).
    pub kill_sign_kind: String,
    /// The day something of another horror mod's was last near the target.
    pub horror_seen_day: f32,
    /// The other mods the game runs with, by id and version, as the host told them: for the
    /// debug screen, and for a mind that would keep out of another mod's way.
    pub mods: Vec<(String, String)>,
    /// What of theirs stood within five blocks of its place when it chose it: what they build
    /// there after meets its want, this does not (a snapshot, where a count into a list that
    /// drops its oldest and its broken would drift).
    pub placed_at_place: Vec<BlockPos>,
    /// The confrontation's command, so a refusal from the host ends it at once. Not saved.
    #[serde(skip)]
    pub confront_id: u64,
    /// The lights it placed itself, for the endings (its torches out one by one, or lit at
    /// once).
    pub its_lights: Vec<BlockPos>,
    /// The world's difficulty, as last seen, for the endings.
    pub difficulty: String,
    /// The debug screen asked for the fight now, where the target stands. Not saved.
    #[serde(skip)]
    pub fight_now: bool,
    /// How the confrontation ended (`won`, `lost`, `fled`), taken by the ending on its next
    /// tick. Not saved.
    #[serde(skip)]
    pub fight_ended: Option<String>,
    /// The day the target went into another dimension (gone from the world without leaving
    /// the game), until they come back (`busy_away`).
    pub elsewhere_since: Option<f32>,
    /// The target has just come back from another dimension, after this many days; taken by
    /// the brain on its next tick (`busy_away`). Not saved.
    #[serde(skip)]
    pub back_after: Option<f32>,
    /// The target just woke, taken by the brain on its next tick (`foot_of_bed`). Not saved.
    #[serde(skip)]
    pub woke_now: bool,
    /// The target has just been crouched still ten seconds, taken by the brain on its next
    /// tick (`breath`). Not saved.
    #[serde(skip)]
    pub crouched_now: bool,
    /// A door the target just used, taken by the brain on its next tick (`door_held`). Not
    /// saved.
    #[serde(skip)]
    pub door_used_now: Option<BlockPos>,
    /// An offering is armed for the target's next fresh loot (`arm_loot`). Not saved: the
    /// host forgets it on a restart, so it is armed again.
    #[serde(skip)]
    pub loot_armed: bool,
    /// A chest of theirs the target just opened, taken by the brain on its next tick
    /// (`chest_answers`). Not saved.
    #[serde(skip)]
    pub opened_now: Option<BlockPos>,
    /// The house the target is in when it is not their home, from the mind's own scan
    /// (`brain::sounds::house_spot`): its doors and chests. Not saved.
    #[serde(skip)]
    pub here: Option<Here>,
    /// The id of that scan, so its answer is told from the village's and the ore's.
    #[serde(skip)]
    pub here_scan: u64,
    /// The home read's scan and its box: the house around the home, read often
    /// (`Memory::read_home`). Not saved.
    #[serde(skip)]
    pub home_scan: (u64, BlockPos, BlockPos),
    /// The target built or broke something near the home since the last read.
    #[serde(skip)]
    pub home_dirty: bool,
    /// What it dug and holds, by block id and count: the only blocks and items it may give
    /// without having taken them first (`chest_full_of_one`). Filled from the host's reports
    /// ("replaced minecraft:dirt"), spent as it gives.
    pub stores: Vec<(String, u32)>,
    /// Every line it wrote this stay (signs, chat, names), newest last: none is used twice.
    pub lines_written: Vec<String>,
    /// What the target just typed, cleaned (`text::heard`), and whether it was about the
    /// lodger: taken by the brain on its next tick to answer or not. Not saved.
    #[serde(skip)]
    pub heard: Option<(String, bool)>,
    /// What they hit and killed, by kind, most killed first.
    pub tally: Vec<Tally>,
    /// The events the target would have noticed (the catalogue's bold rows), kept for the
    /// whole stay where `done` trims: what it took and where it left things is never forgotten.
    pub landmarks: Vec<Done>,
    /// What happened to the target, kept for the whole stay.
    pub milestones: Vec<Milestone>,
    /// What is around the home, from the host's pass, once a day.
    pub surroundings: Option<Surround>,
    /// Boats and minecarts near the home, from the last entity pass.
    pub vehicles: Vec<Creature>,
    /// Blocks a scan found near the village (doors, the bell), for the village events.
    pub scanned: Vec<(String, BlockPos)>,
    /// Flowers, pumpkins and saplings around the home, by block id, from the home read.
    pub plants: Vec<(String, BlockPos)>,
    /// Natural blocks the target dug out near the home, where and what, for `backfill`.
    pub holes: Vec<(BlockPos, String)>,
    /// Ore a scan found in the rock around the target (`scan_ore`), kept apart from the
    /// village's: one scan answering used to wipe the other.
    pub ores: Vec<(String, BlockPos)>,
    /// Which event sent each scan still to be answered, by its id. Not saved.
    #[serde(skip)]
    pub scans_out: Vec<(u64, String)>,
    /// The day as of the last observation, so the offer can read "just now" without a clock.
    pub today: f32,
    /// The trail being walked right now.
    pub trail: Vec<BlockPos>,
    /// Routes walked, most walked first.
    pub routes: Vec<Route>,
    /// The target's sessions, newest last.
    pub sessions: Vec<Session>,
    /// What the target did after each event the lodger landed.
    pub reactions: Vec<Reaction>,
    /// The clock as of the last tick, for the sessions.
    pub last_real_millis: u64,
    pub last_day_time: u32,
    /// What it wants from this player, chosen once at move-in (`docs/entity.md`, What it
    /// wants): `a place of its own`, `to be looked at`, `one thing of theirs`; empty until then.
    pub want: String,
    /// Every player seen, by index and name, for the target ask in multiplayer.
    pub others: Vec<(u32, String, String)>,
    /// Whether the target is left for the brain to pick (multiplayer, `multiplayer.pick` at
    /// `model`) rather than being the first player seen.
    pub defer_target: bool,
    /// Things that just happened that the brain may react to, drained each tick:
    /// `slept`, `opened_chest`, `died`, `lit`, `walled`, `watched`, `left`.
    pub triggers: Vec<String>,
    /// The building blocks the target placed, with their ids, for the events that change one
    /// into its cousin (owner: "replace certain blocks of the house").
    pub walls: Vec<(BlockPos, String)>,
    /// Their habits: the door, the day, sleep, what they carry, lights, looking back
    /// (`crate::habits`).
    pub habits: crate::habits::Habits,
    /// Who comes to the home and who is with the target now (`crate::people`).
    pub people: crate::people::People,
    /// The places they stay away from home, most stayed at first (`crate::places`).
    pub sites: Vec<crate::places::Site>,
    /// The dark places of the last cave read: air with no light, below the surface, around
    /// the target near home (`crate::caves`).
    pub caves: Vec<BlockPos>,
    /// The crop blocks of the last field read, around the crops by the home (`crate::farm`).
    pub field: Vec<BlockPos>,
    /// Their tunnels, what they dug underground grouped as it was dug (`crate::tunnels`).
    pub tunnels: Vec<crate::tunnels::Tunnel>,
    /// Obsidian they placed: a portal frame (`portal_lit`).
    pub placed_obsidian: Vec<BlockPos>,
    /// What it builds, and how far each has got (`crate::building`).
    pub builds: Vec<crate::building::Build>,
    /// Where they last stood fishing (`dock`).
    pub last_fished: Option<BlockPos>,
}

/// Caps, with the reason beside each.
const MOST_LINES_WRITTEN: usize = 128;
/// The most characters of the situation's lines before the last ones at the default context
/// (`describe_chars(1536)`); the brain passes its own config's (`describe_within`).
pub const DESCRIBE_CHARS: usize = 4_676;

/// How many characters of memory lines fit a context of `tokens`: 200 tokens kept for the
/// frame, the question, its longest choice and a writing prompt's examples, and 3.5 characters
/// a token, under what Qwen's tokenizer gets from plain English (about 4), so it errs short.
/// A prompt past the context fails and the question falls to the rules.
pub fn describe_chars(tokens: u32) -> usize {
    (tokens.saturating_sub(200) as usize * 7) / 2
}
/// How long a walk taken once must be to stand as the route until one is walked twice.
pub const FIRST_ROUTE: f64 = 32.0;
const MOST_CONTAINERS: usize = 32; // the least-opened of those not theirs dropped first
const MOST_DONE: usize = 256; // oldest dropped first
const MOST_FAVOURITES: usize = 8;
const MOST_CELLS: usize = 512; // the least-stood-in dropped first
const MOST_CHAT: usize = 8;
const MOST_PLACED: usize = 2048; // a whole house; older ones fall off, which is the risk the docs name
const MOST_LIGHTS: usize = 64;
const MOST_DOORS: usize = 16;
const MOST_FIXTURES: usize = 128; // oldest dropped first
const MOST_DECOR: usize = 32;
const MOST_CREATURES: usize = 48;
const MOST_TALLY: usize = 32; // the least-hit kind dropped first
const MOST_LANDMARKS: usize = 512; // a stay never reaches it
const MOST_ITS_LIGHTS: usize = 128;
const MOST_MILESTONES: usize = 128;
const MOST_SCANNED: usize = 64;
const MOST_VEHICLES: usize = 8;
const MOST_ROUTES: usize = 16; // the least-walked dropped first
const MOST_ROUTE_POINTS: usize = 128; // two blocks apart: a route of 256 blocks
const LEAST_ROUTE_POINTS: usize = 10;
const MOST_SESSIONS: usize = 30;
const MOST_REACTIONS: usize = 128; // oldest dropped first
const MOST_WALLS: usize = 512; // oldest dropped first

/// The plain kind of a placed block an event can use, or none.
fn fixture_kind(id: &str) -> Option<&'static str> {
    const KINDS: &[&str] = &[
        "fence_gate",
        "glass_pane",
        "glass",
        "lever",
        "campfire",
        "cauldron",
        "crafting_table",
        "note_block",
        "bell",
        "jukebox",
        "lectern",
        "anvil",
        "carpet",
        "stairs",
        "banner",
        "piston",
        "redstone_lamp",
        "sign",
        "trapdoor",
        "pressure_plate",
        "button",
        "flower_pot",
        "composter",
        "hopper",
        "shulker_box",
        "ladder",
        "beehive",
        "bee_nest",
        "brewing_stand",
        "grindstone",
    ];
    if is_mod_furniture(id) {
        return Some("furniture");
    }
    let name = id.rsplit(':').next().unwrap_or(id);

    KINDS
        .iter()
        .find(|kind| name == **kind || name.ends_with(&format!("_{kind}")))
        .copied()
}

fn is_decor(kind: &str) -> bool {
    matches!(
        kind,
        "minecraft:item_frame"
            | "minecraft:glow_item_frame"
            | "minecraft:painting"
            | "minecraft:armor_stand"
    )
}

/// A light's block standing and on a wall, from the id of either: `minecraft:wall_torch` is
/// `(minecraft:torch, minecraft:wall_torch)`; a lantern has no wall form.
pub fn light_kinds(id: &str) -> (String, String) {
    for (standing, wall) in [
        ("minecraft:soul_torch", "minecraft:soul_wall_torch"),
        ("minecraft:redstone_torch", "minecraft:redstone_wall_torch"),
        ("minecraft:copper_torch", "minecraft:copper_wall_torch"),
        ("minecraft:torch", "minecraft:wall_torch"),
    ] {
        if id == standing || id == wall {
            return (standing.to_string(), wall.to_string());
        }
    }

    (id.to_string(), String::new())
}

/// The block a position is in.
pub fn block_of(pos: [f64; 3]) -> BlockPos {
    block(pos)
}

fn block(pos: [f64; 3]) -> BlockPos {
    [
        pos[0].floor() as i32,
        pos[1].floor() as i32,
        pos[2].floor() as i32,
    ]
}

pub fn distance(a: BlockPos, b: BlockPos) -> f64 {
    let dx = (a[0] - b[0]) as f64;
    let dy = (a[1] - b[1]) as f64;
    let dz = (a[2] - b[2]) as f64;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Whether the target faces `pos` from where `player` stands, within `blocks`.
fn in_view_from(player: &Player, pos: BlockPos, blocks: f64) -> bool {
    let at = block(player.pos);
    if distance(at, pos) > blocks {
        return false;
    }
    let dx = (pos[0] - at[0]) as f64;
    let dz = (pos[2] - at[2]) as f64;
    if dx == 0.0 && dz == 0.0 {
        return true;
    }
    let facing_x = -(player.yaw as f64).to_radians().sin();
    let facing_z = (player.yaw as f64).to_radians().cos();
    let length = (dx * dx + dz * dz).sqrt();

    (dx * facing_x + dz * facing_z) / length > 0.5
}

/// The woods and the colours the game has, from which the families below are made, so every
/// kind of planks and every colour of wool is covered without a table per block.
pub const WOODS: &[&str] = &[
    "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove", "cherry", "bamboo",
    "pale_oak", "crimson", "warped",
];
pub const COLOURS: &[&str] = &[
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];
/// Families that are their own lists.
const STONES: &[&str] = &[
    "stone",
    "cobblestone",
    "mossy_cobblestone",
    "stone_bricks",
    "cracked_stone_bricks",
    "mossy_stone_bricks",
    "chiseled_stone_bricks",
    "andesite",
    "polished_andesite",
    "diorite",
    "granite",
    "tuff",
    "deepslate_bricks",
    "cracked_deepslate_bricks",
    "deepslate_tiles",
];
/// Seconds crouched and still before the breathing at their ear (`breath`).
pub const CROUCHED_STILL: f32 = 10.0;

const BRICKS: &[&str] = &["bricks", "mud_bricks", "nether_bricks", "red_nether_bricks"];
// Never a redstone torch: one set lit beside a door, a piston or TNT powers it.
const TORCHES: &[&str] = &["torch", "soul_torch"];
const WALL_TORCHES: &[&str] = &["wall_torch", "soul_wall_torch"];
const LANTERNS: &[&str] = &["lantern", "soul_lantern"];
const PANES: &[&str] = &["glass_pane", "iron_bars"];
const GLASSES: &[&str] = &["glass", "tinted_glass"];

/// The family a building block belongs to, as the ids of every member, or none for a block
/// the lodger never changes (a chest, a bed, a door, the ground). The families are the woods
/// by suffix (`_planks`, `_log`, `_stripped_log`, `_fence`, `_slab`, `_stairs`), the colours by
/// kind (`_wool`, `_concrete`, `_terracotta`, `_carpet`, `_stained_glass`,
/// `_stained_glass_pane`), the stones, the bricks, the torches and the lanterns.
pub fn family(id: &str) -> Vec<String> {
    let name = id.rsplit(':').next().unwrap_or(id);
    let with = |members: &[&str]| -> Vec<String> {
        members.iter().map(|m| format!("minecraft:{m}")).collect()
    };
    for list in [
        STONES,
        BRICKS,
        TORCHES,
        WALL_TORCHES,
        LANTERNS,
        PANES,
        GLASSES,
    ] {
        if list.contains(&name) {
            return with(list);
        }
    }
    for suffix in ["_planks", "_log", "_fence", "_slab", "_stairs", "_trapdoor"] {
        if let Some(wood) = name.strip_suffix(suffix) {
            let wood = wood.strip_prefix("stripped_").unwrap_or(wood);
            if WOODS.contains(&wood) {
                let stripped = name.starts_with("stripped_");
                return WOODS
                    .iter()
                    .map(|w| {
                        format!(
                            "minecraft:{}{w}{suffix}",
                            if stripped { "stripped_" } else { "" }
                        )
                    })
                    .collect();
            }
        }
    }
    for kind in [
        "_wool",
        "_concrete",
        "_terracotta",
        "_carpet",
        "_stained_glass_pane",
        "_stained_glass",
    ] {
        if let Some(colour) = name.strip_suffix(kind) {
            if COLOURS.contains(&colour) {
                return COLOURS
                    .iter()
                    .map(|c| format!("minecraft:{c}{kind}"))
                    .collect();
            }
        }
    }

    Vec::new()
}

/// The other members of a block's family: what it may be swapped for. Empty for a block the
/// lodger never changes.
pub fn cousins(id: &str) -> Vec<String> {
    family(id).into_iter().filter(|m| m != id).collect()
}

/// Whether an item id is a dyed kind: `red_wool`, `lime_concrete`, `blue_stained_glass`.
pub fn is_coloured(id: &str) -> bool {
    const COLOURS: &[&str] = &[
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "light_gray",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black",
    ];
    let name = id.rsplit(':').next().unwrap_or(id);
    (name.ends_with("_wool")
        || name.ends_with("_concrete")
        || name.ends_with("_stained_glass")
        || name.ends_with("_bed")
        || name.ends_with("_carpet")
        || name.ends_with("_terracotta"))
        && COLOURS.iter().any(|c| name.starts_with(c))
}

fn is_light(id: &str) -> bool {
    id.ends_with("torch") || id.ends_with("lantern") || id.contains("candle") || is_mod_lamp(id)
}

/// A lamp of another mod's (their light, as a torch is), never the game's redstone lamp.
pub fn is_mod_lamp(id: &str) -> bool {
    let (space, name) = id.split_once(':').unwrap_or(("minecraft", id));
    space != "minecraft" && name.ends_with("lamp")
}

/// A piece of another mod's furniture, which it moves as it moves a work block.
pub fn is_mod_furniture(id: &str) -> bool {
    let (space, name) = id.split_once(':').unwrap_or(("minecraft", id));
    space != "minecraft"
        && [
            "_chair",
            "_table",
            "_desk",
            "_shelf",
            "_sofa",
            "_couch",
            "_stool",
            "_bench",
            "_cabinet",
            "_drawer",
            "_wardrobe",
        ]
        .iter()
        .any(|s| name.ends_with(s))
}

/// Words in the ids of mods that are horror mods of their own: with one loaded, the lodger
/// makes room (owner).
const HORROR_WORDS: &[&str] = &[
    "horror",
    "dweller",
    "stalker",
    "mimic",
    "herobrine",
    "skinwalker",
    "nextbot",
    "scp",
    "backrooms",
    "obsessed",
    "creepypasta",
    "jumpscare",
    "haunt",
    "weeping",
    "from_the_fog",
    "fromthefog",
    "goatman",
    "siren_head",
    "sirenhead",
    "paranormal",
    "dread",
];

/// Whether a mod's id reads as a horror mod's.
pub fn is_horror_mod(id: &str) -> bool {
    let id = id.to_lowercase();
    HORROR_WORDS.iter().any(|w| id.contains(w))
}

fn is_door(id: &str) -> bool {
    id.ends_with("_door") && !id.contains("trapdoor")
}

/// Which of the four sides of `spot` a block is on, if it walls that side: one or two blocks
/// out along that side, nearer the middle than the corner, level with the spot or one above. A
/// block on the diagonal walls no side.
fn side_of(spot: BlockPos, block: BlockPos) -> Option<usize> {
    let (dx, dy, dz) = (block[0] - spot[0], block[1] - spot[1], block[2] - spot[2]);
    if !(0..=1).contains(&dy) || dx.abs() > 2 || dz.abs() > 2 || dx.abs() == dz.abs() {
        return None;
    }
    Some(if dx.abs() > dz.abs() {
        usize::from(dx < 0)
    } else {
        2 + usize::from(dz < 0)
    })
}

/// Whether blocks the target placed close `spot` in on all four sides.
fn encloses(spot: BlockPos, placed: &[BlockPos]) -> bool {
    let mut sides = [false; 4];
    for block in placed {
        if let Some(side) = side_of(spot, *block) {
            sides[side] = true;
        }
    }

    sides.iter().all(|s| *s)
}

fn is_bed(id: &str) -> bool {
    id.ends_with("_bed")
}

/// A block that stores items the lodger may use: a chest, a trapped chest, a barrel, a furnace,
/// a smoker. Never an ender chest, which is the player's alone everywhere.
fn is_storage(id: &str) -> bool {
    (id.ends_with("chest") && !id.ends_with("ender_chest"))
        || id.ends_with("barrel")
        || id.ends_with("furnace")
        || id.ends_with("smoker")
}

impl Memory {
    /// Takes in a tick's observations. The target is the first player seen until the brain picks
    /// another (multiplayer, a later phase).
    pub fn observe(&mut self, observations: &Observations, day: f32) {
        self.today = day;
        for player in &observations.players {
            // An index is the host's join order for the server's life: after a restart another
            // player may hold it. A uuid already known takes its new index.
            if let Some(known) = self
                .others
                .iter_mut()
                .find(|(_, _, uuid)| !uuid.is_empty() && *uuid == player.uuid)
            {
                known.0 = player.index;
                known.1 = player.name.clone();
            } else if !player.uuid.is_empty()
                || !self.others.iter().any(|(i, _, _)| *i == player.index)
            {
                // Full: one long gone (its index given up, not the target) makes room; the
                // seventeenth player ever seen used to be never known.
                if self.others.len() >= 16 {
                    let target_uuid = self.target_uuid.clone();
                    if let Some(gone) = self
                        .others
                        .iter()
                        .position(|(i, _, u)| *i == u32::MAX && *u != target_uuid)
                    {
                        self.others.remove(gone);
                    }
                }
                if self.others.len() < 16 {
                    self.others
                        .push((player.index, player.name.clone(), player.uuid.clone()));
                }
            }
            // Whoever held this index before and is someone else holds none now.
            for other in self.others.iter_mut() {
                if other.0 == player.index && !player.uuid.is_empty() && other.2 != player.uuid {
                    other.0 = u32::MAX;
                }
            }
            if !self.target_uuid.is_empty() && player.uuid == self.target_uuid {
                self.target = Some(player.index);
            }
        }
        // The first player seen is the target, unless the brain is to pick among several and
        // several are here: a player alone is the target at once, so nothing they do is missed.
        if self.target.is_none() && (!self.defer_target || observations.players.len() == 1) {
            if let Some(first) = observations.players.first() {
                self.set_target(first.index, &first.name, &first.uuid);
            }
        }
        // Who else is here and where (`multiplayer.rs`), and when the target was last seen, by
        // the real clock.
        self.others_online.clear();
        self.others_at.clear();
        for player in &observations.players {
            // The target by uuid: after a restart another player may hold their old index.
            let is_target = Some(player.index) == self.target
                && (self.target_uuid.is_empty()
                    || player.uuid.is_empty()
                    || player.uuid == self.target_uuid);
            if is_target {
                self.target_seen_unix = crate::multiplayer::now_unix();
            } else {
                self.others_online.push(player.index);
                self.others_at.push(block(player.pos));
            }
        }

        // Near in the last five seconds or so: an entity pass with nothing near says no more.
        self.last.other_horror_near =
            self.horror_seen_day > 0.0 && (day - self.horror_seen_day) < 0.004;
        if !observations.mods.is_empty() {
            self.mods = observations
                .mods
                .iter()
                .take(512)
                .map(|m| (m.id.clone(), m.version.clone()))
                .collect();
        }
        if let Some(world) = &observations.world {
            self.last.raining = world.raining;
            if !world.difficulty.is_empty() {
                self.difficulty = world.difficulty.clone();
            }
        }

        if let Some(around) = &observations.surroundings {
            self.surroundings = Some(Surround {
                day,
                around: around.around,
                features: around
                    .features
                    .iter()
                    .filter(|f| !f.kind.starts_with("house_"))
                    .cloned()
                    .collect(),
            });
            self.read_house(&around.features);
        }

        for scan in &observations.scans {
            if self.home_scan.0 != 0 && scan.id == self.home_scan.0 {
                let (_, min, max) = self.home_scan;
                self.read_home(&scan.positions, min, max);
                continue;
            }
            if self.here_scan != 0 && scan.id == self.here_scan {
                let of = |test: fn(&str) -> bool| -> Vec<BlockPos> {
                    scan.positions
                        .iter()
                        .filter(|(kind, _)| test(kind))
                        .map(|(_, pos)| *pos)
                        .take(16)
                        .collect()
                };
                self.here = Some(Here {
                    doors: lower_halves(&of(|k| k.ends_with("_door"))),
                    containers: of(|k| k.ends_with("chest") || k.ends_with("barrel")),
                    panes: of(|k| k.ends_with("glass_pane") || k.ends_with("glass")),
                    day,
                });
                continue;
            }
            let by = self
                .scans_out
                .iter()
                .find(|(id, _)| *id == scan.id)
                .map(|(_, event)| event.clone());
            self.scans_out.retain(|(id, _)| *id != scan.id);
            if by.as_deref() == Some("field_read") {
                self.field = scan
                    .positions
                    .iter()
                    .take(crate::farm::MOST_FIELD)
                    .map(|(_, pos)| *pos)
                    .collect();
                continue;
            }
            if by.as_deref() == Some("cave_read") {
                self.caves = scan
                    .positions
                    .iter()
                    .take(crate::caves::MOST_CAVE)
                    .map(|(_, pos)| *pos)
                    .collect();
                continue;
            }
            let found: Vec<(String, BlockPos)> = scan
                .positions
                .iter()
                .take(MOST_SCANNED)
                .map(|(kind, pos)| (kind.rsplit(':').next().unwrap_or(kind).to_string(), *pos))
                .collect();
            match by.as_deref() {
                Some("scan_ore") => self.ores = found,
                _ => self.scanned = found,
            }
        }

        let Some(index) = self.target else { return };
        // The target by their uuid when it is known: another player at their old index is not
        // them.
        let player = observations
            .players
            .iter()
            .find(|p| p.index == index)
            .filter(|p| {
                self.target_uuid.is_empty() || p.uuid.is_empty() || p.uuid == self.target_uuid
            });
        // Someone else at the target's index: their events and their chat are not the target's
        // (they used to move the home's bed, the placed blocks and the temper).
        let index = if player.is_none() && observations.players.iter().any(|p| p.index == index) {
            u32::MAX
        } else {
            index
        };
        self.observe_people(observations, day);

        match player {
            Some(player) => {
                self.observe_player(player);
                self.sample_habits(player, day);
            }
            None => {
                // Gone from this world without leaving the game: into another dimension.
                let left = observations
                    .events
                    .iter()
                    .any(|e| e.player == index && e.kind == EventKind::Left);
                if self.last.online && !left {
                    self.elsewhere_since = Some(day);
                }
                self.last.online = false;
                self.habits.at_home = None;
            }
        }

        if !observations.entities.is_empty() {
            let at = block(self.last.pos);
            self.last.creatures_near = observations
                .entities
                .iter()
                .filter(|e| {
                    !e.hostile
                        && !is_decor(&e.kind)
                        && !matches!(
                            e.kind.as_str(),
                            "minecraft:item"
                                | "minecraft:player"
                                | "minecraft:experience_orb"
                                | "minecraft:arrow"
                                | "minecraft:boat"
                                | "minecraft:minecart"
                        )
                        && distance(block(e.pos), at) <= 32.0
                })
                .take(MOST_CREATURES)
                .map(|e| Creature {
                    id: e.id,
                    kind: e.kind.rsplit(':').next().unwrap_or(&e.kind).to_string(),
                    pos: e.pos,
                    named: !e.name.is_empty(),
                    pet: !e.owner.is_empty() && e.owner == self.target_uuid,
                })
                .collect();

            self.last.leashed = observations
                .entities
                .iter()
                .filter(|e| e.leashed)
                .map(|e| e.id)
                .take(MOST_CREATURES)
                .collect();
            let horror: Vec<&str> = self
                .mods
                .iter()
                .filter(|(id, _)| is_horror_mod(id))
                .map(|(id, _)| id.as_str())
                .collect();
            if !horror.is_empty()
                && observations.entities.iter().any(|e| {
                    let space = e.kind.split(':').next().unwrap_or("");
                    (horror.contains(&space) || is_horror_mod(space))
                        && distance(block(e.pos), at) <= 64.0
                })
            {
                self.horror_seen_day = day;
            }
            self.last.other_horror_near =
                self.horror_seen_day > 0.0 && (day - self.horror_seen_day) < 0.004;
            self.last.hostiles_near = observations
                .entities
                .iter()
                .filter(|e| e.hostile && distance(block(e.pos), at) <= 48.0)
                .take(MOST_CREATURES)
                .map(|e| Creature {
                    id: e.id,
                    kind: e.kind.rsplit(':').next().unwrap_or(&e.kind).to_string(),
                    pos: e.pos,
                    named: !e.name.is_empty(),
                    pet: false,
                })
                .collect();

            // Decor is what hangs or stands near the home; the pass replaces the list, since
            // the ids last only while the world is loaded.
            if let Some(home) = &self.home {
                let center = home.center;
                self.vehicles = observations
                    .entities
                    .iter()
                    .filter(|e| {
                        (e.kind.ends_with("boat")
                            || e.kind.ends_with("minecart")
                            || e.kind.ends_with("raft"))
                            && distance(block(e.pos), center) <= 64.0
                    })
                    .take(MOST_VEHICLES)
                    .map(|e| Creature {
                        id: e.id,
                        kind: e.kind.rsplit(':').next().unwrap_or(&e.kind).to_string(),
                        pos: e.pos,
                        named: false,
                        pet: false,
                    })
                    .collect();
                self.decor = observations
                    .entities
                    .iter()
                    .filter(|e| {
                        is_decor(&e.kind) && distance(block(e.pos), center) <= HOME_RADIUS as f64
                    })
                    .take(MOST_DECOR)
                    .map(|e| Decor {
                        kind: e
                            .kind
                            .rsplit(':')
                            .next()
                            .unwrap_or(&e.kind)
                            .replace("glow_", ""),
                        id: e.id,
                        pos: block(e.pos),
                    })
                    .collect();
            }
        }

        for event in &observations.events {
            // Its diary bought, by whoever bought it: never brought to them after, and its chain
            // over (the book is gone from the villager either way).
            if event.kind == EventKind::Bought
                && self.diary_offered > 0.0
                && event.detail.ends_with("written_book")
            {
                self.diary_bought = true;
                if self.chains.iter().any(|o| o.id == "the_diary") {
                    self.chains.retain(|o| o.id != "the_diary");
                    self.chains_done.push("the_diary".to_string());
                }
            }
            if event.player != index {
                continue;
            }
            // How it takes this, before the rest of the memory moves (`temper.rs`).
            self.temper_event(event, day);

            match event.kind {
                EventKind::OpenedContainer => {
                    if let Some(pos) = event.pos {
                        let used = self.used_container().map(|c| c.pos) == Some(pos);
                        self.container_opened(pos, &event.detail, &event.contents, day);
                        if used {
                            self.trigger("opened_chest");
                        }
                        if self.containers.iter().any(|c| c.pos == pos && c.theirs()) {
                            self.opened_now = Some(pos);
                        }
                        // Opening the chest something was done to is coming to look.
                        for reaction in self
                            .reactions
                            .iter_mut()
                            .filter(|r| r.pos == Some(pos) && day - r.day < REACTION_DAYS)
                        {
                            reaction.looked = true;
                        }
                    }
                }
                EventKind::SleptBegan => {
                    self.trigger("slept");
                    self.habit_slept(day);
                    if !self.milestones.iter().any(|m| m.kind == "first_sleep") {
                        self.milestone("first_sleep", day, event.pos, "");
                    }
                    if let Some(pos) = event.pos {
                        let home = self.home.get_or_insert_with(|| Home {
                            center: pos,
                            floor_y: pos[1],
                            ..Home::default()
                        });
                        home.bed = Some(pos);
                        home.center = pos;
                        home.floor_y = pos[1];
                    }
                }
                EventKind::Joined => {
                    // Joined, not back from elsewhere: a missing first observation set it.
                    self.elsewhere_since = None;
                    self.sessions.push(Session {
                        began_real: self.last_real_millis,
                        ended_real: 0,
                        began_day_time: self.last_day_time,
                    });
                    if self.sessions.len() > MOST_SESSIONS {
                        self.sessions.remove(0);
                    }
                }
                EventKind::Left => {
                    self.cut_trail();
                    if let Some(session) = self.sessions.last_mut() {
                        session.ended_real = self.last_real_millis;
                    }
                    // Leaving within a minute of an event is a reaction to it.
                    let mut left = false;
                    for reaction in self
                        .reactions
                        .iter_mut()
                        .filter(|r| r.kind.is_empty() && day - r.day < ticks(1_200))
                    {
                        reaction.kind = "left".to_string();
                        left = true;
                    }
                    if left {
                        self.triggers.push("left".to_string());
                    }
                }
                EventKind::PlacedBlock => {
                    if let Some(pos) = event.pos {
                        self.mark_home_dirty(pos);
                        push_capped(&mut self.placed, pos, MOST_PLACED);
                        self.react_to_placing(pos, &event.detail);
                        self.react_to_fixing(pos);
                        // A light is kept with its id too (candles, the soul swap): it has no
                        // family, so candles were never among them and `candles_lit` never came.
                        if !cousins(&event.detail).is_empty() || is_light(&event.detail) {
                            self.walls.retain(|(p, _)| *p != pos);
                            push_capped(&mut self.walls, (pos, event.detail.clone()), MOST_WALLS);
                        }
                        // A bed placed is as good a sign of home as a bed slept in, and comes
                        // hours sooner: a player cannot sleep before night.
                        if is_bed(&event.detail) {
                            let home = self.home.get_or_insert_with(|| Home {
                                center: pos,
                                floor_y: pos[1],
                                ..Home::default()
                            });
                            home.bed = Some(pos);
                            home.center = pos;
                            home.floor_y = pos[1];
                        }
                        if is_storage(&event.detail) {
                            self.container_placed(pos, &event.detail);
                        }
                        if let Some(kind) = fixture_kind(&event.detail) {
                            self.fixtures.retain(|f| f.pos != pos);
                            push_capped(
                                &mut self.fixtures,
                                Fixture {
                                    kind: kind.to_string(),
                                    pos,
                                },
                                MOST_FIXTURES,
                            );
                        }
                        if event.detail == "minecraft:obsidian" {
                            push_capped(&mut self.placed_obsidian, pos, 64);
                        }
                        if is_light(&event.detail) {
                            self.habits.lights_placed += 1;
                            self.habits.last_light = Some((pos, event.detail.clone()));
                            if self.last.underground {
                                self.habits.cave_lights.push(pos);
                                if self.habits.cave_lights.len() > 32 {
                                    self.habits.cave_lights.remove(0);
                                }
                            }
                        }
                        // Only by the home: a mine lit three hundred blocks off used to fill
                        // the home's lights, and a far base's door became its usual door.
                        let near_home = self
                            .home
                            .as_ref()
                            .is_none_or(|h| distance(h.center, pos) <= (HOME_RADIUS * 2) as f64);
                        if near_home && (is_light(&event.detail) || is_door(&event.detail)) {
                            let home = self.home.get_or_insert_with(|| Home {
                                center: pos,
                                floor_y: pos[1],
                                ..Home::default()
                            });
                            // One placed where one is remembered replaces it: a door put back
                            // after the floor under it was dug is still one door.
                            home.lights.retain(|p| *p != pos);
                            home.doors.retain(|p| *p != pos);
                            if is_light(&event.detail) {
                                push_capped(&mut home.lights, pos, MOST_LIGHTS);
                            } else {
                                push_capped(&mut home.doors, pos, MOST_DOORS);
                            }
                        }
                    }
                }
                EventKind::Died => {
                    self.milestone("died", day, event.pos, "");
                    self.trigger("died");
                }
                EventKind::ChangedDimension => {
                    self.milestone("crossed", day, event.pos, &event.detail);
                    if let Some(since) = self.elsewhere_since.take() {
                        self.back_after = Some(day - since);
                    }
                }
                EventKind::Ate => {
                    self.last.ate_day = day;
                }
                EventKind::Hit | EventKind::Killed => {
                    let kind = event
                        .detail
                        .rsplit(':')
                        .next()
                        .unwrap_or(&event.detail)
                        .to_string();
                    if kind.is_empty() || kind == "player" {
                        continue;
                    }
                    let killed = event.kind == EventKind::Killed;
                    if killed {
                        self.killed_now = Some(kind.clone());
                    }
                    // Hitting out in the minute after something showed itself is fighting it.
                    for reaction in self
                        .reactions
                        .iter_mut()
                        .filter(|r| r.effect == "seen" && day - r.day < ticks(1_200))
                    {
                        reaction.fought = true;
                    }
                    if killed && !self.tally.iter().any(|t| t.kind == kind && t.kills > 0) {
                        self.milestone("first_kill", day, event.pos, &kind);
                    }
                    match self.tally.iter_mut().find(|t| t.kind == kind) {
                        Some(tally) => {
                            if killed {
                                tally.kills += 1;
                            } else {
                                tally.hits += 1;
                            }
                        }
                        None => {
                            self.tally.push(Tally {
                                kind,
                                hits: u32::from(!killed),
                                kills: u32::from(killed),
                            });
                            if self.tally.len() > MOST_TALLY {
                                let least = self
                                    .tally
                                    .iter()
                                    .enumerate()
                                    .min_by_key(|(_, t)| t.hits + t.kills)
                                    .map(|(i, _)| i)
                                    .unwrap_or(0);
                                self.tally.remove(least);
                            }
                        }
                    }
                    self.tally
                        .sort_by_key(|t| std::cmp::Reverse((t.kills, t.hits)));
                }
                EventKind::BrokeBlock => {
                    if let Some(pos) = event.pos {
                        // Gone, so not its light any more: the leave ending put out a torch of
                        // theirs set where its had been.
                        self.its_lights.retain(|p| *p != pos);
                        if self.last.underground
                            && crate::tunnels::is_rock(&event.detail)
                            && !self.placed.contains(&pos)
                        {
                            self.dug_underground(pos, day);
                        }
                        self.placed_obsidian.retain(|p| *p != pos);
                        self.mark_home_dirty(pos);
                        self.react_to_fixing(pos);
                        // Natural ground dug near the home, not something they had placed.
                        let natural = [
                            "dirt",
                            "grass_block",
                            "coarse_dirt",
                            "stone",
                            "deepslate",
                            "sand",
                            "gravel",
                            "andesite",
                            "diorite",
                            "granite",
                            "tuff",
                        ]
                        .iter()
                        .any(|k| event.detail.rsplit(':').next() == Some(*k));
                        if natural
                            && !self.placed.contains(&pos)
                            && self
                                .home
                                .as_ref()
                                .is_some_and(|h| distance(h.center, pos) <= 16.0)
                        {
                            push_capped(&mut self.holes, (pos, event.detail.clone()), 32);
                        }
                        // What stood on it drops with it (a door, a torch, a lever on the
                        // floor), and the game sends no break for those.
                        let above = [pos[0], pos[1] + 1, pos[2]];
                        if self
                            .habits
                            .last_light
                            .as_ref()
                            .is_some_and(|(p, _)| *p == pos || *p == above)
                        {
                            self.habits.last_light = None;
                        }
                        self.placed.retain(|p| *p != pos);
                        if let Some(home) = &mut self.home {
                            home.lights.retain(|p| *p != pos && *p != above);
                            home.doors.retain(|p| *p != pos && *p != above);
                        }
                        self.containers.retain(|c| c.pos != pos);
                        self.fixtures.retain(|f| f.pos != pos && f.pos != above);
                        self.walls.retain(|(p, _)| *p != pos);
                    }
                }
                EventKind::RangBell => self.rang_bell_now = event.pos,
                EventKind::Knocked => self.knocked_now = event.pos,
                // Its diary bought: taken above, from anyone.
                EventKind::Bought => {}
                EventKind::FigureSeen => {
                    push_capped(
                        &mut self.figure_looks,
                        FigureLook::parse(&event.detail, day, event.pos),
                        MOST_FIGURE_LOOKS,
                    );
                }
                EventKind::FightWon => self.fight_ended = Some("won".to_string()),
                EventKind::FightLost => self.fight_ended = Some("lost".to_string()),
                EventKind::FightFled => self.fight_ended = Some("fled".to_string()),
                EventKind::Fished => self.last_fished = Some(block(self.last.pos)),
                EventKind::UsedDoor => {
                    self.door_used_now = event.pos;
                    if let Some(pos) = event.pos {
                        self.habit_door(pos);
                    }
                }
                // Their fresh loot got its offering: what it gave it no longer holds.
                EventKind::LootTouched => {
                    self.loot_armed = false;
                    if !event.detail.is_empty() {
                        if self
                            .held
                            .as_ref()
                            .is_some_and(|h| h.item.id == event.detail)
                        {
                            self.held = None;
                        } else if self.stored(&event.detail) > 0 {
                            self.store(&event.detail, -1);
                        }
                        self.record("loot_gift", event.pos, day, true, &event.detail);
                    }
                }
                EventKind::ClosedContainer => {
                    if let Some(pos) = event.pos {
                        if let Some(container) = self.containers.iter_mut().find(|c| c.pos == pos) {
                            container.contents = event.contents.clone();
                        }
                    }
                }
                _ => {}
            }
        }

        for line in &observations.chat {
            if line.player != index {
                continue;
            }
            self.last_said = line.text.clone();
            let about = about_the_lodger(&line.text) || self.about_its_want(&line.text);
            if about {
                push_capped(&mut self.chat, line.text.clone(), MOST_CHAT);
                self.temper_chat(&line.text, day);
            }
            if let Some(words) = crate::brain::text::heard(&line.text) {
                self.heard = Some((words, about));
            }
        }
    }

    /// The house the target sleeps in, as the host read it: its doors, lights, chests and bed
    /// become the home's, whoever placed them, so a village house or torches it did not place
    /// count (owner: "the home's, placed or not"). What the target placed stays as
    /// it was; nothing read here is ever broken.
    fn read_house(&mut self, features: &[craftkit::world::Feature]) {
        let Some(home) = self.home.as_mut() else {
            return;
        };
        for feature in features {
            let pos = feature.pos;
            match feature.kind.as_str() {
                "house_door" if !home.doors.contains(&pos) => {
                    push_capped(&mut home.doors, pos, MOST_DOORS)
                }
                "house_light" if !home.lights.contains(&pos) => {
                    push_capped(&mut home.lights, pos, MOST_LIGHTS)
                }
                "house_bed" if home.bed.is_none() => home.bed = Some(pos),
                _ => {}
            }
        }
        for feature in features.iter().filter(|f| f.kind == "house_chest") {
            match self.containers.iter_mut().find(|c| c.pos == feature.pos) {
                Some(container) => container.in_home = true,
                None => {
                    self.containers.push(Container {
                        pos: feature.pos,
                        kind: feature.detail.clone(),
                        in_home: true,
                        ..Container::default()
                    });
                    self.drop_least_opened_container();
                }
            }
        }
    }

    /// Something built or broken within twelve blocks of the home: read it again soon.
    fn mark_home_dirty(&mut self, pos: BlockPos) {
        if self
            .home
            .as_ref()
            .is_some_and(|h| distance(h.center, pos) <= 12.0)
        {
            self.home_dirty = true;
        }
    }

    /// How many of a block it holds from digging.
    pub fn stored(&self, id: &str) -> u32 {
        self.stores
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }

    /// Adds to, or takes from, what it holds; never below none.
    pub fn store(&mut self, id: &str, change: i64) {
        match self.stores.iter_mut().find(|(i, _)| i == id) {
            Some((_, n)) => *n = (*n as i64 + change).max(0) as u32,
            None if change > 0 => self.stores.push((id.to_string(), change as u32)),
            None => {}
        }
        self.stores.retain(|(_, n)| *n > 0);
    }

    /// The home read: inside its box, what it found is the home's (whoever placed it) and what
    /// it did not find is gone. Doors, lights, containers, the bed, and the fixtures it covers.
    /// Read when they come home, after they build near it, and every ten minutes there
    /// (owner: "why only read the house once a day?").
    pub fn read_home(&mut self, found: &[(String, BlockPos)], min: BlockPos, max: BlockPos) {
        let inside = |p: &BlockPos| (0..3).all(|i| p[i] >= min[i] && p[i] <= max[i]);
        let of = |test: &dyn Fn(&str) -> bool| -> Vec<BlockPos> {
            found
                .iter()
                .filter(|(k, _)| test(k))
                .map(|(_, p)| *p)
                .collect()
        };
        let doors = lower_halves(&of(&|k| k.ends_with("_door")));
        // Every light placing counts, lamps and candles too: the read used to drop them.
        let lights = of(&|k| is_light(k));
        let beds = of(&|k| k.ends_with("_bed"));
        let Some(home) = self.home.as_mut() else {
            return;
        };
        home.doors.retain(|p| !inside(p));
        for d in doors {
            push_capped(&mut home.doors, d, MOST_DOORS);
        }
        home.lights.retain(|p| !inside(p));
        for l in lights {
            push_capped(&mut home.lights, l, MOST_LIGHTS);
        }
        let bed_gone = home.bed.is_some_and(|b| {
            inside(&b)
                && !beds
                    .iter()
                    .any(|f| (0..3).map(|i| (f[i] - b[i]).abs()).sum::<i32>() <= 1)
        });
        if bed_gone || home.bed.is_none() {
            home.bed = beds.first().copied();
        }

        let stores = of(&|k| is_storage(k));
        self.containers
            .retain(|c| !inside(&c.pos) || stores.contains(&c.pos));
        for (kind, pos) in found.iter().filter(|(k, _)| is_storage(k)) {
            match self.containers.iter_mut().find(|c| c.pos == *pos) {
                Some(container) => container.in_home = true,
                None => {
                    self.containers.push(Container {
                        pos: *pos,
                        kind: kind.clone(),
                        in_home: true,
                        ..Container::default()
                    });
                    self.drop_least_opened_container();
                }
            }
        }

        self.plants.retain(|(_, p)| !inside(p));
        for (id, pos) in found {
            if is_flower(id) || id == "minecraft:pumpkin" || id.ends_with("_sapling") {
                self.plants.push((id.clone(), *pos));
            }
        }
        self.plants.truncate(32);
        self.fixtures.retain(|f| {
            !(inside(&f.pos) && READ_FIXTURES.contains(&f.kind.as_str()))
                || found.iter().any(|(_, p)| *p == f.pos)
        });
        for (id, pos) in found {
            if let Some(kind) = fixture_kind(id).filter(|k| READ_FIXTURES.contains(k)) {
                if !self.fixtures.iter().any(|f| f.pos == *pos) {
                    push_capped(
                        &mut self.fixtures,
                        Fixture {
                            kind: kind.to_string(),
                            pos: *pos,
                        },
                        MOST_FIXTURES,
                    );
                }
            }
        }
    }

    /// Keeps a line it wrote, so it is not written again.
    pub fn wrote(&mut self, line: String) {
        if !line.is_empty() && !self.lines_written.contains(&line) {
            push_capped(&mut self.lines_written, line, MOST_LINES_WRITTEN);
        }
    }

    /// Whether a line names what it wants: `place`, `look`, `mine`, by the want's own words.
    fn about_its_want(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        let words: &[&str] = match self.want.as_str() {
            "a place of its own" => &["place", "room", "space", "build"],
            "to be looked at" => &["look", "see", "watch", "saw"],
            "one thing of theirs" => &["mine", "take", "took", "stole", "missing"],
            _ => &[],
        };

        words.iter().any(|w| lower.contains(w))
    }

    fn observe_player(&mut self, player: &Player) {
        self.last.pos = player.pos;
        self.last.yaw = player.yaw;
        self.last.online = true;
        if self.last.sleeping && !player.sleeping {
            self.woke_now = true;
        }
        self.last.sleeping = player.sleeping;
        self.last.walking = player.movement.pace > 0.5;
        let crouched = player.sneaking && player.movement.still_seconds >= CROUCHED_STILL;
        if crouched && !self.last.crouched_still {
            self.crouched_now = true;
        }
        self.last.crouched_still = crouched;
        self.last.sneaking = player.sneaking;
        self.last.underground = player.sky_light == 0 && player.pos[1] < 56.0;
        // Under a roof, not under the ground: a cave is no house (the dev client played a
        // house's door sound 55 blocks down).
        self.last.indoors = player.sky_light < 8 && !self.last.underground;
        self.last.main_hand = player.main_hand.id.clone();
        self.last.look_backs = player.movement.look_backs;
        self.last.turns = player.movement.turns;

        // The trail: a point every two blocks; cut when they stand still a minute or sleep.
        let at = block(player.pos);
        if player.movement.still_seconds >= 60.0 || player.sleeping {
            self.cut_trail();
        } else if self
            .trail
            .last()
            .map(|p| distance(*p, at) >= 2.0)
            .unwrap_or(true)
        {
            self.trail.push(at);
            if self.trail.len() > MOST_ROUTE_POINTS {
                self.trail.remove(0);
            }
        }

        self.watch_reactions(player);

        if player.movement.still_seconds >= 1.0 {
            // Cells by floor division, so -1 is in cell -1, not 0 (until now a home west
            // or north of the origin was centred up to five blocks off).
            let cell = [
                (player.pos[0].floor() as i32).div_euclid(CELL),
                (player.pos[1].floor() as i32).div_euclid(CELL),
                (player.pos[2].floor() as i32).div_euclid(CELL),
            ];
            match self.still_cells.iter_mut().find(|(c, _)| *c == cell) {
                Some((_, ticks)) => *ticks += 1,
                None => {
                    // The least of the others makes room, never the new one.
                    if self.still_cells.len() >= MOST_CELLS {
                        let least = self
                            .still_cells
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, (_, t))| *t)
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        self.still_cells.remove(least);
                    }
                    self.still_cells.push((cell, 1));
                }
            }

            // Without a bed, the home is where they stand still most.
            if self.home.as_ref().map(|h| h.bed.is_none()).unwrap_or(true) {
                if let Some((cell, _)) = self.still_cells.iter().max_by_key(|(_, t)| *t) {
                    let center = [
                        cell[0] * CELL + CELL / 2,
                        cell[1] * CELL,
                        cell[2] * CELL + CELL / 2,
                    ];
                    let home = self.home.get_or_insert_with(|| Home {
                        center,
                        floor_y: center[1],
                        ..Home::default()
                    });
                    home.center = center;
                    home.floor_y = center[1];
                }
            }
        }

        if !player.main_hand.id.is_empty() {
            match self
                .favourites
                .iter_mut()
                .find(|(id, _)| *id == player.main_hand.id)
            {
                Some((_, count)) => *count += 1,
                None => {
                    // The least held makes room: pushed past a full list and cut, a newcomer
                    // was cut at once, and the first eight things ever held stayed the
                    // favourites for good.
                    if self.favourites.len() >= MOST_FAVOURITES {
                        self.favourites.pop();
                    }
                    self.favourites.push((player.main_hand.id.clone(), 1));
                }
            }
            self.favourites
                .sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            self.favourites.truncate(MOST_FAVOURITES);
        }
    }

    /// A container the target placed: theirs from the start, opened or not.
    fn container_placed(&mut self, pos: BlockPos, kind: &str) {
        match self.containers.iter_mut().find(|c| c.pos == pos) {
            Some(container) => {
                container.placed = true;
                container.kind = kind.to_string();
            }
            None => {
                self.containers.push(Container {
                    pos,
                    kind: kind.to_string(),
                    placed: true,
                    ..Container::default()
                });
                self.drop_least_opened_container();
            }
        }
    }

    fn drop_least_opened_container(&mut self) {
        if self.containers.len() > MOST_CONTAINERS {
            let least = self
                .containers
                .iter()
                .enumerate()
                .min_by_key(|(_, c)| (c.theirs(), c.opens))
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.containers.remove(least);
        }
    }

    fn container_opened(&mut self, pos: BlockPos, kind: &str, contents: &[ItemStack], day: f32) {
        match self.containers.iter_mut().find(|c| c.pos == pos) {
            Some(container) => {
                if container.opens == 0 || day.floor() > container.last_opened_day.floor() {
                    container.days += 1;
                }
                container.opens += 1;
                container.last_opened_day = day;
                container.contents = contents.to_vec();
                container.kind = kind.to_string();
            }
            None => {
                self.containers.push(Container {
                    pos,
                    kind: kind.to_string(),
                    opens: 1,
                    last_opened_day: day,
                    contents: contents.to_vec(),
                    placed: false,
                    days: 1,
                    in_home: false,
                });
                self.drop_least_opened_container();
            }
        }
    }

    /// Records an event's outcome. One the target would notice (a bold row) that landed is a
    /// landmark too, kept for the whole stay. Every event that landed and does something to
    /// them (`Event::effect`) is watched for a day for how they took it (N2), several at once,
    /// the oldest closed when more than eight are open.
    pub fn record(&mut self, event: &str, pos: Option<BlockPos>, day: f32, ok: bool, reason: &str) {
        let done = Done {
            event: event.to_string(),
            pos,
            day,
            ok,
            reason: reason.to_string(),
        };
        let found = crate::events::find(event);
        if ok {
            self.chain_landed(event, day);
            if event == "diary_sold" {
                self.diary_offered = day.max(0.01);
            }
        }
        if ok && found.is_some_and(|e| !e.small) {
            push_capped(&mut self.landmarks, done.clone(), MOST_LANDMARKS);
        }
        let effect = found.map(|e| e.effect()).unwrap_or("");
        if ok && !effect.is_empty() {
            let open: Vec<usize> = self
                .reactions
                .iter()
                .enumerate()
                .filter(|(_, r)| r.kind.is_empty())
                .map(|(i, _)| i)
                .collect();
            if open.len() >= MOST_OPEN_REACTIONS {
                self.reactions[open[0]].kind = "ignored".to_string();
            }
            let anchor = pos.unwrap_or_else(|| block(self.last.pos));
            push_capped(
                &mut self.reactions,
                Reaction {
                    event: event.to_string(),
                    day,
                    pos,
                    kind: String::new(),
                    effect: effect.to_string(),
                    anchor,
                    turns_at: self.last.turns,
                    looks_at: self.last.look_backs,
                    nearest: distance(block(self.last.pos), anchor),
                    ..Reaction::default()
                },
                MOST_REACTIONS,
            );
        }
        push_capped(&mut self.done, done, MOST_DONE);
    }

    /// Each open reaction watched this tick: a look (three seconds facing the spot within 8
    /// blocks, standing) and a watch (a minute of it), a start (more turns or look-backs within
    /// ten seconds), running (in the first minute, 12 blocks farther from it than the nearest
    /// they came, at a run, having come within 24).
    fn watch_reactions(&mut self, player: &Player) {
        let today = self.today;
        let at = block(player.pos);
        let mut watched = false;
        for reaction in self
            .reactions
            .iter_mut()
            .filter(|r| today - r.day < REACTION_DAYS)
        {
            let since = today - reaction.day;
            if let Some(spot) = reaction.pos {
                if player.movement.still_seconds >= 1.0 && in_view_from(player, spot, 8.0) {
                    reaction.looked_ticks += 1;
                    if reaction.looked_ticks >= 60 {
                        reaction.looked = true;
                    }
                    if reaction.looked_ticks >= 20 * 60 && reaction.kind.is_empty() {
                        reaction.kind = "watched".to_string();
                        watched = true;
                    }
                } else {
                    reaction.looked_ticks = 0;
                }
            }
            if since < ticks(200)
                && (player.movement.turns > reaction.turns_at
                    || player.movement.look_backs > reaction.looks_at)
            {
                reaction.startled = true;
            }
            if since < ticks(1_200) {
                let d = distance(at, reaction.anchor);
                reaction.nearest = reaction.nearest.min(d);
                if reaction.nearest <= 24.0
                    && d >= reaction.nearest + 12.0
                    && player.movement.pace >= 4.0
                {
                    reaction.ran = true;
                }
            }
        }
        if watched {
            self.triggers.push("watched".to_string());
        }
    }

    /// A block placed or broken where an open `house` reaction changed one: fixed, and how
    /// many minutes it took them.
    fn react_to_fixing(&mut self, pos: BlockPos) {
        let today = self.today;
        for reaction in self.reactions.iter_mut().filter(|r| {
            r.effect == "house" && r.fixed_minutes.is_none() && today - r.day < REACTION_DAYS
        }) {
            let Some(spot) = reaction.pos else { continue };
            if (0..3).all(|i| (spot[i] - pos[i]).abs() <= 1) {
                reaction.fixed_minutes = Some(((today - reaction.day) * 20.0).round() as u32);
            }
        }
    }

    /// The clock, once a tick from the stay: real time and the time of day for the sessions,
    /// and the day that closes a reaction unanswered.
    pub fn clock(&mut self, now: &craftkit::world::Now, day: f32) {
        self.last_real_millis = now.real_millis;
        self.last_day_time = now.day_time;
        for reaction in self
            .reactions
            .iter_mut()
            .filter(|r| r.kind.is_empty() && day - r.day >= REACTION_DAYS)
        {
            reaction.kind = "ignored".to_string();
        }
    }

    /// The trail ends here: a long enough one becomes a route, or is counted into the route it
    /// repeats (starting and ending within 8 blocks of it).
    fn cut_trail(&mut self) {
        if self.trail.len() < LEAST_ROUTE_POINTS {
            self.trail.clear();
            return;
        }
        let trail = std::mem::take(&mut self.trail);
        let (start, end) = (trail[0], trail[trail.len() - 1]);
        let same = self.routes.iter_mut().find(|r| {
            (distance(r.start(), start) <= 8.0 && distance(r.end(), end) <= 8.0)
                || (distance(r.start(), end) <= 8.0 && distance(r.end(), start) <= 8.0)
        });
        match same {
            Some(route) => route.walked += 1,
            None => {
                if self.routes.len() >= MOST_ROUTES {
                    let least = self
                        .routes
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, r)| r.walked)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.routes.remove(least);
                }
                self.routes.push(Route {
                    points: trail,
                    walked: 1,
                    first_day: self.today,
                });
            }
        }
        self.routes.sort_by_key(|r| std::cmp::Reverse(r.walked));
    }

    /// A block placed answers the open reactions to things left in a place (a change to the
    /// house, words, their things, something seen): a light within 16 blocks lit the spot; a
    /// block that closes the spot in on its last open side walled it ([`encloses`]). Building
    /// near it is not walling it: the dev client's house went up around a sign and read as
    /// walled. A sound's or the sky's spot is no place to light.
    fn react_to_placing(&mut self, pos: BlockPos, block_id: &str) {
        let light = is_light(block_id);
        let mut answered: Vec<(usize, &'static str)> = Vec::new();
        for (i, reaction) in self.reactions.iter().enumerate() {
            if !reaction.kind.is_empty()
                || !matches!(
                    reaction.effect.as_str(),
                    "house" | "words" | "things" | "seen"
                )
            {
                continue;
            }
            let Some(spot) = reaction.pos else { continue };
            if light && distance(spot, pos) <= 16.0 {
                answered.push((i, "lit"));
            } else if !light && side_of(spot, pos).is_some() && encloses(spot, &self.placed) {
                answered.push((i, "walled"));
            }
        }
        let mut said: Vec<&'static str> = Vec::new();
        for (i, kind) in answered {
            self.reactions[i].kind = kind.to_string();
            if !said.contains(&kind) {
                said.push(kind);
                self.triggers.push(kind.to_string());
            }
        }
    }

    /// Makes a player the target.
    pub fn set_target(&mut self, index: u32, name: &str, uuid: &str) {
        self.target = Some(index);
        self.target_name = name.to_string();
        self.target_uuid = uuid.to_string();
    }

    fn trigger(&mut self, kind: &str) {
        if self.triggers.len() < 8 {
            self.triggers.push(kind.to_string());
        }
    }

    /// The route walked most, if any was walked twice; before that, the longest single walk of
    /// [`FIRST_ROUTE`] blocks or more, so a player's first day has one (the dev client's
    /// `behind_you` and `animal_gone` waited on "no route").
    pub fn route(&self) -> Option<&Route> {
        self.routes.iter().find(|r| r.walked >= 2).or_else(|| {
            self.routes
                .iter()
                .filter(|r| r.length() >= FIRST_ROUTE)
                .max_by(|a, b| a.length().total_cmp(&b.length()))
        })
    }

    /// The far end of the most-walked route: the end farther from the home.
    pub fn route_end(&self) -> Option<BlockPos> {
        let route = self.route()?;
        let center = self.home.as_ref().map(|h| h.center)?;

        Some(
            if distance(route.start(), center) > distance(route.end(), center) {
                route.start()
            } else {
                route.end()
            },
        )
    }

    /// The usual session, in minutes, from the closed sessions; none before two.
    pub fn usual_minutes(&self) -> Option<u32> {
        let closed: Vec<u64> = self
            .sessions
            .iter()
            .filter(|s| s.ended_real > s.began_real)
            .map(|s| (s.ended_real - s.began_real) / 60_000)
            .collect();
        if closed.len() < 2 {
            return None;
        }

        Some((closed.iter().sum::<u64>() / closed.len() as u64) as u32)
    }

    /// Whether they usually join after dark, from the sessions' game time.
    pub fn plays_at_night(&self) -> bool {
        let nights = self
            .sessions
            .iter()
            .filter(|s| s.began_day_time >= 13_000 && s.began_day_time <= 23_000)
            .count();

        self.sessions.len() >= 2 && nights * 2 > self.sessions.len()
    }

    fn milestone(&mut self, kind: &str, day: f32, pos: Option<BlockPos>, detail: &str) {
        push_capped(
            &mut self.milestones,
            Milestone {
                kind: kind.to_string(),
                day,
                pos,
                detail: detail.to_string(),
            },
            MOST_MILESTONES,
        );
    }

    /// How many times an event landed in this stay: from `done`, or from the landmarks when
    /// `done` has trimmed, whichever remembers more.
    pub fn times(&self, event: &str) -> usize {
        let recent = self
            .done
            .iter()
            .filter(|d| d.event == event && d.ok)
            .count();
        let kept = self.landmarks.iter().filter(|d| d.event == event).count();

        recent.max(kept)
    }

    /// The container the target opens most: a chest or barrel, never a furnace, which has its
    /// own events.
    pub fn used_container(&self) -> Option<&Container> {
        self.containers
            .iter()
            .filter(|c| !c.is_furnace() && c.theirs())
            .max_by_key(|c| c.opens)
    }

    /// The second chest the target opens, if there is one.
    pub fn other_container(&self) -> Option<&Container> {
        let used = self.used_container()?.pos;

        self.containers
            .iter()
            .filter(|c| !c.is_furnace() && c.theirs() && c.pos != used)
            .max_by_key(|c| c.opens)
    }

    /// The furnace the target used most.
    pub fn furnace(&self) -> Option<&Container> {
        self.containers
            .iter()
            .filter(|c| c.is_furnace() && c.theirs())
            .max_by_key(|c| c.opens)
    }

    /// The fixtures of a plain kind (`fence_gate`), oldest first.
    pub fn fixtures<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Fixture> + 'a {
        self.fixtures.iter().filter(move |f| f.kind == kind)
    }

    /// Whether any fixture of this kind is known.
    /// What the memory knows at a spot, in a word: `door`, `bed`, `chest`, `wall`, `light`,
    /// `fixture`, `decor`, or none. A plan keeps it, and is cancelled when it is gone.
    pub fn thing_at(&self, pos: BlockPos) -> Option<&'static str> {
        let home = self.home.as_ref();
        if home.is_some_and(|h| h.doors.contains(&pos)) {
            Some("door")
        } else if home.is_some_and(|h| h.bed == Some(pos)) {
            Some("bed")
        } else if self.containers.iter().any(|c| c.pos == pos && c.theirs()) {
            Some("chest")
        } else if self.walls.iter().any(|(p, _)| *p == pos) {
            Some("wall")
        } else if home.is_some_and(|h| h.lights.contains(&pos)) {
            Some("light")
        } else if self.fixtures.iter().any(|f| f.pos == pos) {
            Some("fixture")
        } else if self.decor.iter().any(|d| d.pos == pos) {
            Some("decor")
        } else {
            None
        }
    }

    /// Another horror mod loaded: the lodger acts half as often.
    pub fn sharing_with_horror(&self) -> bool {
        self.mods.iter().any(|(id, _)| is_horror_mod(id))
    }

    /// A light it placed, for the endings.
    pub fn its_light(&mut self, pos: BlockPos) {
        if !self.its_lights.contains(&pos) {
            push_capped(&mut self.its_lights, pos, MOST_ITS_LIGHTS);
        }
    }

    /// The block they build with most at home, by the walls it knows (`ender_holds`).
    pub fn block_built_most(&self) -> Option<String> {
        // Their building, at home: not a light (a lit mine made the torch their block), not a
        // block placed far off.
        let home = self.home.as_ref().map(|h| h.center);
        let mut counts: Vec<(&str, usize)> = Vec::new();
        for (_, id) in self.walls.iter().filter(|(p, id)| {
            !is_light(id) && home.is_none_or(|h| distance(h, *p) <= (HOME_RADIUS * 2) as f64)
        }) {
            match counts.iter_mut().find(|(i, _)| *i == id.as_str()) {
                Some((_, n)) => *n += 1,
                None => counts.push((id.as_str(), 1)),
            }
        }
        counts
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map(|(id, _)| id.to_string())
    }

    pub fn has_fixture(&self, kind: &str) -> bool {
        self.fixtures(kind).next().is_some()
    }

    /// The decor of a kind (`item_frame`, `painting`, `armor_stand`).
    pub fn decor<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Decor> + 'a {
        self.decor.iter().filter(move |d| d.kind == kind)
    }

    /// The creatures of a kind near the target (`sheep`, `villager`), or every one for "".
    pub fn creatures<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Creature> + 'a {
        self.last
            .creatures_near
            .iter()
            .filter(move |c| kind.is_empty() || c.kind == kind)
    }

    /// A feature of the surroundings by kind (`water`, `crops`, `snow`, `village`), if the
    /// pass found one.
    pub fn feature(&self, kind: &str) -> Option<&Feature> {
        self.surroundings
            .as_ref()?
            .features
            .iter()
            .find(|f| f.kind == kind)
    }

    /// Whether the surroundings are due: none yet, or a day old.
    pub fn surroundings_due(&self, day: f32) -> bool {
        self.surroundings
            .as_ref()
            .map(|s| day - s.day >= 1.0)
            .unwrap_or(true)
    }

    /// Where the scanned blocks of a kind are, the village's (`door`, `bell`) and the ore's
    /// (`_ore`), each from its own last scan.
    pub fn scanned<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = BlockPos> + 'a {
        self.scanned
            .iter()
            .chain(self.ores.iter())
            .filter(move |(k, _)| k.ends_with(kind))
            .map(|(_, p)| *p)
    }

    /// Whether the target is within `blocks` of `pos` and walking toward it.
    pub fn approaching(&self, pos: BlockPos, blocks: f64) -> bool {
        if !self.last.online || !self.last.walking {
            return false;
        }
        let at = block(self.last.pos);
        if distance(at, pos) > blocks || distance(at, pos) < 2.0 {
            return false;
        }

        self.in_view(pos, blocks)
    }

    /// The tally its kill sign counts: the kind it wrote, or the kind killed most before it
    /// kept which.
    pub fn sign_tally(&self) -> Option<&Tally> {
        if self.kill_sign_kind.is_empty() {
            return self.most_killed();
        }
        self.tally.iter().find(|t| t.kind == self.kill_sign_kind)
    }

    /// The kind the target has killed most, with its count, once they have killed three.
    pub fn most_killed(&self) -> Option<&Tally> {
        self.tally.first().filter(|t| t.kills >= 3)
    }

    /// The target's tamed animals near them.
    pub fn pets(&self) -> impl Iterator<Item = &Creature> {
        self.last.creatures_near.iter().filter(|c| c.pet)
    }

    /// A favourite tool with wear (a pickaxe, an axe, a sword, a shovel, a hoe) in any chest.
    pub fn tool_in_container(&self) -> Option<(&Container, &ItemStack)> {
        let is_tool = |id: &str| {
            [
                "_pickaxe", "_axe", "_sword", "_shovel", "_hoe", "_bow", "shears",
            ]
            .iter()
            .any(|t| id.ends_with(t))
        };
        for container in self.containers.iter().filter(|c| !c.is_furnace()) {
            if let Some(stack) = container.contents.iter().find(|s| is_tool(&s.id)) {
                return Some((container, stack));
            }
        }

        None
    }

    /// The coloured stacks (wool, concrete, glass, beds, carpets) in the used container.
    pub fn coloured_stacks(&self) -> usize {
        self.used_container()
            .map(|c| c.contents.iter().filter(|s| is_coloured(&s.id)).count())
            .unwrap_or(0)
    }

    /// A stack in the used container matching `test`, with the container.
    pub fn in_used_container(
        &self,
        test: impl Fn(&str) -> bool,
    ) -> Option<(&Container, &ItemStack)> {
        let container = self.used_container()?;
        let stack = container.contents.iter().find(|s| test(&s.id))?;

        Some((container, stack))
    }

    /// The favourite item that sits in a used container, with the container and the slot.
    pub fn favourite_in_container(&self) -> Option<(&Container, &ItemStack)> {
        for (favourite, _) in &self.favourites {
            // Theirs, and never a furnace.
            for container in self
                .containers
                .iter()
                .filter(|c| c.theirs() && !c.is_furnace())
            {
                if let Some(stack) = container.contents.iter().find(|s| s.id == *favourite) {
                    return Some((container, stack));
                }
            }
        }

        None
    }

    /// Whether the target is more than `blocks` from `pos`, or asleep, or offline.
    pub fn target_away_from(&self, pos: BlockPos, blocks: f64) -> bool {
        !self.last.online || self.last.sleeping || distance(block(self.last.pos), pos) > blocks
    }

    /// Whether `pos` is in front of the target, within `blocks`: the host cannot tell line of
    /// sight for the mind, so being within the angle counts as in view.
    pub fn in_view(&self, pos: BlockPos, blocks: f64) -> bool {
        if !self.last.online || self.last.sleeping {
            return false;
        }

        let at = block(self.last.pos);
        if distance(at, pos) > blocks {
            return false;
        }

        let dx = (pos[0] - at[0]) as f64;
        let dz = (pos[2] - at[2]) as f64;
        if dx == 0.0 && dz == 0.0 {
            return true;
        }

        // Yaw 0 faces +z, 90 faces -x, as the game has it.
        let facing_x = -(self.last.yaw as f64).to_radians().sin();
        let facing_z = (self.last.yaw as f64).to_radians().cos();
        let length = (dx * dx + dz * dz).sqrt();
        let cos = (dx * facing_x + dz * facing_z) / length;

        cos > 0.5 // within 60 degrees
    }

    /// The last events, newest first.
    pub fn recent(&self, count: usize) -> impl Iterator<Item = &Done> {
        self.done.iter().rev().take(count)
    }

    /// How they take each kind of event, from the settled reactions (a day passed, or they
    /// did something about it): the effect, how many they noticed, and how many settled. Only
    /// effects with three settled or more, most noticed first.
    pub fn attention(&self) -> Vec<(String, u32, u32)> {
        let mut by: Vec<(String, u32, u32)> = Vec::new();
        for reaction in &self.reactions {
            let settled = reaction.noticed() || self.today - reaction.day >= REACTION_DAYS;
            if !settled || reaction.effect.is_empty() {
                continue;
            }
            let noticed = u32::from(reaction.noticed());
            match by.iter_mut().find(|(e, _, _)| *e == reaction.effect) {
                Some((_, n, t)) => {
                    *n += noticed;
                    *t += 1;
                }
                None => by.push((reaction.effect.clone(), noticed, 1)),
            }
        }
        by.retain(|(_, _, t)| *t >= 3);
        by.sort_by(|a, b| (b.1 * a.2).cmp(&(a.1 * b.2)));

        by
    }

    /// How much an effect gets their notice, 0 to 1, if it has been seen settled three times.
    pub fn notice_rate(&self, effect: &str) -> Option<f64> {
        self.attention()
            .into_iter()
            .find(|(e, _, _)| e == effect)
            .map(|(_, n, t)| n as f64 / t as f64)
    }

    /// The reaction lines: what gets their attention and what does not, how fast they put
    /// things right, whether they run or fight.
    pub fn reaction_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let attention = self.attention();
        let rate = |(_, n, t): &(String, u32, u32)| *n as f64 / *t as f64;
        let best = attention.first().filter(|a| rate(a) >= 0.5);
        let worst = attention.last().filter(|a| rate(a) <= 0.25);
        let words = |effect: &str| match effect {
            "sounds" => "Sounds",
            "words" => "Words",
            "things" => "Their things moved",
            "house" => "Changes to the house",
            "seen" => "Things that show themselves",
            "sky" => "The sky",
            _ => "What happens at night",
        };
        match (best, worst) {
            (Some(b), Some(w)) if b.0 != w.0 => lines.push(format!(
                "{} get their attention; {} do not.",
                words(&b.0),
                words(&w.0).to_lowercase()
            )),
            (Some(b), _) => lines.push(format!("{} get their attention.", words(&b.0))),
            (None, Some(w)) => lines.push(format!("{} do not get their attention.", words(&w.0))),
            _ => {}
        }

        let mut fixed: Vec<u32> = self
            .reactions
            .iter()
            .filter_map(|r| r.fixed_minutes)
            .collect();
        if fixed.len() >= 2 {
            fixed.sort_unstable();
            lines.push(format!(
                "They put things right within about {} minutes.",
                fixed[fixed.len() / 2].max(1)
            ));
        }
        let ran = self.reactions.iter().filter(|r| r.ran).count();
        let fought = self.reactions.iter().filter(|r| r.fought).count();
        if ran >= 2 && ran >= fought {
            lines.push("When something shows itself, they run.".to_string());
        } else if fought >= 2 {
            lines.push("When something shows itself, they fight.".to_string());
        }

        lines
    }

    /// The lines that tell more than the core of `describe`, most telling first: the habits
    /// (N1). Each only once there is enough to say it; `describe` takes them while they fit.
    pub fn detail_lines(&self) -> Vec<String> {
        let mut lines = self.people_lines();
        lines.extend(self.reaction_lines());
        lines.extend(self.place_lines());
        lines.extend(self.habit_lines());

        lines
    }

    /// How long the core lines of `describe` are without the details, for the budget's test.
    #[cfg(test)]
    fn describe_core_len(&self, day: f32) -> usize {
        let all = self.describe(day);
        let details = self.detail_lines();
        all[..all.len() - 1]
            .iter()
            .filter(|l| !details.contains(l))
            .map(|l| l.len() + 1)
            .sum()
    }

    /// The lines the brain reads, within the default context's budget. A template, never
    /// prose: see `docs/memory.md`.
    pub fn describe(&self, day: f32) -> Vec<String> {
        self.describe_within(day, DESCRIBE_CHARS)
    }

    /// [`Memory::describe`] with the details taken while the lines stay under `chars`.
    pub fn describe_within(&self, day: f32, chars: usize) -> Vec<String> {
        let mut lines = Vec::new();

        if let Some(home) = &self.home {
            lines.push(format!(
                "The player sleeps in a house with {} door{} and {} light{}.",
                home.doors.len(),
                if home.doors.len() == 1 { "" } else { "s" },
                home.lights.len(),
                if home.lights.len() == 1 { "" } else { "s" }
            ));
        } else {
            lines.push("The player has no home yet that it knows of.".to_string());
        }

        if let Some((favourite, _)) = self.favourites.first() {
            lines.push(format!("They reach for {} most.", plain(favourite)));
        }

        if let Some(container) = self.used_container() {
            lines.push(format!(
                "The chest {} is opened most, {} times so far. It holds {}.",
                self.where_words(container.pos),
                container.opens,
                if container.contents.is_empty() {
                    "nothing".to_string()
                } else {
                    container
                        .contents
                        .iter()
                        .take(3)
                        .map(|s| plain(&s.id))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ));
        }

        for done in self.recent(3) {
            let when = ago_words(day - done.day, "Today");
            // Its own landmarks referred back to (N3): what it did there before.
            let again = done
                .pos
                .filter(|_| done.ok)
                .and_then(|p| self.landmark_near(p, done.day, &done.event))
                .map(|l| {
                    format!(
                        ", where it once did {}",
                        crate::events::find(&l.event)
                            .map(|e| e.menu_words())
                            .unwrap_or(&l.event)
                    )
                })
                .unwrap_or_default();
            lines.push(format!(
                "{} it {} {}{again}.",
                when,
                if done.ok { "did" } else { "could not" },
                crate::events::find(&done.event)
                    .map(|e| e.menu_words())
                    .unwrap_or(&done.event)
            ));
        }

        // The latest landmark not among the recent lines: what it did before those, that they
        // would remember.
        if let Some(landmark) = self
            .landmarks
            .iter()
            .rev()
            .find(|l| !self.recent(3).any(|d| d.day == l.day && d.event == l.event))
        {
            lines.push(format!(
                "{} it did {}{}.",
                ago_words(day - landmark.day, "Earlier today"),
                crate::events::find(&landmark.event)
                    .map(|e| e.menu_words())
                    .unwrap_or(&landmark.event),
                landmark
                    .pos
                    .zip(self.home.as_ref())
                    .map(|(p, _)| format!(", {}", self.where_words(p)))
                    .unwrap_or_default()
            ));
        }

        // The chains under way (`chains.rs`).
        lines.extend(self.chain_words(day));

        // Its temper, when far from resting, and why (`temper.rs`).
        if let Some(words) = self.temper.words(self.temper.resting) {
            lines.push(words);
        }

        // The last figure they saw, within three days: how they looked at it.
        if let Some(look) = self
            .figure_looks
            .last()
            .filter(|l| day - l.day <= 3.0 && l.looks > 0)
        {
            let seconds = (look.looked_ticks as f32 / 20.0).round().max(1.0) as u32;
            lines.push(format!(
                "{} they looked at the figure for {} second{}{}.",
                ago_words(day - look.day, "Today"),
                seconds,
                if seconds == 1 { "" } else { "s" },
                if look.chased() {
                    format!(" and went {} blocks toward it", look.toward.round() as u32)
                } else {
                    " and kept their distance".to_string()
                }
            ));
        }

        if let Some(milestone) = self.milestones.last() {
            let when = ago_words(day - milestone.day, "Today");
            let what = match milestone.kind.as_str() {
                "died" => "they died".to_string(),
                "first_sleep" => "they slept for the first time".to_string(),
                "first_kill" => format!(
                    "they killed their first {}",
                    milestone.detail.replace('_', " ")
                ),
                "crossed" => format!(
                    "they went to {}",
                    milestone
                        .detail
                        .rsplit(':')
                        .next()
                        .unwrap_or(&milestone.detail)
                        .replace('_', " ")
                ),
                other => other.to_string(),
            };
            lines.push(format!(
                "{when} {what}{}.",
                milestone
                    .pos
                    .zip(self.home.as_ref())
                    .filter(|_| milestone.kind == "died")
                    .map(|(p, _)| format!(", {}", self.where_words(p)))
                    .unwrap_or_default()
            ));
        }

        if let Some(route) = self.route() {
            if self.home.is_some() {
                let end = self.route_end().unwrap_or(route.end());
                lines.push(format!(
                    "They walk a way of {} blocks to {}, {} times so far.",
                    route.length().round() as i32,
                    self.where_words(end),
                    route.walked
                ));
            }
        }

        if let Some(minutes) = self.usual_minutes() {
            lines.push(format!(
                "They usually play for about {} minutes{}.",
                minutes.max(1),
                if self.plays_at_night() {
                    ", and join after dark"
                } else {
                    ""
                }
            ));
        }

        if let Some(reaction) = self.reactions.iter().rev().find(|r| !r.kind.is_empty()) {
            let what = crate::events::find(&reaction.event)
                .map(|e| e.menu_words())
                .unwrap_or(&reaction.event);
            let did = match reaction.kind.as_str() {
                "lit" => "they lit the place up",
                "walled" => "they walled the place in",
                "watched" => "they stood and watched the place",
                "left" => "they left the game",
                _ => "they did nothing about it",
            };
            lines.push(format!("After it did {what}, {did}."));
        }

        if let Some(around) = &self.surroundings {
            let mut parts = Vec::new();
            for kind in [
                "water",
                "village",
                "mineshaft",
                "crops",
                "snow",
                "sand",
                "trees",
            ] {
                if let Some(feature) = around.features.iter().find(|f| f.kind == kind) {
                    let words = match kind {
                        "water" => format!("water {} blocks across", feature.size),
                        "village" => "a village".to_string(),
                        "mineshaft" => "an old mine".to_string(),
                        "crops" => "a field".to_string(),
                        "snow" => "snow".to_string(),
                        "sand" => "sand".to_string(),
                        _ => "trees".to_string(),
                    };
                    if self.home.is_some() {
                        parts.push(format!("{} {}", words, self.where_words(feature.pos)));
                    } else {
                        parts.push(words);
                    }
                }
            }
            if !parts.is_empty() {
                parts.truncate(3);
                lines.push(format!("Around the house: {}.", parts.join("; ")));
            }
        }

        if !self.want.is_empty() {
            lines.push(format!("It wants {}.", self.want));
        }

        if !self.last.mood.is_empty() {
            lines.push(format!("The player seems {}.", self.last.mood));
        }

        if let Some(tally) = self.most_killed() {
            lines.push(format!(
                "They have killed {} {} so far.",
                tally.kills,
                crate::brain::text::plural(&tally.kind, tally.kills),
            ));
        }

        if let Some(held) = &self.held {
            lines.push(format!(
                "It holds the player's {}, taken {}.",
                bare(&held.item.id),
                ago_words(day - held.taken_day, "today").to_lowercase()
            ));
        }

        if let Some(line) = self.chat.last() {
            lines.push(format!("The player said: \"{}\".", line));
        }

        // The details last, while they fit: the situation, a question and its longest choice
        // must stay inside the model's context, and a prompt past it fails and falls to the
        // rules.
        let mut used: usize = lines.iter().map(|l| l.len() + 1).sum();
        for line in self.detail_lines() {
            if used + line.len() + 1 > chars {
                break;
            }
            used += line.len() + 1;
            lines.push(line);
        }

        lines.push(format!(
            "Now the player is {}{}{}.",
            if !self.last.online {
                "away from the game"
            } else if self.last.sleeping {
                "asleep"
            } else if self.last.indoors {
                "indoors"
            } else {
                "outdoors"
            },
            if self.last.online && self.last.walking {
                ", walking"
            } else {
                ""
            },
            if self.last.look_backs >= 2 {
                ", and keeps looking behind them"
            } else {
                ""
            }
        ));

        lines
    }
}

fn push_capped<T>(list: &mut Vec<T>, value: T, cap: usize) {
    list.push(value);
    if list.len() > cap {
        list.remove(0);
    }
}

/// Items named without `a`: what a player has some of, not one of (`dirt`, `bread`), and names
/// already plural (`oak planks`).
const MASS: &[&str] = &[
    "dirt",
    "coarse dirt",
    "rooted dirt",
    "mud",
    "cobblestone",
    "cobbled deepslate",
    "stone",
    "deepslate",
    "granite",
    "diorite",
    "andesite",
    "tuff",
    "calcite",
    "netherrack",
    "sand",
    "red sand",
    "gravel",
    "clay",
    "snow",
    "ice",
    "glass",
    "obsidian",
    "bread",
    "wheat",
    "sugar",
    "leaf litter",
    "moss",
    "kelp",
    "seagrass",
    "short grass",
    "coal",
    "charcoal",
    "raw iron",
    "raw gold",
    "raw copper",
    "redstone",
    "string",
    "flint",
    "gunpowder",
    "leather",
    "beef",
    "porkchop",
    "mutton",
    "chicken",
    "rabbit",
    "cod",
    "salmon",
    "rotten flesh",
    "bone meal",
    "glowstone dust",
    "lapis lazuli",
    "quartz",
    "paper",
    "wool",
    "white wool",
];

/// `minecraft:iron_pickaxe` as `iron pickaxe`.
pub fn bare(id: &str) -> String {
    id.rsplit(':').next().unwrap_or(id).replace('_', " ")
}

/// `minecraft:iron_pickaxe` as `an iron pickaxe`, `minecraft:dirt` as `dirt`.
pub fn plain(id: &str) -> String {
    let name = bare(id);
    if MASS.contains(&name.as_str()) || name.ends_with('s') {
        return name;
    }
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };

    format!("{article} {name}")
}

/// How long ago, in the situation's words: `today` (as given) under a day, `Yesterday`, then
/// `3 days ago`.
pub(crate) fn ago_words(ago: f32, today: &str) -> String {
    match ago.max(0.0).floor() as u32 {
        0 => today.to_string(),
        1 => "Yesterday".to_string(),
        n => format!("{n} days ago"),
    }
}

fn about_the_lodger(text: &str) -> bool {
    let lower = text.to_lowercase();
    [
        "door",
        "chest",
        "torch",
        "house",
        "figure",
        "someone",
        "something",
        "who",
        "what",
        "?",
    ]
    .iter()
    .any(|word| lower.contains(word))
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_home_west_and_north_of_the_origin_is_centred_where_they_stand() {
        use craftkit::world::{Movement, Observations, Player};
        for at in [[-11.5, 64.0, -11.5], [-2.5, 64.0, -2.5], [9.5, 64.0, 9.5]] {
            let mut memory = Memory {
                target: Some(0),
                ..Memory::default()
            };
            let still = Observations {
                players: vec![Player {
                    pos: at,
                    movement: Movement {
                        still_seconds: 5.0,
                        ..Movement::default()
                    },
                    ..Player::default()
                }],
                ..Observations::default()
            };
            memory.observe(&still, 0.1);
            let center = memory.home.as_ref().unwrap().center;
            assert!(
                (center[0] as f64 - at[0]).abs() <= 3.0 && (center[2] as f64 - at[2]).abs() <= 3.0,
                "{at:?} centred at {center:?}"
            );
        }
    }

    #[test]
    fn with_another_horror_mod_it_makes_room_and_holds_off_near_its_things() {
        use craftkit::world::{Entity, LoadedModInfo, Observations, Player};
        assert!(is_horror_mod("cave_dweller") && is_horror_mod("the_man_from_the_fog"));
        assert!(!is_horror_mod("sodium") && !is_horror_mod("create"));
        let mut memory = Memory {
            target: Some(0),
            ..Memory::default()
        };
        let mods = vec![LoadedModInfo {
            id: "cave_dweller".to_string(),
            version: "1".to_string(),
        }];
        let seen = |kind: &str, x: f64| Observations {
            players: vec![Player::default()],
            entities: vec![Entity {
                id: 5,
                kind: kind.to_string(),
                pos: [x, 0.0, 0.0],
                ..Entity::default()
            }],
            mods: mods.clone(),
            ..Observations::default()
        };
        memory.observe(&seen("cave_dweller:cave_dweller", 30.0), 0.1);
        assert!(memory.sharing_with_horror());
        assert!(memory.last.other_horror_near);
        memory.observe(&seen("cave_dweller:cave_dweller", 200.0), 0.2);
        assert!(!memory.last.other_horror_near, "far off");
        memory.observe(&seen("minecraft:zombie", 10.0), 0.3);
        assert!(!memory.last.other_horror_near, "a zombie is the game's");
    }

    #[test]
    fn another_mod_s_lamps_are_lights_and_its_chairs_furniture() {
        assert!(is_light("supplementaries:wall_lamp"));
        assert!(!is_light("minecraft:redstone_lamp"));
        assert_eq!(fixture_kind("mcwfurnitures:oak_chair"), Some("furniture"));
        assert_eq!(
            fixture_kind("minecraft:crafting_table"),
            Some("crafting_table")
        );
        assert!(HOME_KINDS.contains(&"*chest") && HOME_KINDS.contains(&"*barrel"));
    }

    #[test]
    fn the_other_mods_are_kept_as_the_host_tells_them() {
        use craftkit::world::{LoadedModInfo, Observations};
        let mut memory = Memory::default();
        memory.observe(
            &Observations {
                mods: vec![LoadedModInfo {
                    id: "sodium".to_string(),
                    version: "0.7".to_string(),
                }],
                ..Observations::default()
            },
            0.1,
        );
        assert_eq!(memory.mods, vec![("sodium".to_string(), "0.7".to_string())]);
        memory.observe(&Observations::default(), 0.2);
        assert_eq!(memory.mods.len(), 1, "told once, kept");
    }

    #[test]
    fn after_a_restart_the_target_is_found_by_their_uuid_not_their_old_index() {
        use craftkit::world::{Observations, Player};
        let player = |index: u32, name: &str, uuid: &str, x: f64| Player {
            index,
            name: name.to_string(),
            uuid: uuid.to_string(),
            pos: [x, 64.0, 0.0],
            ..Player::default()
        };
        let mut memory = Memory::default();
        memory.observe(
            &Observations {
                players: vec![player(0, "alice", "a-1", 1.0)],
                ..Observations::default()
            },
            0.1,
        );
        assert_eq!(memory.target, Some(0));
        // The server restarts; bob joins first and is index 0, alice is 1.
        memory.observe(
            &Observations {
                players: vec![
                    player(0, "bob", "b-2", 50.0),
                    player(1, "alice", "a-1", 7.0),
                ],
                ..Observations::default()
            },
            0.2,
        );
        assert_eq!(memory.target, Some(1));
        assert_eq!(memory.last.pos[0], 7.0, "alice observed, not bob");
        assert!(
            memory.others.iter().any(|(i, n, _)| *i == 0 && n == "bob"),
            "{:?}",
            memory.others
        );
        // Bob alone at index 0: not alice.
        memory.observe(
            &Observations {
                players: vec![player(0, "bob", "b-2", 50.0)],
                ..Observations::default()
            },
            0.3,
        );
        assert!(!memory.last.online);
    }
    use super::*;
    use craftkit::world::{Event, Movement};

    fn player(index: u32, pos: [f64; 3], still: f32) -> Player {
        Player {
            index,
            name: "razor".to_string(),
            pos,
            sky_light: 15,
            movement: Movement {
                still_seconds: still,
                ..Movement::default()
            },
            main_hand: ItemStack {
                id: "minecraft:iron_pickaxe".to_string(),
                count: 1,
                slot: 0,
            },
            ..Player::default()
        }
    }

    fn obs(players: Vec<Player>, events: Vec<Event>) -> Observations {
        Observations {
            players,
            events,
            ..Observations::default()
        }
    }

    #[test]
    fn the_first_player_seen_is_the_target_and_their_favourite_is_what_they_hold() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(3, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);

        assert_eq!(memory.target, Some(3));
        assert_eq!(memory.target_name, "razor");
        assert_eq!(memory.favourites[0].0, "minecraft:iron_pickaxe");
    }

    #[test]
    fn another_player_at_the_target_s_old_index_is_not_the_target() {
        let mut memory = Memory::default();
        memory.set_target(0, "razor", "uuid-r");
        let mut friend = player(0, [100.0, 64.0, 100.0], 0.0);
        friend.uuid = "uuid-f".to_string();
        friend.name = "friend".to_string();
        memory.observe(
            &obs(
                vec![friend],
                vec![Event {
                    player: 0,
                    kind: EventKind::SleptBegan,
                    pos: Some([100, 64, 100]),
                    ..Event::default()
                }],
            ),
            1.0,
        );

        assert!(
            memory.home.is_none(),
            "the friend's bed is not the target's home"
        );
        assert_eq!(
            memory.others_online,
            vec![0],
            "the friend is another player here"
        );
        assert_eq!(memory.target_seen_unix, 0, "the target was not seen");
    }

    #[test]
    fn a_figure_looked_at_is_kept_and_said_in_the_situation() {
        let mut memory = Memory::default();
        memory.observe(
            &obs(
                vec![player(0, [0.0, 64.0, 0.0], 0.0)],
                vec![Event {
                    player: 0,
                    kind: EventKind::FigureSeen,
                    pos: Some([20, 64, 0]),
                    detail: "looked=60 looks=2 longest=40 toward=11.5 end=approached".to_string(),
                    ..Event::default()
                }],
            ),
            2.0,
        );

        let look = memory.figure_looks.last().unwrap();
        assert_eq!(look.looked_ticks, 60);
        assert!(look.looked_long() && look.chased());
        assert!(
            memory.describe(2.5).iter().any(|l| l
                == "Today they looked at the figure for 3 seconds and went 12 blocks toward it."),
            "{:?}",
            memory.describe(2.5)
        );
        assert!(!FigureLook::parse("looked=0 looks=0 toward=0.0 end=timed", 1.0, None).chased());
    }

    #[test]
    fn a_thing_held_long_after_eight_others_becomes_a_favourite() {
        let mut memory = Memory::default();
        let holding = |id: &str| {
            let mut p = player(0, [0.0, 64.0, 0.0], 0.0);
            p.main_hand.id = id.to_string();
            obs(vec![p], vec![])
        };
        for i in 0..8 {
            for _ in 0..3 {
                memory.observe(&holding(&format!("minecraft:thing_{i}")), 0.0);
            }
        }
        for _ in 0..20 {
            memory.observe(&holding("minecraft:diamond_sword"), 0.0);
        }

        assert_eq!(memory.favourites.len(), 8);
        assert_eq!(memory.favourites[0].0, "minecraft:diamond_sword");
    }

    #[test]
    fn a_bed_slept_in_is_the_home_and_a_chest_placed_and_opened_is_the_used_container() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(0, [10.0, 64.0, 10.0], 0.0)], vec![]), 0.0);
        memory.observe(
            &obs(
                vec![player(0, [10.0, 64.0, 10.0], 0.0)],
                vec![
                    Event {
                        player: 0,
                        kind: EventKind::SleptBegan,
                        pos: Some([10, 64, 10]),
                        ..Event::default()
                    },
                    Event {
                        player: 0,
                        kind: EventKind::PlacedBlock,
                        pos: Some([12, 64, 10]),
                        detail: "minecraft:chest".to_string(),
                        ..Event::default()
                    },
                    Event {
                        player: 0,
                        kind: EventKind::OpenedContainer,
                        pos: Some([12, 64, 10]),
                        detail: "minecraft:chest".to_string(),
                        contents: vec![ItemStack {
                            id: "minecraft:iron_pickaxe".to_string(),
                            count: 1,
                            slot: 4,
                        }],
                    },
                ],
            ),
            0.5,
        );

        let home = memory.home.as_ref().unwrap();
        assert_eq!(home.bed, Some([10, 64, 10]));
        assert_eq!(memory.used_container().unwrap().pos, [12, 64, 10]);
        let (container, stack) = memory.favourite_in_container().unwrap();
        assert_eq!(container.pos, [12, 64, 10]);
        assert_eq!(stack.slot, 4);
    }

    fn at(kind: EventKind, pos: BlockPos, detail: &str, contents: Vec<ItemStack>) -> Event {
        Event {
            player: 0,
            kind,
            pos: Some(pos),
            detail: detail.to_string(),
            contents,
        }
    }

    fn bread() -> Vec<ItemStack> {
        vec![ItemStack {
            id: "minecraft:bread".to_string(),
            count: 3,
            slot: 0,
        }]
    }

    #[test]
    fn a_chest_opened_on_the_way_past_is_not_theirs_until_used_on_three_days() {
        // The dev client: a village chest opened once became "the chest opened most".
        let mut memory = Memory::default();
        let here = vec![player(0, [0.0, 64.0, 0.0], 0.0)];
        let open = |memory: &mut Memory, day: f32| {
            memory.observe(
                &obs(
                    here.clone(),
                    vec![at(
                        EventKind::OpenedContainer,
                        [60, 64, 0],
                        "minecraft:chest",
                        bread(),
                    )],
                ),
                day,
            )
        };
        open(&mut memory, 0.4);
        open(&mut memory, 0.6);
        assert!(
            memory.used_container().is_none(),
            "opened once, then again the same day"
        );
        assert_eq!(memory.containers[0].days, 1);

        open(&mut memory, 1.2);
        assert!(memory.used_container().is_none(), "two days");
        open(&mut memory, 2.1);
        assert_eq!(
            memory.used_container().map(|c| c.pos),
            Some([60, 64, 0]),
            "three days: a house moved into"
        );
    }

    #[test]
    fn a_chest_they_placed_is_theirs_before_it_is_opened_and_what_they_took_is_known_on_close() {
        let mut memory = Memory::default();
        let here = vec![player(0, [0.0, 64.0, 0.0], 0.0)];
        memory.observe(
            &obs(
                here.clone(),
                vec![at(
                    EventKind::PlacedBlock,
                    [2, 64, 0],
                    "minecraft:barrel",
                    vec![],
                )],
            ),
            0.1,
        );
        assert_eq!(memory.used_container().map(|c| c.pos), Some([2, 64, 0]));

        memory.observe(
            &obs(
                here.clone(),
                vec![at(
                    EventKind::OpenedContainer,
                    [2, 64, 0],
                    "minecraft:barrel",
                    bread(),
                )],
            ),
            0.2,
        );
        assert_eq!(memory.used_container().unwrap().contents.len(), 1);
        memory.observe(
            &obs(
                here,
                vec![at(
                    EventKind::ClosedContainer,
                    [2, 64, 0],
                    "minecraft:barrel",
                    vec![],
                )],
            ),
            0.2,
        );
        assert!(
            memory.used_container().unwrap().contents.is_empty(),
            "they took the bread"
        );
    }

    #[test]
    fn a_door_put_back_is_one_door_and_a_door_whose_floor_was_dug_is_forgotten() {
        // The dev client: two doors, dug under and put back, were four.
        let mut memory = Memory::default();
        let here = vec![player(0, [0.0, 64.0, 0.0], 0.0)];
        let place = |pos| at(EventKind::PlacedBlock, pos, "minecraft:oak_door", vec![]);
        memory.observe(
            &obs(here.clone(), vec![place([3, 64, 0]), place([-3, 64, 0])]),
            0.1,
        );
        memory.observe(&obs(here.clone(), vec![place([3, 64, 0])]), 0.2);
        assert_eq!(memory.home.as_ref().unwrap().doors.len(), 2);

        memory.observe(
            &obs(
                here,
                vec![at(
                    EventKind::BrokeBlock,
                    [-3, 63, 0],
                    "minecraft:dirt",
                    vec![],
                )],
            ),
            0.3,
        );
        assert_eq!(memory.home.as_ref().unwrap().doors, vec![[3, 64, 0]]);
    }

    #[test]
    fn a_house_built_around_a_sign_is_not_walling_it_but_closing_it_in_is() {
        // The dev client: a sign by the bed, the house built on, read as walled.
        let spot = [0, 65, 0];
        let building: Vec<BlockPos> = vec![
            [0, 64, 0],
            [1, 64, 0],
            [-1, 64, 1],
            [3, 65, 0],
            [2, 66, 2],
            [1, 65, 1],
        ];
        assert!(
            !encloses(spot, &building),
            "a floor, a far wall, a corner, a roof"
        );
        assert_eq!(side_of(spot, [1, 65, 1]), None, "a corner walls no side");
        assert_eq!(side_of(spot, [2, 65, 1]), Some(0));
        assert_eq!(side_of(spot, [0, 66, -2]), Some(3));

        let mut memory = Memory::default();
        memory.reactions.push(Reaction {
            event: "sign".to_string(),
            pos: Some(spot),
            effect: "words".to_string(),
            ..Reaction::default()
        });
        let here = vec![player(0, [20.0, 64.0, 0.0], 0.0)];
        let place = |pos| at(EventKind::PlacedBlock, pos, "minecraft:cobblestone", vec![]);
        let mut events: Vec<Event> = building.iter().map(|p| place(*p)).collect();
        events.extend([place([1, 65, 0]), place([-1, 65, 0]), place([0, 65, 1])]);
        memory.observe(&obs(here.clone(), events), 0.2);
        assert_eq!(memory.reactions[0].kind, "", "three sides closed: not yet");

        memory.observe(&obs(here, vec![place([0, 66, -1])]), 0.3);
        assert_eq!(memory.reactions[0].kind, "walled");
    }

    #[test]
    fn monsters_near_are_kept_apart_from_the_animals_within_forty_eight_blocks() {
        let mut memory = Memory::default();
        let entity = |id, kind: &str, x: f64, hostile| craftkit::world::Entity {
            id,
            kind: kind.to_string(),
            pos: [x, 64.0, 0.0],
            hostile,
            ..Default::default()
        };
        let mut seen = obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]);
        seen.entities = vec![
            entity(1, "minecraft:zombie", 10.0, true),
            entity(2, "minecraft:skeleton", 40.0, true),
            entity(3, "minecraft:creeper", 60.0, true),
            entity(4, "minecraft:cow", 5.0, false),
        ];
        memory.observe(&seen, 0.5);

        let kinds: Vec<&str> = memory
            .last
            .hostiles_near
            .iter()
            .map(|c| c.kind.as_str())
            .collect();
        assert_eq!(kinds, ["zombie", "skeleton"], "the creeper is past 48");
        assert_eq!(memory.creatures("").count(), 1, "only the cow is an animal");
    }

    #[test]
    fn the_house_they_sleep_in_is_the_home_s_whoever_built_it() {
        // The dev client: a village house read as "0 doors and 0 lights", its
        // torches and chest untouchable.
        let mut memory = Memory::default();
        let here = vec![player(0, [0.0, 64.0, 0.0], 0.0)];
        memory.observe(
            &obs(
                here.clone(),
                vec![Event {
                    player: 0,
                    kind: EventKind::SleptBegan,
                    pos: Some([0, 64, 0]),
                    ..Event::default()
                }],
            ),
            0.1,
        );
        let feature = |kind: &str, pos: BlockPos, detail: &str| craftkit::world::Feature {
            kind: kind.to_string(),
            pos,
            size: 1,
            detail: detail.to_string(),
        };
        let mut seen = obs(here, vec![]);
        seen.surroundings = Some(craftkit::world::Surroundings {
            id: 1,
            around: [0, 64, 0],
            features: vec![
                feature("water", [30, 62, 0], ""),
                feature("house_door", [3, 64, 0], "minecraft:oak_door"),
                feature("house_light", [1, 66, 2], "minecraft:wall_torch"),
                feature("house_light", [-2, 66, 2], "minecraft:torch"),
                feature("house_chest", [-1, 64, -2], "minecraft:chest"),
                feature("house_bed", [0, 64, 0], "minecraft:red_bed"),
            ],
        });
        memory.observe(&seen, 0.2);

        let home = memory.home.as_ref().unwrap();
        assert_eq!(home.doors, vec![[3, 64, 0]]);
        assert_eq!(home.lights.len(), 2);
        assert_eq!(
            memory.used_container().map(|c| c.pos),
            Some([-1, 64, -2]),
            "the house's chest is the home's, never opened"
        );
        let lines = memory.describe(0.2);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("1 door") && l.contains("2 lights")),
            "{lines:?}"
        );
        assert!(
            !lines.iter().any(|l| l.contains("house_")),
            "the house is not its surroundings: {lines:?}"
        );
        assert_eq!(memory.surroundings.as_ref().unwrap().features.len(), 1);
    }

    #[test]
    fn a_long_first_walk_is_the_route_until_one_is_walked_twice() {
        let walk = |points: Vec<BlockPos>, walked| Route {
            points,
            walked,
            ..Route::default()
        };
        let mut memory = Memory::default();
        memory.routes.push(walk(vec![[0, 64, 0], [10, 64, 0]], 1));
        assert!(memory.route().is_none(), "ten blocks is a stroll");

        memory
            .routes
            .push(walk(vec![[0, 64, 0], [20, 64, 0], [40, 64, 0]], 1));
        assert_eq!(
            memory.route().map(|r| r.length()),
            Some(40.0),
            "a first day's route"
        );

        memory.routes.push(walk(vec![[0, 64, 0], [0, 64, 12]], 2));
        assert_eq!(
            memory.route().map(|r| r.walked),
            Some(2),
            "one walked twice wins, however short"
        );
    }

    #[test]
    fn a_home_read_is_the_truth_inside_its_box_and_leaves_the_rest_alone() {
        let mut memory = Memory {
            home: Some(Home {
                center: [0, 64, 0],
                bed: Some([1, 64, 1]),
                doors: vec![[3, 64, 0], [-3, 64, 0], [40, 64, 0]],
                lights: vec![[2, 66, 2]],
                ..Home::default()
            }),
            ..Memory::default()
        };
        memory.fixtures.push(Fixture {
            kind: "lever".to_string(),
            pos: [1, 65, -2],
        });
        memory.fixtures.push(Fixture {
            kind: "carpet".to_string(),
            pos: [0, 64, 3],
        });
        memory.containers.push(Container {
            pos: [-2, 64, 2],
            kind: "minecraft:chest".to_string(),
            placed: true,
            ..Container::default()
        });
        let found = |k: &str, p: BlockPos| (k.to_string(), p);
        memory.read_home(
            &[
                found("minecraft:oak_door", [3, 64, 0]),
                found("minecraft:oak_door", [3, 65, 0]),
                found("minecraft:wall_torch", [-1, 66, 3]),
                found("minecraft:glass_pane", [4, 65, 1]),
                found("minecraft:barrel", [2, 64, -3]),
                found("minecraft:red_bed", [-4, 64, 4]),
            ],
            [-8, 61, -8],
            [8, 69, 8],
        );

        let home = memory.home.as_ref().unwrap();
        assert_eq!(
            home.doors,
            vec![[40, 64, 0], [3, 64, 0]],
            "one door gone, the far one kept, a door is one"
        );
        assert_eq!(home.lights, vec![[-1, 66, 3]], "the old torch is gone");
        assert_eq!(home.bed, Some([-4, 64, 4]), "the bed moved");
        assert!(
            memory.containers.iter().all(|c| c.pos != [-2, 64, 2]),
            "the chest is gone"
        );
        assert!(memory
            .containers
            .iter()
            .any(|c| c.pos == [2, 64, -3] && c.theirs()));
        assert!(
            memory.has_fixture("lever"),
            "a read that looks for no levers keeps them"
        );
        assert!(
            !memory.has_fixture("carpet"),
            "a carpet it looked for and did not find is gone"
        );
        assert!(memory.has_fixture("glass_pane"));
    }

    #[test]
    fn an_ore_scan_leaves_the_village_s_doors_and_bell_alone() {
        let mut memory = Memory::default();
        let answer = |id: u64, found: &[(&str, BlockPos)]| {
            let mut seen = obs(vec![], vec![]);
            seen.scans = vec![craftkit::world::Scan {
                id,
                counts: vec![],
                positions: found.iter().map(|(k, p)| (k.to_string(), *p)).collect(),
            }];
            seen
        };
        memory.scans_out.push((5, "scan_village".to_string()));
        memory.observe(
            &answer(
                5,
                &[
                    ("minecraft:oak_door", [60, 64, 60]),
                    ("minecraft:bell", [61, 65, 60]),
                ],
            ),
            0.5,
        );
        memory.scans_out.push((6, "scan_ore".to_string()));
        memory.observe(&answer(6, &[("minecraft:iron_ore", [20, 40, 20])]), 0.6);

        assert_eq!(memory.scanned("bell").count(), 1, "the village kept");
        assert_eq!(memory.scanned("_ore").count(), 1);
        assert!(memory.scans_out.is_empty());
    }

    #[test]
    fn natural_ground_they_dig_by_the_house_is_a_hole_and_what_they_placed_is_not() {
        let mut memory = Memory {
            home: Some(Home {
                center: [0, 64, 0],
                ..Home::default()
            }),
            ..Memory::default()
        };
        let here = vec![player(0, [0.0, 64.0, 0.0], 0.0)];
        let broke = |pos, id: &str| at(EventKind::BrokeBlock, pos, id, vec![]);
        memory.observe(
            &obs(
                here.clone(),
                vec![at(
                    EventKind::PlacedBlock,
                    [2, 64, 2],
                    "minecraft:dirt",
                    vec![],
                )],
            ),
            0.1,
        );
        memory.observe(
            &obs(
                here,
                vec![
                    broke([3, 63, 0], "minecraft:grass_block"),
                    broke([2, 64, 2], "minecraft:dirt"),
                    broke([60, 63, 0], "minecraft:stone"),
                    broke([1, 64, 1], "minecraft:oak_planks"),
                ],
            ),
            0.2,
        );
        assert_eq!(
            memory.holes,
            vec![([3, 63, 0], "minecraft:grass_block".to_string())],
            "theirs, natural, near: one"
        );
    }

    #[test]
    fn a_bed_placed_is_the_home_before_anyone_sleeps_in_it() {
        let mut memory = Memory::default();
        memory.observe(
            &obs(
                vec![player(0, [5.0, 64.0, 5.0], 0.0)],
                vec![Event {
                    player: 0,
                    kind: EventKind::PlacedBlock,
                    pos: Some([7, 64, 5]),
                    detail: "minecraft:red_bed".to_string(),
                    ..Event::default()
                }],
            ),
            0.0,
        );

        let home = memory.home.as_ref().unwrap();
        assert_eq!(home.bed, Some([7, 64, 5]));
        assert_eq!(home.center, [7, 64, 5]);
    }

    #[test]
    fn standing_still_makes_a_home_without_a_bed_and_the_cells_are_capped() {
        let mut memory = Memory::default();
        for _ in 0..30 {
            memory.observe(
                &obs(vec![player(0, [100.0, 64.0, 100.0], 2.0)], vec![]),
                0.0,
            );
        }
        assert_eq!(memory.home.as_ref().unwrap().center, [102, 64, 102]);

        for i in 0..700 {
            memory.observe(
                &obs(vec![player(0, [i as f64 * 8.0, 64.0, 0.0], 2.0)], vec![]),
                0.0,
            );
        }
        assert!(memory.still_cells.len() <= MOST_CELLS);
        assert_eq!(
            memory.home.as_ref().unwrap().center,
            [102, 64, 102],
            "the densest cell stays the home"
        );
    }

    #[test]
    fn in_view_is_in_front_and_near_and_away_is_far_or_asleep() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
        // Yaw 0 faces +z.
        assert!(memory.in_view([0, 64, 10], 24.0));
        assert!(!memory.in_view([0, 64, -10], 24.0), "behind");
        assert!(!memory.in_view([0, 64, 40], 24.0), "too far");
        assert!(!memory.target_away_from([0, 64, 5], 24.0));
        assert!(memory.target_away_from([0, 64, 30], 24.0));

        let mut asleep = player(0, [0.0, 64.0, 0.0], 0.0);
        asleep.sleeping = true;
        memory.observe(&obs(vec![asleep], vec![]), 0.0);
        assert!(memory.target_away_from([0, 64, 5], 24.0));
        assert!(!memory.in_view([0, 64, 5], 24.0));
    }

    #[test]
    fn describe_is_a_handful_of_plain_lines() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
        memory.record("open_door", Some([1, 64, 1]), 0.2, true, "");
        let lines = memory.describe(1.5);

        assert!(
            lines.iter().any(|l| l.contains("iron pickaxe")),
            "{lines:?}"
        );
        assert!(
            lines.iter().any(|l| l == "Yesterday it did open the door."),
            "{lines:?}"
        );
        assert!(
            lines
                .last()
                .unwrap()
                .starts_with("Now the player is outdoors"),
            "{lines:?}"
        );
        assert!(lines.len() <= 12);
    }

    #[test]
    fn done_is_capped_and_plain_names_read_well() {
        let mut memory = Memory::default();
        for i in 0..300 {
            memory.record("wait", None, i as f32, true, "");
        }
        assert_eq!(memory.done.len(), MOST_DONE);
        assert_eq!(plain("minecraft:iron_pickaxe"), "an iron pickaxe");
        assert_eq!(plain("minecraft:torch"), "a torch");
        // The dev client: "a dirt", "a leaf litter", "6 sheeps".
        assert_eq!(plain("minecraft:dirt"), "dirt");
        assert_eq!(plain("minecraft:leaf_litter"), "leaf litter");
        assert_eq!(plain("minecraft:oak_planks"), "oak planks");
        assert_eq!(plain("minecraft:apple"), "an apple");
        assert_eq!(ago_words(0.4, "Earlier today"), "Earlier today");
        assert_eq!(ago_words(1.2, "Today"), "Yesterday");
        assert_eq!(ago_words(3.9, "Today"), "3 days ago");
    }

    #[test]
    fn a_walk_repeated_becomes_a_route_and_the_lodger_knows_its_far_end() {
        let mut memory = Memory {
            home: Some(Home {
                bed: Some([0, 64, 0]),
                center: [0, 64, 0],
                floor_y: 64,
                doors: vec![],
                lights: vec![],
            }),
            ..Memory::default()
        };
        // Twice: out 40 blocks east, a point every two, then a minute's stop.
        for _ in 0..2 {
            for i in 0..21 {
                memory.observe(
                    &obs(vec![player(0, [i as f64 * 2.0, 64.0, 0.0], 0.0)], vec![]),
                    1.0,
                );
            }
            memory.observe(&obs(vec![player(0, [40.0, 64.0, 0.0], 61.0)], vec![]), 1.0);
            for i in (0..21).rev() {
                memory.observe(
                    &obs(vec![player(0, [i as f64 * 2.0, 64.0, 0.0], 0.0)], vec![]),
                    1.0,
                );
            }
            memory.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 61.0)], vec![]), 1.0);
        }

        assert!(!memory.routes.is_empty());
        let route = memory.route().expect("a route walked twice");
        assert!(route.walked >= 2, "{}", route.walked);
        assert_eq!(memory.route_end(), Some([40, 64, 0]));
        assert!(
            memory
                .describe(2.0)
                .iter()
                .any(|l| l.starts_with("They walk a way of")),
            "{:?}",
            memory.describe(2.0)
        );

        let short = {
            let mut m = Memory::default();
            m.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
            m.observe(&obs(vec![player(0, [4.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
            m.observe(&obs(vec![player(0, [4.0, 64.0, 0.0], 61.0)], vec![]), 0.0);
            m
        };
        assert!(short.routes.is_empty(), "a few blocks is no route");
    }

    #[test]
    fn sessions_give_the_usual_length_and_a_reaction_is_read_off_what_they_do_next() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
        let joined = |m: &mut Memory, real: u64, time: u32| {
            m.clock(
                &craftkit::world::Now {
                    tick: 0,
                    day: 0,
                    day_time: time,
                    real_millis: real,
                },
                0.0,
            );
            m.observe(
                &obs(
                    vec![player(0, [0.0, 64.0, 0.0], 0.0)],
                    vec![Event {
                        player: 0,
                        kind: EventKind::Joined,
                        ..Event::default()
                    }],
                ),
                0.0,
            );
        };
        let left = |m: &mut Memory, real: u64| {
            m.clock(
                &craftkit::world::Now {
                    tick: 0,
                    day: 0,
                    day_time: 0,
                    real_millis: real,
                },
                0.0,
            );
            m.observe(
                &obs(
                    vec![],
                    vec![Event {
                        player: 0,
                        kind: EventKind::Left,
                        ..Event::default()
                    }],
                ),
                0.0,
            );
        };
        joined(&mut memory, 0, 14_000);
        left(&mut memory, 40 * 60_000);
        joined(&mut memory, 100 * 60_000, 15_000);
        left(&mut memory, 160 * 60_000);
        assert_eq!(memory.usual_minutes(), Some(50));
        assert!(memory.plays_at_night());
        assert!(memory
            .describe(1.0)
            .iter()
            .any(|l| l.contains("about 50 minutes") && l.contains("after dark")));

        // A bold event lands; a torch placed nearby within the day is "lit".
        memory.record("take", Some([10, 64, 10]), 1.0, true, "");
        assert_eq!(memory.reactions.len(), 1);
        assert!(memory.reactions[0].kind.is_empty());
        memory.observe(
            &obs(
                vec![player(0, [12.0, 64.0, 10.0], 0.0)],
                vec![Event {
                    player: 0,
                    kind: EventKind::PlacedBlock,
                    pos: Some([12, 65, 10]),
                    detail: "minecraft:torch".to_string(),
                    ..Event::default()
                }],
            ),
            1.1,
        );
        assert_eq!(memory.reactions[0].kind, "lit");
        assert!(memory
            .describe(1.2)
            .iter()
            .any(|l| l.contains("they lit the place up")));

        // The next lands and a day passes with nothing: ignored.
        memory.record("grave", Some([20, 64, 20]), 2.0, true, "");
        memory.clock(
            &craftkit::world::Now {
                tick: 0,
                day: 3,
                day_time: 0,
                real_millis: 0,
            },
            3.1,
        );
        assert_eq!(memory.reactions[1].kind, "ignored");
    }

    #[test]
    fn bold_events_and_the_target_s_milestones_are_never_forgotten() {
        let mut memory = Memory::default();
        memory.record("take", Some([2, 64, 0]), 1.0, true, "");
        memory.record("knock", None, 1.1, true, "");
        for i in 0..300 {
            memory.record("wait", None, 2.0 + i as f32, true, "");
        }

        assert!(
            memory.done.iter().all(|d| d.event != "take"),
            "trimmed from done"
        );
        assert_eq!(memory.landmarks.len(), 1, "a small knock is no landmark");
        assert_eq!(memory.landmarks[0].event, "take");
        assert_eq!(memory.times("take"), 1);

        memory.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
        memory.observe(
            &obs(
                vec![player(0, [0.0, 64.0, 0.0], 0.0)],
                vec![
                    Event {
                        player: 0,
                        kind: EventKind::Killed,
                        pos: Some([5, 64, 5]),
                        detail: "minecraft:cow".to_string(),
                        ..Event::default()
                    },
                    Event {
                        player: 0,
                        kind: EventKind::Killed,
                        pos: Some([6, 64, 5]),
                        detail: "minecraft:cow".to_string(),
                        ..Event::default()
                    },
                    Event {
                        player: 0,
                        kind: EventKind::Died,
                        pos: Some([40, 60, 0]),
                        ..Event::default()
                    },
                ],
            ),
            3.0,
        );
        let kinds: Vec<&str> = memory.milestones.iter().map(|m| m.kind.as_str()).collect();
        assert_eq!(kinds, ["first_kill", "died"], "one first kill per kind");
        assert_eq!(memory.tally[0].kills, 2);

        let lines = memory.describe(5.0);
        assert!(lines.iter().any(|l| l.contains("they died")), "{lines:?}");
        assert!(
            lines.iter().any(|l| l.contains("take one thing they use")),
            "{lines:?}"
        );
    }

    #[test]
    fn the_habits_are_fed_by_what_the_target_does() {
        let mut memory = Memory::default();
        let event = |kind: EventKind, pos: BlockPos, detail: &str| Event {
            player: 0,
            kind,
            pos: Some(pos),
            detail: detail.to_string(),
            ..Event::default()
        };
        memory.observe(
            &obs(
                vec![player(0, [0.0, 64.0, 0.0], 0.0)],
                vec![
                    event(EventKind::PlacedBlock, [0, 64, 0], "minecraft:red_bed"),
                    event(EventKind::PlacedBlock, [3, 64, 0], "minecraft:oak_door"),
                    event(EventKind::PlacedBlock, [1, 65, 1], "minecraft:torch"),
                    event(EventKind::UsedDoor, [3, 65, 0], ""),
                ],
            ),
            0.1,
        );
        memory.observe(
            &obs(
                vec![player(0, [0.0, 64.0, 0.0], 0.0)],
                vec![event(EventKind::SleptBegan, [0, 64, 0], "")],
            ),
            0.53,
        );

        assert_eq!(memory.habits.lights_placed, 1);
        assert_eq!(memory.habits.doors, vec![([3, 64, 0], 1)]);
        assert_eq!(memory.habits.nights_slept, 1);
        assert_eq!(memory.habits.samples, 2, "a sample each observation here");
        memory.observe(&obs(vec![], vec![]), 0.6);
        assert_eq!(memory.habits.at_home, None, "away from the game");
    }

    #[test]
    fn a_full_memory_keeps_to_its_budget_and_ends_on_now() {
        let mut memory = Memory {
            target: Some(0),
            want: "a place of its own".to_string(),
            ..Memory::default()
        };
        memory.home = Some(Home {
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0], [-3, 64, 0]],
            ..Home::default()
        });
        memory.last.mood = "uneasy".to_string();
        memory
            .chat
            .push("who keeps opening my door at night and why".to_string());
        for i in 0..5 {
            memory.record("take", Some([2, 64, i]), 1.0 + i as f32 * 0.1, true, "");
        }
        memory.habits.hours = [
            [40, 30, 0, 0, 0],
            [0, 0, 40, 0, 0],
            [0, 0, 0, 40, 0],
            [40, 0, 0, 0, 0],
        ];
        memory.habits.doors = vec![([3, 64, 0], 9)];
        memory.habits.leaves = vec![2_000; 5];
        memory.habits.returns = vec![12_000; 5];
        memory.habits.nights_slept = 5;
        memory.habits.sleep_times = vec![12_600; 5];
        memory.habits.days_seen = 5;
        memory.habits.carried = vec![
            ("minecraft:iron_sword".to_string(), 5),
            ("minecraft:shield".to_string(), 5),
            ("minecraft:iron_helmet".to_string(), 5),
        ];
        memory.habits.lights_placed = 40;
        memory.habits.samples = 100;
        memory.habits.uneasy = 60;

        let budget = describe_chars(512);
        let lines = memory.describe_within(1.6, budget);
        let last = lines.last().unwrap();
        assert!(last.starts_with("Now the player is"), "{lines:?}");
        let before: usize = lines[..lines.len() - 1].iter().map(|l| l.len() + 1).sum();
        assert!(
            before <= budget.max(memory.describe_core_len(1.6)),
            "{before} chars: {lines:?}"
        );
        assert!(
            lines.iter().any(|l| l.contains("door on the east side")),
            "the first detail fits: {lines:?}"
        );
        assert!(
            lines.iter().any(|l| l == "They often look behind them."),
            "512 tokens take every detail: {lines:?}"
        );
        let core = memory.describe_core_len(1.6);
        let tight = memory.describe_within(1.6, core + 60);
        assert!(tight.len() < lines.len(), "a tight budget drops details");
        assert!(tight.last().unwrap().starts_with("Now the player is"));
        let used: usize = tight[..tight.len() - 1].iter().map(|l| l.len() + 1).sum();
        assert!(used <= core + 60, "{used} > {}", core + 60);
        assert_eq!(describe_chars(1536), DESCRIBE_CHARS);
    }

    fn mover(pos: [f64; 3], pace: f32, turns: u32) -> Player {
        let mut p = player(0, pos, 0.0);
        p.movement.pace = pace;
        p.movement.turns = turns;
        p
    }

    #[test]
    fn every_event_that_lands_is_watched_and_several_at_once() {
        let mut memory = Memory {
            target: Some(0),
            ..Memory::default()
        };
        memory.record("knock", Some([0, 64, 0]), 1.0, true, "");
        memory.record("sign", Some([5, 64, 0]), 1.0, true, "");
        memory.record("wait", None, 1.0, true, "");
        memory.record("cairn", Some([9, 64, 0]), 1.0, false, "no room");

        let kinds: Vec<(&str, &str)> = memory
            .reactions
            .iter()
            .map(|r| (r.event.as_str(), r.effect.as_str()))
            .collect();
        assert_eq!(
            kinds,
            [("knock", "sounds"), ("sign", "words")],
            "a small one too; not wait, not a refusal"
        );
        assert!(
            memory.reactions.iter().all(|r| r.kind.is_empty()),
            "both open"
        );
        assert_eq!(memory.landmarks.len(), 1, "landmarks stay the bold ones");

        for i in 0..MOST_OPEN_REACTIONS {
            memory.record("knock", None, 1.1 + i as f32 * 0.01, true, "");
        }
        assert_eq!(
            memory.reactions[0].kind, "ignored",
            "the oldest closed past eight open"
        );
        let open = memory
            .reactions
            .iter()
            .filter(|r| r.kind.is_empty())
            .count();
        assert_eq!(open, MOST_OPEN_REACTIONS);
    }

    #[test]
    fn a_start_a_look_a_fix_a_run_and_a_fight_are_each_seen() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![mover([0.0, 64.0, 0.0], 0.0, 1)], vec![]), 1.0);

        // A knock: more turns within ten seconds is a start.
        memory.record("knock", None, 1.0, true, "");
        memory.observe(
            &obs(vec![mover([0.0, 64.0, 0.0], 0.0, 3)], vec![]),
            1.0 + ticks(100),
        );
        assert!(memory.reactions[0].startled);
        assert!(memory.reactions[0].noticed());

        // A hole: a block placed in it a quarter of a day on (a day is twenty minutes) is
        // fixed after five minutes.
        memory.record("hole", Some([3, 63, 3]), 1.1, true, "");
        memory.observe(
            &obs(
                vec![mover([0.0, 64.0, 0.0], 0.0, 3)],
                vec![at(
                    EventKind::PlacedBlock,
                    [3, 63, 3],
                    "minecraft:dirt",
                    vec![],
                )],
            ),
            1.35,
        );
        assert_eq!(memory.reactions[1].fixed_minutes, Some(5));

        // A figure: they come within 10 blocks, then sprint 20 farther off in the minute, and
        // hit something.
        memory.record("far_figure", Some([10, 64, 0]), 2.0, true, "");
        memory.observe(
            &obs(vec![mover([2.0, 64.0, 0.0], 1.0, 3)], vec![]),
            2.0 + ticks(20),
        );
        memory.observe(
            &obs(
                vec![mover([-20.0, 64.0, 0.0], 6.0, 3)],
                vec![at(EventKind::Hit, [-20, 64, 0], "minecraft:zombie", vec![])],
            ),
            2.0 + ticks(400),
        );
        let figure = &memory.reactions[2];
        assert!(figure.ran, "{figure:?}");
        assert!(figure.fought);

        // A sign nobody looks at is not noticed until a day has passed, then settled unnoticed.
        memory.record("sign", Some([40, 64, 40]), 3.0, true, "");
        assert!(!memory.reactions[3].noticed());
    }

    #[test]
    fn a_light_near_a_sound_is_no_answer_to_it_but_near_a_sign_it_is() {
        let mut memory = Memory::default();
        memory.record("knock", Some([0, 64, 0]), 1.0, true, "");
        memory.record("sign", Some([2, 64, 0]), 1.0, true, "");
        memory.observe(
            &obs(
                vec![player(0, [0.0, 64.0, 0.0], 0.0)],
                vec![at(
                    EventKind::PlacedBlock,
                    [1, 65, 0],
                    "minecraft:torch",
                    vec![],
                )],
            ),
            1.1,
        );

        assert_eq!(memory.reactions[0].kind, "");
        assert_eq!(memory.reactions[1].kind, "lit");
        assert_eq!(memory.triggers, vec!["lit".to_string()], "said once");
    }

    #[test]
    fn opening_the_chest_it_moved_things_in_is_coming_to_look() {
        let mut memory = Memory::default();
        memory.record("shuffle_chest", Some([2, 64, 0]), 1.0, true, "");
        memory.observe(
            &obs(
                vec![player(0, [0.0, 64.0, 0.0], 0.0)],
                vec![at(
                    EventKind::OpenedContainer,
                    [2, 64, 0],
                    "minecraft:chest",
                    bread(),
                )],
            ),
            1.2,
        );

        assert!(memory.reactions[0].looked);
    }

    #[test]
    fn what_gets_their_attention_and_what_does_not_is_said_and_leans_the_rules() {
        let mut memory = Memory {
            today: 5.0,
            ..Memory::default()
        };
        let reaction = |event: &str, effect: &str, startled: bool| Reaction {
            event: event.to_string(),
            effect: effect.to_string(),
            day: 1.0,
            kind: "ignored".to_string(),
            startled,
            ..Reaction::default()
        };
        for _ in 0..3 {
            memory.reactions.push(reaction("sign", "words", true));
            memory.reactions.push(reaction("knock", "sounds", false));
        }
        memory.reactions.push(reaction("hole", "house", true)); // too few to say
        for minutes in [3, 9, 5] {
            memory.reactions.push(Reaction {
                fixed_minutes: Some(minutes),
                ..reaction("hole", "house", false)
            });
        }

        assert_eq!(memory.notice_rate("words"), Some(1.0));
        assert_eq!(memory.notice_rate("sounds"), Some(0.0));
        let lines = memory.reaction_lines();
        assert!(
            lines.contains(&"Words get their attention; sounds do not.".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"They put things right within about 5 minutes.".to_string()),
            "{lines:?}"
        );
        assert!(
            memory.detail_lines().starts_with(&lines),
            "among the first details"
        );
    }
}
