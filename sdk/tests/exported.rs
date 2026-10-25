//! A mod of two lines, exported and driven through the harness: the path every real mod takes.

use craftkit::{Init, InitReply, Mod, ModInfo};
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

impl Mod for Tiny {
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
                config: serde_json::json!({ "given": init.config, "own_dir": init.own_dir() }),
                unknown: vec![],
                resources: vec![],
            },
        ))
    }
}

craftkit::export_mod!(Tiny);

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
    // Other tests in this file init the same static mod, in parallel, so at least one.
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
    assert_eq!(harness.warm(), serde_json::json!(true));
}

#[test]
fn a_resource_defaults_to_required_with_no_magic_or_hash() {
    let _turn = turn();
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
fn the_library_speaks_the_sdk_s_abi_version() {
    let _turn = turn();
    assert_eq!(craftkit_abi_version(), craftkit::ABI_VERSION);
}
