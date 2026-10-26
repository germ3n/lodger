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
    /// The model's `When?`: `away`, `sleep`, `home` (as they come home), `now`, or empty for
    /// whenever the needs hold.
    #[serde(default)]
    pub when: String,
    /// Why it has not happened yet though its moment came (`in view`), for the debug screen;
    /// empty while it waits for its moment or its conditions.
    #[serde(default)]
    pub waiting: String,
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
    /// The last tick a reaction was asked, so triggers ten seconds apart cost one ask.
    #[serde(default)]
    pub last_react_tick: u64,
    /// The tick a deferred target was first waited for.
    #[serde(default)]
    pub waiting_for_target_since: u64,
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
    /// The places `Where?` offered, by words, in the order given.
    pub places: Vec<(String, BlockPos)>,
    /// `plan`, `react`, `want` or `witness`.
    pub kind: String,
    /// For a `react`: what set it off.
    pub trigger: String,
}

/// A line the model is writing.
#[derive(Clone, Debug, PartialEq)]
pub struct Writing {
    pub id: u64,
    pub event: String,
    pub kind: Kind,
    pub mob: String,
    pub count: u32,
    /// A `bank` ask: the answer is a choice among bank lines, not written text.
    pub bank: bool,
}

/// The planning questions, in the order they are asked.
pub const SOON: &str = "Should it do something soon?";
pub const SOON_CHOICES: [&str; 3] = ["nothing yet", "something small", "something it will notice"];
pub const WHAT: &str = "What should it do?";
pub const WHEN: &str = "When?";
pub const WHEN_CHOICES: [&str; 4] = [
    "while they are away",
    "while they sleep",
    "as they come home",
    "now",
];
pub const WHERE: &str = "Where?";
pub const FEEL: &str = "How does the player feel?";
pub const FEEL_CHOICES: [&str; 5] = ["calm", "curious", "uneasy", "afraid", "angry"];
pub const ANSWER: &str = "Should it answer this?";
pub const ANSWER_CHOICES: [&str; 4] = [
    "no",
    "with something small",
    "by showing itself",
    "by giving back",
];
pub const WANT: &str = "What does it want from this player?";
pub const WANT_CHOICES: [&str; 3] = [
    "a place of its own",
    "to be looked at",
    "one thing of theirs",
];
pub const WITNESS: &str = "Which player does it move in with?";
pub const ORDINALS: [&str; 8] = [
    "the first",
    "the second",
    "the third",
    "the fourth",
    "the fifth",
    "the sixth",
    "the seventh",
    "the eighth",
];
pub const SIGN_SAYS: &str = "What does the sign say?";
/// At least this long between reactions: ten seconds.
const REACT_EVERY_TICKS: u64 = 10 * TPS;
/// A deferred target is taken without the model after this long alone: five minutes.
const ALONE_TICKS: u64 = 300 * TPS;

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
    #[allow(clippy::too_many_arguments)]
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

        // The target, when it is the brain's to pick (multiplayer with `multiplayer.pick` at
        // `model`): the model once there are two to choose from, the first alone after a while.
        if memory.target.is_none() && memory.defer_target && self.asking.is_none() {
            if self.waiting_for_target_since == 0 {
                self.waiting_for_target_since = now.tick.max(1);
            }
            let others = memory.others.clone();
            let alone = now.tick.saturating_sub(self.waiting_for_target_since) > ALONE_TICKS;
            if others.len() >= 2 || alone {
                let asked = others.len() >= 2
                    && self.ask_once(
                        model,
                        config,
                        "witness",
                        WITNESS,
                        ORDINALS
                            .iter()
                            .take(others.len().min(8))
                            .map(|o| o.to_string())
                            .collect(),
                        others
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                format!(
                                    "Player {} is here.",
                                    ORDINALS[i.min(7)].trim_start_matches("the ")
                                )
                            })
                            .collect(),
                        now,
                        day,
                        arc.stage,
                        log,
                    );
                if !asked {
                    if let Some((index, name, uuid)) = others.first().cloned() {
                        memory.set_target(index, &name, &uuid);
                        log.note(&format!("the target is {name}, the first seen"));
                    }
                }
            }
            return commands;
        }

        // The want, once, at move-in.
        if memory.want.is_empty() && memory.target.is_some() && self.asking.is_none() {
            let asked = self.ask_once(
                model,
                config,
                "want",
                WANT,
                WANT_CHOICES.iter().map(|c| c.to_string()).collect(),
                memory.describe(day),
                now,
                day,
                arc.stage,
                log,
            );
            if !asked {
                let pick = WANT_CHOICES[(self.rng.next_u64() % 3) as usize];
                memory.want = pick.to_string();
                log.note(&format!("it wants {pick} (the rules' pick)"));
            }
        }

        // A reaction, on a trigger, ten seconds apart at most, when nothing else is asked.
        let may_react = self.asking.is_none()
            && self.plan.is_none()
            && (self.last_react_tick == 0
                || now.tick.saturating_sub(self.last_react_tick) >= REACT_EVERY_TICKS);
        if may_react && !memory.triggers.is_empty() {
            let trigger = memory.triggers.remove(0);
            {
                self.last_react_tick = now.tick.max(1);
                let mut lines = memory.describe(day);
                lines.push(format!("Just now: {}.", trigger_words(&trigger)));
                if let Some(model) = model.filter(|m| m.loaded() && !config.brain.force_rules) {
                    let questions = vec![
                        Question {
                            text: FEEL.to_string(),
                            choices: FEEL_CHOICES.iter().map(|c| c.to_string()).collect(),
                        },
                        Question {
                            text: ANSWER.to_string(),
                            choices: ANSWER_CHOICES.iter().map(|c| c.to_string()).collect(),
                        },
                    ];
                    let sender = self.replies().0.clone();
                    if let Ok(id) = model.ask_async(
                        &situation::situation(&lines),
                        questions,
                        Some("react".to_string()),
                        sender,
                    ) {
                        self.asking = Some(Ask {
                            id,
                            day,
                            stage: arc.stage,
                            asked_tick: now.tick,
                            menu: Vec::new(),
                            situation: lines,
                            places: Vec::new(),
                            kind: "react".to_string(),
                            trigger,
                        });
                    }
                }
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
                    if plan.forced
                        || (act::ready(event, memory, config) && when_holds(&plan.when, memory))
                    {
                        let checked = if plan.forced {
                            Ok(())
                        } else {
                            rules::check(event, plan.pos, memory, arc.stage, day, config)
                        };
                        let written = self
                            .written
                            .iter()
                            .find(|(e, _)| *e == plan.event)
                            .map(|(_, t)| t.clone());
                        let made = match checked {
                            Ok(()) => {
                                let mut ids = || self.id();
                                act::commands(
                                    event,
                                    plan.pos,
                                    memory,
                                    &mut ids,
                                    now,
                                    written.as_deref(),
                                )
                            }
                            Err(_) => Vec::new(),
                        };
                        // Digging near what they built is refused whoever asked, the debug
                        // screen's Fire too: the host keeps a placed block itself from being
                        // replaced, but not the ground under it.
                        if let Some(dug) = made.iter().find_map(|(command, _)| {
                            act::dug(command).filter(|p| !act::clear_of_built(memory, *p))
                        }) {
                            let why = "too near what they built";
                            log.note(&format!(
                                "{} would dig at {}: {why}",
                                plan.event,
                                log::spot_words(dug)
                            ));
                            memory.record(&plan.event, plan.pos, day, false, why);
                            self.outcome(&plan.event, why);
                            self.plan = None;
                            return commands;
                        }
                        // In view is a reason to wait, not to give up: every block it would
                        // change, not just its spot, stays out of sight until they look away,
                        // within the plan's day.
                        let seen = !plan.forced
                            && (checked == Err(rules::Refused::InView)
                                || (checked.is_ok()
                                    && rules::hidden_from_view(event.action)
                                    && made.iter().any(|(command, _)| {
                                        act::spots(command).iter().any(|spot| {
                                            memory.in_view(*spot, config.events.min_distance as f64)
                                        })
                                    })));
                        if seen {
                            if let Some(plan) = &mut self.plan {
                                plan.waiting = "in view".to_string();
                            }
                            return commands;
                        }
                        match checked {
                            Ok(()) => {
                                self.written.retain(|(e, _)| *e != plan.event);
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

    #[allow(clippy::too_many_arguments)]
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
        // Nothing but `wait` on offer is nothing to decide: no ask, no decision logged. The
        // dev client asked the model every interval for a menu of one (2026-11-09).
        if menu.iter().all(|e| e.id == "wait") {
            return;
        }
        let lines = memory.describe(day);

        // The model, when it is there and wanted: the two planning questions go to the worker
        // and the answer comes back on a later tick. Anything in the way, the rule brain.
        if let Some(model) = model.filter(|m| m.loaded() && !config.brain.force_rules) {
            let places = act::named_places(memory);
            let mut questions = vec![
                Question {
                    text: SOON.to_string(),
                    choices: SOON_CHOICES.iter().map(|c| c.to_string()).collect(),
                },
                Question {
                    text: WHAT.to_string(),
                    choices: menu.iter().map(|e| e.menu_words().to_string()).collect(),
                },
                Question {
                    text: WHEN.to_string(),
                    choices: WHEN_CHOICES.iter().map(|c| c.to_string()).collect(),
                },
            ];
            if places.len() >= 2 {
                questions.push(Question {
                    text: WHERE.to_string(),
                    choices: places.iter().map(|(w, _)| w.to_string()).collect(),
                });
            }
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
                        places: places.iter().map(|(w, p)| (w.to_string(), *p)).collect(),
                        kind: "plan".to_string(),
                        trigger: String::new(),
                    });
                    return;
                }
                Err(err) => log.note(&format!(
                    "the model refused the question ({err}); the rules answer"
                )),
            }
        }

        let chosen = rules_brain::choose_wanting(&menu, arc.stage, &memory.want, &mut self.rng);
        let menu_ids: Vec<String> = menu.iter().map(|e| e.id.to_string()).collect();
        self.settle(
            chosen, &menu_ids, lines, "rules", now, day, memory, arc, config, log, model, None,
            None,
        );
    }

    /// One question with its choices to the model, when it is there and wanted; `true` when
    /// asked, so the caller falls back when not.
    #[allow(clippy::too_many_arguments)]
    fn ask_once(
        &mut self,
        model: Option<&Model>,
        config: &Config,
        kind: &str,
        question: &str,
        choices: Vec<String>,
        lines: Vec<String>,
        now: &Now,
        day: f32,
        stage: u8,
        log: &mut log::Log,
    ) -> bool {
        let Some(model) = model.filter(|m| m.loaded() && !config.brain.force_rules) else {
            return false;
        };
        let sender = self.replies().0.clone();
        match model.ask_async(
            &situation::situation(&lines),
            vec![Question {
                text: question.to_string(),
                choices,
            }],
            Some(kind.to_string()),
            sender,
        ) {
            Ok(id) => {
                self.asking = Some(Ask {
                    id,
                    day,
                    stage,
                    asked_tick: now.tick,
                    menu: Vec::new(),
                    situation: lines,
                    places: Vec::new(),
                    kind: kind.to_string(),
                    trigger: String::new(),
                });
                true
            }
            Err(err) => {
                log.note(&format!("the model refused the {kind} question ({err})"));
                false
            }
        }
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
        memory: &mut Memory,
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
                match (ask.kind.as_str(), reply.result) {
                    ("plan", Ok(answers)) => {
                        self.answered(ask, &answers, now, memory, arc, config, log, model)
                    }
                    ("plan", Err(err)) => {
                        log.note(&format!(
                            "the model did not answer ({err}); the rules answer"
                        ));
                        self.rules_for(ask, now, memory, arc, config, log);
                    }
                    ("react", Ok(answers)) => {
                        self.reacted(ask, &answers, now, memory, arc, config, log, model)
                    }
                    ("want", Ok(answers)) => {
                        let want = answers
                            .first()
                            .map(|a| a.choice.clone())
                            .filter(|c| WANT_CHOICES.contains(&c.as_str()))
                            .unwrap_or_else(|| WANT_CHOICES[0].to_string());
                        log.note(&format!("it wants {want} (the model's pick)"));
                        memory.want = want;
                    }
                    ("witness", Ok(answers)) => {
                        let index = answers
                            .first()
                            .and_then(|a| ORDINALS.iter().position(|o| *o == a.choice))
                            .unwrap_or(0);
                        if let Some((i, name, uuid)) = memory.others.get(index).cloned() {
                            log.note(&format!("the target is {name} (the model's pick)"));
                            memory.set_target(i, &name, &uuid);
                        }
                    }
                    (kind, Err(err)) => {
                        log.note(&format!(
                            "the model did not answer the {kind} question ({err})"
                        ));
                        if kind == "want" {
                            memory.want =
                                WANT_CHOICES[(self.rng.next_u64() % 3) as usize].to_string();
                        }
                    }
                    (_, Ok(_)) => {}
                }
            } else if self
                .writing
                .as_ref()
                .map(|w| w.id == reply.id)
                .unwrap_or(false)
            {
                let writing = self.writing.take().unwrap();
                let written = if writing.bank {
                    reply
                        .result
                        .as_ref()
                        .ok()
                        .and_then(|answers| answers.first())
                        .map(|a| a.choice.clone())
                        .unwrap_or_default()
                } else {
                    reply.text.unwrap_or_default()
                };
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
        let when = answers
            .get(2)
            .map(|a| match a.choice.as_str() {
                "while they are away" => "away",
                "while they sleep" => "sleep",
                "as they come home" => "home",
                "now" => "now",
                _ => "",
            })
            .unwrap_or("")
            .to_string();
        let place = answers
            .get(3)
            .and_then(|a| ask.places.iter().find(|(w, _)| *w == a.choice))
            .map(|(_, p)| *p);
        let Ask {
            day,
            menu: menu_ids,
            situation,
            ..
        } = ask;
        self.settle(
            chosen,
            &menu_ids,
            situation,
            "model",
            now,
            day,
            memory,
            arc,
            config,
            log,
            model,
            Some(when),
            place,
        );
    }

    /// The model's reading of a trigger: the mood is kept, and an answer above the bar becomes
    /// a small event now, the far figure, or the held thing given back.
    #[allow(clippy::too_many_arguments)]
    fn reacted(
        &mut self,
        ask: Ask,
        answers: &[Answer],
        now: &Now,
        memory: &mut Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
        model: Option<&Model>,
    ) {
        if let Some(feel) = answers.first() {
            if FEEL_CHOICES.contains(&feel.choice.as_str()) {
                memory.last.mood = feel.choice.clone();
            }
        }
        let Some(answer) = answers.get(1) else { return };
        log.note(&format!(
            "the model, after {}: they feel {} ({:.0}%), answer {} ({:.0}%)",
            ask.trigger,
            memory.last.mood,
            answers[0].confidence * 100.0,
            answer.choice,
            answer.confidence * 100.0
        ));
        if (answer.confidence as f64) < config.brain.react_threshold || answer.choice == "no" {
            return;
        }
        let menu = offer::offer(
            events::CATALOGUE,
            arc.stage,
            memory,
            now,
            ask.day,
            config,
            &mut self.rng,
        );
        let wanted = match answer.choice.as_str() {
            "with something small" => menu.iter().copied().find(|e| e.small && e.id != "wait"),
            "by showing itself" => menu.iter().copied().find(|e| e.id == "far_figure"),
            "by giving back" => menu
                .iter()
                .copied()
                .find(|e| e.id == "place_taken" || e.id == "return_taken"),
            _ => None,
        };
        let Some(chosen) = wanted else {
            log.note("nothing on the menu answers it");
            return;
        };
        let menu_ids: Vec<String> = menu.iter().map(|e| e.id.to_string()).collect();
        let Ask { day, situation, .. } = ask;
        self.settle(
            chosen,
            &menu_ids,
            situation,
            "model/react",
            now,
            day,
            memory,
            arc,
            config,
            log,
            model,
            Some("now".to_string()),
            None,
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
        let chosen = rules_brain::choose_wanting(&menu, arc.stage, &memory.want, &mut self.rng);
        let Ask {
            day,
            menu: menu_ids,
            situation,
            ..
        } = ask;
        self.settle(
            chosen, &menu_ids, situation, "rules", now, day, memory, arc, config, log, None, None,
            None,
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
        when: Option<String>,
        place: Option<BlockPos>,
    ) {
        self.decisions += 1;
        let pos = match place {
            Some(p) if act::flexible(chosen) => Some(p),
            _ => act::place(chosen, memory, &mut self.rng),
        };
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

        if let (Some(model), Some(kind)) = (model, act::text_kind(chosen)) {
            let tally = memory.most_killed();
            let mob = tally.map(|t| t.kind.clone()).unwrap_or_default();
            let count = tally.map(|t| t.kills).unwrap_or(0);
            let examples: Vec<String> = kind
                .examples()
                .iter()
                .map(|e| text::fill(e, &mob, count))
                .collect();
            let sender = self.replies().0.clone();
            let asked = if config.brain.text == "generate" {
                let refs: Vec<&str> = examples.iter().map(String::as_str).collect();
                let prompt = situation::writing_prompt(&lines, kind.purpose(), &refs);
                model.generate_async(&prompt, kind.leash(), Some("text".to_string()), sender)
            } else {
                // `bank`: the model picks among eight of the kind's lines.
                let choices: Vec<String> = examples.iter().take(8).cloned().collect();
                model.ask_async(
                    &situation::situation(&lines),
                    vec![Question {
                        text: format!("{} {}", kind.purpose(), SIGN_SAYS),
                        choices,
                    }],
                    Some("text".to_string()),
                    sender,
                )
            };
            if let Ok(id) = asked {
                self.writing = Some(Writing {
                    id,
                    event: chosen.id.to_string(),
                    kind,
                    mob,
                    count,
                    bank: config.brain.text != "generate",
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
        let when = when.unwrap_or_default();
        // The model answers `When?` before it knows the event, so a moment the event's needs
        // rule out is dropped: `wall_swap` (away or asleep) planned "as they come home" waited
        // its whole day in the dev client (2026-11-09).
        let when = if when_fits(chosen, &when) {
            when
        } else {
            log.note(&format!(
                "{} cannot happen {}; whenever its needs hold",
                chosen.id,
                when_words(&when)
            ));
            String::new()
        };
        let delay = if when == "now" {
            0
        } else {
            self.rng.next_u64() % (4 * config.brain.plan_every as u64 * TPS).max(1)
        };
        self.plan = Some(Plan {
            event: chosen.id.to_string(),
            pos,
            from_tick: now.tick + delay,
            until_tick: now.tick + delay + 24_000,
            decided_day: day,
            forced: false,
            when,
            waiting: String::new(),
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
            when: String::new(),
            waiting: String::new(),
        });
        log.note(&format!("forced {}", event.id));

        match pos {
            Some(p) => format!(
                "{} queued at {}, {} command(s)",
                event.id,
                log::spot_words(p),
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
/// Whether the plan's `When?` holds right now. `away` is the target away or asleep; `sleep`
/// asleep; `home` within 24 blocks of the home and walking toward it; `now` and empty always.
/// Whether a planned moment can ever hold together with the event's needs: "as they come home"
/// never meets "away or asleep", "while they sleep" never meets "walking" or "outdoors", and
/// "while they are away" never meets "in bed" or "indoors" (the home's rooms).
pub fn when_fits(event: &Event, when: &str) -> bool {
    use events::Need::*;
    let rules_out: &[events::Need] = match when {
        "home" => &[TargetAwayOrAsleep, TargetInBed],
        "sleep" => &[TargetWalking, TargetOutdoors],
        "away" => &[TargetInBed, TargetIndoors],
        _ => &[],
    };

    !event.needs.iter().any(|n| rules_out.contains(n))
}

/// A planned moment in the log's words.
fn when_words(when: &str) -> &str {
    match when {
        "away" => "while they are away",
        "sleep" => "while they sleep",
        "home" => "as they come home",
        other => other,
    }
}

pub fn when_holds(when: &str, memory: &Memory) -> bool {
    let home = memory.home.as_ref().map(|h| h.center);
    match when {
        "away" => home
            .map(|h| memory.target_away_from(h, 24.0))
            .unwrap_or(!memory.last.online),
        "sleep" => memory.last.online && memory.last.sleeping,
        "home" => home.map(|h| memory.approaching(h, 24.0)).unwrap_or(false),
        _ => true,
    }
}

/// A trigger in the situation's words.
fn trigger_words(trigger: &str) -> &str {
    match trigger {
        "slept" => "the player went to bed",
        "opened_chest" => "the player opened the chest they use",
        "died" => "the player died",
        "lit" => "the player lit up the place where it acted",
        "walled" => "the player walled in the place where it acted",
        "watched" => "the player stood watching the place where it acted",
        "left" => "the player left the game right after it acted",
        other => other,
    }
}

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
    use std::sync::Mutex;

    type Seen = std::sync::Arc<Mutex<Vec<String>>>;

    /// A fake model: answers `Should it do something soon?` with `soon`, the menu question with
    /// its `pick`th choice, any other question with the choice in `by_question` for its text
    /// (or its first choice), and writes `written` when asked to write.
    struct Fake {
        soon: &'static str,
        pick: usize,
        written: Result<&'static str, &'static str>,
        by_question: Vec<(&'static str, &'static str)>,
        seen: Seen,
    }

    impl Fake {
        fn new(
            soon: &'static str,
            pick: usize,
            written: Result<&'static str, &'static str>,
        ) -> Self {
            Self {
                soon,
                pick,
                written,
                by_question: Vec::new(),
                seen: std::sync::Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn answering(mut self, question: &'static str, choice: &'static str) -> Self {
            self.by_question.push((question, choice));
            self
        }
    }

    impl Scorer for Fake {
        fn score(
            &mut self,
            situation: &str,
            questions: &[Question],
        ) -> Result<Vec<Answer>, String> {
            self.seen.lock().unwrap().push(situation.to_string());
            Ok(questions
                .iter()
                .map(|q| {
                    let choice = if q.text == SOON {
                        self.soon.to_string()
                    } else if q.text == WHAT {
                        q.choices[self.pick.min(q.choices.len() - 1)].clone()
                    } else if let Some((_, c)) =
                        self.by_question.iter().find(|(t, _)| q.text.starts_with(t))
                    {
                        c.to_string()
                    } else {
                        q.choices[0].clone()
                    };
                    Answer {
                        choice,
                        confidence: 0.9,
                    }
                })
                .collect())
        }

        fn generate(&mut self, _: &str, _: crate::model::Leash) -> Result<String, String> {
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
        let model = model_with(Fake::new("something small", 1, Err("nothing")));
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

        let model = model_with(Fake::new("nothing yet", 1, Err("nothing")));
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
        let model = model_with(Fake::new("something small", 1, Err("nothing")));
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
    fn when_and_where_from_the_model_shape_the_plan_and_the_want_is_asked_once() {
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
        memory.containers.push(crate::memory::Container {
            pos: [2, 64, 0],
            kind: "minecraft:chest".to_string(),
            opens: 3,
            placed: true,
            days: 1,
            last_opened_day: 0.0,
            contents: vec![],
        });
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, true);
        let fake = Fake::new("something it will notice", 1, Err("no"))
            .answering("When?", "while they sleep")
            .answering("Where?", "on the bed")
            .answering("What does it want", "to be looked at");
        let seen = fake.seen.clone();
        let model = model_with(fake);
        let mut brain = Brain::new(8);

        // The want is asked first, once, and answered on a later tick.
        let mut now = Now::default();
        for _ in 0..200 {
            brain.tick(
                &now,
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            );
            if !memory.want.is_empty() && brain.plan.is_some() {
                break;
            }
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            seen.lock()
                .unwrap()
                .iter()
                .filter(|s| s.contains("What you know"))
                .count()
                >= 2
        );
        let plan = brain.plan.as_ref().expect("a plan");
        assert_eq!(plan.event, "sign");
        assert_eq!(plan.when, "sleep");
        assert_eq!(plan.pos, Some([0, 64, 0]), "the model's place, on the bed");
        assert!(
            !when_holds("sleep", &memory),
            "the target is offline, not asleep"
        );
        memory.last.online = true;
        memory.last.sleeping = true;
        assert!(when_holds("sleep", &memory));
        model.shutdown();
    }

    #[test]
    fn a_trigger_asks_how_they_feel_and_a_small_answer_happens_now() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 600;
        let mut memory = home_memory();
        memory.want = "a place of its own".to_string();
        memory.last.online = true;
        memory.last.indoors = true;
        memory.last.pos = [1.0, 64.0, 1.0];
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, true);
        let fake = Fake::new("nothing yet", 0, Err("no"))
            .answering("How does the player feel?", "uneasy")
            .answering("Should it answer this?", "with something small");
        let model = model_with(fake);
        let mut brain = Brain::new(9);
        memory.triggers.push("slept".to_string());

        let mut now = Now {
            tick: 100,
            ..Now::default()
        };
        for _ in 0..100 {
            brain.tick(
                &now,
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            );
            if brain.plan.is_some() {
                break;
            }
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(memory.last.mood, "uneasy");
        // The answer was planned for now and acted on at once, so the story holds it.
        let reaction = brain
            .story
            .iter()
            .find(|d| d.brain == "model/react")
            .unwrap_or_else(|| panic!("a reaction in the story; log: {:?}", log.lines));
        assert!(
            events::find(&reaction.chosen).unwrap().small,
            "{}",
            reaction.chosen
        );
        assert_eq!(reaction.outcome, "sent to the world");
        assert!(
            reaction.situation.iter().any(|l| l.contains("went to bed")),
            "{:?}",
            reaction.situation
        );
        assert!(memory
            .describe(0.0)
            .iter()
            .any(|l| l.contains("seems uneasy")));
        model.shutdown();
    }

    #[test]
    fn a_deferred_target_is_the_model_s_pick_among_two_players() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        let mut memory = Memory {
            defer_target: true,
            ..Memory::default()
        };
        memory.others.push((0, "ann".to_string(), "u0".to_string()));
        memory.others.push((1, "bob".to_string(), "u1".to_string()));
        let mut arc = Arc {
            stage: 1,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, true);
        let model = model_with(
            Fake::new("nothing yet", 0, Err("no")).answering("Which player", "the second"),
        );
        let mut brain = Brain::new(10);
        let mut now = Now::default();
        for _ in 0..100 {
            brain.tick(
                &now,
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            );
            if memory.target.is_some() {
                break;
            }
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(memory.target, Some(1));
        assert_eq!(memory.target_name, "bob");
        model.shutdown();

        // Without a model, alone for five minutes, the first seen is it.
        let mut memory = Memory {
            defer_target: true,
            ..Memory::default()
        };
        memory.others.push((4, "cy".to_string(), "u4".to_string()));
        let mut brain = Brain::new(11);
        brain.tick(
            &Now {
                tick: 1,
                ..Now::default()
            },
            0.0,
            &mut memory,
            &mut arc,
            &config,
            &mut log,
            None,
        );
        assert_eq!(memory.target, None);
        brain.tick(
            &Now {
                tick: 2 + ALONE_TICKS,
                ..Now::default()
            },
            0.0,
            &mut memory,
            &mut arc,
            &config,
            &mut log,
            None,
        );
        assert_eq!(memory.target, Some(4));
    }

    #[test]
    fn in_bank_mode_the_model_picks_the_sign_s_line_among_the_bank_s() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 1;
        config.brain.text = "bank".to_string();
        config.events.disabled = events::CATALOGUE
            .iter()
            .filter(|e| e.id != "wait" && e.id != "sign")
            .map(|e| e.id.to_string())
            .collect();
        let mut memory = home_memory();
        memory.want = "a place of its own".to_string();
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, true);
        let model = model_with(
            Fake::new("something it will notice", 1, Err("no"))
                .answering("The sign it puts up", "under"),
        );
        let mut brain = Brain::new(12);
        let mut now = Now::default();
        for _ in 0..200 {
            brain.tick(
                &now,
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
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(
            brain.written,
            vec![("sign".to_string(), "under".to_string())]
        );
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

        let model = model_with(Fake::new("something it will notice", 1, Ok("Look Under!")));
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
        let model = model_with(Fake::new(
            "something it will notice",
            1,
            Ok("you fucking idiot go away now please"),
        ));
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
    fn a_plan_whose_torches_they_would_see_waits_until_they_look_away() {
        // The dev client, 2026-11-09: a path of torches went up 40 blocks off, in plain sight.
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        let mut memory = home_memory();
        memory.place = Some([40, 64, 0]);
        memory.last.online = true;
        memory.last.pos = [-30.0, 64.0, 0.5];
        memory.last.yaw = -90.0; // facing +x: the house and the path beyond it
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        brain.plan = Some(Plan {
            event: "light_path".to_string(),
            until_tick: 24_000,
            ..Plan::default()
        });
        assert!(
            rules::check(
                events::find("light_path").unwrap(),
                None,
                &memory,
                2,
                0.0,
                &config
            )
            .is_ok(),
            "no spot of its own: the rule alone would let it through"
        );

        let mut now = Now {
            tick: 20,
            ..Now::default()
        };
        let sent = brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert!(
            !sent.iter().any(|c| matches!(c, Command::SetBlock { .. })),
            "no torch while they look: {sent:?}"
        );
        let plan = brain.plan.as_ref().expect("still planned, not dropped");
        assert_eq!(plan.waiting, "in view");

        memory.last.yaw = 90.0; // facing -x, away from it all
        now.tick += 20;
        let sent = brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert!(
            brain.plan.is_none() && sent.iter().any(|c| matches!(c, Command::SetBlock { .. })),
            "it went ahead once they looked away: {sent:?}"
        );
    }

    #[test]
    fn a_spot_in_view_at_the_moment_keeps_the_plan_waiting_and_a_sound_never_waits() {
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        let mut memory = home_memory();
        memory.last.online = true;
        memory.last.pos = [-30.0, 64.0, 0.5];
        memory.last.yaw = -90.0;
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        brain.plan = Some(Plan {
            event: "close_door".to_string(),
            pos: Some([3, 64, 0]),
            until_tick: 24_000,
            ..Plan::default()
        });
        let now = Now {
            tick: 20,
            ..Now::default()
        };
        brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert_eq!(
            brain.plan.as_ref().map(|p| p.waiting.as_str()),
            Some("in view"),
            "33 blocks off and facing it: once dropped, now waiting"
        );

        assert!(!rules::hidden_from_view(
            events::find("late_steps").unwrap().action
        ));
        assert!(rules::hidden_from_view(
            events::find("light_path").unwrap().action
        ));
    }

    #[test]
    fn a_moment_the_event_s_needs_rule_out_is_dropped_for_whenever_they_hold() {
        // The dev client, 2026-11-09: `wall_swap` planned "as they come home" never came.
        let swap = events::find("wall_swap").unwrap();
        assert!(!when_fits(swap, "home"));
        assert!(when_fits(swap, "sleep"));
        assert!(when_fits(swap, "away"));
        assert!(when_fits(swap, ""));
        assert!(!when_fits(events::find("open_chest").unwrap(), "away"));
        assert!(!when_fits(events::find("late_steps").unwrap(), "sleep"));
        for event in events::CATALOGUE {
            assert!(
                when_fits(event, "") && when_fits(event, "now"),
                "{}",
                event.id
            );
        }
    }

    #[test]
    fn a_menu_of_nothing_but_wait_asks_nothing_and_decides_nothing() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 1;
        config.events.disabled = events::CATALOGUE
            .iter()
            .filter(|e| e.id != "wait")
            .map(|e| e.id.to_string())
            .collect();
        let mut memory = home_memory();
        memory.want = "a place of its own".to_string();
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, true);
        let fake = Fake::new("something it will notice", 0, Err("no"));
        let seen = fake.seen.clone();
        let model = model_with(fake);
        let mut brain = Brain::new(3);
        let mut now = Now::default();
        for _ in 0..30 {
            brain.tick(
                &now,
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                Some(&model),
            );
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        assert_eq!(brain.decisions, 0);
        assert!(seen.lock().unwrap().is_empty(), "the model was asked");
        model.shutdown();
    }

    #[test]
    fn a_hole_next_to_what_they_built_digs_nothing_even_when_forced() {
        // The dev client, 2026-11-09: `hole` dug under a crafting table at the lodger's place.
        let config = Config::default();
        let mut memory = home_memory();
        memory.placed.push([340, 64, 50]);
        let mut arc = Arc {
            stage: 3,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        for forced in [false, true] {
            let mut brain = Brain::new(1);
            brain.plan = Some(Plan {
                event: "hole".to_string(),
                pos: Some([343, 64, 50]),
                until_tick: 24_000,
                forced,
                ..Plan::default()
            });
            let sent = brain.tick(
                &Now {
                    tick: 20,
                    ..Now::default()
                },
                0.0,
                &mut memory,
                &mut arc,
                &config,
                &mut log,
                None,
            );
            assert!(
                !sent.iter().any(|c| act::dug(c).is_some()),
                "forced {forced}: {sent:?}"
            );
            assert!(brain.plan.is_none(), "refused, not waiting");
            assert_eq!(
                memory.done.last().map(|d| d.reason.as_str()),
                Some("too near what they built")
            );
        }
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
