//! From a decided event to the host's commands, resolved against the memory at the moment it
//! runs. Each action here is one of the general commands the host carries out; the event's
//! `fits` is the host's check when the command arrives, and `ready` is the mind's check that the
//! moment has come.

use crate::brain::text::{self, Kind};
use crate::events::{Action, Event, Need, Where};
use crate::memory::{Held, Memory};
use crate::model::Rng;
use craftkit::world::{BlockPos, Command, ItemStack, Now, Pos};

/// Whether the event's moment is here: its needs still hold.
pub fn ready(event: &Event, memory: &Memory, _config: &crate::config::Config) -> bool {
    let now = Now::default();
    event.needs.iter().all(|need| {
        matches!(*need, Need::Night | Need::Day) || super::offer::meets(*need, memory, &now)
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

/// The place the event happens, from the memory, or none for a thing with no place.
pub fn place(event: &Event, memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    let home = memory.home.as_ref();

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
        | Action::ScanVillage => None,
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
        Action::ReturnTaken => memory.held.as_ref().map(|h| h.from),
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
        Action::Hole | Action::Cairn | Action::Sapling | Action::Sign | Action::Grave => {
            spot(memory, rng)
        }
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
    _now: &Now,
) -> Vec<(Command, u64)> {
    let target = memory.target.unwrap_or(0);
    let at = block_center;
    let seed = pos
        .map(|p| (p[0] + p[2]).unsigned_abs() as usize)
        .unwrap_or(0)
        + memory.done.len();

    match event.action {
        Action::Wait => vec![],
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
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::AddTorch => pos
            .map(|p| vec![(set(ids, [p[0], p[1] + 1, p[2]], "minecraft:torch", &[]), 0)])
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
                made.push((
                    set(ids, [p[0], p[1] + 1, p[2]], "minecraft:torch", &[]),
                    i as u64 * 20,
                ));
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
                },
                0,
            )],
            None => vec![],
        },
        Action::PlaceTaken => match (&memory.held, pos) {
            (Some(held), Some(p)) => vec![(
                Command::DropItem {
                    id: ids(),
                    pos: [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5],
                    item: held.item.clone(),
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
        Action::Stare => memory
            .creatures("")
            .take(8)
            .map(|creature| {
                (
                    Command::LookAt {
                        id: ids(),
                        entity: creature.id,
                        at: memory.last.pos,
                        seconds: 20.0,
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
                made.push((set(ids, [p[0], p[1] - 2, p[2]], "minecraft:torch", &[]), 50));
                made
            })
            .unwrap_or_default(),
        Action::Cairn => pos
            .map(|p| {
                (0..3)
                    .map(|height| {
                        (
                            set(
                                ids,
                                [p[0], p[1] + 1 + height, p[2]],
                                "minecraft:cobblestone",
                                &[],
                            ),
                            height as u64 * 15,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Action::Sapling => match (pos, memory.in_used_container(|id| id.ends_with("sapling"))) {
            (Some(p), Some((container, stack))) => vec![
                (
                    Command::TakeItem {
                        id: ids(),
                        from: container.pos,
                        slot: stack.slot,
                        count: 1,
                    },
                    0,
                ),
                (
                    set(
                        ids,
                        [p[0], p[1] + 1, p[2]],
                        &stack.id,
                        &["minecraft:short_grass"],
                    ),
                    1,
                ),
            ],
            _ => vec![],
        },
        Action::Sign => pos
            .map(|p| {
                let word = text::pick(Kind::SignWord, seed, "", 0);
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
                    vec![text::pick(Kind::KillSign, seed, &tally.kind, tally.kills)],
                )
            }
            _ => vec![],
        },
        Action::Grave => match (pos, memory.most_killed()) {
            (Some(p), Some(tally)) => {
                let mut made = vec![
                    (
                        set(ids, [p[0], p[1] + 1, p[2]], "minecraft:cobblestone", &[]),
                        0,
                    ),
                    (
                        set(ids, [p[0], p[1] + 2, p[2]], "minecraft:cobblestone", &[]),
                        15,
                    ),
                ];
                made.extend(
                    sign_at(
                        ids,
                        [p[0] + 1, p[1] + 1, p[2]],
                        vec![text::pick(Kind::GraveSign, seed, &tally.kind, tally.kills)],
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
                        name: text::pick(Kind::AnimalName, seed, "", 0),
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
                if event.id == "pet_moved" {
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
                text: text::pick(Kind::ChatLine, seed, "", 0),
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
                        text: text::pick(Kind::KillLine, seed, &tally.kind, tally.kills),
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
                text: text::pick(Kind::Whisper, seed, "", 0),
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

/// A sign placed and written: the block now, the text a tick later.
fn sign_at(
    ids: &mut impl FnMut() -> u64,
    pos: BlockPos,
    lines: Vec<String>,
) -> Vec<(Command, u64)> {
    vec![
        (
            set(ids, pos, "minecraft:oak_sign", &["minecraft:short_grass"]),
            0,
        ),
        (
            Command::SetSign {
                id: ids(),
                pos,
                lines,
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
        memory.containers.push(Container {
            pos: [-2, 64, 0],
            kind: "minecraft:chest".to_string(),
            opens: 2,
            last_opened_day: 0.0,
            contents: vec![stack("minecraft:dirt", 12, 0)],
        });
        memory.containers.push(Container {
            pos: [0, 64, 2],
            kind: "minecraft:furnace".to_string(),
            opens: 3,
            last_opened_day: 0.0,
            contents: vec![stack("minecraft:iron_ingot", 3, 2)],
        });
        for (kind, pos) in [
            ("crafting_table", [1, 64, 2]),
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
        let made = commands(door, p, &memory, &mut ids(), &Now::default());
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
        );
        assert_eq!(made.len(), 2);
        assert!(
            matches!(
                made[0].0,
                // Cobblestone, in slot 5, is the first smeltable thing in the chest.
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
        assert!(
            matches!(
                made[1].0,
                Command::MoveItem {
                    from_slot: 7,
                    to_slot: 1,
                    count: 1,
                    ..
                }
            ),
            "{:?}",
            made[1].0
        );

        let out = find("take_output").unwrap();
        let made = commands(
            out,
            place(out, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
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
        );
        assert_eq!(made.len(), 4);
        assert!(
            matches!(&made[3].0, Command::SetSign { lines, .. } if lines[0].contains("cow") || lines[0] == "rest")
        );

        let line = find("kill_line").unwrap();
        let made = commands(line, None, &memory, &mut ids(), &Now::default());
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
                || event.id == "door_opens_itself"
            {
                continue;
            }
            let pos = place(event, &memory, &mut rng);
            let made = commands(event, pos, &memory, &mut ids(), &now);
            assert!(!made.is_empty(), "{} makes no command", event.id);
            for (command, _) in &made {
                assert!(command.id() > 1, "{} gave a command no id", event.id);
            }
        }
    }

    #[test]
    fn boats_move_across_the_water_or_drift_and_the_village_is_scanned_then_opened() {
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let now = Now::default();

        let moved = find("boat_moved").unwrap();
        let made = commands(moved, None, &memory, &mut ids(), &now);
        match &made[0].0 {
            Command::MoveEntity { entity, to, .. } => {
                assert_eq!(*entity, 301);
                assert!((to[0] - 40.0).abs() < 0.01, "across the water: {to:?}");
            }
            other => panic!("{other:?}"),
        }

        let drift = find("boat_drifts").unwrap();
        let made = commands(drift, None, &memory, &mut ids(), &now);
        assert!(
            matches!(made[0].0, Command::PushEntity { entity: 301, .. }),
            "{:?}",
            made[0].0
        );
        let rolls = find("minecart_rolls").unwrap();
        let made = commands(rolls, None, &memory, &mut ids(), &now);
        assert!(
            matches!(made[0].0, Command::PushEntity { entity: 302, .. }),
            "{:?}",
            made[0].0
        );

        let scan = find("scan_village").unwrap();
        let made = commands(scan, None, &memory, &mut ids(), &now);
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
        let made = commands(gone, p, &memory, &mut ids(), &now);
        assert!(
            matches!(&made[0].0, Command::MoveEntity { entity: 201, to, .. } if (to[0] - 41.5).abs() < 0.01),
            "{:?}",
            made[0].0
        );

        let path = find("light_path").unwrap();
        let made = commands(path, None, &memory, &mut ids(), &now);
        assert!(made.len() >= 3);
        for (command, _) in &made {
            match command {
                Command::SetBlock { pos, .. } => {
                    assert!(pos[0] > 0 && pos[2] == 0, "toward the route's end: {pos:?}")
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn frames_turn_the_same_way_stands_follow_the_target_and_the_pet_moves_to_a_corner() {
        let memory = rich_memory();
        let mut rng = Rng::new(2);
        let now = Now::default();

        let frames = find("frames_turn").unwrap();
        let made = commands(frames, None, &memory, &mut ids(), &now);
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
        let made = commands(watch, None, &memory, &mut ids(), &now);
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
        );
        assert!(
            matches!(made[0].0, Command::MoveEntity { entity: 204, .. }),
            "{:?}",
            made[0].0
        );
    }
}
