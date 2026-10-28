//! Every command the protocol has, the host carries out: a command added to `sdk/src/world.rs`
//! without an executor fails here, not in a player's log as "the host does not carry out ...".
//! The repository's own sources are read, the SDK's enum and the host's dispatch.

use std::fs;

fn repo(path: &str) -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../").to_string() + path)
        .unwrap_or_else(|err| panic!("{path}: {err}"))
}

/// `MoveBlock` as the wire names it, `move_block`.
fn snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn every_command_kind_has_an_executor_in_the_host() {
    let world = repo("sdk/src/world.rs");
    let start = world.find("pub enum Command").expect("the Command enum");
    let body = &world[start..];
    let body = &body[..body.find("\n}\n").expect("its end")];
    let kinds: Vec<String> = body
        .lines()
        .filter_map(|l| l.strip_prefix("    ")?.strip_suffix(" {"))
        .filter(|name| name.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        .map(snake)
        .collect();
    assert!(kinds.len() > 30, "the enum was read: {kinds:?}");

    let host = repo("host/common/src/main/java/craftkit/host/world/Executors.java")
        + &repo("host/common/src/main/java/craftkit/host/world/Worlds.java");
    let missing: Vec<&String> = kinds
        .iter()
        .filter(|kind| !host.contains(&format!("case \"{kind}\"")))
        .collect();

    assert!(
        missing.is_empty(),
        "commands the host does not carry out: {missing:?}"
    );
}
