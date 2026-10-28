//! The sound events' variants: each plays one of a few fitting sounds, at its own pitch, rhythm
//! and movement, never the same variant twice running (owner, 2026-11-09: "3 to 5 each", "so it
//! never sounds the same twice"). A sound of `@step`, `@place` or `@break` is the sound of a
//! real block, the one under the target's feet or in their hand (`block_sound`), so footsteps
//! on grass sound like grass.

use crate::memory::Memory;
use craftkit::world::{BlockPos, Command, Pos};

/// Where a variant's sounds come from, over its beats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moves {
    /// From the event's spot, every beat.
    Still,
    /// Behind the target, a few blocks, every beat (their own steps, a moment late).
    Behind,
    /// From eight blocks off toward two, as if coming closer.
    Closer,
    /// From two blocks off to eight, going away.
    Away,
}

/// One way an event can sound: its beats (a sound and the tick it plays at), a pitch drawn
/// between two, a volume, and how it moves.
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub beats: &'static [(&'static str, u64)],
    pub pitch: (f32, f32),
    pub volume: f32,
    pub moves: Moves,
}

const fn v(
    beats: &'static [(&'static str, u64)],
    pitch: (f32, f32),
    volume: f32,
    moves: Moves,
) -> Variant {
    Variant {
        beats,
        pitch,
        volume,
        moves,
    }
}

use Moves::*;

const DOOR_OPEN: &str = "minecraft:block.wooden_door.open";
const DOOR_CLOSE: &str = "minecraft:block.wooden_door.close";

/// The variants of a sound event, or none for an event that is not one.
pub fn variants(event: &str) -> Option<&'static [Variant]> {
    Some(match event {
        "door_sound" => {
            const WAYS: &[Variant] = &[
                v(&[(DOOR_OPEN, 0)], (0.9, 1.0), 0.8, Still),
                v(&[(DOOR_CLOSE, 0)], (0.8, 0.9), 0.6, Still),
                v(&[(DOOR_OPEN, 0), (DOOR_CLOSE, 30)], (0.85, 1.0), 0.7, Still),
                v(
                    &[("minecraft:block.wooden_trapdoor.open", 0)],
                    (0.6, 0.7),
                    0.7,
                    Still,
                ),
                v(&[(DOOR_CLOSE, 0)], (0.7, 0.8), 0.3, Still),
            ];
            WAYS
        }
        "chest_sound" => {
            const WAYS: &[Variant] = &[
                v(&[("minecraft:block.chest.open", 0)], (0.9, 1.0), 0.7, Still),
                v(
                    &[("minecraft:block.chest.close", 0)],
                    (0.9, 1.0),
                    0.7,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.chest.open", 0),
                        ("minecraft:block.chest.close", 25),
                    ],
                    (0.85, 1.0),
                    0.7,
                    Still,
                ),
                v(
                    &[("minecraft:block.barrel.open", 0)],
                    (0.8, 0.9),
                    0.6,
                    Still,
                ),
                v(
                    &[("minecraft:block.chest.locked", 0)],
                    (0.7, 0.8),
                    0.5,
                    Still,
                ),
            ];
            WAYS
        }
        "late_steps" => {
            const WAYS: &[Variant] = &[
                v(&[("@step", 0), ("@step", 8)], (0.9, 1.0), 0.5, Behind),
                v(
                    &[("@step", 0), ("@step", 8), ("@step", 16), ("@step", 24)],
                    (0.9, 1.0),
                    0.5,
                    Behind,
                ),
                v(
                    &[
                        ("@step", 0),
                        ("@step", 7),
                        ("@step", 14),
                        ("@step", 21),
                        ("@step", 28),
                    ],
                    (0.85, 0.95),
                    0.6,
                    Closer,
                ),
                v(
                    &[("@step", 0), ("@step", 5), ("@step", 10), ("@step", 15)],
                    (1.0, 1.1),
                    0.5,
                    Away,
                ),
            ];
            WAYS
        }
        "late_eating" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.generic.eat", 0),
                        ("minecraft:entity.generic.eat", 4),
                        ("minecraft:entity.generic.eat", 8),
                    ],
                    (0.9, 1.1),
                    0.5,
                    Behind,
                ),
                v(
                    &[("minecraft:entity.generic.eat", 0)],
                    (0.8, 0.9),
                    0.5,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.generic.eat", 0),
                        ("minecraft:entity.generic.eat", 5),
                        ("minecraft:entity.player.burp", 14),
                    ],
                    (0.9, 1.0),
                    0.5,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.generic.drink", 0),
                        ("minecraft:entity.generic.drink", 6),
                    ],
                    (0.9, 1.0),
                    0.5,
                    Behind,
                ),
            ];
            WAYS
        }
        "place_sound" => {
            const WAYS: &[Variant] = &[
                v(&[("@place", 0)], (0.9, 1.0), 0.7, Behind),
                v(&[("@place", 0), ("@place", 12)], (0.9, 1.0), 0.6, Behind),
                v(&[("@break", 0), ("@place", 15)], (0.9, 1.0), 0.6, Behind),
                v(&[("@break", 0)], (0.8, 0.9), 0.6, Behind),
            ];
            WAYS
        }
        // Soft knocks in stages two and three; the heavy one is `knock_heavy`.
        "knock" => {
            const WAYS: &[Variant] = &[
                v(
                    &[(DOOR_CLOSE, 0), (DOOR_CLOSE, 8), (DOOR_CLOSE, 16)],
                    (1.6, 1.7),
                    0.4,
                    Still,
                ),
                v(
                    &[(DOOR_CLOSE, 0), (DOOR_CLOSE, 8), (DOOR_CLOSE, 40)],
                    (1.6, 1.7),
                    0.4,
                    Still,
                ),
                v(
                    &[
                        (DOOR_CLOSE, 0),
                        (DOOR_CLOSE, 7),
                        (DOOR_CLOSE, 14),
                        (DOOR_CLOSE, 21),
                    ],
                    (1.7, 1.8),
                    0.35,
                    Still,
                ),
                v(&[(DOOR_CLOSE, 0)], (1.5, 1.6), 0.45, Still),
            ];
            WAYS
        }
        "knock_heavy" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.zombie.attack_wooden_door", 0),
                        ("minecraft:entity.zombie.attack_wooden_door", 20),
                        ("minecraft:entity.zombie.attack_wooden_door", 40),
                    ],
                    (0.6, 0.7),
                    0.6,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.zombie.attack_wooden_door", 0),
                        ("minecraft:entity.zombie.attack_wooden_door", 30),
                    ],
                    (0.5, 0.6),
                    0.7,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.zombie.attack_wooden_door", 0),
                        ("minecraft:entity.zombie.attack_wooden_door", 6),
                        ("minecraft:entity.zombie.attack_wooden_door", 12),
                        ("minecraft:entity.zombie.attack_wooden_door", 18),
                        ("minecraft:entity.zombie.attack_wooden_door", 24),
                    ],
                    (0.7, 0.8),
                    0.5,
                    Still,
                ),
            ];
            WAYS
        }
        "lever_click" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:block.lever.click", 0)],
                    (0.6, 0.7),
                    0.6,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.lever.click", 0),
                        ("minecraft:block.lever.click", 12),
                    ],
                    (0.55, 0.65),
                    0.6,
                    Still,
                ),
                v(
                    &[("minecraft:block.stone_button.click_on", 0)],
                    (0.6, 0.7),
                    0.6,
                    Still,
                ),
                v(
                    &[("minecraft:block.tripwire.click_on", 0)],
                    (0.8, 0.9),
                    0.5,
                    Still,
                ),
            ];
            WAYS
        }
        "page_turn" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:item.book.page_turn", 0)],
                    (0.9, 1.0),
                    0.7,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:item.book.page_turn", 0),
                        ("minecraft:item.book.page_turn", 30),
                        ("minecraft:item.book.page_turn", 60),
                    ],
                    (0.9, 1.0),
                    0.6,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:item.book.page_turn", 0),
                        ("minecraft:item.book.page_turn", 3),
                        ("minecraft:item.book.page_turn", 6),
                        ("minecraft:item.book.page_turn", 9),
                        ("minecraft:item.book.page_turn", 12),
                    ],
                    (1.0, 1.2),
                    0.6,
                    Still,
                ),
            ];
            WAYS
        }
        "arrow_hit" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.arrow.hit", 0)],
                    (0.9, 1.1),
                    0.8,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.arrow.shoot", 0),
                        ("minecraft:entity.arrow.hit", 10),
                    ],
                    (0.9, 1.0),
                    0.7,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.arrow.hit", 0),
                        ("minecraft:entity.arrow.hit", 12),
                    ],
                    (0.9, 1.0),
                    0.7,
                    Behind,
                ),
            ];
            WAYS
        }
        "wolf_far" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.wolf_sad.whine", 0)],
                    (0.7, 0.8),
                    2.0,
                    Still,
                ),
                v(
                    &[("minecraft:entity.wolf_big.growl", 0)],
                    (0.5, 0.6),
                    2.0,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.wolf.ambient", 0),
                        ("minecraft:entity.wolf.ambient", 30),
                    ],
                    (0.6, 0.7),
                    2.0,
                    Still,
                ),
                v(
                    &[("minecraft:entity.wolf_angry.growl", 0)],
                    (0.5, 0.6),
                    2.0,
                    Still,
                ),
            ];
            WAYS
        }
        "your_voice" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.player.hurt", 0)],
                    (0.9, 1.0),
                    0.7,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.player.hurt", 0),
                        ("minecraft:entity.player.hurt", 14),
                    ],
                    (0.9, 1.0),
                    0.6,
                    Still,
                ),
                v(
                    &[("minecraft:entity.player.hurt_freeze", 0)],
                    (0.9, 1.0),
                    0.6,
                    Still,
                ),
                v(
                    &[("minecraft:entity.player.hurt_drown", 0)],
                    (0.9, 1.0),
                    0.6,
                    Still,
                ),
            ];
            WAYS
        }
        "chest_answers" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:block.chest.close", 3)],
                    (0.9, 1.0),
                    0.6,
                    Still,
                ),
                v(
                    &[("minecraft:block.chest.close", 1)],
                    (0.7, 0.8),
                    0.7,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.chest.open", 0),
                        ("minecraft:block.chest.close", 6),
                    ],
                    (0.9, 1.0),
                    0.5,
                    Still,
                ),
                v(
                    &[("minecraft:block.barrel.close", 3)],
                    (0.9, 1.0),
                    0.6,
                    Still,
                ),
            ];
            WAYS
        }
        _ => return None,
    })
}

/// The variant to play this time: the next one after the last, from where the seed starts, so
/// two in a row are never the same (one variant is always itself).
pub fn pick(variants: &'static [Variant], seed: usize, times: usize) -> &'static Variant {
    &variants[(seed + times) % variants.len()]
}

/// The commands a variant makes for `player`, from `spot` (the event's place) or around the
/// target as it moves, each with its tick.
pub fn commands(
    variant: &Variant,
    spot: Option<Pos>,
    memory: &Memory,
    player: u32,
    seed: usize,
    ids: &mut impl FnMut() -> u64,
) -> Vec<(Command, u64)> {
    let at = memory.last.pos;
    let yaw = (memory.last.yaw as f64).to_radians();
    // Behind is the way they face, turned round: yaw 0 faces +z.
    let back = [yaw.sin(), -yaw.cos()];
    let n = variant.beats.len().max(1) as f64;
    let pitch =
        variant.pitch.0 + (variant.pitch.1 - variant.pitch.0) * ((seed % 100) as f32 / 100.0);

    variant
        .beats
        .iter()
        .enumerate()
        .map(|(i, (sound, tick))| {
            let along = i as f64 / (n - 1.0).max(1.0);
            let distance = match variant.moves {
                Still => 0.0,
                Behind => 3.0,
                Closer => 8.0 - 6.0 * along,
                Away => 2.0 + 6.0 * along,
            };
            let pos = match (variant.moves, spot) {
                (Still, Some(spot)) => spot,
                _ => [
                    at[0] + back[0] * distance,
                    at[1],
                    at[2] + back[1] * distance,
                ],
            };
            let command = match sound.strip_prefix('@') {
                Some(kind) => Command::BlockSound {
                    id: ids(),
                    player,
                    block: if kind == "step" { "under" } else { "held" }.to_string(),
                    of: kind.to_string(),
                    pos,
                    volume: variant.volume,
                    pitch,
                },
                None => Command::PlaySound {
                    id: ids(),
                    player,
                    sound: sound.to_string(),
                    pos,
                    volume: variant.volume,
                    pitch,
                },
            };
            (command, *tick)
        })
        .collect()
}

/// The doors, chests, lecterns and jukeboxes the mind scans for in a house the target is in
/// that is not their home (`Memory::here`).
pub const HOUSE_KINDS: &[&str] = &[
    "minecraft:oak_door",
    "minecraft:spruce_door",
    "minecraft:birch_door",
    "minecraft:jungle_door",
    "minecraft:acacia_door",
    "minecraft:dark_oak_door",
    "minecraft:mangrove_door",
    "minecraft:cherry_door",
    "minecraft:bamboo_door",
    "minecraft:pale_oak_door",
    "minecraft:crimson_door",
    "minecraft:warped_door",
    "minecraft:iron_door",
    "minecraft:copper_door",
    "minecraft:chest",
    "minecraft:barrel",
    "minecraft:trapped_chest",
    "minecraft:lectern",
    "minecraft:jukebox",
];

/// The block of a house the sound comes from: a door for a door's sound or a knock, a chest
/// for a chest's; the house they are in first when it is not their home, else their home.
pub fn house_spot(event: &str, memory: &Memory, seed: usize) -> Option<BlockPos> {
    let want_chest = event == "chest_sound";
    let home_near = memory.home.as_ref().is_some_and(|h| {
        let at = memory.last.pos;
        let dx = at[0] - h.center[0] as f64;
        let dz = at[2] - h.center[2] as f64;
        (dx * dx + dz * dz).sqrt() <= 24.0
    });
    let pick = |list: &[BlockPos]| (!list.is_empty()).then(|| list[seed % list.len()]);

    if !home_near {
        if let Some(here) = &memory.here {
            let found = if want_chest {
                pick(&here.containers)
            } else {
                pick(&here.doors)
            };
            if found.is_some() {
                return found;
            }
        }
    }
    if want_chest {
        let chests: Vec<BlockPos> = memory
            .containers
            .iter()
            .filter(|c| c.theirs() && !c.is_furnace())
            .map(|c| c.pos)
            .collect();
        pick(&chests)
    } else {
        memory.home.as_ref().and_then(|h| pick(&h.doors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_event_has_three_to_five_ways_to_sound() {
        for id in [
            "door_sound",
            "chest_sound",
            "late_steps",
            "late_eating",
            "place_sound",
            "knock",
            "knock_heavy",
            "lever_click",
            "page_turn",
            "arrow_hit",
            "wolf_far",
            "your_voice",
        ] {
            let n = variants(id)
                .unwrap_or_else(|| panic!("{id} has none"))
                .len();
            assert!((3..=5).contains(&n), "{id} has {n}");
        }
    }

    #[test]
    fn two_in_a_row_are_never_the_same_way() {
        let vs = variants("door_sound").unwrap();
        for seed in 0..10 {
            for times in 0..10 {
                assert!(!std::ptr::eq(
                    pick(vs, seed, times),
                    pick(vs, seed, times + 1)
                ));
            }
        }
    }

    #[test]
    fn footsteps_are_the_block_underfoot_and_come_from_behind_or_closer() {
        let mut memory = Memory::default();
        memory.last.pos = [0.0, 64.0, 0.0];
        memory.last.yaw = 0.0; // facing +z: behind is -z
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        let behind = commands(
            &variants("late_steps").unwrap()[0],
            None,
            &memory,
            0,
            0,
            &mut ids,
        );
        for (command, _) in &behind {
            match command {
                Command::BlockSound { block, of, pos, .. } => {
                    assert_eq!((block.as_str(), of.as_str()), ("under", "step"));
                    assert!(pos[2] < -2.0, "behind them: {pos:?}");
                }
                other => panic!("{other:?}"),
            }
        }
        let closer = commands(
            &variants("late_steps").unwrap()[2],
            None,
            &memory,
            0,
            0,
            &mut ids,
        );
        let first = match closer[0].0 {
            Command::BlockSound { pos, .. } => pos[2],
            _ => 0.0,
        };
        let last = match closer.last().unwrap().0 {
            Command::BlockSound { pos, .. } => pos[2],
            _ => 0.0,
        };
        assert!(first < last, "coming closer: {first} then {last}");
        assert!(
            closer.windows(2).all(|w| w[0].1 < w[1].1),
            "one beat after another"
        );
    }

    #[test]
    fn a_house_sound_comes_from_the_house_they_are_in_when_it_is_not_their_home() {
        let mut memory = Memory::default();
        memory.home = Some(crate::memory::Home {
            center: [0, 64, 0],
            doors: vec![[3, 64, 0]],
            ..Default::default()
        });
        memory.last.pos = [200.0, 64.0, 0.0];
        assert_eq!(
            house_spot("knock", &memory, 0),
            Some([3, 64, 0]),
            "no other house known"
        );
        memory.here = Some(crate::memory::Here {
            doors: vec![[204, 64, 1]],
            containers: vec![[201, 64, -2]],
            ..Default::default()
        });
        assert_eq!(house_spot("knock", &memory, 0), Some([204, 64, 1]));
        assert_eq!(house_spot("chest_sound", &memory, 0), Some([201, 64, -2]));
        memory.last.pos = [1.0, 64.0, 1.0];
        assert_eq!(
            house_spot("knock", &memory, 0),
            Some([3, 64, 0]),
            "home first, near it"
        );
    }
}
