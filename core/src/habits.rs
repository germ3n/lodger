//! The target's habits (the review's N1, owner 2026-11-09: "we should make better memory"):
//! the door they come and go by, where each part of the day goes (home, underground, the
//! field, the village, away), when they leave home and come back, whether they sleep, what they
//! carry every day, how much they light and how often they look behind them. Sampled every
//! hundred ticks from what the memory already sees; plain counts, read back as a few lines of
//! the situation text (`Memory::describe`) and by the events that want the usual door.

use craftkit::world::{BlockPos, Player};
use serde::{Deserialize, Serialize};

use crate::memory::{distance, Memory, HOME_RADIUS};

/// Where the target can be, in the order `Habits::hours` counts them.
pub const PLACES: [&str; 5] = ["home", "underground", "field", "village", "away"];
/// The four parts of a day, from sunrise: `day_time` 0, 6000, 12000 and 18000 on.
pub const QUARTERS: [&str; 4] = ["mornings", "afternoons", "evenings", "nights"];

/// Ticks between two samples: a sample is one count, so the counts are comparable.
const SAMPLE_TICKS: f32 = 100.0;
/// Samples a part of the day needs before it says where that part goes: a minute and a half.
const LEAST_QUARTER_SAMPLES: u32 = 18;
const MOST_TIMES: usize = 16; // the last leaves, returns and sleeps
const MOST_CARRIED: usize = 32; // the least-carried dropped first
const MOST_DOORS: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Habits {
    /// The last sample, as hundreds of ticks into the stay.
    pub sample: i64,
    /// Samples per part of the day and place, `[QUARTERS][PLACES]`.
    pub hours: [[u32; 5]; 4],
    /// The home's doors they used, and how often.
    pub doors: Vec<(BlockPos, u32)>,
    /// Whether they were at home at the last sample; none while away from the game.
    pub at_home: Option<bool>,
    /// The time of day (0 to 24000) they last left home and came back, newest last.
    pub leaves: Vec<u32>,
    pub returns: Vec<u32>,
    /// Nights they slept and nights they stayed up through, one count a night.
    pub nights_slept: u32,
    pub nights_up: u32,
    /// The stay day whose night was counted last.
    pub night_counted: Option<i64>,
    /// The time of day they lay down, newest last.
    pub sleep_times: Vec<u32>,
    /// Items held or worn, by id, and on how many days.
    pub carried: Vec<(String, u32)>,
    /// What was held or worn on the day being counted.
    pub carried_today: Vec<String>,
    /// Days they played, and the last of them.
    pub days_seen: u32,
    pub day_seen: Option<i64>,
    /// Lights they placed anywhere.
    pub lights_placed: u32,
    /// Samples online, and those of them looking behind them twice or more in the last minute.
    pub samples: u32,
    pub uneasy: u32,
}

/// `day_time` in words: `at dawn`, `in the morning`, `around noon`, `in the afternoon`,
/// `at dusk`, `at night`.
pub fn time_words(day_time: u32) -> &'static str {
    match day_time % 24_000 {
        0..=999 | 22_500..=23_999 => "at dawn",
        1_000..=4_999 => "in the morning",
        5_000..=6_999 => "around noon",
        7_000..=11_499 => "in the afternoon",
        11_500..=13_499 => "at dusk",
        _ => "at night",
    }
}

fn place_phrase(place: usize) -> &'static str {
    match PLACES[place] {
        "home" => "at home",
        "underground" => "underground",
        "field" => "out at the field",
        "village" => "in the village",
        _ => "away from home",
    }
}

fn is_armour(id: &str) -> bool {
    [
        "helmet",
        "chestplate",
        "leggings",
        "boots",
        "elytra",
        "turtle_scute",
    ]
    .iter()
    .any(|k| id.ends_with(k))
}

/// The middle of a list of times of day, or none for fewer than three.
fn usual(times: &[u32]) -> Option<u32> {
    if times.len() < 3 {
        return None;
    }
    let mut sorted = times.to_vec();
    sorted.sort_unstable();

    Some(sorted[sorted.len() / 2])
}

impl Memory {
    /// The day's hundred-tick sample of where they are and what they hold; once per sample
    /// however often it is called.
    pub(crate) fn sample_habits(&mut self, player: &Player, day: f32) {
        let sample = (day * 24_000.0 / SAMPLE_TICKS).floor() as i64;
        if sample == self.habits.sample {
            return;
        }
        self.habits.sample = sample;
        let day_time = (day.fract() * 24_000.0) as u32;
        let today = day.floor() as i64;
        let at = [
            player.pos[0].floor() as i32,
            player.pos[1].floor() as i32,
            player.pos[2].floor() as i32,
        ];

        // Where: home, underground, the field, the village or away.
        let home_distance = self.home.as_ref().map(|h| flat_distance(h.center, at));
        let near = |kind: &str, blocks: f64| {
            self.surroundings.as_ref().is_some_and(|s| {
                s.features
                    .iter()
                    .any(|f| f.kind == kind && flat_distance(f.pos, at) <= blocks)
            })
        };
        let place = if home_distance.is_some_and(|d| d <= (HOME_RADIUS + 8) as f64) {
            0
        } else if self.last.underground {
            1
        } else if near("crops", 16.0) {
            2
        } else if near("village", 64.0) {
            3
        } else {
            4
        };
        let quarter = (day_time / 6_000).min(3) as usize;
        self.habits.hours[quarter][place] += 1;

        // Leaving and coming back, with room between so the edge does not count twice.
        if let Some(d) = home_distance {
            let inside = if d <= HOME_RADIUS as f64 {
                Some(true)
            } else if d > (HOME_RADIUS + 16) as f64 {
                Some(false)
            } else {
                None
            };
            match (self.habits.at_home, inside) {
                (Some(true), Some(false)) => push_capped(&mut self.habits.leaves, day_time),
                (Some(false), Some(true)) => push_capped(&mut self.habits.returns, day_time),
                _ => {}
            }
            if inside.is_some() {
                self.habits.at_home = inside;
            }
        }

        // A night stayed up through: late, awake, and no sleep counted for it.
        if (21_000..23_000).contains(&day_time)
            && !player.sleeping
            && self.habits.night_counted != Some(today)
        {
            self.habits.nights_up += 1;
            self.habits.night_counted = Some(today);
        }

        // What they hold and wear, counted once a day.
        if self.habits.day_seen != Some(today) {
            self.habits.day_seen = Some(today);
            self.habits.days_seen += 1;
            self.habits.carried_today.clear();
        }
        let items = [&player.main_hand, &player.off_hand]
            .into_iter()
            .chain(player.armor.iter())
            .filter(|s| !s.id.is_empty())
            .map(|s| s.id.clone())
            .collect::<Vec<_>>();
        for id in items {
            if self.habits.carried_today.contains(&id) {
                continue;
            }
            self.habits.carried_today.push(id.clone());
            match self.habits.carried.iter_mut().find(|(i, _)| *i == id) {
                Some((_, days)) => *days += 1,
                None => {
                    self.habits.carried.push((id, 1));
                    if self.habits.carried.len() > MOST_CARRIED {
                        let least = self
                            .habits
                            .carried
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, (_, days))| *days)
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        self.habits.carried.remove(least);
                    }
                }
            }
        }

        self.habits.samples += 1;
        if player.movement.look_backs >= 2 {
            self.habits.uneasy += 1;
        }
    }

    /// They lay down: one night slept, whatever time it is.
    pub(crate) fn habit_slept(&mut self, day: f32) {
        let today = day.floor() as i64;
        push_capped(
            &mut self.habits.sleep_times,
            (day.fract() * 24_000.0) as u32,
        );
        if self.habits.night_counted != Some(today) {
            self.habits.nights_slept += 1;
            self.habits.night_counted = Some(today);
        }
    }

    /// They used a door: counted when it is one of the home's (either half of it).
    pub(crate) fn habit_door(&mut self, pos: BlockPos) {
        let Some(door) = self.home.as_ref().and_then(|h| {
            h.doors
                .iter()
                .find(|d| d[0] == pos[0] && d[2] == pos[2] && (d[1] - pos[1]).abs() <= 1)
                .copied()
        }) else {
            return;
        };
        match self.habits.doors.iter_mut().find(|(d, _)| *d == door) {
            Some((_, uses)) => *uses += 1,
            None => {
                self.habits.doors.push((door, 1));
                if self.habits.doors.len() > MOST_DOORS {
                    self.habits.doors.remove(0);
                }
            }
        }
    }

    /// The door they come and go by: the home's door they used most, once they have used one
    /// three times; before that, the home's first door. The door events knock and open there.
    pub fn usual_door(&self) -> Option<BlockPos> {
        let home = self.home.as_ref()?;
        self.habits
            .doors
            .iter()
            .filter(|(d, uses)| *uses >= 3 && home.doors.contains(d))
            .max_by_key(|(_, uses)| *uses)
            .map(|(d, _)| *d)
            .or_else(|| home.doors.first().copied())
    }

    /// Where each part of the day mostly goes, by place index, for the parts with enough
    /// samples.
    pub fn day_parts(&self) -> Vec<(usize, usize)> {
        self.habits
            .hours
            .iter()
            .enumerate()
            .filter(|(_, row)| row.iter().sum::<u32>() >= LEAST_QUARTER_SAMPLES)
            .map(|(quarter, row)| {
                let place = (0..PLACES.len()).max_by_key(|p| row[*p]).unwrap_or(0);
                (quarter, place)
            })
            .collect()
    }

    /// The habit lines of the situation text: each only once there is enough to say it.
    pub fn habit_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let habits = &self.habits;

        // The door, when there are two or more to choose from and one is the habit.
        if let (Some(home), Some(door)) = (&self.home, self.usual_door()) {
            let total: u32 = habits.doors.iter().map(|(_, u)| u).sum();
            let uses = habits
                .doors
                .iter()
                .find(|(d, _)| *d == door)
                .map(|(_, u)| *u)
                .unwrap_or(0);
            if home.doors.len() >= 2 && uses >= 3 && uses * 5 >= total * 3 {
                lines.push(format!(
                    "They come and go by the door on the {} side.",
                    side_of_house(home.center, door)
                ));
            }
        }

        // The day: parts that go to the same place said together.
        let parts = self.day_parts();
        if !parts.is_empty() {
            let mut groups: Vec<(Vec<usize>, usize)> = Vec::new();
            for (quarter, place) in parts {
                match groups.iter_mut().find(|(_, p)| *p == place) {
                    Some((quarters, _)) => quarters.push(quarter),
                    None => groups.push((vec![quarter], place)),
                }
            }
            let said: Vec<String> = groups
                .iter()
                .map(|(quarters, place)| {
                    let names: Vec<&str> = quarters.iter().map(|q| QUARTERS[*q]).collect();
                    format!("{} {}", and_list(&names), place_phrase(*place))
                })
                .collect();
            let mut line = said.join("; ");
            if let Some(first) = line.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            lines.push(format!("{line}, mostly."));
        }

        match (usual(&habits.leaves), usual(&habits.returns)) {
            (Some(leave), Some(back)) => lines.push(format!(
                "They usually leave home {} and come back {}.",
                time_words(leave),
                time_words(back)
            )),
            (Some(leave), None) => {
                lines.push(format!("They usually leave home {}.", time_words(leave)))
            }
            _ => {}
        }

        let nights = habits.nights_slept + habits.nights_up;
        if nights >= 2 {
            if habits.nights_slept * 3 >= nights * 2 {
                let soon =
                    usual(&habits.sleep_times).is_some_and(|t| (12_000..13_500).contains(&t));
                lines.push(format!(
                    "They sleep most nights{}.",
                    if soon { ", as soon as they can" } else { "" }
                ));
            } else if habits.nights_up * 3 >= nights * 2 {
                lines.push("They stay up most nights.".to_string());
            } else {
                lines.push("They sleep some nights and stay up through others.".to_string());
            }
        }

        if habits.days_seen >= 2 {
            let always: Vec<&(String, u32)> = habits
                .carried
                .iter()
                .filter(|(_, days)| days * 4 >= habits.days_seen * 3)
                .collect();
            let worn: Vec<String> = always
                .iter()
                .filter(|(id, _)| is_armour(id))
                .map(|(id, _)| crate::memory::plain(id))
                .take(2)
                .collect();
            let held: Vec<String> = always
                .iter()
                .filter(|(id, _)| !is_armour(id))
                .map(|(id, _)| crate::memory::plain(id))
                .take(3)
                .collect();
            if !held.is_empty() {
                lines.push(format!("Every day they carry {}.", held.join(" and ")));
            }
            if !worn.is_empty() {
                lines.push(format!("Every day they wear {}.", worn.join(" and ")));
            }
        }

        if habits.days_seen >= 1 && habits.lights_placed >= 3 {
            lines.push(format!(
                "They place about {} lights a day.",
                (habits.lights_placed as f32 / habits.days_seen as f32).round() as u32
            ));
        } else if habits.days_seen >= 2 && habits.lights_placed == 0 {
            lines.push("They never place a light.".to_string());
        }

        if habits.samples >= 30 {
            if habits.uneasy * 5 >= habits.samples {
                lines.push("They often look behind them.".to_string());
            } else if habits.uneasy * 30 < habits.samples {
                lines.push("They seldom look behind them.".to_string());
            }
        }

        lines
    }
}

/// `a`, `a and b`, `a, b and c`.
pub fn and_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// Distance on the ground, height left out: a house on a hill is still near.
fn flat_distance(a: BlockPos, b: BlockPos) -> f64 {
    distance([a[0], 0, a[2]], [b[0], 0, b[2]])
}

/// Which side of the house a block is on: `north`, `south`, `east` or `west`.
pub fn side_of_house(center: BlockPos, pos: BlockPos) -> &'static str {
    let dx = pos[0] - center[0];
    let dz = pos[2] - center[2];
    if dx.abs() > dz.abs() {
        if dx > 0 {
            "east"
        } else {
            "west"
        }
    } else if dz > 0 {
        "south"
    } else {
        "north"
    }
}

fn push_capped(list: &mut Vec<u32>, value: u32) {
    list.push(value);
    if list.len() > MOST_TIMES {
        list.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Home;
    use craftkit::world::{ItemStack, Movement};

    fn player_at(pos: [f64; 3]) -> Player {
        Player {
            pos,
            sky_light: 15,
            main_hand: ItemStack {
                id: "minecraft:iron_sword".to_string(),
                count: 1,
                slot: 0,
            },
            ..Player::default()
        }
    }

    fn with_home() -> Memory {
        Memory {
            home: Some(Home {
                center: [0, 64, 0],
                floor_y: 64,
                doors: vec![[0, 64, -4], [4, 64, 0]],
                ..Home::default()
            }),
            ..Memory::default()
        }
    }

    /// Samples from `from` to `to` (days), one every hundred ticks, standing at `pos`.
    fn spend(memory: &mut Memory, from: f32, to: f32, pos: [f64; 3]) {
        let mut day = from;
        while day < to {
            memory.sample_habits(&player_at(pos), day);
            day += SAMPLE_TICKS / 24_000.0;
        }
    }

    #[test]
    fn a_sample_counts_once_however_often_it_is_called() {
        let mut memory = with_home();
        for _ in 0..50 {
            memory.sample_habits(&player_at([0.5, 64.0, 0.5]), 0.1);
        }

        assert_eq!(memory.habits.samples, 1);
        assert_eq!(memory.habits.hours[0][0], 1, "a morning at home");
    }

    #[test]
    fn the_day_reads_as_where_each_part_goes() {
        let mut memory = with_home();
        memory.last.underground = true;
        spend(&mut memory, 0.0, 0.25, [200.5, 20.0, 0.5]);
        memory.last.underground = false;
        spend(&mut memory, 0.25, 0.5, [300.5, 70.0, 0.5]);
        spend(&mut memory, 0.5, 1.0, [1.5, 64.0, 1.5]);

        let lines = memory.habit_lines();
        assert!(
            lines.contains(&"Mornings underground; afternoons away from home; evenings and nights at home, mostly.".to_string()),
            "{lines:?}"
        );
        assert!(memory.day_parts().contains(&(0, 1)));
    }

    #[test]
    fn leaving_and_coming_back_are_told_by_their_usual_time() {
        let mut memory = with_home();
        for day in 0..3 {
            let d = day as f32;
            spend(&mut memory, d, d + 0.2, [0.5, 64.0, 0.5]);
            spend(&mut memory, d + 0.2, d + 0.4, [100.5, 64.0, 0.5]);
            spend(&mut memory, d + 0.4, d + 1.0, [0.5, 64.0, 0.5]);
        }

        assert_eq!(memory.habits.leaves.len(), 3);
        assert_eq!(memory.habits.returns.len(), 3);
        let lines = memory.habit_lines();
        assert!(
            lines.contains(
                &"They usually leave home in the morning and come back in the afternoon."
                    .to_string()
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn walking_the_edge_of_home_is_not_leaving() {
        let mut memory = with_home();
        spend(&mut memory, 0.0, 0.1, [0.5, 64.0, 0.5]);
        // Between 16 and 32 blocks out: neither in nor out.
        spend(&mut memory, 0.1, 0.2, [25.5, 64.0, 0.5]);
        spend(&mut memory, 0.2, 0.3, [0.5, 64.0, 0.5]);

        assert!(memory.habits.leaves.is_empty());
        assert!(memory.habits.returns.is_empty());
    }

    #[test]
    fn the_door_they_use_most_is_the_usual_one_and_said_by_its_side() {
        let mut memory = with_home();
        assert_eq!(
            memory.usual_door(),
            Some([0, 64, -4]),
            "the first before any use"
        );
        for _ in 0..4 {
            memory.habit_door([4, 65, 0]); // the upper half counts for the door
        }
        memory.habit_door([0, 64, -4]);
        memory.habit_door([50, 64, 50]); // not the home's

        assert_eq!(memory.usual_door(), Some([4, 64, 0]));
        assert!(memory
            .habit_lines()
            .contains(&"They come and go by the door on the east side.".to_string()));
    }

    #[test]
    fn nights_slept_and_stayed_up_are_counted_once_each() {
        let mut memory = with_home();
        memory.habit_slept(0.52);
        memory.habit_slept(0.53);
        memory.habit_slept(1.52);
        spend(&mut memory, 2.0, 3.0, [0.5, 64.0, 0.5]);

        assert_eq!(memory.habits.nights_slept, 2);
        assert_eq!(memory.habits.nights_up, 1);
        let lines = memory.habit_lines();
        assert!(
            lines.contains(&"They sleep most nights, as soon as they can.".to_string()),
            "{lines:?}"
        );
    }

    #[test]
    fn what_is_held_every_day_is_carried_and_armour_is_worn() {
        let mut memory = with_home();
        let mut player = player_at([0.5, 64.0, 0.5]);
        player.armor = vec![ItemStack {
            id: "minecraft:iron_helmet".to_string(),
            count: 1,
            slot: 0,
        }];
        for day in 0..4 {
            memory.sample_habits(&player, day as f32 + 0.1);
            memory.sample_habits(&player, day as f32 + 0.2);
        }

        assert_eq!(memory.habits.days_seen, 4);
        assert!(memory
            .habits
            .carried
            .contains(&("minecraft:iron_sword".to_string(), 4)));
        let lines = memory.habit_lines();
        assert!(
            lines.contains(&"Every day they carry an iron sword.".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"Every day they wear an iron helmet.".to_string()),
            "{lines:?}"
        );
    }

    #[test]
    fn looking_behind_them_often_is_said() {
        let mut memory = with_home();
        let mut player = player_at([0.5, 64.0, 0.5]);
        player.movement = Movement {
            look_backs: 3,
            ..Movement::default()
        };
        let mut day = 0.0;
        for _ in 0..40 {
            memory.sample_habits(&player, day);
            day += SAMPLE_TICKS / 24_000.0;
        }

        assert!(memory
            .habit_lines()
            .contains(&"They often look behind them.".to_string()));
    }

    #[test]
    fn three_parts_read_as_a_list() {
        assert_eq!(
            and_list(&["mornings", "evenings", "nights"]),
            "mornings, evenings and nights"
        );
        assert_eq!(and_list(&["nights"]), "nights");
    }

    #[test]
    fn time_words_cover_the_day() {
        assert_eq!(time_words(0), "at dawn");
        assert_eq!(time_words(3_000), "in the morning");
        assert_eq!(time_words(6_000), "around noon");
        assert_eq!(time_words(12_500), "at dusk");
        assert_eq!(time_words(18_000), "at night");
        assert_eq!(time_words(23_000), "at dawn");
    }
}
