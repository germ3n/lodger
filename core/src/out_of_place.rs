//! Things out of place (`out_of_place`, `docs/events_todo.md` 110, owner's: "random items that
//! don't belong somewhere ... not just limited to caves, just in general"): a small scene set out
//! where they will come on it, a cave they have not entered or their own way back, never the same
//! twice running; taken away again ten minutes on, so it gives nothing it did not take.

use craftkit::world::{BlockPos, BlueprintPart};

use crate::memory::{block_of, distance, Memory};

/// How long a scene stands, ticks: ten minutes.
pub const STANDS_TICKS: u64 = 10 * 60 * 20;

/// The scenes, by turns: each its parts over the floor (`y` 1 on it).
pub fn scene(n: usize) -> (&'static str, Vec<BlueprintPart>) {
    let part = |at: [i32; 3], block: &str, state: &str| BlueprintPart {
        at,
        block: block.to_string(),
        state: state.to_string(),
    };
    match n % 5 {
        0 => (
            "a made bed",
            vec![
                part([0, 1, 0], "minecraft:red_bed", "part=foot,facing=south"),
                part([0, 1, 1], "minecraft:red_bed", "part=head,facing=south"),
            ],
        ),
        1 => (
            "a table laid, a cake and a lit candle",
            vec![
                part([0, 1, 0], "minecraft:oak_fence", ""),
                part([0, 2, 0], "minecraft:oak_pressure_plate", ""),
                part([1, 1, 0], "minecraft:candle_cake", "lit=true"),
            ],
        ),
        2 => (
            "a chair facing nothing",
            vec![part(
                [0, 1, 0],
                "minecraft:oak_stairs",
                "facing=north,half=bottom",
            )],
        ),
        3 => (
            "a flower in a pot",
            vec![part([0, 1, 0], "minecraft:potted_poppy", "")],
        ),
        _ => (
            "a carpet and a lantern lit",
            vec![
                part([0, 1, 0], "minecraft:red_carpet", ""),
                part([1, 1, 0], "minecraft:lantern", "hanging=false"),
            ],
        ),
    }
}

impl Memory {
    /// Where a scene goes: underground, the floor of a cave they have not entered; above, their
    /// own way back, a point of their trail sixteen to thirty blocks from them. The floor's block.
    pub fn out_of_place_spot(&self) -> Option<BlockPos> {
        let at = block_of(self.last.pos);
        if self.last.underground {
            return self.lantern_spot().map(|p| [p[0], p[1] - 1, p[2]]);
        }
        self.trail
            .iter()
            .find(|p| (16.0..=30.0).contains(&distance(**p, at)))
            .map(|p| [p[0], p[1] - 1, p[2]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_scenes_by_turns_each_over_its_floor() {
        let names: std::collections::HashSet<&str> = (0..5).map(|n| scene(n).0).collect();
        assert_eq!(names.len(), 5, "never the same twice running");
        for n in 0..5 {
            assert!(
                scene(n).1.iter().all(|p| p.at[1] >= 1),
                "on the floor, not in it"
            );
        }
        let mut memory = Memory::default();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.trail = vec![[0, 64, -40], [0, 64, -20], [0, 64, -5]];
        assert_eq!(
            memory.out_of_place_spot(),
            Some([0, 63, -20]),
            "the way back, on its floor"
        );
    }
}
