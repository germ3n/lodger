//! How the lodger decides: the menu offered (`offer`), the pick (`rules_brain` today; the model
//! in phase 4), the plan it becomes, the commands the plan turns into (`act`), and the log of
//! every decision (`log`). The `Brain` lives in a `Stay` and is driven once a tick.

pub mod act;
pub mod log;
pub mod offer;
pub mod rules_brain;

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
pub struct Plan {
    pub event: String,
    pub pos: Option<BlockPos>,
    /// The tick it may run from; it waits past this for its conditions.
    pub from_tick: u64,
    /// The tick it is dropped at if its conditions never came.
    pub until_tick: u64,
    pub decided_day: f32,
}

/// A command sent and not yet reported.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
                    if act::ready(event, memory, config) {
                        match rules::check(event, plan.pos, memory, arc.stage, day, config) {
                            Ok(()) => {
                                let mut ids = || self.id();
                                let made = act::commands(event, plan.pos, memory, &mut ids, now);
                                arc.spend(event.cost());
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
                                if self.pending.is_empty() {
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

        // Soon: within the next ten minutes, when its conditions hold; dropped after a day.
        let delay = self.rng.next_u64() % (10 * 60 * TPS);
        self.plan = Some(Plan {
            event: chosen.id.to_string(),
            pos,
            from_tick: now.tick + delay,
            until_tick: now.tick + 24_000,
            decided_day: day,
        });
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
