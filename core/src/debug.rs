//! The lodger's pages on the host's debug screen: its gauges (stage, budget, plan, what the
//! memory holds), its lines (the situation as the brain reads it, the plan, the last decisions),
//! every event with whether it is offered right now and why not, and the buttons (force an
//! event, set a stage, plan now, clear the plan, forget cooldowns, forget everything). The
//! screen itself is the host's; see `docs/sdk.md`, The debug screen.

use crate::brain::log::Log;
use crate::brain::offer;
use crate::config::Config;
use crate::events::{self, CATALOGUE};
use crate::rules;
use crate::stay::Stay;
use craftkit::debug::{ActionRow, DebugReply, DebugRequest, EventRow, Gauge, Item, Section};
use craftkit::world::Now;

/// Answers one request about one stay. `now` is the stay's idea of the clock from its last
/// tick, since a debug call comes between ticks.
pub fn answer(
    stay: &mut Stay,
    request: &DebugRequest,
    config: &Config,
    log: &mut Log,
    now: &Now,
) -> DebugReply {
    let day = stay.day(now);

    match request {
        DebugRequest::Snapshot => snapshot(stay, config, log, now, day),
        DebugRequest::Force { event, player } => {
            let Some(row) = events::find(event) else {
                return DebugReply::message(format!("no event called {event}"));
            };
            // With its name and uuid, as a target found by watching has (a bare index signed
            // its books "you" and was lost after a restart).
            if stay.memory.target.is_none() {
                let known = stay
                    .memory
                    .others
                    .iter()
                    .find(|(i, _, _)| i == player)
                    .map(|(_, name, uuid)| (name.clone(), uuid.clone()))
                    .unwrap_or_default();
                stay.memory.set_target(*player, &known.0, &known.1);
            }
            if !offer::switched_on(row, config) {
                return DebugReply::message(format!(
                    "{} is in a group the config turned off (events.sounds, glitches or chat)",
                    row.id
                ));
            }
            let message = stay.brain.force(row, &stay.memory, day, log);
            let mut reply = snapshot(stay, config, log, now, day);
            reply.message = message;
            reply
        }
        DebugRequest::SetStage { stage } => {
            if *stage > 6 {
                return DebugReply::message("stages run 0 to 5; 6 is over");
            }
            stay.arc.set_stage(*stage, day);
            log.note(&format!("stage set to {stage} from the debug screen"));
            let mut reply = snapshot(stay, config, log, now, day);
            reply.message = format!("stage {stage}");
            reply
        }
        DebugRequest::Action { id } => {
            let message = match id.as_str() {
                "move_in" => {
                    stay.arc.stage = stay.arc.stage.max(1);
                    stay.arc.stage_began_day = day;
                    "moved in: stage 1".to_string()
                }
                "next_stage" => {
                    stay.arc.set_stage((stay.arc.stage + 1).min(6), day);
                    format!("stage {}", stay.arc.stage)
                }
                "plan_now" => {
                    stay.brain.next_plan_tick = 0;
                    "the brain decides on the next tick, if it holds fewer plans than it may"
                        .to_string()
                }
                "clear_plan" => {
                    let ids: Vec<u64> = stay.brain.plans.iter().map(|p| p.id).collect();
                    for id in &ids {
                        stay.brain.drop_plan(*id, day, &mut stay.memory, log);
                    }
                    // The commands queued for an event under way go, and so do the host's waits
                    // on them (a wait left behind kept the event from being recorded); what is
                    // queued for no event (an ore or a crop on its way to their chest, the
                    // ending's beats) stays, or it was lost.
                    let waited: Vec<u64> = stay.brain.pending.iter().map(|p| p.id).collect();
                    let dropped: Vec<u64> = stay
                        .brain
                        .later
                        .iter()
                        .map(|l| l.command.id())
                        .filter(|id| waited.contains(id))
                        .collect();
                    stay.brain
                        .later
                        .retain(|l| !dropped.contains(&l.command.id()));
                    // What was sent already still reports and is recorded.
                    stay.brain.pending.retain(|p| !dropped.contains(&p.id));
                    // What the events under way took goes back to its chest.
                    stay.brain.abandon_all();
                    match ids.len() {
                        0 => "there were no plans".to_string(),
                        n => format!("dropped {n} plan(s)"),
                    }
                }
                other if other.starts_with("plan_drop:") || other.starts_with("plan_now:") => {
                    let (verb, number) = other.split_once(':').unwrap_or_default();
                    match number.parse::<u64>() {
                        Ok(id) if verb == "plan_drop" => {
                            stay.brain.drop_plan(id, day, &mut stay.memory, log)
                        }
                        Ok(id) => stay.brain.plan_now(id),
                        Err(_) => format!("no plan {number}"),
                    }
                }
                "forget_cooldowns" => {
                    stay.memory.done.clear();
                    stay.arc.spent_today = 0.0;
                    "every event may happen again, and the day's budget is whole".to_string()
                }
                "fight_now" => {
                    // Stage five, an ending chosen and its script begun, and the fight asked for
                    // where they stand: for trying it without waiting weeks.
                    if stay.arc.stage != 5 {
                        stay.arc.set_stage(5, day);
                    }
                    if stay.arc.ending.is_empty() {
                        stay.arc.ending = "keep".to_string();
                        stay.arc.script_day = crate::ending::first_day(now);
                    }
                    stay.memory.fight_now = true;
                    "the fight begins where they stand".to_string()
                }
                // `/craftkit lodger status`: where the stay is, in a few lines.
                "status" => status_lines(stay, day, config),
                // `/craftkit lodger leave`: the leaves ending, begun at the next dawn.
                "leave" => {
                    if stay.arc.over() {
                        format!("it is gone already ({})", stay.arc.ended)
                    } else if !stay.arc.moved_in() {
                        stay.arc.stage = 6;
                        stay.arc.ended = "left".to_string();
                        "it will not move in now".to_string()
                    } else {
                        if stay.arc.stage != 5 {
                            stay.arc.set_stage(5, day);
                        }
                        stay.arc.ending = "leave".to_string();
                        stay.arc.script_day = crate::ending::first_day(now);
                        stay.arc.beats_done = 0;
                        "it is leaving: its last three days begin at the next dawn, and \
                         everything it holds goes back"
                            .to_string()
                    }
                }
                // `/craftkit lodger reset`: the stay forgotten and begun again, what it holds
                // given back first.
                "reset" => {
                    // What events under way took, put back where it came from, with the rest.
                    let first = stay.brain.next_command_id;
                    stay.brain.abandon_all();
                    let put_back: Vec<crate::brain::Later> = stay
                        .brain
                        .later
                        .iter()
                        .filter(|l| l.command.id() >= first)
                        .cloned()
                        .collect();
                    let give_back = {
                        let brain = &mut stay.brain;
                        let mut ids = || {
                            let id = brain.next_command_id;
                            brain.next_command_id += 1;
                            id
                        };
                        crate::ending::commands(
                            crate::ending::Beat::ReturnHeld,
                            &mut stay.memory,
                            now,
                            &mut ids,
                        )
                    };
                    let next_id = stay.brain.next_command_id;
                    let (target, name, uuid) = (
                        stay.memory.target,
                        stay.memory.target_name.clone(),
                        stay.memory.target_uuid.clone(),
                    );
                    let mods = std::mem::take(&mut stay.memory.mods);
                    let defer = stay.memory.defer_target;
                    *stay = Stay::new(&craftkit::world::WorldInfo {
                        id: stay.world.clone(),
                        seed: stay.seed,
                        ..Default::default()
                    });
                    stay.brain.next_command_id = next_id;
                    stay.memory.target = target;
                    stay.memory.target_name = name;
                    stay.memory.target_uuid = uuid;
                    stay.memory.mods = mods;
                    stay.memory.defer_target = defer;
                    stay.brain.later.extend(put_back);
                    let gave = !give_back.is_empty();
                    for (command, wait) in give_back {
                        stay.brain.later.push(crate::brain::Later {
                            at_tick: now.tick + wait,
                            command,
                            while_still: false,
                        });
                    }
                    if gave {
                        "forgotten: the stay begins again, and what it held goes back".to_string()
                    } else {
                        "forgotten: the stay begins again".to_string()
                    }
                }
                "forget_all" => {
                    let target = stay.memory.target;
                    let name = stay.memory.target_name.clone();
                    let uuid = stay.memory.target_uuid.clone();
                    // Kept too: the mods loaded (sent once, at the first tick), the pick of
                    // target from the config, and what it holds of theirs, so nothing is lost.
                    let mods = std::mem::take(&mut stay.memory.mods);
                    let defer = stay.memory.defer_target;
                    let held = stay.memory.held.take();
                    stay.memory = Default::default();
                    stay.memory.target = target;
                    stay.memory.target_name = name;
                    stay.memory.target_uuid = uuid;
                    stay.memory.mods = mods;
                    stay.memory.defer_target = defer;
                    stay.memory.held = held;
                    stay.brain.plans.clear();
                    stay.brain.later.clear();
                    stay.brain.pending.clear();
                    // What the events under way took goes back to its chest, after the clear.
                    stay.brain.abandon_all();
                    "the memory is empty; it learns the house again from here".to_string()
                }
                other => format!("no action called {other}"),
            };
            log.note(&format!("debug: {message}"));
            let mut reply = snapshot(stay, config, log, now, day);
            reply.message = message;
            reply
        }
    }
}

/// When the next decision comes, in words: before it moves in there is none (the brain decides
/// nothing until then, and "in about 0s" stood there for days).
fn next_decision(stay: &Stay, config: &Config, now: &Now, day: f32) -> String {
    if !stay.arc.moved_in() {
        let left = (config.stages.start_after_days as f32 - day).max(0.0);
        return format!("no decision until it moves in, in about {left:.1} days");
    }
    if stay.brain.asking.is_some() {
        return "a decision now: the model is thinking".to_string();
    }
    format!(
        "a decision in about {}s",
        stay.brain.next_plan_tick.saturating_sub(now.tick) / 20
    )
}

fn snapshot(stay: &Stay, config: &Config, log: &Log, now: &Now, day: f32) -> DebugReply {
    let memory = &stay.memory;
    let arc = &stay.arc;
    let brain = &stay.brain;

    let mut gauges = vec![Gauge::new(
        "stage",
        arc.stage as f32,
        5.0,
        if arc.stage == 0 {
            format!(
                "not moved in; day {day:.1} of {}",
                config.stages.start_after_days
            )
        } else if arc.over() {
            "over".to_string()
        } else {
            format!("stage {} of 5", arc.stage)
        },
    )];
    if arc.moved_in() {
        let index = (arc.stage - 1) as usize;
        let in_stage = (day - arc.stage_began_day).max(0.0);
        let most = config.stages.most_days[index] as f32;
        gauges.push(Gauge::new(
            "days in stage",
            in_stage,
            most.max(0.1),
            format!(
                "{in_stage:.1} of at most {most:.0} days, {} landed",
                arc.landed_this_stage
            ),
        ));
        let budget = arc.budget(config);
        let spent = if arc.budget_day == now.day {
            arc.spent_today
        } else {
            0.0
        };
        gauges.push(Gauge::new(
            "today's budget",
            spent,
            budget.max(0.1),
            format!("{spent:.1} of {budget:.1} events"),
        ));
    }
    let most = config.brain.most_plans.max(1);
    gauges.push(Gauge::new(
        "plans",
        brain.plans.len() as f32,
        most as f32,
        match soonest(brain) {
            Some(plan) => format!(
                "{} of {most}; next {} {}",
                brain.plans.len(),
                plan.event,
                plan_when(plan, now)
            ),
            None => format!("none of {most}; {}", next_decision(stay, config, now, day)),
        },
    ));
    // The other mods: how many, and the first few by name.
    gauges.push(Gauge::new(
        "mods",
        memory.mods.len() as f32,
        memory.mods.len().max(1) as f32,
        if memory.mods.is_empty() {
            "no other mods told".to_string()
        } else {
            let names: Vec<&str> = memory
                .mods
                .iter()
                .take(6)
                .map(|(id, _)| id.as_str())
                .collect();
            format!(
                "{} other mods: {}{}",
                memory.mods.len(),
                names.join(", "),
                if memory.mods.len() > 6 { ", ..." } else { "" }
            )
        },
    ));
    gauges.push(Gauge::new(
        "decisions",
        brain.decisions as f32,
        (brain.decisions.max(10)) as f32,
        format!(
            "{} decisions, {} events landed, {} did not",
            brain.decisions,
            memory
                .done
                .iter()
                .filter(|d| d.ok && d.event != "wait")
                .count(),
            memory.done.iter().filter(|d| !d.ok).count()
        ),
    ));
    gauges.push(Gauge::new(
        "want",
        if memory.want.is_empty() { 0.0 } else { 1.0 },
        1.0,
        format!(
            "{}{}",
            if memory.want.is_empty() {
                "not chosen yet".to_string()
            } else {
                memory.want.clone()
            },
            if memory.last.mood.is_empty() {
                String::new()
            } else {
                format!("; they seem {}", memory.last.mood)
            }
        ),
    ));
    // Its temper (`temper.rs`): the value, where it rests, and what moved it last.
    let temper = &memory.temper;
    gauges.push(Gauge::new(
        "temper",
        temper.value,
        1.0,
        format!(
            "{:.2}, resting {:.2}{}{}",
            temper.value,
            temper.resting,
            if !config.events.temper {
                ", off"
            } else if temper.value >= temper.resting + 0.2 {
                ", angry"
            } else if temper.value <= temper.resting - 0.15 {
                ", pleased"
            } else {
                ""
            },
            temper
                .causes
                .last()
                .map(|c| format!("; last: {} ({:+.2})", c.what, c.by))
                .unwrap_or_default()
        ),
    ));
    // The players (`multiplayer.rs`): the target, the others here, and a change of target.
    gauges.push(Gauge::new(
        "players",
        memory.others_online.len() as f32,
        memory.others.len().max(1) as f32,
        format!(
            "{}; {} other{} here{}",
            if memory.target_name.is_empty() {
                "no target yet".to_string()
            } else {
                format!("lives with {}", memory.target_name)
            },
            memory.others_online.len(),
            if memory.others_online.len() == 1 {
                ""
            } else {
                "s"
            },
            if memory.retargeted {
                ", the target changed once"
            } else {
                ""
            }
        ),
    ));
    gauges.push(Gauge::new(
        "brain",
        if config.model.enabled { 1.0 } else { 0.0 },
        1.0,
        if config.model.enabled {
            format!("the model decides ({})", config.model.size)
        } else {
            "the rules decide: playing without the model".to_string()
        },
    ));
    gauges.push(Gauge::new(
        "memory",
        (memory.containers.len() + memory.fixtures.len() + memory.decor.len()) as f32,
        64.0,
        format!(
            "{} chests, {} fixtures, {} decor, {} placed blocks, {} creatures near",
            memory.containers.len(),
            memory.fixtures.len(),
            memory.decor.len(),
            memory.placed.len(),
            memory.last.creatures_near.len()
        ),
    ));
    if !memory.tally.is_empty() {
        let kills: u32 = memory.tally.iter().map(|t| t.kills).sum();
        gauges.push(Gauge::new(
            "kills",
            kills as f32,
            (kills.max(10)) as f32,
            kills_words(&memory.tally),
        ));
    }

    let mut sections = vec![Section::new("As the brain reads it", memory.describe(day))];
    let mut plans = Section::new("Plans", Vec::new());
    for plan in &brain.plans {
        plans.items.push(Item {
            text: format!(
                "#{} {}{} at {}",
                plan.id,
                plan.event,
                if plan.forced { " (forced)" } else { "" },
                plan.pos
                    .map(crate::brain::log::spot_words)
                    .unwrap_or_else(|| "no place".to_string())
            ),
            detail: plan_when(plan, now),
            actions: vec![
                ActionRow::new(&format!("plan_drop:{}", plan.id), "Drop"),
                ActionRow::new(&format!("plan_now:{}", plan.id), "Now"),
            ],
        });
    }
    if plans.items.is_empty() {
        plans.lines.push(format!(
            "nothing planned; {}",
            next_decision(stay, config, now, day)
        ));
    }
    sections.push(plans);
    let mut plan_lines = Vec::new();
    for pending in &brain.pending {
        plan_lines.push(format!(
            "waiting on the host: {} (command {})",
            pending.event, pending.id
        ));
    }
    for later in &brain.later {
        plan_lines.push(format!(
            "in {} ticks: {}",
            later.at_tick.saturating_sub(now.tick),
            serde_json::to_string(&later.command).unwrap_or_default()
        ));
    }
    if !plan_lines.is_empty() {
        sections.push(Section::new("Under way", plan_lines));
    }
    // The chains (`chains.rs`): each open one with its steps landed and the next, then those
    // walked to the end and those that fell out.
    let mut chain_lines = Vec::new();
    for open in &memory.chains {
        if let Some(chain) = crate::chains::find(&open.id) {
            let steps: Vec<String> = chain
                .steps
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let name = s.events.join("/");
                    if open.landed.contains(&i) {
                        format!("[{name}]")
                    } else {
                        name
                    }
                })
                .collect();
            chain_lines.push(format!(
                "{}: {} (last step day {:.1}{})",
                open.id,
                steps.join(" > "),
                open.landed_day,
                if memory.chain_next().iter().any(|e| open
                    .next(chain)
                    .is_some_and(|i| chain.steps[i].events.contains(e)))
                {
                    ", next weighed up"
                } else {
                    ", next waiting on them"
                }
            ));
        }
    }
    let walked = &memory.chains_walked;
    let fell: Vec<&String> = memory
        .chains_done
        .iter()
        .filter(|d| !walked.contains(d))
        .collect();
    if !walked.is_empty() {
        chain_lines.push(format!("walked to the end: {}", walked.join(", ")));
    }
    if !fell.is_empty() {
        chain_lines.push(format!(
            "fell out: {}",
            fell.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if chain_lines.is_empty() {
        chain_lines.push("none begun".to_string());
    }
    sections.push(Section::new("Chains", chain_lines));
    // How the figure was looked at, the last few (`figure_seen`).
    if !memory.figure_looks.is_empty() {
        sections.push(Section::new(
            "Figure looks",
            memory
                .figure_looks
                .iter()
                .rev()
                .take(6)
                .map(|l| {
                    format!(
                        "day {:.2}: {:.1}s in {} look{}, {:.0} blocks toward it, {}{}",
                        l.day,
                        l.looked_ticks as f32 / 20.0,
                        l.looks,
                        if l.looks == 1 { "" } else { "s" },
                        l.toward,
                        l.end,
                        if l.chased() {
                            ", chased"
                        } else if l.looked_long() {
                            ", looked long"
                        } else {
                            ""
                        }
                    )
                })
                .collect(),
        ));
    }
    sections.push(Section::new(
        "Done lately",
        memory
            .recent(12)
            .map(|d| {
                format!(
                    "day {:.2}: {} {}{}",
                    d.day,
                    d.event,
                    if d.ok { "done" } else { "not done" },
                    if d.reason.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", d.reason)
                    }
                )
            })
            .collect(),
    ));
    let mut fixtures: Vec<String> = Vec::new();
    for f in &memory.fixtures {
        fixtures.push(format!(
            "{} at ({},{},{})",
            f.kind, f.pos[0], f.pos[1], f.pos[2]
        ));
    }
    for d in &memory.decor {
        fixtures.push(format!(
            "{} #{} at ({},{},{})",
            d.kind, d.id, d.pos[0], d.pos[1], d.pos[2]
        ));
    }
    for c in &memory.containers {
        fixtures.push(format!(
            "{} at ({},{},{}), opened {} times, {} stacks",
            c.kind.rsplit(':').next().unwrap_or(&c.kind),
            c.pos[0],
            c.pos[1],
            c.pos[2],
            c.opens,
            c.contents.len()
        ));
    }
    fixtures.truncate(40);
    sections.push(Section::new("Known blocks", fixtures));
    sections.push(Section::new(
        "Creatures near",
        memory
            .last
            .creatures_near
            .iter()
            .take(16)
            .map(|c| {
                format!(
                    "{} #{}{}{}",
                    c.kind,
                    c.id,
                    if c.pet { ", pet" } else { "" },
                    if c.named { ", named" } else { "" }
                )
            })
            .collect(),
    ));
    // The story: each decision, newest first, with what it saw, what it was offered, what it
    // chose and what came of it (owner: "show events that happened, what is about
    // to happen, why it happened").
    let mut story = Vec::new();
    if let Some(plan) = soonest(brain) {
        story.push(format!(
            "NEXT: {}{} {} ({} planned)",
            plan.event,
            plan.pos
                .map(|p| format!(" at {}", crate::brain::log::spot_words(p)))
                .unwrap_or_default(),
            plan_when(plan, now),
            brain.plans.len()
        ));
    } else {
        story.push(format!("NEXT: {}", next_decision(stay, config, now, day)));
    }
    for decision in brain.story.iter().rev().take(12) {
        story.push(format!(
            "day {:.2}, stage {}: chose {}{} from [{}] -> {}",
            decision.day,
            decision.stage,
            decision.chosen,
            decision
                .pos
                .map(|p| format!(" at {}", crate::brain::log::spot_words(p)))
                .unwrap_or_default(),
            decision.menu.join(" "),
            if decision.outcome.is_empty() {
                "waiting for its moment"
            } else {
                &decision.outcome
            }
        ));
        if decision.chosen != "wait" {
            for line in decision.situation.iter().take(4) {
                story.push(format!("    because: {line}"));
            }
        }
    }
    sections.insert(1, Section::new("Story", story));
    sections.push(Section::new(
        "Never forgotten",
        memory
            .landmarks
            .iter()
            .rev()
            .take(10)
            .map(|l| format!("day {:.2}: {}", l.day, l.event))
            .chain(
                memory
                    .milestones
                    .iter()
                    .rev()
                    .take(6)
                    .map(|m| format!("day {:.2}: {} {}", m.day, m.kind, m.detail)),
            )
            .collect(),
    ));
    sections.push(Section::new(
        "Decision log",
        log.lines.iter().rev().take(6).rev().cloned().collect(),
    ));

    let events = CATALOGUE
        .iter()
        .filter(|event| event.id != "wait")
        .map(|event| {
            let times = memory
                .done
                .iter()
                .filter(|d| d.event == event.id && d.ok)
                .count() as u32;
            let why = if !crate::temper::offered_in(event, arc.stage, &memory.temper) {
                format!("stage {}", event.stages_words())
            } else if let Some(unmet) = offer::unmet(event, memory, now) {
                unmet
            } else if let Err(refused) = rules::check(event, None, memory, arc.stage, day, config) {
                refused.words().to_string()
            } else {
                String::new()
            };
            EventRow {
                id: event.id.to_string(),
                words: event.menu_words().to_string(),
                stages: event.stages_words(),
                offered: why.is_empty(),
                why,
                times,
                about: about(event, memory, config),
            }
        })
        .collect();

    DebugReply {
        message: String::new(),
        gauges,
        sections,
        events,
        actions: vec![
            ActionRow::new("move_in", "Move in now"),
            ActionRow::new("next_stage", "Next stage"),
            ActionRow::new("plan_now", "Decide now"),
            ActionRow::new("clear_plan", "Drop the plan"),
            ActionRow::new("forget_cooldowns", "Forget cooldowns"),
            ActionRow::new("fight_now", "Fight it now"),
            ActionRow::new("forget_all", "Forget everything"),
        ],
    }
}

/// An event's tooltip: what the catalogue says it does and needs, its cooldown and cap as the
/// config scales them, and how it went last. Lines split by `\n`.
fn about(event: &crate::events::Event, memory: &crate::memory::Memory, config: &Config) -> String {
    let mut lines = Vec::new();
    if let Some((what, needs)) = events::described(event.id) {
        lines.push(what.replace('`', ""));
        lines.push(format!("Needs: {}", needs.replace('`', "")));
    }
    lines.push(format!(
        "Stages {}; cooldown {:.2} days; at most {} a stay; {}",
        event.stages_words(),
        event.cooldown_days * config.events.cooldown_scale as f32,
        event.most_per_stay,
        if event.cost() < 1.0 { "small" } else { "full" }
    ));
    match memory.done.iter().rev().find(|d| d.event == event.id) {
        Some(done) if done.ok => lines.push(format!("Last: day {:.2}, done", done.day)),
        Some(done) => lines.push(format!(
            "Last: day {:.2}, not done: {}",
            done.day, done.reason
        )),
        None => lines.push("Never yet in this world".to_string()),
    }

    lines.join("\n")
}

/// The plan due soonest, forced ones first.
fn soonest(brain: &crate::brain::Brain) -> Option<&crate::brain::Plan> {
    brain.plans.iter().min_by_key(|p| (!p.forced, p.from_tick))
}

/// When a plan happens, in the screen's words: `next tick (forced)`, `in 40s`, or why it waits.
fn plan_when(plan: &crate::brain::Plan, now: &Now) -> String {
    if plan.forced {
        "on the next tick (forced)".to_string()
    } else if plan.from_tick > now.tick {
        format!("in {}s", (plan.from_tick - now.tick) / 20)
    } else if plan.waiting.is_empty() {
        "due".to_string()
    } else {
        plan.waiting.clone()
    }
}

/// Every kind killed, most first, with the hits: `11 sheep, 3 pigs, 1 creeper; 18 hits`. Six
/// kinds at most, then how many more. The dev client's line named the first kind alone, so a
/// pig killed seemed not counted.
fn kills_words(tally: &[crate::memory::Tally]) -> String {
    let mut killed: Vec<&crate::memory::Tally> = tally.iter().filter(|t| t.kills > 0).collect();
    killed.sort_by_key(|t| std::cmp::Reverse(t.kills));
    let mut words: Vec<String> = killed
        .iter()
        .take(6)
        .map(|t| {
            format!(
                "{} {}",
                t.kills,
                crate::brain::text::plural(&t.kind, t.kills)
            )
        })
        .collect();
    if killed.len() > 6 {
        words.push(format!("{} more kinds", killed.len() - 6));
    }
    if words.is_empty() {
        words.push("none killed".to_string());
    }
    let hits: u32 = tally.iter().map(|t| t.hits).sum();

    format!("{}; {hits} hits", words.join(", "))
}

/// The stay in a few lines, for `/craftkit lodger status`.
fn status_lines(stay: &Stay, day: f32, config: &Config) -> String {
    let arc = &stay.arc;
    let mut lines = Vec::new();
    lines.push(if arc.over() {
        stay.status_line()
    } else if !arc.moved_in() {
        format!("not moved in yet (day {:.1} of the stay)", day)
    } else if arc.stage == 5 {
        format!(
            "stage 5 of 5, the end: {} (day {:.1} of the stay)",
            if arc.ending.is_empty() {
                "not chosen yet"
            } else {
                &arc.ending
            },
            day
        )
    } else {
        format!(
            "stage {} of 5, {:.1} days into it (day {:.1} of the stay)",
            arc.stage,
            (day - arc.stage_began_day).max(0.0),
            day
        )
    });
    lines.push(if stay.memory.target_name.is_empty() {
        "no one it lives with yet".to_string()
    } else {
        format!("it lives with {}", stay.memory.target_name)
    });
    let landed = stay.memory.landmarks.len();
    lines.push(format!(
        "{} thing{} done that they would notice{}",
        landed,
        if landed == 1 { "" } else { "s" },
        stay.memory
            .held
            .as_ref()
            .map(|h| format!(
                "; it holds their {}",
                h.item
                    .id
                    .rsplit(':')
                    .next()
                    .unwrap_or(&h.item.id)
                    .replace('_', " ")
            ))
            .unwrap_or_default()
    ));

    lines.push(if config.model.enabled {
        format!("the model decides ({})", config.model.size)
    } else {
        "the rules decide: it plays without the model".to_string()
    });

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kills_line_names_every_kind_most_first() {
        let t = |kind: &str, kills, hits| crate::memory::Tally {
            kind: kind.to_string(),
            kills,
            hits,
        };
        assert_eq!(
            kills_words(&[t("sheep", 11, 9), t("creeper", 1, 4), t("pig", 3, 5)]),
            "11 sheep, 3 pigs, 1 creeper; 18 hits"
        );
        assert_eq!(kills_words(&[t("zombie", 0, 2)]), "none killed; 2 hits");
    }
    use crate::memory::Home;
    use craftkit::world::WorldInfo;

    fn stay() -> Stay {
        let mut stay = Stay::new(&WorldInfo {
            id: "minecraft:overworld".to_string(),
            seed: 1,
            name: "w".to_string(),
        });
        stay.memory.target = Some(0);
        stay.memory.home = Some(Home {
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![],
        });
        stay.memory.last.online = true;
        stay.memory.last.indoors = false;
        stay.memory.last.pos = [1.0, 64.0, 1.0];
        stay
    }

    #[test]
    fn status_says_where_the_stay_is_and_leave_begins_the_leaves_ending() {
        let mut stay = stay();
        let config = Config::default();
        let mut log = Log::new(None, 16, true);
        let now = Now {
            day: 4,
            day_time: 6_000,
            ..Now::default()
        };
        let act = |stay: &mut Stay, log: &mut Log, id: &str| {
            answer(
                stay,
                &DebugRequest::Action { id: id.to_string() },
                &config,
                log,
                &now,
            )
            .message
        };

        assert!(act(&mut stay, &mut log, "status").starts_with("not moved in yet"));
        stay.arc.stage = 3;
        stay.memory.target_name = "razor".to_string();
        let status = act(&mut stay, &mut log, "status");
        assert!(status.contains("stage 3 of 5"), "{status}");
        assert!(status.contains("it lives with razor"), "{status}");

        assert!(act(&mut stay, &mut log, "leave").starts_with("it is leaving"));
        assert_eq!(stay.arc.stage, 5);
        assert_eq!(stay.arc.ending, "leave");
        assert_eq!(stay.arc.script_day, 5, "at the next dawn");
    }

    #[test]
    fn reset_forgets_the_stay_keeps_the_target_and_gives_back_what_it_holds() {
        let mut stay = stay();
        let config = Config::default();
        let mut log = Log::new(None, 16, true);
        let now = Now::default();
        stay.arc.stage = 4;
        stay.memory.set_target(2, "razor", "uuid-r");
        stay.memory.mods = vec![("sodium".to_string(), "1".to_string())];
        stay.memory.held = Some(crate::memory::Held {
            item: craftkit::world::ItemStack {
                id: "minecraft:clock".to_string(),
                count: 1,
                slot: 0,
            },
            from: [1, 64, 1],
            ..Default::default()
        });
        stay.memory.home = Some(crate::memory::Home {
            center: [0, 64, 0],
            floor_y: 64,
            ..Default::default()
        });

        let message = answer(
            &mut stay,
            &DebugRequest::Action {
                id: "reset".to_string(),
            },
            &config,
            &mut log,
            &now,
        )
        .message;

        assert!(message.starts_with("forgotten"), "{message}");
        assert_eq!(stay.arc.stage, 0, "begun again");
        assert!(stay.memory.home.is_none() && stay.memory.held.is_none());
        assert_eq!(stay.memory.target_uuid, "uuid-r");
        assert_eq!(stay.memory.mods.len(), 1);
    }

    #[test]
    fn the_panel_shows_temper_the_brain_the_chains_and_the_figure_looks() {
        let mut stay = stay();
        let config = Config::default();
        let mut log = Log::new(None, 16, true);
        stay.memory.chain_landed("light_path", 1.0);
        stay.memory.figure_looks.push(crate::memory::FigureLook {
            day: 1.2,
            looked_ticks: 50,
            looks: 1,
            end: "timed".to_string(),
            ..Default::default()
        });

        let reply = answer(
            &mut stay,
            &DebugRequest::Snapshot,
            &config,
            &mut log,
            &Now::default(),
        );
        assert!(reply.gauges.iter().any(|g| g.label == "temper"));
        assert!(reply
            .gauges
            .iter()
            .any(|g| g.label == "brain" && g.text.contains("model")));
        let chains = reply.sections.iter().find(|s| s.title == "Chains").unwrap();
        assert!(
            chains.lines[0].starts_with("the_place: [light_path] > hole"),
            "{:?}",
            chains.lines
        );
        let looks = reply
            .sections
            .iter()
            .find(|s| s.title == "Figure looks")
            .unwrap();
        assert!(
            looks.lines[0].contains("2.5s in 1 look"),
            "{:?}",
            looks.lines
        );
    }

    #[test]
    fn a_snapshot_lists_every_event_with_why_it_is_not_offered() {
        let mut stay = stay();
        let config = Config::default();
        let mut log = Log::new(None, 16, true);
        let now = Now::default();

        let reply = answer(&mut stay, &DebugRequest::Snapshot, &config, &mut log, &now);
        assert_eq!(reply.events.len(), CATALOGUE.len() - 1);
        assert!(reply.gauges[0].text.starts_with("not moved in"));
        let door = reply.events.iter().find(|e| e.id == "open_door").unwrap();
        assert_eq!(door.why, "stage 1 to 5", "stage 0 offers nothing");

        stay.arc.stage = 2;
        let reply = answer(&mut stay, &DebugRequest::Snapshot, &config, &mut log, &now);
        let door = reply.events.iter().find(|e| e.id == "open_door").unwrap();
        assert_eq!(door.why, "the target is home and awake");
        assert!(
            door.about.contains("\nNeeds: ") && door.about.contains("Never yet"),
            "the tooltip: {}",
            door.about
        );
        let music = reply.events.iter().find(|e| e.id == "cut_music").unwrap();
        assert!(music.offered, "{}", music.why);
        let plans = reply.sections.iter().find(|s| s.title == "Plans").unwrap();
        assert!(
            plans.items.is_empty() && plans.lines[0].starts_with("nothing planned"),
            "{plans:?}"
        );
        assert_eq!(reply.gauges[0].text, "stage 2 of 5");
        let story = reply.sections.iter().find(|s| s.title == "Story").unwrap();
        assert!(story.lines[0].starts_with("NEXT:"), "{:?}", story.lines);
    }

    #[test]
    fn the_story_keeps_each_decision_with_its_menu_and_outcome() {
        let mut stay = stay();
        let mut config = Config::default();
        config.stages.start_after_days = 0;
        config.brain.plan_every = 1;
        let mut log = Log::new(None, 16, true);
        stay.memory.last.online = false;
        let mut now = Now::default();
        for _ in 0..400 {
            stay.tick(&now, &config, &mut log);
            now.tick += 20;
            if stay.brain.story.len() >= 3 {
                break;
            }
        }
        assert!(
            stay.brain.story.len() >= 3,
            "{} decisions",
            stay.brain.story.len()
        );
        let first = &stay.brain.story[0];
        assert!(first.menu.contains(&"wait".to_string()));
        assert!(!first.situation.is_empty());
        let reply = answer(&mut stay, &DebugRequest::Snapshot, &config, &mut log, &now);
        let story = reply.sections.iter().find(|s| s.title == "Story").unwrap();
        assert!(
            story.lines.iter().any(|l| l.contains("chose")),
            "{:?}",
            story.lines
        );
    }

    #[test]
    fn forcing_an_event_queues_it_for_the_next_tick_past_every_rule() {
        let mut stay = stay();
        let config = Config::default();
        let mut log = Log::new(None, 16, true);
        let now = Now::default();
        stay.arc.stage = 1;

        let reply = answer(
            &mut stay,
            &DebugRequest::Force {
                event: "open_door".to_string(),
                player: 0,
            },
            &config,
            &mut log,
            &now,
        );
        assert!(
            reply.message.starts_with("open_door queued at (3,64,0)"),
            "{}",
            reply.message
        );
        let plan = stay.brain.plans.first().unwrap();
        assert!(plan.forced);

        // The target is home and awake, which the rules would refuse; forced, it still goes.
        let commands = stay.tick(
            &Now {
                tick: 1,
                ..Now::default()
            },
            &config,
            &mut log,
        );
        // The daily survey of the surroundings may come first; the forced door is among them.
        assert!(
            commands.iter().any(|c| matches!(
                c,
                craftkit::world::Command::ToggleBlock {
                    pos: [3, 64, 0],
                    ..
                }
            )),
            "{commands:?}"
        );
        // The forced plan ran and is gone (the brain may plan another of its own beside it).
        assert!(
            stay.brain.plans.iter().all(|p| !p.forced),
            "{:?}",
            stay.brain.plans
        );
        assert_eq!(stay.arc.spent_today, 0.0, "a forced event costs no budget");

        let nothing = answer(
            &mut stay,
            &DebugRequest::Force {
                event: "take".to_string(),
                player: 0,
            },
            &config,
            &mut log,
            &now,
        );
        assert_eq!(
            nothing.message,
            "take cannot happen: no favourite in a chest"
        );

        let unknown = answer(
            &mut stay,
            &DebugRequest::Force {
                event: "nope".to_string(),
                player: 0,
            },
            &config,
            &mut log,
            &now,
        );
        assert_eq!(unknown.message, "no event called nope");
    }

    #[test]
    fn the_stage_and_the_actions_change_the_stay() {
        let mut stay = stay();
        let config = Config::default();
        let mut log = Log::new(None, 16, true);
        let now = Now::default();

        let reply = answer(
            &mut stay,
            &DebugRequest::SetStage { stage: 4 },
            &config,
            &mut log,
            &now,
        );
        assert_eq!(reply.message, "stage 4");
        assert_eq!(stay.arc.stage, 4);

        answer(
            &mut stay,
            &DebugRequest::Action {
                id: "next_stage".to_string(),
            },
            &config,
            &mut log,
            &now,
        );
        assert_eq!(stay.arc.stage, 5);

        stay.memory.record("knock", None, 0.0, true, "");
        answer(
            &mut stay,
            &DebugRequest::Action {
                id: "forget_cooldowns".to_string(),
            },
            &config,
            &mut log,
            &now,
        );
        assert!(stay.memory.done.is_empty());

        let reply = answer(
            &mut stay,
            &DebugRequest::Action {
                id: "forget_all".to_string(),
            },
            &config,
            &mut log,
            &now,
        );
        assert!(stay.memory.home.is_none());
        assert_eq!(stay.memory.target, Some(0), "the target is kept");
        assert!(reply.message.contains("empty"));

        let reply = answer(
            &mut stay,
            &DebugRequest::Action {
                id: "nope".to_string(),
            },
            &config,
            &mut log,
            &now,
        );
        assert_eq!(reply.message, "no action called nope");
    }

    #[test]
    fn before_it_moves_in_the_next_decision_says_so() {
        let stay = Stay::new(&WorldInfo {
            id: "w".to_string(),
            ..Default::default()
        });
        let config = Config::default();
        let said = next_decision(&stay, &config, &Now::default(), 0.5);
        assert!(said.contains("until it moves in"), "{said}");
    }
}
