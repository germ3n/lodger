//! Places (the review's N3, owner): the house in rooms, named by what stands in
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
                // The least of the others makes room, never the new one (until now the
                // new one went at once and its sample landed on another site).
                if self.sites.len() >= MOST_SITES {
                    let least = self
                        .sites
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, s)| (s.samples, s.last_day))
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.sites.remove(least);
                }
                self.sites.push(Site {
                    kind: "spot".to_string(),
                    pos: at,
                    samples: 0,
                    days: 0,
                    last_day: -1,
                });
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

    /// The sites they keep, most stayed at first: away from the home as it is now, never one
    /// the home has come to since (the owner saw "a second base 9 blocks west of the house",
    /// a base from before the bed was moved to it).
    pub fn kept_sites(&self) -> impl Iterator<Item = &Site> {
        let center = self.home.as_ref().map(|h| h.center);
        self.sites.iter().filter(move |s| {
            s.samples >= SITE_SAMPLES && center.is_none_or(|c| flat(c, s.pos) > SITE_FROM_HOME)
        })
    }

    /// The home moved: the sites it is at now are the home, not places away from it.
    pub(crate) fn forget_sites_at_home(&mut self) {
        if let Some(center) = self.home.as_ref().map(|h| h.center) {
            self.sites.retain(|s| flat(center, s.pos) > SITE_FROM_HOME);
        }
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

    /// Where they last died, when no `death_light` of its own stands within three blocks of it.
    pub fn unmarked_death(&self) -> Option<BlockPos> {
        let died = self
            .milestones
            .iter()
            .rev()
            .find(|m| m.kind == "died")?
            .pos?;
        let marked = self
            .landmarks
            .iter()
            .any(|l| l.event == "death_light" && l.pos.is_some_and(|p| distance(p, died) <= 3.0));

        (!marked).then_some(died)
    }

    /// Where `light_taken` carries their last light: six to ten blocks on from it along the
    /// way they went, the trail they are walking or the route, never back toward the house,
    /// and not in view.
    pub fn light_taken_to(&self) -> Option<BlockPos> {
        let (from, _) = self.habits.last_light.as_ref()?;
        let center = self.home.as_ref().map(|h| h.center);
        let ways = std::iter::once(&self.trail).chain(self.route().map(|r| &r.points));
        for way in ways {
            let Some(near) = (0..way.len())
                .min_by(|a, b| distance(way[*a], *from).total_cmp(&distance(way[*b], *from)))
            else {
                continue;
            };
            if distance(way[near], *from) > 6.0 {
                continue;
            }
            // On is away from the house: toward the way's end farther from it.
            let forward = match center {
                Some(c) => distance(way[way.len() - 1], c) >= distance(way[0], c),
                None => true,
            };
            let ahead: Vec<BlockPos> = if forward {
                way[near..].to_vec()
            } else {
                way[..=near].iter().rev().copied().collect()
            };
            if let Some(to) = ahead
                .into_iter()
                .find(|p| (6.0..=10.0).contains(&distance(*p, *from)))
            {
                if !self.in_view(to, 16.0) {
                    return Some(to);
                }
            }
        }

        None
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

    #[test]
    fn a_thirteenth_site_is_kept_and_the_least_of_the_others_makes_room() {
        let mut memory = memory();
        for i in 0..MOST_SITES as i32 {
            for _ in 0..(i + 2) {
                memory.sample_site([100 + i * 60, 64, 100], 1.0);
            }
        }
        assert_eq!(memory.sites.len(), MOST_SITES);
        memory.placed = vec![[-500, 64, 0], [-501, 64, 0], [-500, 65, 0]];
        memory.sample_site([-500, 64, 0], 2.0);
        assert_eq!(memory.sites.len(), MOST_SITES);
        assert!(
            memory
                .sites
                .iter()
                .any(|s| s.pos == [-500, 64, 0] && s.samples == 1),
            "the new one kept, with its sample: {:?}",
            memory.sites
        );
        assert!(
            !memory.sites.iter().any(|s| s.pos == [100, 64, 100]),
            "the least stayed at made room"
        );
        assert!(
            memory.sites.iter().all(|s| s.kind != "base"),
            "no other site took its kind (one sample is not yet a base): {:?}",
            memory.sites
        );
    }

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
    fn a_base_the_home_moves_to_is_the_home_and_no_longer_away_from_it() {
        let mut memory = memory();
        for x in 0..BASE_BLOCKS as i32 {
            memory.placed.push([-300 + x % 8, 64, x / 8]);
        }
        for _ in 0..SITE_SAMPLES {
            memory.sample_site([-298, 64, 2], 1.5);
        }
        assert!(memory.site("base").is_some());

        // The bed moved to a house nine blocks from it: no "second base 9 blocks west".
        memory.home.as_mut().unwrap().center = [-289, 64, 2];
        assert!(
            memory.place_lines().is_empty(),
            "{:?}",
            memory.place_lines()
        );
        assert!(memory.site("base").is_none());
        assert_ne!(memory.where_words([-298, 64, 2]), "at their other base");
        // And it is let go as the home moves.
        memory.forget_sites_at_home();
        assert!(memory.sites.is_empty(), "{:?}", memory.sites);
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

    #[test]
    fn a_death_is_marked_once_and_a_newer_one_again() {
        let mut memory = memory();
        assert_eq!(memory.unmarked_death(), None);
        let died = |pos: BlockPos, day: f32| crate::memory::Milestone {
            kind: "died".to_string(),
            day,
            pos: Some(pos),
            detail: String::new(),
        };
        memory.milestones.push(died([40, 60, 40], 1.0));
        assert_eq!(memory.unmarked_death(), Some([40, 60, 40]));

        memory.record("death_light", Some([40, 61, 40]), 1.1, true, "");
        assert_eq!(memory.unmarked_death(), None, "marked");
        memory.milestones.push(died([-80, 30, 5], 2.0));
        assert_eq!(
            memory.unmarked_death(),
            Some([-80, 30, 5]),
            "the newer death"
        );
    }

    #[test]
    fn their_last_light_goes_on_the_way_they_went_and_never_back_toward_the_house() {
        let mut memory = memory();
        // A walk out from the house along x, a point every two blocks.
        memory.trail = (0..30).map(|i| [4 + i * 2, 64, 2]).collect();
        memory.habits.last_light = Some(([20, 65, 3], "minecraft:wall_torch".to_string()));

        let to = memory.light_taken_to().unwrap();
        assert!(to[0] > 20, "on, away from the house: {to:?}");
        let d = distance(to, [20, 65, 3]);
        assert!((6.0..=10.0).contains(&d), "{d}");

        // Walked the other way, toward the house: still away from it.
        memory.trail.reverse();
        assert!(memory.light_taken_to().unwrap()[0] > 20);

        memory.habits.last_light = Some(([20, 65, 40], "minecraft:torch".to_string()));
        assert_eq!(memory.light_taken_to(), None, "no way on near it");
    }

    #[test]
    fn placing_a_light_makes_it_the_last_and_breaking_what_holds_it_forgets_it() {
        use craftkit::world::{Event, EventKind, Observations, Player};
        let mut memory = memory();
        memory.target = Some(0);
        let seen = |events: Vec<Event>| Observations {
            players: vec![Player::default()],
            events,
            ..Observations::default()
        };
        let event = |kind: EventKind, pos: BlockPos, detail: &str| Event {
            player: 0,
            kind,
            pos: Some(pos),
            detail: detail.to_string(),
            ..Event::default()
        };
        memory.observe(
            &seen(vec![event(
                EventKind::PlacedBlock,
                [9, 65, 9],
                "minecraft:torch",
            )]),
            0.1,
        );
        assert_eq!(
            memory.habits.last_light,
            Some(([9, 65, 9], "minecraft:torch".to_string()))
        );
        memory.observe(
            &seen(vec![event(
                EventKind::BrokeBlock,
                [9, 64, 9],
                "minecraft:dirt",
            )]),
            0.2,
        );
        assert_eq!(
            memory.habits.last_light, None,
            "the torch dropped with the ground under it"
        );
    }

    #[test]
    fn a_light_is_carried_as_what_it_is() {
        assert_eq!(
            crate::memory::light_kinds("minecraft:wall_torch"),
            (
                "minecraft:torch".to_string(),
                "minecraft:wall_torch".to_string()
            )
        );
        assert_eq!(
            crate::memory::light_kinds("minecraft:soul_torch"),
            (
                "minecraft:soul_torch".to_string(),
                "minecraft:soul_wall_torch".to_string()
            )
        );
        assert_eq!(
            crate::memory::light_kinds("minecraft:lantern"),
            ("minecraft:lantern".to_string(), String::new())
        );
    }
}
