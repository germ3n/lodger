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
use craftkit::debug::{ActionRow, DebugReply, DebugRequest, EventRow, Gauge, Section};
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
            if stay.memory.target.is_none() {
                stay.memory.target = Some(*player);
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
            stay.arc.stage = *stage;
            stay.arc.stage_began_day = day;
            stay.arc.landed_this_stage = 0;
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
                    stay.arc.stage = (stay.arc.stage + 1).min(6);
                    stay.arc.stage_began_day = day;
                    stay.arc.landed_this_stage = 0;
                    format!("stage {}", stay.arc.stage)
                }
                "plan_now" => {
                    stay.brain.next_plan_tick = 0;
                    stay.brain.plan = None;
                    "the brain decides on the next tick".to_string()
                }
                "clear_plan" => {
                    let had = stay.brain.plan.take().map(|p| p.event);
                    stay.brain.later.clear();
                    match had {
                        Some(event) => format!("dropped the plan for {event}"),
                        None => "there was no plan".to_string(),
                    }
                }
                "forget_cooldowns" => {
                    stay.memory.done.clear();
                    stay.arc.spent_today = 0.0;
                    "every event may happen again, and the day's budget is whole".to_string()
                }
                "forget_all" => {
                    let target = stay.memory.target;
                    let name = stay.memory.target_name.clone();
                    stay.memory = Default::default();
                    stay.memory.target = target;
                    stay.memory.target_name = name;
                    stay.brain.plan = None;
                    stay.brain.later.clear();
                    stay.brain.pending.clear();
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
    match &brain.plan {
        Some(plan) => {
            let wait = plan.from_tick.saturating_sub(now.tick) as f32 / 20.0;
            gauges.push(Gauge::new(
                "plan",
                if plan.forced {
                    1.0
                } else {
                    1.0 - (wait / (4.0 * config.brain.plan_every as f32)).min(1.0)
                },
                1.0,
                if plan.forced {
                    format!("{} forced, next tick", plan.event)
                } else if wait > 0.0 {
                    format!("{} in {wait:.0}s, then when its moment comes", plan.event)
                } else {
                    format!("{} waiting for its moment", plan.event)
                },
            ));
        }
        None => {
            let wait = brain.next_plan_tick.saturating_sub(now.tick) as f32 / 20.0;
            gauges.push(Gauge::new(
                "plan",
                1.0 - (wait / config.brain.plan_every as f32).min(1.0),
                1.0,
                format!("none; next decision in {wait:.0}s"),
            ));
        }
    }
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
    if let Some(tally) = memory.tally.first() {
        gauges.push(Gauge::new(
            "kills",
            tally.kills as f32,
            (tally.kills.max(10)) as f32,
            format!(
                "{} {}s killed, {} hit; {} kinds",
                tally.kills,
                tally.kind,
                tally.hits,
                memory.tally.len()
            ),
        ));
    }

    let mut sections = vec![Section::new("As the brain reads it", memory.describe(day))];
    let mut plan_lines = Vec::new();
    if let Some(plan) = &brain.plan {
        plan_lines.push(format!(
            "{}{} at {}",
            plan.event,
            if plan.forced { " (forced)" } else { "" },
            plan.pos
                .map(|p| format!("({},{},{})", p[0], p[1], p[2]))
                .unwrap_or_else(|| "no place".to_string())
        ));
    }
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
    if plan_lines.is_empty() {
        plan_lines.push("nothing planned".to_string());
    }
    sections.push(Section::new("Plan", plan_lines));
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
    // chose and what came of it (owner, 2026-11-09: "show events that happened, what is about
    // to happen, why it happened").
    let mut story = Vec::new();
    if let Some(plan) = &brain.plan {
        story.push(format!(
            "NEXT: {}{} {}",
            plan.event,
            plan.pos
                .map(|p| format!(" at ({},{},{})", p[0], p[1], p[2]))
                .unwrap_or_default(),
            if plan.forced {
                "on the next tick (forced)".to_string()
            } else if plan.from_tick > now.tick {
                format!(
                    "in about {}s, once its moment comes",
                    (plan.from_tick - now.tick) / 20
                )
            } else {
                "as soon as its moment comes".to_string()
            }
        ));
    } else {
        story.push(format!(
            "NEXT: a decision in about {}s",
            brain.next_plan_tick.saturating_sub(now.tick) / 20
        ));
    }
    for decision in brain.story.iter().rev().take(12) {
        story.push(format!(
            "day {:.2}, stage {}: chose {}{} from [{}] -> {}",
            decision.day,
            decision.stage,
            decision.chosen,
            decision
                .pos
                .map(|p| format!(" at ({},{},{})", p[0], p[1], p[2]))
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
            let why = if !event.offered_in(arc.stage) {
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
            ActionRow::new("forget_all", "Forget everything"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let music = reply.events.iter().find(|e| e.id == "cut_music").unwrap();
        assert!(music.offered, "{}", music.why);
        assert!(reply.sections.iter().any(|s| s.title == "Plan"));
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
        let plan = stay.brain.plan.as_ref().unwrap();
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
        assert!(stay.brain.plan.is_none());
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
}
