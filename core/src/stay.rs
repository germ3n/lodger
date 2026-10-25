//! One lodger in one world: the `World` the host opens, observes and ticks. The stay owns the
//! memory, the arc and the brain, saves itself as JSON behind a format byte, so a save written
//! by this version is read by the next (a field added with `#[serde(default)]` costs no
//! migration) and a newer format is refused rather than misread,
//! and does the per-tick work: take the observations in, advance the stage, let the brain
//! decide and act, hand back the commands.

use crate::arc::Arc;
use crate::brain::{log::Log, Brain};
use crate::config::Config;
use crate::memory::Memory;
use craftkit::world::{Command, Now, Observations, Report, WorldInfo};
use serde::{Deserialize, Serialize};

/// The first byte of every save. Bumped when the layout changes; a newer byte is refused.
/// The save's format byte. Format 3 is JSON after the byte; formats 1 and 2 were bincode and
/// came from development builds alone, so only their counts are read.
pub const FORMAT: u8 = 3;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
    /// The clock as of the last tick, for the debug screen's questions between ticks. Not
    /// saved: a reopened stay reads the clock again on its first tick.
    #[serde(skip)]
    pub last_now: Now,
}

/// The first bincode layout's leading fields, which formats 1 and 2 both began with.
struct StayOld {
    world: String,
    seed: i64,
    ticks: u64,
    players: u32,
    done: u64,
    not_done: u64,
}

/// Reads the leading fields of a bincode save (formats 1 and 2): little-endian lengths and
/// integers, as bincode 1 wrote them. The rest of a format 2 save, the mind, is left behind.
fn read_old(body: &[u8]) -> Result<StayOld, String> {
    fn u64_at(body: &[u8], at: &mut usize) -> Result<u64, String> {
        let bytes: [u8; 8] = body
            .get(*at..*at + 8)
            .ok_or_else(|| "the save is damaged: too short".to_string())?
            .try_into()
            .unwrap();
        *at += 8;
        Ok(u64::from_le_bytes(bytes))
    }
    let mut at = 0;
    let len = u64_at(body, &mut at)? as usize;
    let world = std::str::from_utf8(
        body.get(at..at + len)
            .ok_or_else(|| "the save is damaged: too short".to_string())?,
    )
    .map_err(|_| "the save is damaged: the world is not text".to_string())?
    .to_string();
    at += len;
    let seed = u64_at(body, &mut at)? as i64;
    let ticks = u64_at(body, &mut at)?;
    let players = u32::from_le_bytes(
        body.get(at..at + 4)
            .ok_or_else(|| "the save is damaged: too short".to_string())?
            .try_into()
            .unwrap(),
    );
    at += 4;
    let done = u64_at(body, &mut at)?;
    let not_done = u64_at(body, &mut at)?;

    Ok(StayOld {
        world,
        seed,
        ticks,
        players,
        done,
        not_done,
    })
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
                serde_json::from_slice(body).map_err(|err| format!("the save is damaged: {err}"))?
            }
            1 | 2 => {
                // Formats 1 and 2 were bincode from development builds: the counts carry over
                // and the rest starts fresh, which for format 1 is what it would have been.
                let old = read_old(body)?;
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
        bytes.extend(serde_json::to_vec(self).expect("a stay serializes"));

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
        self.last_now = now.clone();
        if self.first_day.is_none() {
            self.first_day = Some(now.day);
        }

        // Only the overworld is lived in; the lodger never follows into another dimension.
        if self.world != "minecraft:overworld" || !config.enabled {
            return Vec::new();
        }

        let day = self.day(now);
        self.memory.clock(now, day);
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
    fn a_bincode_save_of_format_one_or_two_opens_with_its_counts_and_a_fresh_mind() {
        // What bincode 1 wrote for the leading fields: a length-prefixed string and
        // little-endian integers.
        let mut bytes = vec![2u8];
        let world = b"minecraft:overworld";
        bytes.extend((world.len() as u64).to_le_bytes());
        bytes.extend(world);
        bytes.extend(7i64.to_le_bytes());
        bytes.extend(400u64.to_le_bytes());
        bytes.extend(1u32.to_le_bytes());
        bytes.extend(3u64.to_le_bytes());
        bytes.extend(1u64.to_le_bytes());
        bytes.extend([0u8; 40]); // the mind that follows is left behind

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
        assert!(Stay::open(&info(), &[2, 1, 2])
            .unwrap_err()
            .contains("damaged"));
    }

    #[test]
    fn a_save_with_a_field_this_lodger_does_not_know_still_opens() {
        let mut stay = Stay::new(&info());
        stay.ticks = 9;
        let mut value: serde_json::Value = serde_json::from_slice(&stay.save()[1..]).unwrap();
        value["memory"]["something_newer"] = serde_json::json!(1);
        value["memory"].as_object_mut().unwrap().remove("tally");
        let mut bytes = vec![FORMAT];
        bytes.extend(serde_json::to_vec(&value).unwrap());

        let back = Stay::open(&info(), &bytes).unwrap();
        assert_eq!(back.ticks, 9);
        assert!(back.memory.tally.is_empty());
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
