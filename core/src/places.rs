//! Places (the review's N3, owner 2026-11-09): the house in rooms, named by what stands in
//! them ("the room with the bed"), upstairs and below; the target's sites away from home (a
//! mine, a second base, a spot they keep going back to); and the lodger's own landmarks
//! referred back to when it acts near one again. Rooms are read from what the memory already
//! has (the bed, the furnaces, the work blocks, the chests), not from walls: the home read
//! finds blocks by kind, never air, so a room is "near the bed and on its floor".

use craftkit::world::BlockPos;
use serde::{Deserialize, Serialize};

use crate::memory::{distance, Done, Memory};

/// Farther than this from the home's center, a place they stay is a site of its own.
pub const SITE_FROM_HOME: f64 = 48.0;
/// Samples at one place within this many blocks are the same site.
const SITE_BLOCKS: f64 = 24.0;
/// Samples before a site is said: two and a half minutes there.
const SITE_SAMPLES: u32 = 30;
/// Blocks they placed within 12 of a site that make it a base.
const BASE_BLOCKS: usize = 40;
const MOST_SITES: usize = 12; // the least-stayed-at dropped first
/// A room reaches this far from what names it, on its floor.
const ROOM_BLOCKS: f64 = 5.0;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Site {
    /// `mine` (underground), `base` (they built there), or `spot`.
    pub kind: String,
    /// Where they first stayed, the site's middle.
    pub pos: BlockPos,
    /// Samples spent there (one every hundred ticks).
    pub samples: u32,
    /// Days they were there, and the last.
    pub days: u32,
    pub last_day: i64,
}

impl Memory {
    /// A sample away from home: the site they are at counts it, or a new one starts.
    pub(crate) fn sample_site(&mut self, at: BlockPos, day: f32) {
        let Some(center) = self.home.as_ref().map(|h| h.center) else {
            return;
        };
        if flat(center, at) <= SITE_FROM_HOME {
            return;
        }
        let today = day.floor() as i64;
        let underground = self.last.underground;
        let built = self
            .placed
            .iter()
            .filter(|p| distance(**p, at) <= 12.0)
            .count();
        let index = match self
            .sites
            .iter()
            .position(|s| flat(s.pos, at) <= SITE_BLOCKS && (s.pos[1] - at[1]).abs() <= 24)
        {
            Some(index) => index,
            None => {
                self.sites.push(Site {
                    kind: "spot".to_string(),
                    pos: at,
                    samples: 0,
                    days: 0,
                    last_day: -1,
                });
                if self.sites.len() > MOST_SITES {
                    let least = self
                        .sites
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, s)| (s.samples, s.last_day))
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.sites.remove(least);
                }
                self.sites.len() - 1
            }
        };
        let site = &mut self.sites[index];
        site.samples += 1;
        if site.last_day != today {
            site.days += 1;
            site.last_day = today;
        }
        // A base outranks a mine, a mine a spot: what they built there says the most.
        if built >= BASE_BLOCKS {
            site.kind = "base".to_string();
        } else if underground && site.kind == "spot" {
            site.kind = "mine".to_string();
        }
        self.sites.sort_by_key(|s| std::cmp::Reverse(s.samples));
    }

    /// The sites they keep, most stayed at first.
    pub fn kept_sites(&self) -> impl Iterator<Item = &Site> {
        self.sites.iter().filter(|s| s.samples >= SITE_SAMPLES)
    }

    /// The site of a kind they keep most, if any: `mine`, `base`, `spot`.
    pub fn site(&self, kind: &str) -> Option<&Site> {
        self.kept_sites().find(|s| s.kind == kind)
    }

    /// The room a block of the house is in, by what stands nearest on its floor: the bed, a
    /// furnace, a crafting table, a lectern, a jukebox, or three chests or more together (the
    /// storeroom). None outside the house or where nothing names the room.
    pub fn room_of(&self, pos: BlockPos) -> Option<&'static str> {
        let home = self.home.as_ref()?;
        if distance(home.center, pos) > 12.0 {
            return None;
        }
        let mut anchors: Vec<(BlockPos, &'static str)> = Vec::new();
        if let Some(bed) = home.bed {
            anchors.push((bed, "the room with the bed"));
        }
        for container in self.containers.iter().filter(|c| c.in_home || c.placed) {
            if container.is_furnace() {
                anchors.push((container.pos, "the room with the furnace"));
            }
        }
        for (kind, words) in [
            ("crafting_table", "the room with the crafting table"),
            ("lectern", "the room with the lectern"),
            ("jukebox", "the room with the jukebox"),
        ] {
            for fixture in self.fixtures(kind) {
                anchors.push((fixture.pos, words));
            }
        }
        let chests: Vec<BlockPos> = self
            .containers
            .iter()
            .filter(|c| !c.is_furnace() && (c.in_home || c.placed))
            .map(|c| c.pos)
            .collect();
        for chest in &chests {
            if chests
                .iter()
                .filter(|o| distance(**o, *chest) <= 3.0)
                .count()
                >= 3
            {
                anchors.push((*chest, "the storeroom"));
            }
        }

        anchors
            .into_iter()
            .filter(|(a, _)| (a[1] - pos[1]).abs() <= 2 && flat(*a, pos) <= ROOM_BLOCKS)
            .min_by(|a, b| flat(a.0, pos).total_cmp(&flat(b.0, pos)))
            .map(|(_, words)| words)
    }

    /// Where a block is, in the situation's words: the room of the house, upstairs or below
    /// it, or how far and which way from the house.
    pub fn where_words(&self, pos: BlockPos) -> String {
        let Some(home) = &self.home else {
            return "somewhere".to_string();
        };
        let d = distance(home.center, pos);
        if let Some(room) = self.room_of(pos) {
            return format!("in {room}");
        }
        if d <= 8.0 {
            return if pos[1] >= home.floor_y + 4 {
                "upstairs in the house".to_string()
            } else if pos[1] <= home.floor_y - 3 {
                "under the house".to_string()
            } else {
                "in the house".to_string()
            };
        }
        if let Some(site) = self
            .kept_sites()
            .find(|s| distance(s.pos, pos) <= SITE_BLOCKS)
        {
            return format!("at their {}", site_name(&site.kind));
        }

        format!(
            "{} blocks {} of the house",
            flat(home.center, pos).round() as i32,
            crate::habits::side_of_house(home.center, pos)
        )
    }

    /// An earlier landmark of its own near a place, another event than `event`: what it did
    /// there before, to refer back to.
    pub fn landmark_near(&self, pos: BlockPos, before_day: f32, event: &str) -> Option<&Done> {
        self.landmarks.iter().rev().find(|l| {
            l.day < before_day && l.event != event && l.pos.is_some_and(|p| distance(p, pos) <= 6.0)
        })
    }

    /// The places line: the sites they keep away from home.
    pub fn place_lines(&self) -> Vec<String> {
        let said: Vec<String> = self
            .kept_sites()
            .take(2)
            .map(|s| format!("{} {}", site_article(&s.kind), self.where_far(s.pos)))
            .collect();
        if said.is_empty() {
            return Vec::new();
        }
        let said: Vec<&str> = said.iter().map(String::as_str).collect();

        vec![format!(
            "Away from home they keep {}.",
            crate::habits::and_list(&said)
        )]
    }

    fn where_far(&self, pos: BlockPos) -> String {
        match &self.home {
            Some(home) => format!(
                "{} blocks {} of the house",
                flat(home.center, pos).round() as i32,
                crate::habits::side_of_house(home.center, pos)
            ),
            None => "somewhere".to_string(),
        }
    }
}

fn site_name(kind: &str) -> &'static str {
    match kind {
        "mine" => "mine",
        "base" => "other base",
        _ => "spot away from home",
    }
}

fn site_article(kind: &str) -> &'static str {
    match kind {
        "mine" => "a mine",
        "base" => "a second base",
        _ => "a spot",
    }
}

fn flat(a: BlockPos, b: BlockPos) -> f64 {
    distance([a[0], 0, a[2]], [b[0], 0, b[2]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Container, Fixture, Home};

    fn memory() -> Memory {
        Memory {
            home: Some(Home {
                bed: Some([0, 64, 0]),
                center: [2, 64, 2],
                floor_y: 64,
                ..Home::default()
            }),
            ..Memory::default()
        }
    }

    fn container(pos: BlockPos, kind: &str) -> Container {
        Container {
            pos,
            kind: kind.to_string(),
            in_home: true,
            ..Container::default()
        }
    }

    #[test]
    fn a_room_is_named_by_what_stands_nearest_on_its_floor() {
        let mut memory = memory();
        memory
            .containers
            .push(container([8, 64, 2], "minecraft:furnace"));
        memory.fixtures.push(Fixture {
            kind: "crafting_table".to_string(),
            pos: [8, 64, 5],
        });
        for z in 0..3 {
            memory
                .containers
                .push(container([-4, 64, 6 + z], "minecraft:chest"));
        }

        assert_eq!(memory.room_of([1, 64, 1]), Some("the room with the bed"));
        assert_eq!(
            memory.room_of([9, 64, 2]),
            Some("the room with the furnace")
        );
        assert_eq!(
            memory.room_of([8, 64, 6]),
            Some("the room with the crafting table")
        );
        assert_eq!(memory.room_of([-3, 64, 7]), Some("the storeroom"));
        assert_eq!(memory.room_of([0, 69, 0]), None, "another floor");
        assert_eq!(memory.where_words([1, 64, 1]), "in the room with the bed");
        assert_eq!(memory.where_words([3, 69, 3]), "upstairs in the house");
        assert_eq!(memory.where_words([3, 60, 3]), "under the house");
        assert_eq!(
            memory.where_words([2, 64, 40]),
            "38 blocks south of the house"
        );
    }

    #[test]
    fn two_chests_are_no_storeroom() {
        let mut memory = memory();
        memory.home.as_mut().unwrap().bed = None;
        for z in 0..2 {
            memory
                .containers
                .push(container([-4, 64, 6 + z], "minecraft:chest"));
        }

        assert_eq!(memory.room_of([-4, 64, 6]), None);
    }

    #[test]
    fn a_place_they_stay_away_from_home_becomes_a_site_once_it_is_kept() {
        let mut memory = memory();
        memory.last.underground = true;
        for i in 0..SITE_SAMPLES {
            memory.sample_site([200 + (i % 5) as i32, 20, 0], 0.1);
        }
        memory.last.underground = false;
        memory.sample_site([10, 64, 10], 0.2); // home: no site

        assert_eq!(memory.sites.len(), 1);
        let mine = memory.site("mine").unwrap();
        assert_eq!(mine.samples, SITE_SAMPLES);
        assert_eq!(mine.days, 1);
        assert_eq!(
            memory.place_lines(),
            vec!["Away from home they keep a mine 198 blocks east of the house.".to_string()]
        );
        assert_eq!(memory.where_words([205, 20, 3]), "at their mine");
    }

    #[test]
    fn building_at_a_site_makes_it_a_base() {
        let mut memory = memory();
        for x in 0..BASE_BLOCKS as i32 {
            memory.placed.push([-300 + x % 8, 64, x / 8]);
        }
        for _ in 0..SITE_SAMPLES {
            memory.sample_site([-298, 64, 2], 1.5);
        }

        assert!(memory.site("base").is_some());
        assert!(memory.place_lines()[0].contains("a second base 300 blocks west"));
    }

    #[test]
    fn a_short_stop_is_no_site_to_speak_of() {
        let mut memory = memory();
        for _ in 0..5 {
            memory.sample_site([500, 64, 0], 0.1);
        }

        assert!(memory.place_lines().is_empty());
        assert_eq!(memory.kept_sites().count(), 0);
    }

    #[test]
    fn an_earlier_landmark_of_another_event_is_found_near_a_place() {
        let mut memory = memory();
        memory.landmarks.push(Done {
            event: "sign".to_string(),
            pos: Some([5, 64, 5]),
            day: 1.0,
            ok: true,
            reason: String::new(),
        });

        assert!(memory.landmark_near([6, 64, 6], 2.0, "cairn").is_some());
        assert!(
            memory.landmark_near([6, 64, 6], 2.0, "sign").is_none(),
            "the same event"
        );
        assert!(
            memory.landmark_near([6, 64, 6], 0.5, "cairn").is_none(),
            "not before"
        );
        assert!(
            memory.landmark_near([30, 64, 6], 2.0, "cairn").is_none(),
            "too far"
        );
    }
}
