//! The frame and the scoring compared: the same late-stage situations and menus scored by the
//! real model under the old frame and the new one, each way of scoring a choice
//! (`model::SCORING`), at temperature 0, every answer printed; and two `When?` questions whose
//! right answer the situation gives away, to see whether the scores read the situation at all. Ignored by default: it
//! needs `core/models/qwen-1.5b.gguf`, or another model named by `MODEL`; `ONLY=<scoring>` runs one way of scoring;
//! `GPU=1` with `--features metal` (or `vulkan`) runs it on the GPU, many times faster. Run with
//! `cargo test --release -p lodger-core --test frames -- --ignored --nocapture`.

use lodger_core::brain::{SOON, SOON_CHOICES, WHAT, WHEN, WHEN_CHOICES};
use lodger_core::model::{Model, Question, Settings};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

const OLD: &str = "You are a quiet presence living unseen in a player's house in a game. \
You never chase, never hurt, and never hurry. You learn the player's habits and act on them \
when they are away or asleep, to make them feel known. You want one thing from them.\n\n\
What you know:\n";

fn new_frame(stage: u8) -> String {
    lodger_core::brain::situation::frame(stage)
}

/// The events of each kind on the menus, by id: what harms or shows it, and what stays quiet.
const LOUD: &[&str] = &[
    "choke",
    "villager_stabs",
    "vex_single",
    "while_unseen",
    "ceiling_crawl",
    "figure_swing",
];
const QUIET: &[&str] = &[
    "move_light",
    "tidy_chest",
    "painting_flip",
    "knock",
    "cobwebs",
    "drip",
];

fn words(id: &str) -> String {
    lodger_core::events::find(id)
        .map(|e| e.menu_words().to_string())
        .unwrap_or_else(|| panic!("{id}"))
}

fn situations() -> Vec<Vec<String>> {
    vec![
        vec![
            "The player sleeps in a house with 1 door and 4 lights.".to_string(),
            "They reach for iron pickaxe most.".to_string(),
            "Yesterday it showed itself far away.".to_string(),
            "Today it moved a torch.".to_string(),
            "The player is at home, awake, at night.".to_string(),
        ],
        vec![
            "The player sleeps in a house with 2 doors and 6 lights.".to_string(),
            "The chest by the door is opened most, 40 times so far. It holds bread, coal, iron ingot.".to_string(),
            "Two days ago it took the iron sword.".to_string(),
            "The player chased its figure and broke its torches.".to_string(),
            "The player is outdoors, walking, at dusk.".to_string(),
        ],
    ]
}

#[test]
#[ignore = "needs the real model under core/models; prints every answer"]
fn the_old_frame_and_the_new_scored_on_late_stage_menus() {
    // Another model with MODEL=<path> (the 4B in a game instance's config/lodger/models).
    let path = std::env::var("MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("models")
                .join("qwen-1.5b.gguf")
        });
    if !path.is_file() {
        eprintln!("no model; nothing to compare");
        return;
    }
    let model = Model::new(Settings {
        path,
        threads: 4,
        context: 1024,
        temperature: 0.0,
        seed: 7,
        // GPU=1 runs it all on the GPU (built with `--features metal` or `vulkan`).
        gpu_layers: if std::env::var("GPU").is_ok() { 999 } else { 0 },
        pause_ms: 1,
    });
    model.warm().expect("the model loads");
    let mut menu: Vec<String> = vec!["wait".to_string()];
    for (loud, quiet) in LOUD.iter().zip(QUIET) {
        menu.push(words(loud));
        menu.push(words(quiet));
    }
    let loud_words: Vec<String> = LOUD.iter().map(|id| words(id)).collect();
    let questions = vec![
        Question {
            text: SOON.to_string(),
            choices: SOON_CHOICES.iter().map(|c| c.to_string()).collect(),
        },
        Question {
            text: WHAT.to_string(),
            choices: menu.clone(),
        },
    ];
    let when = vec![Question {
        text: WHEN.to_string(),
        choices: WHEN_CHOICES.iter().map(|c| c.to_string()).collect(),
    }];
    let obvious = [
        (
            "The player is asleep in their bed. It is the middle of the night.",
            "while they sleep",
        ),
        (
            "The player left home this morning and is far away, mining.",
            "while they are away",
        ),
    ];
    for (mode, scoring) in [
        "plain",
        "spaced",
        "calibrated",
        "listed",
        "half",
        "lettered",
        "mixed",
    ]
    .iter()
    .enumerate()
    {
        if std::env::var("ONLY").is_ok_and(|only| only != *scoring) {
            continue;
        }
        lodger_core::model::SCORING.store(mode as u8, Ordering::Relaxed);
        for (name, frame) in [
            ("old", OLD.to_string()),
            ("new 4", new_frame(4)),
            ("new 5", new_frame(5)),
        ] {
            let mut loud = 0;
            let mut picks = std::collections::BTreeSet::new();
            for (i, lines) in situations().iter().enumerate() {
                for turn in 0..3 {
                    // The menu turned, so where a choice sits is not what is measured.
                    let mut turned = questions.clone();
                    turned[1].choices.rotate_left(turn * 4);
                    let mut text = frame.clone();
                    for line in lines {
                        text.push_str(line);
                        text.push('\n');
                    }
                    let answers = model.ask(&text, &turned).expect("answered");
                    if loud_words.contains(&answers[1].choice) {
                        loud += 1;
                    }
                    picks.insert(answers[1].choice.clone());
                    println!(
                        "{scoring} {name} s{i} t{turn}: soon {} ({:.2}); what {} ({:.2})",
                        answers[0].choice,
                        answers[0].confidence,
                        answers[1].choice,
                        answers[1].confidence
                    );
                }
            }
            let mut right = 0;
            for (line, wanted) in obvious {
                let answer = &model
                    .ask(&format!("{frame}{line}\n"), &when)
                    .expect("answered")[0];
                println!(
                    "{scoring} {name} when: {line} -> {} ({:.2})",
                    answer.choice, answer.confidence
                );
                if answer.choice == wanted {
                    right += 1;
                }
            }
            println!("SUMMARY {scoring} {name}: {loud} of 6 loud, {} different picks, {right} of 2 obvious whens right", picks.len());
        }
    }
    model.shutdown();
}

/// Whether the menu's wording decides the pick: six situations, each with one event that plainly
/// fits it, on one menu with the two that won every menu before; scored with the catalogue's
/// words and with short ones. Ignored; `MODEL` and `GPU=1` as above.
#[test]
#[ignore = "needs the real model under core/models; prints every answer"]
fn short_menu_words_against_long_on_situations_with_one_plain_fit() {
    let path = std::env::var("MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("models")
                .join("qwen-1.5b.gguf")
        });
    if !path.is_file() {
        eprintln!("no model; nothing to compare");
        return;
    }
    let model = Model::new(Settings {
        path,
        threads: 4,
        context: 1024,
        temperature: 0.0,
        seed: 7,
        gpu_layers: if std::env::var("GPU").is_ok() { 999 } else { 0 },
        pause_ms: 1,
    });
    model.warm().expect("the model loads");
    // Each situation, the event that fits it, and its short words.
    let cases = [
        (
            "The player is asleep in their bed. It is the middle of the night.",
            "choke",
            "choke them in their sleep",
        ),
        (
            "The player is in the village, trading with a villager.",
            "villager_stabs",
            "stab them through a villager",
        ),
        (
            "The player is deep in a cave, their torches along the tunnel behind them.",
            "cave_torches_flicker",
            "flicker their cave torches",
        ),
        (
            "The player is walking through a dark forest.",
            "peeks_from_the_trees",
            "peek at them from the trees",
        ),
        (
            "The player just opened their chest by the door.",
            "tidy_chest",
            "sort the chest",
        ),
        (
            "The player is swimming across a deep lake.",
            "swims_beneath",
            "swim beneath them",
        ),
    ];
    let others = [
        ("knock", "knock on the door"),
        ("move_light", "move a torch"),
    ];
    let scoring = std::env::var("SCORING")
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(1);
    lodger_core::model::SCORING.store(scoring, Ordering::Relaxed);
    for wording in ["long", "short"] {
        let words_of = |id: &str, short: &str| {
            if wording == "short" {
                short.to_string()
            } else {
                lodger_core::events::find(id)
                    .unwrap()
                    .menu_words()
                    .to_string()
            }
        };
        let mut menu: Vec<(String, &str)> = vec![("wait".to_string(), "wait")];
        for (_, id, short) in cases {
            menu.push((words_of(id, short), id));
        }
        for (id, short) in others {
            menu.push((words_of(id, short), id));
        }
        let mut fits = 0;
        for (line, wanted, _) in cases {
            for turn in 0..3 {
                let mut choices: Vec<(String, &str)> = menu.clone();
                choices.rotate_left(turn * 3);
                let question = Question {
                    text: WHAT.to_string(),
                    choices: choices.iter().map(|(w, _)| w.clone()).collect(),
                };
                let text = format!("{}{line}\n", new_frame(4));
                let answer = &model.ask(&text, &[question]).expect("answered")[0];
                let picked = choices
                    .iter()
                    .find(|(w, _)| *w == answer.choice)
                    .map(|(_, id)| *id)
                    .unwrap_or("?");
                if picked == wanted {
                    fits += 1;
                }
                println!(
                    "{wording} {wanted} t{turn}: {picked} ({:.2})",
                    answer.confidence
                );
            }
        }
        println!("FITS scoring {scoring} {wording}: {fits} of 18");
    }
    model.shutdown();
}

/// How long one piece of the model's work holds the GPU, and a decision's time, by chunk size:
/// what a frame may wait behind (owner: "whenever the model makes a decision the game stutters a
/// bit"). Ignored; `MODEL` and `GPU=1` as above.
#[test]
#[ignore = "needs the real model; prints the bursts"]
fn the_longest_burst_and_a_decision_s_time_by_chunk() {
    let path = std::env::var("MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("models")
                .join("qwen-1.5b.gguf")
        });
    if !path.is_file() {
        return;
    }
    let model = Model::new(Settings {
        path,
        threads: 4,
        context: 1536,
        temperature: 0.0,
        seed: 7,
        gpu_layers: if std::env::var("GPU").is_ok() { 999 } else { 0 },
        pause_ms: 1,
    });
    model.warm().expect("the model loads");
    let mut menu: Vec<String> = vec!["wait".to_string()];
    for (loud, quiet) in LOUD.iter().zip(QUIET) {
        menu.push(words(loud));
        menu.push(words(quiet));
    }
    let questions = vec![
        Question {
            text: SOON.to_string(),
            choices: SOON_CHOICES.iter().map(|c| c.to_string()).collect(),
        },
        Question {
            text: WHAT.to_string(),
            choices: menu,
        },
        Question {
            text: WHEN.to_string(),
            choices: WHEN_CHOICES.iter().map(|c| c.to_string()).collect(),
        },
    ];
    let mut text = new_frame(4);
    for line in &situations()[0] {
        text.push_str(line);
        text.push('\n');
    }
    lodger_core::model::CHUNK_TUNES.store(false, Ordering::Relaxed);
    for chunk in [64usize, 32, 16, 8, 0] {
        // 0: tuning itself, from 64, as the game runs it.
        lodger_core::model::CHUNK_TUNES.store(chunk == 0, Ordering::Relaxed);
        lodger_core::model::CHUNK.store(if chunk == 0 { 64 } else { chunk }, Ordering::Relaxed);
        model.ask(&text, &questions).expect("answered");
        lodger_core::model::take_longest_burst();
        let began = std::time::Instant::now();
        for _ in 0..3 {
            model.ask(&text, &questions).expect("answered");
        }
        let each = began.elapsed() / 3;
        let burst = lodger_core::model::take_longest_burst();
        println!(
            "CHUNK {chunk}: longest burst {:.1} ms, a decision {:.0} ms",
            burst.as_secs_f64() * 1000.0,
            each.as_secs_f64() * 1000.0
        );
    }
    model.shutdown();
}
