//! How the lodger decides: the menu offered (`offer`), the pick (`rules_brain` today; the model
//! in phase 4), the plan it becomes, the commands the plan turns into (`act`), and the log of
//! every decision (`log`). The `Brain` lives in a `Stay` and is driven once a tick.

pub mod act;
pub mod log;
pub mod offer;
pub mod rules_brain;
pub mod situation;
pub mod text;

use crate::arc::Arc;
use crate::brain::text::Kind;
use crate::config::Config;
use crate::events::{self, Event};
use crate::memory::Memory;
use crate::model::{Answer, Model, Question, Reply, Rng};
use crate::rules;
use craftkit::world::{BlockPos, Command, Now, Report};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};

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
    /// The tick the surroundings may be asked for again, so an unanswered pass is retried, not
    /// repeated every tick.
    pub next_survey_tick: u64,
    pub next_command_id: u64,
    pub rng: Rng,
    /// Decisions made, for the status line.
    pub decisions: u64,
    /// The last decisions in full, newest last, for the debug screen's story: what was offered,
    /// what was chosen and why (the situation as read), and what came of it.
    pub story: Vec<Decision>,
    /// The question out to the model, if one is; answered on a later tick.
    #[serde(skip)]
    pub asking: Option<Ask>,
    /// The line the model is writing for the planned event, if one is.
    #[serde(skip)]
    pub writing: Option<Writing>,
    /// Lines the model wrote, by event, used when the plan runs.
    #[serde(skip)]
    pub written: Vec<(String, String)>,
    /// The channel the model answers on. Made on first use; not saved, not compared, not cloned
    /// (a cloned brain starts a channel of its own).
    #[serde(skip)]
    replies: Channel,
}

/// The reply channel, wrapped so the brain can still derive `Clone`, `PartialEq` and
/// `Default`: a channel is never equal to another and a clone is a fresh one.
#[derive(Default)]
pub struct Channel(Option<(Sender<Reply>, Receiver<Reply>)>);

impl Clone for Channel {
    fn clone(&self) -> Self {
        Channel(None)
    }
}

impl PartialEq for Channel {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() {
            "Channel(open)"
        } else {
            "Channel(none)"
        })
    }
}

/// A planning question out to the model.
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub id: u64,
    pub day: f32,
    pub stage: u8,
    pub asked_tick: u64,
    /// The menu, as event ids in the order the choices were given.
    pub menu: Vec<String>,
    pub situation: Vec<String>,
}

/// A line the model is writing.
#[derive(Clone, Debug, PartialEq)]
pub struct Writing {
    pub id: u64,
    pub event: String,
    pub kind: Kind,
    pub mob: String,
    pub count: u32,
}

/// The planning questions, in the order they are asked.
pub const SOON: &str = "Should it do something soon?";
pub const SOON_CHOICES: [&str; 3] = ["nothing yet", "something small", "something it will notice"];
pub const WHAT: &str = "What should it do?";

/// How long a question may wait before the rule brain answers instead: a minute.
const ASK_PATIENCE_TICKS: u64 = 60 * TPS;

/// One decision as the debug screen tells it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Decision {
    pub day: f32,
    pub stage: u8,
    /// Which brain answered: `rules` today, `model` in phase 4.
    pub brain: String,
    pub menu: Vec<String>,
    pub chosen: String,
    pub pos: Option<BlockPos>,
    /// The situation lines the choice was made on.
    pub situation: Vec<String>,
    /// What came of it: empty until the plan ran or lapsed, then `done`, or why not.
    pub outcome: String,
}

/// How many decisions the story keeps.
const MOST_STORY: usize = 24;

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
        model: Option<&Model>,
    ) -> Vec<Command> {
        let mut commands = Vec::new();

        self.take_replies(now, day, memory, arc, config, log, model);

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

        // The surroundings, once a day around the home, and again when the home moved: the
        // host's pass is sliced over ticks and answers as an observation.
        if let Some(home) = &memory.home {
            let moved = memory
                .surroundings
                .as_ref()
                .map(|s| crate::memory::distance(s.around, home.center) > 16.0)
                .unwrap_or(false);
            if (memory.surroundings_due(day) || moved) && now.tick >= self.next_survey_tick {
                self.next_survey_tick = now.tick + 6_000;
                commands.push(Command::Surroundings {
                    id: self.id(),
                    around: home.center,
                    radius: 96,
                });
            }
        }

        if self.plan.is_none() && self.asking.is_none() && now.tick >= self.next_plan_tick {
            self.next_plan_tick = now.tick + config.brain.plan_every as u64 * TPS;
            self.decide(now, day, memory, arc, config, log, model);
        }

        if let Some(plan) = self.plan.clone() {
            if now.tick > plan.until_tick {
                memory.record(&plan.event, plan.pos, day, false, "its moment never came");
                self.outcome(&plan.event, "its moment never came");
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
                                let written = self
                                    .written
                                    .iter()
                                    .find(|(e, _)| *e == plan.event)
                                    .map(|(_, t)| t.clone());
                                self.written.retain(|(e, _)| *e != plan.event);
                                let mut ids = || self.id();
                                let made = act::commands(
                                    event,
                                    plan.pos,
                                    memory,
                                    &mut ids,
                                    now,
                                    written.as_deref(),
                                );
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
                                if !plan.forced {
                                    self.outcome(
                                        &plan.event,
                                        if made.is_empty() {
                                            "nothing to do it with"
                                        } else {
                                            "sent to the world"
                                        },
                                    );
                                }
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
                                self.outcome(&plan.event, refused.words());
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
        model: Option<&Model>,
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
        let lines = memory.describe(day);

        // The model, when it is there and wanted: the two planning questions go to the worker
        // and the answer comes back on a later tick. Anything in the way, the rule brain.
        if let Some(model) = model.filter(|m| m.loaded() && !config.brain.force_rules) {
            let questions = vec![
                Question {
                    text: SOON.to_string(),
                    choices: SOON_CHOICES.iter().map(|c| c.to_string()).collect(),
                },
                Question {
                    text: WHAT.to_string(),
                    choices: menu.iter().map(|e| e.menu_words().to_string()).collect(),
                },
            ];
            let sender = self.replies().0.clone();
            match model.ask_async(
                &situation::situation(&lines),
                questions,
                Some("plan".to_string()),
                sender,
            ) {
                Ok(id) => {
                    self.asking = Some(Ask {
                        id,
                        day,
                        stage: arc.stage,
                        asked_tick: now.tick,
                        menu: menu.iter().map(|e| e.id.to_string()).collect(),
                        situation: lines,
                    });
                    return;
                }
                Err(err) => log.note(&format!(
                    "the model refused the question ({err}); the rules answer"
                )),
            }
        }

        let chosen = rules_brain::choose(&menu, arc.stage, &mut self.rng);
        let menu_ids: Vec<String> = menu.iter().map(|e| e.id.to_string()).collect();
        self.settle(
            chosen, &menu_ids, lines, "rules", now, day, memory, arc, config, log, model,
        );
    }

    /// The channel the model answers on, made on first use.
    fn replies(&mut self) -> &(Sender<Reply>, Receiver<Reply>) {
        if self.replies.0.is_none() {
            self.replies.0 = Some(channel());
        }
        self.replies.0.as_ref().unwrap()
    }

    /// Reads what the model answered since the last tick: a planning answer becomes the
    /// decision, a written line is kept for its event. A question that waited a minute goes to
    /// the rules instead.
    #[allow(clippy::too_many_arguments)]
    fn take_replies(
        &mut self,
        now: &Now,
        day: f32,
        memory: &Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
        model: Option<&Model>,
    ) {
        let mut received = Vec::new();
        if let Some((_, rx)) = &self.replies.0 {
            while let Ok(reply) = rx.try_recv() {
                received.push(reply);
            }
        }

        for reply in received {
            if self
                .asking
                .as_ref()
                .map(|a| a.id == reply.id)
                .unwrap_or(false)
            {
                let ask = self.asking.take().unwrap();
                match reply.result {
                    Ok(answers) => {
                        self.answered(ask, &answers, now, memory, arc, config, log, model)
                    }
                    Err(err) => {
                        log.note(&format!(
                            "the model did not answer ({err}); the rules answer"
                        ));
                        self.rules_for(ask, now, memory, arc, config, log);
                    }
                }
            } else if self
                .writing
                .as_ref()
                .map(|w| w.id == reply.id)
                .unwrap_or(false)
            {
                let writing = self.writing.take().unwrap();
                let written = reply.text.unwrap_or_default();
                let kept = match &reply.result {
                    Ok(_) => text::check(writing.kind, &written),
                    Err(_) => None,
                };
                log.wrote(&format!("{:?}", writing.kind), &written, kept.as_deref());
                if let Some(line) = kept {
                    self.written.retain(|(e, _)| *e != writing.event);
                    self.written.push((writing.event, line));
                }
            }
        }

        if let Some(ask) = &self.asking {
            if now.tick.saturating_sub(ask.asked_tick) > ASK_PATIENCE_TICKS {
                let ask = self.asking.take().unwrap();
                log.note("the model took too long; the rules answer");
                self.rules_for(ask, now, memory, arc, config, log);
            }
        }
        let _ = day;
    }

    /// The model's two answers made into a decision: `nothing yet`, a confidence under the
    /// threshold, or `nothing` on the menu is `wait`; otherwise the chosen words name the event.
    #[allow(clippy::too_many_arguments)]
    fn answered(
        &mut self,
        ask: Ask,
        answers: &[Answer],
        now: &Now,
        memory: &Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
        model: Option<&Model>,
    ) {
        let menu: Vec<&Event> = ask.menu.iter().filter_map(|id| events::find(id)).collect();
        let Some(wait) = events::find("wait") else {
            return;
        };
        let soon = answers.first();
        let what = answers.get(1);
        let chosen = match (soon, what) {
            (Some(soon), Some(what))
                if soon.choice != SOON_CHOICES[0]
                    && soon.confidence as f64 >= config.brain.event_threshold =>
            {
                menu.iter()
                    .copied()
                    .find(|e| e.menu_words() == what.choice)
                    .unwrap_or(wait)
            }
            _ => wait,
        };
        let confidence = what.map(|a| a.confidence).unwrap_or(0.0);
        log.note(&format!(
            "the model: {} ({:.0}%), {} ({:.0}%)",
            soon.map(|a| a.choice.as_str()).unwrap_or("?"),
            soon.map(|a| a.confidence * 100.0).unwrap_or(0.0),
            what.map(|a| a.choice.as_str()).unwrap_or("?"),
            confidence * 100.0
        ));
        let Ask {
            day,
            menu: menu_ids,
            situation,
            ..
        } = ask;
        self.settle(
            chosen, &menu_ids, situation, "model", now, day, memory, arc, config, log, model,
        );
    }

    fn rules_for(
        &mut self,
        ask: Ask,
        now: &Now,
        memory: &Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
    ) {
        let menu: Vec<&Event> = ask.menu.iter().filter_map(|id| events::find(id)).collect();
        if menu.is_empty() {
            return;
        }
        let chosen = rules_brain::choose(&menu, arc.stage, &mut self.rng);
        let Ask {
            day,
            menu: menu_ids,
            situation,
            ..
        } = ask;
        self.settle(
            chosen, &menu_ids, situation, "rules", now, day, memory, arc, config, log, None,
        );
    }

    /// A decision made, by whichever brain: logged, told in the story, and planned if it is
    /// not `wait`. With a model and a line to write, the writing is asked for at once so it is
    /// there when the plan runs.
    #[allow(clippy::too_many_arguments)]
    fn settle(
        &mut self,
        chosen: &Event,
        menu_ids: &[String],
        lines: Vec<String>,
        brain: &str,
        now: &Now,
        day: f32,
        memory: &Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
        model: Option<&Model>,
    ) {
        self.decisions += 1;
        let pos = act::place(chosen, memory, &mut self.rng);
        let menu: Vec<&Event> = menu_ids.iter().filter_map(|id| events::find(id)).collect();
        log.decided(arc.stage, day, &lines, &menu, chosen.id, pos, brain);
        self.story.push(Decision {
            day,
            stage: arc.stage,
            brain: brain.to_string(),
            menu: menu_ids.to_vec(),
            chosen: chosen.id.to_string(),
            pos,
            situation: lines.clone(),
            outcome: if chosen.id == "wait" {
                "waited".to_string()
            } else {
                String::new()
            },
        });
        if self.story.len() > MOST_STORY {
            self.story.remove(0);
        }

        if chosen.id == "wait" {
            return;
        }

        if let (Some(model), Some(kind), true) = (
            model,
            act::text_kind(chosen),
            config.brain.text == "generate",
        ) {
            let tally = memory.most_killed();
            let mob = tally.map(|t| t.kind.clone()).unwrap_or_default();
            let count = tally.map(|t| t.kills).unwrap_or(0);
            let examples: Vec<String> = kind
                .examples()
                .iter()
                .map(|e| text::fill(e, &mob, count))
                .collect();
            let refs: Vec<&str> = examples.iter().map(String::as_str).collect();
            let prompt = situation::writing_prompt(&lines, kind.purpose(), &refs);
            let sender = self.replies().0.clone();
            if let Ok(id) = model.generate_async(
                &prompt,
                kind.most_words() * 2 + 2,
                Some("text".to_string()),
                sender,
            ) {
                self.writing = Some(Writing {
                    id,
                    event: chosen.id.to_string(),
                    kind,
                    mob,
                    count,
                });
            }
        }

        if !arc.affords(chosen.cost(), now.day, config) {
            log.note("the day's budget is spent");
            self.writing = None;
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

    /// Writes what came of the newest decision for this event that has no final outcome yet.
    fn outcome(&mut self, event: &str, outcome: &str) {
        if let Some(decision) = self.story.iter_mut().rev().find(|d| {
            d.chosen == event && (d.outcome.is_empty() || d.outcome == "sent to the world")
        }) {
            decision.outcome = outcome.to_string();
        }
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
        let made = act::commands(event, pos, memory, &mut ids, &Now::default(), None);
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
                    self.outcome(
                        &pending.event,
                        if report.done { "done" } else { &report.reason },
                    );
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
    use crate::model::{Scorer, Settings, Worker};

    /// A fake model: answers the first question with `soon` and the second with the menu's
    /// `pick`th choice, and writes `written` when asked to write.
    struct Fake {
        soon: &'static str,
        pick: usize,
        written: Result<&'static str, &'static str>,
    }

    impl Scorer for Fake {
        fn score(&mut self, _: &str, questions: &[Question]) -> Result<Vec<Answer>, String> {
            Ok(vec![
                Answer {
                    choice: self.soon.to_string(),
                    confidence: 0.9,
                },
                Answer {
                    choice: questions[1].choices[self.pick.min(questions[1].choices.len() - 1)]
                        .clone(),
                    confidence: 0.6,
                },
            ])
        }

        fn generate(&mut self, _: &str, _: usize) -> Result<String, String> {
            self.written.map(str::to_string).map_err(str::to_string)
        }
    }

    fn model_with(fake: Fake) -> Model {
        let settings = Settings {
            path: std::path::PathBuf::from("fake"),
            threads: 1,
            context: 256,
            temperature: 0.0,
            seed: 1,
        };
        Model::with_worker(settings, Worker::start(move || Ok(fake)).unwrap())
    }

    fn home_memory() -> Memory {
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
        memory
    }

    /// Ticks until the brain has a plan or a decision, with the clock moving `step` ticks.
    fn run_until_decided(
        brain: &mut Brain,
        memory: &mut Memory,
        arc: &mut Arc,
        config: &Config,
        model: Option<&Model>,
        log: &mut log::Log,
    ) -> Now {
        let mut now = Now::default();
        for _ in 0..400 {
            brain.tick(&now, 0.0, memory, arc, config, log, model);
            if brain.decisions > 0 {
                break;
            }
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        now
    }

    #[test]
    fn the_model_s_answer_names_the_event_and_the_story_says_which_brain() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 1;
        let mut memory = home_memory();
        let mut arc = Arc::default();
        arc.tick(0.0, &memory, &config);
        let mut log = log::Log::new(None, 16, true);
        let model = model_with(Fake {
            soon: "something small",
            pick: 1,
            written: Err("nothing"),
        });
        let mut brain = Brain::new(2);

        run_until_decided(
            &mut brain,
            &mut memory,
            &mut arc,
            &config,
            Some(&model),
            &mut log,
        );

        let decision = brain.story.last().expect("a decision");
        assert_eq!(decision.brain, "model");
        assert_ne!(decision.chosen, "wait", "the second menu choice was picked");
        assert_eq!(decision.menu[1], decision.chosen);
        assert!(brain.plan.is_some());
        assert!(
            log.lines.iter().any(|l| l.contains("brain=model")),
            "{:?}",
            log.lines
        );
        model.shutdown();
    }

    #[test]
    fn nothing_yet_from_the_model_is_a_wait_and_a_refusal_falls_back_to_the_rules() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 1;
        let mut memory = home_memory();
        let mut arc = Arc::default();
        arc.tick(0.0, &memory, &config);
        let mut log = log::Log::new(None, 16, true);

        let model = model_with(Fake {
            soon: "nothing yet",
            pick: 1,
            written: Err("nothing"),
        });
        let mut brain = Brain::new(3);
        run_until_decided(
            &mut brain,
            &mut memory,
            &mut arc,
            &config,
            Some(&model),
            &mut log,
        );
        assert_eq!(brain.story.last().unwrap().chosen, "wait");
        assert_eq!(brain.story.last().unwrap().brain, "model");
        model.shutdown();

        // A model that is shut down refuses the question: the rules answer at once.
        let mut brain = Brain::new(4);
        let mut log = log::Log::new(None, 16, true);
        run_until_decided(
            &mut brain,
            &mut memory,
            &mut arc,
            &config,
            Some(&model),
            &mut log,
        );
        assert_eq!(brain.story.last().unwrap().brain, "rules");

        // `brain.force_rules` keeps the model out of it.
        let model = model_with(Fake {
            soon: "something small",
            pick: 1,
            written: Err("nothing"),
        });
        config.brain.force_rules = true;
        let mut brain = Brain::new(5);
        run_until_decided(
            &mut brain,
            &mut memory,
            &mut arc,
            &config,
            Some(&model),
            &mut log,
        );
        assert_eq!(brain.story.last().unwrap().brain, "rules");
        model.shutdown();
    }

    #[test]
    fn a_line_the_model_wrote_goes_on_the_sign_and_a_bad_one_is_dropped_for_the_bank() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 1;
        config.events.disabled = events::CATALOGUE
            .iter()
            .filter(|e| e.id != "wait" && e.id != "sign")
            .map(|e| e.id.to_string())
            .collect();
        let mut memory = home_memory();
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, true);

        let model = model_with(Fake {
            soon: "something it will notice",
            pick: 1,
            written: Ok("Look Under!"),
        });
        let mut brain = Brain::new(6);
        let now = run_until_decided(
            &mut brain,
            &mut memory,
            &mut arc,
            &config,
            Some(&model),
            &mut log,
        );
        assert_eq!(brain.story.last().unwrap().chosen, "sign");
        // The written line arrives on a later tick, cleaned.
        let mut later = now;
        for _ in 0..50 {
            later.tick += TPS;
            brain.tick(
                &later,
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            );
            if !brain.written.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(
            brain.written,
            vec![("sign".to_string(), "look under".to_string())]
        );
        assert!(
            log.lines.iter().any(|l| l.contains("wrote (SignWord)")),
            "{:?}",
            log.lines
        );

        // When the plan runs, the sign carries it.
        let plan = brain.plan.clone().unwrap();
        let mut at = Now {
            tick: plan.from_tick,
            ..later
        };
        let mut sign_lines = None;
        for _ in 0..3 {
            for command in brain.tick(
                &at,
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            ) {
                if let Command::SetSign { lines, .. } = command {
                    sign_lines = Some(lines);
                }
            }
            at.tick += 5;
        }
        // The sign's text is sent two ticks after the block; take it from the later queue.
        for _ in 0..5 {
            for command in brain.tick(
                &at,
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            ) {
                if let Command::SetSign { lines, .. } = command {
                    sign_lines = Some(lines);
                }
            }
            at.tick += 1;
        }
        assert_eq!(sign_lines.expect("a sign").join(" ").trim(), "look under");
        model.shutdown();

        // A bad line is dropped and the bank's word stands.
        let model = model_with(Fake {
            soon: "something it will notice",
            pick: 1,
            written: Ok("you fucking idiot go away now please"),
        });
        let mut brain = Brain::new(7);
        let mut log = log::Log::new(None, 16, true);
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        run_until_decided(
            &mut brain,
            &mut memory,
            &mut arc,
            &config,
            Some(&model),
            &mut log,
        );
        for i in 0..50 {
            brain.tick(
                &Now {
                    tick: 100 + i * TPS,
                    ..Now::default()
                },
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            );
            if log.lines.iter().any(|l| l.contains("dropped")) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(brain.written.is_empty());
        assert!(
            log.lines.iter().any(|l| l.contains("-> dropped")),
            "{:?}",
            log.lines
        );
        model.shutdown();
    }

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
            brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
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
