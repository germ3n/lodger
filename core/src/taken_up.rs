//! Their things taken up into the sky (`taken_up`, `docs/events_todo.md` 103, owner's), and given
//! back as they fall out of it in front of their door (`sky_gives_back`). The host picks and keeps
//! the stacks (`take_up`, under `taken_up_<n>_<i>`); the memory knows only that some are up there,
//! and since when. Nothing is ever kept for good: an ending and a reset give them back too.

use craftkit::world::{BlockPos, Command, Pos};

use crate::memory::Memory;

/// The names the host keeps them under begin so.
pub const KEEP: &str = "taken_up_";
/// At most this many stacks a time, and the fewest.
pub const MOST: u32 = 4;
/// Up there at least this long (a day of the stay) before they fall back.
pub const UP_DAYS: f32 = 0.5;

impl Memory {
    /// Whether its things have been up there long enough to fall back.
    pub fn taken_up_due(&self) -> bool {
        self.taken_up_day
            .is_some_and(|day| self.today - day >= UP_DAYS)
    }
}

/// How many it takes this time: one to [`MOST`], by the times it did it and where.
pub fn how_many(times: usize, at: BlockPos) -> u32 {
    let seed = times as i64 * 31 + at[0] as i64 * 7 + at[2] as i64 * 13;

    (seed.rem_euclid(MOST as i64) + 1) as u32
}

/// Everything up there dropped at `pos`, out of the sky: a command that does nothing when none
/// is kept. `memory.taken_up_day` is cleared by the caller once it is sent.
pub fn fall_back(id: u64, pos: Pos) -> Command {
    Command::DropKept {
        id,
        prefix: KEEP.to_string(),
        pos,
    }
}

/// Where they fall back by the home: two out from the door they use, else the home's middle,
/// else at the target's feet.
pub fn fall_spot(memory: &Memory) -> Pos {
    if let (Some(door), Some(home)) = (memory.usual_door(), memory.home.as_ref()) {
        let out = crate::brain::act::out_of(memory, home.center, door);
        return [
            (door[0] + out[0] * 2) as f64 + 0.5,
            door[1] as f64,
            (door[2] + out[1] * 2) as f64 + 0.5,
        ];
    }
    if let Some(home) = memory.home.as_ref() {
        return [
            home.center[0] as f64 + 0.5,
            home.floor_y as f64 + 1.0,
            home.center[2] as f64 + 0.5,
        ];
    }

    memory.last.pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_to_four_a_time_and_due_after_half_a_day() {
        for times in 0..20 {
            let n = how_many(times, [times as i32 * 5, 64, -3]);
            assert!((1..=MOST).contains(&n), "{n}");
        }
        let counts: std::collections::HashSet<u32> =
            (0..20).map(|t| how_many(t, [3, 64, 9])).collect();
        assert!(counts.len() > 1, "a random number, not the same every time");

        assert!(!Memory::default().taken_up_due());
        let mut memory = Memory {
            taken_up_day: Some(1.0),
            ..Memory::default()
        };
        memory.today = 1.2;
        assert!(!memory.taken_up_due(), "not yet");
        memory.today = 1.6;
        assert!(memory.taken_up_due());
    }
}
