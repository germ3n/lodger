//! A mod of a few lines, exported and driven through the harness: the path every real mod takes.

use craftkit::debug::{DebugReply, DebugRequest, Gauge};
use craftkit::{Command, Init, InitReply, Mod, ModInfo, Now, Observations, Report, WorldInfo};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static MADE: AtomicUsize = AtomicUsize::new(0);

/// The exported functions share one static mod, so the tests that init and shut it down take
/// turns, as the host's calls do.
static TURN: Mutex<()> = Mutex::new(());

fn turn() -> std::sync::MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(|err| err.into_inner())
}

struct Tiny;

/// Tiny's world: counts what it saw, answers a log line every tenth tick, saves its count.
struct Counter {
    seed: i64,
    ticks: u64,
    players_seen: u64,
    reports: u64,
}

impl Mod for Tiny {
    type World = Counter;

    fn info() -> ModInfo {
        ModInfo::new("tiny", "Tiny", "0.0.1")
    }

    fn init(init: &Init) -> Result<(Self, InitReply), String> {
        if init.config == "refuse" {
            return Err("refused on purpose".to_string());
        }

        if init.config == "panic" {
            panic!("panicked on purpose");
        }

        MADE.fetch_add(1, Ordering::SeqCst);

        Ok((
            Tiny,
            InitReply {
                settings: Vec::new(),
                config: serde_json::json!({ "given": init.config, "own_dir": init.own_dir() }),
                unknown: vec![],
                warnings: Vec::new(),
                resources: vec![],
                rewrite: false,
            },
        ))
    }

    fn open(&self, info: &WorldInfo, saved: &[u8]) -> Result<Counter, String> {
        let ticks = if saved.is_empty() {
            0
        } else {
            u64::from_le_bytes(saved.try_into().map_err(|_| "bad save".to_string())?)
        };

        Ok(Counter {
            seed: info.seed,
            ticks,
            players_seen: 0,
            reports: 0,
        })
    }

    fn observe(&self, world: &mut Counter, observations: &Observations) {
        world.players_seen += observations.players.len() as u64;
    }

    fn tick(&self, world: &mut Counter, now: &Now) -> Vec<Command> {
        world.ticks += 1;

        if now.tick.is_multiple_of(10) {
            vec![Command::Log {
                id: now.tick,
                text: format!(
                    "seed {} tick {} players {} reports {}",
                    world.seed, world.ticks, world.players_seen, world.reports
                ),
            }]
        } else {
            vec![]
        }
    }

    fn report(&self, world: &mut Counter, reports: &[Report]) {
        world.reports += reports.len() as u64;
    }

    fn save(&self, world: &Counter) -> Vec<u8> {
        world.ticks.to_le_bytes().to_vec()
    }

    fn debug(&self, world: &mut Counter, request: &DebugRequest) -> DebugReply {
        match request {
            DebugRequest::Snapshot => DebugReply {
                gauges: vec![Gauge::new(
                    "ticks",
                    world.ticks as f32,
                    100.0,
                    format!("{} ticks", world.ticks),
                )],
                ..DebugReply::default()
            },
            DebugRequest::Action { id } if id == "reset" => {
                world.ticks = 0;
                DebugReply::message("the count is zero again")
            }
            other => DebugReply::message(format!("tiny cannot {other:?}")),
        }
    }
}

craftkit::export_mod!(Tiny);

#[test]
fn the_debug_screen_s_questions_reach_the_mod_s_world_and_come_back_as_a_reply() {
    let _turn = turn();
    let harness = craftkit::harness!().unwrap();
    harness.init("{}");
    let world = harness
        .open_world(
            &WorldInfo {
                id: "minecraft:overworld".to_string(),
                seed: 1,
                name: "w".to_string(),
            },
            &[],
        )
        .unwrap();
    world.tick(&Now::default());
    world.tick(&Now::default());

    let panel = world.debug(&DebugRequest::Snapshot);
    assert_eq!(panel.gauges[0].text, "2 ticks");
    assert_eq!(panel.gauges[0].value, 2.0);

    let reply = world.debug(&DebugRequest::Action {
        id: "reset".to_string(),
    });
    assert_eq!(reply.message, "the count is zero again");
    assert_eq!(world.debug(&DebugRequest::Snapshot).gauges[0].value, 0.0);

    let refused = world.debug(&DebugRequest::SetStage { stage: 3 });
    assert!(refused.message.starts_with("tiny cannot"));
    world.close();
}

#[test]
fn a_mod_is_loaded_asked_and_shut_down_through_the_abi() {
    let _turn = turn();
    let harness = craftkit::harness!().unwrap();

    let info = harness.info();
    assert_eq!(info["id"], "tiny");
    assert_eq!(info["version"], "0.0.1");
    assert_eq!(info["sdk"], craftkit::VERSION);

    let reply = harness.init("{}");
    assert_eq!(reply["config"]["given"], "{}");
    assert_eq!(reply["unknown"], serde_json::json!([]));
    assert!(MADE.load(Ordering::SeqCst) >= 1);

    harness.shutdown();
}

#[test]
fn a_refused_init_is_an_error_and_a_panicking_one_too() {
    let _turn = turn();
    let harness = craftkit::harness!().unwrap();

    assert_eq!(harness.init("refuse")["error"], "refused on purpose");
    assert_eq!(
        harness.init("panic")["error"],
        "the mod panicked: panicked on purpose"
    );
}

#[test]
fn init_carries_the_directories_and_the_mod_knows_its_own_folder() {
    let _turn = turn();
    let harness = craftkit::harness!().unwrap();
    let init = Init {
        config: "{}".to_string(),
        game_dir: "/game".to_string(),
        config_dir: "/game/config".to_string(),
        id: "tiny".to_string(),
        ..Init::default()
    };

    assert_eq!(
        harness.init_with(&init)["config"]["own_dir"],
        "/game/config/tiny"
    );
}

#[test]
fn init_with_bytes_that_are_not_init_json_is_an_error_naming_the_host() {
    let _turn = turn();
    let text = "{ nope";
    let answer = unsafe { craftkit::abi::take(craftkit_init(text.as_ptr(), text.len())) };

    assert!(answer["error"]
        .as_str()
        .unwrap()
        .starts_with("bad JSON from the host"));
}

#[test]
fn a_mod_with_nothing_to_load_is_ready_at_once_and_warm_is_a_no_op() {
    let _turn = turn();
    let harness = craftkit::harness!().unwrap();
    harness.init("{}");

    let status = harness.status();
    assert_eq!(status["ready"], true);
    assert_eq!(status["line"], "Ready");
    assert_eq!(harness.warm(), serde_json::json!({ "ok": true }));
}

#[test]
fn a_resource_defaults_to_required_with_no_magic_or_hash() {
    let resource: craftkit::Resource = serde_json::from_str(
        r#"{ "id": "m", "title": "t", "url": "https://x/m", "path": "models/m" }"#,
    )
    .unwrap();

    assert!(resource.required);
    assert_eq!(resource.size_hint, 0);
    assert_eq!(resource.magic, "");
    assert_eq!(resource.sha256, "");
    assert!(resource.license.is_empty());
}

#[test]
fn a_world_is_opened_observed_ticked_reported_saved_and_reopened_from_its_bytes() {
    let _turn = turn();
    let harness = craftkit::harness!().unwrap();
    harness.init("{}");
    let info = WorldInfo {
        id: "minecraft:overworld".to_string(),
        seed: 42,
        name: "New World".to_string(),
    };

    let world = harness.open_world(&info, &[]).unwrap();
    let observations = Observations {
        players: vec![craftkit::world::Player {
            index: 0,
            name: "razor".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert_eq!(
        world.observe(&observations),
        serde_json::json!({ "ok": true })
    );

    let mut commands = Vec::new();
    for tick in 1..=10u64 {
        commands.extend(world.tick(&Now {
            tick,
            ..Default::default()
        }));
    }
    assert_eq!(commands.len(), 1, "one log line in ten ticks: {commands:?}");
    assert_eq!(commands[0].id(), 10);
    match &commands[0] {
        Command::Log { text, .. } => assert_eq!(text, "seed 42 tick 10 players 1 reports 0"),
        other => panic!("{other:?}"),
    }

    world.report(&[Report {
        id: 10,
        done: true,
        reason: String::new(),
    }]);
    let saved = world.save();
    assert_eq!(saved, 10u64.to_le_bytes());
    assert_eq!(world.close(), serde_json::json!({ "ok": true }));

    let again = harness.open_world(&info, &saved).unwrap();
    let later: Vec<Command> = again.tick(&Now {
        tick: 20,
        ..Default::default()
    });
    match &later[0] {
        Command::Log { text, .. } => assert!(text.starts_with("seed 42 tick 11"), "{text}"),
        other => panic!("{other:?}"),
    }
    again.close();
}

#[test]
fn a_closed_or_unknown_handle_is_an_error_not_a_panic() {
    let _turn = turn();
    let harness = craftkit::harness!().unwrap();
    harness.init("{}");

    let answer = unsafe { craftkit::abi::take(craftkit_world_save(999)) };
    assert!(answer["error"]
        .as_str()
        .unwrap()
        .contains("no open world with handle 999"));
}

#[test]
fn the_library_speaks_the_sdk_s_abi_version() {
    assert_eq!(craftkit_abi_version(), craftkit::ABI_VERSION);
}
