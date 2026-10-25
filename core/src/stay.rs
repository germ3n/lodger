//! One lodger in one world: the `World` the host opens, observes and ticks.
//!
//! Phase 2's first cut: the stay opens from its saved bytes or fresh, counts its ticks and the
//! players it saw, and saves itself as `bincode` behind a format byte, so a save written by this
//! version is read by the next and a newer one refused rather than misread. The memory, the
//! brain and the arc attach here in the phases that follow (`docs/todo.md`).

use craftkit::{Command, Now, Observations, Report, WorldInfo};
use serde::{Deserialize, Serialize};

/// The first byte of every save. Bumped when the layout changes; a newer byte is refused.
pub const FORMAT: u8 = 1;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stay {
    /// The dimension's id, so a stay saved in one world is not opened for another.
    pub world: String,
    pub seed: i64,
    /// Ticks this stay has been open, over all its sessions.
    pub ticks: u64,
    /// The last tick's player count, for `/lodger status`.
    pub players: u32,
    /// Commands reported done and not done, for the log.
    pub done: u64,
    pub not_done: u64,
}

impl Stay {
    /// A fresh stay in `info`'s world.
    pub fn new(info: &WorldInfo) -> Self {
        Self {
            world: info.id.clone(),
            seed: info.seed,
            ..Self::default()
        }
    }

    /// Reads a save, or starts fresh from empty bytes. A save from another format byte or
    /// another world is refused by name.
    pub fn open(info: &WorldInfo, saved: &[u8]) -> Result<Self, String> {
        if saved.is_empty() {
            return Ok(Self::new(info));
        }

        let (format, body) = saved
            .split_first()
            .ok_or_else(|| "an empty save".to_string())?;

        if *format != FORMAT {
            return Err(format!(
                "the save is format {format}; this lodger reads format {FORMAT}"
            ));
        }

        let stay: Self =
            bincode::deserialize(body).map_err(|err| format!("the save is damaged: {err}"))?;

        if stay.world != info.id {
            return Err(format!(
                "the save is for {} but this world is {}",
                stay.world, info.id
            ));
        }

        Ok(stay)
    }

    /// The save: the format byte, then the stay.
    pub fn save(&self) -> Vec<u8> {
        let mut bytes = vec![FORMAT];
        bytes.extend(bincode::serialize(self).expect("a stay serializes"));

        bytes
    }

    pub fn observe(&mut self, observations: &Observations) {
        self.players = observations.players.len() as u32;
    }

    pub fn tick(&mut self, _now: &Now) -> Vec<Command> {
        self.ticks += 1;

        Vec::new()
    }

    pub fn report(&mut self, reports: &[Report]) {
        for report in reports {
            if report.done {
                self.done += 1;
            } else {
                self.not_done += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> WorldInfo {
        WorldInfo {
            id: "minecraft:overworld".to_string(),
            seed: 7,
            name: "w".to_string(),
        }
    }

    #[test]
    fn a_stay_round_trips_through_its_save_with_the_format_byte_first() {
        let mut stay = Stay::new(&info());
        stay.tick(&Now::default());
        stay.tick(&Now::default());
        stay.report(&[
            Report {
                id: 1,
                done: true,
                reason: String::new(),
            },
            Report {
                id: 2,
                done: false,
                reason: "in view".to_string(),
            },
        ]);

        let bytes = stay.save();
        assert_eq!(bytes[0], FORMAT);
        let back = Stay::open(&info(), &bytes).unwrap();
        assert_eq!(back, stay);
        assert_eq!(back.ticks, 2);
        assert_eq!((back.done, back.not_done), (1, 1));
    }

    #[test]
    fn empty_bytes_are_a_fresh_stay_in_this_world() {
        let stay = Stay::open(&info(), &[]).unwrap();

        assert_eq!(stay.world, "minecraft:overworld");
        assert_eq!(stay.seed, 7);
        assert_eq!(stay.ticks, 0);
    }

    #[test]
    fn a_newer_format_and_another_world_are_refused_by_name() {
        let mut newer = Stay::new(&info()).save();
        newer[0] = FORMAT + 1;
        assert!(Stay::open(&info(), &newer).unwrap_err().contains("format"));

        let other = WorldInfo {
            id: "minecraft:the_nether".to_string(),
            ..info()
        };
        let err = Stay::open(&other, &Stay::new(&info()).save()).unwrap_err();
        assert!(err.contains("minecraft:overworld") && err.contains("minecraft:the_nether"));

        assert!(Stay::open(&info(), &[FORMAT, 1, 2])
            .unwrap_err()
            .contains("damaged"));
    }
}
