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
- **A jar per loader,** with no Java of the mod's own: the crate's shared library for each of
  six platforms (Linux, macOS, Windows; x86_64 and arm64) under `natives/<os>-<arch>/`, the
  mod's assets (lang files, textures, sounds), `META-INF/craftkit.json` for the host, the
  loader's manifest (`fabric.mod.json`, or a `META-INF/neoforge.mods.toml` with no language
  loader), and the host itself nested inside (`META-INF/jars/` on Fabric, `META-INF/jarjar/` on
  NeoForge), so a player installs one file and never meets craftkit by name. Two Rust mods on
  one server each nest a host; the loader keeps the newest once (the owner's call, 2026-11-09;
  before it the host was a jar of its own and one content jar served both loaders).
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

| Function | Since | What it does |
| --- | --- | --- |
| `craftkit_abi_version() -> u32` | phase 0 | The ABI the library speaks. The host refuses any other, by name, before calling anything else. |
| `craftkit_info() -> json` | phase 0 | `ModInfo`: id, name, version, the SDK version built with. |
| `craftkit_init(init, len) -> json` | phase 0 | `Init` JSON in: the config file's text, the game and config directories and the mod's id; `InitReply` out: the effective config and the unknown keys. Makes the one `Mod` of the process. A mod keeps what is not its config under `<config_dir>/<id>/` (`Init::own_dir`). |
| `craftkit_warm() -> json` | ABI 3 | Loads what the mod loads before play (the lodger: its model), blocking. The host calls it on a thread of its own once the resources are there. `true`, or an error. |
| `craftkit_status() -> json` | ABI 3 | `Status`: ready or not, one line why, whether `warm` is working, the last error. Safe from any thread; the title screen polls it. |
| `craftkit_shutdown()` | phase 0 | The process is ending. Drops the `Mod`. |
| `craftkit_free(s)` | phase 0 | Frees an answer. Null does nothing. |
| `craftkit_world_open(info, len, saved, len) -> json` | ABI 4 | A world loaded: its `WorldInfo` (id, seed, name) and its saved bytes, empty for a new one; `{"handle": n}` back. |
| `craftkit_world_observe(handle, json, len) -> json` | ABI 4 | What the host saw this tick, an `Observations`. |
| `craftkit_world_tick(handle, now, len) -> json` | ABI 4 | Advance one tick with a `Now`; the `Command` list to carry out back. |
| `craftkit_world_report(handle, json, len) -> json` | ABI 4 | How last tick's commands went, a `Report` list. |
| `craftkit_world_save(handle) -> json` | ABI 4 | `{"bytes": "<base64>"}`, the world's state for the save. |
| `craftkit_world_close(handle) -> json` | ABI 4 | The world unloaded. `craftkit_shutdown` closes any still open. |
| `craftkit_world_debug(handle, json, len) -> json` | ABI 5 | The debug screen's question about one world, a `DebugRequest` (`snapshot`, `force`, `set_stage`, `action`); a `DebugReply` back (a message, gauges, sections, every event with whether it is offered and why not, buttons). `Mod::debug` has a default that has no page. |

A change to any signature or JSON shape bumps `ABI_VERSION` in `sdk/src/lib.rs` and
`Native.ABI_VERSION` in the host together (`VersionsTest` checks they agree), and this table.
ABI 2 (2026-11-09) gave `init` the directories and the id; ABI 1 took the config text alone.
ABI 3 (2026-11-09) added `warm`, `status` and `InitReply.resources`. ABI 4 (2026-11-09) added
the six world functions and `sdk/src/world.rs`, the per-world shapes. ABI 5 (2026-11-09) added
`craftkit_world_debug` and `sdk/src/debug.rs`, nine commands (`open_lid`, `rotate_frame`,
`swap_painting`, `pose_stand`, `name_entity`, `dye_sheep`, `move_entity`, `player_list`,
`game_message`, and `push_entity`, `rotate_block`, `set_frame_item`, `equip_entity` and
`damage_item` the same night, and `figure.seconds` and `figure.vanish_within`),
`set_block.replace` and the `killed` event. ABI 6 (2026-11-09) gave `set_block` `settle` and
`wall` (a block stands where a player could place it, never over air) and `closed_container`
its position and the contents as the player left them. ABI 7 (2026-11-09) gave a debug
`Section` its `items`: rows with buttons of their own (a plan with Drop and Now). ABI 8
(2026-11-09) added `hold`: a mob held where it stands, facing a player, attacking no one (the second hit it takes wakes it), its
AI given back as it was, and never saved frozen (a tag, a sweep, a release on unload). ABI 9
(2026-11-09) added `move_player`, a player moved at most 32 blocks to a spot with room to stand,
where `woke_elsewhere` had sent the player as a made-up entity id no host knew. ABI 10
(2026-11-09) gave `set_block` `rest` (on something solid, never in the air) and `only` (over
one of `replace`, never into air), and `set_sign` `settle` (the sign looked for down and up
its column, where a settled sign landed). ABI 11 (2026-11-09) gave a debug `EventRow` its
`about`, the screen's tooltip; and the host answers a `teleport` request from the screen itself
(an operator clicking a spot), with no mod involved. The surroundings pass also reads the house around its middle
block by block, each door, light, chest and bed head as a `house_*` feature at its own
position (2026-11-09, no change of shape). ABI 12 (2026-11-09) added `block_sound`: the step, place, break
or hit sound of a real block (`under` the player, `held` in their hand, or by id), so a mod's
footsteps sound like what is underfoot. ABI 13 (2026-11-09): `take_item`'s `keep` keeps the very stack
taken, saved with the world, and `put_item` and `drop_item`'s `kept` give it back, components
and all (`put_item` with `wear` and a `name` as it goes back), where before a taken enchanted
pickaxe came back plain; `rename_item`; and `move_item` moves what is there, up to its count,
into the slot asked or the first that fits. ABI 14 (2026-11-09): `sign_book` (a book and quill signed as a
written book with no author) and `mark_map` (a target mark on a filled map). ABI 15 (2026-11-09): `move_block`'s `around`, a search for room on
solid ground, and a single-block container moved with what it holds; `move_item` takes the
first slot with something in it when the one asked is empty. A `scan` kind starting with `*` is a suffix (`*_stairs`), no
change of shape (2026-11-09). ABI 16 (2026-11-09): `arm_loot`, an offering for a player's next
fresh loot that the host applies as the loot is made (the game's own unpacking, then one item
added or swapped, the swapped one kept), answered by the `loot_touched` event. ABI 17 (2026-11-09): `set_state` (one property of a block, a
door's other half with it; `@next` for the next value), and `set_block`'s `state` applied at
last (`facing=north`), which the host had ignored. ABI 18 (2026-11-09): `grow`, bone meal as a player uses it, until
the block is something else. ABI 19 (2026-11-09): the `used_door` event, and `set_state`'s
`@turn` turning a bed round where it stands. ABI 20 (2026-11-09): a `Resource`'s `choices`, other forms of it
the player may pick on the preparing screen, each a label and the change to the mod's config
that picks it; the host merges the change into the mod's config file and initialises the mod
again for its new resources. ABI 21 (2026-11-09): `InitReply.settings`, the keys a mod offers on
the host's settings screen (a button for each such mod on the title screen): each a key, a
label, a group and a kind (`choice`, `toggle`, `number` with its range); Done writes the changes
into the config file and initialises the mod again, and anything that leaves to fetch (a new
model size) brings the preparing screen up. ABI 22 (2026-11-10): `InitReply.warnings`, config
values of the wrong type the mod mended or left at their defaults, in words, which the host logs
once (`lodger: config: brain.most_plans: invalid type ...; the default kept`); the settings
screen writes a number with a whole step as a whole number (`2`, never `2.0`). ABI 23
(2026-11-10): `scan` reads every `step`th block (the limit counts the blocks read, not the box),
keeps only `dark` blocks (no sky light, no block light) when asked, and answers at most `most`
positions. ABI 24 (2026-11-10): `swap_block`, another block in a block's place with every
property they share kept (a wall torch's facing), only over one of `only`; `hide_block`, a block
taken away for `seconds` and put back as it was, at the time or at once when its level saves or
closes. ABI 25 (2026-11-10): `warp_mob`, an animal frozen with its head on a player, perhaps
`scale`d, `flip`ped (the Dinnerbone name, hidden, never over a name of its own), `creep`ing a
step closer while not looked at, perhaps moved `to` a place first; put back as it was, where it
was, after `seconds` (two minutes at most) or when the player comes within six blocks; a mob saved
while warped carries a tag of how it was and is put back when next seen. ABI 26 (2026-11-10):
`shift_bed`, a bed one block to its side, both halves, never one someone lies in, a respawn point
at it moved with it; `bed_beside`, a bed of another block beside one, the same facing; and
`swap_block` over a bed turns both its halves together. ABI 27 (2026-11-10): `place_entity`
(an armour stand or a farm animal, for a while when `seconds` are given and then gone, gone at
once when the level closes; wearing a `kept` stack), `release_bees` (a hive's bees held still
around a place, then home), `move_leash` (a lead to the nearest other fence post within eight
blocks), `charge` (an animal runs at a player and hits, `spare` keeping them above half a heart);
`equip_entity`'s `kept`, `hold`'s `at` (face a point, not the player), and an entity's `leashed`.
ABI 28 (2026-11-10): `set_sign`'s `one_line` (the last line with text takes the first of
`lines`, the rest kept, a blank sign refused) and `set_frame_item`'s `from` and `slot` (the very
stack from a container's slot, only into an empty frame), and three `game_message` kinds:
`advancement` (one the player has, its line, and the toast through the `advancement_toast` hint),
`death` (one of the game's one-name death lines, by key) and `no_ping` (their own entry without a
connection for a minute). ABI 29 (2026-11-10): `jukebox` (an empty one plays a disc from a slot,
one playing stops, its disc back), `turn_page` (a lectern's book to another page), `brew` (an
ingredient into a stand that holds potions, a blaze powder when it has no fuel; the game brews),
`strip` (one enchantment, never a curse, off an item in a container); `set_state`'s `@up` (one
value higher, refused at the top) and `toggle_block` ringing a bell. ABI 30 (2026-11-10):
`prints` (footprints in the snow, a layer off at each step, left and right of a line) and
`place_entity`'s `face` (held still facing a player for its time); the client's `ripples` and
`bobber` hints.

## Resources

A mod's `init` reply lists the files it needs that are not in its jar (`Resource`: id, title,
URL, path, a size hint, the first bytes the file must have, an optional SHA-256, required or
not). The host fetches the missing required ones before the game is playable: on a client a
screen over the title screen with a progress bar, Quit, and Try again on failure; on a
dedicated server at startup, blocking, with progress in the log. Then it calls `warm` and polls
`status` until the mod says it is ready. Optional resources download in the background and the
mod goes without until they land. A resource with a `license` (a name, a URL, a short text) is
shown first, once: on a client a screen with the text, a button to read the full terms in a
browser, Accept and Quit, the acceptance kept in `config/craftkit/accepted.json`; on a
dedicated server the terms go to the log with the line to add to that file, and the resource
waits for the operator (the owner's call, 2026-11-09: no one accepts terms on another's behalf). One downloader (`Downloader.java`) serves every mod, writes
through a `.part` file, checks the magic on the first bytes and the hash over the whole file at
the end, and never replaces a good file with a failed download; a connection quiet for thirty
seconds is dropped and the download goes on from where it stopped (an HTTP range, five tries),
and a part file cut off by the network is taken up at the next start.

## The `Mod` trait

```rust
pub trait Mod: Sized + Send + Sync + 'static {
    type World: Send + 'static;
    fn info() -> ModInfo;
    fn init(init: &Init) -> Result<(Self, InitReply), String>;
    fn warm(&self) -> Result<(), String> { Ok(()) }
    fn status(&self) -> Status { /* ready */ }
    fn open(&self, info: &WorldInfo, saved: &[u8]) -> Result<Self::World, String>;
    fn observe(&self, world: &mut Self::World, observations: &Observations);
    fn tick(&self, world: &mut Self::World, now: &Now) -> Vec<Command>;
    fn report(&self, world: &mut Self::World, reports: &[Report]);
    fn save(&self, world: &Self::World) -> Vec<u8>;
    fn close(&self, world: Self::World) { drop(world) }
}
```

One `Mod` per process, made by `init`, with a `World` per loaded world. The host calls every
world function on the server thread, once a tick at most, so a mod needs no locks of its own
beyond what its own threads need; `warm` and `status` come from host threads, which is why the
mod is `Sync`. A mod that works on threads of its own (the lodger's model worker) brings its
answers back inside `tick`, as the lodger's [Architecture](architecture.md#threads) describes.

## The protocol

`sdk/src/protocol.rs` and `sdk/src/world.rs`: the JSON shapes, as `serde` structs, so the host's
Gson and the mod's `serde_json` read the same bytes. Every field has a default, so an older host
and a newer mod, or the reverse, read each other within an ABI version. The per-world shapes,
written 2026-11-09 from the lodger's needs and generalized as a second mod needs them:

- **Observations,** the mod's senses, rich enough that a mod can know the world the way a
  player does without ever asking Java for it:
  - *players:* position, facing, velocity and the movement derived from them over the last
    seconds (turns, look-backs, backing away, stops, sprint bursts), sneaking, sprinting,
    swimming, riding, sleeping,
    health, hunger, air, experience level, held items, armour, active effects, dimension, the
    biome under them, the light level at them, whether they are indoors (sky light), what they
    look at (block or entity, within 8 blocks), what they just did (opened a container with its
    contents, placed or broke a block, slept or were refused, ate, fished, mounted, logged in or
    out, died, respawned, changed dimension, typed a chat line);
  - *the world around each player,* once a second within 64 blocks: time of day, weather, moon
    phase, the biome at the player and the biomes within 64 blocks; every entity within 64
    blocks with its kind, position, name if any, owner if tamed, whether it looks at the player,
    whether it is hostile; and on request (a `Scan` command, answered next tick) a summary of
    blocks in a box: counts by kind, the positions of named kinds (doors, chests, beds, torches,
    ores), light levels, whether a position is in view of a player;
  - *the surroundings,* once an in-game day within 160 blocks at most of a point the mod names
    (the lodger asks 96 around the home): surface water and how wide, lava, snow, sand, trees,
    crops, lights, beds, containers, doors, a bell, and the generated structures here by tag
    (`village`, `mineshaft`, `ruined_portal`, `shipwreck`, `ocean_ruin`), each at its middle.
    Pens with their animals, caves not visited and tunnels dug by players are the todo's.

  The per-tick payload stays small because the world's state comes on request and on a
  schedule, not every tick: a quiet tick carries a player's position and nothing else.
- **Scoring,** for mods that use a model as the lodger does: the SDK has no model in it; the
  lodger's `model/` is the lodger's. What the SDK gives such a mod is the observations above in
  a form `describe` can turn into lines of text, and a worker-thread pattern (`sdk::work`) for
  bringing answers back inside `tick`.
- **Commands:** log a line, scan a box for block kinds, look around a point for the
  surroundings' features, play a sound to a player at a place, send a client hint (a kind, a
  place, a duration; the kinds the client renders are in [Glitches](architecture.md#what-the-client-knows)),
  send a system message, toggle a door, trapdoor, gate, lever, campfire or candle, move a
  block within reach, set a block where there is nothing or over one of the natural blocks the
  command names (`replace`: a path worn into grass, a hole in the ground, never through a
  block with contents), open a chest's lid for a moment, move items between containers, take
  and put items, drop an item that does not despawn, write a sign, cancel or refuse a player's
  sleep, turn an entity's head for a while, turn an armour stand's head or have it follow a
  player, turn an item frame's item, swap a painting for another of its size, name an entity,
  dye a sheep, move an entity (a player two blocks at most, to wake them a block from the bed),
  push an entity (a velocity: a boat drifts, a minecart rolls), turn a block a quarter turn
  (stairs, a banner), put an item in a frame or empty it, dress an entity that wears things (an
  armour stand), wear down a tool in a container, break a block as a player would so its item
  drops (beds, torches, carpets, panes, flowers, signs and lanterns alone; the host refuses the
  rest), show a figure in a player's own skin to that
  player alone (no entity exists on the server: a player-list entry with their profile and a
  player entity as packets to one client, standing or walking a path a point every half second,
  facing them, holding an item, gone after its seconds or when they come within its vanishing
  distance) and take it away, show a second entry with a player's own name in their player
  list for a while, send a line in the game's own words to one player (`joined`, `left`, `says`
  in their own name, `whisper` from themselves, `sleepers` on the action bar), spawn and drive a
  figure in a player's skin (phase 5). Each has a `fits` the host's executor checks before
  running (see [Events re-check](entity.md#events-re-check-before-they-happen)), and an unknown
  command is reported as such rather than dropped.
- **Reports:** per command, `done` or `did not fit` with why.
- **Now:** the tick, the world's time, the real clock, so a mod never reads a clock itself.
- **Integrations:** the host knows which other mods are loaded (`Observations.mods`, the ids
  and versions) and implements a few hint kinds against well-known ones, each behind a presence
  check, so a Rust mod can ask for "dynamic lights off for a minute" and get it where the mod
  exists and nothing where it does not. The list is [Other mods](ideas.md#other-mods); each one
  built is a line in this page.
- **The debug screen** (`sdk/src/debug.rs`, 2026-11-09): the host owns a screen any Rust mod
  gets (the debug key, F8 unless rebound, or `/craftkit <id> debug`), a panel down the right
  third of the screen at three quarters text size that leaves the world in view, with its own Senses and
  Log tabs filled from the host's side (the asking player's movement and state as meters, the
  entities near, the last events sensed, the last commands carried out), and a Mind and an
  Events tab the mod fills through `Mod::debug`: asked a `Snapshot` every half second it
  answers a `DebugReply` of gauges (a label, a value over a max, words), titled sections of
  lines and of items (a row with a quieter line and buttons of its own, ABI 7), every event with its stages, whether it is offered right now, why not, and how often
  it happened, and buttons; asked `Force { event, player }`, `SetStage { stage }` or
  `Action { id }` it does the thing and answers a message. Operators and the owner of a
  singleplayer world alone; the server checks. The same requests are chat commands,
  `/craftkit <id> show|force <event>|stage <n>|action <id>`, for a console. A mod with no debug
  page leaves the default and the host's tabs still show. The config pages of the menu as first
  designed (every key of the mod's config file, written back on change) are still to come and
  will sit on the same screen.

Positions are block positions as three integers, items are ids as strings, players are indices
into the observation's player list. Nothing in the protocol is a Minecraft class.

## What the host does for a mod

The host owns the Minecraft side so a mod never has to: the FFM binding, the per-tick exchange,
the senses (the observations above, the scans and the daily surroundings pass, sliced over ticks
so none of it shows in a tick profile), the saved data (a mod's world bytes as a byte array in
the world's saved data), the executors for every command and their `fits` checks, the client
side (the glitch renderers for hints, sounds, the debug screen, figure renderers in phase 5)
and the packets to carry them, the commands (`/craftkit status`, and each mod's debug commands
as `/craftkit <id> ...`), and the handshake. A
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
