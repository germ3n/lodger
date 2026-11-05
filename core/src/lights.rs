//! The target's lights, for the lights events of batch 4 (owner, 2026-11-10): the ones behind
//! them that go out one by one as they pass at night and come back a minute later
//! (`lights_out`), and the one that turns to soul fire while they are away (`soul_swap`).
//! Their lights are the ones they placed, by id: the building blocks the memory keeps
//! (`Memory::walls`, torches and lanterns among them) and the last light they placed.

use craftkit::world::BlockPos;

use crate::memory::{block_of, distance, Memory};

/// Behind them within this many blocks, a light goes out as they pass.
const BEHIND_BLOCKS: f64 = 20.0;
/// The most lights that go out at once.
const MOST_OUT: usize = 5;

/// A light's soul fire form, the same shape: none for what has none (a candle, a soul light).
pub fn soul_of(id: &str) -> Option<&'static str> {
    match id {
        "minecraft:torch" => Some("minecraft:soul_torch"),
        "minecraft:wall_torch" => Some("minecraft:soul_wall_torch"),
        "minecraft:lantern" => Some("minecraft:soul_lantern"),
        "minecraft:campfire" => Some("minecraft:soul_campfire"),
        _ => None,
    }
}

fn is_light(id: &str) -> bool {
    id.ends_with("torch") || id.ends_with("lantern") || crate::memory::is_mod_lamp(id)
}

impl Memory {
    /// The lights they placed and have not broken, by position and id.
    pub fn their_lights(&self) -> Vec<(BlockPos, String)> {
        let mut lights: Vec<(BlockPos, String)> = self
            .walls
            .iter()
            .filter(|(_, id)| is_light(id))
            .cloned()
            .collect();
        if let Some(last) = &self.habits.last_light {
            if !lights.iter().any(|(p, _)| *p == last.0) {
                lights.push(last.clone());
            }
        }

        lights
    }

    /// The lights behind them as they go, nearest first: within 20 blocks, on the side they
    /// face away from, out of their view. At most five.
    pub fn lights_behind(&self) -> Vec<(BlockPos, String)> {
        let at = block_of(self.last.pos);
        let yaw = (self.last.yaw as f64).to_radians();
        // Facing: yaw 0 faces +z, 90 faces -x.
        let ahead = [-yaw.sin(), yaw.cos()];
        let mut behind: Vec<(BlockPos, String)> = self
            .their_lights()
            .into_iter()
            .filter(|(p, _)| {
                let (dx, dz) = ((p[0] - at[0]) as f64, (p[2] - at[2]) as f64);
                distance(*p, at) <= BEHIND_BLOCKS
                    && dx * ahead[0] + dz * ahead[1] < 0.0
                    && !self.in_view(*p, BEHIND_BLOCKS)
            })
            .collect();
        behind.sort_by(|a, b| distance(a.0, at).total_cmp(&distance(b.0, at)));
        behind.truncate(MOST_OUT);

        behind
    }

    /// A light of theirs that has a soul fire form, out of their view: the one farthest from
    /// them, so it is found later rather than seen change.
    pub fn soulable_light(&self) -> Option<(BlockPos, String)> {
        let at = block_of(self.last.pos);
        self.their_lights()
            .into_iter()
            .filter(|(p, id)| soul_of(id).is_some() && !self.in_view(*p, 32.0))
            .max_by(|a, b| distance(a.0, at).total_cmp(&distance(b.0, at)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_facing_south_at_origin() -> Memory {
        let mut memory = Memory::default();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 0.0; // facing +z
        memory.last.online = true;
        memory
    }

    #[test]
    fn the_lights_behind_are_the_ones_on_the_far_side_from_their_facing_nearest_first() {
        let mut memory = memory_facing_south_at_origin();
        memory.walls = vec![
            ([0, 65, -4], "minecraft:wall_torch".to_string()),
            ([1, 65, -12], "minecraft:torch".to_string()),
            ([0, 65, 6], "minecraft:torch".to_string()), // ahead
            ([0, 65, -40], "minecraft:torch".to_string()), // too far
            ([2, 64, -2], "minecraft:oak_planks".to_string()), // no light
        ];

        let behind: Vec<BlockPos> = memory.lights_behind().into_iter().map(|(p, _)| p).collect();
        assert_eq!(behind, vec![[0, 65, -4], [1, 65, -12]]);
    }

    #[test]
    fn the_last_light_counts_and_is_not_counted_twice() {
        let mut memory = memory_facing_south_at_origin();
        memory.walls = vec![([0, 65, -4], "minecraft:torch".to_string())];
        memory.habits.last_light = Some(([0, 65, -4], "minecraft:torch".to_string()));
        assert_eq!(memory.their_lights().len(), 1);
        memory.habits.last_light = Some(([3, 65, -6], "minecraft:lantern".to_string()));
        assert_eq!(memory.their_lights().len(), 2);
    }

    #[test]
    fn a_light_turns_to_its_own_soul_form_and_the_farthest_is_chosen() {
        assert_eq!(
            soul_of("minecraft:wall_torch"),
            Some("minecraft:soul_wall_torch")
        );
        assert_eq!(soul_of("minecraft:lantern"), Some("minecraft:soul_lantern"));
        assert_eq!(soul_of("minecraft:soul_torch"), None);
        assert_eq!(soul_of("minecraft:candle"), None);

        let mut memory = memory_facing_south_at_origin();
        memory.last.online = false;
        memory.walls = vec![
            ([0, 65, -4], "minecraft:torch".to_string()),
            ([30, 65, 0], "minecraft:lantern".to_string()),
            ([60, 65, 0], "minecraft:soul_torch".to_string()),
        ];
        assert_eq!(
            memory.soulable_light(),
            Some(([30, 65, 0], "minecraft:lantern".to_string()))
        );
    }

    #[test]
    fn the_lights_go_out_one_by_one_for_a_minute_and_the_soul_swap_keeps_the_shape() {
        use craftkit::world::{Command, Now};
        let mut memory = memory_facing_south_at_origin();
        memory.walls = vec![
            ([0, 65, -4], "minecraft:wall_torch".to_string()),
            ([1, 65, -12], "minecraft:torch".to_string()),
        ];
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        let now = Now::default();
        let out = crate::events::find("lights_out").unwrap();
        let made =
            crate::brain::act::commands(out, Some([0, 65, -4]), &memory, &mut ids, &now, None);
        let hidden: Vec<(BlockPos, f32, u64)> = made
            .iter()
            .filter_map(|(c, delay)| match c {
                Command::HideBlock {
                    pos, seconds, only, ..
                } => {
                    assert_eq!(only.len(), 1);
                    Some((*pos, *seconds, *delay))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            hidden,
            vec![([0, 65, -4], 60.0, 0), ([1, 65, -12], 60.0, 15)]
        );

        memory.last.online = false;
        let soul = crate::events::find("soul_swap").unwrap();
        let made =
            crate::brain::act::commands(soul, Some([1, 65, -12]), &memory, &mut ids, &now, None);
        assert!(
            matches!(&made[0].0, Command::SwapBlock { block, only, .. }
                if block == "minecraft:soul_torch" && only == &["minecraft:torch".to_string()]),
            "{made:?}"
        );
    }
}
