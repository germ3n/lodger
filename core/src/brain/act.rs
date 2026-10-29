//! From a decided event to the host's commands, resolved against the memory at the moment it
//! runs. Each action here is one of the general commands the host carries out; the event's
//! `fits` is the host's check when the command arrives, and `ready` is the mind's check that the
//! moment has come.

use crate::brain::text::{self, Kind};
use crate::events::{Action, Event, FigureWhere, Need, Where};
use crate::memory::distance;
use crate::memory::{Held, Memory};
use crate::model::Rng;
use craftkit::world::{BlockPos, Command, ItemStack, Now, Pos};

/// Whether the event's moment is here: its needs still hold.
pub fn ready(event: &Event, memory: &Memory, _config: &crate::config::Config) -> bool {
    let now = Now::default();
    event.needs.iter().all(|need| {
        matches!(*need, Need::Night | Need::Day | Need::Dawn)
            || super::offer::meets(*need, memory, &now)
    })
}

/// Whether an item id is something a furnace turns into something else.
pub fn smeltable(id: &str) -> bool {
    id.starts_with("minecraft:raw_")
        || id.ends_with("_ore")
        || matches!(
            id,
            "minecraft:beef"
                | "minecraft:porkchop"
                | "minecraft:chicken"
                | "minecraft:mutton"
                | "minecraft:rabbit"
                | "minecraft:cod"
                | "minecraft:salmon"
                | "minecraft:potato"
                | "minecraft:kelp"
                | "minecraft:cobblestone"
                | "minecraft:sand"
                | "minecraft:red_sand"
                | "minecraft:clay_ball"
                | "minecraft:cactus"
                | "minecraft:netherrack"
                | "minecraft:wet_sponge"
        )
}

/// Whether an item id burns in a furnace.
pub fn fuel(id: &str) -> bool {
    id.ends_with("_log")
        || id.ends_with("_planks")
        || id.ends_with("_wood")
        || id.ends_with("_sapling")
        || matches!(
            id,
            "minecraft:coal"
                | "minecraft:charcoal"
                | "minecraft:coal_block"
                | "minecraft:stick"
                | "minecraft:blaze_rod"
                | "minecraft:lava_bucket"
                | "minecraft:dried_kelp_block"
                | "minecraft:bamboo"
        )
}

fn block_center(p: BlockPos) -> Pos {
    [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5]
}

fn block_of(p: Pos) -> BlockPos {
    [
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    ]
}

/// The lodger's spot: its chosen place, or a point eight to twelve blocks from the home's
/// center, away from the door, picked once from the seed so it stays the same spot.
fn spot(memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    if let Some(place) = memory.place {
        return Some(place);
    }
    let home = memory.home.as_ref()?;
    let angle = (rng.next_u64() % 8) as f64 * std::f64::consts::FRAC_PI_4;
    let distance = 8.0 + (rng.next_u64() % 5) as f64;

    Some([
        home.center[0] + (distance * angle.cos()).round() as i32,
        home.floor_y,
        home.center[2] + (distance * angle.sin()).round() as i32,
    ])
}

/// How far from anything the target placed the lodger digs: never nearer
/// (`docs/good-to-know.md`, What it never does).
pub const DIG_CLEAR: f64 = 8.0;

/// A spot to dig, clear of everything the target placed by [`DIG_CLEAR`]: its place if that is
/// clear, else the first clear one of eight around the home, 12 to 20 blocks out. None when the
/// target has built all around: the event is not offered. The dev client's `hole` went in
/// under a crafting table at the lodger's place, which the target had built up to (2026-11-09).
fn dig_spot(memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    if let Some(place) = memory.place.filter(|p| clear_of_built(memory, *p)) {
        return Some(place);
    }
    let home = memory.home.as_ref()?;
    let first = rng.next_u64() % 8;
    let distance = 12.0 + (rng.next_u64() % 9) as f64;
    (0..8)
        .map(|i| {
            let angle = ((first + i) % 8) as f64 * std::f64::consts::FRAC_PI_4;
            [
                home.center[0] + (distance * angle.cos()).round() as i32,
                home.floor_y,
                home.center[2] + (distance * angle.sin()).round() as i32,
            ]
        })
        .find(|p| clear_of_built(memory, *p))
}

/// Whether nothing the target placed is within [`DIG_CLEAR`] of `pos`.
pub fn clear_of_built(memory: &Memory, pos: BlockPos) -> bool {
    memory.placed.iter().all(|p| {
        let (dx, dy, dz) = (
            (p[0] - pos[0]) as f64,
            (p[1] - pos[1]) as f64,
            (p[2] - pos[2]) as f64,
        );
        (dx * dx + dy * dy + dz * dz).sqrt() > DIG_CLEAR
    })
}

/// The block a command digs out of natural ground, if it digs: air set where only ground may
/// be replaced. A path worn into grass or water poured on a floor is not digging.
pub fn dug(command: &Command) -> Option<BlockPos> {
    match command {
        Command::SetBlock {
            pos,
            block,
            replace,
            ..
        } if block == "minecraft:air"
            && replace.iter().any(|r| SOLID_GROUND.contains(&r.as_str())) =>
        {
            Some(*pos)
        }
        _ => None,
    }
}

/// The place the event happens, from the memory, or none for a thing with no place.
pub fn place(event: &Event, memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    let home = memory.home.as_ref();
    // The house's sounds come from a real door or chest of the house they are in, theirs
    // first (owner, 2026-11-09: "any house, yours first").
    if matches!(
        event.id,
        "door_sound" | "chest_sound" | "knock" | "knock_heavy" | "lever_click" | "your_voice"
    ) {
        return super::sounds::house_spot(event.id, memory, rng.next_u64() as usize);
    }

    match event.action {
        Action::Wait
        | Action::Hint(_, _)
        | Action::RefuseRest
        | Action::Stare
        | Action::SoundAtTarget(_)
        | Action::SoundFar(_)
        | Action::TabEntry
        | Action::Message(_)
        | Action::Says
        | Action::SaysKills
        | Action::Whisper
        | Action::RepeatsYou
        | Action::Answers
        | Action::HostilesStare
        | Action::NameAnimal
        | Action::DyeSheep
        | Action::GolemTurned
        | Action::StandWatch
        | Action::FramesTurn
        | Action::ToggleCandles
        | Action::CarpetShift
        | Action::LightPath
        | Action::MoveVehicle
        | Action::PushVehicle(_)
        | Action::MiningSounds
        | Action::VillageDoors
        | Action::ScanVillage
        | Action::ScanOre
        | Action::FrameEmptied
        | Action::FrameSwap
        | Action::StandDress
        | Action::ToolUsed
        | Action::ColdFurnace
        | Action::TidyByColour
        | Action::IcePath
        | Action::BedOccupied => None,
        Action::OreMarked => memory.scanned("_ore").next(),
        Action::WallSwap(suffix) => {
            let candidates: Vec<BlockPos> = memory
                .walls
                .iter()
                .filter(|(_, id)| suffix.is_empty() || id.ends_with(suffix))
                .map(|(p, _)| *p)
                .collect();
            pick_ref(&candidates, rng)
        }
        Action::BedBreaks => home.and_then(|h| h.bed),
        Action::Figure(FigureWhere::Roof, _, _) => {
            home.map(|h| [h.center[0], h.floor_y + 5, h.center[2]])
        }
        Action::Figure(FigureWhere::Doorway, _, _) => home.and_then(|h| pick(&h.doors, rng)),
        Action::Figure(FigureWhere::AtChest, _, _) => memory.used_container().map(|c| c.pos),
        Action::Figure(FigureWhere::InBed, _, _) => home.and_then(|h| h.bed),
        Action::Figure(FigureWhere::AcrossWater, _, _) => memory.feature("water").map(|w| {
            // The far side: past the water's middle from the target, by its width.
            let at = block_of(memory.last.pos);
            let step = step_toward(at, w.pos);
            let reach = w.size as i32 / 2 + 2;
            [
                w.pos[0] + step[0] * reach,
                -10_000,
                w.pos[2] + step[1] * reach,
            ]
        }),
        Action::Figure(FigureWhere::Behind, _, _) => memory.route().map(|r| {
            // The route's end farther from where they stand now.
            let at = block_of(memory.last.pos);
            let end = if distance(r.start(), at) > distance(r.end(), at) {
                r.start()
            } else {
                r.end()
            };
            [end[0], -10_000, end[2]]
        }),
        Action::Figure(FigureWhere::Route, _, _) => memory
            .route()
            .map(|r| [r.start()[0], -10_000, r.start()[2]]),
        Action::Figure(FigureWhere::Far, _, _) => {
            // Fifty to seventy blocks from the target, away from the home.
            let at = block_of(memory.last.pos);
            let from = home.map(|h| h.center).unwrap_or(at);
            let dx = (at[0] - from[0]) as f64;
            let dz = (at[2] - from[2]) as f64;
            let length = (dx * dx + dz * dz).sqrt();
            let (ux, uz) = if length < 1.0 {
                let angle = rng.next_f64() * std::f64::consts::TAU;
                (angle.cos(), angle.sin())
            } else {
                (dx / length, dz / length)
            };
            let reach = 50.0 + (rng.next_u64() % 21) as f64;
            Some([
                at[0] + (ux * reach).round() as i32,
                -10_000,
                at[2] + (uz * reach).round() as i32,
            ])
        }
        Action::RotateFixture(kind) => pick_ref(
            memory
                .fixtures(kind)
                .map(|f| f.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::SoundPair(kind, _, _) => pick_ref(
            memory
                .fixtures(kind)
                .map(|f| f.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::OpenOnEnter => memory.used_container().map(|c| c.pos),
        Action::PlantFlower => home.and_then(|h| pick(&h.doors, rng)),
        Action::ReplantOther => memory.feature("crops").map(|f| f.pos),
        Action::WokeElsewhere => home.and_then(|h| h.bed),
        Action::BucketFloor => home.map(|h| [h.center[0] + 2, h.floor_y, h.center[2] + 2]),
        Action::SnowCleared | Action::Trample | Action::DoorAhead => {
            home.and_then(|h| pick(&h.doors, rng))
        }
        Action::HarvestOne => memory.feature("crops").map(|f| f.pos),
        Action::PlateLaid => memory.used_container().map(|c| c.pos),
        Action::ToggleDoor | Action::Wake | Action::PathWorn | Action::KillSign => {
            home.and_then(|h| pick(&h.doors, rng))
        }
        Action::ToggleFixture(kind) | Action::SoundAtFixture(kind, _) => pick_ref(
            memory
                .fixtures(kind)
                .map(|f| f.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::Taps(kind, _, _) => {
            if kind.is_empty() {
                home.and_then(|h| pick(&h.doors, rng))
            } else {
                pick_ref(
                    memory
                        .fixtures(kind)
                        .map(|f| f.pos)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    rng,
                )
            }
        }
        Action::Sound(_) => home.map(|h| {
            // Another room: six blocks off the center, a random way.
            let angle = rng.next_f64() * std::f64::consts::TAU;
            [
                h.center[0] + (6.0 * angle.cos()).round() as i32,
                h.floor_y + 1,
                h.center[2] + (6.0 * angle.sin()).round() as i32,
            ]
        }),
        Action::OpenLid
        | Action::ShuffleContainer
        | Action::SplitStack
        | Action::TidyContainer
        | Action::SwapStacks
        | Action::FillGap => memory.used_container().map(|c| c.pos),
        Action::CookOne | Action::TakeOutput => memory.furnace().map(|f| f.pos),
        Action::ItemOnTable => memory.fixtures("crafting_table").next().map(|f| f.pos),
        Action::EggOnBed => home.and_then(|h| h.bed),
        Action::Take => memory.favourite_in_container().map(|(c, _)| c.pos),
        Action::ReturnTaken | Action::ReturnUsed => memory.held.as_ref().map(|h| h.from),
        Action::HopperEmptied => memory.fixtures("hopper").next().map(|f| f.pos),
        Action::ShulkerMoved => memory.fixtures("shulker_box").next().map(|f| f.pos),
        Action::RenameOne | Action::BookWord | Action::MapMarked => {
            memory.used_container().map(|c| c.pos)
        }
        // Another chest of theirs than the one just opened, one of the house's.
        Action::ChestAnswers => {
            let opened = memory.opened_now;
            pick_ref(
                memory
                    .containers
                    .iter()
                    .filter(|c| c.theirs() && !c.is_furnace() && Some(c.pos) != opened)
                    .map(|c| c.pos)
                    .collect::<Vec<_>>()
                    .as_slice(),
                rng,
            )
        }
        Action::PlaceTaken => memory
            .place
            .or_else(|| home.and_then(|h| h.bed))
            .or_else(|| home.map(|h| h.center)),
        Action::MoveLight => home.and_then(|h| pick(&h.lights, rng)),
        Action::AddTorch => home.map(|h| {
            let dx = (rng.next_u64() % 7) as i32 - 3;
            let dz = (rng.next_u64() % 7) as i32 - 3;
            [h.center[0] + dx, h.floor_y, h.center[2] + dz]
        }),
        Action::Hole => dig_spot(memory, rng),
        Action::Cairn | Action::Sapling | Action::Sign | Action::Grave => spot(memory, rng),
        Action::Cauldron => memory.fixtures("cauldron").next().map(|f| f.pos),
        Action::RotateFrame => pick_ref(
            memory
                .decor("item_frame")
                .map(|d| d.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::SwapPainting => pick_ref(
            memory
                .decor("painting")
                .map(|d| d.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::StandTurn => memory.decor("armor_stand").next().map(|d| d.pos),
        Action::PetGrowl => memory.pets().next().map(|p| block_of(p.pos)),
        Action::MoveCreature(_, Where::Home) => home.map(|h| h.center),
        Action::MoveCreature(_, Where::OutsideDoor) => home.and_then(|h| pick(&h.doors, rng)),
        Action::MoveCreature(_, Where::RouteEnd) => memory.route_end(),
        Action::MoveCreature(_, Where::Corner) => {
            home.map(|h| [h.center[0] + 3, h.floor_y, h.center[2] + 3])
        }
    }
}

fn pick<T: Copy>(list: &[T], rng: &mut Rng) -> Option<T> {
    if list.is_empty() {
        None
    } else {
        Some(list[(rng.next_u64() % list.len() as u64) as usize])
    }
}

fn pick_ref(list: &[BlockPos], rng: &mut Rng) -> Option<BlockPos> {
    pick(list, rng)
}

fn sound_at(
    ids: &mut impl FnMut() -> u64,
    target: u32,
    sound: &str,
    pos: Pos,
    volume: f32,
) -> Command {
    Command::PlaySound {
        id: ids(),
        player: target,
        sound: sound.to_string(),
        pos,
        volume,
        pitch: 1.0,
    }
}

fn set(ids: &mut impl FnMut() -> u64, pos: BlockPos, block: &str, replace: &[&str]) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: block.to_string(),
        state: String::new(),
        replace: replace.iter().map(|r| r.to_string()).collect(),
        settle: 0,
        wall: String::new(),
        rest: false,
        only: false,
    }
}

/// A full block on the ground near `pos`, as a player would build: within three blocks down
/// or up, on something solid (the ground, or the block placed before it), never in the air.
fn on_ground(ids: &mut impl FnMut() -> u64, pos: BlockPos, block: &str) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: block.to_string(),
        state: String::new(),
        replace: GROUND_COVER.iter().map(|r| r.to_string()).collect(),
        settle: 3,
        wall: String::new(),
        rest: true,
        only: false,
    }
}

/// What lies on the ground and a block placed there may take the place of: grass, ferns,
/// snow, never anything built.
const GROUND_COVER: &[&str] = &[
    "minecraft:short_grass",
    "minecraft:tall_grass",
    "minecraft:fern",
    "minecraft:snow",
];

/// The blocks a command changes where the target could see it: what the view rule keeps out of
/// sight. What happens inside a chest, to an entity by its id, or to the screen is not here; an
/// event's own spot covers the entities.
pub fn spots(command: &Command) -> Vec<BlockPos> {
    let block = |p: [f64; 3]| {
        [
            p[0].floor() as i32,
            p[1].floor() as i32,
            p[2].floor() as i32,
        ]
    };
    match command {
        Command::ToggleBlock { pos, .. }
        | Command::SetBlock { pos, .. }
        | Command::RotateBlock { pos, .. }
        | Command::BreakBlock { pos, .. }
        | Command::SetSign { pos, .. } => vec![*pos],
        Command::MoveBlock { from, to, .. } => vec![*from, *to],
        Command::DropItem { pos, .. } => vec![block(*pos)],
        Command::MoveEntity { to, .. } => vec![block(*to)],
        _ => vec![],
    }
}

/// The words a command puts in the world: a sign's lines, a chat line, a name.
pub fn texts(command: &Command) -> Vec<String> {
    match command {
        Command::SetSign { lines, .. } => lines.iter().filter(|l| !l.is_empty()).cloned().collect(),
        Command::GameMessage { text, .. } if !text.is_empty() => vec![text.clone()],
        Command::NameEntity { name, .. } => vec![name.clone()],
        _ => vec![],
    }
}

/// A torch near `pos`, as a player would place one: on the ground within `settle` blocks down
/// or up, else on a wall there, never floating. The mind knows positions, not terrain; the
/// host finds the spot (`docs/sdk.md`, `set_block`).
fn torch(ids: &mut impl FnMut() -> u64, pos: BlockPos, settle: u8) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: "minecraft:torch".to_string(),
        state: String::new(),
        replace: vec![],
        settle,
        wall: "minecraft:wall_torch".to_string(),
        rest: false,
        only: false,
    }
}

/// Natural ground the lodger may wear, dig or build on; never anything a player builds with.
const GROUND: &[&str] = &[
    "minecraft:grass_block",
    "minecraft:dirt",
    "minecraft:coarse_dirt",
    "minecraft:podzol",
    "minecraft:stone",
    "minecraft:gravel",
    "minecraft:sand",
    "minecraft:snow",
    "minecraft:short_grass",
    "minecraft:tall_grass",
    "minecraft:fern",
];

/// The ground itself, not what lies on it (snow, grass, ferns): taking one of these out is
/// digging.
const SOLID_GROUND: &[&str] = &[
    "minecraft:grass_block",
    "minecraft:dirt",
    "minecraft:coarse_dirt",
    "minecraft:podzol",
    "minecraft:stone",
    "minecraft:gravel",
    "minecraft:sand",
];

/// A step of one block from `from` toward `to`, along the longer axis.
fn step_toward(from: BlockPos, to: BlockPos) -> [i32; 2] {
    let dx = to[0] - from[0];
    let dz = to[2] - from[2];
    if dx.abs() >= dz.abs() {
        [dx.signum(), 0]
    } else {
        [0, dz.signum()]
    }
}

/// The moves that sort a container by item id into the first slots, each as (from, to, count),
/// using one empty slot as the hand. None when there is no empty slot or it is sorted already.
pub fn tidy_moves(contents: &[ItemStack], size: u32) -> Vec<(u32, u32, u32)> {
    let mut slots: Vec<Option<ItemStack>> = vec![None; size as usize];
    for stack in contents {
        if (stack.slot as usize) < slots.len() {
            slots[stack.slot as usize] = Some(stack.clone());
        }
    }
    let mut wanted: Vec<ItemStack> = contents.to_vec();
    wanted.sort_by(|a, b| a.id.cmp(&b.id).then(b.count.cmp(&a.count)));

    let mut moves = Vec::new();
    for (target, stack) in wanted.iter().enumerate() {
        let target = target as u32;
        let from = slots
            .iter()
            .position(|s| {
                s.as_ref()
                    .map(|s| s.id == stack.id && s.count == stack.count)
                    .unwrap_or(false)
            })
            .map(|i| i as u32);
        let Some(from) = from else { continue };
        if from == target {
            continue;
        }
        if slots[target as usize].is_some() {
            let Some(empty) = slots.iter().position(|s| s.is_none()) else {
                return Vec::new();
            };
            let moved = slots[target as usize].take().unwrap();
            moves.push((target, empty as u32, moved.count));
            slots[empty] = Some(moved);
        }
        let moved = slots[from as usize].take().unwrap();
        moves.push((from, target, moved.count));
        slots[target as usize] = Some(moved);
    }

    moves
}

/// The commands for the event at `pos`, each with the ticks to wait before sending it (0 for
/// now). `ids` hands out command ids.
pub fn commands(
    event: &Event,
    pos: Option<BlockPos>,
    memory: &Memory,
    ids: &mut impl FnMut() -> u64,
    now: &Now,
    written: Option<&str>,
) -> Vec<(Command, u64)> {
    let target = memory.target.unwrap_or(0);
    let at = block_center;
    let seed = pos
        .map(|p| (p[0] + p[2]).unsigned_abs() as usize)
        .unwrap_or(0)
        + memory.done.len();
    // A sound event plays one of its ways to sound, the next after the last: from the house's
    // door or chest, from far off, or around the target as it moves.
    if let Some(ways) = super::sounds::variants(event.id) {
        let spot = match event.action {
            Action::SoundFar(_) => {
                let angle = (seed % 8) as f64 * std::f64::consts::FRAC_PI_4;
                Some([
                    memory.last.pos[0] + 36.0 * angle.cos(),
                    memory.last.pos[1],
                    memory.last.pos[2] + 36.0 * angle.sin(),
                ])
            }
            _ => pos.map(block_center),
        };
        let way = super::sounds::pick(ways, seed, memory.times(event.id));
        return super::sounds::commands(way, spot, memory, target, seed, ids);
    }
    // The words: what the model wrote for this event, checked, or the bank's.
    let words = |kind: Kind, mob: &str, count: u32| -> String {
        written
            .map(|w| w.to_string())
            .unwrap_or_else(|| text::pick_fresh(kind, seed, mob, count, &memory.lines_written))
    };

    match event.action {
        Action::Wait => vec![],
        // Its sounds are its ways to sound, above.
        Action::ChestAnswers => vec![],
        Action::ToggleDoor | Action::ToggleFixture(_) => pos
            .map(|p| vec![(Command::ToggleBlock { id: ids(), pos: p }, 0)])
            .unwrap_or_default(),
        Action::ToggleCandles => memory
            .fixtures("candle")
            .map(|f| {
                (
                    Command::ToggleBlock {
                        id: ids(),
                        pos: f.pos,
                    },
                    0,
                )
            })
            .collect(),
        Action::Sound(sound) | Action::SoundAtFixture(_, sound) => pos
            .map(|p| vec![(sound_at(ids, target, sound, at(p), 1.0), 0)])
            .unwrap_or_default(),
        Action::SoundAtTarget(sound) => {
            vec![(sound_at(ids, target, sound, memory.last.pos, 0.8), 10)]
        }
        Action::SoundFar(sound) => {
            let angle = (seed % 8) as f64 * std::f64::consts::FRAC_PI_4;
            let far = [
                memory.last.pos[0] + 36.0 * angle.cos(),
                memory.last.pos[1],
                memory.last.pos[2] + 36.0 * angle.sin(),
            ];
            vec![(sound_at(ids, target, sound, far, 2.0), 0)]
        }
        Action::Taps(_, sound, count) => pos
            .map(|p| {
                (0..count)
                    .map(|i| (sound_at(ids, target, sound, at(p), 0.9), i as u64 * 10))
                    .collect()
            })
            .unwrap_or_default(),
        Action::OpenLid => pos
            .map(|p| {
                vec![(
                    Command::OpenLid {
                        id: ids(),
                        pos: p,
                        seconds: 1.5,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::MoveLight => pos
            .map(|p| {
                // A step of one to three blocks along one axis, same height.
                let steps = [
                    [2, 0, 0],
                    [-2, 0, 0],
                    [0, 0, 2],
                    [0, 0, -2],
                    [3, 0, 0],
                    [0, 0, -3],
                ];
                let step = steps[(p[0] + p[2]).rem_euclid(steps.len() as i32) as usize];
                vec![(
                    Command::MoveBlock {
                        id: ids(),
                        from: p,
                        to: [p[0] + step[0], p[1], p[2] + step[2]],
                        around: 0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::AddTorch => pos
            .map(|p| vec![(torch(ids, [p[0], p[1] + 1, p[2]], 2), 0)])
            .unwrap_or_default(),
        Action::LightPath => {
            let Some(home) = memory.home.as_ref() else {
                return vec![];
            };
            let Some(spot) = memory
                .place
                .or_else(|| memory.route_end())
                .or_else(|| Some([home.center[0] + 10, home.floor_y, home.center[2] + 10]))
            else {
                return vec![];
            };
            let count = 3 + (seed % 5) as i32;
            let mut made = Vec::new();
            let mut p = home.center;
            for i in 0..count {
                let step = step_toward(p, spot);
                p = [p[0] + step[0] * 3, p[1], p[2] + step[1] * 3];
                made.push((torch(ids, [p[0], p[1] + 1, p[2]], 4), i as u64 * 20));
            }
            made
        }
        Action::ShuffleContainer => match (pos, memory.used_container()) {
            (Some(p), Some(container)) if !container.contents.is_empty() => {
                match container.empty_slots().first() {
                    Some(&to_slot) => {
                        let stack = &container.contents[seed % container.contents.len()];
                        vec![(
                            Command::MoveItem {
                                id: ids(),
                                from: p,
                                from_slot: stack.slot,
                                to: p,
                                to_slot,
                                count: stack.count,
                            },
                            0,
                        )]
                    }
                    None => vec![],
                }
            }
            _ => vec![],
        },
        Action::SplitStack => match (pos, memory.used_container()) {
            (Some(p), Some(container)) => {
                match (
                    container.contents.iter().find(|s| s.count >= 2),
                    container.empty_slots().first(),
                ) {
                    (Some(stack), Some(&to_slot)) => vec![(
                        Command::MoveItem {
                            id: ids(),
                            from: p,
                            from_slot: stack.slot,
                            to: p,
                            to_slot,
                            count: stack.count / 2,
                        },
                        0,
                    )],
                    _ => vec![],
                }
            }
            _ => vec![],
        },
        Action::TidyContainer => match (pos, memory.used_container()) {
            (Some(p), Some(container)) => tidy_moves(&container.contents, 27)
                .into_iter()
                .enumerate()
                .map(|(i, (from_slot, to_slot, count))| {
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: p,
                            from_slot,
                            to: p,
                            to_slot,
                            count,
                        },
                        i as u64,
                    )
                })
                .collect(),
            _ => vec![],
        },
        Action::SwapStacks => match (memory.used_container(), memory.other_container()) {
            (Some(a), Some(b)) => {
                let (Some(sa), Some(sb)) = (a.contents.first(), b.contents.first()) else {
                    return vec![];
                };
                let (Some(&ea), Some(&eb)) = (a.empty_slots().first(), b.empty_slots().first())
                else {
                    return vec![];
                };
                vec![
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: a.pos,
                            from_slot: sa.slot,
                            to: b.pos,
                            to_slot: eb,
                            count: sa.count,
                        },
                        0,
                    ),
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: b.pos,
                            from_slot: sb.slot,
                            to: a.pos,
                            to_slot: ea,
                            count: sb.count,
                        },
                        1,
                    ),
                ]
            }
            _ => vec![],
        },
        Action::FillGap => match (memory.used_container(), memory.other_container()) {
            (Some(a), Some(b)) => {
                let Some(biggest) = a
                    .contents
                    .iter()
                    .filter(|s| s.count >= 2)
                    .max_by_key(|s| s.count)
                else {
                    return vec![];
                };
                let Some(&empty) = b.empty_slots().first() else {
                    return vec![];
                };
                vec![(
                    Command::MoveItem {
                        id: ids(),
                        from: a.pos,
                        from_slot: biggest.slot,
                        to: b.pos,
                        to_slot: empty,
                        count: biggest.count / 2,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::CookOne => {
            let (Some(furnace), Some((chest, raw)), Some((_, burn))) = (
                memory.furnace(),
                memory.in_used_container(smeltable),
                memory.in_used_container(fuel),
            ) else {
                return vec![];
            };
            let mut made = vec![(
                Command::MoveItem {
                    id: ids(),
                    from: chest.pos,
                    from_slot: raw.slot,
                    to: furnace.pos,
                    to_slot: 0,
                    count: 1,
                },
                0,
            )];
            if !furnace.contents.iter().any(|s| s.slot == 1) {
                made.push((
                    Command::MoveItem {
                        id: ids(),
                        from: chest.pos,
                        from_slot: burn.slot,
                        to: furnace.pos,
                        to_slot: 1,
                        count: 1,
                    },
                    1,
                ));
            }
            made
        }
        Action::TakeOutput => match (memory.furnace(), memory.used_container()) {
            (Some(furnace), Some(chest)) => {
                let (Some(out), Some(&empty)) = (
                    furnace.contents.iter().find(|s| s.slot == 2),
                    chest.empty_slots().first(),
                ) else {
                    return vec![];
                };
                vec![(
                    Command::MoveItem {
                        id: ids(),
                        from: furnace.pos,
                        from_slot: 2,
                        to: chest.pos,
                        to_slot: empty,
                        count: out.count,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::ItemOnTable => match (pos, memory.favourite_in_container()) {
            (Some(table), Some((container, stack))) => vec![
                (
                    Command::TakeItem {
                        id: ids(),
                        from: container.pos,
                        slot: stack.slot,
                        count: 1,
                        keep: "item_on_table".to_string(),
                    },
                    0,
                ),
                (
                    Command::DropItem {
                        id: ids(),
                        pos: [
                            table[0] as f64 + 0.5,
                            table[1] as f64 + 1.1,
                            table[2] as f64 + 0.5,
                        ],
                        item: ItemStack {
                            id: stack.id.clone(),
                            count: 1,
                            slot: 0,
                        },
                        kept: "item_on_table".to_string(),
                    },
                    1,
                ),
            ],
            _ => vec![],
        },
        Action::EggOnBed => pos
            .map(|bed| {
                vec![(
                    Command::DropItem {
                        id: ids(),
                        pos: [
                            bed[0] as f64 + 0.5,
                            bed[1] as f64 + 0.7,
                            bed[2] as f64 + 0.5,
                        ],
                        item: ItemStack {
                            id: "minecraft:egg".to_string(),
                            count: 1,
                            slot: 0,
                        },
                        kept: String::new(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::Take => match memory.favourite_in_container() {
            Some((container, stack)) => vec![(
                Command::TakeItem {
                    id: ids(),
                    from: container.pos,
                    slot: stack.slot,
                    count: 1,
                    keep: "held".to_string(),
                },
                0,
            )],
            None => vec![],
        },
        Action::ReturnTaken => match &memory.held {
            Some(held) => vec![(
                Command::PutItem {
                    id: ids(),
                    into: held.from,
                    slot: Some(held.slot),
                    item: held.item.clone(),
                    kept: "held".to_string(),
                    wear: 0,
                    name: String::new(),
                },
                0,
            )],
            None => vec![],
        },
        // What it took, given back a little used: a tool worn some more, anything else named.
        Action::ReturnUsed => match &memory.held {
            Some(held) => {
                let tool = [
                    "_pickaxe", "_axe", "_sword", "_shovel", "_hoe", "bow", "shears",
                ]
                .iter()
                .any(|t| held.item.id.ends_with(t));
                vec![(
                    Command::PutItem {
                        id: ids(),
                        into: held.from,
                        slot: Some(held.slot),
                        item: held.item.clone(),
                        kept: "held".to_string(),
                        wear: if tool { 4 + (seed % 9) as u32 } else { 0 },
                        name: if tool {
                            String::new()
                        } else {
                            words(Kind::ItemName, "", 0)
                        },
                    },
                    0,
                )]
            }
            None => vec![],
        },
        // What waits in the hopper goes into the chest beside or under it that is theirs; the
        // mind never sees inside a hopper, so the host moves whatever is there.
        Action::HopperEmptied => pos
            .and_then(|hopper| {
                memory
                    .containers
                    .iter()
                    .filter(|c| c.theirs() && !c.is_furnace())
                    .find(|c| {
                        let d = [
                            c.pos[0] - hopper[0],
                            c.pos[1] - hopper[1],
                            c.pos[2] - hopper[2],
                        ];
                        d[0].abs() + d[1].abs() + d[2].abs() == 1
                    })
                    .map(|chest| {
                        (0..5)
                            .map(|slot| {
                                (
                                    Command::MoveItem {
                                        id: ids(),
                                        from: hopper,
                                        from_slot: slot,
                                        to: chest.pos,
                                        to_slot: 0,
                                        count: 64,
                                    },
                                    slot as u64,
                                )
                            })
                            .collect()
                    })
            })
            .unwrap_or_default(),
        // Across the room: three to five blocks the seed's way, wherever there is room and
        // ground, its contents with it.
        Action::ShulkerMoved => pos
            .map(|p| {
                let way = [[1, 0], [-1, 0], [0, 1], [0, -1]][seed % 4];
                let reach = 3 + (seed % 3) as i32;
                vec![(
                    Command::MoveBlock {
                        id: ids(),
                        from: p,
                        to: [p[0] + way[0] * reach, p[1], p[2] + way[1] * reach],
                        around: 2,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::BookWord => match memory.in_used_container(|id| id == "minecraft:writable_book") {
            Some((container, book)) => {
                let word = words(Kind::SignWord, "", 0);
                vec![(
                    Command::SignBook {
                        id: ids(),
                        pos: container.pos,
                        slot: book.slot,
                        title: word.clone(),
                        pages: vec![word],
                    },
                    0,
                )]
            }
            None => vec![],
        },
        // Its place, or the home's middle if it has none yet.
        Action::MapMarked => match memory.in_used_container(|id| id == "minecraft:filled_map") {
            Some((container, map)) => {
                let at = memory
                    .place
                    .or_else(|| memory.home.as_ref().map(|h| h.center));
                at.map(|at| {
                    vec![(
                        Command::MarkMap {
                            id: ids(),
                            pos: container.pos,
                            slot: map.slot,
                            at,
                        },
                        0,
                    )]
                })
                .unwrap_or_default()
            }
            None => vec![],
        },
        Action::RenameOne => match (pos, memory.used_container()) {
            (Some(p), Some(container)) if !container.contents.is_empty() => {
                // Their favourite if it is in there, else whatever the seed lands on.
                let favourite = memory.favourites.first().map(|(id, _)| id.as_str());
                let stack = container
                    .contents
                    .iter()
                    .find(|s| Some(s.id.as_str()) == favourite)
                    .unwrap_or(&container.contents[seed % container.contents.len()]);
                vec![(
                    Command::RenameItem {
                        id: ids(),
                        pos: p,
                        slot: stack.slot,
                        name: words(Kind::ItemName, "", 0),
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::PlaceTaken => match (&memory.held, pos) {
            (Some(held), Some(p)) => vec![(
                Command::DropItem {
                    id: ids(),
                    pos: [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5],
                    item: held.item.clone(),
                    kept: "held".to_string(),
                },
                0,
            )],
            _ => vec![],
        },
        Action::Hint(kind, seconds) => {
            let detail = match kind {
                "hotbar_slip" => memory
                    .favourites
                    .first()
                    .map(|(id, _)| id.clone())
                    .unwrap_or_default(),
                _ => String::new(),
            };
            let hint_pos = match kind {
                "camera_nudge" => memory
                    .place
                    .or_else(|| memory.home.as_ref().map(|h| h.center)),
                _ => None,
            };
            vec![(
                Command::Hint {
                    id: ids(),
                    player: target,
                    hint: kind.to_string(),
                    pos: hint_pos,
                    seconds,
                    detail,
                },
                0,
            )]
        }
        Action::RefuseRest => vec![(
            Command::CancelSleep {
                id: ids(),
                player: target,
                seconds: 120.0,
            },
            0,
        )],
        Action::Wake => {
            let mut made = vec![(
                Command::CancelSleep {
                    id: ids(),
                    player: target,
                    seconds: 0.0,
                },
                0,
            )];
            if let Some(door) = pos {
                made.push((
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                    },
                    5,
                ));
            }
            made
        }
        // Held still while they look: a look alone fought the animals' own walking, and they
        // jittered (the owner, 2026-11-09).
        Action::Stare => memory
            .creatures("")
            .take(8)
            .map(|creature| {
                (
                    Command::Hold {
                        id: ids(),
                        entity: creature.id,
                        player: target,
                        seconds: 10.0 + (seed % 21) as f32,
                    },
                    0,
                )
            })
            .collect(),
        Action::Hole => pos
            .map(|p| {
                let mut made: Vec<(Command, u64)> = (0..3)
                    .map(|depth| {
                        (
                            set(ids, [p[0], p[1] - depth, p[2]], "minecraft:air", GROUND),
                            depth as u64 * 15,
                        )
                    })
                    .collect();
                made.push((torch(ids, [p[0], p[1] - 2, p[2]], 1), 50));
                made
            })
            .unwrap_or_default(),
        Action::Cairn => pos
            .map(|p| {
                (0..3)
                    .map(|height| {
                        (
                            on_ground(
                                ids,
                                [p[0], p[1] + 1 + height, p[2]],
                                "minecraft:cobblestone",
                            ),
                            height as u64 * 15,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Action::Sapling => match (pos, memory.in_used_container(|id| id.ends_with("sapling"))) {
            // Planted first, on the ground within three blocks, and only then taken from the
            // chest: a planting that fails drops the take, so no sapling of theirs is lost.
            (Some(p), Some((container, stack))) => vec![
                (
                    Command::SetBlock {
                        id: ids(),
                        pos: [p[0], p[1] + 1, p[2]],
                        block: stack.id.clone(),
                        state: String::new(),
                        replace: GROUND_COVER.iter().map(|r| r.to_string()).collect(),
                        settle: 3,
                        wall: String::new(),
                        rest: false,
                        only: false,
                    },
                    0,
                ),
                (
                    Command::TakeItem {
                        id: ids(),
                        from: container.pos,
                        slot: stack.slot,
                        count: 1,
                        keep: String::new(),
                    },
                    2,
                ),
            ],
            _ => vec![],
        },
        Action::Sign => pos
            .map(|p| {
                let word = words(Kind::SignWord, "", 0);
                sign_at(ids, [p[0], p[1] + 1, p[2]], vec![String::new(), word])
            })
            .unwrap_or_default(),
        Action::KillSign => match (pos, memory.most_killed()) {
            (Some(door), Some(tally)) => {
                let Some(home) = memory.home.as_ref() else {
                    return vec![];
                };
                // Two blocks out from the door, away from the house.
                let step = step_toward(home.center, door);
                let p = [door[0] + step[0] * 2, door[1], door[2] + step[1] * 2];
                sign_at(
                    ids,
                    p,
                    vec![words(Kind::KillSign, &tally.kind, tally.kills)],
                )
            }
            _ => vec![],
        },
        Action::Grave => match (pos, memory.most_killed()) {
            (Some(p), Some(tally)) => {
                let mut made = vec![
                    (
                        on_ground(ids, [p[0], p[1] + 1, p[2]], "minecraft:cobblestone"),
                        0,
                    ),
                    (
                        on_ground(ids, [p[0], p[1] + 2, p[2]], "minecraft:cobblestone"),
                        15,
                    ),
                ];
                made.extend(
                    sign_at(
                        ids,
                        [p[0] + 1, p[1] + 1, p[2]],
                        vec![words(Kind::GraveSign, &tally.kind, tally.kills)],
                    )
                    .into_iter()
                    .map(|(c, t)| (c, t + 30)),
                );
                made
            }
            _ => vec![],
        },
        Action::PathWorn => match (pos, memory.home.as_ref()) {
            (Some(door), Some(home)) => {
                let spot = memory.place.unwrap_or([
                    home.center[0] + 10,
                    home.floor_y,
                    home.center[2] + 10,
                ]);
                let mut p = [door[0], door[1] - 1, door[2]];
                let step = step_toward(home.center, door);
                p = [p[0] + step[0], p[1], p[2] + step[1]];
                let count = 6 + (seed % 5) as i32;
                let mut made = Vec::new();
                for i in 0..count {
                    let step = step_toward(p, [spot[0], p[1], spot[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                    made.push((
                        set(ids, p, "minecraft:dirt_path", &["minecraft:grass_block"]),
                        i as u64 * 10,
                    ));
                }
                made
            }
            _ => vec![],
        },
        Action::CarpetShift => {
            let carpets: Vec<BlockPos> = memory.fixtures("carpet").map(|f| f.pos).collect();
            let mut sorted = carpets.clone();
            // The farthest along x moves first, so none lands on another.
            sorted.sort_by_key(|p| std::cmp::Reverse(p[0]));
            sorted
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    (
                        Command::MoveBlock {
                            id: ids(),
                            from: p,
                            to: [p[0] + 1, p[1], p[2]],
                            around: 0,
                        },
                        i as u64,
                    )
                })
                .collect()
        }
        Action::Cauldron => pos
            .map(|p| {
                vec![
                    (
                        set(ids, p, "minecraft:water_cauldron", &["minecraft:cauldron"]),
                        0,
                    ),
                    (
                        set(ids, p, "minecraft:cauldron", &["minecraft:water_cauldron"]),
                        1,
                    ),
                ]
            })
            .unwrap_or_default(),
        Action::RotateFrame => pos
            .and_then(|p| memory.decor("item_frame").find(|d| d.pos == p))
            .map(|frame| {
                vec![(
                    Command::RotateFrame {
                        id: ids(),
                        entity: frame.id,
                        rotation: None,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::FramesTurn => {
            let rotation = (seed % 8) as u32;
            memory
                .decor("item_frame")
                .map(|frame| {
                    (
                        Command::RotateFrame {
                            id: ids(),
                            entity: frame.id,
                            rotation: Some(rotation),
                        },
                        0,
                    )
                })
                .collect()
        }
        Action::SwapPainting => pos
            .and_then(|p| memory.decor("painting").find(|d| d.pos == p))
            .map(|painting| {
                vec![(
                    Command::SwapPainting {
                        id: ids(),
                        entity: painting.id,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::StandTurn => {
            let Some(stand) = memory.decor("armor_stand").next() else {
                return vec![];
            };
            let look = memory
                .home
                .as_ref()
                .and_then(|h| h.doors.first().copied().or(h.bed))
                .map(at)
                .unwrap_or(memory.last.pos);
            vec![(
                Command::PoseStand {
                    id: ids(),
                    entity: stand.id,
                    at: Some(look),
                    follow: None,
                    seconds: 0.0,
                },
                0,
            )]
        }
        Action::StandWatch => memory
            .decor("armor_stand")
            .map(|stand| {
                (
                    Command::PoseStand {
                        id: ids(),
                        entity: stand.id,
                        at: None,
                        follow: Some(target),
                        seconds: 90.0,
                    },
                    0,
                )
            })
            .collect(),
        Action::NameAnimal => memory
            .creatures("")
            .find(|c| !c.named && c.kind != "villager")
            .map(|c| {
                vec![(
                    Command::NameEntity {
                        id: ids(),
                        entity: c.id,
                        name: words(Kind::AnimalName, "", 0),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::DyeSheep => {
            const COLOURS: &[&str] = &["red", "black", "light_blue", "purple", "lime", "pink"];
            memory
                .creatures("sheep")
                .next()
                .map(|sheep| {
                    vec![(
                        Command::DyeSheep {
                            id: ids(),
                            entity: sheep.id,
                            colour: COLOURS[seed % COLOURS.len()].to_string(),
                        },
                        0,
                    )]
                })
                .unwrap_or_default()
        }
        Action::PetGrowl => memory
            .pets()
            .next()
            .map(|pet| {
                let sound = if pet.kind == "cat" {
                    "minecraft:entity.cat.hiss"
                } else {
                    "minecraft:entity.wolf.growl"
                };
                vec![
                    (sound_at(ids, target, sound, pet.pos, 1.0), 0),
                    (sound_at(ids, target, sound, pet.pos, 1.0), 25),
                ]
            })
            .unwrap_or_default(),
        Action::MoveCreature(kind, _) => {
            let creature = if kind.is_empty() {
                if event.id.starts_with("pet_") {
                    memory.pets().next()
                } else {
                    memory
                        .creatures("")
                        .find(|c| !c.pet && c.kind != "villager")
                }
            } else {
                memory.creatures(kind).next()
            };
            match (creature, pos) {
                (Some(c), Some(p)) => {
                    let mut made = vec![(
                        Command::MoveEntity {
                            id: ids(),
                            entity: c.id,
                            to: [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5],
                        },
                        0,
                    )];
                    if let Some(home) = memory.home.as_ref() {
                        made.push((
                            Command::LookAt {
                                id: ids(),
                                entity: c.id,
                                at: at(home.center),
                                seconds: 600.0,
                            },
                            1,
                        ));
                    }
                    made
                }
                _ => vec![],
            }
        }
        Action::GolemTurned => {
            match (memory.creatures("iron_golem").next(), memory.home.as_ref()) {
                (Some(golem), Some(home)) => vec![(
                    Command::LookAt {
                        id: ids(),
                        entity: golem.id,
                        at: at(home.center),
                        seconds: 1200.0,
                    },
                    0,
                )],
                _ => vec![],
            }
        }
        Action::TabEntry => vec![(
            Command::PlayerList {
                id: ids(),
                player: target,
                seconds: 60.0,
            },
            0,
        )],
        Action::Message(kind) => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: kind.to_string(),
                text: String::new(),
            },
            0,
        )],
        Action::Says => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: "says".to_string(),
                text: words(Kind::ChatLine, "", 0),
            },
            0,
        )],
        Action::SaysKills => memory
            .most_killed()
            .map(|tally| {
                vec![(
                    Command::GameMessage {
                        id: ids(),
                        player: target,
                        message: "says".to_string(),
                        text: words(Kind::KillLine, &tally.kind, tally.kills),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::Whisper => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: "whisper".to_string(),
                text: words(Kind::Whisper, "", 0),
            },
            0,
        )],
        Action::MoveVehicle => {
            let (Some(vehicle), Some(water)) = (memory.vehicles.first(), memory.feature("water"))
            else {
                return vec![];
            };
            // Adrift: the water's middle. Moved: across it, the same distance past the middle.
            let to = if event.id == "boat_adrift" {
                [
                    water.pos[0] as f64 + 0.5,
                    water.pos[1] as f64 + 0.6,
                    water.pos[2] as f64 + 0.5,
                ]
            } else {
                [
                    2.0 * water.pos[0] as f64 - vehicle.pos[0],
                    water.pos[1] as f64 + 0.6,
                    2.0 * water.pos[2] as f64 - vehicle.pos[2],
                ]
            };
            vec![(
                Command::MoveEntity {
                    id: ids(),
                    entity: vehicle.id,
                    to,
                },
                0,
            )]
        }
        Action::PushVehicle(kind) => memory
            .vehicles
            .iter()
            .find(|v| v.kind.contains(kind) || (kind == "boat" && v.kind.contains("raft")))
            .map(|vehicle| {
                // Away from the target, slowly: a drift, not a shove.
                let dx = vehicle.pos[0] - memory.last.pos[0];
                let dz = vehicle.pos[2] - memory.last.pos[2];
                let length = (dx * dx + dz * dz).sqrt().max(1.0);
                let speed = if kind == "boat" { 0.12 } else { 0.3 };
                vec![(
                    Command::PushEntity {
                        id: ids(),
                        entity: vehicle.id,
                        velocity: [dx / length * speed, 0.0, dz / length * speed],
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::BucketFloor => match (
            pos,
            memory.in_used_container(|id| id == "minecraft:water_bucket"),
        ) {
            (Some(p), Some((container, stack))) => vec![
                (
                    Command::TakeItem {
                        id: ids(),
                        from: container.pos,
                        slot: stack.slot,
                        count: 1,
                        keep: String::new(),
                    },
                    0,
                ),
                (
                    Command::PutItem {
                        id: ids(),
                        into: container.pos,
                        slot: Some(stack.slot),
                        item: ItemStack {
                            id: "minecraft:bucket".to_string(),
                            count: 1,
                            slot: 0,
                        },
                        kept: String::new(),
                        wear: 0,
                        name: String::new(),
                    },
                    1,
                ),
                (set(ids, p, "minecraft:water", GROUND), 2),
            ],
            _ => vec![],
        },
        Action::SnowCleared => match (pos, memory.home.as_ref()) {
            (Some(door), Some(home)) => {
                let spot = memory.place.unwrap_or([
                    home.center[0] + 10,
                    home.floor_y,
                    home.center[2] + 10,
                ]);
                let mut p = door;
                let mut made = Vec::new();
                for i in 0..8 {
                    let step = step_toward(p, [spot[0], p[1], spot[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                    made.push((
                        set(ids, p, "minecraft:air", &["minecraft:snow"]),
                        i as u64 * 8,
                    ));
                }
                made
            }
            _ => vec![],
        },
        Action::Trample => match (pos, memory.feature("crops")) {
            (Some(door), Some(field)) => {
                let mut p = [field.pos[0], field.pos[1] - 1, field.pos[2]];
                let mut made = Vec::new();
                for i in 0..5 {
                    made.push((
                        set(ids, p, "minecraft:dirt", &["minecraft:farmland"]),
                        i as u64 * 10,
                    ));
                    let step = step_toward(p, [door[0], p[1], door[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                }
                made
            }
            _ => vec![],
        },
        Action::HarvestOne => match (pos, memory.used_container()) {
            (Some(field), Some(chest)) => {
                let Some(&empty) = chest.empty_slots().first() else {
                    return vec![];
                };
                let dx = (seed % 5) as i32 - 2;
                let dz = ((seed / 5) % 5) as i32 - 2;
                vec![
                    (
                        set(
                            ids,
                            [field[0] + dx, field[1], field[2] + dz],
                            "minecraft:air",
                            &[
                                "minecraft:wheat",
                                "minecraft:carrots",
                                "minecraft:potatoes",
                                "minecraft:beetroots",
                            ],
                        ),
                        0,
                    ),
                    (
                        Command::PutItem {
                            id: ids(),
                            into: chest.pos,
                            slot: Some(empty),
                            item: ItemStack {
                                id: "minecraft:wheat".to_string(),
                                count: 1,
                                slot: 0,
                            },
                            kept: String::new(),
                            wear: 0,
                            name: String::new(),
                        },
                        1,
                    ),
                ]
            }
            _ => vec![],
        },
        Action::MiningSounds => {
            let angle = (seed % 8) as f64 * std::f64::consts::FRAC_PI_4;
            let at = [
                memory.last.pos[0] + 9.0 * angle.cos(),
                memory.last.pos[1] - 1.0,
                memory.last.pos[2] + 9.0 * angle.sin(),
            ];
            (0..6)
                .map(|i| {
                    (
                        sound_at(ids, target, "minecraft:block.stone.break", at, 0.6),
                        i as u64 * 14 + 10,
                    )
                })
                .collect()
        }
        Action::PlateLaid => match (
            pos,
            memory.in_used_container(|id| id.ends_with("pressure_plate")),
        ) {
            (Some(chest), Some((container, stack))) => {
                let step = memory
                    .home
                    .as_ref()
                    .map(|h| step_toward(chest, h.center))
                    .filter(|s| *s != [0, 0])
                    .unwrap_or([1, 0]);
                vec![
                    (
                        Command::TakeItem {
                            id: ids(),
                            from: container.pos,
                            slot: stack.slot,
                            count: 1,
                            keep: String::new(),
                        },
                        0,
                    ),
                    (
                        set(
                            ids,
                            [chest[0] + step[0], chest[1], chest[2] + step[1]],
                            &stack.id,
                            &[],
                        ),
                        1,
                    ),
                ]
            }
            _ => vec![],
        },
        Action::DoorAhead => pos
            .map(|door| {
                vec![(
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::VillageDoors => memory
            .scanned("door")
            .take(6)
            .enumerate()
            .map(|(i, door)| {
                (
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                    },
                    i as u64 * 20,
                )
            })
            .collect(),
        Action::ScanVillage => memory
            .feature("village")
            .map(|village| {
                vec![(
                    Command::Scan {
                        id: ids(),
                        min: [village.pos[0] - 19, village.pos[1] - 8, village.pos[2] - 19],
                        max: [
                            village.pos[0] + 19,
                            village.pos[1] + 15,
                            village.pos[2] + 19,
                        ],
                        kinds: vec![
                            "minecraft:oak_door".to_string(),
                            "minecraft:spruce_door".to_string(),
                            "minecraft:birch_door".to_string(),
                            "minecraft:acacia_door".to_string(),
                            "minecraft:jungle_door".to_string(),
                            "minecraft:dark_oak_door".to_string(),
                            "minecraft:bell".to_string(),
                        ],
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::RotateFixture(_) => pos
            .map(|p| vec![(Command::RotateBlock { id: ids(), pos: p }, 0)])
            .unwrap_or_default(),
        Action::SoundPair(_, first, second) => pos
            .map(|p| {
                vec![
                    (sound_at(ids, target, first, at(p), 0.8), 0),
                    (sound_at(ids, target, second, at(p), 0.8), 12),
                ]
            })
            .unwrap_or_default(),
        Action::FrameEmptied => match (
            memory.decor("item_frame").next(),
            memory.home.as_ref().and_then(|h| h.bed),
        ) {
            (Some(frame), Some(bed)) => vec![
                (
                    Command::SetFrameItem {
                        id: ids(),
                        entity: frame.id,
                        item: ItemStack::default(),
                    },
                    0,
                ),
                // The host answers what the frame held? It does not; the memory does not know
                // it either, so the bed gets a stand-in the frame could have held.
                (
                    Command::DropItem {
                        id: ids(),
                        pos: [
                            bed[0] as f64 + 0.5,
                            bed[1] as f64 + 0.7,
                            bed[2] as f64 + 0.5,
                        ],
                        item: ItemStack {
                            id: memory
                                .favourites
                                .first()
                                .map(|(id, _)| id.clone())
                                .unwrap_or_else(|| "minecraft:stick".to_string()),
                            count: 1,
                            slot: 0,
                        },
                        kept: String::new(),
                    },
                    1,
                ),
            ],
            _ => vec![],
        },
        Action::FrameSwap => match (memory.decor("item_frame").next(), memory.used_container()) {
            (Some(frame), Some(chest)) => {
                let Some(stack) = chest.contents.iter().find(|s| s.count == 1) else {
                    return vec![];
                };
                vec![
                    (
                        Command::TakeItem {
                            id: ids(),
                            from: chest.pos,
                            slot: stack.slot,
                            count: 1,
                            keep: String::new(),
                        },
                        0,
                    ),
                    (
                        Command::SetFrameItem {
                            id: ids(),
                            entity: frame.id,
                            item: ItemStack {
                                id: stack.id.clone(),
                                count: 1,
                                slot: 0,
                            },
                        },
                        1,
                    ),
                ]
            }
            _ => vec![],
        },
        Action::StandDress => {
            let armour = |id: &str| {
                ["_helmet", "_chestplate", "_leggings", "_boots"]
                    .iter()
                    .any(|a| id.ends_with(a))
            };
            match (
                memory.decor("armor_stand").next(),
                memory.in_used_container(armour),
            ) {
                (Some(stand), Some((chest, stack))) => vec![
                    (
                        Command::TakeItem {
                            id: ids(),
                            from: chest.pos,
                            slot: stack.slot,
                            count: 1,
                            keep: String::new(),
                        },
                        0,
                    ),
                    (
                        Command::EquipEntity {
                            id: ids(),
                            entity: stand.id,
                            item: ItemStack {
                                id: stack.id.clone(),
                                count: 1,
                                slot: 0,
                            },
                        },
                        1,
                    ),
                ],
                _ => vec![],
            }
        }
        Action::ToolUsed => memory
            .tool_in_container()
            .map(|(chest, stack)| {
                vec![(
                    Command::DamageItem {
                        id: ids(),
                        pos: chest.pos,
                        slot: stack.slot,
                        amount: 20 + (seed % 30) as u32,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::ColdFurnace => match (memory.furnace(), memory.used_container()) {
            (Some(furnace), Some(chest)) => {
                let (Some(fuel), Some(&empty)) = (
                    furnace.contents.iter().find(|s| s.slot == 1),
                    chest.empty_slots().first(),
                ) else {
                    return vec![];
                };
                vec![(
                    Command::MoveItem {
                        id: ids(),
                        from: furnace.pos,
                        from_slot: 1,
                        to: chest.pos,
                        to_slot: empty,
                        count: fuel.count,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::OpenOnEnter => pos
            .map(|p| {
                vec![(
                    Command::OpenLid {
                        id: ids(),
                        pos: p,
                        seconds: 8.0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::TidyByColour => match (memory.used_container(), pos) {
            (Some(container), _) => {
                // Sorted by the colour word first, so wool and concrete of one colour sit together.
                let mut by_colour = container.contents.clone();
                for stack in &mut by_colour {
                    if crate::memory::is_coloured(&stack.id) {
                        let name = stack.id.rsplit(':').next().unwrap_or(&stack.id).to_string();
                        let (colour, kind) = name.rsplit_once('_').unwrap_or((&name, ""));
                        stack.id = format!("0{colour}_{kind}");
                    }
                }
                tidy_moves(&by_colour, 27)
                    .into_iter()
                    .enumerate()
                    .map(|(i, (from_slot, to_slot, count))| {
                        (
                            Command::MoveItem {
                                id: ids(),
                                from: container.pos,
                                from_slot,
                                to: container.pos,
                                to_slot,
                                count,
                            },
                            i as u64,
                        )
                    })
                    .collect()
            }
            _ => vec![],
        },
        Action::PlantFlower => {
            let flower = |id: &str| {
                [
                    "poppy",
                    "dandelion",
                    "tulip",
                    "orchid",
                    "allium",
                    "daisy",
                    "cornflower",
                    "lily_of_the_valley",
                ]
                .iter()
                .any(|f| id.ends_with(f))
            };
            match (pos, memory.in_used_container(flower), memory.home.as_ref()) {
                (Some(door), Some((chest, stack)), Some(home)) => {
                    let step = step_toward(home.center, door);
                    let p = [
                        door[0] + step[0] * 2 + step[1],
                        door[1],
                        door[2] + step[1] * 2 + step[0],
                    ];
                    vec![
                        (
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: stack.slot,
                                count: 1,
                                keep: String::new(),
                            },
                            0,
                        ),
                        (set(ids, p, &stack.id, &["minecraft:short_grass"]), 1),
                    ]
                }
                _ => vec![],
            }
        }
        Action::ReplantOther => {
            let seeds = |id: &str| {
                ["carrot", "potato", "beetroot_seeds"]
                    .iter()
                    .any(|s| id.ends_with(s))
            };
            match (pos, memory.in_used_container(seeds)) {
                (Some(field), Some((chest, stack))) => {
                    let crop = if stack.id.ends_with("carrot") {
                        "minecraft:carrots"
                    } else if stack.id.ends_with("potato") {
                        "minecraft:potatoes"
                    } else {
                        "minecraft:beetroots"
                    };
                    vec![
                        (
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: stack.slot,
                                count: 1,
                                keep: String::new(),
                            },
                            0,
                        ),
                        (set(ids, field, crop, &["minecraft:wheat"]), 1),
                    ]
                }
                _ => vec![],
            }
        }
        Action::IcePath => match (memory.feature("water"), memory.home.as_ref()) {
            (Some(water), Some(home)) => {
                let mut p = water.pos;
                let mut made = Vec::new();
                for i in 0..6 {
                    // On the water's surface wherever it is within two blocks, never on land.
                    made.push((
                        Command::SetBlock {
                            id: ids(),
                            pos: p,
                            block: "minecraft:ice".to_string(),
                            state: String::new(),
                            replace: vec!["minecraft:water".to_string()],
                            settle: 2,
                            wall: String::new(),
                            rest: false,
                            only: true,
                        },
                        i as u64 * 10,
                    ));
                    let step = step_toward(p, [home.center[0], p[1], home.center[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                }
                made
            }
            _ => vec![],
        },
        Action::BedOccupied => vec![
            (
                Command::GameMessage {
                    id: ids(),
                    player: target,
                    message: "bed_occupied".to_string(),
                    text: String::new(),
                },
                0,
            ),
            (
                Command::CancelSleep {
                    id: ids(),
                    player: target,
                    seconds: 0.0,
                },
                1,
            ),
        ],
        Action::WokeElsewhere => pos
            .map(|bed| {
                vec![
                    (
                        Command::CancelSleep {
                            id: ids(),
                            player: target,
                            seconds: 0.0,
                        },
                        0,
                    ),
                    (
                        Command::MovePlayer {
                            id: ids(),
                            player: target,
                            to: [
                                bed[0] as f64 + 1.5,
                                bed[1] as f64 + 0.6,
                                bed[2] as f64 + 0.5,
                            ],
                        },
                        2,
                    ),
                ]
            })
            .unwrap_or_default(),
        Action::ScanOre => {
            let at = block_of(memory.last.pos);
            vec![(
                Command::Scan {
                    id: ids(),
                    min: [at[0] - 12, at[1] - 6, at[2] - 12],
                    max: [at[0] + 12, at[1] + 6, at[2] + 12],
                    kinds: [
                        "coal", "iron", "copper", "gold", "redstone", "lapis", "diamond", "emerald",
                    ]
                    .iter()
                    .flat_map(|ore| {
                        [
                            format!("minecraft:{ore}_ore"),
                            format!("minecraft:deepslate_{ore}_ore"),
                        ]
                    })
                    .collect(),
                },
                0,
            )]
        }
        Action::OreMarked => pos
            .map(|ore| vec![(torch(ids, [ore[0], ore[1] + 1, ore[2]], 1), 0)])
            .unwrap_or_default(),
        Action::WallSwap(_) => pos
            .and_then(|p| memory.walls.iter().find(|(q, _)| *q == p))
            .and_then(|(p, id)| {
                // Another member of the block's family, by the seed: any wood, any colour.
                let others = crate::memory::cousins(id);
                (!others.is_empty()).then(|| (*p, id.clone(), others[seed % others.len()].clone()))
            })
            .map(|(p, id, cousin)| vec![(set(ids, p, &cousin, &[id.as_str()]), 0)])
            .unwrap_or_default(),
        Action::BedBreaks => pos
            .map(|bed| {
                vec![
                    (
                        Command::CancelSleep {
                            id: ids(),
                            player: target,
                            seconds: 0.0,
                        },
                        0,
                    ),
                    (
                        Command::BreakBlock {
                            id: ids(),
                            pos: bed,
                        },
                        3,
                    ),
                ]
            })
            .unwrap_or_default(),
        Action::Figure(where_, seconds, vanish) => pos
            .map(|p| {
                let on_ground = p[1] <= -1_000;
                let standing = if on_ground {
                    [p[0] as f64 + 0.5, -10_000.0, p[2] as f64 + 0.5]
                } else {
                    [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5]
                };
                let path: Vec<Pos> = match where_ {
                    FigureWhere::Route => memory
                        .route()
                        .map(|r| {
                            r.points
                                .iter()
                                .map(|q| [q[0] as f64 + 0.5, -10_000.0, q[2] as f64 + 0.5])
                                .collect()
                        })
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };
                vec![(
                    Command::Figure {
                        id: ids(),
                        figure: 1,
                        skin_of: target,
                        pos: standing,
                        path,
                        held: memory
                            .favourites
                            .first()
                            .map(|(id, _)| ItemStack {
                                id: id.clone(),
                                count: 1,
                                slot: 0,
                            })
                            .unwrap_or_default(),
                        name_shown: matches!(where_, FigureWhere::Far | FigureWhere::Behind),
                        seconds,
                        vanish_within: vanish,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Until dawn: the night's remaining ticks, a minute at least, ten minutes at most.
        Action::HostilesStare => {
            let to_dawn = 23_000u32.saturating_sub(now.day_time) as f32 / 20.0;
            let seconds = to_dawn.clamp(60.0, 600.0);
            memory
                .last
                .hostiles_near
                .iter()
                .take(16)
                .map(|mob| {
                    (
                        Command::Hold {
                            id: ids(),
                            entity: mob.id,
                            player: target,
                            seconds,
                        },
                        0,
                    )
                })
                .collect()
        }
        Action::Answers => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: "says".to_string(),
                text: words(Kind::Reply, "", 0),
            },
            0,
        )],
        Action::RepeatsYou => {
            if memory.last_said.is_empty() {
                return vec![];
            }
            vec![(
                Command::GameMessage {
                    id: ids(),
                    player: target,
                    message: "says".to_string(),
                    text: memory.last_said.clone(),
                },
                0,
            )]
        }
    }
}

/// Whether the event's place may be the model's `Where?` answer rather than the memory's
/// default: the things the lodger leaves or makes at a spot.
pub fn flexible(event: &Event) -> bool {
    matches!(
        event.action,
        Action::PlaceTaken
            | Action::Sign
            | Action::Cairn
            | Action::Hole
            | Action::Grave
            | Action::Sapling
    )
}

/// The places the memory can name for `Where?`, each with its words, in a fixed order so the
/// model's answer maps back.
pub fn named_places(memory: &Memory) -> Vec<(&'static str, BlockPos)> {
    let mut places = Vec::new();
    if let Some(home) = &memory.home {
        if let Some(door) = home.doors.first() {
            let step = step_toward(home.center, *door);
            places.push((
                "by the door",
                [door[0] + step[0], door[1], door[2] + step[1]],
            ));
        }
        if let Some(bed) = home.bed {
            places.push(("on the bed", bed));
        }
    }
    if let Some(chest) = memory.used_container() {
        places.push((
            "by the chest they use",
            [chest.pos[0] + 1, chest.pos[1], chest.pos[2]],
        ));
    }
    if let Some(end) = memory.route_end() {
        places.push(("at the end of their path", end));
    }
    if let Some(place) = memory.place {
        places.push(("at its place", place));
    }

    places
}

/// The kind of line an event writes, if it writes one: what the model is asked to write.
pub fn text_kind(event: &Event) -> Option<Kind> {
    match event.action {
        Action::Sign => Some(Kind::SignWord),
        Action::KillSign => Some(Kind::KillSign),
        Action::Grave => Some(Kind::GraveSign),
        Action::NameAnimal => Some(Kind::AnimalName),
        Action::Says => Some(Kind::ChatLine),
        Action::SaysKills => Some(Kind::KillLine),
        Action::Whisper => Some(Kind::Whisper),
        Action::Answers => Some(Kind::Reply),
        Action::RenameOne | Action::ReturnUsed => Some(Kind::ItemName),
        Action::BookWord => Some(Kind::SignWord),
        _ => None,
    }
}

/// A sign placed and written: the block now, the text a tick later.
fn sign_at(
    ids: &mut impl FnMut() -> u64,
    pos: BlockPos,
    lines: Vec<String>,
) -> Vec<(Command, u64)> {
    // On the ground within three blocks, else on a wall there; the text finds it where it
    // landed. The dev client's signs went into the air on a slope (2026-11-09).
    vec![
        (
            Command::SetBlock {
                id: ids(),
                pos,
                block: "minecraft:oak_sign".to_string(),
                state: String::new(),
                replace: GROUND_COVER.iter().map(|r| r.to_string()).collect(),
                settle: 3,
                wall: "minecraft:oak_wall_sign".to_string(),
                rest: false,
                only: false,
            },
            0,
        ),
        (
            Command::SetSign {
                id: ids(),
                pos,
                lines,
                settle: 3,
            },
            2,
        ),
    ]
}

/// The host took the favourite: the memory holds it now and the container's contents lose it.
pub fn took(memory: &mut Memory, pos: Option<BlockPos>) {
    let Some(pos) = pos else { return };
    let favourite = memory.favourites.first().map(|(id, _)| id.clone());
    let Some(container) = memory.containers.iter_mut().find(|c| c.pos == pos) else {
        return;
    };
    let Some(index) = container
        .contents
        .iter()
        .position(|s| Some(&s.id) == favourite.as_ref())
        .or_else(|| (!container.contents.is_empty()).then_some(0))
    else {
        return;
    };
    let stack = &mut container.contents[index];
    let held = Held {
        item: ItemStack {
            id: stack.id.clone(),
            count: 1,
            slot: 0,
        },
        from: pos,
        slot: stack.slot,
        taken_day: memory.done.last().map(|d| d.day).unwrap_or(0.0),
    };
    if stack.count <= 1 {
        container.contents.remove(index);
    } else {
        stack.count -= 1;
    }
    memory.held = Some(held);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{find, CATALOGUE};
    use crate::memory::{Container, Creature, Decor, Fixture, Home, Tally};

    fn stack(id: &str, count: u32, slot: u32) -> ItemStack {
        ItemStack {
            id: id.to_string(),
            count,
            slot,
        }
    }

    fn memory() -> Memory {
        let mut memory = Memory {
            target: Some(0),
            ..Memory::default()
        };
        memory.last.online = false;
        memory.home = Some(Home {
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![[1, 65, 1]],
        });
        memory
            .favourites
            .push(("minecraft:iron_pickaxe".to_string(), 9));
        memory.containers.push(Container {
            pos: [2, 64, 0],
            kind: "minecraft:chest".to_string(),
            opens: 5,
            placed: true,
            days: 1,
            in_home: false,
            last_opened_day: 0.0,
            contents: vec![
                stack("minecraft:iron_pickaxe", 1, 4),
                stack("minecraft:cobblestone", 40, 5),
                stack("minecraft:raw_iron", 6, 6),
                stack("minecraft:coal", 9, 7),
                stack("minecraft:oak_sapling", 2, 8),
            ],
        });
        memory
    }

    /// The memory with everything the catalogue could ask for.
    fn rich_memory() -> Memory {
        let mut memory = memory();
        // A book and quill and a map in the chest they use, for `book_word` and `map_marked`.
        memory.containers[0].contents.extend([
            stack("minecraft:writable_book", 1, 12),
            stack("minecraft:filled_map", 1, 13),
        ]);
        memory.containers.push(Container {
            pos: [-2, 64, 0],
            kind: "minecraft:chest".to_string(),
            opens: 2,
            placed: true,
            days: 1,
            in_home: false,
            last_opened_day: 0.0,
            contents: vec![stack("minecraft:dirt", 12, 0)],
        });
        memory.containers.push(Container {
            pos: [0, 64, 2],
            kind: "minecraft:furnace".to_string(),
            opens: 3,
            placed: true,
            days: 1,
            in_home: false,
            last_opened_day: 0.0,
            contents: vec![stack("minecraft:iron_ingot", 3, 2)],
        });
        for (kind, pos) in [
            ("crafting_table", [1, 64, 2]),
            ("hopper", [2, 65, 0]),
            ("shulker_box", [-4, 64, -4]),
            ("anvil", [-1, 64, 2]),
            ("lectern", [-1, 64, -2]),
            ("glass_pane", [0, 65, -3]),
            ("carpet", [1, 64, 1]),
            ("carpet", [2, 64, 1]),
            ("carpet", [3, 64, 1]),
            ("campfire", [6, 64, 6]),
            ("cauldron", [-2, 64, -2]),
            ("fence_gate", [8, 64, 0]),
            ("bell", [10, 65, 0]),
            ("note_block", [0, 64, -4]),
            ("lever", [2, 65, -2]),
            ("candle", [0, 65, 1]),
        ] {
            memory.fixtures.push(Fixture {
                kind: kind.to_string(),
                pos,
            });
        }
        for (kind, id, pos) in [
            ("item_frame", 101, [0, 66, -3]),
            ("item_frame", 102, [1, 66, -3]),
            ("item_frame", 103, [2, 66, -3]),
            ("painting", 104, [-2, 66, -3]),
            ("armor_stand", 105, [-2, 64, 2]),
        ] {
            memory.decor.push(Decor {
                kind: kind.to_string(),
                id,
                pos,
            });
        }
        for (kind, id, pet, named) in [
            ("cow", 201, false, false),
            ("sheep", 202, false, false),
            ("chicken", 203, false, false),
            ("villager", 204, false, false),
            ("iron_golem", 205, false, false),
            ("wolf", 206, true, true),
        ] {
            memory.last.creatures_near.push(Creature {
                id,
                kind: kind.to_string(),
                pos: [10.0, 64.0, 10.0],
                named,
                pet,
            });
        }
        for (kind, id) in [("zombie", 301), ("skeleton", 302), ("spider", 303)] {
            memory.last.hostiles_near.push(Creature {
                id,
                kind: kind.to_string(),
                pos: [20.0, 64.0, 20.0],
                ..Creature::default()
            });
        }
        memory.tally.push(Tally {
            kind: "cow".to_string(),
            hits: 20,
            kills: 14,
        });
        memory.last_said = "who is there".to_string();
        memory.last.raining = true;
        memory.last.underground = true;
        memory.last.ate_day = 0.0;
        memory.last.walking = true;
        memory.containers[0]
            .contents
            .push(stack("minecraft:water_bucket", 1, 9));
        memory.containers[0]
            .contents
            .push(stack("minecraft:oak_pressure_plate", 2, 10));
        memory.fixtures.push(Fixture {
            kind: "redstone_lamp".to_string(),
            pos: [0, 66, 2],
        });
        memory.vehicles.push(Creature {
            id: 301,
            kind: "oak_boat".to_string(),
            pos: [20.0, 63.0, 0.0],
            named: false,
            pet: false,
        });
        memory.vehicles.push(Creature {
            id: 302,
            kind: "minecart".to_string(),
            pos: [5.0, 64.0, 5.0],
            named: false,
            pet: false,
        });
        memory.surroundings = Some(crate::memory::Surround {
            day: 0.0,
            around: [0, 64, 0],
            features: vec![
                craftkit::world::Feature {
                    kind: "water".to_string(),
                    pos: [30, 62, 0],
                    size: 20,
                    detail: String::new(),
                },
                craftkit::world::Feature {
                    kind: "crops".to_string(),
                    pos: [-12, 64, 0],
                    size: 6,
                    detail: String::new(),
                },
                craftkit::world::Feature {
                    kind: "snow".to_string(),
                    pos: [0, 64, 12],
                    size: 30,
                    detail: String::new(),
                },
                craftkit::world::Feature {
                    kind: "village".to_string(),
                    pos: [60, 64, 60],
                    size: 40,
                    detail: String::new(),
                },
            ],
        });
        memory.scanned.push(("oak_door".to_string(), [58, 64, 61]));
        memory.scanned.push(("bell".to_string(), [60, 65, 60]));
        for (kind, pos) in [
            ("stairs", [4, 65, 3]),
            ("banner", [0, 66, 3]),
            ("piston", [-3, 64, 3]),
            ("jukebox", [-3, 64, -3]),
        ] {
            memory.fixtures.push(Fixture {
                kind: kind.to_string(),
                pos,
            });
        }
        for (id, count, slot) in [
            ("minecraft:iron_helmet", 1, 11),
            ("minecraft:poppy", 3, 12),
            ("minecraft:carrot", 5, 13),
            ("minecraft:red_wool", 4, 14),
            ("minecraft:blue_wool", 2, 15),
            ("minecraft:red_concrete", 7, 16),
            ("minecraft:lime_wool", 1, 17),
            ("minecraft:black_wool", 9, 18),
            ("minecraft:white_bed", 1, 19),
        ] {
            memory.containers[0].contents.push(stack(id, count, slot));
        }
        memory.containers[2]
            .contents
            .push(stack("minecraft:coal", 3, 1));
        memory.scanned.push(("iron_ore".to_string(), [20, 40, 20]));
        memory
            .walls
            .push(([5, 65, 5], "minecraft:oak_planks".to_string()));
        memory
            .walls
            .push(([5, 66, 5], "minecraft:glass_pane".to_string()));
        memory
            .walls
            .push(([1, 65, 1], "minecraft:torch".to_string()));
        memory.routes.push(crate::memory::Route {
            points: (0..20).map(|i| [3 + i * 2, 64, 0]).collect(),
            walked: 3,
            first_day: 0.0,
        });
        memory
    }

    fn ids() -> impl FnMut() -> u64 {
        let mut next = 1;
        move || {
            next += 1;
            next
        }
    }

    #[test]
    fn a_door_toggles_a_chest_opens_its_lid_and_a_take_removes_the_favourite() {
        let memory = memory();
        let mut rng = Rng::new(1);

        let door = find("open_door").unwrap();
        let p = place(door, &memory, &mut rng);
        assert_eq!(p, Some([3, 64, 0]));
        let made = commands(door, p, &memory, &mut ids(), &Now::default(), None);
        assert!(matches!(
            made[0].0,
            Command::ToggleBlock {
                pos: [3, 64, 0],
                ..
            }
        ));

        let chest = find("open_chest").unwrap();
        let made = commands(
            chest,
            place(chest, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::OpenLid {
                    pos: [2, 64, 0],
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let take = find("take").unwrap();
        let made = commands(
            take,
            place(take, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::TakeItem {
                    from: [2, 64, 0],
                    slot: 4,
                    count: 1,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn taking_moves_the_item_from_the_container_to_the_hand_and_returning_puts_it_back() {
        let mut memory = memory();
        took(&mut memory, Some([2, 64, 0]));

        let held = memory.held.as_ref().unwrap();
        assert_eq!(held.item.id, "minecraft:iron_pickaxe");
        assert_eq!(held.slot, 4);
        assert!(memory.containers[0]
            .contents
            .iter()
            .all(|s| s.id != "minecraft:iron_pickaxe"));

        let back = find("return_taken").unwrap();
        let mut rng = Rng::new(1);
        let made = commands(
            back,
            place(back, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::PutItem {
                    into: [2, 64, 0],
                    slot: Some(4),
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn shuffling_moves_a_stack_to_an_empty_slot_and_splitting_halves_one() {
        let memory = memory();
        let mut rng = Rng::new(1);

        let shuffle = find("shuffle_chest").unwrap();
        let made = commands(
            shuffle,
            place(shuffle, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        match &made[0].0 {
            Command::MoveItem {
                from, to, to_slot, ..
            } => {
                assert_eq!(from, to);
                assert!(![4, 5, 6, 7, 8].contains(to_slot));
            }
            other => panic!("{other:?}"),
        }

        let split = find("stack_split").unwrap();
        let made = commands(
            split,
            place(split, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::MoveItem {
                    from_slot: 5,
                    count: 20,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn tidying_sorts_a_chest_into_its_first_slots_with_one_empty_slot_as_the_hand() {
        let contents = vec![
            stack("minecraft:torch", 5, 0),
            stack("minecraft:apple", 3, 1),
            stack("minecraft:cobblestone", 40, 9),
        ];
        let moves = tidy_moves(&contents, 27);

        // Replay the moves and check the order that comes out.
        let mut slots: Vec<Option<ItemStack>> = vec![None; 27];
        for s in &contents {
            slots[s.slot as usize] = Some(s.clone());
        }
        for (from, to, _) in &moves {
            assert!(slots[*to as usize].is_none(), "move onto a full slot");
            slots[*to as usize] = slots[*from as usize].take();
        }
        let ids: Vec<&str> = slots
            .iter()
            .take(3)
            .map(|s| s.as_ref().unwrap().id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "minecraft:apple",
                "minecraft:cobblestone",
                "minecraft:torch"
            ]
        );

        let full: Vec<ItemStack> = (0..27).map(|i| stack("minecraft:dirt", 1, i)).collect();
        assert!(tidy_moves(&full, 27).is_empty(), "no hand, no tidying");
    }

    #[test]
    fn cooking_moves_a_raw_thing_and_a_fuel_and_taking_output_empties_the_furnace() {
        let memory = rich_memory();
        let mut rng = Rng::new(1);

        let cook = find("cook_one").unwrap();
        let made = commands(
            cook,
            place(cook, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        // The furnace holds fuel already, so one move: the raw thing. Cobblestone, in slot 5,
        // is the first smeltable thing in the chest.
        assert_eq!(made.len(), 1);
        assert!(
            matches!(
                made[0].0,
                Command::MoveItem {
                    from_slot: 5,
                    to_slot: 0,
                    count: 1,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let out = find("take_output").unwrap();
        let made = commands(
            out,
            place(out, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::MoveItem {
                    from: [0, 64, 2],
                    from_slot: 2,
                    to: [2, 64, 0],
                    count: 3,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn a_kill_sign_counts_the_kills_and_a_grave_names_the_kind() {
        let memory = rich_memory();
        let mut rng = Rng::new(1);

        let sign = find("kill_sign").unwrap();
        let made = commands(
            sign,
            place(sign, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::SetBlock {
                    pos: [5, 64, 0],
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
        match &made[1].0 {
            Command::SetSign { lines, .. } => assert!(
                lines[0].contains("14") || lines[0].contains("cow"),
                "{lines:?}"
            ),
            other => panic!("{other:?}"),
        }

        let grave = find("grave").unwrap();
        let made = commands(
            grave,
            place(grave, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 4);
        assert!(
            matches!(&made[3].0, Command::SetSign { lines, .. } if lines[0].contains("cow") || lines[0] == "rest")
        );

        let line = find("kill_line").unwrap();
        let made = commands(line, None, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[0].0, Command::GameMessage { message, text, .. } if message == "says" && text.contains("cow"))
        );
    }

    #[test]
    fn a_worn_path_replaces_grass_only_and_a_hole_digs_natural_ground_only() {
        let memory = rich_memory();
        let mut rng = Rng::new(1);

        let path = find("path_worn").unwrap();
        let made = commands(
            path,
            place(path, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(made.len() >= 6, "{} blocks of path", made.len());
        for (command, _) in &made {
            match command {
                Command::SetBlock { block, replace, .. } => {
                    assert_eq!(block, "minecraft:dirt_path");
                    assert_eq!(replace, &["minecraft:grass_block".to_string()]);
                }
                other => panic!("{other:?}"),
            }
        }

        let hole = find("hole").unwrap();
        let made = commands(
            hole,
            place(hole, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 4);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, replace, .. } if block == "minecraft:air" && replace.contains(&"minecraft:grass_block".to_string()) && !replace.iter().any(|r| r.contains("planks")))
        );
        assert!(
            matches!(&made[3].0, Command::SetBlock { block, .. } if block == "minecraft:torch")
        );
    }

    #[test]
    fn every_event_makes_at_least_one_command_when_the_memory_has_everything() {
        let memory = rich_memory();
        let mut rng = Rng::new(7);
        let now = Now::default();

        for event in CATALOGUE {
            if event.id == "wait"
                || event.id == "return_taken"
                || event.id == "place_taken"
                || event.id == "returned_used"
                || event.id == "door_opens_itself"
            {
                continue;
            }
            let pos = place(event, &memory, &mut rng);
            let made = commands(event, pos, &memory, &mut ids(), &now, None);
            assert!(!made.is_empty(), "{} makes no command", event.id);
            for (command, _) in &made {
                assert!(command.id() > 1, "{} gave a command no id", event.id);
                // No made-up ids: `woke_elsewhere` sent the player as entity u32::MAX - 0,
                // which no host knows (2026-11-09).
                let text = serde_json::to_string(command).unwrap();
                assert!(
                    !text.contains("4294967") && !text.contains(&format!("{}", i64::MAX)),
                    "{} sent a placeholder id: {text}",
                    event.id
                );
            }
        }
    }

    #[test]
    fn boats_move_across_the_water_or_drift_and_the_village_is_scanned_then_opened() {
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let now = Now::default();

        let moved = find("boat_moved").unwrap();
        let made = commands(moved, None, &memory, &mut ids(), &now, None);
        match &made[0].0 {
            Command::MoveEntity { entity, to, .. } => {
                assert_eq!(*entity, 301);
                assert!((to[0] - 40.0).abs() < 0.01, "across the water: {to:?}");
            }
            other => panic!("{other:?}"),
        }

        let drift = find("boat_drifts").unwrap();
        let made = commands(drift, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(made[0].0, Command::PushEntity { entity: 301, .. }),
            "{:?}",
            made[0].0
        );
        let rolls = find("minecart_rolls").unwrap();
        let made = commands(rolls, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(made[0].0, Command::PushEntity { entity: 302, .. }),
            "{:?}",
            made[0].0
        );

        let scan = find("scan_village").unwrap();
        let made = commands(scan, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::Scan { kinds, .. } if kinds.iter().any(|k| k == "minecraft:bell"))
        );
        let doors = find("village_doors").unwrap();
        let made = commands(
            doors,
            place(doors, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::ToggleBlock {
                    pos: [58, 64, 61],
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let trample = find("trample").unwrap();
        let made = commands(
            trample,
            place(trample, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(made.len() >= 2);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, replace, .. } if block == "minecraft:dirt" && replace == &["minecraft:farmland".to_string()])
        );
    }

    #[test]
    fn an_animal_is_led_to_the_far_end_of_the_route_and_the_path_of_torches_follows_it() {
        let memory = rich_memory();
        let mut rng = Rng::new(4);
        let now = Now::default();

        assert_eq!(memory.route_end(), Some([41, 64, 0]));
        let gone = find("animal_gone").unwrap();
        let p = place(gone, &memory, &mut rng);
        assert_eq!(p, Some([41, 64, 0]));
        let made = commands(gone, p, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::MoveEntity { entity: 201, to, .. } if (to[0] - 41.5).abs() < 0.01),
            "{:?}",
            made[0].0
        );

        let path = find("light_path").unwrap();
        let made = commands(path, None, &memory, &mut ids(), &now, None);
        assert!(made.len() >= 3);
        for (command, _) in &made {
            match command {
                Command::SetBlock {
                    pos, settle, wall, ..
                } => {
                    assert!(pos[0] > 0 && pos[2] == 0, "toward the route's end: {pos:?}");
                    // The dev client, 2026-11-09: the path floated over a slope.
                    assert!(*settle >= 3, "the host may find the ground: {settle}");
                    assert_eq!(wall, "minecraft:wall_torch");
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn digging_is_air_into_ground_and_a_worn_path_or_a_puddle_is_not() {
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        assert_eq!(
            dug(&set(&mut ids, [1, 63, 1], "minecraft:air", GROUND)),
            Some([1, 63, 1])
        );
        assert_eq!(
            dug(&set(
                &mut ids,
                [1, 63, 1],
                "minecraft:dirt_path",
                &["minecraft:grass_block"]
            )),
            None
        );
        assert_eq!(
            dug(&set(&mut ids, [1, 64, 1], "minecraft:water", GROUND)),
            None
        );
        assert_eq!(
            dug(&set(
                &mut ids,
                [1, 64, 1],
                "minecraft:air",
                &["minecraft:snow"]
            )),
            None,
            "clearing snow is not digging"
        );
    }

    #[test]
    fn a_hole_is_dug_clear_of_what_they_built_or_not_at_all() {
        let mut memory = Memory::default();
        memory.home = Some(crate::memory::Home {
            center: [0, 64, 0],
            floor_y: 64,
            ..Default::default()
        });
        memory.place = Some([10, 64, 0]);
        memory.placed.push([12, 64, 0]); // they built up to its place
        let hole = find("hole").unwrap();
        for seed in 0..20 {
            let spot = place(hole, &memory, &mut Rng::new(seed)).expect("a clear spot");
            assert!(clear_of_built(&memory, spot), "{spot:?}");
            assert_ne!(spot, [10, 64, 0]);
        }

        // Built all around, out to twenty blocks: nowhere to dig.
        for x in (-24..=24).step_by(4) {
            for z in (-24..=24).step_by(4) {
                memory.placed.push([x, 64, z]);
            }
        }
        assert_eq!(place(hole, &memory, &mut Rng::new(3)), None);
    }

    #[test]
    fn what_it_builds_lands_on_the_ground_and_ice_only_on_water() {
        // The dev client, 2026-11-09: signs in the air, cairns in a village's stairs, ice in
        // the lake bed.
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let now = Now::default();
        let made_by = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        for (command, _) in made_by("cairn", &mut rng) {
            assert!(
                matches!(&command, Command::SetBlock { rest: true, settle, .. } if *settle >= 3),
                "{command:?}"
            );
        }
        let sign = made_by("sign", &mut rng);
        assert!(
            matches!(&sign[0].0, Command::SetBlock { wall, settle, .. } if wall == "minecraft:oak_wall_sign" && *settle >= 3),
            "{:?}",
            sign[0].0
        );
        assert!(
            matches!(&sign[1].0, Command::SetSign { settle, .. } if *settle >= 3),
            "the text looks where the sign landed"
        );
        for (command, _) in made_by("ice_path", &mut rng) {
            assert!(
                matches!(&command, Command::SetBlock { only: true, replace, .. } if replace == &["minecraft:water".to_string()]),
                "{command:?}"
            );
        }
        let sapling = made_by("sapling", &mut rng);
        assert!(
            matches!(sapling[0].0, Command::SetBlock { .. })
                && matches!(sapling[1].0, Command::TakeItem { .. })
                && sapling[1].1 > sapling[0].1,
            "planted before it is taken: {sapling:?}"
        );
    }

    #[test]
    fn what_it_took_comes_back_itself_worn_or_named_and_a_thing_in_the_chest_gets_a_name() {
        let mut memory = rich_memory();
        let now = Now::default();
        let held = |id: &str| crate::memory::Held {
            item: stack(id, 1, 0),
            from: [2, 64, 0],
            slot: 4,
            ..Default::default()
        };

        memory.held = Some(held("minecraft:iron_pickaxe"));
        let used = find("returned_used").unwrap();
        let made = commands(used, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::PutItem { kept, wear, name, .. } if kept == "held" && *wear > 0 && name.is_empty()),
            "the pickaxe itself, worn: {:?}",
            made[0].0
        );
        memory.held = Some(held("minecraft:bread"));
        let made = commands(used, None, &memory, &mut ids(), &now, Some("borrowed"));
        assert!(
            matches!(&made[0].0, Command::PutItem { wear: 0, name, .. } if name == "borrowed"),
            "bread is named, not worn: {:?}",
            made[0].0
        );

        let take = commands(find("take").unwrap(), None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&take[0].0, Command::TakeItem { keep, .. } if keep == "held"),
            "what it takes the host keeps: {:?}",
            take[0].0
        );
        let back = commands(
            find("return_taken").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(matches!(&back[0].0, Command::PutItem { kept, .. } if kept == "held"));

        let rename = find("renamed_item").unwrap();
        let made = commands(
            rename,
            Some([2, 64, 0]),
            &memory,
            &mut ids(),
            &now,
            Some("mine"),
        );
        assert!(
            matches!(&made[0].0, Command::RenameItem { name, slot: 4, .. } if name == "mine"),
            "the favourite, the pickaxe in slot 4: {:?}",
            made[0].0
        );
    }

    #[test]
    fn the_book_is_signed_with_one_word_and_the_map_marked_at_its_place() {
        let mut memory = rich_memory();
        memory.place = Some([40, 64, -12]);
        let now = Now::default();
        let book = commands(
            find("book_word").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            Some("under"),
        );
        assert!(
            matches!(&book[0].0, Command::SignBook { slot: 12, title, pages, .. } if title == "under" && pages == &["under".to_string()]),
            "{:?}",
            book[0].0
        );
        let map = commands(
            find("map_marked").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&map[0].0, Command::MarkMap { slot: 13, at, .. } if *at == [40, 64, -12]),
            "{:?}",
            map[0].0
        );
    }

    #[test]
    fn the_hopper_empties_into_its_chest_and_the_box_moves_with_what_it_holds() {
        let memory = rich_memory();
        let now = Now::default();
        let mut rng = Rng::new(2);
        let hopper = find("hopper_emptied").unwrap();
        let made = commands(
            hopper,
            place(hopper, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert_eq!(made.len(), 5, "its five slots");
        assert!(
            made.iter().all(|(c, _)| matches!(c, Command::MoveItem { from, to, .. } if *from == [2, 65, 0] && *to == [2, 64, 0])),
            "from the hopper into the chest under it: {made:?}"
        );
        let shulker = find("shulker_moved").unwrap();
        let made = commands(
            shulker,
            place(shulker, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&made[0].0, Command::MoveBlock { from, around: 2, to, .. } if *from == [-4, 64, -4] && to != from),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn waking_elsewhere_moves_the_player_and_the_animals_stand_still_to_stare() {
        let memory = rich_memory();
        let now = Now::default();
        let woke = find("woke_elsewhere").unwrap();
        let bed = memory.home.as_ref().unwrap().bed;
        let made = commands(woke, bed, &memory, &mut ids(), &now, None);
        assert!(
            made.iter().any(|(c, _)| matches!(c, Command::MovePlayer { player, .. } if *player == memory.target.unwrap_or(0))),
            "{made:?}"
        );

        let stare = find("stare").unwrap();
        let made = commands(stare, None, &memory, &mut ids(), &now, None);
        assert!(!made.is_empty());
        for (command, _) in &made {
            assert!(
                matches!(command, Command::Hold { seconds, .. } if (10.0..=30.0).contains(seconds)),
                "{command:?}"
            );
        }
    }

    #[test]
    fn the_monsters_near_are_held_staring_until_dawn_and_no_longer_than_ten_minutes() {
        let memory = rich_memory();
        let stare = find("hostiles_stare").unwrap();
        let at = |day_time| Now {
            day_time,
            ..Now::default()
        };
        let made = commands(stare, None, &memory, &mut ids(), &at(18_000), None);
        assert_eq!(made.len(), 3, "one hold for each of the three");
        for (command, later) in &made {
            match command {
                Command::Hold {
                    seconds, player, ..
                } => {
                    assert_eq!(*seconds, 250.0, "5000 ticks to dawn");
                    assert_eq!(*player, memory.target.unwrap_or(0));
                    assert_eq!(*later, 0);
                }
                other => panic!("{other:?}"),
            }
        }
        let early = commands(stare, None, &memory, &mut ids(), &at(13_000), None);
        assert!(matches!(early[0].0, Command::Hold { seconds, .. } if seconds == 500.0));
        let late = commands(stare, None, &memory, &mut ids(), &at(22_900), None);
        assert!(
            matches!(late[0].0, Command::Hold { seconds, .. } if seconds == 60.0),
            "a minute at least"
        );
        assert!(
            !crate::rules::hidden_from_view(stare.action),
            "it is about being seen"
        );
    }

    #[test]
    fn a_wall_block_becomes_its_cousin_and_the_bed_breaks_into_its_item() {
        let memory = rich_memory();
        let mut rng = Rng::new(8);
        let now = Now::default();

        let swap = find("window_bars").unwrap();
        let p = place(swap, &memory, &mut rng);
        assert_eq!(p, Some([5, 66, 5]));
        let made = commands(swap, p, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, replace, .. } if block == "minecraft:iron_bars" && replace == &["minecraft:glass_pane".to_string()]),
            "{:?}",
            made[0].0
        );

        let torch = find("torch_blue").unwrap();
        let made = commands(
            torch,
            place(torch, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, replace, .. } if block != "minecraft:torch" && block.ends_with("torch") && replace == &["minecraft:torch".to_string()]),
            "{:?}",
            made[0].0
        );

        let wall = find("wall_swap").unwrap();
        let made = commands(wall, Some([5, 65, 5]), &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, .. } if block.ends_with("_planks") && block != "minecraft:oak_planks"),
            "{:?}",
            made[0].0
        );
        assert_eq!(
            crate::memory::family("minecraft:cherry_planks").len(),
            crate::memory::WOODS.len()
        );
        assert_eq!(
            crate::memory::cousins("minecraft:lime_wool").len(),
            crate::memory::COLOURS.len() - 1
        );
        assert!(
            crate::memory::cousins("minecraft:oak_door").is_empty(),
            "doors are two blocks and stay"
        );

        let bed = find("bed_breaks").unwrap();
        let made = commands(
            bed,
            place(bed, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(matches!(made[0].0, Command::CancelSleep { .. }));
        assert!(
            matches!(
                made[1].0,
                Command::BreakBlock {
                    pos: [0, 64, 0],
                    ..
                }
            ),
            "{:?}",
            made[1].0
        );
        assert_eq!(
            crate::memory::cousins("minecraft:chest"),
            Vec::<String>::new(),
            "a chest has no family"
        );
    }

    #[test]
    fn frames_turn_the_same_way_stands_follow_the_target_and_the_pet_moves_to_a_corner() {
        let memory = rich_memory();
        let mut rng = Rng::new(2);
        let now = Now::default();

        let frames = find("frames_turn").unwrap();
        let made = commands(frames, None, &memory, &mut ids(), &now, None);
        assert_eq!(made.len(), 3);
        let rotations: Vec<Option<u32>> = made
            .iter()
            .map(|(c, _)| match c {
                Command::RotateFrame { rotation, .. } => *rotation,
                other => panic!("{other:?}"),
            })
            .collect();
        assert!(rotations.iter().all(|r| r.is_some() && *r == rotations[0]));

        let watch = find("stand_watch").unwrap();
        let made = commands(watch, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(
                made[0].0,
                Command::PoseStand {
                    entity: 105,
                    follow: Some(0),
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let pet = find("pet_moved").unwrap();
        let made = commands(
            pet,
            place(pet, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(made[0].0, Command::MoveEntity { entity: 206, .. }),
            "{:?}",
            made[0].0
        );

        let villager = find("villager_at_door").unwrap();
        let made = commands(
            villager,
            place(villager, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(made[0].0, Command::MoveEntity { entity: 204, .. }),
            "{:?}",
            made[0].0
        );
    }
}
