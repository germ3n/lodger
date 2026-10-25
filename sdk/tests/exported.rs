//! A mod of two lines, exported and driven through the harness: the path every real mod takes.

use craftkit::{InitReply, Mod, ModInfo};
use std::sync::atomic::{AtomicUsize, Ordering};

static MADE: AtomicUsize = AtomicUsize::new(0);

struct Tiny;

impl Mod for Tiny {
    fn info() -> ModInfo {
        ModInfo::new("tiny", "Tiny", "0.0.1")
    }

    fn init(config: &str) -> Result<(Self, InitReply), String> {
        if config == "refuse" {
            return Err("refused on purpose".to_string());
        }

        if config == "panic" {
            panic!("panicked on purpose");
        }

        MADE.fetch_add(1, Ordering::SeqCst);

        Ok((
            Tiny,
            InitReply {
                config: serde_json::json!({ "given": config }),
                unknown: vec![],
            },
        ))
    }
}

craftkit::export_mod!(Tiny);

#[test]
fn a_mod_is_loaded_asked_and_shut_down_through_the_abi() {
    let harness = craftkit::harness!().unwrap();

    let info = harness.info();
    assert_eq!(info["id"], "tiny");
    assert_eq!(info["version"], "0.0.1");
    assert_eq!(info["sdk"], craftkit::VERSION);

    let reply = harness.init("{}");
    assert_eq!(reply["config"]["given"], "{}");
    assert_eq!(reply["unknown"], serde_json::json!([]));
    assert_eq!(MADE.load(Ordering::SeqCst), 1);

    harness.shutdown();
}

#[test]
fn a_refused_init_is_an_error_and_a_panicking_one_too() {
    let harness = craftkit::harness!().unwrap();

    assert_eq!(harness.init("refuse")["error"], "refused on purpose");
    assert_eq!(
        harness.init("panic")["error"],
        "the mod panicked: panicked on purpose"
    );
}

#[test]
fn the_library_speaks_the_sdk_s_abi_version() {
    assert_eq!(craftkit_abi_version(), craftkit::ABI_VERSION);
}
