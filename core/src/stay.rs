//! One lodger in one world: the `World` the host opens, observes and ticks. The stay owns the
//! memory, the arc and the brain, saves itself as `bincode` behind a format byte, so a save
//! written by this version is read by the next and a newer one refused rather than misread,
//! and does the per-tick work: take the observations in, advance the stage, let the brain
//! decide and act, hand back the commands.

use crate::arc::Arc;
use crate::brain::{log::Log, Brain};
use crate::config::Config;
use crate::memory::Memory;
use craftkit::world::{Command, Now, Observations, Report, WorldInfo};
use serde::{Deserialize, Serialize};

/// The first byte of every save. Bumped when the layout changes; a newer byte is refused.
pub const FORMAT: u8 = 2;

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
    pub memory: Memory,
    pub arc: Arc,
    pub brain: Brain,
    /// The in-game day the stay began, so its days count from its own start.
    pub first_day: Option<u64>,
}

/// The stay's layout in format 1, kept so those saves open.
#[derive(Deserialize)]
struct StayV1 {
    world: String,
    seed: i64,
    ticks: u64,
    players: u32,
    done: u64,
    not_done: u64,
}

impl Stay {
    /// A fresh stay in `info`'s world.
    pub fn new(info: &WorldInfo) -> Self {
        Self {
            world: info.id.clone(),
            seed: info.seed,
            brain: Brain::new(info.seed as u64 ^ 0x6c6f_6467_6572),
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

        let stay: Self = match *format {
            FORMAT => {
                bincode::deserialize(body).map_err(|err| format!("the save is damaged: {err}"))?
            }
            1 => {
                // Format 1 was the stay before it had a memory, an arc and a brain: the counts
                // alone. They carry over; the rest starts fresh, which is what it would have been.
                let old: StayV1 = bincode::deserialize(body)
                    .map_err(|err| format!("the save is damaged: {err}"))?;
                let mut stay = Self::new(&WorldInfo {
                    id: old.world.clone(),
                    seed: old.seed,
                    name: String::new(),
                });
                stay.ticks = old.ticks;
                stay.players = old.players;
                stay.done = old.done;
                stay.not_done = old.not_done;
                stay
            }
            newer => {
                return Err(format!(
                    "the save is format {newer}; this lodger reads format {FORMAT}"
                ));
            }
        };

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

    /// The stay's own day, as a fraction, from the world's clock.
    pub fn day(&self, now: &Now) -> f32 {
        let first = self.first_day.unwrap_or(now.day);

        (now.day.saturating_sub(first)) as f32 + now.day_time as f32 / 24_000.0
    }

    pub fn observe(&mut self, observations: &Observations, now_day: f32) {
        self.players = observations.players.len() as u32;
        self.memory.observe(observations, now_day);
    }

    /// One tick. The lodger's config and log come from the mod, which is one per process.
    pub fn tick(&mut self, now: &Now, config: &Config, log: &mut Log) -> Vec<Command> {
        self.ticks += 1;
        if self.first_day.is_none() {
            self.first_day = Some(now.day);
        }

        // Only the overworld is lived in; the lodger never follows into another dimension.
        if self.world != "minecraft:overworld" || !config.enabled {
            return Vec::new();
        }

        let day = self.day(now);
        if let Some(stage) = self.arc.tick(day, &self.memory, config) {
            log.note(&format!("stage {stage} begins on day {day:.1}"));
        }

        self.brain
            .tick(now, day, &mut self.memory, &mut self.arc, config, log)
    }

    pub fn report(&mut self, reports: &[Report], log: &mut Log) {
        for report in reports {
            if report.done {
                self.done += 1;
            } else {
                self.not_done += 1;
            }
        }

        self.brain.report(reports, &mut self.memory, log);
    }

    /// One line for `/lodger status`.
    pub fn status_line(&self) -> String {
        if !self.arc.moved_in() {
            if self.arc.over() {
                return "the stay is over".to_string();
            }

            return "not moved in yet".to_string();
        }

        format!(
            "stage {}, {} with {}, {} decisions, {} events landed",
            self.arc.stage,
            if self.memory.target.is_some() {
                "staying"
            } else {
                "no target"
            },
            if self.memory.target_name.is_empty() {
                "nobody"
            } else {
                &self.memory.target_name
            },
            self.brain.decisions,
            self.memory
                .done
                .iter()
                .filter(|d| d.ok && d.event != "wait")
                .count()
        )
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

    fn log() -> Log {
        Log::new(None, 16, true)
    }

    #[test]
    fn a_stay_round_trips_through_its_save_with_the_format_byte_first() {
        let config = Config::default();
        let mut stay = Stay::new(&info());
        stay.tick(&Now::default(), &config, &mut log());
        stay.tick(&Now::default(), &config, &mut log());
        stay.report(
            &[
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
            ],
            &mut log(),
        );

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
        assert_eq!(stay.status_line(), "not moved in yet");
    }

    #[test]
    fn a_format_one_save_opens_with_its_counts_and_a_fresh_mind() {
        #[derive(Serialize)]
        struct Old {
            world: String,
            seed: i64,
            ticks: u64,
            players: u32,
            done: u64,
            not_done: u64,
        }
        let mut bytes = vec![1u8];
        bytes.extend(
            bincode::serialize(&Old {
                world: "minecraft:overworld".to_string(),
                seed: 7,
                ticks: 400,
                players: 1,
                done: 3,
                not_done: 1,
            })
            .unwrap(),
        );

        let stay = Stay::open(&info(), &bytes).unwrap();
        assert_eq!(stay.ticks, 400);
        assert_eq!((stay.done, stay.not_done), (3, 1));
        assert!(!stay.arc.moved_in());
        assert_eq!(stay.save()[0], FORMAT, "saved again in the new format");
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

    #[test]
    fn the_stay_s_days_count_from_its_first_tick() {
        let config = Config::default();
        let mut stay = Stay::new(&info());
        let now = Now {
            tick: 1,
            day: 100,
            day_time: 12_000,
            real_millis: 0,
        };
        stay.tick(&now, &config, &mut log());
        assert!((stay.day(&now) - 0.5).abs() < 1e-6);
        let later = Now {
            day: 103,
            day_time: 0,
            ..now
        };
        assert!((stay.day(&later) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn nothing_happens_outside_the_overworld() {
        let config = Config::default();
        let nether = WorldInfo {
            id: "minecraft:the_nether".to_string(),
            ..info()
        };
        let mut stay = Stay::new(&nether);
        for tick in 0..100_000u64 {
            let now = Now {
                tick,
                day: tick / 24_000,
                day_time: (tick % 24_000) as u32,
                real_millis: 0,
            };
            assert!(stay.tick(&now, &config, &mut log()).is_empty());
        }
        assert!(!stay.arc.moved_in());
    }
}
