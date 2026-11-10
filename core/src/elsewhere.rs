//! The Nether and the End (batch 16, owner: "the ghasts say your name, an enderman
//! holds your block"). The lodger lives in the overworld alone; its stay there is the one with
//! a stage, a want and rules. A stay of another dimension acts only through this: each tick the
//! overworld's stay says here which of the other dimensions' events its rules allow now, and
//! what they need from home (the target, their name, the block they build with); the other
//! stay, seeing the target in its world, does one of them now and then and says so here; the
//! overworld's stay records it on its next tick, so its caps, cooldowns and budget count it.

use crate::config::Config;
use crate::events;
use crate::stay::Stay;
use craftkit::world::{Command, Now};

/// The other dimensions' events, and the world each is done in.
pub const ROWS: &[(&str, &str)] = &[
    ("ghast_name", "minecraft:the_nether"),
    ("ender_holds", "minecraft:the_end"),
];
/// The chance a second, while one is allowed and the target is there: about once in five
/// minutes.
pub const CHANCE_A_SECOND: f64 = 1.0 / 300.0;

/// What the overworld's stay tells the others, and what they did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Elsewhere {
    pub target: Option<u32>,
    /// The target's uuid: the other stay matches players by it, so the index alone left it
    /// on whoever it saw first there.
    pub target_uuid: String,
    pub name: String,
    /// The block they build with most at home, for `ender_holds`.
    pub block: Option<String>,
    /// Of [`ROWS`], what the overworld's rules allow at this moment.
    pub allowed: Vec<&'static str>,
    /// Done in another dimension since the overworld last looked, with where.
    pub done: Vec<(&'static str, String)>,
}

impl Stay {
    /// The overworld's side, after its own tick: what was done elsewhere recorded, and what is
    /// allowed now said.
    pub fn share(&mut self, now: &Now, config: &Config, shared: &mut Elsewhere) {
        let day = self.day(now);
        for (id, world) in shared.done.drain(..) {
            if let Some(event) = events::find(id) {
                self.memory.record(id, None, day, true, &world);
                self.arc.spend(event.cost_in(config), now.day);
                // The enderman's block came out of what it holds, never from nothing.
                if id == "ender_holds" {
                    if let Some(block) = &shared.block {
                        self.memory.store(block, -1);
                    }
                }
            }
        }
        shared.target = self.memory.target;
        shared.target_uuid = self.memory.target_uuid.clone();
        shared.name = self.memory.target_name.clone();
        // The block they build with most, when it has one of its own (from its digging) to
        // give: until now it was made from nothing.
        shared.block = self
            .memory
            .block_built_most()
            .filter(|b| self.memory.stored(b) > 0);
        shared.allowed.clear();
        if !self.arc.moved_in() || !config.enabled {
            return;
        }
        for (id, _) in ROWS {
            let Some(event) = events::find(id) else {
                continue;
            };
            if crate::brain::offer::switched_on(event, config)
                && crate::rules::check(event, None, &self.memory, self.arc.stage, day, config)
                    .is_ok()
                && self.arc.affords(event.cost_in(config), now.day, config)
                && (*id != "ender_holds" || shared.block.is_some())
            {
                shared.allowed.push(id);
            }
        }
    }

    /// Another dimension's side: with the target here and one of its events allowed, now and
    /// then, that event.
    pub fn tick_elsewhere(&mut self, now: &Now, shared: &mut Elsewhere) -> Vec<Command> {
        self.ticks += 1;
        self.last_now = now.clone();
        if !now.tick.is_multiple_of(20) || shared.target.is_none() {
            return Vec::new();
        }
        if self.memory.target != shared.target || self.memory.target_uuid != shared.target_uuid {
            if let Some(index) = shared.target {
                self.memory
                    .set_target(index, &shared.name.clone(), &shared.target_uuid.clone());
            }
            self.memory.last = Default::default();
            return Vec::new();
        }
        if !self.memory.last.online {
            return Vec::new();
        }
        let world = self.world.clone();
        let Some(&(id, _)) = ROWS
            .iter()
            .find(|(id, w)| *w == world && shared.allowed.contains(id))
        else {
            return Vec::new();
        };
        let near = id != "ender_holds"
            || self
                .memory
                .last
                .hostiles_near
                .iter()
                .any(|m| m.kind == "enderman");
        if !near || self.brain.rng.next_f64() >= CHANCE_A_SECOND {
            return Vec::new();
        }
        let Some(event) = events::find(id) else {
            return Vec::new();
        };
        self.memory.target_name = shared.name.clone();
        let mut next = self.ticks.wrapping_mul(1000);
        let mut ids = || {
            next += 1;
            next
        };
        let made = crate::brain::act::commands(
            event,
            None,
            &self.memory,
            &mut ids,
            now,
            shared.block.as_deref(),
        );
        if made.is_empty() {
            return Vec::new();
        }
        shared.allowed.retain(|a| *a != id);
        shared.done.push((id, world));
        // What waits a moment goes with the rest: another dimension's stay keeps no later.
        made.into_iter().map(|(command, _)| command).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craftkit::world::{Entity, Observations, Player, WorldInfo};

    fn stay(world: &str) -> Stay {
        Stay::new(&WorldInfo {
            id: world.to_string(),
            ..WorldInfo::default()
        })
    }

    fn player_at(pos: [f64; 3]) -> Observations {
        Observations {
            players: vec![Player {
                index: 0,
                name: "Player612".to_string(),
                pos,
                ..Player::default()
            }],
            ..Observations::default()
        }
    }

    #[test]
    fn the_ghasts_cry_their_name_once_the_overworld_allows_it_and_it_is_counted_there() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        let mut home = stay("minecraft:overworld");
        home.memory.target = Some(0);
        home.memory.target_name = "Player612".to_string();
        home.arc.stage = 3;
        let mut shared = Elsewhere::default();
        let now = Now {
            tick: 40,
            day: 0,
            ..Now::default()
        };
        home.share(&now, &config, &mut shared);
        assert_eq!(
            shared.allowed,
            vec!["ghast_name"],
            "no block of theirs: no enderman"
        );
        assert_eq!(shared.target, Some(0));

        // In the Nether, the target there: now and then, their name in the ghasts' cries.
        let mut nether = stay("minecraft:the_nether");
        let mut sent = Vec::new();
        for i in 0..20_000u64 {
            let at = Now {
                tick: 40 + i * 20,
                ..Now::default()
            };
            nether.observe(&player_at([0.0, 70.0, 0.0]), 0.0);
            sent = nether.tick_elsewhere(&at, &mut shared);
            if !sent.is_empty() {
                break;
            }
        }
        assert!(
            matches!(&sent[0], Command::Hint { hint, detail, seconds, .. }
                if hint == "ghast_name" && detail == "Player612" && *seconds == 300.0),
            "{sent:?}"
        );
        assert!(
            matches!(&sent[1], Command::PlaySound { sound, .. } if sound.ends_with("ghast.ambient"))
        );
        assert!(
            shared.allowed.is_empty(),
            "once, until the overworld says again"
        );

        // The overworld counts it on its next tick, and its rules hold it back after.
        home.share(&now, &config, &mut shared);
        assert_eq!(home.memory.times("ghast_name"), 1);
        assert!(shared.done.is_empty());
        assert!(
            !shared.allowed.contains(&"ghast_name"),
            "the same as the last event: {:?}",
            shared.allowed
        );
    }

    #[test]
    fn an_enderman_holds_the_block_they_build_with_and_nothing_happens_before_stage_three() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        let mut home = stay("minecraft:overworld");
        home.memory.target = Some(0);
        home.arc.stage = 2;
        home.memory.walls = vec![
            ([1, 64, 0], "minecraft:cobblestone".to_string()),
            ([2, 64, 0], "minecraft:cobblestone".to_string()),
            ([3, 64, 0], "minecraft:spruce_planks".to_string()),
        ];
        // None of its own to give: no enderman holds one.
        let mut shared = Elsewhere::default();
        home.arc.stage = 3;
        home.share(&Now::default(), &config, &mut shared);
        assert!(
            !shared.allowed.contains(&"ender_holds"),
            "{:?}",
            shared.allowed
        );
        home.memory.store("minecraft:cobblestone", 1);
        home.arc.stage = 2;
        let mut shared = Elsewhere::default();
        home.share(&Now::default(), &config, &mut shared);
        assert!(shared.allowed.is_empty(), "stage two: {:?}", shared.allowed);
        home.arc.stage = 3;
        home.share(&Now::default(), &config, &mut shared);
        assert!(shared.allowed.contains(&"ender_holds"));
        assert_eq!(shared.block.as_deref(), Some("minecraft:cobblestone"));

        let mut end = stay("minecraft:the_end");
        let mut seen = player_at([0.0, 64.0, 0.0]);
        seen.entities = vec![Entity {
            id: 900,
            kind: "minecraft:enderman".to_string(),
            pos: [10.0, 64.0, 0.0],
            hostile: true,
            ..Entity::default()
        }];
        let mut sent = Vec::new();
        for i in 0..20_000u64 {
            end.observe(&seen, 0.0);
            sent = end.tick_elsewhere(
                &Now {
                    tick: 20 + i * 20,
                    ..Now::default()
                },
                &mut shared,
            );
            if !sent.is_empty() {
                break;
            }
        }
        assert!(
            matches!(&sent[0], Command::CarryBlock { entity: 900, block, .. } if block == "minecraft:cobblestone"),
            "{sent:?}"
        );
        // Counted at home, and spent from what it holds.
        home.share(&Now::default(), &config, &mut shared);
        assert_eq!(home.memory.stored("minecraft:cobblestone"), 0);
        // Never the ghasts' in the End, nor the enderman's in the Nether.
        assert!(sent.iter().all(|c| !matches!(c, Command::Hint { .. })));
    }

    #[test]
    fn the_other_stay_follows_the_overworld_s_target_not_whoever_it_saw_first() {
        let mut nether = stay("minecraft:the_nether");
        nether.memory.set_target(1, "someone", "uuid-b");
        let mut shared = Elsewhere {
            target: Some(1),
            target_uuid: "uuid-a".to_string(),
            name: "Player612".to_string(),
            ..Elsewhere::default()
        };
        nether.tick_elsewhere(
            &Now {
                tick: 20,
                ..Now::default()
            },
            &mut shared,
        );

        assert_eq!(nether.memory.target_uuid, "uuid-a");
        assert_eq!(nether.memory.target_name, "Player612");
    }

    #[test]
    fn with_no_enderman_near_or_the_target_away_nothing_is_done() {
        let config = Config::default();
        let mut home = stay("minecraft:overworld");
        home.memory.target = Some(0);
        home.arc.stage = 4;
        home.memory.walls = vec![([1, 64, 0], "minecraft:stone_bricks".to_string())];
        home.memory.store("minecraft:stone_bricks", 2);
        let mut shared = Elsewhere::default();
        home.share(&Now::default(), &config, &mut shared);
        let mut end = stay("minecraft:the_end");
        for i in 0..5_000u64 {
            end.observe(&player_at([0.0, 64.0, 0.0]), 0.0);
            let sent = end.tick_elsewhere(
                &Now {
                    tick: 20 + i * 20,
                    ..Now::default()
                },
                &mut shared,
            );
            assert!(sent.is_empty(), "no enderman: {sent:?}");
        }
        let mut nether = stay("minecraft:the_nether");
        for i in 0..5_000u64 {
            nether.observe(&Observations::default(), 0.0);
            let sent = nether.tick_elsewhere(
                &Now {
                    tick: 20 + i * 20,
                    ..Now::default()
                },
                &mut shared,
            );
            assert!(sent.is_empty(), "no target there: {sent:?}");
        }
    }
}
