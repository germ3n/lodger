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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Container {
    pub pos: BlockPos,
    /// The block id: `minecraft:chest`, `minecraft:furnace`, `minecraft:barrel`.
    pub kind: String,
    pub opens: u32,
    pub last_opened_day: f32,
    pub contents: Vec<ItemStack>,
}

impl Container {
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
    /// `lit` (a light placed within 16 blocks), `walled` (three or more blocks placed within 4),
    /// `watched` (stood facing the spot a minute), `left` (left the game within a minute),
    /// `ignored` (a day passed), or empty while it is still being watched.
    pub kind: String,
}

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
    pub look_backs: u32,
    pub raining: bool,
    /// Under the ground: no sky light and below the surface.
    pub underground: bool,
    /// The day they last ate, for the sound that answers it.
    pub ate_day: f32,
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
    /// Ticks the target has stood facing the newest watched reaction's spot.
    pub watching_ticks: u32,
    /// The clock as of the last tick, for the sessions.
    pub last_real_millis: u64,
    pub last_day_time: u32,
}

/// Caps, with the reason beside each.
const MOST_CONTAINERS: usize = 32; // least-opened dropped first
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
const MOST_MILESTONES: usize = 128;
const MOST_SCANNED: usize = 64;
const MOST_VEHICLES: usize = 8;
const MOST_ROUTES: usize = 16; // the least-walked dropped first
const MOST_ROUTE_POINTS: usize = 128; // two blocks apart: a route of 256 blocks
const LEAST_ROUTE_POINTS: usize = 10;
const MOST_SESSIONS: usize = 30;
const MOST_REACTIONS: usize = 128; // oldest dropped first

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
    ];
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
    id.ends_with("torch") || id.ends_with("lantern") || id.contains("candle")
}

fn is_door(id: &str) -> bool {
    id.ends_with("_door") && !id.contains("trapdoor")
}

fn is_bed(id: &str) -> bool {
    id.ends_with("_bed")
}

impl Memory {
    /// Takes in a tick's observations. The target is the first player seen until the brain picks
    /// another (multiplayer, a later phase).
    pub fn observe(&mut self, observations: &Observations, day: f32) {
        self.today = day;
        if self.target.is_none() {
            if let Some(first) = observations.players.first() {
                self.target = Some(first.index);
                self.target_name = first.name.clone();
                self.target_uuid = first.uuid.clone();
            }
        }

        if let Some(world) = &observations.world {
            self.last.raining = world.raining;
        }

        if let Some(around) = &observations.surroundings {
            self.surroundings = Some(Surround {
                day,
                around: around.around,
                features: around.features.clone(),
            });
        }

        for scan in &observations.scans {
            self.scanned = scan
                .positions
                .iter()
                .take(MOST_SCANNED)
                .map(|(kind, pos)| (kind.rsplit(':').next().unwrap_or(kind).to_string(), *pos))
                .collect();
        }

        let Some(index) = self.target else { return };
        let player = observations.players.iter().find(|p| p.index == index);

        match player {
            Some(player) => self.observe_player(player),
            None => self.last.online = false,
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
            if event.player != index {
                continue;
            }

            match event.kind {
                EventKind::OpenedContainer => {
                    if let Some(pos) = event.pos {
                        self.container_opened(pos, &event.detail, &event.contents, day);
                    }
                }
                EventKind::SleptBegan => {
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
                    if let Some(reaction) =
                        self.reactions.iter_mut().rev().find(|r| r.kind.is_empty())
                    {
                        if (day - reaction.day) * 24_000.0 < 1_200.0 {
                            reaction.kind = "left".to_string();
                        }
                    }
                }
                EventKind::PlacedBlock => {
                    if let Some(pos) = event.pos {
                        push_capped(&mut self.placed, pos, MOST_PLACED);
                        self.react_to_placing(pos, &event.detail);
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
                        if is_light(&event.detail) || is_door(&event.detail) {
                            let home = self.home.get_or_insert_with(|| Home {
                                center: pos,
                                floor_y: pos[1],
                                ..Home::default()
                            });
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
                }
                EventKind::ChangedDimension => {
                    self.milestone("crossed", day, event.pos, &event.detail);
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
                        self.placed.retain(|p| *p != pos);
                        if let Some(home) = &mut self.home {
                            home.lights.retain(|p| *p != pos);
                            home.doors.retain(|p| *p != pos);
                        }
                        self.containers.retain(|c| c.pos != pos);
                        self.fixtures.retain(|f| f.pos != pos);
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
            if about_the_lodger(&line.text) {
                push_capped(&mut self.chat, line.text.clone(), MOST_CHAT);
            }
        }
    }

    fn observe_player(&mut self, player: &Player) {
        self.last.pos = player.pos;
        self.last.yaw = player.yaw;
        self.last.online = true;
        self.last.indoors = player.sky_light < 8;
        self.last.sleeping = player.sleeping;
        self.last.walking = player.movement.pace > 0.5;
        self.last.underground = player.sky_light == 0 && player.pos[1] < 56.0;
        self.last.main_hand = player.main_hand.id.clone();
        self.last.look_backs = player.movement.look_backs;

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

        // A reaction being watched: standing a minute facing the spot within 8 blocks.
        let watched_spot = self
            .reactions
            .iter()
            .rev()
            .find(|r| r.kind.is_empty())
            .and_then(|r| r.pos);
        if let Some(spot) = watched_spot {
            if player.movement.still_seconds >= 1.0 && in_view_from(player, spot, 8.0) {
                self.watching_ticks += 1;
                if self.watching_ticks >= 20 * 60 {
                    if let Some(reaction) =
                        self.reactions.iter_mut().rev().find(|r| r.kind.is_empty())
                    {
                        reaction.kind = "watched".to_string();
                    }
                    self.watching_ticks = 0;
                }
            } else {
                self.watching_ticks = 0;
            }
        }

        if player.movement.still_seconds >= 1.0 {
            let cell = [
                player.pos[0].floor() as i32 / CELL,
                player.pos[1].floor() as i32 / CELL,
                player.pos[2].floor() as i32 / CELL,
            ];
            match self.still_cells.iter_mut().find(|(c, _)| *c == cell) {
                Some((_, ticks)) => *ticks += 1,
                None => {
                    self.still_cells.push((cell, 1));
                    if self.still_cells.len() > MOST_CELLS {
                        let least = self
                            .still_cells
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, (_, t))| *t)
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        self.still_cells.remove(least);
                    }
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
                None => self.favourites.push((player.main_hand.id.clone(), 1)),
            }
            self.favourites
                .sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            self.favourites.truncate(MOST_FAVOURITES);
        }
    }

    fn container_opened(&mut self, pos: BlockPos, kind: &str, contents: &[ItemStack], day: f32) {
        match self.containers.iter_mut().find(|c| c.pos == pos) {
            Some(container) => {
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
                });
                if self.containers.len() > MOST_CONTAINERS {
                    let least = self
                        .containers
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, c)| c.opens)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.containers.remove(least);
                }
            }
        }
    }

    /// Records an event's outcome. One the target would notice (a bold row) that landed is a
    /// landmark too, kept for the whole stay.
    pub fn record(&mut self, event: &str, pos: Option<BlockPos>, day: f32, ok: bool, reason: &str) {
        let done = Done {
            event: event.to_string(),
            pos,
            day,
            ok,
            reason: reason.to_string(),
        };
        if ok
            && crate::events::find(event)
                .map(|e| !e.small)
                .unwrap_or(false)
        {
            push_capped(&mut self.landmarks, done.clone(), MOST_LANDMARKS);
            // An older reaction still open was ignored; this one is watched for a day.
            for reaction in self.reactions.iter_mut().filter(|r| r.kind.is_empty()) {
                reaction.kind = "ignored".to_string();
            }
            push_capped(
                &mut self.reactions,
                Reaction {
                    event: event.to_string(),
                    day,
                    pos,
                    kind: String::new(),
                },
                MOST_REACTIONS,
            );
            self.watching_ticks = 0;
        }
        push_capped(&mut self.done, done, MOST_DONE);
    }

    /// The clock, once a tick from the stay: real time and the time of day for the sessions,
    /// and the day that closes a reaction unanswered.
    pub fn clock(&mut self, now: &craftkit::world::Now, day: f32) {
        self.last_real_millis = now.real_millis;
        self.last_day_time = now.day_time;
        if let Some(reaction) = self.reactions.iter_mut().rev().find(|r| r.kind.is_empty()) {
            if day - reaction.day >= 1.0 {
                reaction.kind = "ignored".to_string();
            }
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
                self.routes.push(Route {
                    points: trail,
                    walked: 1,
                    first_day: self.today,
                });
                if self.routes.len() > MOST_ROUTES {
                    let least = self
                        .routes
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, r)| r.walked)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.routes.remove(least);
                }
            }
        }
        self.routes.sort_by_key(|r| std::cmp::Reverse(r.walked));
    }

    /// A block placed answers the newest open reaction: a light within 16 blocks lit the spot,
    /// three solid blocks within 4 walled it.
    fn react_to_placing(&mut self, pos: BlockPos, block_id: &str) {
        let near = self
            .placed
            .iter()
            .filter(|p| {
                self.reactions
                    .iter()
                    .rev()
                    .find(|r| r.kind.is_empty())
                    .and_then(|r| r.pos)
                    .map(|spot| distance(spot, **p) <= 4.0)
                    .unwrap_or(false)
            })
            .count();
        let Some(reaction) = self.reactions.iter_mut().rev().find(|r| r.kind.is_empty()) else {
            return;
        };
        let Some(spot) = reaction.pos else { return };
        if is_light(block_id) && distance(spot, pos) <= 16.0 {
            reaction.kind = "lit".to_string();
        } else if !is_light(block_id) && distance(spot, pos) <= 4.0 && near >= 3 {
            reaction.kind = "walled".to_string();
        }
    }

    /// The route walked most, if any was walked twice.
    pub fn route(&self) -> Option<&Route> {
        self.routes.iter().find(|r| r.walked >= 2)
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
            .filter(|c| !c.is_furnace())
            .max_by_key(|c| c.opens)
    }

    /// The second chest the target opens, if there is one.
    pub fn other_container(&self) -> Option<&Container> {
        let used = self.used_container()?.pos;

        self.containers
            .iter()
            .filter(|c| !c.is_furnace() && c.pos != used)
            .max_by_key(|c| c.opens)
    }

    /// The furnace the target used most.
    pub fn furnace(&self) -> Option<&Container> {
        self.containers
            .iter()
            .filter(|c| c.is_furnace())
            .max_by_key(|c| c.opens)
    }

    /// The fixtures of a plain kind (`fence_gate`), oldest first.
    pub fn fixtures<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Fixture> + 'a {
        self.fixtures.iter().filter(move |f| f.kind == kind)
    }

    /// Whether any fixture of this kind is known.
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

    /// Where the village's blocks of a kind are, from the last scan (`door`, `bell`).
    pub fn scanned<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = BlockPos> + 'a {
        self.scanned
            .iter()
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
            for container in &self.containers {
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

    /// The lines the brain reads. A template, never prose: see `docs/memory.md`.
    pub fn describe(&self, day: f32) -> Vec<String> {
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
                place_words(self.home.as_ref(), container.pos),
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
            let ago = (day - done.day).max(0.0);
            let when = if ago < 1.0 {
                "Today".to_string()
            } else {
                format!("{} days ago", ago.floor() as u32)
            };
            lines.push(format!(
                "{} it {} {}.",
                when,
                if done.ok { "did" } else { "could not" },
                crate::events::find(&done.event)
                    .map(|e| e.menu_words())
                    .unwrap_or(&done.event)
            ));
        }

        // The oldest landmark not among the recent lines: what it did that they would remember.
        if let Some(landmark) = self
            .landmarks
            .iter()
            .rev()
            .find(|l| !self.recent(3).any(|d| d.day == l.day && d.event == l.event))
        {
            let ago = (day - landmark.day).max(0.0);
            lines.push(format!(
                "{} it {}{}.",
                if ago < 1.0 {
                    "Earlier today".to_string()
                } else {
                    format!("{} days ago", ago.floor() as u32)
                },
                crate::events::find(&landmark.event)
                    .map(|e| e.menu_words())
                    .unwrap_or(&landmark.event),
                landmark
                    .pos
                    .zip(self.home.as_ref())
                    .map(|(p, h)| format!(", {}", place_words(Some(h), p)))
                    .unwrap_or_default()
            ));
        }

        if let Some(milestone) = self.milestones.last() {
            let ago = (day - milestone.day).max(0.0);
            let when = if ago < 1.0 {
                "Today".to_string()
            } else {
                format!("{} days ago", ago.floor() as u32)
            };
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
                    .map(|(p, h)| format!(", {}", place_words(Some(h), p)))
                    .unwrap_or_default()
            ));
        }

        if let Some(route) = self.route() {
            if let Some(home) = &self.home {
                let end = self.route_end().unwrap_or(route.end());
                lines.push(format!(
                    "They walk a way of {} blocks to {}, {} times so far.",
                    route.length().round() as i32,
                    place_words(Some(home), end),
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
                    if let Some(home) = &self.home {
                        parts.push(format!(
                            "{} {}",
                            words,
                            place_words(Some(home), feature.pos)
                        ));
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

        if let Some(tally) = self.most_killed() {
            lines.push(format!(
                "They have killed {} {}{} so far.",
                tally.kills,
                tally.kind.replace('_', " "),
                if tally.kills == 1 { "" } else { "s" }
            ));
        }

        if let Some(held) = &self.held {
            lines.push(format!(
                "It holds the player's {}, taken {} days ago.",
                plain(&held.item.id),
                (day - held.taken_day).max(0.0).floor() as u32
            ));
        }

        if let Some(line) = self.chat.last() {
            lines.push(format!("The player said: \"{}\".", line));
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

/// `minecraft:iron_pickaxe` as `an iron pickaxe`.
pub fn plain(id: &str) -> String {
    let name = id.rsplit(':').next().unwrap_or(id).replace('_', " ");
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };

    format!("{article} {name}")
}

fn place_words(home: Option<&Home>, pos: BlockPos) -> String {
    match home {
        Some(home) => {
            let d = distance(home.center, pos);
            if d <= 6.0 {
                "in the house".to_string()
            } else {
                let dx = pos[0] - home.center[0];
                let dz = pos[2] - home.center[2];
                let dir = if dx.abs() > dz.abs() {
                    if dx > 0 {
                        "east"
                    } else {
                        "west"
                    }
                } else if dz > 0 {
                    "south"
                } else {
                    "north"
                };
                format!("{} blocks {} of the house", d.round() as i32, dir)
            }
        }
        None => "somewhere".to_string(),
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
    fn a_bed_slept_in_is_the_home_and_a_chest_opened_is_the_used_container() {
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
            lines
                .iter()
                .any(|l| l == "1 days ago it did open the door."),
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
}
