//! The sound events' variants: each plays one of a few fitting sounds, at its own pitch, rhythm
//! and movement, never the same variant twice running (owner: "3 to 5 each", "so it
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
    /// On the floor above, three blocks up, crossing from one side to the other.
    Above,
    /// Inside the wall beside them, a block and a half to their right.
    Beside,
    /// Two blocks behind, rising five blocks over the beats, as up a ladder.
    Rising,
    /// At their ear: half a block behind, at head height.
    Ear,
    /// Under the floor they stand on, two and a half blocks down.
    Below,
    /// Far under them, sixteen blocks down.
    Deep,
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
        // docs/events_todo.md, 116.
        "breathing_in_the_wall" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 45),
                        ("minecraft:entity.player.breath", 90),
                    ],
                    (0.5, 0.6),
                    0.5,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 60),
                    ],
                    (0.42, 0.5),
                    0.6,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 40),
                        ("minecraft:entity.player.breath", 80),
                        ("minecraft:entity.player.breath", 120),
                    ],
                    (0.55, 0.65),
                    0.45,
                    Beside,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 117.
        "scream_indoors" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.goat.screaming.ambient", 0)],
                    (0.7, 0.8),
                    0.9,
                    Above,
                ),
                v(
                    &[("minecraft:entity.goat.screaming.hurt", 0)],
                    (0.6, 0.7),
                    0.8,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:entity.goat.screaming.ambient", 0),
                        ("minecraft:entity.goat.screaming.ambient", 30),
                    ],
                    (0.75, 0.85),
                    0.7,
                    Above,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 118.
        "item_break_behind" => {
            const WAYS: &[Variant] = &[
                v(&[("minecraft:entity.item.break", 0)], (1.0, 1.1), 0.8, Ear),
                v(
                    &[("minecraft:entity.item.break", 0)],
                    (0.8, 0.9),
                    0.9,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.item.break", 0),
                        ("minecraft:entity.item.break", 6),
                    ],
                    (0.9, 1.0),
                    0.7,
                    Behind,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 119.
        "armour_behind" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:item.armor.equip_iron", 0),
                        ("minecraft:item.armor.equip_iron", 9),
                        ("minecraft:item.armor.equip_iron", 18),
                        ("minecraft:item.armor.equip_iron", 27),
                        ("minecraft:item.armor.equip_iron", 36),
                        ("minecraft:item.armor.equip_iron", 45),
                    ],
                    (0.7, 0.8),
                    0.5,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:item.armor.equip_chain", 0),
                        ("minecraft:item.armor.equip_chain", 9),
                        ("minecraft:item.armor.equip_chain", 18),
                        ("minecraft:item.armor.equip_chain", 27),
                        ("minecraft:item.armor.equip_chain", 36),
                        ("minecraft:item.armor.equip_chain", 45),
                    ],
                    (0.7, 0.8),
                    0.5,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:item.armor.equip_netherite", 0),
                        ("minecraft:item.armor.equip_netherite", 10),
                        ("minecraft:item.armor.equip_netherite", 20),
                        ("minecraft:item.armor.equip_netherite", 30),
                        ("minecraft:item.armor.equip_netherite", 40),
                    ],
                    (0.6, 0.7),
                    0.5,
                    Behind,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 120.
        "breath_in_the_chest" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.player.breath", 0)],
                    (0.5, 0.6),
                    0.7,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 35),
                    ],
                    (0.45, 0.55),
                    0.6,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:block.chest.close", 20),
                    ],
                    (0.5, 0.6),
                    0.6,
                    Still,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 121.
        "rain_on_the_roof" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:weather.rain.above", 0),
                        ("minecraft:weather.rain.above", 20),
                        ("minecraft:weather.rain.above", 40),
                        ("minecraft:weather.rain.above", 60),
                        ("minecraft:weather.rain.above", 80),
                        ("minecraft:weather.rain.above", 100),
                        ("minecraft:weather.rain.above", 120),
                        ("minecraft:weather.rain.above", 140),
                        ("minecraft:weather.rain.above", 160),
                        ("minecraft:weather.rain.above", 180),
                        ("minecraft:weather.rain.above", 200),
                        ("minecraft:weather.rain.above", 220),
                        ("minecraft:weather.rain.above", 240),
                        ("minecraft:weather.rain.above", 260),
                        ("minecraft:weather.rain.above", 280),
                    ],
                    (0.9, 1.0),
                    0.6,
                    Above,
                ),
                v(
                    &[
                        ("minecraft:weather.rain.above", 0),
                        ("minecraft:weather.rain.above", 25),
                        ("minecraft:weather.rain.above", 50),
                        ("minecraft:weather.rain.above", 75),
                        ("minecraft:weather.rain.above", 100),
                        ("minecraft:weather.rain.above", 125),
                        ("minecraft:weather.rain.above", 150),
                        ("minecraft:weather.rain.above", 175),
                        ("minecraft:weather.rain.above", 200),
                        ("minecraft:weather.rain.above", 225),
                    ],
                    (0.8, 0.9),
                    0.6,
                    Above,
                ),
                v(
                    &[
                        ("minecraft:weather.rain.above", 0),
                        ("minecraft:weather.rain.above", 15),
                        ("minecraft:weather.rain.above", 30),
                        ("minecraft:weather.rain.above", 45),
                        ("minecraft:weather.rain.above", 60),
                        ("minecraft:weather.rain.above", 75),
                        ("minecraft:weather.rain.above", 90),
                        ("minecraft:weather.rain.above", 105),
                        ("minecraft:weather.rain.above", 120),
                        ("minecraft:weather.rain.above", 135),
                        ("minecraft:weather.rain.above", 150),
                        ("minecraft:weather.rain.above", 165),
                        ("minecraft:weather.rain.above", 180),
                        ("minecraft:weather.rain.above", 195),
                        ("minecraft:weather.rain.above", 210),
                        ("minecraft:weather.rain.above", 225),
                        ("minecraft:weather.rain.above", 240),
                        ("minecraft:weather.rain.above", 255),
                        ("minecraft:weather.rain.above", 270),
                        ("minecraft:weather.rain.above", 285),
                    ],
                    (0.95, 1.05),
                    0.5,
                    Above,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 122.
        "sniffing" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.sniffer.sniffing", 0),
                        ("minecraft:entity.sniffer.sniffing", 25),
                        ("minecraft:entity.sniffer.sniffing", 50),
                    ],
                    (0.8, 0.9),
                    0.7,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.sniffer.sniffing", 0),
                        ("minecraft:entity.sniffer.sniffing", 20),
                        ("minecraft:entity.sniffer.sniffing", 40),
                        ("minecraft:entity.sniffer.sniffing", 60),
                    ],
                    (0.7, 0.8),
                    0.6,
                    Closer,
                ),
                v(
                    &[
                        ("minecraft:entity.sniffer.searching", 0),
                        ("minecraft:entity.sniffer.searching", 25),
                        ("minecraft:entity.sniffer.searching", 50),
                    ],
                    (0.8, 0.9),
                    0.6,
                    Away,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 123.
        "gasp_when_turning" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.player.breath", 0)],
                    (1.5, 1.7),
                    0.8,
                    Behind,
                ),
                v(
                    &[("minecraft:entity.player.breath", 0)],
                    (1.6, 1.8),
                    0.6,
                    Ear,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 3),
                    ],
                    (1.5, 1.6),
                    0.7,
                    Behind,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 124.
        "water_where_none" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.player.swim", 0),
                        ("minecraft:entity.player.swim", 12),
                        ("minecraft:entity.player.swim", 24),
                    ],
                    (0.8, 0.9),
                    0.6,
                    Beside,
                ),
                v(
                    &[("minecraft:entity.generic.splash", 0)],
                    (0.8, 0.9),
                    0.7,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:block.water.ambient", 0),
                        ("minecraft:block.water.ambient", 30),
                        ("minecraft:block.water.ambient", 60),
                    ],
                    (0.9, 1.0),
                    0.6,
                    Below,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 125.
        "doors_slam_far" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.wooden_door.close", 0),
                        ("minecraft:block.wooden_door.close", 1),
                        ("minecraft:block.wooden_door.close", 2),
                        ("minecraft:block.wooden_door.close", 3),
                        ("minecraft:block.wooden_door.close", 4),
                    ],
                    (0.8, 0.9),
                    2.0,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.wooden_door.close", 0),
                        ("minecraft:block.iron_door.close", 1),
                        ("minecraft:block.wooden_door.close", 2),
                        ("minecraft:block.wooden_door.close", 2),
                    ],
                    (0.8, 0.9),
                    2.0,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.wooden_door.close", 0),
                        ("minecraft:block.wooden_door.close", 1),
                        ("minecraft:block.wooden_door.close", 2),
                        ("minecraft:block.wooden_door.close", 3),
                        ("minecraft:block.wooden_door.close", 4),
                        ("minecraft:block.wooden_door.close", 5),
                    ],
                    (0.7, 0.8),
                    2.0,
                    Still,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 129.
        "chains_dragging" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.chain.step", 0),
                        ("minecraft:block.chain.step", 6),
                        ("minecraft:block.chain.step", 12),
                        ("minecraft:block.chain.step", 18),
                        ("minecraft:block.chain.step", 24),
                        ("minecraft:block.chain.step", 30),
                        ("minecraft:block.chain.step", 36),
                        ("minecraft:block.chain.step", 42),
                    ],
                    (0.6, 0.7),
                    0.7,
                    Closer,
                ),
                v(
                    &[
                        ("minecraft:block.chain.step", 0),
                        ("minecraft:block.chain.step", 8),
                        ("minecraft:block.chain.step", 16),
                        ("minecraft:block.chain.step", 24),
                        ("minecraft:block.chain.step", 32),
                        ("minecraft:block.chain.step", 40),
                    ],
                    (0.6, 0.7),
                    0.6,
                    Away,
                ),
                v(
                    &[
                        ("minecraft:block.chain.place", 0),
                        ("minecraft:block.chain.place", 10),
                        ("minecraft:block.chain.place", 20),
                        ("minecraft:block.chain.place", 30),
                    ],
                    (0.5, 0.6),
                    0.6,
                    Behind,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 130.
        "shriek_below" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:block.sculk_shrieker.shriek", 0)],
                    (0.8, 0.9),
                    2.5,
                    Deep,
                ),
                v(
                    &[
                        ("minecraft:block.sculk_shrieker.shriek", 0),
                        ("minecraft:block.sculk_shrieker.shriek", 40),
                    ],
                    (0.7, 0.8),
                    2.0,
                    Deep,
                ),
                v(
                    &[
                        ("minecraft:entity.warden.heartbeat", 0),
                        ("minecraft:entity.warden.heartbeat", 20),
                        ("minecraft:entity.warden.heartbeat", 40),
                        ("minecraft:entity.warden.heartbeat", 60),
                        ("minecraft:block.sculk_shrieker.shriek", 80),
                    ],
                    (0.8, 0.9),
                    2.0,
                    Deep,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 131.
        "witch_laughs" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.witch.celebrate", 0)],
                    (0.9, 1.0),
                    2.5,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.witch.ambient", 0),
                        ("minecraft:entity.witch.ambient", 30),
                    ],
                    (0.8, 0.9),
                    2.0,
                    Still,
                ),
                v(
                    &[("minecraft:entity.witch.celebrate", 0)],
                    (0.7, 0.8),
                    3.0,
                    Still,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 134.
        "glass_rubbed" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.glass.step", 0),
                        ("minecraft:block.glass.step", 4),
                        ("minecraft:block.glass.step", 8),
                        ("minecraft:block.glass.step", 12),
                        ("minecraft:block.glass.step", 16),
                        ("minecraft:block.glass.step", 20),
                    ],
                    (0.4, 0.5),
                    0.5,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.glass.hit", 0),
                        ("minecraft:block.glass.hit", 6),
                        ("minecraft:block.glass.hit", 12),
                        ("minecraft:block.glass.hit", 18),
                    ],
                    (0.5, 0.6),
                    0.5,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.glass.step", 0),
                        ("minecraft:block.glass.step", 3),
                        ("minecraft:block.glass.step", 6),
                        ("minecraft:block.glass.step", 9),
                        ("minecraft:block.glass.step", 12),
                        ("minecraft:block.glass.step", 15),
                        ("minecraft:block.glass.step", 18),
                        ("minecraft:block.glass.step", 21),
                        ("minecraft:block.glass.step", 24),
                        ("minecraft:block.glass.step", 27),
                    ],
                    (0.35, 0.45),
                    0.4,
                    Still,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 135.
        "bones_underfoot" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.bone_block.step", 0),
                        ("minecraft:block.bone_block.step", 7),
                        ("minecraft:block.bone_block.step", 14),
                        ("minecraft:block.bone_block.step", 21),
                        ("minecraft:block.bone_block.step", 28),
                        ("minecraft:block.bone_block.step", 35),
                        ("minecraft:block.bone_block.step", 42),
                        ("minecraft:block.bone_block.step", 49),
                        ("minecraft:block.bone_block.step", 56),
                        ("minecraft:block.bone_block.step", 63),
                    ],
                    (0.9, 1.0),
                    0.6,
                    Ear,
                ),
                v(
                    &[
                        ("minecraft:block.bone_block.step", 0),
                        ("minecraft:block.bone_block.step", 8),
                        ("minecraft:block.bone_block.step", 16),
                        ("minecraft:block.bone_block.step", 24),
                        ("minecraft:block.bone_block.step", 32),
                        ("minecraft:block.bone_block.step", 40),
                        ("minecraft:block.bone_block.step", 48),
                        ("minecraft:block.bone_block.step", 56),
                    ],
                    (0.8, 0.9),
                    0.6,
                    Ear,
                ),
                v(
                    &[
                        ("minecraft:block.bone_block.step", 0),
                        ("minecraft:block.bone_block.step", 6),
                        ("minecraft:block.bone_block.step", 12),
                        ("minecraft:block.bone_block.step", 18),
                        ("minecraft:block.bone_block.step", 24),
                        ("minecraft:block.bone_block.step", 30),
                        ("minecraft:block.bone_block.step", 36),
                        ("minecraft:block.bone_block.step", 42),
                        ("minecraft:block.bone_block.step", 48),
                        ("minecraft:block.bone_block.step", 54),
                        ("minecraft:block.bone_block.step", 60),
                        ("minecraft:block.bone_block.step", 66),
                    ],
                    (0.95, 1.05),
                    0.5,
                    Ear,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 136.
        "bubbling_under" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.bubble_column.upwards_ambient", 0),
                        ("minecraft:block.bubble_column.upwards_ambient", 20),
                        ("minecraft:block.bubble_column.upwards_ambient", 40),
                        ("minecraft:block.bubble_column.upwards_ambient", 60),
                    ],
                    (0.8, 0.9),
                    0.7,
                    Below,
                ),
                v(
                    &[
                        ("minecraft:block.bubble_column.whirlpool_ambient", 0),
                        ("minecraft:block.bubble_column.whirlpool_ambient", 25),
                        ("minecraft:block.bubble_column.whirlpool_ambient", 50),
                    ],
                    (0.8, 0.9),
                    0.7,
                    Below,
                ),
                v(
                    &[
                        ("minecraft:block.water.ambient", 0),
                        ("minecraft:block.water.ambient", 30),
                        ("minecraft:block.water.ambient", 60),
                    ],
                    (0.7, 0.8),
                    0.8,
                    Below,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 137.
        "the_bell_in_the_chest" => {
            const WAYS: &[Variant] = &[
                v(&[("minecraft:block.bell.use", 0)], (1.4, 1.5), 0.4, Still),
                v(
                    &[("minecraft:block.bell.resonate", 0)],
                    (1.2, 1.3),
                    0.3,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.bell.use", 0),
                        ("minecraft:block.bell.use", 20),
                    ],
                    (1.6, 1.7),
                    0.3,
                    Still,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 138.
        "panting_behind" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 10),
                        ("minecraft:entity.player.breath", 20),
                        ("minecraft:entity.player.breath", 30),
                        ("minecraft:entity.player.breath", 40),
                        ("minecraft:entity.player.breath", 50),
                    ],
                    (1.2, 1.3),
                    0.8,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 8),
                        ("minecraft:entity.player.breath", 16),
                        ("minecraft:entity.player.breath", 24),
                        ("minecraft:entity.player.breath", 32),
                        ("minecraft:entity.player.breath", 40),
                        ("minecraft:entity.player.breath", 48),
                        ("minecraft:entity.player.breath", 56),
                    ],
                    (1.3, 1.4),
                    0.7,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 12),
                        ("minecraft:entity.player.breath", 24),
                        ("minecraft:entity.player.breath", 36),
                        ("minecraft:entity.player.breath", 48),
                    ],
                    (1.1, 1.2),
                    0.7,
                    Ear,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 139.
        "iron_door_below" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:block.iron_door.close", 0)],
                    (0.7, 0.8),
                    2.5,
                    Deep,
                ),
                v(
                    &[
                        ("minecraft:block.iron_door.close", 0),
                        ("minecraft:block.iron_door.close", 15),
                    ],
                    (0.7, 0.8),
                    2.0,
                    Deep,
                ),
                v(
                    &[
                        ("minecraft:block.iron_door.open", 0),
                        ("minecraft:block.iron_door.close", 25),
                    ],
                    (0.6, 0.7),
                    2.0,
                    Deep,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 140.
        "ice_cracks" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:block.glass.break", 0)],
                    (0.5, 0.6),
                    0.6,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.glass.break", 0),
                        ("minecraft:block.glass.break", 8),
                    ],
                    (0.45, 0.55),
                    0.5,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.glass.hit", 0),
                        ("minecraft:block.glass.hit", 5),
                        ("minecraft:block.glass.hit", 10),
                    ],
                    (0.5, 0.6),
                    0.6,
                    Still,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 141.
        "bush_rustles" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:block.sweet_berry_bush.place", 0)],
                    (0.8, 0.9),
                    0.8,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:block.azalea_leaves.step", 0),
                        ("minecraft:block.azalea_leaves.step", 3),
                        ("minecraft:block.azalea_leaves.step", 6),
                        ("minecraft:block.azalea_leaves.step", 9),
                    ],
                    (0.8, 0.9),
                    0.8,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:block.grass.step", 0),
                        ("minecraft:block.grass.step", 2),
                        ("minecraft:block.grass.step", 4),
                        ("minecraft:block.grass.step", 6),
                        ("minecraft:block.grass.step", 8),
                    ],
                    (0.7, 0.8),
                    0.8,
                    Beside,
                ),
            ];
            WAYS
        }
        // docs/events_todo.md, 142.
        "snoring_beside" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.fox.sleep", 0),
                        ("minecraft:entity.fox.sleep", 40),
                        ("minecraft:entity.fox.sleep", 80),
                        ("minecraft:entity.fox.sleep", 120),
                        ("minecraft:entity.fox.sleep", 160),
                    ],
                    (0.5, 0.6),
                    0.6,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:entity.fox.sleep", 0),
                        ("minecraft:entity.fox.sleep", 50),
                        ("minecraft:entity.fox.sleep", 100),
                        ("minecraft:entity.fox.sleep", 150),
                    ],
                    (0.45, 0.55),
                    0.6,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:entity.fox.sleep", 0),
                        ("minecraft:entity.fox.sleep", 35),
                        ("minecraft:entity.fox.sleep", 70),
                        ("minecraft:entity.fox.sleep", 105),
                        ("minecraft:entity.fox.sleep", 140),
                        ("minecraft:entity.fox.sleep", 175),
                    ],
                    (0.55, 0.6),
                    0.5,
                    Beside,
                ),
            ];
            WAYS
        }
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
        // Their own knock, answered from the door a moment later: two back, or three, or one.
        "answer_knock" => {
            const WAYS: &[Variant] = &[
                v(
                    &[(DOOR_CLOSE, 20), (DOOR_CLOSE, 28)],
                    (1.6, 1.7),
                    0.45,
                    Still,
                ),
                v(
                    &[(DOOR_CLOSE, 30), (DOOR_CLOSE, 38)],
                    (1.5, 1.6),
                    0.4,
                    Still,
                ),
                v(
                    &[(DOOR_CLOSE, 20), (DOOR_CLOSE, 28), (DOOR_CLOSE, 36)],
                    (1.6, 1.7),
                    0.4,
                    Still,
                ),
                v(&[(DOOR_CLOSE, 40)], (1.4, 1.5), 0.5, Still),
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
        "steps_above" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.wood.step", 0),
                        ("minecraft:block.wood.step", 9),
                        ("minecraft:block.wood.step", 18),
                        ("minecraft:block.wood.step", 27),
                    ],
                    (0.8, 0.9),
                    0.5,
                    Above,
                ),
                v(
                    &[
                        ("minecraft:block.wood.step", 0),
                        ("minecraft:block.wood.step", 12),
                        ("minecraft:block.wood.step", 24),
                    ],
                    (0.7, 0.8),
                    0.6,
                    Above,
                ),
                v(
                    &[
                        ("minecraft:block.wood.step", 0),
                        ("minecraft:block.wood.step", 6),
                        ("minecraft:block.wood.step", 12),
                        ("minecraft:block.wood.step", 18),
                        ("minecraft:block.wood.step", 24),
                    ],
                    (0.9, 1.0),
                    0.4,
                    Above,
                ),
                v(
                    &[
                        ("minecraft:block.wool.step", 0),
                        ("minecraft:block.wool.step", 10),
                        ("minecraft:block.wool.step", 20),
                    ],
                    (0.8, 0.9),
                    0.6,
                    Above,
                ),
            ];
            WAYS
        }
        // Right under the bed, as they lie down (batch 5): slow, then nothing.
        "under_the_bed" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.silverfish.step", 0),
                        ("minecraft:entity.silverfish.step", 12),
                        ("minecraft:entity.silverfish.step", 24),
                        ("minecraft:entity.silverfish.step", 36),
                    ],
                    (0.5, 0.6),
                    0.5,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.wood.hit", 0),
                        ("minecraft:block.wood.hit", 14),
                        ("minecraft:block.wood.hit", 30),
                    ],
                    (1.5, 1.7),
                    0.25,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.wool.step", 0),
                        ("minecraft:entity.silverfish.step", 10),
                        ("minecraft:block.wool.step", 22),
                    ],
                    (0.6, 0.7),
                    0.4,
                    Still,
                ),
            ];
            WAYS
        }
        "scratching" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.silverfish.step", 0),
                        ("minecraft:entity.silverfish.step", 3),
                        ("minecraft:entity.silverfish.step", 6),
                        ("minecraft:entity.silverfish.step", 9),
                    ],
                    (0.6, 0.7),
                    0.5,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:block.wood.hit", 0),
                        ("minecraft:block.wood.hit", 4),
                        ("minecraft:block.wood.hit", 8),
                    ],
                    (1.4, 1.6),
                    0.3,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:block.sand.step", 0),
                        ("minecraft:block.sand.step", 5),
                        ("minecraft:block.sand.step", 10),
                        ("minecraft:block.sand.step", 15),
                    ],
                    (0.6, 0.7),
                    0.4,
                    Beside,
                ),
                v(
                    &[
                        ("minecraft:entity.silverfish.step", 0),
                        ("minecraft:entity.silverfish.step", 20),
                    ],
                    (0.5, 0.6),
                    0.5,
                    Beside,
                ),
            ];
            WAYS
        }
        "ladder_climb" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.ladder.step", 0),
                        ("minecraft:block.ladder.step", 8),
                        ("minecraft:block.ladder.step", 16),
                        ("minecraft:block.ladder.step", 24),
                        ("minecraft:block.ladder.step", 32),
                    ],
                    (0.9, 1.0),
                    0.6,
                    Rising,
                ),
                v(
                    &[
                        ("minecraft:block.ladder.step", 0),
                        ("minecraft:block.ladder.step", 5),
                        ("minecraft:block.ladder.step", 10),
                        ("minecraft:block.ladder.step", 15),
                    ],
                    (1.0, 1.1),
                    0.6,
                    Rising,
                ),
                v(
                    &[
                        ("minecraft:block.ladder.step", 0),
                        ("minecraft:block.ladder.step", 12),
                        ("minecraft:block.ladder.step", 24),
                    ],
                    (0.8, 0.9),
                    0.5,
                    Rising,
                ),
            ];
            WAYS
        }
        "pickup_pop" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:entity.item.pickup", 0)],
                    (0.9, 1.1),
                    0.5,
                    Behind,
                ),
                v(
                    &[
                        ("minecraft:entity.item.pickup", 0),
                        ("minecraft:entity.item.pickup", 6),
                    ],
                    (1.0, 1.2),
                    0.5,
                    Behind,
                ),
                v(
                    &[("minecraft:entity.experience_orb.pickup", 0)],
                    (0.6, 0.8),
                    0.4,
                    Behind,
                ),
            ];
            WAYS
        }
        "cold_crackle" => {
            const WAYS: &[Variant] = &[
                v(
                    &[("minecraft:block.furnace.fire_crackle", 0)],
                    (0.8, 0.9),
                    0.8,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.furnace.fire_crackle", 0),
                        ("minecraft:block.furnace.fire_crackle", 18),
                        ("minecraft:block.furnace.fire_crackle", 40),
                    ],
                    (0.7, 0.9),
                    0.7,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.furnace.fire_crackle", 0),
                        ("minecraft:block.candle.extinguish", 30),
                    ],
                    (0.8, 0.9),
                    0.7,
                    Still,
                ),
            ];
            WAYS
        }
        "breath_close" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 40),
                    ],
                    (0.6, 0.7),
                    0.3,
                    Ear,
                ),
                v(
                    &[("minecraft:entity.player.breath", 0)],
                    (0.5, 0.6),
                    0.35,
                    Ear,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 30),
                        ("minecraft:entity.player.breath", 60),
                    ],
                    (0.6, 0.7),
                    0.25,
                    Ear,
                ),
                v(
                    &[
                        ("minecraft:entity.warden.heartbeat", 0),
                        ("minecraft:entity.warden.heartbeat", 20),
                        ("minecraft:entity.warden.heartbeat", 40),
                    ],
                    (0.9, 1.0),
                    0.3,
                    Ear,
                ),
            ];
            WAYS
        }
        // Crouched still, breathing at their ear, slow, a little nearer each time; the brain
        // drops what is left as soon as they move.
        "breath" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 50),
                        ("minecraft:entity.player.breath", 100),
                        ("minecraft:entity.player.breath", 150),
                        ("minecraft:entity.player.breath", 200),
                    ],
                    (0.55, 0.65),
                    0.3,
                    Ear,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 70),
                        ("minecraft:entity.player.breath", 140),
                        ("minecraft:entity.warden.heartbeat", 180),
                        ("minecraft:entity.player.breath", 210),
                    ],
                    (0.5, 0.6),
                    0.3,
                    Ear,
                ),
                v(
                    &[
                        ("minecraft:entity.player.breath", 0),
                        ("minecraft:entity.player.breath", 40),
                        ("minecraft:entity.player.breath", 80),
                        ("minecraft:entity.player.breath", 120),
                        ("minecraft:entity.player.breath", 160),
                        ("minecraft:entity.player.breath", 200),
                        ("minecraft:entity.player.breath", 240),
                    ],
                    (0.6, 0.7),
                    0.25,
                    Ear,
                ),
            ];
            WAYS
        }
        "cave_air" => {
            const WAYS: &[Variant] = &[
                v(&[("minecraft:ambient.cave", 0)], (0.8, 1.0), 0.8, Behind),
                v(&[("minecraft:ambient.cave", 0)], (0.5, 0.6), 1.0, Away),
                v(&[("minecraft:ambient.cave", 0)], (1.1, 1.3), 0.7, Closer),
            ];
            WAYS
        }
        "door_scratch" => {
            const WAYS: &[Variant] = &[
                v(
                    &[
                        ("minecraft:block.wood.hit", 0),
                        ("minecraft:block.wood.hit", 3),
                        ("minecraft:block.wood.hit", 6),
                        ("minecraft:block.wood.hit", 9),
                        ("minecraft:block.wood.hit", 12),
                    ],
                    (1.5, 1.7),
                    0.35,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:block.wood.hit", 0),
                        ("minecraft:block.wood.hit", 5),
                        ("minecraft:block.wood.hit", 30),
                        ("minecraft:block.wood.hit", 35),
                    ],
                    (1.3, 1.5),
                    0.35,
                    Still,
                ),
                v(
                    &[
                        ("minecraft:entity.silverfish.step", 0),
                        ("minecraft:entity.silverfish.step", 4),
                        ("minecraft:entity.silverfish.step", 8),
                    ],
                    (0.5, 0.6),
                    0.5,
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
                Still | Above | Beside | Below | Deep => 0.0,
                Behind | Rising => 3.0,
                Closer => 8.0 - 6.0 * along,
                Away => 2.0 + 6.0 * along,
                Ear => 0.5,
            };
            // Their right: facing +z (yaw 0) it is -x, west (until now it was their left).
            let right = [back[1], -back[0]];
            let pos = match (variant.moves, spot) {
                (Still, Some(spot)) => spot,
                (Above, _) => {
                    let across = -3.0 + 6.0 * along;
                    [
                        at[0] + right[0] * across,
                        at[1] + 3.0,
                        at[2] + right[1] * across,
                    ]
                }
                (Beside, _) => [at[0] + right[0] * 1.5, at[1] + 1.0, at[2] + right[1] * 1.5],
                (Rising, _) => [
                    at[0] + back[0] * 2.0,
                    at[1] + 5.0 * along,
                    at[2] + back[1] * 2.0,
                ],
                (Ear, _) => [at[0] + back[0] * 0.5, at[1] + 1.6, at[2] + back[1] * 0.5],
                (Below, _) => [at[0], at[1] - 2.5, at[2]],
                (Deep, _) => [at[0], at[1] - 16.0, at[2]],
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

/// The events whose sound comes from a door, a chest or a window of a house.
pub const HOUSE_SOUNDS: &[&str] = &[
    "the_bell_in_the_chest",
    "glass_rubbed",
    "breath_in_the_chest",
    "door_sound",
    "chest_sound",
    "knock",
    "knock_heavy",
    "lever_click",
    "your_voice",
    "door_scratch",
    "open_door",
    "close_door",
    "window_tap",
    "door_hinge",
];

/// The block of a house the sound comes from: a door for a door's sound or a knock, a chest
/// for a chest's; the house they are in first when it is not their home, else their home when
/// they are near it. None away from home and from any house: a door sound 55 blocks down in a
/// cave, with no door anywhere near, played at the target's own feet (the dev
/// client).
pub fn house_spot(event: &str, memory: &Memory, seed: usize) -> Option<BlockPos> {
    let want_chest = matches!(
        event,
        "chest_sound" | "breath_in_the_chest" | "the_bell_in_the_chest"
    );
    let want_pane = matches!(event, "window_tap" | "glass_rubbed");
    let home_near = memory.home.as_ref().is_some_and(|h| {
        let at = memory.last.pos;
        let dx = at[0] - h.center[0] as f64;
        let dz = at[2] - h.center[2] as f64;
        (dx * dx + dz * dz).sqrt() <= 24.0
    });
    let pick = |list: &[BlockPos]| (!list.is_empty()).then(|| list[seed % list.len()]);
    // A door of theirs opened or turned is the home's, found when they come back; never
    // another house's (it was the last house they ever stood in, however far).
    let home_change = matches!(event, "open_door" | "close_door" | "door_hinge");

    if !home_near && !home_change {
        // A sound is heard where they are: the house they stand in now, if it is now (`here`
        // was kept for good, and a knock played at a house long left, unheard).
        let fresh = memory
            .here
            .as_ref()
            // The house is read every thirty seconds; a little more than that is now.
            .filter(|h| memory.today - h.day <= 0.03);
        if let Some(here) = fresh {
            let found = if want_chest {
                pick(&here.containers)
            } else if want_pane {
                pick(&here.panes)
            } else {
                pick(&here.doors)
            };
            if found.is_some() {
                return found;
            }
        }
        return None;
    }
    if want_pane {
        let panes: Vec<BlockPos> = memory.fixtures("glass_pane").map(|f| f.pos).collect();
        return pick(&panes);
    }
    if want_chest {
        let chests: Vec<BlockPos> = memory
            .containers
            .iter()
            .filter(|c| c.theirs() && !c.is_furnace())
            .map(|c| c.pos)
            .collect();
        pick(&chests)
    } else if matches!(event, "knock" | "knock_heavy" | "door_scratch") {
        // Something wanting in comes to the door they use (N1).
        memory.usual_door()
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
            "steps_above",
            "scratching",
            "under_the_bed",
            "ladder_climb",
            "pickup_pop",
            "cold_crackle",
            "breath_close",
            "breath",
            "answer_knock",
            "cave_air",
            "door_scratch",
            "snoring_beside",
            "bush_rustles",
            "ice_cracks",
            "iron_door_below",
            "panting_behind",
            "the_bell_in_the_chest",
            "bubbling_under",
            "bones_underfoot",
            "glass_rubbed",
            "witch_laughs",
            "shriek_below",
            "chains_dragging",
            "doors_slam_far",
            "water_where_none",
            "gasp_when_turning",
            "sniffing",
            "rain_on_the_roof",
            "breath_in_the_chest",
            "armour_behind",
            "item_break_behind",
            "scream_indoors",
            "breathing_in_the_wall",
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
    fn steps_cross_the_floor_above_scratches_stay_in_the_wall_and_a_climb_rises() {
        let mut memory = Memory::default();
        memory.last.pos = [0.0, 64.0, 0.0];
        memory.last.yaw = 0.0;
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        let heights = |id: &str, way: usize, ids: &mut dyn FnMut() -> u64| -> Vec<[f64; 3]> {
            commands(
                &variants(id).unwrap()[way],
                None,
                &memory,
                0,
                0,
                &mut || ids(),
            )
            .into_iter()
            .map(|(c, _)| match c {
                Command::PlaySound { pos, .. } => pos,
                other => panic!("{other:?}"),
            })
            .collect()
        };
        let above = heights("steps_above", 0, &mut ids);
        assert!(
            above.iter().all(|p| (p[1] - 67.0).abs() < 0.01),
            "on the floor above"
        );
        assert!(
            (above[0][0] - above.last().unwrap()[0]).abs() > 4.0,
            "crossing it"
        );
        let wall = heights("scratching", 0, &mut ids);
        assert!(
            wall.windows(2).all(|w| w[0] == w[1]),
            "one place in the wall"
        );
        // A block and a half to their right: facing +z, that is west, -x.
        assert!(
            wall[0][0] < -1.4 && wall[0][2].abs() < 0.01,
            "to their right: {wall:?}"
        );
        let climb = heights("ladder_climb", 0, &mut ids);
        assert!(
            climb.windows(2).all(|w| w[1][1] > w[0][1]),
            "rising: {climb:?}"
        );
        let ear = heights("breath_close", 0, &mut ids);
        let d = ((ear[0][0]).powi(2) + (ear[0][2]).powi(2)).sqrt();
        assert!(
            d < 1.0 && (ear[0][1] - 65.6).abs() < 0.01,
            "at the ear: {ear:?}"
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
            None,
            "200 blocks from home and in no house: no door to knock on (was the home's door, out of hearing)"
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

    #[test]
    fn a_cave_is_no_house_and_a_house_sound_there_is_neither_offered_nor_made() {
        use craftkit::world::{Now, Observations, Player};
        let mut memory = Memory::default();
        memory.home = Some(crate::memory::Home {
            center: [-2, 80, -14],
            ..Default::default()
        });
        let underground = Player {
            pos: [-133.0, 55.0, -86.0],
            sky_light: 0,
            ..Player::default()
        };
        memory.observe(
            &Observations {
                players: vec![underground],
                ..Observations::default()
            },
            0.25,
        );
        assert!(memory.last.underground);
        assert!(!memory.last.indoors, "a cave is not indoors");
        assert_eq!(house_spot("door_sound", &memory, 0), None);

        let config = crate::config::Config::default();
        let now = Now {
            day_time: 6_000,
            ..Now::default()
        };
        let menu = crate::brain::offer::offer(
            crate::events::CATALOGUE,
            5,
            &memory,
            &now,
            0.25,
            &config,
            &mut crate::model::Rng::new(1),
        );
        assert!(
            menu.iter().all(|e| !HOUSE_SOUNDS.contains(&e.id)),
            "{:?}",
            menu.iter().map(|e| e.id).collect::<Vec<_>>()
        );
        let door = crate::events::find("door_sound").unwrap();
        let made = crate::brain::act::commands(door, None, &memory, &mut || 1, &now, None);
        assert!(made.is_empty(), "no spot, no sound: {made:?}");
    }

    #[test]
    fn below_is_under_their_floor_and_deep_far_under() {
        let mut memory = Memory::default();
        memory.last.pos = [0.5, 64.0, 0.5];
        let below = v(
            &[("minecraft:block.water.ambient", 0)],
            (1.0, 1.0),
            1.0,
            Below,
        );
        let deep = v(
            &[("minecraft:block.water.ambient", 0)],
            (1.0, 1.0),
            1.0,
            Deep,
        );
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        let at = |made: Vec<(Command, u64)>| match &made[0].0 {
            Command::PlaySound { pos, .. } => *pos,
            other => panic!("{other:?}"),
        };
        assert_eq!(at(commands(&below, None, &memory, 0, 0, &mut ids))[1], 61.5);
        assert_eq!(at(commands(&deep, None, &memory, 0, 0, &mut ids))[1], 48.0);
    }
}
