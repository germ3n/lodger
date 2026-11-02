//! The target's tunnels (batch 13, owner 2026-11-10): what they dig underground, grouped as it
//! is dug, a dig within three blocks of a tunnel's last carrying it on; each tunnel's mouth,
//! end, way and floor; the newest and the oldest; the ore ahead of one; the one they stand in.

use craftkit::world::BlockPos;
use serde::{Deserialize, Serialize};

use crate::memory::{block_of, distance, Memory};

/// A tunnel counts from this many columns.
pub const LEAST_TUNNEL: usize = 10;
const MOST_TUNNELS: usize = 16;
const MOST_BLOCKS: usize = 256;
/// A dig within this of a tunnel's last block carries it on.
const CARRIES_ON: f64 = 3.0;

/// The natural rock a tunnel is dug through, its ores among it.
pub fn is_rock(id: &str) -> bool {
    let name = id.rsplit(':').next().unwrap_or(id);
    matches!(
        name,
        "stone"
            | "deepslate"
            | "andesite"
            | "diorite"
            | "granite"
            | "tuff"
            | "calcite"
            | "dirt"
            | "gravel"
            | "netherrack"
            | "basalt"
            | "blackstone"
    ) || name.ends_with("_ore")
}

/// The rock a dig of its own may take, by id: never anything a player placed or built with.
pub const ROCK_IDS: &[&str] = &[
    "minecraft:stone",
    "minecraft:deepslate",
    "minecraft:andesite",
    "minecraft:diorite",
    "minecraft:granite",
    "minecraft:tuff",
    "minecraft:calcite",
    "minecraft:dirt",
    "minecraft:gravel",
    "minecraft:grass_block",
    "minecraft:coal_ore",
    "minecraft:iron_ore",
    "minecraft:copper_ore",
    "minecraft:gold_ore",
    "minecraft:redstone_ore",
    "minecraft:lapis_ore",
    "minecraft:diamond_ore",
    "minecraft:emerald_ore",
    "minecraft:deepslate_coal_ore",
    "minecraft:deepslate_iron_ore",
    "minecraft:deepslate_copper_ore",
    "minecraft:deepslate_gold_ore",
    "minecraft:deepslate_redstone_ore",
    "minecraft:deepslate_lapis_ore",
    "minecraft:deepslate_diamond_ore",
    "minecraft:deepslate_emerald_ore",
];

/// What digging a block gives, as a player's pick would: stone is cobblestone, an ore its
/// drop; anything else itself.
pub fn dug_as(id: &str) -> String {
    let name = id.rsplit(':').next().unwrap_or(id);
    let base = name.strip_prefix("deepslate_").unwrap_or(name);
    let drop = match base {
        "stone" => "cobblestone",
        "deepslate" => "cobbled_deepslate",
        "grass_block" => "dirt",
        "coal_ore" => "coal",
        "iron_ore" => "raw_iron",
        "copper_ore" => "raw_copper",
        "gold_ore" => "raw_gold",
        "redstone_ore" => "redstone",
        "lapis_ore" => "lapis_lazuli",
        "diamond_ore" => "diamond",
        "emerald_ore" => "emerald",
        _ => return id.to_string(),
    };
    if name == "deepslate" {
        return "minecraft:cobbled_deepslate".to_string();
    }

    format!("minecraft:{drop}")
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tunnel {
    /// The blocks dug, in the order dug.
    pub blocks: Vec<BlockPos>,
    pub first_day: f32,
    pub last_day: f32,
}

impl Tunnel {
    /// Its columns in the order dug, each at its floor (the lowest block dug there).
    pub fn columns(&self) -> Vec<BlockPos> {
        let mut out: Vec<BlockPos> = Vec::new();
        for b in &self.blocks {
            match out.iter_mut().find(|c| c[0] == b[0] && c[2] == b[2]) {
                Some(c) if b[1] < c[1] => *c = *b,
                Some(_) => {}
                None => out.push(*b),
            }
        }

        out
    }

    pub fn length(&self) -> usize {
        self.columns().len()
    }

    pub fn mouth(&self) -> BlockPos {
        self.columns()[0]
    }

    pub fn end(&self) -> BlockPos {
        *self.columns().last().unwrap_or(&self.blocks[0])
    }

    /// The way it goes, mouth to end, along the axis it goes most: `[dx, dz]` of one block.
    pub fn way(&self) -> [i32; 2] {
        let (m, e) = (self.mouth(), self.end());
        let (dx, dz) = (e[0] - m[0], e[2] - m[2]);
        if dx.abs() >= dz.abs() {
            [dx.signum().max(if dx == 0 { 1 } else { -1 }), 0]
        } else {
            [0, dz.signum()]
        }
    }

    /// Whether a place is in it: within two blocks of a block dug.
    pub fn holds(&self, pos: BlockPos) -> bool {
        self.blocks.iter().any(|b| distance(*b, pos) <= 2.0)
    }
}

impl Memory {
    /// A dig underground: carried into the tunnel it continues, or the start of another.
    pub(crate) fn dug_underground(&mut self, pos: BlockPos, day: f32) {
        let carried = self
            .tunnels
            .iter_mut()
            .filter(|t| {
                t.blocks
                    .last()
                    .is_some_and(|b| distance(*b, pos) <= CARRIES_ON)
            })
            .max_by(|a, b| a.last_day.total_cmp(&b.last_day));
        match carried {
            Some(tunnel) => {
                if !tunnel.blocks.contains(&pos) && tunnel.blocks.len() < MOST_BLOCKS {
                    tunnel.blocks.push(pos);
                }
                tunnel.last_day = day;
            }
            None => {
                self.tunnels.push(Tunnel {
                    blocks: vec![pos],
                    first_day: day,
                    last_day: day,
                });
                if self.tunnels.len() > MOST_TUNNELS {
                    let least = self
                        .tunnels
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, t)| t.blocks.len())
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.tunnels.remove(least);
                }
            }
        }
    }

    /// Their tunnels of ten columns or more.
    pub fn real_tunnels(&self) -> impl Iterator<Item = &Tunnel> {
        self.tunnels.iter().filter(|t| t.length() >= LEAST_TUNNEL)
    }

    pub fn newest_tunnel(&self) -> Option<&Tunnel> {
        self.real_tunnels()
            .max_by(|a, b| a.last_day.total_cmp(&b.last_day))
    }

    pub fn oldest_tunnel(&self) -> Option<&Tunnel> {
        self.real_tunnels()
            .min_by(|a, b| a.first_day.total_cmp(&b.first_day))
    }

    /// The tunnel they stand in.
    pub fn tunnel_at_target(&self) -> Option<&Tunnel> {
        let at = block_of(self.last.pos);
        self.real_tunnels().find(|t| t.holds(at))
    }

    /// The ore ahead of their newest tunnel: within 16 blocks past its end, along its way.
    pub fn ore_ahead(&self) -> Option<BlockPos> {
        let tunnel = self.newest_tunnel()?;
        let (end, way) = (tunnel.end(), tunnel.way());
        self.ores
            .iter()
            .map(|(_, p)| *p)
            .filter(|p| {
                let ahead = (p[0] - end[0]) * way[0] + (p[2] - end[2]) * way[1];
                ahead > 0 && ahead <= 16 && distance(*p, end) <= 18.0
            })
            .min_by(|a, b| distance(*a, end).total_cmp(&distance(*b, end)))
    }

    /// Where the way back closes: the tunnel's floor four to six columns behind them, toward
    /// its mouth.
    pub fn behind_in_tunnel(&self) -> Option<BlockPos> {
        let tunnel = self.tunnel_at_target()?;
        let at = block_of(self.last.pos);
        let columns = tunnel.columns();
        let here = (0..columns.len())
            .min_by(|a, b| distance(columns[*a], at).total_cmp(&distance(columns[*b], at)))?;
        (here >= 5).then(|| columns[here - 5])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tunnel dug east along z 0 at y 20, two high, `n` columns from x 0.
    fn dig(memory: &mut Memory, n: i32, z: i32, day: f32) {
        for x in 0..n {
            memory.dug_underground([x, 21, z], day);
            memory.dug_underground([x, 20, z], day);
        }
    }

    #[test]
    fn digs_in_a_line_are_one_tunnel_with_its_mouth_end_and_way() {
        let mut memory = Memory::default();
        dig(&mut memory, 14, 0, 1.0);

        assert_eq!(memory.tunnels.len(), 1);
        let tunnel = memory.newest_tunnel().unwrap();
        assert_eq!(tunnel.length(), 14);
        assert_eq!(tunnel.mouth(), [0, 20, 0], "the floor");
        assert_eq!(tunnel.end(), [13, 20, 0]);
        assert_eq!(tunnel.way(), [1, 0]);
    }

    #[test]
    fn a_dig_far_off_starts_another_and_a_short_one_is_no_tunnel() {
        let mut memory = Memory::default();
        dig(&mut memory, 12, 0, 1.0);
        dig(&mut memory, 4, 40, 2.0);

        assert_eq!(memory.tunnels.len(), 2);
        assert_eq!(memory.real_tunnels().count(), 1);
        dig(&mut memory, 12, 80, 3.0);
        assert_eq!(memory.oldest_tunnel().unwrap().mouth(), [0, 20, 0]);
        assert_eq!(memory.newest_tunnel().unwrap().mouth(), [0, 20, 80]);
    }

    #[test]
    fn the_ore_ahead_and_the_way_back() {
        let mut memory = Memory::default();
        dig(&mut memory, 12, 0, 1.0);
        memory.ores = vec![
            ("iron_ore".to_string(), [20, 20, 1]),
            ("coal_ore".to_string(), [-5, 20, 0]),
        ];
        assert_eq!(
            memory.ore_ahead(),
            Some([20, 20, 1]),
            "ahead, not behind the mouth"
        );

        memory.last.pos = [10.5, 20.0, 0.5];
        assert_eq!(memory.behind_in_tunnel(), Some([5, 20, 0]));
        memory.last.pos = [2.5, 20.0, 0.5];
        assert_eq!(
            memory.behind_in_tunnel(),
            None,
            "too near the mouth to close"
        );
    }

    #[test]
    fn digging_gives_what_a_pick_would() {
        assert_eq!(dug_as("minecraft:stone"), "minecraft:cobblestone");
        assert_eq!(dug_as("minecraft:deepslate"), "minecraft:cobbled_deepslate");
        assert_eq!(dug_as("minecraft:deepslate_iron_ore"), "minecraft:raw_iron");
        assert_eq!(dug_as("minecraft:diamond_ore"), "minecraft:diamond");
        assert_eq!(dug_as("minecraft:gravel"), "minecraft:gravel");
        assert!(is_rock("minecraft:tuff") && is_rock("minecraft:copper_ore"));
        assert!(!is_rock("minecraft:oak_planks"));
    }
}
