# The SDK

craftkit (a working name; the crate name is free on crates.io as of 2026-11-08 and the owner picks
the final one later) is how a Minecraft mod gets written in Rust: a crate the mod is written
against (`sdk/`), and a host (`host/`), a Java mod for Fabric and NeoForge, that loads Rust mods
and carries out what they decide. The lodger is its first mod. This page is the SDK's design;
[Architecture](architecture.md) is how the lodger sits on it.

## What a Rust mod is

- **A crate** that implements `craftkit::Mod` and exports it with `craftkit::export_mod!`, built
  as a `cdylib`. It knows nothing of Minecraft's classes: it reads observations and answers
  commands, both plain `serde` structs (see [The protocol](#the-protocol)).
- **A content jar,** with no Java in it: the crate's shared library for each platform under
  `natives/<os>-<arch>/`, the mod's assets (lang files, textures, sounds), and three manifests:
  `META-INF/craftkit.json` for the host, `fabric.mod.json` for Fabric, and a `META-INF/neoforge.mods.toml` with no language
  loader for NeoForge. Both loaders load such a jar as a mod with
  resources, so one jar serves both, and the host finds it by its manifest.
- **A config file,** `config/<id>.json`, whose schema the mod owns. The host reads the text,
  hands it over, and writes back the effective config the mod answers when the file was missing.

```json
{ "id": "lodger", "library": "lodger_core", "config": "lodger.json" }
```

`id` is the mod id the other two manifests name; `library` is the cargo library name, so the
file is `lib<library>.dylib`, `lib<library>.so` or `<library>.dll`; `config` is optional and
defaults to `<id>.json`.

## What the host is

One Java mod, `craftkit`, built once for Fabric and once for NeoForge from one `common` source
tree (the MultiLoader layout: `host/common`, `host/fabric`, `host/neoforge`). Both loaders use
Mojang's mappings since Minecraft 26.x, so the common code compiles against vanilla once and each
loader adds its entry point and events. The host:

- finds every loaded jar with a `META-INF/craftkit.json`, extracts this platform's native to
  `config/craftkit/natives/<id>/<os>-<arch>/` (the same bytes are left alone), binds it through
  Java's FFM API (`Native.java`, the only Java that knows there is a native), checks the ABI
  version, reads its info, hands it its config;
- answers `/craftkit status` to operators: each mod, its version, the SDK it was built with, the
  native's platform, or why it did not load;
- refuses clients that do not run the host, with a configuration-phase channel on Fabric and a
  required payload on NeoForge, because Rust mods' client-side effects need the host on both
  sides;
- from phase 2 on, runs each mod's worlds once a tick: observations in, commands out, and the
  executors that carry commands out against the game (see [What the host does for a mod](#what-the-host-does-for-a-mod)).

A mod's assets load as any mod's do, so a content jar without a usable native (another platform,
an ABI mismatch) still contributes its lang files and textures and the host says why the mod
itself is not running.

## The C ABI

Exported by `export_mod!`, bound by `Native.java`. Every function takes UTF-8 bytes by pointer
and length and returns a NUL-terminated UTF-8 JSON string the host frees with `craftkit_free`,
so the Java side is a handful of method handles and no struct layouts. Errors are JSON too
(`{"error": "..."}`), never a panic across the boundary: each exported function runs under
`catch_unwind`, and a panic is logged by the host as an error.

| Function | ABI 1 | What it does |
| --- | --- | --- |
| `craftkit_abi_version() -> u32` | phase 0 | The ABI the library speaks. The host refuses any other, by name, before calling anything else. |
| `craftkit_info() -> json` | phase 0 | `ModInfo`: id, name, version, the SDK version built with. |
| `craftkit_init(config, len) -> json` | phase 0 | The config file's text in; `InitReply` out: the effective config and the unknown keys. Makes the one `Mod` of the process. |
| `craftkit_shutdown()` | phase 0 | The process is ending. Drops the `Mod`. |
| `craftkit_free(s)` | phase 0 | Frees an answer. Null does nothing. |
| `craftkit_world_open(state, len) -> handle` | phase 2 | A world loaded, with its saved bytes (empty for a new one). |
| `craftkit_world_observe(handle, json, len)` | phase 2 | What the body saw this tick. |
| `craftkit_world_tick(handle, now, len) -> json` | phase 2 | Advance one tick; the commands to carry out. |
| `craftkit_world_report(handle, json, len)` | phase 2 | How last tick's commands went. |
| `craftkit_world_save(handle) -> bytes` | phase 2 | The world's state for the save. |
| `craftkit_world_close(handle)` | phase 2 | The world unloaded. |

A change to any signature or JSON shape bumps `ABI_VERSION` in `sdk/src/lib.rs` and
`Native.ABI_VERSION` in the host together (`VersionsTest` checks they agree), and this table.

## The `Mod` trait

```rust
pub trait Mod: Sized + Send + 'static {
    fn info() -> ModInfo;
    fn init(config: &str) -> Result<(Self, InitReply), String>;
    // phase 2:
    // type World: Serialize + DeserializeOwned;
    // fn open(&mut self, saved: &[u8]) -> Result<Self::World, String>;
    // fn observe(&mut self, world: &mut Self::World, observations: Observations);
    // fn tick(&mut self, world: &mut Self::World, now: Now) -> Vec<Command>;
    // fn report(&mut self, world: &mut Self::World, reports: Vec<Report>);
}
```

One `Mod` per process, made by `init`, with a `World` per loaded world. The host calls every
function on the server thread, once a tick at most, so a mod needs no locks of its own; a mod
that works on threads of its own (the lodger's model worker) brings its answers back inside
`tick`, as the lodger's [Architecture](architecture.md#threads) describes.

## The protocol

`sdk/src/protocol.rs`: the JSON shapes, as `serde` structs, so the host's Gson and the mod's
`serde_json` read the same bytes. Phase 0 has `ModInfo`, `InitReply` and `Failure`. Phase 2 adds,
from the lodger's needs first and generalized as a second mod needs them:

- **Observations,** the mod's senses, rich enough that a mod can know the world the way a
  player does without ever asking Java for it:
  - *players:* position, facing, velocity, sneaking, sprinting, swimming, riding, sleeping,
    health, hunger, air, experience level, held items, armour, active effects, dimension, the
    biome under them, the light level at them, whether they are indoors (sky light), what they
    look at (block or entity, within 8 blocks), what they just did (opened a container with its
    contents, placed or broke a block, slept or were refused, ate, fished, mounted, logged in or
    out, died, respawned, changed dimension);
  - *the world around each player,* once a second within 64 blocks: time of day, weather, moon
    phase, the biome at the player and the biomes within 64 blocks; every entity within 64
    blocks with its kind, position, name if any, owner if tamed, whether it looks at the player,
    whether it is hostile; and on request (a `Scan` command, answered next tick) a summary of
    blocks in a box: counts by kind, the positions of named kinds (doors, chests, beds, torches,
    ores), light levels, whether a position is in view of a player;
  - *the surroundings,* once an in-game day within 128 blocks of a point the mod names: water
    bodies and their extent, snow and sand, villages with their doors and bell, fields, pens with
    their animals, caves not visited, tunnels dug by players with their sizes, structures.

  The per-tick payload stays small because the world's state comes on request and on a
  schedule, not every tick: a quiet tick carries a player's position and nothing else.
- **Scoring,** for mods that use a model as the lodger does: the SDK has no model in it; the
  lodger's `model/` is the lodger's. What the SDK gives such a mod is the observations above in
  a form `describe` can turn into lines of text, and a worker-thread pattern (`sdk::work`) for
  bringing answers back inside `tick`.
- **Commands:** open or close a door, move or place a block, move items between a world's
  containers, drop an item, place a sign with given lines, spawn and drive a figure entity, play
  a sound to a player at a place, send a client hint (a kind, a place, a duration), set an
  entity's look, cancel a player's sleep. Each has a `fits` the host's executor checks before
  running (see [Events re-check](entity.md#events-re-check-before-they-happen)).
- **Reports:** per command, `done` or `did not fit` with why.
- **Now:** the tick, the world's time, the real clock, so a mod never reads a clock itself.
- **The menu,** for setup and debugging: a mod declares pages of fields and actions once
  (`Menu` in `init`'s reply), and the host renders them twice over, as a chat command tree
  under `/craftkit <id> ...` and as a GUI screen a client opens with `/craftkit <id> menu`. A
  field is a toggle, a number with bounds, a choice, or a read-only line the mod refreshes; an
  action is a button that sends the mod a `MenuAction` command next tick. The lodger's pages:
  status (stage, target, want, model loaded, last decisions), config (every key of
  `lodger.json`, written back on change), debug (force a stage, force an event now, dump the
  memory's lines, run the rule brain, replay the last situation through the model). The host
  owns rendering, permissions (operators only) and the config write-back; the mod owns what is
  on the pages. A mod with no menu gets the host's status line alone.

Positions are block positions as three integers, items are ids as strings, players are indices
into the observation's player list. Nothing in the protocol is a Minecraft class.

## What the host does for a mod

The host owns the Minecraft side so a mod never has to: the FFM binding, the per-tick exchange,
the senses (the observations above, the scans and the daily surroundings pass, sliced over ticks
so none of it shows in a tick profile), the saved data (a mod's world bytes as a byte array in
the world's saved data), the executors for every command and their `fits` checks, the client
side (figure renderers, sounds, hints, the menu screen) and the packets to carry them, the
commands (`/craftkit status`, and each mod's menu as `/craftkit <id> ...`), and the handshake. A
new command is a row in the protocol, an executor in `host/common`, a game test for it, and a
line in this page.

## Testing a mod without a game

`craftkit::harness!()` builds a `Harness` from the functions `export_mod!` exported in the same
crate, and drives them as the host does: bytes in, JSON out, panics caught, the ABI version
checked. A mod's tests ask it `info` and `init` (and from phase 2 open a world and tick it with
scripted observations) in `cargo test`, in milliseconds. The SDK's own `tests/exported.rs` does
the same with a two-line mod, and is the test of the macro.

## Versions and publishing

The SDK crate and the host carry one version (`sdk/Cargo.toml`, `host/gradle.properties`); a mod
declares the least host it needs in its manifests (`craftkit >= 0.1.0`). The ABI version moves
slower than the crate's and is what the host actually checks. Publishing (the crate to crates.io,
the host jars to Modrinth) waits until a second mod can be written against it, which is the todo's
last phase; until then both live in this repository.
