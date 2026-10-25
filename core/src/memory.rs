//! What the lodger knows about its target, bounded, saved with the stay, and described as the
//! lines the brain reads. See `docs/memory.md`; this is the first cut, the pieces the first
//! events need. Positions are block positions; times are the stay's in-game day as a fraction.

use craftkit::world::{BlockPos, EventKind, ItemStack, Observations, Player};
use serde::{Deserialize, Serialize};

/// How far from the home's center the home reaches.
pub const HOME_RADIUS: i32 = 16;
/// Within this many blocks of the target, nothing happens while in view.
pub const CELL: i32 = 4;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Home {
    /// Where they sleep, if they have.
    pub bed: Option<BlockPos>,
    /// The middle of where they spend their time standing still.
    pub center: BlockPos,
    /// The floor's height, from the bed or the lowest block they placed.
    pub floor_y: i32,
    pub doors: Vec<BlockPos>,
    pub lights: Vec<BlockPos>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Container {
    pub pos: BlockPos,
    pub opens: u32,
    pub last_opened_day: f32,
    pub contents: Vec<ItemStack>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Held {
    pub item: ItemStack,
    pub from: BlockPos,
    pub slot: u32,
    pub taken_day: f32,
}

/// An event that landed, or did not.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Done {
    pub event: String,
    pub pos: Option<BlockPos>,
    pub day: f32,
    pub ok: bool,
    pub reason: String,
}

/// The target as last seen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub pos: [f64; 3],
    pub yaw: f32,
    pub online: bool,
    pub indoors: bool,
    pub sleeping: bool,
    pub walking: bool,
    pub main_hand: String,
    /// Ids of animals and villagers within 32 blocks, from the last entity pass.
    pub creatures_near: Vec<u32>,
    pub look_backs: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    pub target: Option<u32>,
    pub target_name: String,
    pub home: Option<Home>,
    pub containers: Vec<Container>,
    /// Item ids and how often they were in the target's main hand, most first.
    pub favourites: Vec<(String, u32)>,
    pub held: Option<Held>,
    pub done: Vec<Done>,
    pub last: Snapshot,
    /// Coarse cells the target stood still in, and for how many ticks: the home's center comes
    /// from the densest.
    pub still_cells: Vec<([i32; 3], u32)>,
    /// The last chat lines about the lodger, newest last.
    pub chat: Vec<String>,
    /// The lodger's place, where it leaves things, once chosen.
    pub place: Option<BlockPos>,
    /// Blocks the target placed that the lodger must never break: everything they placed.
    pub placed: Vec<BlockPos>,
}

/// Caps, with the reason beside each.
const MOST_CONTAINERS: usize = 32; // least-opened dropped first
const MOST_DONE: usize = 256; // oldest dropped first
const MOST_FAVOURITES: usize = 8;
const MOST_CELLS: usize = 512; // the least-stood-in dropped first
const MOST_CHAT: usize = 8;
const MOST_PLACED: usize = 2048; // a whole house; older ones fall off, which is the risk the docs name
const MOST_LIGHTS: usize = 64;
const MOST_DOORS: usize = 16;

fn block(pos: [f64; 3]) -> BlockPos {
    [
        pos[0].floor() as i32,
        pos[1].floor() as i32,
        pos[2].floor() as i32,
    ]
}

pub fn distance(a: BlockPos, b: BlockPos) -> f64 {
    let dx = (a[0] - b[0]) as f64;
    let dy = (a[1] - b[1]) as f64;
    let dz = (a[2] - b[2]) as f64;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn is_light(id: &str) -> bool {
    id.ends_with("torch") || id.ends_with("lantern") || id.contains("candle")
}

fn is_door(id: &str) -> bool {
    id.ends_with("_door") && !id.contains("trapdoor")
}

impl Memory {
    /// Takes in a tick's observations. The target is the first player seen until the brain picks
    /// another (multiplayer, a later phase).
    pub fn observe(&mut self, observations: &Observations, day: f32) {
        if self.target.is_none() {
            if let Some(first) = observations.players.first() {
                self.target = Some(first.index);
                self.target_name = first.name.clone();
            }
        }

        let Some(index) = self.target else { return };
        let player = observations.players.iter().find(|p| p.index == index);

        match player {
            Some(player) => self.observe_player(player),
            None => self.last.online = false,
        }

        if !observations.entities.is_empty() {
            let at = block(self.last.pos);
            self.last.creatures_near = observations
                .entities
                .iter()
                .filter(|e| {
                    !e.hostile && e.kind != "minecraft:item" && distance(block(e.pos), at) <= 32.0
                })
                .map(|e| e.id)
                .collect();
        }

        for event in &observations.events {
            if event.player != index {
                continue;
            }

            match event.kind {
                EventKind::OpenedContainer => {
                    if let Some(pos) = event.pos {
                        self.container_opened(pos, &event.contents, day);
                    }
                }
                EventKind::SleptBegan => {
                    if let Some(pos) = event.pos {
                        let home = self.home.get_or_insert_with(|| Home {
                            center: pos,
                            floor_y: pos[1],
                            ..Home::default()
                        });
                        home.bed = Some(pos);
                        home.center = pos;
                        home.floor_y = pos[1];
                    }
                }
                EventKind::PlacedBlock => {
                    if let Some(pos) = event.pos {
                        push_capped(&mut self.placed, pos, MOST_PLACED);
                        if is_light(&event.detail) || is_door(&event.detail) {
                            let home = self.home.get_or_insert_with(|| Home {
                                center: pos,
                                floor_y: pos[1],
                                ..Home::default()
                            });
                            if is_light(&event.detail) {
                                push_capped(&mut home.lights, pos, MOST_LIGHTS);
                            } else {
                                push_capped(&mut home.doors, pos, MOST_DOORS);
                            }
                        }
                    }
                }
                EventKind::BrokeBlock => {
                    if let Some(pos) = event.pos {
                        self.placed.retain(|p| *p != pos);
                        if let Some(home) = &mut self.home {
                            home.lights.retain(|p| *p != pos);
                            home.doors.retain(|p| *p != pos);
                        }
                        self.containers.retain(|c| c.pos != pos);
                    }
                }
                _ => {}
            }
        }

        for line in &observations.chat {
            if line.player == index && about_the_lodger(&line.text) {
                push_capped(&mut self.chat, line.text.clone(), MOST_CHAT);
            }
        }
    }

    fn observe_player(&mut self, player: &Player) {
        self.last.pos = player.pos;
        self.last.yaw = player.yaw;
        self.last.online = true;
        self.last.indoors = player.sky_light < 8;
        self.last.sleeping = player.sleeping;
        self.last.walking = player.movement.pace > 0.5;
        self.last.main_hand = player.main_hand.id.clone();
        self.last.look_backs = player.movement.look_backs;

        if player.movement.still_seconds >= 1.0 {
            let cell = [
                player.pos[0].floor() as i32 / CELL,
                player.pos[1].floor() as i32 / CELL,
                player.pos[2].floor() as i32 / CELL,
            ];
            match self.still_cells.iter_mut().find(|(c, _)| *c == cell) {
                Some((_, ticks)) => *ticks += 1,
                None => {
                    self.still_cells.push((cell, 1));
                    if self.still_cells.len() > MOST_CELLS {
                        let least = self
                            .still_cells
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, (_, t))| *t)
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        self.still_cells.remove(least);
                    }
                }
            }

            // Without a bed, the home is where they stand still most.
            if self.home.as_ref().map(|h| h.bed.is_none()).unwrap_or(true) {
                if let Some((cell, _)) = self.still_cells.iter().max_by_key(|(_, t)| *t) {
                    let center = [
                        cell[0] * CELL + CELL / 2,
                        cell[1] * CELL,
                        cell[2] * CELL + CELL / 2,
                    ];
                    let home = self.home.get_or_insert_with(|| Home {
                        center,
                        floor_y: center[1],
                        ..Home::default()
                    });
                    home.center = center;
                    home.floor_y = center[1];
                }
            }
        }

        if !player.main_hand.id.is_empty() {
            match self
                .favourites
                .iter_mut()
                .find(|(id, _)| *id == player.main_hand.id)
            {
                Some((_, count)) => *count += 1,
                None => self.favourites.push((player.main_hand.id.clone(), 1)),
            }
            self.favourites
                .sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            self.favourites.truncate(MOST_FAVOURITES);
        }
    }

    fn container_opened(&mut self, pos: BlockPos, contents: &[ItemStack], day: f32) {
        match self.containers.iter_mut().find(|c| c.pos == pos) {
            Some(container) => {
                container.opens += 1;
                container.last_opened_day = day;
                container.contents = contents.to_vec();
            }
            None => {
                self.containers.push(Container {
                    pos,
                    opens: 1,
                    last_opened_day: day,
                    contents: contents.to_vec(),
                });
                if self.containers.len() > MOST_CONTAINERS {
                    let least = self
                        .containers
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, c)| c.opens)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.containers.remove(least);
                }
            }
        }
    }

    /// Records an event's outcome.
    pub fn record(&mut self, event: &str, pos: Option<BlockPos>, day: f32, ok: bool, reason: &str) {
        push_capped(
            &mut self.done,
            Done {
                event: event.to_string(),
                pos,
                day,
                ok,
                reason: reason.to_string(),
            },
            MOST_DONE,
        );
    }

    /// The container the target opens most.
    pub fn used_container(&self) -> Option<&Container> {
        self.containers.iter().max_by_key(|c| c.opens)
    }

    /// The favourite item that sits in a used container, with the container and the slot.
    pub fn favourite_in_container(&self) -> Option<(&Container, &ItemStack)> {
        for (favourite, _) in &self.favourites {
            for container in &self.containers {
                if let Some(stack) = container.contents.iter().find(|s| s.id == *favourite) {
                    return Some((container, stack));
                }
            }
        }

        None
    }

    /// Whether the target is more than `blocks` from `pos`, or asleep, or offline.
    pub fn target_away_from(&self, pos: BlockPos, blocks: f64) -> bool {
        !self.last.online || self.last.sleeping || distance(block(self.last.pos), pos) > blocks
    }

    /// Whether `pos` is in front of the target, within `blocks`: the host cannot tell line of
    /// sight for the mind, so being within the angle counts as in view.
    pub fn in_view(&self, pos: BlockPos, blocks: f64) -> bool {
        if !self.last.online || self.last.sleeping {
            return false;
        }

        let at = block(self.last.pos);
        if distance(at, pos) > blocks {
            return false;
        }

        let dx = (pos[0] - at[0]) as f64;
        let dz = (pos[2] - at[2]) as f64;
        if dx == 0.0 && dz == 0.0 {
            return true;
        }

        // Yaw 0 faces +z, 90 faces -x, as the game has it.
        let facing_x = -(self.last.yaw as f64).to_radians().sin();
        let facing_z = (self.last.yaw as f64).to_radians().cos();
        let length = (dx * dx + dz * dz).sqrt();
        let cos = (dx * facing_x + dz * facing_z) / length;

        cos > 0.5 // within 60 degrees
    }

    /// The last events, newest first.
    pub fn recent(&self, count: usize) -> impl Iterator<Item = &Done> {
        self.done.iter().rev().take(count)
    }

    /// The lines the brain reads. A template, never prose: see `docs/memory.md`.
    pub fn describe(&self, day: f32) -> Vec<String> {
        let mut lines = Vec::new();

        if let Some(home) = &self.home {
            lines.push(format!(
                "The player sleeps in a house with {} door{} and {} light{}.",
                home.doors.len(),
                if home.doors.len() == 1 { "" } else { "s" },
                home.lights.len(),
                if home.lights.len() == 1 { "" } else { "s" }
            ));
        } else {
            lines.push("The player has no home yet that it knows of.".to_string());
        }

        if let Some((favourite, _)) = self.favourites.first() {
            lines.push(format!("They reach for {} most.", plain(favourite)));
        }

        if let Some(container) = self.used_container() {
            lines.push(format!(
                "The chest {} is opened most, {} times so far. It holds {}.",
                place_words(self.home.as_ref(), container.pos),
                container.opens,
                if container.contents.is_empty() {
                    "nothing".to_string()
                } else {
                    container
                        .contents
                        .iter()
                        .take(3)
                        .map(|s| plain(&s.id))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ));
        }

        for done in self.recent(3) {
            let ago = (day - done.day).max(0.0);
            let when = if ago < 1.0 {
                "Today".to_string()
            } else {
                format!("{} days ago", ago.floor() as u32)
            };
            lines.push(format!(
                "{} it {} {}.",
                when,
                if done.ok { "did" } else { "could not" },
                crate::events::find(&done.event)
                    .map(|e| e.menu_words())
                    .unwrap_or(&done.event)
            ));
        }

        if let Some(held) = &self.held {
            lines.push(format!(
                "It holds the player's {}, taken {} days ago.",
                plain(&held.item.id),
                (day - held.taken_day).max(0.0).floor() as u32
            ));
        }

        if let Some(line) = self.chat.last() {
            lines.push(format!("The player said: \"{}\".", line));
        }

        lines.push(format!(
            "Now the player is {}{}{}.",
            if !self.last.online {
                "away from the game"
            } else if self.last.sleeping {
                "asleep"
            } else if self.last.indoors {
                "indoors"
            } else {
                "outdoors"
            },
            if self.last.online && self.last.walking {
                ", walking"
            } else {
                ""
            },
            if self.last.look_backs >= 2 {
                ", and keeps looking behind them"
            } else {
                ""
            }
        ));

        lines
    }
}

fn push_capped<T>(list: &mut Vec<T>, value: T, cap: usize) {
    list.push(value);
    if list.len() > cap {
        list.remove(0);
    }
}

/// `minecraft:iron_pickaxe` as `an iron pickaxe`.
pub fn plain(id: &str) -> String {
    let name = id.rsplit(':').next().unwrap_or(id).replace('_', " ");
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };

    format!("{article} {name}")
}

fn place_words(home: Option<&Home>, pos: BlockPos) -> String {
    match home {
        Some(home) => {
            let d = distance(home.center, pos);
            if d <= 6.0 {
                "in the house".to_string()
            } else {
                let dx = pos[0] - home.center[0];
                let dz = pos[2] - home.center[2];
                let dir = if dx.abs() > dz.abs() {
                    if dx > 0 {
                        "east"
                    } else {
                        "west"
                    }
                } else if dz > 0 {
                    "south"
                } else {
                    "north"
                };
                format!("{} blocks {} of the house", d.round() as i32, dir)
            }
        }
        None => "somewhere".to_string(),
    }
}

fn about_the_lodger(text: &str) -> bool {
    let lower = text.to_lowercase();
    [
        "door",
        "chest",
        "torch",
        "house",
        "figure",
        "someone",
        "something",
        "who",
        "what",
        "?",
    ]
    .iter()
    .any(|word| lower.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use craftkit::world::{Event, Movement};

    fn player(index: u32, pos: [f64; 3], still: f32) -> Player {
        Player {
            index,
            name: "razor".to_string(),
            pos,
            sky_light: 15,
            movement: Movement {
                still_seconds: still,
                ..Movement::default()
            },
            main_hand: ItemStack {
                id: "minecraft:iron_pickaxe".to_string(),
                count: 1,
                slot: 0,
            },
            ..Player::default()
        }
    }

    fn obs(players: Vec<Player>, events: Vec<Event>) -> Observations {
        Observations {
            players,
            events,
            ..Observations::default()
        }
    }

    #[test]
    fn the_first_player_seen_is_the_target_and_their_favourite_is_what_they_hold() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(3, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);

        assert_eq!(memory.target, Some(3));
        assert_eq!(memory.target_name, "razor");
        assert_eq!(memory.favourites[0].0, "minecraft:iron_pickaxe");
    }

    #[test]
    fn a_bed_slept_in_is_the_home_and_a_chest_opened_is_the_used_container() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(0, [10.0, 64.0, 10.0], 0.0)], vec![]), 0.0);
        memory.observe(
            &obs(
                vec![player(0, [10.0, 64.0, 10.0], 0.0)],
                vec![
                    Event {
                        player: 0,
                        kind: EventKind::SleptBegan,
                        pos: Some([10, 64, 10]),
                        ..Event::default()
                    },
                    Event {
                        player: 0,
                        kind: EventKind::OpenedContainer,
                        pos: Some([12, 64, 10]),
                        detail: "minecraft:chest".to_string(),
                        contents: vec![ItemStack {
                            id: "minecraft:iron_pickaxe".to_string(),
                            count: 1,
                            slot: 4,
                        }],
                    },
                ],
            ),
            0.5,
        );

        let home = memory.home.as_ref().unwrap();
        assert_eq!(home.bed, Some([10, 64, 10]));
        assert_eq!(memory.used_container().unwrap().pos, [12, 64, 10]);
        let (container, stack) = memory.favourite_in_container().unwrap();
        assert_eq!(container.pos, [12, 64, 10]);
        assert_eq!(stack.slot, 4);
    }

    #[test]
    fn standing_still_makes_a_home_without_a_bed_and_the_cells_are_capped() {
        let mut memory = Memory::default();
        for _ in 0..30 {
            memory.observe(
                &obs(vec![player(0, [100.0, 64.0, 100.0], 2.0)], vec![]),
                0.0,
            );
        }
        assert_eq!(memory.home.as_ref().unwrap().center, [102, 64, 102]);

        for i in 0..700 {
            memory.observe(
                &obs(vec![player(0, [i as f64 * 8.0, 64.0, 0.0], 2.0)], vec![]),
                0.0,
            );
        }
        assert!(memory.still_cells.len() <= MOST_CELLS);
        assert_eq!(
            memory.home.as_ref().unwrap().center,
            [102, 64, 102],
            "the densest cell stays the home"
        );
    }

    #[test]
    fn in_view_is_in_front_and_near_and_away_is_far_or_asleep() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
        // Yaw 0 faces +z.
        assert!(memory.in_view([0, 64, 10], 24.0));
        assert!(!memory.in_view([0, 64, -10], 24.0), "behind");
        assert!(!memory.in_view([0, 64, 40], 24.0), "too far");
        assert!(!memory.target_away_from([0, 64, 5], 24.0));
        assert!(memory.target_away_from([0, 64, 30], 24.0));

        let mut asleep = player(0, [0.0, 64.0, 0.0], 0.0);
        asleep.sleeping = true;
        memory.observe(&obs(vec![asleep], vec![]), 0.0);
        assert!(memory.target_away_from([0, 64, 5], 24.0));
        assert!(!memory.in_view([0, 64, 5], 24.0));
    }

    #[test]
    fn describe_is_a_handful_of_plain_lines() {
        let mut memory = Memory::default();
        memory.observe(&obs(vec![player(0, [0.0, 64.0, 0.0], 0.0)], vec![]), 0.0);
        memory.record("open_door", Some([1, 64, 1]), 0.2, true, "");
        let lines = memory.describe(1.5);

        assert!(
            lines.iter().any(|l| l.contains("iron pickaxe")),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l == "1 days ago it did open the door."),
            "{lines:?}"
        );
        assert!(
            lines
                .last()
                .unwrap()
                .starts_with("Now the player is outdoors"),
            "{lines:?}"
        );
        assert!(lines.len() <= 12);
    }

    #[test]
    fn done_is_capped_and_plain_names_read_well() {
        let mut memory = Memory::default();
        for i in 0..300 {
            memory.record("wait", None, i as f32, true, "");
        }
        assert_eq!(memory.done.len(), MOST_DONE);
        assert_eq!(plain("minecraft:iron_pickaxe"), "an iron pickaxe");
        assert_eq!(plain("minecraft:torch"), "a torch");
    }
}
