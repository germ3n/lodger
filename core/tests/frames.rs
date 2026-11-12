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
    for (mode, scoring) in ["plain", "spaced", "calibrated", "listed", "half"]
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
