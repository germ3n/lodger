//! How the lodger decides: the menu offered (`offer`), the pick (the model, or `rules_brain`
//! without it), the plan it becomes, the commands the plan turns into (`act`), and the log of
//! every decision (`log`). The `Brain` lives in a `Stay` and is driven once a tick.

pub mod act;
pub mod log;
pub mod offer;
pub mod rules_brain;
pub mod situation;
pub mod sounds;
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
    /// Its number in this stay, for the debug screen's Drop and Now.
    pub id: u64,
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
    /// Forced, but still kept out of their sight and away from witnesses: what happened while
    /// they were gone (`busy_away`) appeared in front of them as they came back.
    #[serde(default)]
    pub out_of_sight: bool,
    /// The model's `When?`: `away`, `sleep`, `home` (as they come home), `now`, or empty for
    /// whenever the needs hold.
    #[serde(default)]
    pub when: String,
    /// Why it has not happened yet, for the debug screen: `in view`, `waiting: not night`,
    /// `waiting while they sleep`; empty before its moment.
    #[serde(default)]
    pub waiting: String,
    /// What the memory knew at its spot when it was planned (`door`, `chest`, `bed`:
    /// `Memory::thing_at`); gone, the plan is cancelled.
    #[serde(default)]
    pub on: String,
    /// The model was asked for its line, or it needs none: a forced plan asks on its first
    /// tick and waits three seconds for it.
    #[serde(default)]
    pub words_asked: bool,
}

/// What a plan did on a tick.
enum Step {
    /// Still waiting, and why.
    Wait(String),
    /// Done with, carried out or not: it leaves the list.
    Gone,
    /// Carried out: these commands go now.
    Sent(Vec<Command>),
}

/// Events whose commands are spots of their own (a torch each, a block of ice each): one with no
/// footing is skipped and the rest go on, where the rest of any other event is dropped (a path
/// of seven torches stopped at the fourth over water). The event is done if any spot was.
const SPOT_BY_SPOT: &[&str] = &[
    "light_path",
    "ice_path",
    "village_doors",
    "candles_lit",
    "torches_far_end",
    "lights_out",
    "face_the_house",
    "hostiles_stare",
    "frames_turn",
];

/// A command this long after its event's first is its tail (`Pending::tail`): a quarter day.
const TAIL_TICKS: u64 = 6_000;

/// A command sent and not yet reported.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pending {
    pub id: u64,
    pub event: String,
    pub pos: Option<BlockPos>,
    pub day: f32,
    /// It puts what it holds somewhere (its house's chest, its cave room): held no more once
    /// it is done.
    #[serde(default)]
    pub puts_held: bool,
    /// Sent after the event is recorded: a taken thing put back after the event failed, a
    /// villager walked back by morning. Its report is not the event's, and the event does not
    /// wait for it (the night's villager kept the event unrecorded for half a day).
    #[serde(default)]
    pub tail: bool,
    /// A take's chest, slot and item, and the name it keeps it under.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub took: Option<Took>,
    /// A put of something of theirs into a chest (`from`): dropped there if it cannot go in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back: Option<Took>,
}

/// What a take took: from where, which item, kept under what name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Took {
    pub from: BlockPos,
    pub slot: u32,
    pub item: String,
    /// The name it is kept under; empty, spent (an ingredient, a sapling), and given back as
    /// plain items of its kind if the event fails.
    pub keep: String,
    pub count: u32,
}

/// A take done for an event not over yet: put back into its chest if the event fails.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Taken {
    pub event: String,
    pub day: f32,
    pub took: Took,
}

/// A command to send at a later tick: a chest's close after its open.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Later {
    pub at_tick: u64,
    pub command: Command,
    /// Sent only while the target is still crouched and still (`breath`): dropped as they move.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub while_still: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Brain {
    /// The plans waiting for their moments, at most `brain.most_plans`.
    pub plans: Vec<Plan>,
    /// A save from before plans ran side by side had one; [`Brain::migrate`] moves it.
    #[serde(default, skip_serializing)]
    pub plan: Option<Plan>,
    pub next_plan_id: u64,
    pub pending: Vec<Pending>,
    pub later: Vec<Later>,
    /// What the events under way took and keep, until each is over.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub taken: Vec<Taken>,
    /// The events of `SPOT_BY_SPOT` under way with a spot already done: a later spot that fails
    /// does not make the event a failure.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spots_done: Vec<(String, f32)>,
    pub next_plan_tick: u64,
    /// The tick the surroundings may be asked for again, so an unanswered pass is retried, not
    /// repeated every tick.
    pub next_survey_tick: u64,
    /// The tick the house around the target may be scanned again (`Memory::here`).
    #[serde(default)]
    pub next_here_scan: u64,
    /// The tick of the last home read, and whether the target was near the home last tick.
    #[serde(skip)]
    pub last_home_read: u64,
    #[serde(skip)]
    pub was_near_home: bool,
    /// The tick of the last village read.
    #[serde(skip)]
    pub last_village_read: u64,
    /// The tick of the last cave read.
    #[serde(skip)]
    pub last_cave_read: u64,
    /// The tick of the last field read.
    #[serde(skip)]
    pub last_field_read: u64,
    /// The tick of the last tree read.
    #[serde(skip)]
    pub last_tree_read: u64,
    /// The tick the footprints round the house were last sent (`prints_around_the_house`).
    #[serde(skip)]
    pub last_prints_sent: u64,
    /// The tick they began running lit at night (`drawn_to_the_light`), 0 when they are not;
    /// and whether this run has been answered.
    #[serde(skip)]
    pub running_lit_since: u64,
    #[serde(skip)]
    pub running_lit_answered: bool,
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
    /// The lines the model is writing for planned events.
    #[serde(skip)]
    pub writings: Vec<Writing>,
    /// Lines the model wrote, by event, used when the plan runs.
    #[serde(skip)]
    pub written: Vec<(String, String)>,
    /// An answer to what the target typed, waiting for its moment. It needs no plan slot, so a
    /// waiting plan never holds a reply back.
    #[serde(default)]
    pub reply: Option<ChatReply>,
    /// The model's writing job for the reply, if one is out.
    #[serde(skip)]
    pub reply_writing: Option<u64>,
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

/// An answer to the target's chat line (`answers_you`): sent at `due_tick` with the model's
/// line, or the bank's if none came, and dropped at `until_tick` or if they left.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatReply {
    pub due_tick: u64,
    pub until_tick: u64,
    /// The model's line, once written and checked.
    pub written: Option<String>,
}

/// How likely a line about the lodger is answered (`answers_you`), and any other line; at most
/// [`REPLIES_A_DAY`] a day, and the row's cooldown (a quarter day, five minutes of play) apart.
/// The owner's call: "stage 2 and up, often if about it".
pub const REPLY_CHANCE_ABOUT: f64 = 0.6;
pub const REPLY_CHANCE_OTHER: f64 = 0.15;
pub const REPLIES_A_DAY: usize = 3;

/// How likely a door they open shuts again a moment later (`door_held`).
pub const HELD_CHANCE: f64 = 0.25;

/// How likely another chest answers the one they open (`chest_answers`).
pub const ECHO_CHANCE: f64 = 0.3;
/// The chance a bell they rang rings again, its rules allowing.
pub const BELL_CHANCE: f64 = 0.5;
/// The chance the figure stands at the foot of the bed as they wake, its rules allowing.
pub const WOKE_CHANCE: f64 = 0.35;
/// How often coming back through a portal finds a figure beside it (`waits_at_the_portal`).
pub const PORTAL_CHANCE: f64 = 0.3;
/// How often a light placed underground shows a figure (`torch_flash`).
pub const TORCH_FLASH_CHANCE: f64 = 0.15;
/// How long they run lit at night before it may notice them, and how likely it is to then.
pub const RUN_LIT_TICKS: u64 = 5 * TPS;
pub const DRAWN_CHANCE: f64 = 0.35;
/// The events a waking may start, one of those the stage offers picked.
pub const WOKE_EVENTS: &[&str] = &["foot_of_bed", "upside_down"];
/// The chance crouching still ten seconds brings the breathing at their ear (`breath`).
pub const BREATH_CHANCE: f64 = 0.3;
/// The bold things that may have happened at home while they were in another dimension
/// (`busy_away`), in the order tried: the first whose rules and needs hold runs.
pub const WHILE_AWAY: &[&str] = &[
    "house_near",
    "pillar",
    "hut_on_route",
    "dock",
    "bridge",
    "cairn",
    "light_path",
    "path_worn",
    "grave",
    "sapling",
];
/// Gone this long or more (days: three minutes of a twenty-minute day) for `busy_away`.
pub const AWAY_LEAST: f32 = 0.15;
/// The chance a door they knock on knocks back (`answer_knock`).
pub const KNOCK_CHANCE: f64 = 0.5;
/// The events whose later sounds stop as soon as the target moves.
const STOPS_WHEN_THEY_MOVE: &[&str] = &["breath"];

/// How likely a chest of fresh loot gets its offering, and how likely the offering takes a
/// loot item's place rather than an empty slot's (owner: "1 in 5", "mostly add").
pub const LOOT_CHANCE: f32 = 0.2;
pub const LOOT_SWAP: f32 = 0.25;

/// What may turn up in loot from nowhere, loot being the world's (owner): the
/// one exception to "it never spawns anything", and only when it holds nothing of its own.
pub const LOOT_FROM_NOWHERE: &[&str] = &["minecraft:redstone_lamp", "minecraft:candle"];

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
    /// Which brain answered: `model`, or `rules` without it.
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

    /// What an event under way took goes back into the chest it came from (a failure, or the
    /// event given up): a put each, dropped by the chest if it cannot go in.
    fn put_back(&mut self, event: &str, day: f32, pos: Option<BlockPos>) {
        let mine = |t: &Taken| t.event == event && t.day == day;
        let back: Vec<Taken> = self.taken.iter().filter(|t| mine(t)).cloned().collect();
        self.taken.retain(|t| !mine(t));
        for taken in back {
            let id = self.id();
            self.pending.push(Pending {
                id,
                event: event.to_string(),
                pos,
                day,
                tail: true,
                back: Some(taken.took.clone()),
                ..Pending::default()
            });
            self.later.push(Later {
                at_tick: 0,
                command: Command::PutItem {
                    id,
                    into: taken.took.from,
                    slot: Some(taken.took.slot),
                    item: craftkit::world::ItemStack {
                        id: taken.took.item.clone(),
                        count: taken.took.count.max(1),
                        slot: taken.took.slot,
                    },
                    kept: taken.took.keep.clone(),
                    wear: 0,
                    name: String::new(),
                },
                while_still: false,
            });
        }
    }

    /// An event under way given up (the debug screen's Clear or Forget): what it still had to
    /// send is dropped, and what it took is put back, not left kept for good.
    pub fn abandon(&mut self, event: &str, day: f32) {
        let dropped: Vec<u64> = self
            .pending
            .iter()
            .filter(|p| p.event == event && p.day == day)
            .map(|p| p.id)
            .collect();
        self.later.retain(|l| !dropped.contains(&l.command.id()));
        self.pending.retain(|p| !dropped.contains(&p.id));
        self.put_back(event, day, None);
    }

    /// Every event under way given up.
    pub fn abandon_all(&mut self) {
        let under_way: Vec<(String, f32)> = self
            .taken
            .iter()
            .map(|t| (t.event.clone(), t.day))
            .collect();
        for (event, day) in under_way {
            self.abandon(&event, day);
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
        let still = memory.last.online && memory.last.crouched_still;
        for later in self.later.drain(..) {
            if later.while_still && !still {
                continue;
            }
            if later.at_tick <= now.tick {
                commands.push(later.command);
            } else {
                kept.push(later);
            }
        }
        self.later = kept;

        // After the stay: kept, a door of theirs open now and then.
        if arc.over() {
            let mut ids = || self.id();
            commands.extend(crate::ending::after(arc, memory, now, &mut ids));
            return commands;
        }
        if !arc.moved_in() {
            return commands;
        }
        // Its place, chosen once it has moved in and knows their home (until now it was
        // never chosen, and each event that wanted it picked a spot of its own).
        if memory.place.is_none() && memory.home.is_some() {
            memory.place = act::spot(memory, &mut self.rng);
            if let Some(place) = memory.place {
                memory.placed_at_place = memory
                    .placed
                    .iter()
                    .filter(|p| crate::memory::distance(**p, place) <= 5.0)
                    .copied()
                    .take(64)
                    .collect();
            }
        }
        // Stage five: the ending's beats, as their hour comes.
        if arc.stage == 5 {
            let mut ids = || self.id();
            let made = crate::ending::tick(arc, memory, config, now, &mut ids);
            if !made.is_empty() {
                log.note(&format!(
                    "the ending ({}): {} commands",
                    arc.ending,
                    made.len()
                ));
            }
            for (command, wait) in made {
                if wait == 0 {
                    commands.push(command);
                } else {
                    self.later.push(Later {
                        at_tick: now.tick + wait,
                        command,
                        while_still: false,
                    });
                }
            }
            if arc.over() {
                log.note(&format!("the stay is over: {}", arc.ended));
                return commands;
            }
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
                memory.describe_within(day, crate::memory::describe_chars(config.model.context)),
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

        // The home, read again: when they come home, half a minute after they build or break
        // near it, and every ten minutes while they are there. Cheap: a few thousand blocks.
        if let Some(home) = memory.home.clone() {
            let at = memory.last.pos;
            let near = memory.last.online && {
                let (dx, dz) = (at[0] - home.center[0] as f64, at[2] - home.center[2] as f64);
                (dx * dx + dz * dz).sqrt() <= 24.0
            };
            let came_home = near && !self.was_near_home;
            self.was_near_home = near;
            let since = now.tick.saturating_sub(self.last_home_read);
            let due = self.last_home_read == 0
                || came_home
                || (memory.home_dirty && since >= 30 * TPS)
                || (near && since >= 600 * TPS);
            if due {
                let c = home.center;
                let (min, max) = (
                    [c[0] - 8, c[1] - 3, c[2] - 8],
                    [c[0] + 8, c[1] + 5, c[2] + 8],
                );
                let id = self.id();
                commands.push(Command::Scan {
                    step: 0,
                    dark: false,
                    most: 0,
                    id,
                    min,
                    max,
                    kinds: crate::memory::HOME_KINDS
                        .iter()
                        .map(|k| k.to_string())
                        .collect(),
                });
                memory.home_scan = (id, min, max);
                memory.home_dirty = false;
                self.last_home_read = now.tick.max(1);
            }
        }

        // The village, read every five minutes while they are within 64 blocks of it: its
        // doors, its bell, its lanterns, for the village events.
        if let Some(village) = memory.feature("village").map(|v| v.pos) {
            let at = memory.last.pos;
            let (dx, dz) = (at[0] - village[0] as f64, at[2] - village[2] as f64);
            let near = memory.last.online && (dx * dx + dz * dz).sqrt() <= 64.0;
            if near
                && (self.last_village_read == 0
                    || now.tick.saturating_sub(self.last_village_read) >= 300 * TPS)
            {
                let id = self.id();
                commands.push(Command::Scan {
                    step: 0,
                    dark: false,
                    most: 0,
                    id,
                    min: [village[0] - 19, village[1] - 8, village[2] - 19],
                    max: [village[0] + 19, village[1] + 15, village[2] + 19],
                    kinds: ["*_door", "minecraft:bell", "*lantern"]
                        .iter()
                        .map(|k| k.to_string())
                        .collect(),
                });
                memory.scans_out.push((id, "village_read".to_string()));
                self.last_village_read = now.tick.max(1);
            }
        }

        // The caves around them, read every ten minutes while they are above ground within 48
        // blocks of home: the dark air below, every third block, for `lantern_ahead` and
        // `cave_lit` (`crate::caves`).
        if let Some(home) = memory.home.as_ref().map(|h| h.center) {
            let at = memory.last.pos;
            let (dx, dz) = (at[0] - home[0] as f64, at[2] - home[2] as f64);
            let near = memory.last.online
                && !memory.last.underground
                && (dx * dx + dz * dz).sqrt() <= 48.0;
            if near
                && (self.last_cave_read == 0
                    || now.tick.saturating_sub(self.last_cave_read) >= 600 * TPS)
            {
                use crate::caves::{READ_ABOVE, READ_BELOW, READ_STEP, READ_WIDE};
                let c = [
                    at[0].floor() as i32,
                    at[1].floor() as i32,
                    at[2].floor() as i32,
                ];
                let id = self.id();
                commands.push(Command::Scan {
                    id,
                    min: [c[0] - READ_WIDE, c[1] - READ_BELOW, c[2] - READ_WIDE],
                    max: [c[0] + READ_WIDE, c[1] + READ_ABOVE, c[2] + READ_WIDE],
                    kinds: vec![
                        "minecraft:air".to_string(),
                        "minecraft:cave_air".to_string(),
                    ],
                    step: READ_STEP,
                    dark: true,
                    most: crate::caves::MOST_CAVE as u32,
                });
                memory.scans_out.push((id, "cave_read".to_string()));
                self.last_cave_read = now.tick.max(1);
            }
        }

        // The footprints round the house, sent again while they are near home every nine
        // minutes, so the prints stay on their screen (with the glitches on).
        if config.events.glitches && !memory.prints.is_empty() && memory.last.online {
            let near = memory.home.as_ref().is_some_and(|h| {
                let at = memory.last.pos;
                let (dx, dz) = (at[0] - h.center[0] as f64, at[2] - h.center[2] as f64);
                (dx * dx + dz * dz).sqrt() <= crate::prints::NEAR_HOME
            });
            if near
                && (self.last_prints_sent == 0
                    || now.tick.saturating_sub(self.last_prints_sent)
                        >= crate::prints::RESEND_TICKS)
            {
                let target = memory.target.unwrap_or(0);
                let mut ids = || self.id();
                commands.push(act::prints_hint(&mut ids, target, &memory.prints));
                self.last_prints_sent = now.tick.max(1);
            }
        }

        // The trees around them, read every twenty seconds while they are outdoors in a wooded
        // biome: the logs within twenty blocks, for the forest's events (`crate::trees`).
        if memory.last.online
            && !memory.last.indoors
            && !memory.last.underground
            && crate::trees::wooded(&memory.last.biome)
            && (self.last_tree_read == 0
                || now.tick.saturating_sub(self.last_tree_read)
                    >= crate::trees::READ_EVERY_SECONDS * TPS)
        {
            use crate::trees::{READ_ABOVE, READ_BELOW, READ_WIDE};
            let at = memory.last.pos;
            let c = [
                at[0].floor() as i32,
                at[1].floor() as i32,
                at[2].floor() as i32,
            ];
            let id = self.id();
            commands.push(Command::Scan {
                id,
                min: [c[0] - READ_WIDE, c[1] - READ_BELOW, c[2] - READ_WIDE],
                max: [c[0] + READ_WIDE, c[1] + READ_ABOVE, c[2] + READ_WIDE],
                kinds: vec!["*_log".to_string()],
                step: 0,
                dark: false,
                most: crate::trees::MOST_LOGS as u32,
            });
            memory.scans_out.push((id, "tree_read".to_string()));
            self.last_tree_read = now.tick.max(1);
        }

        // The field by the home, read every ten minutes while they are within 64 blocks of it:
        // every crop block around the crops the surroundings saw (`crate::farm`).
        if let Some(crops) = memory.feature("crops").map(|f| f.pos) {
            let at = memory.last.pos;
            let (dx, dz) = (at[0] - crops[0] as f64, at[2] - crops[2] as f64);
            let near = memory.last.online && (dx * dx + dz * dz).sqrt() <= 64.0;
            if near
                && (self.last_field_read == 0
                    || now.tick.saturating_sub(self.last_field_read) >= 600 * TPS)
            {
                use crate::farm::{CROPS, MOST_FIELD, READ_HIGH, READ_WIDE};
                let id = self.id();
                commands.push(Command::Scan {
                    id,
                    min: [
                        crops[0] - READ_WIDE,
                        crops[1] - READ_HIGH,
                        crops[2] - READ_WIDE,
                    ],
                    max: [
                        crops[0] + READ_WIDE,
                        crops[1] + READ_HIGH,
                        crops[2] + READ_WIDE,
                    ],
                    kinds: CROPS.iter().map(|c| c.to_string()).collect(),
                    step: 0,
                    dark: false,
                    most: MOST_FIELD as u32,
                });
                memory.scans_out.push((id, "field_read".to_string()));
                self.last_field_read = now.tick.max(1);
            }
        }

        // The house they are in, when it is not their home: its doors and chests, so a knock
        // in a village house is on that house's door. Every thirty seconds at most.
        if memory.last.online && memory.last.indoors && now.tick >= self.next_here_scan {
            let at = memory.last.pos;
            let away = memory.home.as_ref().is_none_or(|h| {
                let (dx, dz) = (at[0] - h.center[0] as f64, at[2] - h.center[2] as f64);
                (dx * dx + dz * dz).sqrt() > 24.0
            });
            if away {
                let c = [
                    at[0].floor() as i32,
                    at[1].floor() as i32,
                    at[2].floor() as i32,
                ];
                let id = self.id();
                commands.push(Command::Scan {
                    step: 0,
                    dark: false,
                    most: 0,
                    id,
                    min: [c[0] - 8, c[1] - 3, c[2] - 8],
                    max: [c[0] + 8, c[1] + 5, c[2] + 8],
                    kinds: crate::memory::HOME_KINDS
                        .iter()
                        .map(|k| k.to_string())
                        .collect(),
                });
                memory.here_scan = id;
                self.next_here_scan = now.tick + 30 * TPS;
            }
        }

        // What they typed: perhaps an answer, in a few seconds, whatever else is waiting.
        if let Some((heard, about)) = memory.heard.take() {
            self.consider_reply(&heard, about, now, day, memory, arc, config, log, model);
        }
        // Nothing of its own while the fight is on or another mod's horror is near: the
        // reactions below used to go on through both.
        let held_back = arc.fighting_since > 0 || memory.last.other_horror_near;
        // Held back, the reply waits inside its window, unspent (it used to be spent and dropped).
        if !held_back {
            if let Some(command) = self.reply_due(now, day, memory, log) {
                commands.push(command);
            }
        }
        // An offering for their next fresh loot, armed when none is.
        if !memory.loot_armed && memory.last.online {
            if let Some(command) = self.arm_loot(day, memory, arc, config) {
                commands.push(command);
            }
        }

        // A door they open: now and then it shuts again a moment later.
        if let Some(door) = memory.door_used_now.take().filter(|_| !held_back) {
            commands.extend(self.consider_held(door, now, day, memory, arc, config, log));
        }

        // They killed one of what its kill sign counts: the count goes up.
        if let Some(kind) = memory.killed_now.take() {
            // The kind on the sign, not whatever they kill most now; and never while they look
            // at it (`consider_started` asks the rules without a place).
            let counted = memory.sign_tally().is_some_and(|t| t.kind == kind);
            let seen = act::kill_sign_spot(memory)
                .is_none_or(|s| memory.in_view(s, crate::temper::min_distance(config, memory)));
            if counted && !seen && !held_back {
                self.consider_started("count_up", 1.0, now, day, memory, arc, config, log);
            }
        }

        // They rang a bell: perhaps it rings again, once.
        if memory.rang_bell_now.is_some() && !held_back {
            self.consider_started(
                "bell_twice",
                BELL_CHANCE,
                now,
                day,
                memory,
                arc,
                config,
                log,
            );
            memory.rang_bell_now = None;
        }

        // Back from another dimension a while: something happened at home meanwhile.
        if let Some(gone) = memory.back_after.take() {
            if gone >= AWAY_LEAST && !held_back {
                self.consider_back(now, day, memory, arc, config, log);
            }
            // Perhaps a figure beside the portal they came back by, as if it had waited.
            if !held_back {
                self.consider_started(
                    "waits_at_the_portal",
                    PORTAL_CHANCE,
                    now,
                    day,
                    memory,
                    arc,
                    config,
                    log,
                );
            }
        }

        // They knocked on a door: perhaps it knocks back.
        if memory.knocked_now.is_some() && !held_back {
            self.consider_started(
                "answer_knock",
                KNOCK_CHANCE,
                now,
                day,
                memory,
                arc,
                config,
                log,
            );
            memory.knocked_now = None;
        }

        // They crouch still: perhaps breathing at their ear, until they move.
        if std::mem::take(&mut memory.crouched_now) && !held_back {
            self.consider_started("breath", BREATH_CHANCE, now, day, memory, arc, config, log);
        }

        // They woke: perhaps the figure at the foot of the bed for half a second, or hanging
        // from the ceiling over it, whichever the stage offers, one picked.
        if std::mem::take(&mut memory.woke_now) && !held_back {
            let woken: Vec<&str> = WOKE_EVENTS
                .iter()
                .copied()
                .filter(|id| {
                    events::find(id)
                        .is_some_and(|e| (e.from_stage..=e.to_stage).contains(&arc.stage))
                })
                .collect();
            let pick = if woken.is_empty() {
                "foot_of_bed"
            } else {
                woken[(self.rng.next_f64() * woken.len() as f64) as usize % woken.len()]
            };
            self.consider_started(pick, WOKE_CHANCE, now, day, memory, arc, config, log);
        }

        // They came to the offer's figure: what it held out is at their feet, given back.
        if let Some(at) = memory.offer_reached_now.take() {
            if let Some(held) = memory.held.clone() {
                let id = self.id();
                self.pending.push(Pending {
                    id,
                    event: "the_offer".to_string(),
                    pos: Some(at),
                    day,
                    puts_held: true,
                    tail: true,
                    ..Pending::default()
                });
                commands.push(Command::DropItem {
                    id,
                    pos: [at[0] as f64 + 0.5, at[1] as f64 + 0.5, at[2] as f64 + 0.5],
                    item: held.item,
                    kept: "held".to_string(),
                });
            }
        }

        // A light placed underground: perhaps a figure in its light, for a blink.
        if memory.lit_underground_now.take().is_some() && !held_back {
            self.consider_started(
                "torch_flash",
                TORCH_FLASH_CHANCE,
                now,
                day,
                memory,
                arc,
                config,
                log,
            );
        }

        // Running about at night with a light in hand, five seconds of it: perhaps it notices,
        // once a run; the run over (no light, no running, indoors), it may again next time.
        let night = now.day_time >= 13_000 && now.day_time <= 23_000;
        if night && memory.running_lit() {
            if self.running_lit_since == 0 {
                self.running_lit_since = now.tick.max(1);
            } else if !self.running_lit_answered
                && now.tick.saturating_sub(self.running_lit_since) >= RUN_LIT_TICKS
            {
                self.running_lit_answered = true;
                if !held_back {
                    self.consider_started(
                        "drawn_to_the_light",
                        DRAWN_CHANCE,
                        now,
                        day,
                        memory,
                        arc,
                        config,
                        log,
                    );
                }
            }
        } else {
            self.running_lit_since = 0;
            self.running_lit_answered = false;
        }

        // A chest of theirs opened: perhaps another closes in the house at the same moment.
        if memory.opened_now.is_some() && !held_back {
            commands.extend(self.consider_echo(now, day, memory, arc, config, log));
            memory.opened_now = None;
        }
        // Held back, a trigger is spent all the same, not saved for after.
        memory.rang_bell_now = None;
        memory.knocked_now = None;
        memory.opened_now = None;

        // A reaction, on a trigger, ten seconds apart at most, when nothing else is asked.
        let may_react = self.asking.is_none()
            && (self.last_react_tick == 0
                || now.tick.saturating_sub(self.last_react_tick) >= REACT_EVERY_TICKS);
        if may_react && !memory.triggers.is_empty() {
            let trigger = memory.triggers.remove(0);
            {
                self.last_react_tick = now.tick.max(1);
                let mut lines = memory
                    .describe_within(day, crate::memory::describe_chars(config.model.context));
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

        if self.plans.len() < config.brain.most_plans.max(1) as usize
            && self.asking.is_none()
            && now.tick >= self.next_plan_tick
        {
            self.next_plan_tick = now.tick + config.brain.plan_every as u64 * TPS;
            self.decide(now, day, memory, arc, config, log, model);
        }

        // A plan that carries words and has not asked for them (one fired from the debug
        // screen) asks now and gives the model three seconds; Fire wrote from the bank alone
        // and the same six lines came round.
        let unasked: Vec<u64> = self
            .plans
            .iter()
            .filter(|p| !p.words_asked)
            .map(|p| p.id)
            .collect();
        for id in unasked {
            let Some(event) = self
                .plans
                .iter()
                .find(|p| p.id == id)
                .and_then(|p| events::find(&p.event))
            else {
                continue;
            };
            let asks = model.is_some() && !config.brain.force_rules;
            if asks {
                let lines = memory
                    .describe_within(day, crate::memory::describe_chars(config.model.context));
                self.ask_for_words(event, &lines, memory, config, model);
            }
            if let Some(plan) = self.plans.iter_mut().find(|p| p.id == id) {
                plan.words_asked = true;
                if asks {
                    plan.from_tick = plan.from_tick.max(now.tick + 3 * TPS);
                }
            }
        }

        // Every plan on its own: cancelled once it cannot happen, waiting for its moment or
        // for them to look away, or carried out.
        let ids: Vec<u64> = self.plans.iter().map(|p| p.id).collect();
        for id in ids {
            let Some(plan) = self.plans.iter().find(|p| p.id == id).cloned() else {
                continue;
            };
            match self.run_plan(&plan, now, day, memory, arc, config, log) {
                Step::Wait(why) => {
                    if let Some(plan) = self.plans.iter_mut().find(|p| p.id == id) {
                        plan.waiting = why;
                    }
                }
                Step::Gone => self.plans.retain(|p| p.id != id),
                Step::Sent(sent) => {
                    commands.extend(sent);
                    self.plans.retain(|p| p.id != id);
                }
            }
        }

        commands
    }

    /// Asks the model to write the line an event will carry, if it carries one and the model
    /// is there: the kind's examples shown first, the moment's situation, the config's
    /// temperature. The line comes back on a later tick and waits in `written`.
    fn ask_for_words(
        &mut self,
        chosen: &Event,
        lines: &[String],
        memory: &Memory,
        config: &Config,
        model: Option<&Model>,
    ) {
        if let (Some(model), Some(kind)) = (
            model.filter(|_| !config.brain.force_rules),
            act::text_kind(chosen),
        ) {
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
                let prompt = situation::writing_prompt(lines, kind.purpose(), &refs);
                model.generate_async(
                    &prompt,
                    crate::model::Leash {
                        temperature: config.brain.text_temperature as f32,
                        ..kind.leash()
                    },
                    Some("text".to_string()),
                    sender,
                )
            } else {
                // `bank`: the model picks among eight of the kind's lines.
                let choices: Vec<String> = examples.iter().take(8).cloned().collect();
                model.ask_async(
                    &situation::situation(lines),
                    vec![Question {
                        text: format!("{} {}", kind.purpose(), SIGN_SAYS),
                        choices,
                    }],
                    Some("text".to_string()),
                    sender,
                )
            };
            if let Ok(id) = asked {
                self.writings.push(Writing {
                    id,
                    event: chosen.id.to_string(),
                    kind,
                    mob,
                    count,
                    bank: config.brain.text != "generate",
                });
            }
        }
    }

    /// One plan's tick: cancelled if it can no longer happen, gone if its day passed, waiting
    /// (and why) before its moment, for its needs, or while its blocks are in view, refused if
    /// it would dig near what they built, else carried out.
    #[allow(clippy::too_many_arguments)]
    fn run_plan(
        &mut self,
        plan: &Plan,
        now: &Now,
        day: f32,
        memory: &mut Memory,
        arc: &mut Arc,
        config: &Config,
        log: &mut log::Log,
    ) -> Step {
        let Some(event) = events::find(&plan.event) else {
            return Step::Gone;
        };
        if now.tick > plan.until_tick {
            memory.record(&plan.event, plan.pos, day, false, "its moment never came");
            self.outcome(&plan.event, "its moment never came");
            return Step::Gone;
        }
        if !plan.forced {
            if let Some(why) = cancel_reason(event, plan, memory, arc.stage, day, config) {
                self.cancel(plan, &why, day, memory, log);
                return Step::Gone;
            }
        }
        if now.tick < plan.from_tick {
            return Step::Wait(String::new());
        }
        // The fight is on: nothing else of its own meanwhile (an arrow, a splash, a death not its).
        if !plan.forced && arc.fighting_since > 0 {
            return Step::Wait("the fight is on".to_string());
        }
        // Another horror mod's thing near them: never at the same moment as it.
        if !plan.forced && memory.last.other_horror_near {
            return Step::Wait("another mod's thing is near them".to_string());
        }
        if !plan.forced {
            if !act::ready(event, memory, now) {
                let why = offer::unmet(event, memory, now).unwrap_or_default();
                return Step::Wait(format!("waiting: {why}"));
            }
            if !when_holds(&plan.when, memory) {
                return Step::Wait(format!("waiting {}", when_words(&plan.when)));
            }
        }

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
                act::commands(event, plan.pos, memory, &mut ids, now, written.as_deref())
            }
            Err(_) => Vec::new(),
        };
        // Digging near what they built is refused whoever asked, the debug screen's Fire too:
        // the host keeps a placed block itself from being replaced, but not the ground under it.
        // `alcove` alone digs beside what they built: one block of natural stone (owner), the host refusing anything else.
        let digs_near = event.id != "alcove";
        if let Some(dug) = made.iter().find_map(|(command, _)| {
            act::dug(command)
                .map(|p| act::guessed(memory, p))
                .filter(|p| digs_near && !act::clear_of_built(memory, *p))
        }) {
            let why = "too near what they built";
            log.note(&format!(
                "{} would dig at {}: {why}",
                plan.event,
                log::spot_words(dug)
            ));
            memory.record(&plan.event, plan.pos, day, false, why);
            self.outcome(&plan.event, why);
            return Step::Gone;
        }
        // In view is a reason to wait, not to give up: every block it would change, not just
        // its spot, stays out of sight until they look away, within the plan's day.
        let seen = (!plan.forced || plan.out_of_sight)
            && (checked == Err(rules::Refused::InView)
                || (checked.is_ok()
                    && rules::hidden_from_view(event.action)
                    && made.iter().any(|(command, _)| {
                        act::spots(command).iter().any(|spot| {
                            memory.in_view(
                                act::guessed(memory, *spot),
                                crate::temper::min_distance(config, memory),
                            )
                        })
                    })));
        if seen {
            return Step::Wait("in view".to_string());
        }
        // With witness traces off, every block it would change, not only its spot, waits for
        // another player near to go (`multiplayer.rs`).
        let witnessed = (!plan.forced || plan.out_of_sight)
            && matches!(event.effect(), "house" | "things" | "words")
            && {
                let spots: Vec<BlockPos> = made
                    .iter()
                    .flat_map(|(c, _)| act::spots(c))
                    .map(|s| act::guessed(memory, s))
                    .collect();
                // No block to change (frames turned, a stand dressed): where it would be.
                if spots.is_empty() {
                    memory.witness_near(config, plan.pos)
                } else {
                    spots
                        .iter()
                        .any(|spot| memory.witness_near(config, Some(*spot)))
                }
            };
        if witnessed {
            return Step::Wait("another player is near".to_string());
        }
        // Within its cooldown, or the same as the last, is a reason to wait while its window
        // lasts, as `cancel_reason` says; until now it gave the plan up.
        if matches!(
            checked,
            Err(rules::Refused::Cooldown)
                | Err(rules::Refused::SameAsLast)
                | Err(rules::Refused::Witness)
        ) {
            return Step::Wait(checked.unwrap_err().words().to_string());
        }
        if let Err(refused) = checked {
            memory.record(&plan.event, plan.pos, day, false, refused.words());
            self.outcome(&plan.event, refused.words());
            return Step::Gone;
        }

        self.written.retain(|(e, _)| *e != plan.event);
        // Only what was sent spends: an event that made nothing (a tidy chest, no slot free)
        // used the day's budget and counted toward the next stage until now.
        if !plan.forced && (!made.is_empty() || plan.event == "wait") {
            arc.spend(event.cost_in(config), now.day);
        }
        if plan.forced && made.is_empty() {
            memory.record(&plan.event, plan.pos, day, false, "nothing to do it with");
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
        // Nothing made is nothing done: not a landmark, not toward its cap (until now
        // it was recorded as landed). Waiting is the one event that is meant to make nothing.
        if made.is_empty() && !plan.forced {
            memory.record(
                &plan.event,
                plan.pos,
                day,
                plan.event == "wait",
                if plan.event == "wait" {
                    ""
                } else {
                    "nothing to do it with"
                },
            );
        }
        for command in made.iter().map(|(c, _)| c) {
            for line in act::texts(command) {
                memory.wrote(line);
            }
            // A scan's answer is kept by what sent it: the village's and the ore's apart.
            if let Command::Scan { id, .. } = command {
                memory.scans_out.push((*id, plan.event.clone()));
                if memory.scans_out.len() > 8 {
                    memory.scans_out.remove(0);
                }
            }
            // What it gives without having taken it comes out of what it dug.
            // Not the torches it took down off their walls (`one_room_dark`): those were theirs.
            if let Command::PutItem { item, kept, .. } = command {
                if kept.is_empty() && plan.event != "one_room_dark" && memory.stored(&item.id) > 0 {
                    memory.store(&item.id, -(item.count as i64));
                }
            }
            // A build's share: laid, the store spent where it has the block, the rest made up.
            crate::building::account(memory, &plan.event, command, day);
            // The kind the kill sign counts, as written.
            if plan.event == "kill_sign" && matches!(command, Command::SetSign { .. }) {
                if let Some(tally) = memory.most_killed() {
                    memory.kill_sign_kind = tally.kind.clone();
                }
            }
            // A light of its own, kept for the endings.
            if let Command::SetBlock { pos, block, .. } = command {
                if block.ends_with("torch") || block.ends_with("lantern") {
                    memory.its_light(*pos);
                }
            }
            // A block it fills with comes out of what it dug too.
            if let Command::SetBlock { block, .. } = command {
                if plan.event == "tunnel_sealed" {
                    memory.store(block, -1);
                }
                if plan.event == "backfill" {
                    memory.store(block, -1);
                    if let Some(at) = memory.holes.iter().position(
                        |(p, _)| matches!(command, Command::SetBlock { pos, .. } if pos == p),
                    ) {
                        memory.holes.remove(at);
                    }
                }
            }
        }
        let mut sent = Vec::new();
        // The offer's figure stands holding it out: the memory waits for them to come to it.
        if plan.event == "the_offer" {
            memory.offer_waiting = plan.pos;
        }
        for (command, later_ticks) in made {
            let puts_held = matches!(&command, Command::PutItem { kept, .. } if kept == "held");
            self.pending.push(Pending {
                id: command.id(),
                event: plan.event.clone(),
                pos: plan.pos,
                day,
                puts_held,
                tail: later_ticks >= TAIL_TICKS,
                // The torches it took down off their walls, put in their chest: dropped by it
                // if it is full (it was judged from the chest as last seen).
                back: match &command {
                    Command::PutItem {
                        into, item, kept, ..
                    } if plan.event == "one_room_dark" && kept.is_empty() => Some(Took {
                        from: *into,
                        slot: 0,
                        item: item.id.clone(),
                        keep: String::new(),
                        count: item.count,
                    }),
                    _ => None,
                },
                took: match &command {
                    Command::TakeItem {
                        from,
                        slot,
                        keep,
                        expect,
                        count,
                        ..
                    } if !keep.is_empty() || !expect.is_empty() => Some(Took {
                        from: *from,
                        slot: *slot,
                        item: expect.clone(),
                        keep: keep.clone(),
                        count: *count,
                    }),
                    _ => None,
                },
            });
            if later_ticks == 0 {
                sent.push(command);
            } else {
                self.later.push(Later {
                    at_tick: now.tick + later_ticks,
                    command,
                    while_still: false,
                });
            }
        }

        Step::Sent(sent)
    }

    /// A plan given up, with its reason in the log, the memory and the story.
    fn cancel(
        &mut self,
        plan: &Plan,
        why: &str,
        day: f32,
        memory: &mut Memory,
        log: &mut log::Log,
    ) {
        log.note(&format!("cancelled {}: {why}", plan.event));
        memory.record(&plan.event, plan.pos, day, false, why);
        self.outcome(&plan.event, &format!("cancelled: {why}"));
        self.writings.retain(|w| w.event != plan.event);
        self.written.retain(|(e, _)| *e != plan.event);
    }

    /// Drops one plan by its number: the debug screen's Drop. Says which.
    pub fn drop_plan(
        &mut self,
        id: u64,
        day: f32,
        memory: &mut Memory,
        log: &mut log::Log,
    ) -> String {
        let Some(plan) = self.plans.iter().find(|p| p.id == id).cloned() else {
            return format!("no plan {id}");
        };
        self.cancel(&plan, "dropped from the debug screen", day, memory, log);
        self.plans.retain(|p| p.id != id);
        format!("dropped {}", plan.event)
    }

    /// Runs one plan on the next tick past its moment and its rules, as Fire does: the debug
    /// screen's Now. Says which.
    pub fn plan_now(&mut self, id: u64) -> String {
        match self.plans.iter_mut().find(|p| p.id == id) {
            Some(plan) => {
                plan.from_tick = 0;
                plan.forced = true;
                plan.until_tick = u64::MAX;
                format!("{} on the next tick", plan.event)
            }
            None => format!("no plan {id}"),
        }
    }

    /// Whether an event may be planned beside the plans already held: never the same event,
    /// never two of one group (two figures, two chat lines), never two on one spot.
    fn clashes(&self, event: &Event, pos: Option<BlockPos>) -> bool {
        let group = offer::group(event);
        self.plans.iter().any(|plan| {
            plan.event == event.id
                || (pos.is_some() && plan.pos == pos)
                || (group.is_some() && events::find(&plan.event).and_then(offer::group) == group)
        })
    }

    /// A save from before plans ran side by side: its one plan joins the list.
    pub fn migrate(&mut self) {
        if let Some(mut plan) = self.plan.take() {
            if plan.id == 0 {
                self.next_plan_id += 1;
                plan.id = self.next_plan_id;
            }
            self.plans.push(plan);
        }
    }

    fn next_plan_id(&mut self) -> u64 {
        self.next_plan_id += 1;
        self.next_plan_id
    }

    /// The offering for the target's next fresh loot (`loot_gift`): what it holds of theirs
    /// first (the very item, kept by the host), their own head from stage four now and then,
    /// something it dug, else a thing from [`LOOT_FROM_NOWHERE`]. The row's rules hold.
    fn arm_loot(
        &mut self,
        day: f32,
        memory: &mut Memory,
        arc: &Arc,
        config: &Config,
    ) -> Option<Command> {
        let event = events::find("loot_gift")?;
        if !offer::switched_on(event, config)
            || rules::check(event, None, memory, arc.stage, day, config).is_err()
        {
            return None;
        }
        let one = |id: &str| craftkit::world::ItemStack {
            id: id.to_string(),
            count: 1,
            slot: 0,
        };
        let dug = ["minecraft:torch", "minecraft:dirt", "minecraft:cobblestone"]
            .into_iter()
            .find(|id| memory.stored(id) > 0);
        let (item, kept) = if let Some(held) = &memory.held {
            (held.item.clone(), "held".to_string())
        } else if arc.stage >= 4 && self.rng.next_f64() < 0.3 {
            (one("@head"), String::new())
        } else if let Some(id) = dug.filter(|_| self.rng.next_f64() < 0.5) {
            (one(id), String::new())
        } else {
            let pick = LOOT_FROM_NOWHERE[(self.rng.next_u64() as usize) % LOOT_FROM_NOWHERE.len()];
            (one(pick), String::new())
        };
        memory.loot_armed = true;

        Some(Command::ArmLoot {
            id: self.id(),
            player: memory.target.unwrap_or(0),
            chance: LOOT_CHANCE,
            replace: LOOT_SWAP,
            item,
            kept,
        })
    }

    /// Whether the door they just used shuts again (`door_held`): the row's rules and a
    /// chance of [`HELD_CHANCE`].
    #[allow(clippy::too_many_arguments)]
    fn consider_held(
        &mut self,
        door: BlockPos,
        now: &Now,
        day: f32,
        memory: &mut Memory,
        arc: &Arc,
        config: &Config,
        log: &mut log::Log,
    ) -> Vec<Command> {
        let Some(event) = events::find("door_held") else {
            return Vec::new();
        };
        if !offer::switched_on(event, config)
            || rules::check(event, None, memory, arc.stage, day, config).is_err()
            || self.rng.next_f64() >= HELD_CHANCE
        {
            return Vec::new();
        }
        let mut ids = || self.id();
        let made = act::commands(event, Some(door), memory, &mut ids, now, None);
        memory.record(event.id, Some(door), day, true, "");
        log.acted(event.id, Some(door), made.len());
        for (command, later) in made {
            self.later.push(Later {
                at_tick: now.tick + later,
                command,
                while_still: false,
            });
        }

        Vec::new()
    }

    /// An event the target started (a kill, a waking): its row's rules, its other needs, and a
    /// chance; its commands go out with the later ones.
    #[allow(clippy::too_many_arguments)]
    fn consider_started(
        &mut self,
        event: &str,
        chance: f64,
        now: &Now,
        day: f32,
        memory: &mut Memory,
        arc: &Arc,
        config: &Config,
        log: &mut log::Log,
    ) {
        let Some(event) = events::find(event) else {
            return;
        };
        if memory.last.other_horror_near {
            return;
        }
        if !offer::switched_on(event, config)
            || rules::check(event, None, memory, arc.stage, day, config).is_err()
            || !event
                .needs
                .iter()
                .filter(|n| {
                    !matches!(
                        n,
                        events::Need::JustWoke
                            | events::Need::LitUnderground
                            | events::Need::RanLit
                            | events::Need::CameBack
                            | events::Need::KilledNow
                            | events::Need::RangBell
                            | events::Need::CrouchedStill
                            | events::Need::Knocked
                    )
                })
                .all(|n| offer::meets(*n, memory, now))
            || self.rng.next_f64() >= chance
        {
            return;
        }
        let Some(pos) = act::place(event, memory, &mut self.rng) else {
            return;
        };
        let mut ids = || self.id();
        let made = act::commands(event, Some(pos), memory, &mut ids, now, None);
        memory.record(event.id, Some(pos), day, true, "");
        log.acted(event.id, Some(pos), made.len());
        let while_still = STOPS_WHEN_THEY_MOVE.contains(&event.id);
        for (command, later) in made {
            self.later.push(Later {
                at_tick: now.tick + later,
                command,
                while_still,
            });
        }
    }

    /// Coming back from another dimension (`busy_away`): the first of [`WHILE_AWAY`] that its
    /// rules allow and whose lasting needs hold runs at once, as a forced plan, so it was done
    /// before they could see it. Its passing needs (the hour, them away) are of the time they
    /// were gone, so they are not asked.
    fn consider_back(
        &mut self,
        now: &Now,
        day: f32,
        memory: &mut Memory,
        arc: &Arc,
        config: &Config,
        log: &mut log::Log,
    ) {
        let Some(busy) = events::find("busy_away") else {
            return;
        };
        if rules::check(busy, None, memory, arc.stage, day, config).is_err() {
            return;
        }
        for id in WHILE_AWAY {
            let Some(event) = events::find(id) else {
                continue;
            };
            if !offer::switched_on(event, config)
                || rules::check(event, None, memory, arc.stage, day, config).is_err()
                || !event
                    .needs
                    .iter()
                    .filter(|n| n.lasting())
                    .all(|n| offer::meets(*n, memory, now))
            {
                continue;
            }
            let pos = act::place(event, memory, &mut self.rng);
            let id = self.next_plan_id();
            self.plans.push(Plan {
                id,
                event: event.id.to_string(),
                pos,
                from_tick: now.tick,
                until_tick: now.tick + 600 * TPS,
                forced: true,
                out_of_sight: true,
                ..Plan::default()
            });
            memory.record(busy.id, pos, day, true, event.id);
            log.note(&format!(
                "they came back from another world: {} happened while they were gone",
                event.id
            ));
            return;
        }
    }

    /// Whether another chest of the house answers the one the target just opened: the row's
    /// rules, a chance of [`ECHO_CHANCE`], and a chest of theirs other than that one.
    fn consider_echo(
        &mut self,
        now: &Now,
        day: f32,
        memory: &mut Memory,
        arc: &Arc,
        config: &Config,
        log: &mut log::Log,
    ) -> Vec<Command> {
        let Some(event) = events::find("chest_answers") else {
            return Vec::new();
        };
        if !offer::switched_on(event, config)
            || rules::check(event, None, memory, arc.stage, day, config).is_err()
            || self.rng.next_f64() >= ECHO_CHANCE
        {
            return Vec::new();
        }
        let Some(pos) = act::place(event, memory, &mut self.rng) else {
            return Vec::new();
        };
        let mut ids = || self.id();
        let made = act::commands(event, Some(pos), memory, &mut ids, now, None);
        memory.record(event.id, Some(pos), day, true, "");
        log.acted(event.id, Some(pos), made.len());
        let mut sent = Vec::new();
        for (command, later) in made {
            if later == 0 {
                sent.push(command);
            } else {
                self.later.push(Later {
                    at_tick: now.tick + later,
                    command,
                    while_still: false,
                });
            }
        }

        sent
    }

    /// Whether to answer what the target just typed, and if so when: the row's rules (stage,
    /// cooldown, `events.chat`, `events.disabled`), [`REPLIES_A_DAY`], one reply waiting at a
    /// time, and a chance by whether the line was about it. The model writes the answer with
    /// their words in its prompt; the bank stands in.
    #[allow(clippy::too_many_arguments)]
    fn consider_reply(
        &mut self,
        heard: &str,
        about: bool,
        now: &Now,
        day: f32,
        memory: &Memory,
        arc: &Arc,
        config: &Config,
        log: &mut log::Log,
        model: Option<&Model>,
    ) {
        let Some(event) = events::find("answers_you") else {
            return;
        };
        let today = memory
            .done
            .iter()
            .filter(|d| d.event == event.id && d.ok && d.day.floor() == day.floor())
            .count();
        if self.reply.is_some()
            || today >= REPLIES_A_DAY
            || !offer::switched_on(event, config)
            || rules::check(event, None, memory, arc.stage, day, config).is_err()
        {
            return;
        }
        let chance = if about {
            REPLY_CHANCE_ABOUT
        } else {
            REPLY_CHANCE_OTHER
        };
        if self.rng.next_f64() >= chance {
            return;
        }

        // Five seconds to a minute after they typed, as someone typing back.
        let due_tick = now.tick + 5 * TPS + self.rng.next_u64() % (55 * TPS);
        self.reply = Some(ChatReply {
            due_tick,
            until_tick: due_tick + 60 * TPS,
            written: None,
        });
        log.note(&format!(
            "will answer what they typed in {}s",
            (due_tick - now.tick) / TPS
        ));
        if let Some(model) = model
            .filter(|m| m.loaded() && !config.brain.force_rules && config.brain.text == "generate")
        {
            let mut lines =
                memory.describe_within(day, crate::memory::describe_chars(config.model.context));
            lines.push(format!("They just typed: {heard}"));
            let kind = Kind::Reply;
            let examples = kind.examples();
            let prompt = situation::writing_prompt(&lines, kind.purpose(), examples);
            let sender = self.replies().0.clone();
            if let Ok(id) = model.generate_async(
                &prompt,
                crate::model::Leash {
                    temperature: config.brain.text_temperature as f32,
                    ..kind.leash()
                },
                Some("reply".to_string()),
                sender,
            ) {
                self.reply_writing = Some(id);
            }
        }
    }

    /// The reply, when its moment comes: the model's line or the bank's, in their name, to
    /// them alone. Dropped if they left or its minute passed.
    fn reply_due(
        &mut self,
        now: &Now,
        day: f32,
        memory: &mut Memory,
        log: &mut log::Log,
    ) -> Option<Command> {
        let reply = self.reply.as_ref()?;
        if now.tick > reply.until_tick || !memory.last.online {
            self.reply = None;
            self.reply_writing = None;
            return None;
        }
        if now.tick < reply.due_tick {
            return None;
        }
        let reply = self.reply.take()?;
        self.reply_writing = None;
        let event = events::find("answers_you")?;
        let mut ids = || self.id();
        let made = act::commands(event, None, memory, &mut ids, now, reply.written.as_deref());
        memory.record(event.id, None, day, true, "");
        for line in made.iter().flat_map(|(c, _)| act::texts(c)) {
            memory.wrote(line);
        }
        log.acted(event.id, None, made.len());

        made.into_iter().next().map(|(command, _)| command)
    }

    /// Every tick the brain waits for, moved `shift` ticks earlier: the host's clock started
    /// again lower (`Stay`'s `last_tick`). A tick of 0, "never", stays 0; one that would pass
    /// 0 is 1, "at once".
    pub fn rebase(&mut self, shift: u64) {
        let back = |t: &mut u64| {
            if *t != 0 {
                *t = t.saturating_sub(shift).max(1);
            }
        };
        for plan in &mut self.plans {
            back(&mut plan.from_tick);
            back(&mut plan.until_tick);
        }
        for later in &mut self.later {
            back(&mut later.at_tick);
        }
        back(&mut self.next_plan_tick);
        back(&mut self.next_survey_tick);
        back(&mut self.last_react_tick);
        back(&mut self.waiting_for_target_since);
        back(&mut self.next_here_scan);
        if let Some(reply) = &mut self.reply {
            back(&mut reply.due_tick);
            back(&mut reply.until_tick);
        }
        if let Some(ask) = &mut self.asking {
            back(&mut ask.asked_tick);
        }
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
        // Nothing planned beside what clashes with a plan held (the same event, its group).
        let menu: Vec<&Event> = menu
            .into_iter()
            .filter(|e| e.id == "wait" || !self.clashes(e, None))
            .collect();
        // Nothing but `wait` on offer is nothing to decide: no ask, no decision logged. The
        // dev client asked the model every interval for a menu of one.
        if menu.iter().all(|e| e.id == "wait") {
            return;
        }
        let lines =
            memory.describe_within(day, crate::memory::describe_chars(config.model.context));

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

        let chosen = rules_brain::choose_with(
            &menu,
            arc.stage,
            &memory.want,
            &|effect| memory.notice_rate(effect),
            &memory.chain_next(),
            &mut self.rng,
        );
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
                        let kind = kind.to_string();
                        self.unasked(&kind, memory, log);
                    }
                    (_, Ok(_)) => {}
                }
            } else if self.reply_writing == Some(reply.id) {
                self.reply_writing = None;
                let written = reply.text.unwrap_or_default();
                let kept = match &reply.result {
                    Ok(_) => text::check(Kind::Reply, &written)
                        .filter(|l| !text::is_repeat(l, &memory.lines_written)),
                    Err(_) => None,
                };
                log.wrote("Reply", &written, kept.as_deref());
                if let Some(answer) = &mut self.reply {
                    answer.written = kept;
                }
            } else if let Some(at) = self.writings.iter().position(|w| w.id == reply.id) {
                let writing = self.writings.remove(at);
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
                // A line it already used this stay is dropped for a fresh one from the bank.
                let kept = match &reply.result {
                    Ok(_) => text::check(writing.kind, &written)
                        .filter(|l| !text::is_repeat(l, &memory.lines_written)),
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
                // The want and the target have no menu for the rules: until this was here, a
                // model slower than the patience asked them again forever.
                self.unasked(&ask.kind, memory, log);
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

    /// The rules' answer to a `want` or `witness` question the model did not answer: a want at
    /// random, and the first player in the memory as the target.
    fn unasked(&mut self, kind: &str, memory: &mut Memory, log: &mut log::Log) {
        match kind {
            "want" => {
                memory.want = WANT_CHOICES[(self.rng.next_u64() % 3) as usize].to_string();
                log.note(&format!("it wants {} (the rules' pick)", memory.want));
            }
            "witness" => {
                if let Some((i, name, uuid)) = memory.others.first().cloned() {
                    log.note(&format!("the target is {name} (the rules' pick)"));
                    memory.set_target(i, &name, &uuid);
                }
            }
            _ => {}
        }
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
        let chosen = rules_brain::choose_with(
            &menu,
            arc.stage,
            &memory.want,
            &|effect| memory.notice_rate(effect),
            &memory.chain_next(),
            &mut self.rng,
        );
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

        self.ask_for_words(chosen, &lines, memory, config, model);

        // The day's budget counts the plans already held: they spend when they run, and three
        // held at once would all pass a check that saw only what was spent.
        let held: f32 = self
            .plans
            .iter()
            .filter(|p| !p.forced)
            .filter_map(|p| events::find(&p.event))
            .map(|e| e.cost_in(config))
            .sum();
        if !arc.affords(chosen.cost_in(config) + held, now.day, config) {
            log.note("the day's budget is spent");
            self.writings.retain(|w| w.event != chosen.id);
            return;
        }
        if self.clashes(chosen, pos) {
            log.note(&format!("{} clashes with a plan held", chosen.id));
            self.writings.retain(|w| w.event != chosen.id);
            return;
        }
        // At the cap only a reaction plans (decide waits below it): the oldest plan whose moment
        // has not come makes way for it.
        if self.plans.len() >= config.brain.most_plans.max(1) as usize {
            let oldest = self
                .plans
                .iter()
                .filter(|p| !p.forced && p.from_tick > now.tick)
                .min_by_key(|p| p.id)
                .cloned();
            match oldest {
                Some(old) => {
                    // It never ran, so no record in the memory: the log and the story say so.
                    let why = format!("made way for {}", chosen.id);
                    log.note(&format!("cancelled {}: {why}", old.event));
                    self.outcome(&old.event, &format!("cancelled: {why}"));
                    self.writings.retain(|w| w.event != old.event);
                    self.written.retain(|(e, _)| *e != old.event);
                    self.plans.retain(|p| p.id != old.id);
                }
                None => {
                    log.note(&format!("no room for {}: every plan is due", chosen.id));
                    self.writings.retain(|w| w.event != chosen.id);
                    return;
                }
            }
        }

        // Soon: within four planning intervals (six minutes at the default `plan_every`, a
        // minute in a dev config), when its conditions hold; dropped a day after that.
        let when = when.unwrap_or_default();
        // The model answers `When?` before it knows the event, so a moment the event's needs
        // rule out is dropped: `wall_swap` (away or asleep) planned "as they come home" waited
        // its whole day in the dev client.
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
        let id = self.next_plan_id();
        self.plans.push(Plan {
            id,
            event: chosen.id.to_string(),
            pos,
            from_tick: now.tick + delay,
            until_tick: now.tick + delay + 24_000,
            decided_day: day,
            forced: false,
            out_of_sight: false,
            when,
            waiting: String::new(),
            on: pos
                .and_then(|p| memory.thing_at(p))
                .unwrap_or_default()
                .to_string(),
            words_asked: true,
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

        // Fire replaces a plan for the same event; the others wait on.
        self.plans.retain(|p| p.event != event.id);
        let id = self.next_plan_id();
        self.plans.push(Plan {
            id,
            event: event.id.to_string(),
            pos,
            from_tick: 0,
            until_tick: u64::MAX,
            decided_day: day,
            forced: true,
            out_of_sight: false,
            when: String::new(),
            waiting: String::new(),
            on: String::new(),
            words_asked: act::text_kind(event).is_none(),
        });
        // Where, so the log's line is a link to it on the debug screen (owner).
        log.note(&format!(
            "forced {}{}",
            event.id,
            pos.map(|p| format!(" at {}", log::spot_words(p)))
                .unwrap_or_default()
        ));

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
        // The fight refused by the host (creative, a fight on already): over at once, not left
        // to freeze every plan till its timeout.
        for report in reports {
            if memory.confront_id != 0 && report.id == memory.confront_id {
                memory.confront_id = 0;
                if !report.done {
                    log.note(&format!("the fight did not begin: {}", report.reason));
                    memory.fight_ended = Some("refused".to_string());
                }
            }
        }
        for report in reports {
            if let Some(index) = self.pending.iter().position(|p| p.id == report.id) {
                let pending = self.pending.remove(index);
                // A put back that could not go in (the chest gone, or filled since): dropped by
                // the chest, never kept for good.
                if !report.done {
                    if let Some(back) = &pending.back {
                        let id = self.id();
                        self.pending.push(Pending {
                            id,
                            event: pending.event.clone(),
                            pos: pending.pos,
                            day: pending.day,
                            tail: true,
                            ..Pending::default()
                        });
                        self.later.push(Later {
                            at_tick: 0,
                            command: Command::DropItem {
                                id,
                                pos: [
                                    back.from[0] as f64 + 0.5,
                                    back.from[1] as f64 + 1.0,
                                    back.from[2] as f64 + 0.5,
                                ],
                                item: craftkit::world::ItemStack {
                                    id: back.item.clone(),
                                    count: back.count.max(1),
                                    slot: 0,
                                },
                                kept: back.keep.clone(),
                            },
                            while_still: false,
                        });
                    }
                }
                if report.done {
                    if let Some(took) = pending
                        .took
                        .clone()
                        .filter(|t| t.keep != "held" && (!t.keep.is_empty() || !t.item.is_empty()))
                    {
                        self.taken.push(Taken {
                            event: pending.event.clone(),
                            day: pending.day,
                            took,
                        });
                    }
                }
                let others = self
                    .pending
                    .iter()
                    .any(|p| !p.tail && p.event == pending.event && p.day == pending.day);
                let by_spot = SPOT_BY_SPOT.contains(&pending.event.as_str());
                let this = (pending.event.clone(), pending.day);
                if by_spot && report.done && !self.spots_done.contains(&this) {
                    self.spots_done.push(this.clone());
                }
                if pending.tail {
                    if !report.done {
                        log.note(&format!("{}, after it: {}", pending.event, report.reason));
                    }
                } else if by_spot && others {
                    if !report.done {
                        log.note(&format!(
                            "{}: a spot skipped, {}",
                            pending.event, report.reason
                        ));
                    }
                } else if by_spot {
                    // Its last spot: done if any spot was.
                    let any = self.spots_done.contains(&this);
                    self.spots_done.retain(|s| *s != this);
                    let reason = if any { "" } else { report.reason.as_str() };
                    memory.record(&pending.event, pending.pos, pending.day, any, reason);
                    log.reported(&pending.event, any, reason);
                    self.outcome(&pending.event, if any { "done" } else { reason });
                } else if !others || !report.done {
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
                    let mine = |t: &Taken| t.event == pending.event && t.day == pending.day;
                    if report.done {
                        // Over: what it took is where the event put it.
                        self.taken.retain(|t| !mine(t));
                    } else {
                        // The rest of a failed event is not sent: the sign's text after a sign
                        // that did not go down, the take after a planting that failed, the
                        // stand placed for a piece never taken.
                        let dropped: Vec<u64> = self
                            .pending
                            .iter()
                            .filter(|p| p.event == pending.event && p.day == pending.day)
                            .map(|p| p.id)
                            .collect();
                        self.later.retain(|l| !dropped.contains(&l.command.id()));
                        self.pending.retain(|p| !dropped.contains(&p.id));
                        // What it took for it goes back into the chest it came from, or it was
                        // kept for good (the scarecrow's piece when the stand would not move),
                        // or spent for nothing (the stick of a craft whose coal was gone).
                        self.put_back(&pending.event, pending.day, pending.pos);
                    }
                }

                // A build's ground, found by the host on its first share: kept for the rest.
                if let Some(base) = report
                    .reason
                    .strip_prefix("base ")
                    .and_then(|r| r.split_whitespace().next())
                    .and_then(|y| y.parse::<i32>().ok())
                {
                    let kind = crate::building::build_of(&pending.event).to_string();
                    if let Some(build) = memory.builds.iter_mut().find(|b| b.kind == kind) {
                        if !build.ground_known() {
                            build.site[1] = base;
                        }
                    }
                }

                // A ripe crop harvested: its own item to their chest, whichever crop it was.
                if let Some(item) = report
                    .reason
                    .strip_prefix("replaced ")
                    .filter(|_| report.done && pending.event == "harvest_one")
                    .and_then(act::harvest_of)
                {
                    if let Some(chest) = memory.used_container().map(|c| c.pos) {
                        let id = self.id();
                        self.later.push(Later {
                            at_tick: 0,
                            command: Command::PutItem {
                                id,
                                into: chest,
                                slot: None,
                                item: craftkit::world::ItemStack {
                                    id: item.to_string(),
                                    count: 1,
                                    slot: 0,
                                },
                                kept: String::new(),
                                wear: 0,
                                name: String::new(),
                            },
                            while_still: false,
                        });
                    }
                }
                // A dig's block is the lodger's now ("replaced minecraft:dirt"): grass dug is dirt.
                if let Some(was) = report
                    .reason
                    .strip_prefix("replaced ")
                    .filter(|_| report.done && !NOT_DUG.contains(&pending.event.as_str()))
                {
                    // What a pick would give: stone is cobblestone, an ore its drop.
                    let held = crate::tunnels::dug_as(was);
                    let to_chest = pending.event == "mine_beside" && was.ends_with("_ore");
                    match memory.used_container().map(|c| c.pos).filter(|_| to_chest) {
                        // The ore of the tunnel it dug beside theirs goes to their chest.
                        Some(chest) => {
                            let id = self.id();
                            self.later.push(Later {
                                // At once: the next tick sends what is due.
                                at_tick: 0,
                                command: Command::PutItem {
                                    id,
                                    into: chest,
                                    slot: None,
                                    item: craftkit::world::ItemStack {
                                        id: held,
                                        count: 1,
                                        slot: 0,
                                    },
                                    kept: String::new(),
                                    wear: 0,
                                    name: String::new(),
                                },
                                while_still: false,
                            })
                        }
                        None => memory.store(&held, 1),
                    }
                }
                if report.done && pending.puts_held {
                    memory.held = None;
                }
                // The night's prints kept, the ones the command drew.
                if report.done && pending.event == "prints_around_the_house" {
                    let n = memory.times("prints_around_the_house").saturating_sub(1);
                    let new = crate::prints::night(memory, n);
                    memory.keep_prints(new);
                }
                // Up there from now, as far as it knows; fallen back, no longer.
                if report.done && pending.event == "taken_up" && memory.taken_up_day.is_none() {
                    memory.taken_up_day = Some(memory.today);
                }
                if report.done && pending.event == "sky_gives_back" {
                    memory.taken_up_day = None;
                }
                if report.done && pending.event == "take" {
                    // The host took it; the memory holds it from here, what the command took.
                    match &pending.took {
                        Some(took) if !took.item.is_empty() => act::took_this(memory, took),
                        _ => act::took(memory, pending.pos),
                    }
                }
                if report.done
                    && (pending.event == "return_taken"
                        || pending.event == "place_taken"
                        || pending.event == "villager_word"
                        || pending.event == "chicken_lays"
                        || pending.event == "alcove")
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
/// Why a plan can no longer happen, if it cannot: a rule that now refuses it outright (the
/// stage moved on, the config disabled it, its cap reached), a lasting need gone (the chest,
/// the home: `Need::lasting`), or what was at its spot gone (`Plan::on`). A need that comes and
/// goes, a cooldown or the view only make it wait.
fn cancel_reason(
    event: &Event,
    plan: &Plan,
    memory: &Memory,
    stage: u8,
    day: f32,
    config: &Config,
) -> Option<String> {
    if let Err(
        refused @ (rules::Refused::Disabled | rules::Refused::NotOffered | rules::Refused::Capped),
    ) = rules::check(event, plan.pos, memory, stage, day, config)
    {
        return Some(refused.words().to_string());
    }
    if let Some(need) = event
        .needs
        .iter()
        .find(|n| n.lasting() && !offer::meets(**n, memory, &Now::default()))
    {
        return Some(format!("no longer: {}", need.words()));
    }
    if !plan.on.is_empty() {
        let still = plan.pos.and_then(|p| memory.thing_at(p));
        if still != Some(plan.on.as_str()) {
            return Some(format!("its {} is gone", plan.on));
        }
    }

    None
}

/// Events whose "replaced" block is not dug into the lodger's store: a light put out, a light
/// swapped for one it brought, a flower potted, a crop harvested into their chest. Only
/// `one_room_dark` was here before, and the rest each made a torch, a lantern or a flower from
/// nothing.
const NOT_DUG: &[&str] = &[
    "one_room_dark",
    "light_taken",
    "village_lantern",
    "flowers_potted",
    "harvest_one",
];

/// Whether a planned moment can ever hold together with the event's needs: "as they come home"
/// never meets "away or asleep", "while they sleep" never meets "walking" or "outdoors", and
/// "while they are away" never meets "in bed" or "indoors" (the home's rooms).
pub fn when_fits(event: &Event, when: &str) -> bool {
    use events::Need::*;
    let rules_out: &[events::Need] = match when {
        "home" => &[TargetAwayOrAsleep, TargetInBed],
        // Awake, eating, walking to a door or the chest never happen asleep, nor near the
        // home while away; until now such a plan waited out its whole window.
        "sleep" => &[
            TargetWalking,
            TargetOutdoors,
            TargetAwake,
            TargetAte,
            TargetApproachingDoor,
            TargetApproachingChest,
        ],
        "away" => &[
            TargetInBed,
            TargetIndoors,
            TargetApproachingDoor,
            TargetApproachingChest,
        ],
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
            gpu_layers: 0,
            pause_ms: 1,
        };
        Model::with_worker(settings, Worker::start(move || Ok(fake)).unwrap())
    }

    fn home_memory() -> Memory {
        let mut memory = Memory {
            target: Some(0),
            ..Memory::default()
        };
        memory.home = Some(Home {
            outs: Vec::new(),
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
        assert!(!brain.plans.is_empty());
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
            in_home: false,
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
            if !memory.want.is_empty() && !brain.plans.is_empty() {
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
        let plan = brain.plans.first().expect("a plan");
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
            if !brain.plans.is_empty() {
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
            if !brain.written.is_empty() || !memory.lines_written.is_empty() {
                break;
            }
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // The model's pick is waiting for the sign, or already on it when the plan ran in the
        // same tick the answer came.
        let line = brain
            .written
            .first()
            .map(|(_, l)| l.as_str())
            .or(memory.lines_written.first().map(String::as_str));
        assert_eq!(line, Some("under"), "{:?}", brain.written);
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
        let plan = brain.plans.first().cloned().unwrap();
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
        // The dev client: a path of torches went up 40 blocks off, in plain sight.
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
        brain.plans = vec![Plan {
            event: "light_path".to_string(),
            until_tick: 24_000,
            ..Plan::default()
        }];
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
        let plan = brain.plans.first().expect("still planned, not dropped");
        assert_eq!(plan.waiting, "in view");

        memory.last.yaw = 90.0; // facing -x, away from it all
        now.tick += 20;
        let sent = brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert!(
            !brain.plans.iter().any(|p| p.event == "light_path")
                && sent.iter().any(|c| matches!(c, Command::SetBlock { .. })),
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
        brain.plans = vec![Plan {
            event: "close_door".to_string(),
            pos: Some([3, 64, 0]),
            until_tick: 24_000,
            ..Plan::default()
        }];
        let now = Now {
            tick: 20,
            ..Now::default()
        };
        brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert_eq!(
            brain.plans.first().map(|p| p.waiting.as_str()),
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
        // The dev client: `wall_swap` planned "as they come home" never came.
        let swap = events::find("wall_swap").unwrap();
        assert!(!when_fits(swap, "home"));
        assert!(when_fits(swap, "sleep"));
        assert!(when_fits(swap, "away"));
        assert!(when_fits(swap, ""));
        assert!(!when_fits(events::find("open_chest").unwrap(), "away"));
        assert!(!when_fits(events::find("late_steps").unwrap(), "sleep"));
        // Awake is never asleep.
        assert!(!when_fits(events::find("name_called").unwrap(), "sleep"));
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
        // The dev client: `hole` dug under a crafting table at the lodger's place.
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
            brain.plans = vec![Plan {
                event: "hole".to_string(),
                pos: Some([343, 64, 50]),
                until_tick: 24_000,
                forced,
                ..Plan::default()
            }];
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
            assert!(
                !brain.plans.iter().any(|p| p.event == "hole"),
                "refused, not waiting"
            );
            assert_eq!(
                memory.done.last().map(|d| d.reason.as_str()),
                Some("too near what they built")
            );
        }
    }

    /// Ticks a brain a second at a time with `heard` typed on the first tick, until it sends a
    /// chat line or `seconds` pass; the line sent, if any.
    #[allow(clippy::too_many_arguments)]
    fn type_and_wait(
        brain: &mut Brain,
        memory: &mut Memory,
        arc: &mut Arc,
        config: &Config,
        model: Option<&Model>,
        now: &mut Now,
        heard: &str,
        about: bool,
        seconds: u64,
    ) -> Option<String> {
        let mut log = log::Log::new(None, 16, false);
        memory.heard = Some((heard.to_string(), about));
        for _ in 0..seconds {
            for command in brain.tick(now, 0.0, memory, arc, config, &mut log, model) {
                if let Command::GameMessage { text, message, .. } = command {
                    if message == "says" {
                        return Some(text);
                    }
                }
            }
            now.tick += TPS;
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        None
    }

    fn talking(stage: u8) -> (Config, Memory, Arc) {
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.events.cooldown_scale = 0.0;
        config.brain.plan_every = 100_000; // no planning in the way
        let mut memory = home_memory();
        memory.last.online = true;
        memory.want = "a place of its own".to_string();
        let mut arc = Arc {
            stage,
            ..Arc::default()
        };
        arc.tick(0.0, &memory, &config);
        (config, memory, arc)
    }

    #[test]
    fn a_line_about_it_is_answered_within_a_minute_in_the_model_s_words() {
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let (config, mut memory, mut arc) = talking(2);
        let model = model_with(Fake::new("nothing yet", 0, Ok("You know who.")));
        let mut brain = Brain::new(11);
        let mut now = Now {
            tick: 20,
            ..Now::default()
        };
        // A chance, not a certainty: they keep asking until it answers.
        let mut said = None;
        for _ in 0..12 {
            said = type_and_wait(
                &mut brain,
                &mut memory,
                &mut arc,
                &config,
                Some(&model),
                &mut now,
                "who is in my house",
                true,
                70,
            );
            if said.is_some() {
                break;
            }
        }

        assert_eq!(said.as_deref(), Some("you know who"));
        model.shutdown();
    }

    #[test]
    fn nothing_is_answered_in_stage_one_or_with_chat_off() {
        for (stage, chat) in [(1, true), (2, false)] {
            let (mut config, mut memory, mut arc) = talking(stage);
            config.events.chat = chat;
            let mut brain = Brain::new(5);
            let mut now = Now {
                tick: 20,
                ..Now::default()
            };
            for _ in 0..10 {
                let said = type_and_wait(
                    &mut brain,
                    &mut memory,
                    &mut arc,
                    &config,
                    None,
                    &mut now,
                    "hello who is there",
                    true,
                    70,
                );
                assert_eq!(said, None, "stage {stage}, chat {chat}");
            }
        }
    }

    #[test]
    fn at_most_three_answers_a_day_from_the_bank_without_a_model() {
        let (config, mut memory, mut arc) = talking(3);
        let mut brain = Brain::new(9);
        let mut now = Now {
            tick: 20,
            ..Now::default()
        };
        let mut answers = Vec::new();
        for _ in 0..30 {
            if let Some(line) = type_and_wait(
                &mut brain,
                &mut memory,
                &mut arc,
                &config,
                None,
                &mut now,
                "is someone in my house",
                true,
                70,
            ) {
                answers.push(line);
            }
        }

        assert_eq!(answers.len(), REPLIES_A_DAY, "{answers:?}");
        for line in &answers {
            assert!(Kind::Reply.examples().contains(&line.as_str()), "{line}");
        }
    }

    #[test]
    fn plans_held_count_against_the_day_s_budget_before_they_run() {
        // Plans spend when they run; a check that saw only what was spent let three be held
        // on a budget of one (the scripted stay's command count rose past its bound).
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.stages.events_per_day = [1.0, 1.0, 1.0, 1.0, 0.0];
        config.brain.plan_every = 1;
        config.brain.most_plans = 3;
        config.events.cooldown_scale = 0.0;
        let mut memory = home_memory();
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(4);
        // One plan held that cannot run for a long while: the day's one event.
        let id = brain.next_plan_id();
        brain.plans.push(Plan {
            id,
            event: "late_steps".to_string(),
            from_tick: 1_000_000,
            until_tick: 2_000_000,
            ..Plan::default()
        });
        for tick in (20..4_000).step_by(20) {
            let now = Now {
                tick,
                ..Now::default()
            };
            brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        }

        assert!(
            brain.decisions > 10,
            "it kept deciding: {}",
            brain.decisions
        );
        // What ran and what is still held never pass the day's one event.
        let held: f32 = brain
            .plans
            .iter()
            .filter_map(|p| events::find(&p.event))
            .map(|e| e.cost_in(&config))
            .sum();
        assert!(
            arc.spent_today + held <= 1.0 + 1e-6,
            "spent {} with {held} still held: {:?}",
            arc.spent_today,
            brain.plans
        );
    }

    #[test]
    fn plans_run_side_by_side_and_one_is_cancelled_when_its_door_is_gone() {
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 1;
        config.events.cooldown_scale = 0.0;
        let mut memory = home_memory();
        // The target in the game at home: what waits for them to be away or asleep waits, so
        // plans are held side by side (away from the game, every such plan ran the moment it was
        // chosen, and once the catalogue grew none was left waiting beside another).
        memory.last.online = true;
        memory.last.pos = [0.5, 64.0, 0.5];
        let mut arc = Arc {
            stage: 3,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(2);
        let mut now = Now {
            tick: 20,
            ..Now::default()
        };
        // One plan held, waiting far ahead: the brain must choose another beside it. Left to
        // chance, two rarely overlapped once most events send at once.
        let held = brain.next_plan_id();
        brain.plans.push(Plan {
            id: held,
            event: "knock".to_string(),
            from_tick: now.tick + 50_000,
            until_tick: now.tick + 60_000,
            ..Plan::default()
        });
        for _ in 0..40 {
            brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
            if brain.plans.len() >= 2 {
                break;
            }
            now.tick += TPS;
        }
        assert!(brain.plans.len() >= 2, "{:?}", brain.plans);
        let ids: Vec<u64> = brain.plans.iter().map(|p| p.id).collect();
        let mut events: Vec<&str> = brain.plans.iter().map(|p| p.event.as_str()).collect();
        events.dedup();
        assert_eq!(events.len(), brain.plans.len(), "never one event twice");
        assert!(ids.windows(2).all(|w| w[0] != w[1]));

        // A plan on the door, and the door broken: cancelled with its reason.
        let id = brain.next_plan_id();
        brain.plans.push(Plan {
            id,
            event: "close_door".to_string(),
            pos: Some([3, 64, 0]),
            from_tick: now.tick + 10_000,
            until_tick: now.tick + 30_000,
            on: "door".to_string(),
            ..Plan::default()
        });
        memory.home.as_mut().unwrap().doors.clear();
        brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert!(!brain.plans.iter().any(|p| p.id == id), "{:?}", brain.plans);
        assert!(
            memory.done.iter().any(|d| d.event == "close_door"
                && d.reason.starts_with("no longer")
                || d.reason == "its door is gone"),
            "{:?}",
            memory.done
        );
    }

    #[test]
    fn a_plan_dropped_or_run_now_from_the_screen_says_which() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        for event in ["open_door", "cut_music"] {
            let id = brain.next_plan_id();
            brain.plans.push(Plan {
                id,
                event: event.to_string(),
                from_tick: 5_000,
                until_tick: 30_000,
                ..Plan::default()
            });
        }
        assert_eq!(brain.plan_now(2), "cut_music on the next tick");
        assert!(brain.plans[1].forced && brain.plans[1].from_tick == 0);
        assert_eq!(
            brain.drop_plan(1, 0.0, &mut memory, &mut log),
            "dropped open_door"
        );
        assert_eq!(brain.plans.len(), 1);
        assert_eq!(brain.drop_plan(9, 0.0, &mut memory, &mut log), "no plan 9");
    }

    #[test]
    fn a_sign_that_did_not_go_down_gets_no_text_sent_after_it() {
        let config = Config::default();
        let mut memory = home_memory();
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        let id = brain.next_plan_id();
        brain.plans.push(Plan {
            id,
            event: "sign".to_string(),
            pos: Some([8, 64, 0]),
            until_tick: 24_000,
            forced: true,
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
        let block = sent
            .iter()
            .find(|c| matches!(c, Command::SetBlock { .. }))
            .expect("the sign's block went out");
        assert!(
            brain
                .later
                .iter()
                .any(|l| matches!(l.command, Command::SetSign { .. })),
            "the text waits two ticks"
        );

        brain.report(
            &[Report {
                id: block.id(),
                done: false,
                reason: "nothing holds it there".to_string(),
            }],
            &mut memory,
            &mut log,
        );
        assert!(
            !brain
                .later
                .iter()
                .any(|l| matches!(l.command, Command::SetSign { .. })),
            "no text for a sign that is not there"
        );
    }

    #[test]
    fn a_fired_line_is_the_model_s_and_a_line_it_repeats_is_dropped_for_a_fresh_one() {
        // The dev client: Fire never asked the model, and `kill_line` went round
        // the bank's six lines.
        let _turn = crate::model::TEST_TURN
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let config = Config::default();
        let mut memory = home_memory();
        memory.last.online = true;
        memory.tally.push(crate::memory::Tally {
            kind: "salmon".to_string(),
            hits: 0,
            kills: 17,
        });
        let mut arc = Arc {
            stage: 3,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let model = model_with(Fake::new("nothing yet", 0, Ok("the river is quieter now")));
        let mut brain = Brain::new(3);
        let mut now = Now {
            tick: 20,
            ..Now::default()
        };
        let mut fire_and_read = |brain: &mut Brain, memory: &mut Memory, now: &mut Now| {
            let event = events::find("kill_line").unwrap();
            brain.force(event, memory, 0.0, &mut log);
            for _ in 0..200 {
                for command in
                    brain.tick(now, 0.0, memory, &mut arc, &config, &mut log, Some(&model))
                {
                    if let Command::GameMessage { text, .. } = command {
                        return text;
                    }
                }
                now.tick += 2;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            panic!("nothing said");
        };

        let first = fire_and_read(&mut brain, &mut memory, &mut now);
        assert_eq!(
            first, "the river is quieter now",
            "the model's line, not the bank's"
        );
        let second = fire_and_read(&mut brain, &mut memory, &mut now);
        assert_ne!(second, first, "the same line again is dropped");
        assert!(
            Kind::KillLine
                .examples()
                .iter()
                .any(|e| text::fill(e, "salmon", 17) == second),
            "a fresh bank line instead: {second}"
        );
        model.shutdown();
    }

    #[test]
    fn opening_one_chest_another_closes_somewhere_in_the_house_now_and_then() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        config.brain.plan_every = 100_000;
        let mut memory = home_memory();
        memory.last.online = true;
        for pos in [[2, 64, 0], [-3, 64, 1]] {
            memory.containers.push(crate::memory::Container {
                pos,
                kind: "minecraft:chest".to_string(),
                placed: true,
                ..Default::default()
            });
        }
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(6);
        let mut now = Now {
            tick: 20,
            ..Now::default()
        };
        let mut answered = Vec::new();
        for _ in 0..40 {
            memory.opened_now = Some([2, 64, 0]);
            for command in brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None) {
                if let Command::PlaySound { pos, .. } = command {
                    answered.push(pos);
                }
            }
            now.tick += 40;
        }

        assert!(!answered.is_empty(), "never in forty openings");
        assert!(answered.len() < 30, "only now and then: {}", answered.len());
        assert!(
            answered
                .iter()
                .all(|p| (p[0] - -2.5).abs() < 0.01 && (p[2] - 1.5).abs() < 0.01),
            "the other chest, never the one opened: {answered:?}"
        );
    }

    #[test]
    fn a_failed_event_puts_what_it_took_back_in_its_chest_and_stays_a_failure() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        // The scarecrow from the chest: the piece taken (done), the stand taken (fails), then
        // the stand placed and the piece worn, later.
        let took = Took {
            from: [2, 64, 0],
            slot: 7,
            item: "minecraft:iron_helmet".to_string(),
            keep: "scarecrow".to_string(),
            count: 1,
        };
        for id in [1, 2, 3, 4] {
            brain.pending.push(Pending {
                id,
                event: "scarecrow".to_string(),
                day: 5.0,
                took: (id == 1).then(|| took.clone()),
                ..Pending::default()
            });
        }
        brain.later.push(Later {
            at_tick: 100,
            command: Command::PlaceEntity {
                id: 3,
                of: "minecraft:armor_stand".to_string(),
                pos: [0.0, 64.0, 0.0],
                yaw: 0.0,
                seconds: 0.0,
                kept: "scarecrow".to_string(),
                face: None,
                targets: None,
                harm: 0.0,
                wears: Vec::new(),
            },
            while_still: false,
        });
        brain.later.push(Later {
            at_tick: 100,
            command: Command::EquipEntity {
                id: 4,
                entity: 9,
                item: craftkit::world::ItemStack::default(),
                kept: "scarecrow".to_string(),
            },
            while_still: false,
        });
        let report = |id, done| Report {
            id,
            done,
            reason: if done {
                String::new()
            } else {
                "not there any more".to_string()
            },
        };
        brain.report(&[report(1, true), report(2, false)], &mut memory, &mut log);

        assert_eq!(
            brain.later.len(),
            1,
            "no stand from nothing: {:?}",
            brain.later
        );
        let back = match &brain.later[0].command {
            Command::PutItem {
                id,
                into,
                slot,
                kept,
                ..
            } if *into == [2, 64, 0] && *slot == Some(7) && kept == "scarecrow" => *id,
            other => panic!("the piece back in its chest, not {other:?}"),
        };
        assert!(brain.taken.is_empty());
        brain.report(&[report(back, true)], &mut memory, &mut log);
        let scarecrow: Vec<_> = memory
            .done
            .iter()
            .filter(|d| d.event == "scarecrow")
            .collect();
        assert_eq!(scarecrow.len(), 1, "{scarecrow:?}");
        assert!(!scarecrow[0].ok, "put back, it is still a failure");
        assert!(memory.landmarks.iter().all(|d| d.event != "scarecrow"));
    }

    #[test]
    fn a_craft_that_fails_gives_back_the_ingredient_it_spent() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        for (id, took) in [
            (
                1,
                Some(Took {
                    from: [2, 64, 0],
                    slot: 4,
                    item: "minecraft:stick".to_string(),
                    keep: String::new(),
                    count: 1,
                }),
            ),
            (2, None),
        ] {
            brain.pending.push(Pending {
                id,
                event: "crafted_for_you".to_string(),
                day: 5.0,
                took,
                ..Pending::default()
            });
        }
        let report = |id, done| Report {
            id,
            done,
            reason: String::new(),
        };
        brain.report(&[report(1, true), report(2, false)], &mut memory, &mut log);
        assert!(
            matches!(&brain.later[..], [Later { command: Command::PutItem { item, kept, into, .. }, .. }]
                if item.id == "minecraft:stick" && kept.is_empty() && *into == [2, 64, 0]),
            "the stick back: {:?}",
            brain.later
        );
    }

    #[test]
    fn a_put_back_that_cannot_go_in_is_dropped_by_the_chest_and_abandon_puts_back() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        brain.taken.push(Taken {
            event: "scarecrow".to_string(),
            day: 5.0,
            took: Took {
                from: [2, 64, 0],
                slot: 7,
                item: "minecraft:iron_helmet".to_string(),
                keep: "scarecrow".to_string(),
                count: 1,
            },
        });
        brain.abandon("scarecrow", 5.0);
        let back = match &brain.later[..] {
            [Later {
                command: Command::PutItem { id, kept, .. },
                ..
            }] if kept == "scarecrow" => *id,
            other => panic!("put back: {other:?}"),
        };
        assert!(brain.taken.is_empty());
        brain.later.clear();
        brain.report(
            &[Report {
                id: back,
                done: false,
                reason: "the container is full".to_string(),
            }],
            &mut memory,
            &mut log,
        );
        assert!(
            matches!(&brain.later[..], [Later { command: Command::DropItem { kept, pos, .. }, .. }]
                if kept == "scarecrow" && pos[0] == 2.5),
            "dropped by the chest: {:?}",
            brain.later
        );
    }

    #[test]
    fn a_path_of_torches_skips_the_spot_with_no_footing_and_is_done() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        for id in 1..=4 {
            brain.pending.push(Pending {
                id,
                event: "light_path".to_string(),
                day: 5.0,
                ..Pending::default()
            });
        }
        brain.later.push(Later {
            at_tick: 100,
            command: Command::SetBlock {
                id: 4,
                pos: [9, 64, 9],
                block: "minecraft:torch".to_string(),
                state: String::new(),
                replace: vec![],
                settle: 8,
                wall: String::new(),
                rest: false,
                only: false,
            },
            while_still: false,
        });
        let report = |id, done| Report {
            id,
            done,
            reason: if done {
                String::new()
            } else {
                "nothing holds it there".to_string()
            },
        };
        brain.report(
            &[report(1, true), report(2, true), report(3, false)],
            &mut memory,
            &mut log,
        );
        assert_eq!(brain.later.len(), 1, "the next torch still goes");
        assert!(
            memory.done.iter().all(|d| d.event != "light_path"),
            "not over yet"
        );
        brain.report(&[report(4, true)], &mut memory, &mut log);
        assert!(
            memory.done.iter().any(|d| d.event == "light_path" && d.ok),
            "done, a spot skipped"
        );
        assert!(brain.spots_done.is_empty());
    }

    #[test]
    fn a_take_holds_what_its_command_took_not_the_plans_chest() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        brain.pending.push(Pending {
            id: 1,
            event: "take".to_string(),
            pos: Some([99, 64, 99]),
            day: 5.0,
            took: Some(Took {
                from: [2, 64, 0],
                slot: 3,
                item: "minecraft:bread".to_string(),
                keep: "held".to_string(),
                count: 1,
            }),
            ..Pending::default()
        });
        brain.report(
            &[Report {
                id: 1,
                done: true,
                reason: String::new(),
            }],
            &mut memory,
            &mut log,
        );
        let held = memory.held.as_ref().expect("held");
        assert_eq!(held.item.id, "minecraft:bread");
        assert_eq!((held.from, held.slot), ([2, 64, 0], 3));
        assert!(
            brain.taken.is_empty(),
            "held is the take's own, not put back"
        );
    }

    #[test]
    fn the_villager_walked_back_by_morning_does_not_hold_the_night_unrecorded() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        for (id, tail) in [(1, false), (2, false), (3, true)] {
            brain.pending.push(Pending {
                id,
                event: "villager_at_door".to_string(),
                day: 5.0,
                tail,
                ..Pending::default()
            });
        }
        let done = |id| Report {
            id,
            done: true,
            reason: String::new(),
        };
        brain.report(&[done(1), done(2)], &mut memory, &mut log);
        assert!(
            memory
                .done
                .iter()
                .any(|d| d.event == "villager_at_door" && d.ok),
            "recorded as the villager stands there, not by morning"
        );
        brain.report(&[done(3)], &mut memory, &mut log);
        assert_eq!(
            memory
                .done
                .iter()
                .filter(|d| d.event == "villager_at_door")
                .count(),
            1,
            "and once"
        );
    }

    #[test]
    fn what_it_digs_it_holds_and_what_it_gives_from_nothing_comes_out_of_that() {
        let mut memory = home_memory();
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        for (id, was) in [
            (7, "minecraft:grass_block"),
            (8, "minecraft:dirt"),
            (9, "minecraft:stone"),
        ] {
            brain.pending.push(Pending {
                id,
                event: "hole".to_string(),
                ..Pending::default()
            });
            brain.report(
                &[Report {
                    id,
                    done: true,
                    reason: format!("replaced {was}"),
                }],
                &mut memory,
                &mut log,
            );
        }
        assert_eq!(memory.stored("minecraft:dirt"), 2, "grass dug is dirt");
        assert_eq!(
            memory.stored("minecraft:cobblestone"),
            1,
            "stone dug is cobblestone, as a pick gives (it used to be kept as stone)"
        );

        memory.store("minecraft:dirt", 1);
        memory.containers.push(crate::memory::Container {
            pos: [2, 64, 0],
            kind: "minecraft:chest".to_string(),
            placed: true,
            ..Default::default()
        });
        let id = brain.next_plan_id();
        brain.plans.push(Plan {
            id,
            event: "chest_full_of_one".to_string(),
            pos: Some([2, 64, 0]),
            until_tick: 24_000,
            forced: true,
            ..Plan::default()
        });
        let mut arc = Arc {
            stage: 3,
            ..Arc::default()
        };
        let sent = brain.tick(
            &Now {
                tick: 20,
                ..Now::default()
            },
            0.0,
            &mut memory,
            &mut arc,
            &Config::default(),
            &mut log,
            None,
        );
        let given = sent
            .iter()
            .chain(brain.later.iter().map(|l| &l.command))
            .filter(|c| matches!(c, Command::PutItem { .. }))
            .count();
        assert_eq!(given, 3, "the three it held");
        assert_eq!(memory.stored("minecraft:dirt"), 0, "and holds none now");
    }

    #[test]
    fn the_home_is_read_when_they_come_home_after_they_build_and_not_every_tick() {
        let config = Config::default();
        let mut memory = home_memory();
        memory.last.online = true;
        memory.last.pos = [100.0, 64.0, 0.0];
        let mut arc = Arc {
            stage: 1,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        let home_reads = |sent: &[Command], memory: &Memory| {
            sent.iter()
                .filter(|c| matches!(c, Command::Scan { id, .. } if *id == memory.home_scan.0))
                .count()
        };
        let mut tick = |brain: &mut Brain, memory: &mut Memory, at: u64| {
            brain.tick(
                &Now {
                    tick: at,
                    ..Now::default()
                },
                0.0,
                memory,
                &mut arc,
                &config,
                &mut log,
                None,
            )
        };
        let sent = tick(&mut brain, &mut memory, 20);
        assert_eq!(home_reads(&sent, &memory), 1, "read once to begin with");
        let sent = tick(&mut brain, &mut memory, 40);
        assert_eq!(home_reads(&sent, &memory), 0, "not every tick");

        memory.last.pos = [1.0, 64.0, 1.0];
        let sent = tick(&mut brain, &mut memory, 60);
        assert_eq!(home_reads(&sent, &memory), 1, "they came home");

        memory.home_dirty = true;
        let sent = tick(&mut brain, &mut memory, 80);
        assert_eq!(
            home_reads(&sent, &memory),
            0,
            "half a minute after building, not at once"
        );
        let sent = tick(&mut brain, &mut memory, 60 + 31 * TPS);
        assert_eq!(home_reads(&sent, &memory), 1);
    }

    #[test]
    fn the_village_is_read_while_they_are_near_it_every_five_minutes() {
        let config = Config::default();
        let mut memory = home_memory();
        memory.last.online = true;
        memory.surroundings = Some(crate::memory::Surround {
            day: 0.0,
            around: [0, 64, 0],
            features: vec![craftkit::world::Feature {
                kind: "village".to_string(),
                pos: [200, 64, 0],
                size: 40,
                detail: String::new(),
            }],
        });
        memory.last.pos = [0.0, 64.0, 0.0];
        let mut arc = Arc {
            stage: 1,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        let village_reads = |memory: &Memory| {
            memory
                .scans_out
                .iter()
                .filter(|(_, e)| e == "village_read")
                .count()
        };
        let mut tick = |brain: &mut Brain, memory: &mut Memory, at: u64| {
            brain.tick(
                &Now {
                    tick: at,
                    ..Now::default()
                },
                0.0,
                memory,
                &mut arc,
                &config,
                &mut log,
                None,
            );
        };
        tick(&mut brain, &mut memory, 20);
        assert_eq!(village_reads(&memory), 0, "far from it");
        memory.last.pos = [180.0, 64.0, 0.0];
        tick(&mut brain, &mut memory, 40);
        assert_eq!(village_reads(&memory), 1, "near it");
        tick(&mut brain, &mut memory, 60);
        assert_eq!(village_reads(&memory), 1, "not again at once");
        tick(&mut brain, &mut memory, 40 + 300 * TPS);
        assert_eq!(village_reads(&memory), 2, "five minutes on");
    }

    #[test]
    fn an_offering_is_armed_for_fresh_loot_its_own_thing_first_and_again_once_given() {
        let config = Config::default();
        let mut memory = home_memory();
        memory.last.online = true;
        memory.held = Some(crate::memory::Held {
            item: craftkit::world::ItemStack {
                id: "minecraft:iron_pickaxe".to_string(),
                count: 1,
                slot: 4,
            },
            from: [2, 64, 0],
            slot: 4,
            taken_day: 0.0,
        });
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        let mut at = 20;
        let mut tick = |brain: &mut Brain, memory: &mut Memory, arc: &mut Arc| {
            at += 20;
            brain.tick(
                &Now {
                    tick: at,
                    ..Now::default()
                },
                0.0,
                memory,
                arc,
                &config,
                &mut log,
                None,
            )
        };
        let armed = |sent: &[Command]| {
            sent.iter().find_map(|c| match c {
                Command::ArmLoot {
                    item, kept, chance, ..
                } => Some((item.id.clone(), kept.clone(), *chance)),
                _ => None,
            })
        };

        let sent = tick(&mut brain, &mut memory, &mut arc);
        assert_eq!(
            armed(&sent),
            Some((
                "minecraft:iron_pickaxe".to_string(),
                "held".to_string(),
                LOOT_CHANCE
            )),
            "what it took, the very one"
        );
        assert_eq!(
            armed(&tick(&mut brain, &mut memory, &mut arc)),
            None,
            "one at a time"
        );

        // The host gave it: the memory holds it no more, and arms again.
        let mut seen = craftkit::world::Observations::default();
        seen.events.push(craftkit::world::Event {
            player: 0,
            kind: craftkit::world::EventKind::LootTouched,
            pos: Some([300, 40, 12]),
            detail: "minecraft:iron_pickaxe".to_string(),
            contents: vec![],
        });
        memory.observe(&seen, 0.0);
        memory.last.online = true; // the observation carried no players
        assert!(memory.held.is_none());
        assert!(memory.done.iter().any(|d| d.event == "loot_gift" && d.ok));
        let again = armed(&tick(&mut brain, &mut memory, &mut arc)).expect("armed again");
        assert!(
            LOOT_FROM_NOWHERE.contains(&again.0.as_str()),
            "holding nothing, a thing from the short list: {again:?}"
        );

        arc.stage = 1;
        memory.loot_armed = false;
        assert_eq!(
            armed(&tick(&mut brain, &mut memory, &mut arc)),
            None,
            "not before stage two"
        );
    }

    #[test]
    fn an_alcove_may_dig_beside_their_wall_where_nothing_else_may() {
        let config = Config::default();
        let mut memory = home_memory();
        memory
            .walls
            .push(([4, 64, 0], "minecraft:oak_planks".to_string()));
        memory.placed.push([4, 64, 0]);
        let mut arc = Arc {
            stage: 4,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(1);
        let id = brain.next_plan_id();
        brain.plans.push(Plan {
            id,
            event: "alcove".to_string(),
            pos: Some([4, 64, 0]),
            until_tick: 24_000,
            forced: true,
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
            sent.iter().any(|c| act::dug(c).is_some()),
            "dug beside the wall: {sent:?}"
        );
    }

    #[test]
    fn a_door_they_open_now_and_then_shuts_again_a_moment_later() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        config.brain.plan_every = 100_000;
        let mut memory = home_memory();
        memory.last.online = true;
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(3);
        let mut shut = 0;
        for i in 0..40u64 {
            memory.door_used_now = Some([3, 64, 0]);
            let now = Now {
                tick: 20 + i * 40,
                ..Now::default()
            };
            brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
            shut += brain
                .later
                .iter()
                .filter(
                    |l| matches!(l.command, Command::ToggleBlock { pos, .. } if pos == [3, 64, 0]),
                )
                .count();
            brain.later.clear();
        }
        assert!(shut > 0 && shut < 30, "now and then: {shut} of forty");
    }

    #[test]
    fn back_from_another_world_a_while_something_happened_at_home() {
        let config = Config::default();
        let mut memory = home_memory();
        memory.last.online = true;
        memory.place = Some([30, 64, 30]);
        memory.store("minecraft:cobblestone", 16);
        let mut arc = Arc {
            stage: 3,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(9);
        let now = Now {
            tick: 20,
            ..Now::default()
        };

        // Gone a moment: nothing.
        memory.back_after = Some(AWAY_LEAST / 2.0);
        brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert!(brain.plans.iter().all(|p| !p.forced), "{:?}", brain.plans);

        // Gone a while: one of the bold things, run at once, in view or not, so it is done
        // before they can see it.
        memory.back_after = Some(AWAY_LEAST * 4.0);
        let sent = brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        let busy = memory
            .recent(8)
            .find(|d| d.event == "busy_away")
            .expect("busy while they were away")
            .clone();
        assert!(WHILE_AWAY.contains(&busy.reason.as_str()), "{busy:?}");
        let ran: Vec<u64> = brain
            .pending
            .iter()
            .filter(|p| p.event == busy.reason)
            .map(|p| p.id)
            .collect();
        assert!(!ran.is_empty(), "{} sent nothing", busy.reason);
        assert!(
            sent.iter().any(|c| ran.contains(&c.id())),
            "at once: {sent:?}"
        );
    }

    #[test]
    fn every_while_away_event_is_in_the_catalogue() {
        for id in WHILE_AWAY {
            let event = events::find(id).unwrap_or_else(|| panic!("{id}"));
            assert!(!event.small, "{id} is a bold one");
        }
    }

    #[test]
    fn what_it_holds_is_held_until_a_put_of_it_is_done_and_a_restart_keeps_a_reply_due() {
        let mut memory = home_memory();
        memory.held = Some(crate::memory::Held {
            item: craftkit::world::ItemStack {
                id: "minecraft:clock".to_string(),
                count: 1,
                slot: 0,
            },
            ..Default::default()
        });
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(10);
        brain.pending.push(Pending {
            id: 7,
            event: "house_near".to_string(),
            puts_held: true,
            ..Pending::default()
        });
        brain.report(
            &[Report {
                id: 7,
                done: false,
                reason: "the container is full".to_string(),
            }],
            &mut memory,
            &mut log,
        );
        assert!(memory.held.is_some(), "a failed put keeps it held");
        brain.pending.push(Pending {
            id: 8,
            event: "house_near".to_string(),
            puts_held: true,
            ..Pending::default()
        });
        brain.report(
            &[Report {
                id: 8,
                done: true,
                reason: String::new(),
            }],
            &mut memory,
            &mut log,
        );
        assert!(memory.held.is_none(), "put away, held no more");

        // The game restarts with a reply due: it is still due, and the house's read with it.
        brain.reply = Some(ChatReply {
            due_tick: 5_000_100,
            until_tick: 5_002_000,
            ..ChatReply::default()
        });
        brain.next_here_scan = 5_000_600;
        brain.rebase(5_000_000);
        let reply = brain.reply.as_ref().unwrap();
        assert_eq!((reply.due_tick, reply.until_tick), (100, 2_000));
        assert_eq!(brain.next_here_scan, 600);
    }

    #[test]
    fn come_to_the_offer_s_figure_and_what_it_held_out_is_at_their_feet() {
        let mut memory = home_memory();
        memory.held = Some(crate::memory::Held {
            item: craftkit::world::ItemStack {
                id: "minecraft:clock".to_string(),
                count: 1,
                slot: 0,
            },
            ..Default::default()
        });
        memory.offer_waiting = Some([4, 64, 0]);
        // The host: the figure gone as they came within three blocks.
        memory.observe(
            &craftkit::world::Observations {
                players: vec![craftkit::world::Player {
                    index: 0,
                    pos: [3.0, 64.0, 0.5],
                    ..Default::default()
                }],
                events: vec![craftkit::world::Event {
                    player: 0,
                    kind: craftkit::world::EventKind::FigureSeen,
                    pos: Some([4, 64, 0]),
                    detail: "looked=40 looks=1 toward=5 end=approached".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            1.0,
        );
        assert_eq!(memory.offer_waiting, None);
        let mut brain = Brain::new(10);
        let mut arc = Arc {
            stage: 4,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let made = brain.tick(
            &Now::default(),
            1.0,
            &mut memory,
            &mut arc,
            &Config::default(),
            &mut log,
            None,
        );
        let Some(Command::DropItem { id, kept, item, .. }) =
            made.iter().find(|c| matches!(c, Command::DropItem { .. }))
        else {
            panic!("nothing dropped: {made:?}");
        };
        assert_eq!(
            (kept.as_str(), item.id.as_str()),
            ("held", "minecraft:clock")
        );
        brain.report(
            &[Report {
                id: *id,
                done: true,
                reason: String::new(),
            }],
            &mut memory,
            &mut log,
        );
        assert!(memory.held.is_none(), "given back, held no more");

        // Gone any other way, it keeps it.
        memory.held = Some(crate::memory::Held::default());
        memory.offer_waiting = Some([4, 64, 0]);
        memory.observe(
            &craftkit::world::Observations {
                players: vec![craftkit::world::Player {
                    index: 0,
                    pos: [30.0, 64.0, 0.5],
                    ..Default::default()
                }],
                events: vec![craftkit::world::Event {
                    player: 0,
                    kind: craftkit::world::EventKind::FigureSeen,
                    pos: Some([4, 64, 0]),
                    detail: "end=timed".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            1.0,
        );
        assert_eq!(
            (memory.offer_waiting, memory.offer_reached_now),
            (None, None)
        );
    }

    #[test]
    fn a_fight_the_host_refuses_is_over_at_once() {
        let mut memory = home_memory();
        memory.confront_id = 9;
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(11);
        brain.report(
            &[Report {
                id: 9,
                done: false,
                reason: "never in creative or spectator".to_string(),
            }],
            &mut memory,
            &mut log,
        );
        assert_eq!(memory.fight_ended.as_deref(), Some("refused"));
        assert_eq!(memory.confront_id, 0);
    }

    #[test]
    fn a_door_they_knock_on_knocks_back_now_and_then() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        config.brain.plan_every = 100_000;
        let mut memory = home_memory();
        memory.last.online = true;
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(8);
        let most = events::find("answer_knock").unwrap().most_per_stay;
        let mut answered = Vec::new();
        for i in 0..40u64 {
            memory.knocked_now = Some([3, 64, 0]);
            let now = Now {
                tick: 20 + i * 40,
                ..Now::default()
            };
            brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
            assert!(memory.knocked_now.is_none(), "the knock is taken");
            if !brain.later.is_empty() {
                answered.push(i);
            }
            brain.later.clear();
        }
        assert_eq!(
            answered.len() as u32,
            most,
            "answered to its cap: {answered:?}"
        );
        assert!(
            *answered.last().unwrap() >= most as u64,
            "not every knock: {answered:?}"
        );
    }

    #[test]
    fn crouched_still_it_breathes_at_their_ear_until_they_move() {
        let mut config = Config::default();
        config.events.cooldown_scale = 0.0;
        config.brain.plan_every = 100_000;
        let mut memory = home_memory();
        memory.last.online = true;
        let mut arc = Arc {
            stage: 3,
            ..Arc::default()
        };
        let mut log = log::Log::new(None, 16, false);
        let mut brain = Brain::new(5);
        let mut tick = 20;
        // Crouched still: the breathing starts now and then, at their ear.
        let mut started = false;
        for _ in 0..40 {
            memory.crouched_now = true;
            memory.last.crouched_still = true;
            let now = Now {
                tick,
                ..Now::default()
            };
            brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
            tick += 1;
            if !brain.later.is_empty() {
                started = true;
                break;
            }
        }
        assert!(started, "never in forty crouches");
        assert!(
            brain.later.iter().all(|l| l.while_still),
            "every breath waits on their stillness"
        );
        // Still crouched: the first breath, at their ear, and more to come.
        let now = Now {
            tick,
            ..Now::default()
        };
        let sent = brain.tick(&now, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        let first = sent.iter().find_map(|c| match c {
            Command::PlaySound { pos, sound, .. } => Some((*pos, sound.clone())),
            _ => None,
        });
        let (pos, sound) = first.expect("the first breath");
        assert_eq!(sound, "minecraft:entity.player.breath");
        assert!(
            (pos[1] - (memory.last.pos[1] + 1.6)).abs() < 0.01,
            "at head height: {pos:?}"
        );
        assert!(!brain.later.is_empty(), "more breaths to come");

        // They stand and walk: nothing more is heard.
        memory.last.crouched_still = false;
        let later = Now {
            tick: tick + 400,
            ..Now::default()
        };
        let sent = brain.tick(&later, 0.0, &mut memory, &mut arc, &config, &mut log, None);
        assert!(
            !sent.iter().any(|c| matches!(c, Command::PlaySound { .. })),
            "dropped as they moved: {sent:?}"
        );
        assert!(brain.later.is_empty());
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
            outs: Vec::new(),
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
            if !brain.plans.is_empty() {
                break;
            }
            now.tick += 15 * TPS;
        }

        let plan = brain.plans.first().expect("a plan");
        assert!(
            plan.from_tick - now.tick <= 4 * 15 * TPS,
            "{} ticks ahead",
            plan.from_tick - now.tick
        );
        assert_eq!(plan.until_tick, plan.from_tick + 24_000);
    }
}
