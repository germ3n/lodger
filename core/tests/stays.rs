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

/// A player with a house at the origin: a bed, a chest they open each morning, a door and two
/// torches put up the first day, and a day of mining 80 blocks east. Asleep from 13000 to 23000.
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
        if (2_000..11_000).contains(&time) {
            return ([80.5, 40.0, 0.5], false, false, 1.0);
        }
        if (11_000..13_000).contains(&time) {
            return ([3.5, 64.0, 2.5], false, true, 0.0);
        }

        ([3.5, 64.0, 2.5], false, false, 2.0)
    }

    /// Whether the player is home and awake: when nothing of the house may be touched.
    fn home_awake(tick: u64) -> bool {
        let time = tick % 24_000;
        (11_000..13_000).contains(&time) || !(2_000..23_000).contains(&time)
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
                    still_seconds: if still { 5.0 } else { 0.0 },
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

    world.close();
    run
}

#[test]
fn a_stay_of_twelve_days_on_the_rule_brain_acts_within_its_budget_and_never_in_view() {
    let run = run(12, r#"{ "stages": { "start_after_days": 1 } }"#);

    assert!(!run.commands.is_empty(), "nothing happened in twelve days");
    // The budget is 1, 2, 4, 6 a day and the rule brain waits often: twelve days through the
    // first stages hold a few dozen commands at most.
    assert!(
        run.commands.len() < 120,
        "{} commands in twelve days",
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
