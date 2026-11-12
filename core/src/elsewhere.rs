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
use craftkit::world::{BlockPos, Command, Now};

/// The other dimensions' events, and the world each is done in.
pub const ROWS: &[(&str, &str)] = &[
    ("ghast_name", "minecraft:the_nether"),
    ("ender_holds", "minecraft:the_end"),
    ("piglins_stare", "minecraft:the_nether"),
    ("strider_turns_home", "minecraft:the_nether"),
    ("ghast_watches", "minecraft:the_nether"),
    ("hoglins_part", "minecraft:the_nether"),
    ("magma_cubes_settle", "minecraft:the_nether"),
    ("back_to_the_wall", "minecraft:the_nether"),
    ("endermen_turn_away", "minecraft:the_end"),
    ("zombified_follow", "minecraft:the_nether"),
    ("rain_in_the_nether", "minecraft:the_nether"),
    ("their_door_behind", "minecraft:the_nether"),
    ("nether_roof_steps", "minecraft:the_nether"),
    ("home_through_the_portal", "minecraft:the_nether"),
    ("the_dragon_growls", "minecraft:the_end"),
    ("second_wings", "minecraft:the_end"),
    ("purpur_knock", "minecraft:the_end"),
    ("ship_creaks", "minecraft:the_end"),
    ("souls_follow", "minecraft:the_nether"),
    ("breath_in_the_nether", "minecraft:the_nether"),
    ("their_torch_ahead", "minecraft:the_nether"),
    ("name_in_netherrack", "minecraft:the_nether"),
    ("prints_in_soul_sand", "minecraft:the_nether"),
    ("prints_off_the_edge", "minecraft:the_end"),
    ("portal_weeps", "minecraft:the_nether"),
    ("compass_steadies", "minecraft:the_nether"),
    ("standing_in_the_portal", "minecraft:the_nether"),
    ("far_end_of_the_bridge", "minecraft:the_nether"),
    ("name_from_the_void", "minecraft:the_end"),
    ("standing_on_the_void", "minecraft:the_end"),
    ("splash_in_the_nether", "minecraft:the_nether"),
    ("lightning_below", "minecraft:the_nether"),
    ("lightning_below", "minecraft:the_end"),
    ("shulkers_open", "minecraft:the_end"),
    ("gateway_flashes", "minecraft:the_end"),
    ("blaze_ring", "minecraft:the_nether"),
    ("endermen_at_the_edge", "minecraft:the_end"),
    ("end_goes_quiet", "minecraft:the_end"),
    ("chorus_blooms", "minecraft:the_end"),
    ("vines_reach", "minecraft:the_nether"),
    ("portal_goes_out", "minecraft:the_nether"),
    ("lava_goes_still", "minecraft:the_nether"),
    ("end_sky_face", "minecraft:the_end"),
    ("barter_from_home", "minecraft:the_nether"),
    ("elytra_at_home", "minecraft:the_end"),
];
/// The chance a second, while one is allowed and the target is there: about once in five
/// minutes.
pub const CHANCE_A_SECOND: f64 = 1.0 / 300.0;

/// The overworld's home as another world has it: an eighth as far in the Nether, as its portals
/// go; the same in the End.
pub fn home_here(world: &str, home: BlockPos) -> BlockPos {
    if world == "minecraft:the_nether" {
        [home[0].div_euclid(8), home[1], home[2].div_euclid(8)]
    } else {
        home
    }
}

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
    /// The home's middle in the overworld, for what the other worlds do toward it.
    pub home: Option<BlockPos>,
    /// One thing in the chest they use at home, none of their favourites: where, its slot and
    /// what it is, for `barter_from_home`.
    pub chest: Option<(BlockPos, u32, String)>,
    /// An item frame of theirs at home, for `elytra_at_home`.
    pub frame: Option<u32>,
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
        shared.home = self.memory.home.as_ref().map(|h| h.center);
        shared.chest = self.memory.used_container().and_then(|c| {
            c.contents
                .iter()
                .find(|s| !self.memory.favourites.iter().any(|(f, _)| *f == s.id))
                .map(|s| (c.pos, s.slot, s.id.clone()))
        });
        shared.frame = self.memory.decor("item_frame").next().map(|d| d.id);
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
                && (*id != "barter_from_home" || shared.chest.is_some())
                && (*id != "elytra_at_home" || (shared.frame.is_some() && shared.home.is_some()))
            {
                shared.allowed.push(id);
            }
        }
    }

    /// Another dimension's side: what waits a moment sent at its time; and with the target here,
    /// now and then, one of its events the overworld allows whose needs this world meets, chosen
    /// among them.
    pub fn tick_elsewhere(&mut self, now: &Now, shared: &mut Elsewhere) -> Vec<Command> {
        self.ticks += 1;
        self.last_now = now.clone();
        let mut out: Vec<Command> = Vec::new();
        let mut waiting = std::mem::take(&mut self.waiting);
        waiting.retain(|(due, command)| {
            if *due <= now.tick {
                out.push(command.clone());
                false
            } else {
                true
            }
        });
        self.waiting = waiting;
        if !now.tick.is_multiple_of(20) || shared.target.is_none() {
            return out;
        }
        if self.memory.target != shared.target || self.memory.target_uuid != shared.target_uuid {
            if let Some(index) = shared.target {
                self.memory
                    .set_target(index, &shared.name.clone(), &shared.target_uuid.clone());
            }
            self.memory.last = Default::default();
            return out;
        }
        if !self.memory.last.online {
            return out;
        }
        let world = self.world.clone();
        let ready: Vec<&'static str> = ROWS
            .iter()
            .filter(|(id, w)| {
                *w == world && shared.allowed.contains(id) && self.meets_here(id, now)
            })
            .map(|(id, _)| *id)
            .collect();
        if ready.is_empty() || self.brain.rng.next_f64() >= CHANCE_A_SECOND {
            return out;
        }
        let id = ready[(self.brain.rng.next_u64() % ready.len() as u64) as usize];
        let Some(event) = events::find(id) else {
            return out;
        };
        self.memory.target_name = shared.name.clone();
        let mut next = self.ticks.wrapping_mul(1000);
        let mut ids = || {
            next += 1;
            next
        };
        // Where it has no place of its own here, the home's, as this world has it: an eighth as far
        // in the Nether, as a portal's way is.
        let pos = crate::brain::act::place(event, &self.memory, &mut self.brain.rng)
            .or_else(|| shared.home.map(|h| home_here(&world, h)));
        // What the event needs from home, as text: the chest's thing, the frame, else the block.
        let from_home = match id {
            "barter_from_home" => shared
                .chest
                .as_ref()
                .map(|(p, slot, item)| format!("{},{},{},{slot},{item}", p[0], p[1], p[2])),
            "elytra_at_home" => shared
                .frame
                .zip(shared.home)
                .map(|(f, h)| format!("{f};{},{},{}", h[0], h[1], h[2])),
            _ => shared.block.clone(),
        };
        let made = crate::brain::act::commands(
            event,
            pos,
            &self.memory,
            &mut ids,
            now,
            from_home.as_deref(),
        );
        if made.is_empty() {
            return out;
        }
        shared.allowed.retain(|a| *a != id);
        shared.done.push((id, world));
        for (command, delay) in made {
            if delay == 0 {
                out.push(command);
            } else {
                self.waiting.push((now.tick + delay, command));
            }
        }
        out
    }

    /// Whether this world meets an event's needs as they are here, the dimension's own need
    /// aside: an enderman near for `ender_holds`, piglins near for the piglins' stare.
    fn meets_here(&self, id: &str, now: &Now) -> bool {
        let Some(event) = events::find(id) else {
            return false;
        };
        let enderman = id != "ender_holds"
            || self
                .memory
                .last
                .hostiles_near
                .iter()
                .any(|m| m.kind == "enderman");
        enderman
            && event
                .needs
                .iter()
                .filter(|n| !matches!(n, events::Need::InNether | events::Need::InEnd))
                .all(|n| crate::brain::offer::meets(*n, &self.memory, now))
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
        assert!(
            shared.allowed.contains(&"ghast_name"),
            "{:?}",
            shared.allowed
        );
        assert!(
            !shared.allowed.contains(&"ender_holds"),
            "no block of theirs: no enderman"
        );
        assert_eq!(shared.target, Some(0));
        // The ghasts' alone, among the Nether's many.
        shared.allowed = vec!["ghast_name"];

        // In the Nether, the target there: now and then, their name in the ghasts' cries.
        let mut nether = stay("minecraft:the_nether");
        let mut sent = Vec::new();
        let mut sent_at = 0;
        for i in 0..20_000u64 {
            let at = Now {
                tick: 40 + i * 20,
                ..Now::default()
            };
            nether.observe(&player_at([0.0, 70.0, 0.0]), 0.0);
            sent = nether.tick_elsewhere(&at, &mut shared);
            sent_at = at.tick;
            if !sent.is_empty() {
                break;
            }
        }
        assert!(
            matches!(&sent[0], Command::Hint { hint, detail, seconds, .. }
                if hint == "ghast_name" && detail == "Player612" && *seconds == 300.0),
            "{sent:?}"
        );
        // Its cry a moment on, at the time the event gives it (until now every command went at
        // once, the waits dropped).
        let mut later = Vec::new();
        for i in 1..400u64 {
            later.extend(nether.tick_elsewhere(
                &Now {
                    tick: sent_at + i,
                    ..Now::default()
                },
                &mut shared,
            ));
        }
        assert!(
            sent.iter()
                .chain(later.iter())
                .any(|c| matches!(c, Command::PlaySound { sound, .. } if sound.ends_with("ghast.ambient"))),
            "{sent:?} {later:?}"
        );
        assert!(
            !shared.allowed.contains(&"ghast_name"),
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
    fn the_nether_chooses_among_what_its_world_meets_and_nothing_without_it() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        let mut home = stay("minecraft:overworld");
        home.memory.target = Some(0);
        home.memory.target_name = "Player612".to_string();
        home.arc.stage = 3;
        let mut shared = Elsewhere::default();
        home.share(&Now::default(), &config, &mut shared);
        assert!(
            shared.allowed.contains(&"piglins_stare"),
            "{:?}",
            shared.allowed
        );
        shared.allowed = vec!["piglins_stare"];

        let mut nether = stay("minecraft:the_nether");
        let tick = |nether: &mut Stay, shared: &mut Elsewhere, seen: &Observations, from: u64| {
            for i in 0..4_000u64 {
                nether.observe(seen, 0.0);
                let sent = nether.tick_elsewhere(
                    &Now {
                        tick: from + i * 20,
                        ..Now::default()
                    },
                    shared,
                );
                if !sent.is_empty() {
                    return sent;
                }
            }
            Vec::new()
        };
        let alone = player_at([0.0, 70.0, 0.0]);
        assert!(
            tick(&mut nether, &mut shared, &alone, 20).is_empty(),
            "no piglin near: nothing"
        );

        let mut with = player_at([0.0, 70.0, 0.0]);
        with.entities = vec![Entity {
            id: 901,
            kind: "minecraft:piglin".to_string(),
            pos: [6.0, 70.0, 0.0],
            hostile: true,
            ..Entity::default()
        }];
        let sent = tick(&mut nether, &mut shared, &with, 100_000);
        assert!(
            matches!(&sent[..], [Command::Hold { entity: 901, .. }]),
            "{sent:?}"
        );
        assert_eq!(
            shared.done,
            vec![("piglins_stare", "minecraft:the_nether".to_string())]
        );
    }

    #[test]
    fn the_overworld_tells_the_nether_a_thing_from_their_chest_never_a_favourite() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        let mut home = stay("minecraft:overworld");
        home.memory.target = Some(0);
        home.arc.stage = 4;
        home.memory
            .favourites
            .push(("minecraft:diamond".to_string(), 9));
        home.memory.containers.push(crate::memory::Container {
            pos: [2, 64, 0],
            kind: "minecraft:chest".to_string(),
            placed: true,
            opens: 5,
            contents: vec![
                craftkit::world::ItemStack {
                    id: "minecraft:diamond".to_string(),
                    count: 3,
                    slot: 0,
                },
                craftkit::world::ItemStack {
                    id: "minecraft:apple".to_string(),
                    count: 2,
                    slot: 4,
                },
            ],
            ..Default::default()
        });
        let mut shared = Elsewhere::default();
        home.share(&Now::default(), &config, &mut shared);
        assert_eq!(
            shared.chest,
            Some(([2, 64, 0], 4, "minecraft:apple".to_string())),
            "never their diamonds"
        );
        assert!(shared.allowed.contains(&"barter_from_home"));
        shared.allowed = vec!["barter_from_home"];

        let mut nether = stay("minecraft:the_nether");
        let mut seen = player_at([0.0, 70.0, 0.0]);
        seen.entities = vec![Entity {
            id: 902,
            kind: "minecraft:piglin".to_string(),
            pos: [5.0, 70.0, 0.0],
            hostile: true,
            ..Entity::default()
        }];
        let mut sent = Vec::new();
        for i in 0..20_000u64 {
            nether.observe(&seen, 0.0);
            sent = nether.tick_elsewhere(
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
            matches!(&sent[..], [Command::InWorld { dimension, .. }] if dimension == "minecraft:overworld"),
            "the take at home first, the drop a moment on: {sent:?}"
        );
        assert_eq!(nether.waiting.len(), 1, "the drop waits");
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
        // The enderman's block alone, among the End's many.
        shared.allowed = vec!["ender_holds"];

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
        // The enderman's event alone in the End: the End's others need no enderman.
        let all = shared.allowed.clone();
        shared.allowed.retain(|a| *a == "ender_holds");
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
        shared.allowed = all;
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
