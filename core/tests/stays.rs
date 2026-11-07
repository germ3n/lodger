//! Scripted stays: a `Player` script drives a `Stay` tick by tick for days against the rule
//! brain, through the SDK's harness, the same path the host takes, and the tests check what the
//! lodger did and never did. These are the tests that prove the rules.

use craftkit::world::{
    Command, Event, EventKind, ItemStack, Movement, Now, Observations, Player, Report, WorldInfo,
};
// The exported functions the harness drives live at the mind's crate root.
use lodger_core::*;
use std::sync::Mutex;

static TURN: Mutex<()> = Mutex::new(());

/// A player with a house at the origin: a bed, a chest they open each morning, the chest, a door
/// and two torches put up the first day, and a day of mining 80 blocks east. Asleep from 13000 to 23000.
struct Script {
    tick: u64,
}

impl Script {
    fn new() -> Self {
        Self { tick: 0 }
    }

    fn now(&self) -> Now {
        Now {
            tick: self.tick,
            day: self.tick / 24_000,
            day_time: (self.tick % 24_000) as u32,
            real_millis: self.tick * 50,
        }
    }

    /// Where the player is: position, asleep, standing still, pace.
    fn position(&self) -> ([f64; 3], bool, bool, f32) {
        let time = self.tick % 24_000;
        if (13_000..23_000).contains(&time) {
            return ([0.5, 64.0, 0.5], true, false, 0.0);
        }
        // The walk to the mine and back, 77 blocks over 200 ticks, so the memory sees a route.
        if (1_800..2_000).contains(&time) {
            let x = 3.5 + (time - 1_800) as f64 / 200.0 * 77.0;
            return ([x, 64.0, 0.5], false, false, 7.7);
        }
        if (10_800..11_000).contains(&time) {
            let x = 80.5 - (time - 10_800) as f64 / 200.0 * 77.0;
            return ([x, 64.0, 0.5], false, false, 7.7);
        }
        if (2_000..10_800).contains(&time) {
            return ([80.5, 40.0, 0.5], false, true, 0.0);
        }
        if (11_000..13_000).contains(&time) {
            return ([3.5, 64.0, 2.5], false, true, 0.0);
        }

        ([3.5, 64.0, 2.5], false, false, 2.0)
    }

    /// Whether the player is home and awake: when nothing of the house may be touched.
    fn home_awake(tick: u64) -> bool {
        let time = tick % 24_000;
        (11_000..13_000).contains(&time) || !(1_800..23_000).contains(&time)
    }

    fn observations(&self) -> Observations {
        let (pos, sleeping, still, pace) = self.position();
        let time = self.tick % 24_000;
        let mut events = Vec::new();

        if time == 13_000 {
            events.push(Event {
                player: 0,
                kind: EventKind::SleptBegan,
                pos: Some([0, 64, 0]),
                ..Event::default()
            });
        }

        if time == 500 {
            events.push(Event {
                player: 0,
                kind: EventKind::OpenedContainer,
                pos: Some([2, 64, 0]),
                detail: "minecraft:chest".to_string(),
                contents: vec![
                    ItemStack {
                        id: "minecraft:iron_pickaxe".to_string(),
                        count: 1,
                        slot: 0,
                    },
                    ItemStack {
                        id: "minecraft:cobblestone".to_string(),
                        count: 30,
                        slot: 1,
                    },
                ],
            });
        }

        if self.tick == 100 {
            events.push(Event {
                player: 0,
                kind: EventKind::PlacedBlock,
                pos: Some([4, 64, 0]),
                detail: "minecraft:oak_door".to_string(),
                ..Event::default()
            });
        }
        if self.tick == 160 {
            events.push(Event {
                player: 0,
                kind: EventKind::PlacedBlock,
                pos: Some([2, 64, 0]),
                detail: "minecraft:chest".to_string(),
                ..Event::default()
            });
        }
        if self.tick == 120 || self.tick == 140 {
            events.push(Event {
                player: 0,
                kind: EventKind::PlacedBlock,
                pos: Some([if self.tick == 120 { 1 } else { -1 }, 65, 1]),
                detail: "minecraft:torch".to_string(),
                ..Event::default()
            });
        }

        Observations {
            players: vec![Player {
                index: 0,
                name: "razor".to_string(),
                pos,
                yaw: 90.0,
                sleeping,
                sky_light: if pos[0] > 50.0 { 15 } else { 2 },
                main_hand: ItemStack {
                    id: "minecraft:iron_pickaxe".to_string(),
                    count: 1,
                    slot: 0,
                },
                movement: Movement {
                    pace,
                    // A minute's stillness cuts the trail into a route at the mine and at home.
                    still_seconds: if still { 61.0 } else { 0.0 },
                    ..Movement::default()
                },
                ..Player::default()
            }],
            events,
            ..Observations::default()
        }
    }
}

struct Run {
    commands: Vec<(u64, Command)>,
    /// Events that landed, as the stay's own debug page counts them at the end.
    landed: u64,
}

/// The command's kind, and for a hint its kind too, since two different hints are two different
/// events while two `ToggleBlock`s are the same door.
fn kind(command: &Command) -> String {
    if let Command::Hint { hint, .. } = command {
        return format!("Hint:{hint}");
    }

    let text = format!("{command:?}");
    text.split([' ', '(']).next().unwrap().to_string()
}

/// Runs a stay for `days` in-game days, answering every command as done, through the ABI. The
/// clock moves five ticks at a time, still thousands of ticks a day.
fn run(days: u64, config: &str) -> Run {
    let _turn = TURN.lock().unwrap_or_else(|err| err.into_inner());
    let harness = craftkit::harness!().unwrap();
    harness.init(config);
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed: 11,
        name: "scripted".to_string(),
    };
    let world = harness.open_world(&info, &[]).unwrap();
    let mut script = Script::new();
    let mut run = Run {
        commands: Vec::new(),
        landed: 0,
    };

    while script.tick < days * 24_000 {
        world.observe(&script.observations());
        let commands = world.tick(&script.now());
        let reports: Vec<Report> = commands
            .iter()
            .map(|command| Report {
                id: command.id(),
                done: true,
                reason: String::new(),
            })
            .collect();
        for command in commands {
            run.commands.push((script.tick, command));
        }
        if !reports.is_empty() {
            world.report(&reports);
        }
        script.tick += 5;
    }

    // "N decisions, M events landed, K did not": the stay's own count.
    let page = world.debug(&craftkit::debug::DebugRequest::Snapshot);
    run.landed = page
        .gauges
        .iter()
        .find(|g| g.label == "decisions")
        .and_then(|g| g.text.split(", ").nth(1)?.split(' ').next()?.parse().ok())
        .unwrap_or(0);
    world.close();
    run
}

#[test]
fn a_stay_of_twelve_days_on_the_rule_brain_acts_within_its_budget_and_never_in_view() {
    let run = run(12, r#"{ "stages": { "start_after_days": 1 } }"#);

    assert!(!run.commands.is_empty(), "nothing happened in twelve days");
    // The budget is 1, 2, 4, 6 events a day: eleven days after moving in land 66 at most.
    // Events, not commands: since the sound events got their variants a footstep
    // is up to five commands, and the command count stopped measuring the budget. The day's
    // own bound is held by the brain's `plans_held_count_against_the_day_s_budget_before_they_run`.
    assert!(run.landed > 0, "the page counted nothing");
    assert!(run.landed <= 66, "{} events in eleven days", run.landed);
    assert!(
        run.commands.len() < 400,
        "a runaway: {} commands in twelve days",
        run.commands.len()
    );

    for (tick, command) in &run.commands {
        if matches!(
            command,
            Command::ToggleBlock { .. }
                | Command::MoveBlock { .. }
                | Command::SetBlock { .. }
                | Command::MoveItem { .. }
                | Command::TakeItem { .. }
        ) {
            assert!(
                !Script::home_awake(*tick),
                "the house was touched at {} while the player was home and awake: {command:?}",
                tick % 24_000
            );
        }
    }
}

#[test]
fn before_it_moves_in_nothing_happens_and_the_first_stage_offers_only_small_things() {
    let quiet = run(2, r#"{ "stages": { "start_after_days": 3 } }"#);
    assert!(quiet.commands.is_empty(), "{:?}", quiet.commands);

    let run = run(
        3,
        r#"{ "stages": { "start_after_days": 0, "least_days": [9, 9, 9, 9, 9], "most_days": [9, 9, 9, 9, 9] } }"#,
    );
    let kinds: Vec<String> = run.commands.iter().map(|(_, c)| kind(c)).collect();
    assert!(
        !kinds.contains(&"TakeItem".to_string()),
        "a take in stage one: {kinds:?}"
    );
    assert!(
        !kinds.contains(&"CancelSleep".to_string()),
        "sleep refused in stage one: {kinds:?}"
    );
}

#[test]
fn a_saved_stay_reopens_where_it_was() {
    let _turn = TURN.lock().unwrap_or_else(|err| err.into_inner());
    let harness = craftkit::harness!().unwrap();
    harness.init(r#"{ "stages": { "start_after_days": 0 } }"#);
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed: 3,
        name: "scripted".to_string(),
    };
    let world = harness.open_world(&info, &[]).unwrap();
    let mut script = Script::new();
    while script.tick < 30_000 {
        world.observe(&script.observations());
        world.tick(&script.now());
        script.tick += 5;
    }
    let saved = world.save();
    world.close();

    let again = harness.open_world(&info, &saved).unwrap();
    let stay = lodger_core::stay::Stay::open(&info, &again.save()).unwrap();
    assert!(stay.arc.moved_in(), "{}", stay.status_line());
    assert!(stay.memory.home.is_some());
    assert_eq!(stay.memory.target_name, "razor");
    assert!(stay.ticks > 0);
    again.close();
}

#[test]
fn the_same_event_never_lands_twice_in_a_row() {
    let _turn = TURN.lock().unwrap_or_else(|err| err.into_inner());
    let harness = craftkit::harness!().unwrap();
    harness.init(r#"{ "stages": { "start_after_days": 0 } }"#);
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed: 5,
        name: "scripted".to_string(),
    };
    let world = harness.open_world(&info, &[]).unwrap();
    let mut script = Script::new();
    while script.tick < 6 * 24_000 {
        world.observe(&script.observations());
        let commands = world.tick(&script.now());
        let reports: Vec<Report> = commands
            .iter()
            .map(|command| Report {
                id: command.id(),
                done: true,
                reason: String::new(),
            })
            .collect();
        if !reports.is_empty() {
            world.report(&reports);
        }
        script.tick += 5;
    }

    // The rule is about events, not commands: two different events may both move items.
    let stay = lodger_core::stay::Stay::open(&info, &world.save()).unwrap();
    let landed: Vec<&str> = stay
        .memory
        .done
        .iter()
        .filter(|d| d.ok && d.event != "wait")
        .map(|d| d.event.as_str())
        .collect();
    assert!(landed.len() >= 3, "too few events to judge: {landed:?}");
    for pair in landed.windows(2) {
        assert_ne!(pair[0], pair[1], "{} twice in a row in {landed:?}", pair[0]);
    }
    world.close();
}

/// The stay behind a run, read back through its save, for the tests that look at the memory.
fn stay_after(days: u64, config: &str, seed: i64) -> lodger_core::stay::Stay {
    let _turn = TURN.lock().unwrap_or_else(|err| err.into_inner());
    let harness = craftkit::harness!().unwrap();
    harness.init(config);
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed,
        name: "scripted".to_string(),
    };
    let world = harness.open_world(&info, &[]).unwrap();
    let mut script = Script::new();
    while script.tick < days * 24_000 {
        world.observe(&script.observations());
        let commands = world.tick(&script.now());
        let reports: Vec<Report> = commands
            .iter()
            .map(|command| Report {
                id: command.id(),
                done: true,
                reason: String::new(),
            })
            .collect();
        if !reports.is_empty() {
            world.report(&reports);
        }
        script.tick += 5;
    }
    let stay = lodger_core::stay::Stay::open(&info, &world.save()).unwrap();
    world.close();

    stay
}

#[test]
fn no_event_lands_inside_its_cooldown_or_past_its_cap() {
    let stay = stay_after(
        14,
        r#"{ "stages": { "start_after_days": 0, "least_days": [0, 0, 0, 0, 0], "most_days": [3, 3, 3, 3, 3], "events_per_day": [8, 8, 8, 8, 8] } }"#,
        21,
    );
    let landed: Vec<_> = stay
        .memory
        .done
        .iter()
        .filter(|d| d.ok && d.event != "wait")
        .collect();
    assert!(
        landed.len() >= 4,
        "{} events landed in fourteen days",
        landed.len()
    );

    for (i, done) in landed.iter().enumerate() {
        let event = lodger_core::events::find(&done.event).expect(&done.event);
        let times = landed.iter().filter(|d| d.event == done.event).count() as u32;
        assert!(
            times <= event.most_per_stay,
            "{} landed {} times, its cap is {}",
            done.event,
            times,
            event.most_per_stay
        );
        for earlier in &landed[..i] {
            if earlier.event == done.event {
                assert!(
                    done.day - earlier.day >= event.cooldown_days - 1e-3,
                    "{} landed on day {:.2} and again on day {:.2}, inside its {} day cooldown",
                    done.event,
                    earlier.day,
                    done.day,
                    event.cooldown_days
                );
            }
        }
    }
}

#[test]
fn a_disabled_event_never_happens_and_a_group_switched_off_stays_quiet() {
    let run = run(
        8,
        r#"{ "stages": { "start_after_days": 0, "least_days": [0, 0, 0, 0, 0], "most_days": [2, 2, 2, 2, 2] }, "events": { "disabled": ["open_door", "close_door", "wake", "door_opens_itself"], "sounds": false, "glitches": false, "chat": false } }"#,
    );
    for (_, command) in &run.commands {
        match command {
            Command::ToggleBlock { pos, .. } => {
                assert_ne!(*pos, [4, 64, 0], "the door toggled while disabled")
            }
            Command::PlaySound { .. } => panic!("a sound with events.sounds off: {command:?}"),
            Command::Hint { .. } => panic!("a glitch with events.glitches off: {command:?}"),
            // The occupied-bed line is vanilla's action bar from the rest group, not chat.
            Command::GameMessage { message, .. } if message != "bed_occupied" => {
                panic!("chat with events.chat off: {command:?}")
            }
            Command::PlayerList { .. } => {
                panic!("chat with events.chat off: {command:?}")
            }
            _ => {}
        }
    }
}

#[test]
fn sleep_is_only_ever_refused_while_the_player_is_in_bed_and_sounds_play_only_while_they_are_there()
{
    let run = run(
        10,
        r#"{ "stages": { "start_after_days": 0, "least_days": [0, 0, 0, 0, 0], "most_days": [2, 2, 2, 2, 2] } }"#,
    );
    for (tick, command) in &run.commands {
        let time = tick % 24_000;
        match command {
            Command::CancelSleep { seconds, .. } if *seconds > 0.0 => {
                assert!(
                    (13_000..23_000).contains(&time),
                    "sleep refused at {time} with nobody in bed"
                );
            }
            // A sound at the house (within 30 blocks of it) while the player is at the mine
            // would be for nobody; a far sound or a glitch follows the player wherever they are.
            Command::PlaySound { pos, .. } if pos[0].abs() < 30.0 && pos[2].abs() < 30.0 => {
                assert!(
                    !(2_000..10_800).contains(&time),
                    "a sound at the house for a player 80 blocks away at {time}: {command:?}"
                );
            }
            _ => {}
        }
    }
}

#[test]
fn a_calm_preset_acts_less_than_a_relentless_one() {
    let calm = run(
        6,
        r#"{ "preset": "calm", "stages": { "start_after_days": 0, "least_days": [0, 0, 0, 0, 0], "most_days": [2, 2, 2, 2, 2] } }"#,
    );
    let relentless = run(
        6,
        r#"{ "preset": "relentless", "stages": { "start_after_days": 0, "least_days": [0, 0, 0, 0, 0], "most_days": [2, 2, 2, 2, 2] } }"#,
    );

    assert!(
        calm.commands.len() < relentless.commands.len(),
        "calm did {} things, relentless {}",
        calm.commands.len(),
        relentless.commands.len()
    );
}

#[test]
fn the_stay_is_over_after_stage_five_s_ending_and_nothing_happens_after() {
    // Stages one to four a day each; stage five is the ending's three days from its first
    // dawn (it was a day of nothing), so the stay is over by day eight.
    let config = r#"{ "stages": { "start_after_days": 0, "least_days": [0, 0, 0, 0, 0], "most_days": [1, 1, 1, 1, 1], "ending": "leave" } }"#;
    let stay = stay_after(10, config, 9);
    assert!(stay.arc.over(), "{}", stay.status_line());
    assert_eq!(stay.status_line(), "the stay is over: it left");

    let run = run(10, config);
    assert!(
        run.commands.iter().any(|(_, c)| matches!(c, Command::PlaySound { sound, .. } if sound == "minecraft:music.game")),
        "the ending played"
    );
    let late: Vec<_> = run
        .commands
        .iter()
        .filter(|(tick, command)| {
            *tick >= 8 * 24_000 && !matches!(command, Command::Surroundings { .. })
        })
        .collect();
    assert!(late.is_empty(), "after the stay ended: {late:?}");
}

#[test]
fn the_memory_learns_the_house_the_chest_the_favourite_and_the_route_from_the_script() {
    // The days are the stay's own as it observes (until now every observation was day
    // zero, and nothing that counts days ever moved).
    {
        let stay = stay_after(4, r#"{ "stages": { "start_after_days": 0 } }"#, 2);
        assert!(stay.memory.today > 3.0, "today is {}", stay.memory.today);
        assert!(stay.memory.habits.samples > 0, "the habits are sampled");
    }

    let stay = stay_after(4, r#"{ "stages": { "start_after_days": 0 } }"#, 2);
    let memory = &stay.memory;

    let home = memory.home.as_ref().expect("a home");
    assert_eq!(home.bed, Some([0, 64, 0]));
    assert_eq!(home.doors, vec![[4, 64, 0]]);
    assert_eq!(home.lights.len(), 2);
    assert_eq!(memory.used_container().unwrap().pos, [2, 64, 0]);
    assert_eq!(memory.favourites[0].0, "minecraft:iron_pickaxe");
    assert!(
        memory.route().is_some(),
        "the daily walk to the mine is a route: {:?}",
        memory.routes.len()
    );
    assert!(
        memory.sessions.is_empty(),
        "the script never joins or leaves"
    );
    assert!(stay.brain.story.len() >= 3);
    let saved = stay.save();
    assert!(saved.len() < 256 * 1024, "{} bytes saved", saved.len());
}

/// What a tick costs (the optimization run): twelve scripted days through the
/// harness, the host's own path (JSON in and out), every game tick observed and ticked as the
/// host does it, timed. Run in release: `cargo test --release -p lodger-core --test stays --
/// --ignored the_tick_costs --nocapture`.
#[test]
#[ignore]
fn the_tick_costs() {
    let _turn = TURN.lock().unwrap_or_else(|err| err.into_inner());
    let harness = craftkit::harness!().unwrap();
    harness.init(r#"{ "stages": { "start_after_days": 1, "most_days": [2, 3, 3, 3, 3] } }"#);
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed: 11,
        name: "timed".to_string(),
    };
    let world = harness.open_world(&info, &[]).unwrap();
    let mut script = Script::new();
    let mut costs: Vec<(u128, u64)> = Vec::new();
    while script.tick < 12 * 24_000 {
        let observations = script.observations();
        let started = std::time::Instant::now();
        world.observe(&observations);
        let commands = world.tick(&script.now());
        let reports: Vec<Report> = commands
            .iter()
            .map(|command| Report {
                id: command.id(),
                done: true,
                reason: String::new(),
            })
            .collect();
        if !reports.is_empty() {
            world.report(&reports);
        }
        costs.push((started.elapsed().as_micros(), script.tick));
        script.tick += 1;
    }
    let saved = world.save();
    world.close();
    let mut sorted: Vec<u128> = costs.iter().map(|(c, _)| *c).collect();
    sorted.sort_unstable();
    let total: u128 = sorted.iter().sum();
    let at = |q: f64| sorted[((sorted.len() - 1) as f64 * q) as usize];
    let mut worst = costs.clone();
    worst.sort_by_key(|w| std::cmp::Reverse(w.0));
    println!(
        "{} ticks: mean {} us, p50 {} us, p99 {} us, p99.9 {} us, max {} us; save {} bytes",
        sorted.len(),
        total / sorted.len() as u128,
        at(0.5),
        at(0.99),
        at(0.999),
        sorted.last().unwrap(),
        saved.len()
    );
    for (cost, tick) in worst.iter().take(8) {
        println!(
            "  {cost} us at tick {tick} (day {}, {})",
            tick / 24_000,
            tick % 24_000
        );
    }
    // A game tick is 50 ms; the mind should take a small part of it, every tick.
    assert!(at(0.99) < 2_000, "p99 {} us", at(0.99));
}

/// The first day the stay reaches stage four, with the target chasing the figure three times a
/// day or never; or the last day run.
fn day_of_stage_four(provoke: bool, config: &str) -> (u64, f32) {
    let _turn = TURN.lock().unwrap_or_else(|err| err.into_inner());
    let harness = craftkit::harness!().unwrap();
    harness.init(config);
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed: 5,
        name: "scripted".to_string(),
    };
    let world = harness.open_world(&info, &[]).unwrap();
    let mut script = Script::new();
    let days = 20;
    let mut reached = days;
    let mut temper: f32 = 0.0;
    while script.tick < days * 24_000 {
        let mut observations = script.observations();
        if provoke && script.tick.is_multiple_of(8_000) {
            observations.events.push(craftkit::world::Event {
                player: 0,
                kind: craftkit::world::EventKind::FigureSeen,
                pos: Some([30, 64, 0]),
                detail: "looked=60 looks=1 longest=60 toward=14.0 end=approached".to_string(),
                ..Default::default()
            });
        }
        world.observe(&observations);
        let commands = world.tick(&script.now());
        let reports: Vec<Report> = commands
            .iter()
            .map(|command| Report {
                id: command.id(),
                done: true,
                reason: String::new(),
            })
            .collect();
        if !reports.is_empty() {
            world.report(&reports);
        }
        if script.tick.is_multiple_of(24_000) {
            let stay = lodger_core::stay::Stay::open(&info, &world.save()).unwrap();
            temper = temper.max(stay.memory.temper.value);
            if stay.arc.stage >= 4 {
                reached = script.tick / 24_000;
                break;
            }
        }
        script.tick += 5;
    }
    world.close();

    (reached, temper)
}

#[test]
fn a_provoking_player_brings_stage_four_days_sooner_than_a_quiet_one() {
    let config = r#"{ "stages": { "start_after_days": 1 } }"#;
    let (quiet, _) = day_of_stage_four(false, config);
    let (provoked, temper) = day_of_stage_four(true, config);

    assert!(temper > 0.9, "chased every day, it is furious: {temper}");
    assert!(
        provoked + 2 <= quiet,
        "provoked on day {provoked}, quiet on day {quiet}"
    );
}
