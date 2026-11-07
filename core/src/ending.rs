//! The endings (batch 16, owner: "all three as written"; `docs/entity.md`, How it
//! ends). Stage five is three in-game days of a script, from the first dawn after it begins:
//! each ending a list of beats, each at a day and an hour, run in order as the clock reaches
//! them. A beat that is a moment (a sound, a sight, the screen) is skipped when it comes late (the
//! game was closed over it); one that changes the world is done late. After the last, the stay
//! is over and says how: `left`, `took` or `stayed`.
//!
//! Which ending: `stages.ending`, or with `difficulty` the want met leaves, refused takes on
//! normal and hard and keeps on easy and peaceful. Temper is not kept yet, so it has no say.

use craftkit::world::{BlockPos, BlueprintPart, Command, ItemStack, Now, Pos};

use crate::arc::Arc;
use crate::config::Config;
use crate::memory::{distance, Memory};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Leave,
    Take,
    Keep,
}

impl Ending {
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "leave" => Some(Self::Leave),
            "take" => Some(Self::Take),
            "keep" => Some(Self::Keep),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Leave => "leave",
            Self::Take => "take",
            Self::Keep => "keep",
        }
    }

    /// What the stay says once it is over.
    pub fn said(self) -> &'static str {
        match self {
            Self::Leave => "left",
            Self::Take => "took",
            Self::Keep => "stayed",
        }
    }

    pub fn script(self) -> &'static [Step] {
        match self {
            Self::Leave => LEAVES,
            Self::Take => TAKES,
            Self::Keep => KEEPS,
        }
    }
}

/// Which ending this stay has.
pub fn choose(config: &Config, memory: &Memory) -> Ending {
    if let Some(named) = Ending::named(&config.stages.ending) {
        return named;
    }
    if want_met(memory) {
        return Ending::Leave;
    }
    match memory.difficulty.as_str() {
        "normal" | "hard" => Ending::Take,
        _ => Ending::Keep,
    }
}

/// Whether the want was met (`docs/entity.md`, What it wants), read off the memory as the
/// stay ends: a place of its own with three blocks or more of theirs built within five blocks
/// of it, or a chest of theirs there with something in it; to be looked at, three things it
/// showed them looked at and not run from; one thing of theirs, their favourite left in the
/// chest it uses.
pub fn want_met(memory: &Memory) -> bool {
    match memory.want.as_str() {
        "a place of its own" => memory.place.is_some_and(|place| {
            memory
                .placed
                .iter()
                .filter(|p| !memory.placed_at_place.contains(p))
                .filter(|p| distance(**p, place) <= 5.0)
                .count()
                >= 3
                || memory
                    .containers
                    .iter()
                    .any(|c| c.theirs() && !c.contents.is_empty() && distance(c.pos, place) <= 5.0)
        }),
        "to be looked at" => {
            memory
                .reactions
                .iter()
                .filter(|r| r.effect == "seen" && r.looked && !r.ran)
                .count()
                >= 3
        }
        "one thing of theirs" => memory.favourite_in_container().is_some(),
        _ => false,
    }
}

/// Whether the taking ending's one death is dealt: on hard; with `intense`, on normal too; with
/// `relentless`, on every difficulty (`config.rs`'s presets say so).
pub fn deals_death(config: &Config, memory: &Memory) -> bool {
    match config.preset.as_str() {
        "relentless" => true,
        "intense" => matches!(memory.difficulty.as_str(), "normal" | "hard"),
        _ => memory.difficulty == "hard",
    }
}

/// What a beat does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beat {
    // Leaves.
    /// What it holds, back where it was taken from.
    ReturnHeld,
    /// Its torches out one by one, from the farthest from the house in.
    TorchesOut,
    /// Its house's chest emptied onto their doorstep.
    HouseEmptied,
    /// The figure at its door in plain light a full minute, then in.
    FigureAtItsDoor,
    /// Their pets facing its house.
    PetsFaceItsHouse,
    /// The torch in its house out.
    ItsTorchOut,
    /// The music, the first time in a week.
    Music,
    /// Every light of theirs flickers once.
    Flicker,
    /// The figure at the edge of their light, for as long as they look.
    FigureAtEdge,
    /// Its house gone, the ground flat, one torch where its door was.
    HouseGone,
    // Takes.
    /// Their animals stare at the house.
    AnimalsStare,
    /// The sun blinks out.
    SunBlink,
    /// No sleep tonight.
    NoSleep,
    /// Every door of the house open at once, their pets put out.
    DoorsOpen,
    /// Every frame empty, the things on the bed in a line toward the door.
    FramesOnBed,
    /// Every sign of theirs says one word.
    SignsSay,
    /// The figure behind them, this many blocks off.
    FigureBehind(u32),
    /// Their tunnels walled at the mouth.
    TunnelsSealed,
    /// The village bell, once.
    BellRings,
    /// Every torch it ever placed lit, theirs dark on their screen, red rain.
    LightsTurn,
    /// The figure at its place, holding what it took, walks up to one block from them.
    FigureComes,
    /// On hard, they die, once, keeping everything.
    Death,
    /// Their doors shut again: the house as it was, but for what it took.
    DoorsShut,
    // Keeps.
    /// Its house a second room.
    SecondRoom,
    /// The figure in its window.
    FigureInItsWindow,
    /// A cobblestone path from their door to its.
    CobblePath,
    /// Their pets at its door.
    PetsToItsDoor,
    /// Their pets home again.
    PetsHome,
    /// Its door open and lit, the figure in it, for as long as they look.
    DoorwayLit,
}

impl Beat {
    /// A moment: skipped when it comes late, where a change to the world is done late.
    pub fn moment(self) -> bool {
        matches!(
            self,
            Beat::FigureAtItsDoor
                | Beat::PetsFaceItsHouse
                | Beat::Music
                | Beat::Flicker
                | Beat::FigureAtEdge
                | Beat::AnimalsStare
                | Beat::SunBlink
                | Beat::FigureBehind(_)
                | Beat::BellRings
                | Beat::FigureComes
                | Beat::Death
                | Beat::FigureInItsWindow
                | Beat::PetsToItsDoor
                | Beat::DoorwayLit
                | Beat::NoSleep
        )
    }
}

/// One beat: its day of the three (0 to 2) and its hour (the game's day time: 0 is dawn, 6000
/// noon, 12000 dusk, 18000 midnight).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub day: u8,
    pub at: u32,
    pub beat: Beat,
}

const fn s(day: u8, at: u32, beat: Beat) -> Step {
    Step { day, at, beat }
}

/// How long late a moment may come and still be itself.
const LATE: u32 = 2_000;

pub const LEAVES: &[Step] = &[
    s(0, 1_000, Beat::ReturnHeld),
    s(0, 2_000, Beat::TorchesOut),
    s(1, 0, Beat::HouseEmptied),
    s(1, 6_000, Beat::FigureAtItsDoor),
    s(1, 7_500, Beat::PetsFaceItsHouse),
    s(1, 12_000, Beat::ItsTorchOut),
    s(2, 13_000, Beat::Music),
    s(2, 18_000, Beat::Flicker),
    s(2, 18_040, Beat::FigureAtEdge),
    s(2, 23_000, Beat::HouseGone),
];

pub const TAKES: &[Step] = &[
    s(0, 1_000, Beat::AnimalsStare),
    s(0, 2_000, Beat::SunBlink),
    s(0, 3_500, Beat::SunBlink),
    s(0, 5_000, Beat::SunBlink),
    s(0, 6_000, Beat::AnimalsStare),
    s(0, 12_000, Beat::NoSleep),
    s(0, 18_000, Beat::DoorsOpen),
    s(1, 1_000, Beat::FramesOnBed),
    s(1, 1_000, Beat::BellRings),
    s(1, 2_000, Beat::BellRings),
    s(1, 3_000, Beat::SignsSay),
    s(1, 3_000, Beat::BellRings),
    s(1, 4_000, Beat::FigureBehind(64)),
    s(1, 4_000, Beat::BellRings),
    s(1, 5_000, Beat::BellRings),
    s(1, 6_000, Beat::BellRings),
    s(1, 7_000, Beat::FigureBehind(48)),
    s(1, 7_000, Beat::BellRings),
    s(1, 8_000, Beat::TunnelsSealed),
    s(1, 8_000, Beat::BellRings),
    s(1, 9_000, Beat::BellRings),
    s(1, 10_000, Beat::FigureBehind(32)),
    s(1, 10_000, Beat::BellRings),
    s(1, 11_000, Beat::BellRings),
    s(2, 13_000, Beat::LightsTurn),
    s(2, 18_000, Beat::FigureComes),
    s(2, 18_600, Beat::Death),
    s(2, 23_000, Beat::DoorsShut),
];

pub const KEEPS: &[Step] = &[
    s(0, 13_000, Beat::SecondRoom),
    s(0, 14_000, Beat::FigureInItsWindow),
    s(0, 16_000, Beat::FigureInItsWindow),
    s(0, 18_000, Beat::FigureInItsWindow),
    s(0, 20_000, Beat::FigureInItsWindow),
    s(0, 22_000, Beat::FigureInItsWindow),
    s(1, 1_000, Beat::CobblePath),
    s(1, 6_000, Beat::PetsToItsDoor),
    s(1, 8_000, Beat::PetsHome),
    s(2, 18_000, Beat::DoorwayLit),
];

/// The game day the script's first day is: the next dawn after `now`, today if it is still
/// the first hour.
pub fn first_day(now: &Now) -> u64 {
    if now.day_time < 1_000 {
        now.day
    } else {
        now.day + 1
    }
}

/// Stage five's tick: the ending chosen on its first, then the beats whose time has come, one
/// a tick, each with the ticks it waits. Over after the last, at the third day's end.
pub fn tick(
    arc: &mut Arc,
    memory: &mut Memory,
    config: &Config,
    now: &Now,
    ids: &mut impl FnMut() -> u64,
) -> Vec<(Command, u64)> {
    if arc.stage != 5 {
        return Vec::new();
    }
    let ending = match Ending::named(&arc.ending) {
        Some(ending) => ending,
        None => {
            let ending = choose(config, memory);
            arc.ending = ending.name().to_string();
            arc.script_day = first_day(now);
            arc.beats_done = 0;
            return Vec::new();
        }
    };
    // The fourth ending: they come to it, from stage five's first night. While the fight is on
    // the script waits.
    let mut made = confrontation(arc, memory, now, ids);
    if arc.over() || arc.fighting_since > 0 || !made.is_empty() {
        return made;
    }
    if now.day < arc.script_day {
        return made;
    }
    made.extend(beacon(arc, memory, now, ids));
    if !made.is_empty() {
        return made;
    }
    let script = ending.script();
    let day = (now.day - arc.script_day).min(u8::MAX as u64) as u8;
    let Some(step) = script.get(arc.beats_done).copied() else {
        // Every beat done: over at the last day's end, or at once when the clock is past it.
        if day > 2 || (day == 2 && now.day_time >= 23_500) {
            end(arc, memory, ending, now);
        }
        return Vec::new();
    };
    let due = day > step.day || (day == step.day && now.day_time >= step.at);
    if !due {
        return Vec::new();
    }
    arc.beats_done += 1;
    let late = if day > step.day {
        u32::MAX
    } else {
        now.day_time - step.at
    };
    if step.beat.moment() && late > LATE {
        return Vec::new();
    }
    if step.beat == Beat::Death && !deals_death(config, memory) {
        return Vec::new();
    }

    commands(step.beat, memory, now, ids)
}

/// Night: from dusk's end to dawn.
fn night(now: &Now) -> bool {
    (13_000..23_000).contains(&now.day_time)
}

/// The longest a fight waits for its answer before it is taken for over (the host lost it).
const FIGHT_MOST: u64 = 24_000;

/// The confrontation (owner, "a fourth ending: you confront it"): in stage five,
/// at night, they knock on its door (or, with no house, stand at its place) and the fight
/// begins (`Command::Confront`, the host's to run); its answer ends the stay: `beaten` if they
/// won (its house's chest spilled on their step, what it held given back by the host), `kept`
/// if they lost (it keeps what it took and leaves). Run from, it can be come to again.
fn confrontation(
    arc: &mut Arc,
    memory: &mut Memory,
    now: &Now,
    ids: &mut impl FnMut() -> u64,
) -> Vec<(Command, u64)> {
    if let Some(result) = memory.fight_ended.take() {
        arc.fighting_since = 0;
        match result.as_str() {
            "won" => {
                let mut made = Vec::new();
                if let (Some(chest), Some(step)) =
                    (its(memory, "minecraft:chest", ""), doorstep(memory))
                {
                    made.push((
                        Command::SpillContainer {
                            id: ids(),
                            from: chest,
                            to: [
                                step[0] as f64 + 0.5,
                                step[1] as f64 + 0.2,
                                step[2] as f64 + 0.5,
                            ],
                            remove: false,
                        },
                        0,
                    ));
                }
                // The host gave back what it held as the construct broke.
                memory.held = None;
                made.extend(capped(arc, memory, ids));
                finish(arc, memory, "beaten", now);
                return made;
            }
            "lost" => {
                let made = capped(arc, memory, ids);
                finish(arc, memory, "kept", now);
                return made;
            }
            _ => return Vec::new(),
        }
    }
    if arc.fighting_since > 0 {
        if now.tick.saturating_sub(arc.fighting_since) > FIGHT_MOST {
            arc.fighting_since = 0;
        }
        return Vec::new();
    }
    let forced = std::mem::take(&mut memory.fight_now);
    if !forced && (!night(now) || !memory.last.online) {
        return Vec::new();
    }
    let door = its_door(memory).map(|(d, _)| d);
    // With no house, crouching at its place calls it: walking past is no knock.
    let at_place = door.is_none()
        && memory.last.sneaking
        && memory
            .place
            .is_some_and(|p| distance(p, crate::memory::block_of(memory.last.pos)) <= 3.0);
    let knocked = door.is_some() && memory.knocked_now == door;
    if !knocked && !at_place && !forced {
        return Vec::new();
    }
    memory.knocked_now = None;
    // In front of its door, outside: where it rises.
    let at = if forced {
        crate::memory::block_of(memory.last.pos)
    } else {
        its_door(memory)
            .map(|(_, out)| out)
            .or(memory.place)
            .unwrap_or([0, 0, 0])
    };
    let roof = memory
        .build("house_near")
        .filter(|b| b.ground_known())
        .map(|b| {
            [
                b.site[0] as f64 + 0.5,
                b.site[1] as f64 + 5.0,
                b.site[2] as f64 + 0.5,
            ]
        })
        .unwrap_or([at[0] as f64 + 0.5, -10_000.0, at[2] as f64 + 0.5]);
    arc.fighting_since = now.tick.max(1);
    let id = ids();
    memory.confront_id = id;

    vec![(
        Command::Confront {
            id,
            player: memory.target.unwrap_or(0),
            at,
            roof,
            blocks: construct_blocks(memory),
        },
        0,
    )]
}

/// What the construct is built of: what it dug and holds, most first, their building block, a
/// torch for its head; six at most.
pub fn construct_blocks(memory: &Memory) -> Vec<String> {
    let mut stores: Vec<&(String, u32)> = memory.stores.iter().filter(|(_, n)| *n > 0).collect();
    stores.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let mut blocks: Vec<String> = stores.iter().map(|(id, _)| id.clone()).take(4).collect();
    if let Some(theirs) = memory.block_built_most() {
        if !blocks.contains(&theirs) {
            blocks.push(theirs);
        }
    }
    if blocks.is_empty() {
        blocks.push("minecraft:cobblestone".to_string());
    }
    blocks.truncate(5);
    blocks.push("minecraft:torch".to_string());
    blocks
}

/// Its beacon (owner: "real, and dark until night"): from stage five's first dusk,
/// on its house's roof (the middle of the roof iron, the beacon on it), or at its place; lit
/// at dusk, a block of cobblestone over it by day.
fn beacon(
    arc: &mut Arc,
    memory: &Memory,
    now: &Now,
    ids: &mut impl FnMut() -> u64,
) -> Vec<(Command, u64)> {
    let Some(base) = beacon_base(memory) else {
        return Vec::new();
    };
    let cap = [base[0], base[1] + 2, base[2]];
    let dusk = (12_000..13_000).contains(&now.day_time);
    let dawn = now.day_time < 1_000 || now.day_time >= 23_000;
    if dusk && !arc.beacon_built {
        arc.beacon_built = true;
        arc.beacon_capped = false;
        let mut parts = Vec::new();
        for x in -1..=1 {
            for z in -1..=1 {
                parts.push(BlueprintPart {
                    at: [x, 0, z],
                    block: "minecraft:iron_block".to_string(),
                    state: String::new(),
                });
            }
        }
        parts.push(BlueprintPart {
            at: [0, 1, 0],
            block: "minecraft:beacon".to_string(),
            state: String::new(),
        });
        return vec![(
            Command::PlaceBlueprint {
                id: ids(),
                at: base,
                parts,
                replace: vec![
                    "minecraft:oak_log".to_string(),
                    "minecraft:grass_block".to_string(),
                    "minecraft:dirt".to_string(),
                    "minecraft:short_grass".to_string(),
                    "minecraft:snow".to_string(),
                ],
            },
            0,
        )];
    }
    if !arc.beacon_built {
        return Vec::new();
    }
    if dusk && arc.beacon_capped {
        arc.beacon_capped = false;
        return vec![(
            Command::SetBlock {
                id: ids(),
                pos: cap,
                block: "minecraft:air".to_string(),
                state: String::new(),
                replace: vec!["minecraft:cobblestone".to_string()],
                settle: 0,
                wall: String::new(),
                rest: false,
                only: true,
            },
            0,
        )];
    }
    if dawn && !arc.beacon_capped {
        arc.beacon_capped = true;
        return vec![(
            Command::SetBlock {
                id: ids(),
                pos: cap,
                block: "minecraft:cobblestone".to_string(),
                state: String::new(),
                replace: vec![],
                settle: 0,
                wall: String::new(),
                rest: false,
                only: false,
            },
            0,
        )];
    }
    Vec::new()
}

/// Its beacon capped for good as the fight ends the stay: it shines no more.
fn capped(arc: &Arc, memory: &Memory, ids: &mut impl FnMut() -> u64) -> Vec<(Command, u64)> {
    match beacon_base(memory) {
        Some(base) if arc.beacon_built && !arc.beacon_capped => vec![(
            Command::SetBlock {
                id: ids(),
                pos: [base[0], base[1] + 2, base[2]],
                block: "minecraft:cobblestone".to_string(),
                state: String::new(),
                replace: vec![],
                settle: 0,
                wall: String::new(),
                rest: false,
                only: false,
            },
            0,
        )],
        _ => Vec::new(),
    }
}

/// Where its beacon's iron goes: the middle of its house's roof, or its place's ground.
pub fn beacon_base(memory: &Memory) -> Option<BlockPos> {
    match memory
        .build("house_near")
        .filter(|b| b.ground_known() && b.laid > 0)
    {
        Some(house) => Some([house.site[0], house.site[1] + 4, house.site[2]]),
        None => memory.place,
    }
}

/// The stay over by the fight, the way it ended said.
fn finish(arc: &mut Arc, memory: &mut Memory, ended: &str, now: &Now) {
    arc.stage = 6;
    arc.ended = ended.to_string();
    arc.next_open_day = now.day + 21;
    memory.its_lights.clear();
}

/// The stay over, the way it ended said.
fn end(arc: &mut Arc, memory: &mut Memory, ending: Ending, now: &Now) {
    arc.stage = 6;
    arc.ended = ending.said().to_string();
    // A door of theirs open now and then, for good (`keeps`): the first in three weeks.
    arc.next_open_day = now.day + 21;
    memory.its_lights.clear();
}

/// After the stay, when it kept: every three in-game weeks, while they are away or asleep, one
/// of their doors is found open.
pub fn after(
    arc: &mut Arc,
    memory: &Memory,
    now: &Now,
    ids: &mut impl FnMut() -> u64,
) -> Vec<Command> {
    if arc.ended != "stayed" || now.day < arc.next_open_day {
        return Vec::new();
    }
    let away = !memory.last.online || memory.last.sleeping;
    let Some(door) = memory.usual_door().filter(|_| away) else {
        return Vec::new();
    };
    arc.next_open_day = now.day + 21;
    vec![open(ids, door)]
}

/// The parts of its house as they stand, at their places in the world, when it has one.
fn its_house(memory: &Memory) -> Option<Vec<(BlockPos, crate::building::Part)>> {
    let build = memory
        .build("house_near")
        .filter(|b| b.ground_known() && b.laid > 0)?;
    let parts = crate::building::house(build.way, build.grown);
    Some(
        parts
            .into_iter()
            .map(|p| {
                (
                    [
                        build.site[0] + p.at[0],
                        build.site[1] + p.at[1],
                        build.site[2] + p.at[2],
                    ],
                    p,
                )
            })
            .collect(),
    )
}

/// One part of its house by block and half, where it is.
fn its(memory: &Memory, block: &str, state_has: &str) -> Option<BlockPos> {
    its_house(memory)?
        .into_iter()
        .find(|(_, p)| p.block == block && p.state.contains(state_has))
        .map(|(at, _)| at)
}

/// Its door's lower half, and the block outside it toward theirs.
fn its_door(memory: &Memory) -> Option<(BlockPos, BlockPos)> {
    let door = its(memory, "minecraft:oak_door", "half=lower")?;
    let way = memory.build("house_near")?.way;
    Some((door, [door[0] + way[0], door[1], door[2] + way[1]]))
}

fn center(p: BlockPos) -> Pos {
    [p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5]
}

/// Outside their usual door, a step out from the house.
fn doorstep(memory: &Memory) -> Option<BlockPos> {
    let door = memory.usual_door()?;
    let home = memory.home.as_ref()?;
    let (dx, dz) = (door[0] - home.center[0], door[2] - home.center[2]);
    let step = if dx.abs() >= dz.abs() {
        [dx.signum(), 0]
    } else {
        [0, dz.signum()]
    };
    Some([door[0] + step[0], door[1], door[2] + step[1]])
}

fn open(ids: &mut impl FnMut() -> u64, door: BlockPos) -> Command {
    Command::SetState {
        id: ids(),
        pos: door,
        property: "open".to_string(),
        value: "true".to_string(),
    }
}

fn figure(
    ids: &mut impl FnMut() -> u64,
    memory: &Memory,
    at: Pos,
    seconds: f32,
    vanish_within: f32,
    once_seen: bool,
) -> Command {
    Command::Figure {
        id: ids(),
        figure: 1,
        skin_of: memory.target.unwrap_or(0),
        pos: at,
        path: Vec::new(),
        held: ItemStack::default(),
        name_shown: false,
        seconds,
        vanish_within,
        mirror: false,
        once_seen,
        copy_crouch: false,
        swinging: false,
    }
}

const LIGHTS: &[&str] = &[
    "minecraft:torch",
    "minecraft:wall_torch",
    "minecraft:lantern",
    "minecraft:soul_torch",
    "minecraft:soul_wall_torch",
    "minecraft:soul_lantern",
];

/// Its own light out: the block back to air, only if it is still a light.
fn light_out(ids: &mut impl FnMut() -> u64, pos: BlockPos) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: "minecraft:air".to_string(),
        state: String::new(),
        replace: LIGHTS.iter().map(|l| l.to_string()).collect(),
        settle: 0,
        wall: String::new(),
        rest: false,
        only: true,
    }
}

fn torch_at(ids: &mut impl FnMut() -> u64, pos: BlockPos) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: "minecraft:torch".to_string(),
        state: String::new(),
        replace: vec![],
        settle: 0,
        wall: "minecraft:wall_torch".to_string(),
        rest: false,
        only: false,
    }
}

/// What a beat sends, each with the ticks it waits.
pub fn commands(
    beat: Beat,
    memory: &mut Memory,
    now: &Now,
    ids: &mut impl FnMut() -> u64,
) -> Vec<(Command, u64)> {
    let target = memory.target.unwrap_or(0);
    let at = memory.last.pos;
    match beat {
        Beat::ReturnHeld => match (&memory.held, crate::events::find("return_taken")) {
            (Some(_), Some(event)) => {
                let pos = memory.held.as_ref().map(|h| h.from);
                let made = crate::brain::act::commands(event, pos, memory, ids, now, None);
                // Given back is no longer held: a beat has no report to clear it by, and it
                // used to stay "held" for the rest of the stay.
                if !made.is_empty() {
                    memory.held = None;
                }
                made
            }
            _ => Vec::new(),
        },
        Beat::TorchesOut => {
            let home = memory.home.as_ref().map(|h| h.center).unwrap_or([0, 0, 0]);
            let mut lights = memory.its_lights.clone();
            lights.sort_by(|a, b| distance(*b, home).total_cmp(&distance(*a, home)));
            lights
                .into_iter()
                .enumerate()
                .map(|(i, p)| (light_out(ids, p), i as u64 * 300))
                .collect()
        }
        Beat::HouseEmptied => match (its(memory, "minecraft:chest", ""), doorstep(memory)) {
            (Some(chest), Some(step)) => vec![(
                Command::SpillContainer {
                    id: ids(),
                    from: chest,
                    to: [
                        step[0] as f64 + 0.5,
                        step[1] as f64 + 0.2,
                        step[2] as f64 + 0.5,
                    ],
                    remove: false,
                },
                0,
            )],
            _ => Vec::new(),
        },
        Beat::FigureAtItsDoor => {
            let Some((door, out)) = its_door(memory) else {
                return memory
                    .place
                    .map(|p| {
                        vec![(
                            figure(
                                ids,
                                memory,
                                [p[0] as f64 + 0.5, -10_000.0, p[2] as f64 + 0.5],
                                60.0,
                                0.0,
                                false,
                            ),
                            0,
                        )]
                    })
                    .unwrap_or_default();
            };
            let mut going = figure(ids, memory, center(out), 3.0, 0.0, false);
            if let Command::Figure { path, .. } = &mut going {
                *path = vec![center(out), center(door)];
            }
            vec![
                (figure(ids, memory, center(out), 60.0, 0.0, false), 0),
                (going, 1_200),
            ]
        }
        Beat::PetsFaceItsHouse => {
            let Some(house) = memory.build("house_near").map(|b| b.site) else {
                return Vec::new();
            };
            memory
                .pets()
                .map(|p| {
                    (
                        Command::Hold {
                            id: ids(),
                            entity: p.id,
                            player: target,
                            seconds: 600.0,
                            at: Some(center(house)),
                            creep: false,
                        },
                        0,
                    )
                })
                .collect()
        }
        Beat::ItsTorchOut => its(memory, "minecraft:wall_torch", "")
            .map(|p| vec![(light_out(ids, p), 0)])
            .unwrap_or_default(),
        Beat::Music => vec![(
            Command::PlaySound {
                id: ids(),
                player: target,
                sound: "minecraft:music.game".to_string(),
                pos: at,
                volume: 1.0,
                pitch: 1.0,
            },
            0,
        )],
        Beat::Flicker => vec![(
            Command::Hint {
                id: ids(),
                player: target,
                hint: "flicker".to_string(),
                pos: None,
                seconds: 1.5,
                detail: String::new(),
            },
            0,
        )],
        Beat::FigureAtEdge => {
            // Fourteen blocks off, on the side away from the house: where the light ends.
            let home = memory.home.as_ref().map(|h| center(h.center)).unwrap_or(at);
            let (dx, dz) = (at[0] - home[0], at[2] - home[2]);
            let length = (dx * dx + dz * dz).sqrt().max(1e-6);
            let (ux, uz) = if length < 1.0 {
                (1.0, 0.0)
            } else {
                (dx / length, dz / length)
            };
            let spot = [at[0] + ux * 14.0, -10_000.0, at[2] + uz * 14.0];
            vec![(figure(ids, memory, spot, 600.0, 0.0, true), 0)]
        }
        Beat::HouseGone => {
            let Some(house) = its_house(memory) else {
                return Vec::new();
            };
            let site = memory
                .build("house_near")
                .map(|b| b.site)
                .unwrap_or([0, 0, 0]);
            let door = its_door(memory).map(|(d, _)| d);
            let mut made = Vec::new();
            if let (Some(chest), Some(step)) =
                (its(memory, "minecraft:chest", ""), doorstep(memory))
            {
                made.push((
                    Command::SpillContainer {
                        id: ids(),
                        from: chest,
                        to: [
                            step[0] as f64 + 0.5,
                            step[1] as f64 + 0.2,
                            step[2] as f64 + 0.5,
                        ],
                        remove: true,
                    },
                    0,
                ));
            }
            // Its blocks back to air, top down, its floor back to grass: the ground flat.
            let mut parts: Vec<_> = house.into_iter().collect();
            parts.sort_by_key(|(p, _)| std::cmp::Reverse(p[1]));
            let mut blocks: Vec<String> = parts
                .iter()
                .map(|(_, p)| p.block.to_string())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            // Its beacon on the roof goes with the house: the iron where the logs were, the
            // beacon over it, its cap.
            blocks.extend(["minecraft:iron_block", "minecraft:beacon"].map(String::from));
            let beacon: Vec<BlueprintPart> = [[0, 5, 0], [0, 6, 0]]
                .into_iter()
                .map(|at| BlueprintPart {
                    at,
                    block: "minecraft:air".to_string(),
                    state: String::new(),
                })
                .collect();
            made.push((
                Command::PlaceBlueprint {
                    id: ids(),
                    at: site,
                    parts: parts
                        .iter()
                        .map(|(_, p)| BlueprintPart {
                            at: p.at,
                            block: if p.at[1] == 0 {
                                "minecraft:grass_block".to_string()
                            } else {
                                "minecraft:air".to_string()
                            },
                            state: String::new(),
                        })
                        .chain(beacon)
                        .collect(),
                    replace: blocks,
                },
                20,
            ));
            if let Some(door) = door {
                made.push((torch_at(ids, door), 40));
            }
            made
        }
        Beat::AnimalsStare => crate::events::find("face_the_house")
            .map(|event| crate::brain::act::commands(event, None, memory, ids, now, None))
            .unwrap_or_default(),
        Beat::SunBlink => vec![(
            Command::Hint {
                id: ids(),
                player: target,
                hint: "sun_blink".to_string(),
                pos: None,
                seconds: 1.0,
                detail: String::new(),
            },
            0,
        )],
        Beat::NoSleep => vec![(
            Command::CancelSleep {
                id: ids(),
                player: target,
                seconds: 600.0,
            },
            0,
        )],
        Beat::DoorsOpen => {
            let mut made: Vec<(Command, u64)> = memory
                .home
                .as_ref()
                .map(|h| h.doors.clone())
                .unwrap_or_default()
                .into_iter()
                .map(|d| (open(ids, d), 0))
                .collect();
            if let Some(step) = doorstep(memory) {
                let pets: Vec<u32> = memory.pets().map(|p| p.id).collect();
                for (i, pet) in pets.into_iter().enumerate() {
                    made.push((
                        Command::MoveEntity {
                            id: ids(),
                            entity: pet,
                            to: [
                                step[0] as f64 + 0.5 + i as f64 % 3.0,
                                step[1] as f64,
                                step[2] as f64 + 0.5,
                            ],
                        },
                        10,
                    ));
                }
            }
            made
        }
        Beat::FramesOnBed => {
            let (Some(bed), Some(door)) = (
                memory.home.as_ref().and_then(|h| h.bed),
                memory.usual_door(),
            ) else {
                return Vec::new();
            };
            // A line along the bed toward the door, a third of a block apart.
            let (dx, dz) = ((door[0] - bed[0]) as f64, (door[2] - bed[2]) as f64);
            let length = (dx * dx + dz * dz).sqrt().max(1.0);
            let frames: Vec<u32> = memory.decor("item_frame").map(|d| d.id).collect();
            frames
                .into_iter()
                .enumerate()
                .map(|(i, frame)| {
                    let along = i as f64 * 0.35;
                    (
                        Command::SetFrameItem {
                            id: ids(),
                            entity: frame,
                            item: ItemStack::default(),
                            from: None,
                            slot: 0,
                            swap: false,
                            drop_at: Some([
                                bed[0] as f64 + 0.5 + dx / length * along,
                                bed[1] as f64 + 0.7,
                                bed[2] as f64 + 0.5 + dz / length * along,
                            ]),
                        },
                        i as u64 * 2,
                    )
                })
                .collect()
        }
        Beat::SignsSay => {
            let word = match memory.want.as_str() {
                "a place of its own" => "here",
                "to be looked at" => "again",
                _ => "yours",
            };
            let signs: Vec<BlockPos> = memory.fixtures("sign").map(|f| f.pos).collect();
            signs
                .into_iter()
                .map(|sign| {
                    (
                        Command::SetSign {
                            id: ids(),
                            one_line: true,
                            pos: sign,
                            lines: vec![word.to_string()],
                            settle: 0,
                        },
                        0,
                    )
                })
                .collect()
        }
        Beat::FigureBehind(blocks) => {
            let yaw = (memory.last.yaw as f64).to_radians();
            let spot = [
                at[0] + yaw.sin() * blocks as f64,
                -10_000.0,
                at[2] - yaw.cos() * blocks as f64,
            ];
            vec![(figure(ids, memory, spot, 30.0, 16.0, false), 0)]
        }
        Beat::TunnelsSealed => {
            let mouths: Vec<BlockPos> = memory.real_tunnels().map(|t| t.mouth()).collect();
            let mut made = Vec::new();
            for (i, m) in mouths.into_iter().enumerate() {
                for (k, y) in [m[1], m[1] + 1].into_iter().enumerate() {
                    made.push((
                        Command::SetBlock {
                            id: ids(),
                            pos: [m[0], y, m[2]],
                            block: "minecraft:cobblestone".to_string(),
                            state: String::new(),
                            replace: vec![],
                            settle: 0,
                            wall: String::new(),
                            rest: false,
                            only: false,
                        },
                        (i * 2 + k) as u64 * 10,
                    ));
                }
            }
            made
        }
        Beat::BellRings => memory
            .scanned("bell")
            .next()
            .map(|bell| {
                vec![(
                    Command::PlaySound {
                        id: ids(),
                        player: target,
                        sound: "minecraft:block.bell.use".to_string(),
                        pos: center(bell),
                        volume: 2.0,
                        pitch: 1.0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Beat::LightsTurn => {
            let mut made: Vec<(Command, u64)> = memory
                .its_lights
                .clone()
                .into_iter()
                .map(|p| (torch_at(ids, p), 0))
                .collect();
            let theirs: Vec<BlockPos> = memory
                .home
                .as_ref()
                .map(|h| h.lights.clone())
                .unwrap_or_default();
            for light in theirs {
                made.push((
                    Command::HideBlock {
                        id: ids(),
                        pos: light,
                        seconds: 300.0,
                        only: LIGHTS.iter().map(|l| l.to_string()).collect(),
                    },
                    0,
                ));
            }
            made.push((
                Command::Hint {
                    id: ids(),
                    player: target,
                    hint: "red_rain".to_string(),
                    pos: None,
                    seconds: 600.0,
                    detail: String::new(),
                },
                0,
            ));
            made
        }
        Beat::FigureComes => {
            let from = memory
                .place
                .map(center)
                .unwrap_or([at[0] + 30.0, at[1], at[2]]);
            let (dx, dz) = (at[0] - from[0], at[2] - from[2]);
            let length = (dx * dx + dz * dz).sqrt();
            // A point every two blocks, a step each half second: a player's walk.
            let steps = ((length - 1.0) / 2.0).max(0.0) as usize;
            let path: Vec<Pos> = (0..=steps)
                .map(|i| {
                    let t = (i as f64 * 2.0).min(length - 1.0) / length.max(1e-6);
                    [from[0] + dx * t, -10_000.0, from[2] + dz * t]
                })
                .collect();
            let mut comes = figure(
                ids,
                memory,
                [from[0], -10_000.0, from[2]],
                steps as f32 * 0.5 + 15.0,
                0.0,
                false,
            );
            if let Command::Figure { path: p, held, .. } = &mut comes {
                *p = path;
                *held = memory
                    .held
                    .as_ref()
                    .map(|h| h.item.clone())
                    .unwrap_or_default();
            }
            vec![(comes, 0)]
        }
        Beat::Death => {
            vec![(
                Command::KillPlayer {
                    id: ids(),
                    player: target,
                },
                0,
            )]
        }
        Beat::DoorsShut => memory
            .home
            .as_ref()
            .map(|h| h.doors.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|door| {
                (
                    Command::SetState {
                        id: ids(),
                        pos: door,
                        property: "open".to_string(),
                        value: "false".to_string(),
                    },
                    0,
                )
            })
            .collect(),
        Beat::SecondRoom => match crate::events::find("house_grows") {
            Some(event) if memory.next_build("house_grows").is_some() => {
                let made = crate::brain::act::commands(event, None, memory, ids, now, None);
                // Laid as a plan's share is, so the room counts as built and is not sent again.
                for (command, _) in &made {
                    crate::building::account(memory, "house_grows", command, now.day as f32);
                }
                made
            }
            _ => Vec::new(),
        },
        Beat::FigureInItsWindow => {
            let Some(pane) = its(memory, "minecraft:glass_pane", "") else {
                return Vec::new();
            };
            let site = memory.build("house_near").map(|b| b.site).unwrap_or(pane);
            // A block in from the pane, toward the middle of its house.
            let inside = [
                pane[0] + (site[0] - pane[0]).signum(),
                pane[1] - 1,
                pane[2] + (site[2] - pane[2]).signum(),
            ];
            vec![(figure(ids, memory, center(inside), 50.0, 16.0, false), 0)]
        }
        Beat::CobblePath => {
            let (Some(from), Some((_, to))) = (doorstep(memory), its_door(memory)) else {
                return Vec::new();
            };
            let mut p = [from[0], from[1] - 1, from[2]];
            let mut made = Vec::new();
            for i in 0..128 {
                let (dx, dz) = (to[0] - p[0], to[2] - p[2]);
                if dx == 0 && dz == 0 {
                    break;
                }
                let step = if dx.abs() >= dz.abs() {
                    [dx.signum(), 0]
                } else {
                    [0, dz.signum()]
                };
                p = [p[0] + step[0], p[1], p[2] + step[1]];
                made.push((
                    Command::SetBlock {
                        id: ids(),
                        pos: p,
                        block: "minecraft:cobblestone".to_string(),
                        state: String::new(),
                        replace: vec![
                            "minecraft:grass_block".to_string(),
                            "minecraft:dirt".to_string(),
                            "minecraft:dirt_path".to_string(),
                            "minecraft:coarse_dirt".to_string(),
                            "minecraft:podzol".to_string(),
                        ],
                        settle: 3,
                        wall: String::new(),
                        rest: false,
                        only: true,
                    },
                    i as u64 * 4,
                ));
            }
            made
        }
        Beat::PetsToItsDoor => {
            let Some((_, out)) = its_door(memory) else {
                return Vec::new();
            };
            let pets: Vec<u32> = memory.pets().map(|p| p.id).collect();
            pets.into_iter()
                .enumerate()
                .map(|(i, pet)| {
                    (
                        Command::MoveEntity {
                            id: ids(),
                            entity: pet,
                            to: [
                                out[0] as f64 + 0.5 + (i % 3) as f64 - 1.0,
                                out[1] as f64,
                                out[2] as f64 + 0.5,
                            ],
                        },
                        0,
                    )
                })
                .collect()
        }
        Beat::PetsHome => {
            let Some(step) = doorstep(memory) else {
                return Vec::new();
            };
            let pets: Vec<u32> = memory.pets().map(|p| p.id).collect();
            pets.into_iter()
                .map(|pet| {
                    (
                        Command::MoveEntity {
                            id: ids(),
                            entity: pet,
                            to: center(step),
                        },
                        0,
                    )
                })
                .collect()
        }
        Beat::DoorwayLit => {
            let Some((door, out)) = its_door(memory) else {
                return Vec::new();
            };
            let way = memory.build("house_near").map(|b| b.way).unwrap_or([0, 1]);
            let beside = [out[0] + way[1], out[1], out[2] - way[0]];
            vec![
                (open(ids, door), 0),
                (torch_at(ids, beside), 2),
                (figure(ids, memory, center(door), 600.0, 0.0, true), 4),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::Build;
    use crate::memory::{Container, Creature, Decor, Fixture, Home};

    /// A home with a bed, a door, a light, a sign, a frame, a pet, its own house built, its
    /// torches, a tunnel and a bell.
    fn lived_in() -> Memory {
        let mut memory = Memory {
            target: Some(0),
            target_name: "Player612".to_string(),
            want: "one thing of theirs".to_string(),
            ..Memory::default()
        };
        memory.home = Some(Home {
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![[1, 65, 1]],
        });
        memory.habits.doors.push(([3, 64, 0], 5));
        memory.fixtures.push(Fixture {
            kind: "sign".to_string(),
            pos: [2, 65, 2],
        });
        memory.decor.push(Decor {
            kind: "item_frame".to_string(),
            id: 70,
            pos: [-1, 65, 0],
        });
        memory.last.creatures_near.push(Creature {
            id: 90,
            kind: "wolf".to_string(),
            pos: [1.0, 64.0, 1.0],
            named: true,
            pet: true,
        });
        memory.builds.push(Build {
            kind: "house_near".to_string(),
            site: [56, 63, 0],
            way: [-1, 0],
            laid: crate::building::house([-1, 0], false).len(),
            grown: false,
            last_day: 1.0,
        });
        memory.its_lights = vec![[10, 64, 0], [30, 64, 0], [50, 64, 0]];
        memory.scanned = vec![("bell".to_string(), [100, 64, 100])];
        for x in 0..12 {
            memory.dug_underground([x, 21, 40], 1.0);
            memory.dug_underground([x, 20, 40], 1.0);
        }
        memory.last.online = true;
        memory.last.pos = [0.5, 64.0, 0.5];
        memory
    }

    /// Runs the script from its first dawn to its end, a tick every hundred, and answers every
    /// command sent with its day.
    fn run(ending: Ending, memory: &mut Memory) -> (Arc, Vec<(u64, Command)>) {
        let config = Config::default();
        let mut arc = Arc {
            stage: 5,
            ending: ending.name().to_string(),
            script_day: 10,
            ..Arc::default()
        };
        let mut sent = Vec::new();
        let mut n = 0;
        for t in (10 * 24_000u64..14 * 24_000).step_by(100) {
            let now = Now {
                tick: t,
                day: t / 24_000,
                day_time: (t % 24_000) as u32,
                ..Now::default()
            };
            let mut ids = || {
                n += 1;
                n
            };
            for (command, _) in tick(&mut arc, memory, &config, &now, &mut ids) {
                sent.push((now.day - 10, command));
            }
            if arc.over() {
                break;
            }
        }
        (arc, sent)
    }

    #[test]
    fn the_ending_is_the_config_s_or_the_want_s_and_the_difficulty_s() {
        let mut config = Config::default();
        let mut memory = lived_in();
        memory.difficulty = "hard".to_string();
        for (name, ending) in [
            ("leave", Ending::Leave),
            ("take", Ending::Take),
            ("keep", Ending::Keep),
        ] {
            config.stages.ending = name.to_string();
            assert_eq!(choose(&config, &memory), ending);
        }
        config.stages.ending = "difficulty".to_string();
        assert_eq!(choose(&config, &memory), Ending::Take, "refused on hard");
        memory.difficulty = "normal".to_string();
        assert_eq!(choose(&config, &memory), Ending::Take);
        memory.difficulty = "easy".to_string();
        assert_eq!(choose(&config, &memory), Ending::Keep, "refused on easy");
        memory.difficulty = "peaceful".to_string();
        assert_eq!(choose(&config, &memory), Ending::Keep);
        // Their favourite left in the chest it uses: met, on any difficulty.
        memory.favourites = vec![("minecraft:clock".to_string(), 9)];
        memory.containers.push(Container {
            pos: [-2, 64, 1],
            placed: true,
            opens: 5,
            contents: vec![ItemStack {
                id: "minecraft:clock".to_string(),
                count: 1,
                slot: 3,
            }],
            ..Container::default()
        });
        memory.difficulty = "hard".to_string();
        assert!(want_met(&memory));
        assert_eq!(choose(&config, &memory), Ending::Leave);
    }

    #[test]
    fn the_other_wants_are_met_by_what_they_do() {
        let mut memory = lived_in();
        memory.want = "a place of its own".to_string();
        memory.place = Some([30, 64, 30]);
        assert!(!want_met(&memory));
        memory.placed = vec![[30, 64, 31], [31, 64, 30], [29, 65, 30]];
        assert!(want_met(&memory), "they built there");

        memory.want = "to be looked at".to_string();
        assert!(!want_met(&memory));
        for i in 0..3 {
            memory.reactions.push(crate::memory::Reaction {
                event: format!("figure_{i}"),
                effect: "seen".to_string(),
                looked: true,
                ..Default::default()
            });
        }
        assert!(want_met(&memory), "looked at three times, never run from");
        memory.reactions[0].ran = true;
        assert!(!want_met(&memory));
    }

    #[test]
    fn it_leaves_its_torches_out_its_chest_on_their_step_its_house_gone_a_torch_where_its_door_was()
    {
        let mut memory = lived_in();
        let (arc, sent) = run(Ending::Leave, &mut memory);
        assert_eq!(arc.stage, 6);
        assert_eq!(arc.ended, "left");
        // Day one: its torches out, the farthest from the house first.
        let out: Vec<BlockPos> = sent
            .iter()
            .filter(|(d, _)| *d == 0)
            .filter_map(|(_, c)| match c {
                Command::SetBlock {
                    pos,
                    block,
                    only: true,
                    ..
                } if block == "minecraft:air" => Some(*pos),
                _ => None,
            })
            .collect();
        assert_eq!(out, vec![[50, 64, 0], [30, 64, 0], [10, 64, 0]]);
        // Day two: its chest onto their doorstep (out from the door, away from the house).
        assert!(sent.iter().any(|(d, c)| *d == 1
            && matches!(c, Command::SpillContainer { to, remove: false, .. } if to[0] == 4.5)));
        // The last night: the music, the flicker, the figure at the edge of the light.
        assert!(sent.iter().any(|(d, c)| *d == 2
            && matches!(c, Command::PlaySound { sound, .. } if sound == "minecraft:music.game")));
        assert!(sent.iter().any(|(d, c)| *d == 2
            && matches!(
                c,
                Command::Figure {
                    once_seen: true,
                    ..
                }
            )));
        // At dawn: its house to air and its floor to grass, the chest taken, a torch at the door.
        let gone = sent.iter().find_map(|(_, c)| match c {
            Command::PlaceBlueprint { parts, replace, .. }
                if parts.iter().any(|p| p.block == "minecraft:grass_block") =>
            {
                Some((parts.clone(), replace.clone()))
            }
            _ => None,
        });
        let (parts, replace) = gone.expect("its house gone");
        assert!(parts
            .iter()
            .all(|p| (p.at[1] == 0) == (p.block == "minecraft:grass_block")));
        assert!(replace.contains(&"minecraft:cobblestone".to_string()));
        // Its beacon on its roof goes with it, not left in the air.
        assert!(
            replace.contains(&"minecraft:beacon".to_string())
                && replace.contains(&"minecraft:iron_block".to_string())
        );
        assert!(parts
            .iter()
            .any(|p| p.at == [0, 5, 0] && p.block == "minecraft:air"));
        assert!(sent
            .iter()
            .any(|(_, c)| matches!(c, Command::SpillContainer { remove: true, .. })));
        let door = its_door(&memory).unwrap().0;
        assert!(sent.iter().any(|(_, c)| matches!(c, Command::SetBlock { pos, block, .. } if *pos == door && block == "minecraft:torch")));
        assert!(memory.its_lights.is_empty());
    }

    #[test]
    fn it_takes_with_no_sleep_the_doors_open_the_frames_on_the_bed_and_on_hard_one_death() {
        let mut memory = lived_in();
        memory.difficulty = "normal".to_string();
        let (arc, sent) = run(Ending::Take, &mut memory);
        assert_eq!(arc.ended, "took");
        assert!(sent
            .iter()
            .any(|(d, c)| *d == 0 && matches!(c, Command::CancelSleep { .. })));
        assert!(sent.iter().any(|(d, c)| *d == 0
            && matches!(c, Command::SetState { pos, value, .. } if *pos == [3, 64, 0] && value == "true")));
        assert_eq!(
            sent.iter()
                .filter(|(d, c)| *d == 0
                    && matches!(c, Command::Hint { hint, .. } if hint == "sun_blink"))
                .count(),
            3,
            "the sun out three times"
        );
        assert!(sent.iter().any(|(d, c)| *d == 1
            && matches!(
                c,
                Command::SetFrameItem {
                    entity: 70,
                    drop_at: Some(_),
                    ..
                }
            )));
        assert!(sent.iter().any(|(d, c)| *d == 1
            && matches!(c, Command::SetSign { lines, .. } if lines == &vec!["yours".to_string()])));
        assert_eq!(
            sent.iter()
                .filter(|(_, c)| matches!(c, Command::PlaySound { sound, .. } if sound == "minecraft:block.bell.use"))
                .count(),
            11,
            "the bell every hour of the day"
        );
        assert!(sent.iter().any(|(d, c)| *d == 1 && matches!(c, Command::SetBlock { block, pos, .. } if block == "minecraft:cobblestone" && pos[2] == 40)));
        assert!(
            sent.iter()
                .any(|(d, c)| *d == 2
                    && matches!(c, Command::Hint { hint, .. } if hint == "red_rain"))
        );
        assert!(sent
            .iter()
            .any(|(d, c)| *d == 2 && matches!(c, Command::Figure { path, .. } if path.len() > 3)));
        assert!(
            !sent
                .iter()
                .any(|(_, c)| matches!(c, Command::KillPlayer { .. })),
            "no death on normal"
        );
        assert!(
            sent.iter().any(|(d, c)| *d == 2
                && matches!(c, Command::SetState { pos, value, .. } if *pos == [3, 64, 0] && value == "false")),
            "the doors shut again at the end"
        );

        assert!(!deals_death(&Config::default(), &memory));
        assert!(deals_death(
            &crate::config::Preset::Intense.config(),
            &memory
        ));
        let mut easy = lived_in();
        easy.difficulty = "easy".to_string();
        assert!(!deals_death(
            &crate::config::Preset::Intense.config(),
            &easy
        ));
        assert!(deals_death(
            &crate::config::Preset::Relentless.config(),
            &easy
        ));

        let mut hard = lived_in();
        hard.difficulty = "hard".to_string();
        let (_, sent) = run(Ending::Take, &mut hard);
        assert_eq!(
            sent.iter()
                .filter(|(_, c)| matches!(c, Command::KillPlayer { .. }))
                .count(),
            1,
            "once, on hard"
        );
    }

    #[test]
    fn it_keeps_its_window_watched_a_path_to_its_door_and_its_doorway_lit_then_a_door_now_and_then()
    {
        let mut memory = lived_in();
        let (mut arc, sent) = run(Ending::Keep, &mut memory);
        assert_eq!(arc.ended, "stayed");
        assert!(
            sent.iter()
                .any(|(d, c)| *d == 0 && matches!(c, Command::PlaceBlueprint { .. })),
            "the second room"
        );
        assert_eq!(
            sent.iter()
                .filter(|(d, c)| *d == 0 && matches!(c, Command::Figure { .. }))
                .count(),
            5,
            "in its window through the night"
        );
        let path: Vec<BlockPos> = sent
            .iter()
            .filter_map(|(_, c)| match c {
                Command::SetBlock { pos, block, .. }
                    if block == "minecraft:cobblestone" && pos[2] == 0 =>
                {
                    Some(*pos)
                }
                _ => None,
            })
            .collect();
        assert!(path.len() > 40, "door to door: {}", path.len());
        let (door, _) = its_door(&memory).unwrap();
        assert!(sent
            .iter()
            .any(|(d, c)| *d == 2 && matches!(c, Command::SetState { pos, .. } if *pos == door)));

        // Three weeks on, while they are away, a door of theirs open.
        memory.last.online = false;
        let early = Now {
            day: arc.next_open_day - 1,
            ..Now::default()
        };
        assert!(after(&mut arc, &memory, &early, &mut || 1).is_empty());
        let due = Now {
            day: arc.next_open_day,
            ..Now::default()
        };
        let opened = after(&mut arc, &memory, &due, &mut || 1);
        assert!(matches!(&opened[0], Command::SetState { pos, .. } if *pos == [3, 64, 0]));
        assert!(
            after(&mut arc, &memory, &due, &mut || 1).is_empty(),
            "and not again for weeks"
        );
    }

    #[test]
    fn a_moment_that_comes_late_is_skipped_and_a_change_to_the_world_is_done_late() {
        let config = Config::default();
        let mut memory = lived_in();
        let mut arc = Arc {
            stage: 5,
            ending: "leave".to_string(),
            script_day: 10,
            ..Arc::default()
        };
        // The game was closed over the second day: it opens on the third's noon.
        let now = Now {
            tick: 12 * 24_000 + 6_000,
            day: 12,
            day_time: 6_000,
            ..Now::default()
        };
        let mut n = 0;
        let mut sent = Vec::new();
        for _ in 0..LEAVES.len() {
            let mut ids = || {
                n += 1;
                n
            };
            sent.extend(tick(&mut arc, &mut memory, &config, &now, &mut ids));
        }
        assert!(
            sent.iter()
                .any(|(c, _)| matches!(c, Command::SpillContainer { .. })),
            "its chest, late"
        );
        assert!(
            !sent
                .iter()
                .any(|(c, _)| matches!(c, Command::Figure { .. })),
            "the figure at its door at yesterday's noon: gone"
        );
        assert_eq!(
            arc.beats_done, 6,
            "up to the third day's beats, which wait for their hour"
        );
    }

    fn at(day: u64, day_time: u32) -> Now {
        Now {
            tick: day * 24_000 + day_time as u64,
            day,
            day_time,
            ..Now::default()
        }
    }

    #[test]
    fn knocking_on_its_door_at_night_begins_the_fight_and_its_end_ends_the_stay() {
        let config = Config::default();
        let mut memory = lived_in();
        memory.store("minecraft:cobblestone", 40);
        let mut arc = Arc {
            stage: 5,
            ending: "keep".to_string(),
            script_day: 10,
            ..Arc::default()
        };
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        let (door, _) = its_door(&memory).unwrap();
        // By day a knock is a knock.
        memory.knocked_now = Some(door);
        let day = tick(&mut arc, &mut memory, &config, &at(10, 6_000), &mut ids);
        assert!(!day
            .iter()
            .any(|(c, _)| matches!(c, Command::Confront { .. })));
        // At night, on its door: the fight.
        memory.knocked_now = Some(door);
        let night = tick(&mut arc, &mut memory, &config, &at(10, 15_000), &mut ids);
        let (_, out) = its_door(&memory).unwrap();
        assert!(
            matches!(&night[0].0, Command::Confront { at, blocks, .. }
                if *at == out && blocks[0] == "minecraft:cobblestone" && blocks.last().unwrap() == "minecraft:torch"),
            "{night:?}"
        );
        assert!(arc.fighting_since > 0);
        assert_eq!(memory.knocked_now, None, "the knock taken");
        // While it is on, the script waits.
        let done = arc.beats_done;
        tick(&mut arc, &mut memory, &config, &at(11, 2_000), &mut ids);
        assert_eq!(arc.beats_done, done);
        // Run from: it can be come to again.
        memory.fight_ended = Some("fled".to_string());
        tick(&mut arc, &mut memory, &config, &at(11, 2_100), &mut ids);
        assert_eq!(arc.fighting_since, 0);
        assert!(!arc.over());
        // Won: its chest on their step, held no more, the stay over.
        memory.held = Some(crate::memory::Held::default());
        arc.fighting_since = 5;
        memory.fight_ended = Some("won".to_string());
        let won = tick(&mut arc, &mut memory, &config, &at(11, 20_200), &mut ids);
        assert!(won
            .iter()
            .any(|(c, _)| matches!(c, Command::SpillContainer { .. })));
        assert!(memory.held.is_none());
        assert_eq!((arc.stage, arc.ended.as_str()), (6, "beaten"));

        // Lost: over, and it kept what it took.
        let mut lost = Arc {
            stage: 5,
            ending: "take".to_string(),
            fighting_since: 5,
            ..Arc::default()
        };
        memory.fight_ended = Some("lost".to_string());
        tick(&mut lost, &mut memory, &config, &at(11, 20_300), &mut ids);
        assert_eq!((lost.stage, lost.ended.as_str()), (6, "kept"));
    }

    #[test]
    fn with_no_house_crouching_at_its_place_calls_it_and_walking_past_does_not() {
        let config = Config::default();
        let mut memory = lived_in();
        memory.builds.clear();
        memory.place = Some([20, 64, 20]);
        memory.last.pos = [20.5, 64.0, 20.5];
        let mut arc = Arc {
            stage: 5,
            ending: "keep".to_string(),
            script_day: 10,
            ..Arc::default()
        };
        let mut ids = || 1;
        let walked = tick(&mut arc, &mut memory, &config, &at(10, 15_000), &mut ids);
        assert!(!walked
            .iter()
            .any(|(c, _)| matches!(c, Command::Confront { .. })));
        memory.last.sneaking = true;
        let called = tick(&mut arc, &mut memory, &config, &at(10, 15_100), &mut ids);
        assert!(called
            .iter()
            .any(|(c, _)| matches!(c, Command::Confront { at, .. } if *at == [20, 64, 20])));
    }

    #[test]
    fn its_place_is_met_by_what_they_build_there_after_it_chose_it() {
        let mut memory = lived_in();
        memory.want = "a place of its own".to_string();
        memory.place = Some([30, 64, 30]);
        memory.placed = vec![[30, 64, 31], [31, 64, 30], [29, 65, 30]];
        memory.placed_at_place = memory.placed.clone();
        assert!(!want_met(&memory), "built there before it chose it");
        memory
            .placed
            .extend([[30, 65, 31], [31, 65, 30], [29, 66, 30]]);
        assert!(want_met(&memory));
    }

    #[test]
    fn its_beacon_is_built_on_its_roof_lit_at_dusk_and_capped_at_dawn() {
        let config = Config::default();
        let mut memory = lived_in();
        let mut arc = Arc {
            stage: 5,
            ending: "keep".to_string(),
            script_day: 10,
            ..Arc::default()
        };
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        let base = beacon_base(&memory).unwrap();
        assert_eq!(base, [56, 67, 0], "the middle of its roof");
        let built = tick(&mut arc, &mut memory, &config, &at(10, 12_100), &mut ids);
        assert!(
            matches!(&built[0].0, Command::PlaceBlueprint { at, parts, .. }
                if *at == base && parts.len() == 10 && parts.iter().any(|p| p.block == "minecraft:beacon" && p.at == [0, 1, 0])),
            "{built:?}"
        );
        assert!(arc.beacon_built);
        let capped = tick(&mut arc, &mut memory, &config, &at(11, 200), &mut ids);
        assert!(matches!(&capped[0].0, Command::SetBlock { pos, block, .. }
            if *pos == [56, 69, 0] && block == "minecraft:cobblestone"));
        let lit = tick(&mut arc, &mut memory, &config, &at(11, 12_200), &mut ids);
        assert!(
            matches!(&lit[0].0, Command::SetBlock { pos, block, only: true, .. }
            if *pos == [56, 69, 0] && block == "minecraft:air")
        );
        // Built once only.
        let again = tick(&mut arc, &mut memory, &config, &at(11, 12_300), &mut ids);
        assert!(
            !again
                .iter()
                .any(|(c, _)| matches!(c, Command::PlaceBlueprint { parts, .. }
                if parts.iter().any(|p| p.block == "minecraft:beacon"))),
            "built once only: {again:?}"
        );
    }

    #[test]
    fn stage_five_chooses_its_ending_on_its_first_tick_and_waits_for_the_next_dawn() {
        let mut config = Config::default();
        config.stages.ending = "keep".to_string();
        let mut memory = lived_in();
        let mut arc = Arc {
            stage: 5,
            ..Arc::default()
        };
        let now = Now {
            day: 7,
            day_time: 9_000,
            ..Now::default()
        };
        assert!(tick(&mut arc, &mut memory, &config, &now, &mut || 1).is_empty());
        assert_eq!(arc.ending, "keep");
        assert_eq!(arc.script_day, 8, "the next dawn");
        // The arc never cuts a running ending short.
        assert_eq!(arc.tick(400.0, &memory, &config), None);
        assert_eq!(arc.stage, 5);
    }
}
