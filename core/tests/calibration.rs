//! Calibration: a scripted stay against the real model, with every decision printed, so a
//! reader can judge the prompts, the menu words and the thresholds (`docs/calibration/`).
//! Ignored by default: it needs `core/models/qwen-1.5b.gguf` (or `qwen.gguf`) and takes a
//! minute. Run with `cargo test -p lodger-core --test calibration -- --ignored --nocapture`.

use craftkit::world::{
    Event, EventKind, ItemStack, Movement, Now, Observations, Player, Report, WorldInfo,
};
use lodger_core::brain::log::Log;
use lodger_core::config::Config;
use lodger_core::model::{Model, Settings};
use lodger_core::stay::Stay;
use std::path::PathBuf;

fn local_model() -> Option<PathBuf> {
    ["qwen-1.5b.gguf", "qwen.gguf"]
        .iter()
        .map(|name| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("models")
                .join(name)
        })
        .find(|path| path.is_file())
}

/// A player with a house, a chest, a bed, a door, torches, two kills and a daily walk, as the
/// scripted stays have, but observed through `Stay` directly so the model can be handed in.
fn observations(tick: u64) -> Observations {
    let time = tick % 24_000;
    let (pos, sleeping, still, pace) = if (13_000..23_000).contains(&time) {
        ([0.5, 64.0, 0.5], true, false, 0.0)
    } else if (1_800..2_000).contains(&time) {
        (
            [3.5 + (time - 1_800) as f64 / 200.0 * 77.0, 64.0, 0.5],
            false,
            false,
            7.7,
        )
    } else if (10_800..11_000).contains(&time) {
        (
            [80.5 - (time - 10_800) as f64 / 200.0 * 77.0, 64.0, 0.5],
            false,
            false,
            7.7,
        )
    } else if (2_000..10_800).contains(&time) {
        ([80.5, 40.0, 0.5], false, true, 0.0)
    } else {
        ([3.5, 64.0, 2.5], false, time >= 11_000, 1.0)
    };
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
                ItemStack {
                    id: "minecraft:oak_sapling".to_string(),
                    count: 3,
                    slot: 2,
                },
            ],
        });
    }
    if tick == 100 {
        events.push(Event {
            player: 0,
            kind: EventKind::PlacedBlock,
            pos: Some([4, 64, 0]),
            detail: "minecraft:oak_door".to_string(),
            ..Event::default()
        });
    }
    if tick == 160 {
        events.push(Event {
            player: 0,
            kind: EventKind::PlacedBlock,
            pos: Some([2, 64, 0]),
            detail: "minecraft:chest".to_string(),
            ..Event::default()
        });
    }
    if tick == 120 || tick == 140 {
        events.push(Event {
            player: 0,
            kind: EventKind::PlacedBlock,
            pos: Some([if tick == 120 { 1 } else { -1 }, 65, 1]),
            detail: "minecraft:torch".to_string(),
            ..Event::default()
        });
    }
    if tick == 3_000 || tick == 3_200 || tick == 3_400 {
        events.push(Event {
            player: 0,
            kind: EventKind::Killed,
            pos: Some([82, 40, 2]),
            detail: "minecraft:cow".to_string(),
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
                still_seconds: if still { 61.0 } else { 0.0 },
                ..Movement::default()
            },
            ..Player::default()
        }],
        events,
        ..Observations::default()
    }
}

#[test]
#[ignore = "needs the real model under core/models and a minute; prints every decision"]
fn three_days_against_the_real_model_print_every_decision() {
    let Some(path) = local_model() else {
        eprintln!("no model under core/models; nothing to calibrate against");
        return;
    };
    let mut config = Config::parse(
        r#"{ "stages": { "start_after_days": 0, "least_days": [0, 0, 0, 0, 0], "most_days": [1, 1, 1, 1, 1], "events_per_day": [8, 8, 8, 8, 8] }, "brain": { "plan_every": 30 }, "events": { "cooldown_scale": 0.2 } }"#,
    )
    .unwrap();
    config.model.path = path.to_string_lossy().into_owned();
    let model = Model::new(Settings {
        path,
        threads: 4,
        context: 1024,
        temperature: 0.7,
        seed: 7,
        gpu_layers: 0,
    });
    model.warm().expect("the model loads");
    let mut log = Log::new(None, 16, true);
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed: 7,
        name: "calibration".to_string(),
    };
    let mut stay = Stay::new(&info);
    let mut tick = 0u64;
    let mut printed = 0;
    while tick < 3 * 24_000 {
        let now = Now {
            tick,
            day: tick / 24_000,
            day_time: (tick % 24_000) as u32,
            real_millis: tick * 50,
        };
        stay.observe(&observations(tick), stay.day(&now));
        let commands = stay.tick_with(&now, &config, &mut log, Some(&model));
        if !commands.is_empty() {
            let reports: Vec<Report> = commands
                .iter()
                .map(|c| Report {
                    id: c.id(),
                    done: true,
                    reason: String::new(),
                })
                .collect();
            for command in &commands {
                println!("tick {tick}: {command:?}");
            }
            stay.report(&reports, &mut log);
        }
        while printed < log.lines.len() {
            println!("{}", log.lines[printed]);
            printed += 1;
        }
        tick += 5;
        // The model answers on its own thread; a short wait keeps the loop from racing it.
        if stay.brain.asking.is_some() || !stay.brain.writings.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    println!(
        "{} decisions, {} landed; want: {}; mood: {}; written: {:?}",
        stay.brain.decisions,
        stay.memory
            .done
            .iter()
            .filter(|d| d.ok && d.event != "wait")
            .count(),
        stay.memory.want,
        stay.memory.last.mood,
        stay.brain.written
    );
    model.shutdown();
}
