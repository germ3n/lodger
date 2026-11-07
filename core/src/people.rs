//! The people around the target (the review's N4, owner): who comes to the home,
//! counted once a day each, and who is with the target now. Read from the players every
//! observation carries; said as detail lines of the situation text.

use craftkit::world::Observations;
use serde::{Deserialize, Serialize};

use crate::habits::and_list;
use crate::memory::{Memory, HOME_RADIUS};

/// Within this many blocks of the target, another player is with them.
pub const WITH_BLOCKS: f64 = 24.0;
const MOST_VISITORS: usize = 16; // the least-seen dropped first

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Visitor {
    pub name: String,
    /// Days they were seen at the home.
    pub days: u32,
    /// The stay day they were last seen there.
    pub last_day: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct People {
    /// Other players seen at the home, most days first.
    pub visitors: Vec<Visitor>,
    /// Other players with the target as of the last observation, by name.
    pub with: Vec<String>,
}

impl Memory {
    /// Who is near the target and who is at the home, from one observation's players.
    pub(crate) fn observe_people(&mut self, observations: &Observations, day: f32) {
        let Some(index) = self.target else { return };
        let Some(target) = observations.players.iter().find(|p| p.index == index) else {
            self.people.with.clear();
            return;
        };
        let today = day.floor() as i64;
        let center = self.home.as_ref().map(|h| h.center);
        self.people.with.clear();
        for other in observations.players.iter().filter(|p| p.index != index) {
            let dx = other.pos[0] - target.pos[0];
            let dy = other.pos[1] - target.pos[1];
            let dz = other.pos[2] - target.pos[2];
            if (dx * dx + dy * dy + dz * dz).sqrt() <= WITH_BLOCKS {
                self.people.with.push(other.name.clone());
            }
            let at_home = center.is_some_and(|c| {
                let dx = other.pos[0] - c[0] as f64;
                let dz = other.pos[2] - c[2] as f64;
                (dx * dx + dz * dz).sqrt() <= (HOME_RADIUS + 8) as f64
            });
            if !at_home {
                continue;
            }
            match self
                .people
                .visitors
                .iter_mut()
                .find(|v| v.name == other.name)
            {
                Some(visitor) if visitor.last_day != today => {
                    visitor.days += 1;
                    visitor.last_day = today;
                }
                Some(_) => {}
                None => {
                    self.people.visitors.push(Visitor {
                        name: other.name.clone(),
                        days: 1,
                        last_day: today,
                    });
                    if self.people.visitors.len() > MOST_VISITORS {
                        let least = self
                            .people
                            .visitors
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, v)| (v.days, v.last_day))
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        self.people.visitors.remove(least);
                    }
                }
            }
            self.people
                .visitors
                .sort_by_key(|v| std::cmp::Reverse(v.days));
        }
    }

    /// Whether no other player is with the target now.
    pub fn alone(&self) -> bool {
        self.people.with.is_empty()
    }

    /// The people lines: who is with them now (once other players have been seen at all), and
    /// who comes to the house.
    pub fn people_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.people.with.is_empty() {
            let names: Vec<&str> = self.people.with.iter().map(String::as_str).collect();
            lines.push(format!("Now they are with {}.", and_list(&names)));
        } else if self.others.len() >= 2 && self.last.online {
            lines.push("Now they are alone.".to_string());
        }
        if !self.people.visitors.is_empty() {
            let said: Vec<String> = self
                .people
                .visitors
                .iter()
                .take(3)
                .map(|v| match v.days {
                    1 => format!("{} once", v.name),
                    n => format!("{} on {n} days", v.name),
                })
                .collect();
            let said: Vec<&str> = said.iter().map(String::as_str).collect();
            lines.push(format!("Others come to the house: {}.", and_list(&said)));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Home;
    use craftkit::world::Player;

    fn at(index: u32, name: &str, pos: [f64; 3]) -> Player {
        Player {
            index,
            name: name.to_string(),
            pos,
            sky_light: 15,
            ..Player::default()
        }
    }

    fn seen(players: Vec<Player>) -> Observations {
        Observations {
            players,
            ..Observations::default()
        }
    }

    fn memory() -> Memory {
        Memory {
            target: Some(0),
            home: Some(Home {
                center: [0, 64, 0],
                floor_y: 64,
                ..Home::default()
            }),
            ..Memory::default()
        }
    }

    #[test]
    fn a_player_near_the_target_is_with_them_and_one_far_is_not() {
        let mut memory = memory();
        memory.observe_people(
            &seen(vec![
                at(0, "razor", [100.0, 64.0, 0.0]),
                at(1, "alex", [110.0, 64.0, 5.0]),
                at(2, "sam", [400.0, 64.0, 0.0]),
            ]),
            0.2,
        );

        assert_eq!(memory.people.with, vec!["alex".to_string()]);
        assert!(!memory.alone());
        assert!(memory
            .people_lines()
            .contains(&"Now they are with alex.".to_string()));
    }

    #[test]
    fn a_visitor_is_counted_once_a_day_at_the_home() {
        let mut memory = memory();
        for day in [0.2, 0.3, 1.2, 2.5] {
            memory.observe_people(
                &seen(vec![
                    at(0, "razor", [0.0, 64.0, 0.0]),
                    at(1, "alex", [5.0, 64.0, 5.0]),
                ]),
                day,
            );
        }
        memory.observe_people(
            &seen(vec![
                at(0, "razor", [0.0, 64.0, 0.0]),
                at(2, "sam", [10.0, 64.0, 0.0]),
            ]),
            2.6,
        );

        assert_eq!(memory.people.visitors[0].name, "alex");
        assert_eq!(memory.people.visitors[0].days, 3);
        assert!(memory
            .people_lines()
            .contains(&"Others come to the house: alex on 3 days and sam once.".to_string()));
    }

    #[test]
    fn alone_is_said_only_where_there_are_others_at_all() {
        let mut memory = memory();
        memory.last.online = true;
        memory.observe_people(&seen(vec![at(0, "razor", [0.0, 64.0, 0.0])]), 0.1);
        assert!(memory.people_lines().is_empty(), "a single-player world");

        memory.others = vec![
            (0, "razor".to_string(), String::new()),
            (1, "alex".to_string(), String::new()),
        ];
        assert_eq!(
            memory.people_lines(),
            vec!["Now they are alone.".to_string()]
        );
    }

    #[test]
    fn the_target_away_from_the_game_is_with_no_one() {
        let mut memory = memory();
        memory.people.with = vec!["alex".to_string()];
        memory.observe_people(&seen(vec![at(1, "alex", [0.0, 64.0, 0.0])]), 0.1);

        assert!(memory.alone());
    }
}
