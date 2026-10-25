//! How the lodger decides: the menu offered (`offer`), the pick (`rules_brain` today; the model
//! in phase 4), the plan it becomes, the commands the plan turns into (`act`), and the log of
//! every decision (`log`). The `Brain` lives in a `Stay` and is driven once a tick.

pub mod act;
pub mod log;
pub mod offer;
pub mod rules_brain;
pub mod text;

use crate::arc::Arc;
use crate::config::Config;
use crate::events::{self, Event};
use crate::memory::Memory;
use crate::model::Rng;
use crate::rules;
use craftkit::world::{BlockPos, Command, Now, Report};
use serde::{Deserialize, Serialize};

/// Ticks a second.
pub const TPS: u64 = 20;

/// One decided event, waiting for its moment.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plan {
    pub event: String,
    pub pos: Option<BlockPos>,
    /// The tick it may run from; it waits past this for its conditions.
    pub from_tick: u64,
    /// The tick it is dropped at if its conditions never came.
    pub until_tick: u64,
    pub decided_day: f32,
    /// Forced from the debug screen: it happens on the next tick, whatever the needs, the view
    /// and the rules say, and costs the day's budget nothing.
    #[serde(default)]
    pub forced: bool,
}

/// A command sent and not yet reported.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pending {
    pub id: u64,
    pub event: String,
    pub pos: Option<BlockPos>,
    pub day: f32,
}

/// A command to send at a later tick: a chest's close after its open.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Later {
    pub at_tick: u64,
    pub command: Command,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Brain {
    pub plan: Option<Plan>,
    pub pending: Vec<Pending>,
    pub later: Vec<Later>,
    pub next_plan_tick: u64,
    pub next_command_id: u64,
    pub rng: Rng,
    /// Decisions made, for the status line.
    pub decisions: u64,
}

impl Brain {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
            next_command_id: 1,
            ..Self::default()
        }
    }

    fn id(&mut self) -> u64 {
        let id = self.next_command_id;
        self.next_command_id += 1;

        id
    }

    /// One tick: maybe decide, maybe act. Answers the commands to send.
    pub fn tick(
        &mut self,
        now: &Now,
        day: f32,
        memory: &mut Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
    ) -> Vec<Command> {
        let mut commands = Vec::new();

        // Follow-ups whose time has come.
        let mut kept = Vec::new();
        for later in self.later.drain(..) {
            if later.at_tick <= now.tick {
                commands.push(later.command);
            } else {
                kept.push(later);
            }
        }
        self.later = kept;

        if !arc.moved_in() {
            return commands;
        }

        if self.plan.is_none() && now.tick >= self.next_plan_tick {
            self.next_plan_tick = now.tick + config.brain.plan_every as u64 * TPS;
            self.decide(now, day, memory, arc, config, log);
        }

        if let Some(plan) = self.plan.clone() {
            if now.tick > plan.until_tick {
                memory.record(&plan.event, plan.pos, day, false, "its moment never came");
                self.plan = None;
            } else if now.tick >= plan.from_tick {
                if let Some(event) = events::find(&plan.event) {
                    if plan.forced || act::ready(event, memory, config) {
                        let checked = if plan.forced {
                            Ok(())
                        } else {
                            rules::check(event, plan.pos, memory, arc.stage, day, config)
                        };
                        match checked {
                            Ok(()) => {
                                let mut ids = || self.id();
                                let made = act::commands(event, plan.pos, memory, &mut ids, now);
                                if !plan.forced {
                                    arc.spend(event.cost());
                                }
                                if plan.forced && made.is_empty() {
                                    memory.record(
                                        &plan.event,
                                        plan.pos,
                                        day,
                                        false,
                                        "nothing to do it with",
                                    );
                                }
                                log.acted(&plan.event, plan.pos, made.len());
                                for (command, later_ticks) in made {
                                    let id = command.id();
                                    self.pending.push(Pending {
                                        id,
                                        event: plan.event.clone(),
                                        pos: plan.pos,
                                        day,
                                    });
                                    if later_ticks == 0 {
                                        commands.push(command);
                                    } else {
                                        self.later.push(Later {
                                            at_tick: now.tick + later_ticks,
                                            command,
                                        });
                                    }
                                }
                                if self.pending.is_empty() && !plan.forced {
                                    memory.record(&plan.event, plan.pos, day, true, "");
                                }
                                self.plan = None;
                            }
                            Err(refused) => {
                                memory.record(&plan.event, plan.pos, day, false, refused.words());
                                self.plan = None;
                            }
                        }
                    }
                }
            }
        }

        commands
    }

    fn decide(
        &mut self,
        now: &Now,
        day: f32,
        memory: &Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
    ) {
        let menu = offer::offer(
            events::CATALOGUE,
            arc.stage,
            memory,
            now,
            day,
            config,
            &mut self.rng,
        );
        if menu.is_empty() {
            return;
        }

        let chosen = rules_brain::choose(&menu, arc.stage, &mut self.rng);
        self.decisions += 1;
        let pos = act::place(chosen, memory, &mut self.rng);
        let lines = memory.describe(day);
        log.decided(arc.stage, day, &lines, &menu, chosen.id, pos);

        if chosen.id == "wait" {
            return;
        }

        if !arc.affords(chosen.cost(), now.day, config) {
            log.note("the day's budget is spent");
            return;
        }

        // Soon: within four planning intervals (six minutes at the default `plan_every`, a
        // minute in a dev config), when its conditions hold; dropped a day after that.
        let delay = self.rng.next_u64() % (4 * config.brain.plan_every as u64 * TPS).max(1);
        self.plan = Some(Plan {
            event: chosen.id.to_string(),
            pos,
            from_tick: now.tick + delay,
            until_tick: now.tick + delay + 24_000,
            decided_day: day,
            forced: false,
        });
    }

    /// Queues an event to happen on the next tick, whatever the rules say: the debug screen's
    /// `Force`. Answers what was queued, or why nothing could be.
    pub fn force(
        &mut self,
        event: &Event,
        memory: &Memory,
        day: f32,
        log: &mut log::Log,
    ) -> String {
        let pos = act::place(event, memory, &mut self.rng);
        let mut ids = || 0u64;
        let made = act::commands(event, pos, memory, &mut ids, &Now::default());
        if made.is_empty() {
            let why = offer::unmet(event, memory, &Now::default())
                .unwrap_or_else(|| "nothing in the memory to do it with".to_string());
            return format!("{} cannot happen: {why}", event.id);
        }

        self.plan = Some(Plan {
            event: event.id.to_string(),
            pos,
            from_tick: 0,
            until_tick: u64::MAX,
            decided_day: day,
            forced: true,
        });
        log.note(&format!("forced {}", event.id));

        match pos {
            Some(p) => format!(
                "{} queued at ({},{},{}), {} command(s)",
                event.id,
                p[0],
                p[1],
                p[2],
                made.len()
            ),
            None => format!("{} queued, {} command(s)", event.id, made.len()),
        }
    }

    /// How the last tick's commands went.
    pub fn report(&mut self, reports: &[Report], memory: &mut Memory, log: &mut log::Log) {
        for report in reports {
            if let Some(index) = self.pending.iter().position(|p| p.id == report.id) {
                let pending = self.pending.remove(index);
                let others = self
                    .pending
                    .iter()
                    .any(|p| p.event == pending.event && p.day == pending.day);
                if !others || !report.done {
                    memory.record(
                        &pending.event,
                        pending.pos,
                        pending.day,
                        report.done,
                        &report.reason,
                    );
                    log.reported(&pending.event, report.done, &report.reason);
                    if !report.done {
                        self.pending
                            .retain(|p| !(p.event == pending.event && p.day == pending.day));
                    }
                }

                if report.done && pending.event == "take" {
                    // The host took it; the memory holds it from here.
                    act::took(memory, pending.pos);
                }
                if report.done
                    && (pending.event == "return_taken" || pending.event == "place_taken")
                {
                    memory.held = None;
                }
            }
        }
    }
}

/// The events worth offering: a convenience for the harness and the status line.
pub fn offered_now(
    memory: &Memory,
    arc: &Arc,
    now: &Now,
    day: f32,
    config: &Config,
    rng: &mut Rng,
) -> Vec<&'static Event> {
    offer::offer(events::CATALOGUE, arc.stage, memory, now, day, config, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Home;

    #[test]
    fn a_plan_s_moment_is_within_four_planning_intervals() {
        let mut config = Config::default();
        config.brain.plan_every = 15;
        config.stages.start_after_days = 0;
        let mut memory = Memory {
            target: Some(0),
            ..Memory::default()
        };
        memory.home = Some(Home {
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![],
        });
        memory.last.online = false;
        let mut arc = Arc::default();
        arc.tick(0.0, &memory, &config);
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);

        // Decide until something other than wait is planned.
        let mut now = Now::default();
        for _ in 0..200 {
            brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log);
            if brain.plan.is_some() {
                break;
            }
            now.tick += 15 * TPS;
        }

        let plan = brain.plan.as_ref().expect("a plan");
        assert!(
            plan.from_tick - now.tick <= 4 * 15 * TPS,
            "{} ticks ahead",
            plan.from_tick - now.tick
        );
        assert_eq!(plan.until_tick, plan.from_tick + 24_000);
    }
}
