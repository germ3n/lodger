# craftkit

Write Minecraft mods in Rust.

A craftkit mod is a Rust crate that implements [`Mod`](https://docs.rs/craftkit/latest/craftkit/trait.Mod.html)
and exports it with `export_mod!`, built as a shared library (`cdylib`) and shipped in a content
jar beside its assets and a manifest. The **craftkit host**, a Java mod for Fabric and NeoForge,
loads the library through Java's FFM API and talks to it through a small C ABI: once a tick it
hands the mod what the world did (players, entities, blocks used, chat) and takes back what the
mod wants done (set a block, play a sound, show a figure, hint the client). The mod never
touches Minecraft's classes, so it is tested with the bundled harness in plain `cargo test`, and
survives the Minecraft versions the host is updated for.

The crate and the host live together in [github.com/germ3n/craftkit](https://github.com/germ3n/craftkit):
the host is its `host/` Gradle project, and players need its jar in the game beside a craftkit
mod, or nested inside the mod's own jar.

The first mod written on it is [lodger](https://github.com/germ3n/lodger), a horror mod whose
mind is a craftkit crate.

## A mod

```toml
[package]
name = "my_mod"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
craftkit = "0.1"
```

```rust
use craftkit::{Command, Init, InitReply, Mod, ModInfo, Now, Observations, Report, WorldInfo};

struct Hello;

/// One world's state: how many ticks it has seen, saved with the world.
struct World {
    ticks: u64,
}

impl Mod for Hello {
    type World = World;

    fn info() -> ModInfo {
        ModInfo::new("hello", "Hello", "0.1.0")
    }

    fn init(_init: &Init) -> Result<(Self, InitReply), String> {
        Ok((
            Hello,
            InitReply {
                config: serde_json::json!({}),
                unknown: vec![],
                warnings: vec![],
                resources: vec![],
                settings: vec![],
                rewrite: false,
            },
        ))
    }

    fn open(&self, _info: &WorldInfo, saved: &[u8]) -> Result<World, String> {
        let ticks = saved.try_into().map(u64::from_le_bytes).unwrap_or(0);
        Ok(World { ticks })
    }

    fn observe(&self, _world: &mut World, _observations: &Observations) {}

    fn tick(&self, world: &mut World, now: &Now) -> Vec<Command> {
        world.ticks += 1;
        if world.ticks % 200 == 0 {
            vec![Command::Log {
                id: now.tick,
                text: format!("{} ticks in this world", world.ticks),
            }]
        } else {
            vec![]
        }
    }

    fn report(&self, _world: &mut World, _reports: &[Report]) {}

    fn save(&self, world: &World) -> Vec<u8> {
        world.ticks.to_le_bytes().to_vec()
    }
}

craftkit::export_mod!(Hello);

// The harness loads the mod through the same exported functions the host calls.
fn main() {
    let harness = craftkit::harness!().unwrap();
    harness.init("{}");
    let world = harness
        .open_world(
            &WorldInfo {
                id: "minecraft:overworld".to_string(),
                seed: 1,
                name: "test".to_string(),
            },
            &[],
        )
        .unwrap();
    for tick in 1..=200 {
        let made = world.tick(&Now {
            tick,
            ..Default::default()
        });
        assert_eq!(made.len(), usize::from(tick == 200));
    }
    world.close();
}
```

## What is in the crate

- **The `Mod` trait and `export_mod!`:** one mod per process, a `World` per loaded world; the host
  calls the world functions on the server thread, once a tick at most.
- **The protocol:** `Observations` in, `Command`s out, `Report`s on how each command went, in
  `craftkit::world`; the settings, resources and debug screen a mod can offer, in `protocol`
  and `debug`.
- **The C ABI** (`abi`): JSON over a handful of exported functions, versioned by
  `ABI_VERSION`. A host refuses a library built against another ABI.
- **The harness** (`harness`): loads a mod as the host does, without a game, for tests.

The protocol, the ABI and the host are described in full in the
[SDK guide](https://github.com/germ3n/craftkit/blob/master/docs/sdk.md).

## Versions

The crate and the host share a version; a mod's manifest names the least host it needs
(`craftkit >= 0.1.2`). The ABI version moves more slowly and is what the host checks when it
loads a library. Minecraft Java 26.3 on Java 25, Fabric or NeoForge.

## License

MIT.
