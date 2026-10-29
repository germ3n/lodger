# Notes and todo

What lodger still lacks, which as of 2026-11-08 is everything, in the order to build it, and what
got done. The things to keep in mind while running or changing it are in
[Good to know](good-to-know.md), and how each part works is in the [docs](README.md).

## Where it stands

**2026-11-09, paused.** The Rust side (`sdk/`, `core/`) builds and passes: `cargo test
--workspace` 22 tests, clippy and rustfmt clean, `target/release/liblodger_core.dylib` built.
The host (`host/`), the content jar build (`mod/`) and the CI are written but have not compiled
once: the machine's network was too slow to fetch Gradle's 151 MB distribution or a JDK 25, so
every Gradle run stopped at the download. Nothing is committed yet. To pick up:

1. Get Gradle and a JDK in place: `brew install openjdk@25` (or let `host/`'s foojay resolver
   fetch one), and let `./gradlew` download its distribution, or put `gradle-9.8.2-bin.zip` in
   `~/.gradle/wrapper/dists/gradle-9.8.2-bin/<hash>/` yourself.
2. `cd mod && ./gradlew jar`, then `cd host && ./gradlew build`. Expect the Java to need fixes
   against the real Fabric API, NeoForge and Mojang-mapped names (the networking and the
   `IModFile.findResource` calls were written from memory). `host/common`'s JUnit tests run in
   that build.
3. Make the two phase-0 items `[x]` with the date, then the dev-client item.

## Todo

The phases are in dependency order and each ends with something that runs. The mind comes first,
because it can be built and tested without a game, and because the model layer is mostly a port.

### Phase 0: the toolchain, and making the docs true

- [x] **Confirm the versions.** Done 2026-11-08: Minecraft 26.3 (the current stable; the docs
  had guessed 26.2), Fabric Loader 0.20.1, Java 25 (Mojang's manifest), Fabric API 0.165.2+26.3,
  Loom 1.19, Gradle 9.8.2 and Mojang's mappings, from Fabric's metadata and example mod and
  Mojang's manifest. `VersionsTest` keeps the places agreeing. Was: Minecraft 26.3, Fabric Loader
  0.20.1, Java 25, and the Fabric API and Loom releases for them, from fabricmc.net and the Loom
  template, not the secondary sources the docs were written from.
- [x] **The SDK's shape, and the decision to build it now** (owner, 2026-11-08): craftkit
  (working name), a crate with the `Mod` trait, the C ABI, the protocol and a harness, and a host
  for Fabric and NeoForge; Rust mods as content jars with no Java. See [The SDK](sdk.md). Was:
  the lodger as a Fabric mod with its own Java body.
- [x] **The host nested in each mod's jar, natives for six platforms** (owner, 2026-11-09): one
  lodger jar per loader with the craftkit host inside (Fabric `jars`, NeoForge jar-in-jar), the
  CI building the mind on Linux, macOS and Windows runners for x86_64 and arm64 each. Was: the
  host a jar of its own, one content jar for both loaders, three natives.
- [x] **The SDK crate and the mind build and pass** (2026-11-08): `sdk/` with `abi`, `protocol`,
  `harness` and `export_mod!`, `core/` as a `craftkit::Mod` with its config schema, 22 tests
  across both (`cargo test --workspace`), clippy and rustfmt clean, `liblodger_core.dylib` built
  in release. The docs-and-schema test caught a rename that had turned `react_threshold` into
  `reevent_threshold` in [Config](config.md).
- [x] **The host builds for both loaders and passes its tests** (`host/`). Done 2026-11-09:
  `craftkit-fabric-26.3-0.1.0.jar` and `craftkit-neoforge-26.3-0.1.0.jar`, 18 JUnit tests in
  `common` (manifest, natives, config file, loading failures, versions agreeing). Three things
  the real APIs corrected: 26.3 gates commands with `Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)`,
  not `hasPermission(int)`; Fabric's registry is `PayloadTypeRegistry.clientboundConfiguration()`
  over a plain `FriendlyByteBuf`; NeoForge's `IModFile` gives `JarContents`, not paths, so
  `Platform.ContentJar` now hands the host bytes, which suits both loaders. Was: the MultiLoader
  layout, `Native.java` binding the five phase-0 functions through FFM, `LoadedMod` doing the
  manifest, native, ABI, info and config steps, `/craftkit status`, the handshake on each loader,
  and the JUnit tests in `common`.
- [x] **The lodger's content jar builds** (`mod/`). Done 2026-11-09: `./gradlew jar` runs cargo,
  and `lodger-0.1.0.jar` holds `natives/macos-aarch64/liblodger_core.dylib`, `fabric.mod.json`
  and the `neoforge.mods.toml` with no language loader with the versions filled in, `META-INF/craftkit.json`,
  `pack.mcmeta` and the lang file. Was: cargo run by Gradle, the native under
  `natives/<os>-<arch>/`, the three manifests and `pack.mcmeta` filled in.
- [x] **A dev server starts with both jars on each loader.** Done 2026-11-09: `:fabric:runServer`
  and `:neoforge:runServer` in `host/` with `lodger-0.1.0.jar` in their `mods/` both log `loaded
  lodger 0.1.0 (sdk 0.1.0) from macos-aarch64` and `wrote lodger.json with every default`, and
  `config/craftkit/natives/lodger/macos-aarch64/liblodger_core.dylib` appears. The whole FFI
  path, exercised once before anything rides on it. Found on the way: f32 fractions were written
  back as `0.4000000059604645`, now f64; NeoForge calls `lowcodefml` deprecated, so the content
  jar's `neoforge.mods.toml` names no loader. Still to see by hand: `/craftkit status` in a
  client. Was: a dev client starts with both jars and `/craftkit status` shows the lodger.
- [ ] **The CI runs green:** rustfmt, clippy and `cargo test` on six runners with the natives
  uploaded, the host built for both loaders with its tests, the two lodger jars assembled with
  all six natives and the host nested. Written (`.github/workflows/ci.yml`); the first push proves it. The Rust jobs
  now compile llama.cpp (cmake and a C++ compiler on each runner; `Swatinem/rust-cache` keeps
  the build between runs).
- [ ] **`THIRD_PARTY_NOTICES.txt`** generated and checked in CI (see
  [Development](development.md#third-party-notices)).

### Phase 1: the model layer

- [x] **Port `~/engine/game/base/src/ai/` into `core/src/model/`.** Done 2026-11-09: `mod.rs`
  (the `Model` the `Lodger` owns, in place of the engine's static slot; validation; the scoring
  math), `queue.rs` (the worker and keys, verbatim), `download.rs` (paths and `.part` in place of
  the engine's `Fs`), `infer.rs` (the engine's `infer` module; one change, the context's model
  field is private in `llama-cpp-2` 0.1.158 so the vocab is read through the engine's own model
  pointer). Behind a default-on `model` feature, with `metal` and `vulkan` features for the GPU
  experiment. The first build compiled llama.cpp in about six minutes; the library went from
  0.5 MB to 6 MB. Was: the port, with `crate::fs` replaced by a plain path and the Lua library
  left behind.
- [x] **The `Scorer` trait, the fake scorers and the queue tests.** Done 2026-11-09, the
  engine's seven queue tests and the model's own: answers without blocking, a loading failure
  returned from `start`, superseding by key, order kept, the queue bounded at 64, stop finishing
  the running one and refusing the rest, the scorer dropped on its own thread. 32 tests in
  `core`. The ABI side (`ask` as a command the host sends) is phase 2's, with the world calls.
- [x] **The download, as the host's, with the title screen gated** (owner, 2026-11-09: "lock
  them out of the main menu until the model is downloaded and loaded"). Done 2026-11-09: the
  SDK's `Resource` in `InitReply`, `warm` and `status` in the ABI (version 3), the lodger
  declaring its model; the host's `Downloader` (JDK `HttpClient`, `.part`, magic, SHA-256,
  progress), `Preparation` (fetch the missing required resources one at a time, then `warm`
  each mod, progress for a screen or a log), `PreparingScreen` over the title screen with a bar,
  Quit and Try again, `TitleGate` hooked on both loaders' screen events, and
  `Host.prepareBlocking` on both loaders' server-starting events. Six JUnit tests for the
  downloader and three for resources. Was: the engine's `download.rs` ported into the mind,
  which was then removed, so one downloader serves every mod.
- [x] **The real-model tests,** written 2026-11-09 and `#[ignore]`d with the reason: the
  want question on a player who walked toward the figure, and two questions on one situation
  agreeing with each asked alone, and `warm` then `status` through the ABI. Run 2026-11-09 with
  the downloaded model copied to `core/models/qwen.gguf`: all three pass. A player who walked
  toward the figure makes the model answer `to be looked at`. The tests take turns on one lock,
  since llama.cpp's backend is one per process and three loading at once raced.
- [ ] **Measure.** Time and resident memory for a 300-token situation with four questions of
  four choices on two threads, on a laptop and on a small VPS. Put the numbers in
  [Model](model.md) in place of "tens to a few hundred milliseconds".

### Phase 2: the mind without a model

- [x] **The SDK's world calls.** Done 2026-11-09 (ABI 4): `Mod::World`, `open`, `observe`,
  `tick`, `report`, `save`, `close` in the trait and `export_mod!` (worlds by handle in a map the
  macro keeps, closed at shutdown); `sdk/src/world.rs` with `WorldInfo`, `Now`, `Observations`
  (players with their state and movement, entities, the world's weather, events, chat, scans,
  surroundings, loaded mods), `Command` (log, scan, sound, hint, message, toggle, move and set a
  block, move and drop items, sign, cancel sleep, look at, figure) and `Report`; saves as
  base64 in JSON; the harness opens a world and drives it; a two-line mod's world is opened,
  observed, ticked, reported, saved and reopened from its bytes in the SDK's tests. The host's
  side (the world loop, saved data, the first executors) is the next item.
- [x] **The host's world loop.** Done 2026-11-09: `host/common/.../world/Worlds.java` opens a
  world for each mod when a level loads (`ServerLevelEvents.LOAD` on Fabric, `LevelEvent.Load`
  on NeoForge), with its bytes from saved data (`WorldData`, a `SavedDataType` per mod holding
  the ABI's base64, which the game writes to
  `<world>/dimensions/<namespace>/<dimension>/data/craftkit/<mod>.dat`); each server tick it
  sends the level's players' basic state, ticks, carries out `log` and `system_message`
  (`Executors`, which reports an unknown command as not done rather than dropping it), reports,
  and saves every hundred ticks and on unload; the server's stop closes every world. Seen on a
  plain Fabric 26.3 server with the bundled jar: three worlds opened with their saved bytes
  from the previous run, saved again five seconds in, no errors. `NativeRealTest` drives the
  real library through the FFM binding, with the model when `core/models/qwen.gguf` is there.
  Two bugs the server run found: the host read every answer with an `error` key as a failure,
  and a `Status` always has one (empty when nothing is wrong), so a loaded model looked unready
  and the title screen would have stayed gated (`LoadedMod.parseStatus`, with a test); and a
  dropped `Model` left its worker and llama.cpp's backend alive, so a second load in one process
  failed with `BackendAlreadyInitialized` (`impl Drop for Model`, with a real-model test). The
  senses proper and the other executors are phase 3.
- [ ] **The senses, in full** (owner, 2026-11-09: "a good harness that can get the current
  biome, mobs around, etc."): everything under Observations in [The SDK](sdk.md#the-protocol),
  on the host side as `senses/` with one collector per kind (players, entities within 64,
  world state, the `Scan` command, the daily surroundings pass), each sliced over ticks and
  measured, and on the SDK side as typed structs with a `describe` that turns any of them into
  the plain lines a model can read. The harness gets a `World` builder so a test can say
  "a plains biome, a lake 20 blocks north, three cows, night, raining" in a line.
- [x] **The menu, its debug half** (owner, 2026-11-09: "a ui we can pull up with command/key
  that lets us spawn events", "meters showing different senses, gauges, full debug suite").
  Done 2026-11-09 as the debug screen: `sdk/src/debug.rs` (`DebugRequest`: snapshot, force,
  set_stage, action; `DebugReply`: message, gauges, sections, events with offered/why/times,
  actions), `Mod::debug` with a default, `craftkit_world_debug` (ABI 5), the harness's
  `World::debug`; the host's `DebugAskPayload`/`DebugTellPayload`, `Worlds.debug` (the host's
  panel from `Senses.panel` plus the mod's reply), `Host.mayDebug` (operators, or the owner of
  a singleplayer world), `/craftkit <id> debug|show|force|stage|action`; the client's
  `DebugScreen` (Senses, Mind, Events, Log tabs; meters as bars; a filter; a Fire button per
  event; the mod's buttons; a half-second refresh) and `DebugModel` (plain Java, tested) with
  the F8 key on both loaders; the lodger's `core/src/debug.rs` (gauges for stage, days in
  stage, budget, plan, decisions, memory, kills; sections for the situation, the plan, done
  lately, known blocks, creatures, never forgotten, the decision log; every catalogue row with
  the first unmet need or the refusing rule; six actions) and `Brain::force` (a forced plan
  runs on the next tick past needs, view and rules and costs no budget). Tests: the SDK's
  round trip through the ABI, the lodger's snapshot/force/stage/actions, `DebugModelTest`.
  Was: `Menu` in the protocol (pages, fields, actions), the host rendering it as `/craftkit
  <id> ...` commands and as a client screen, operator-only, with the config write-back;
  `MenuAction` commands back to the mod. The lodger's three pages (status, config, debug).
  Still to do: the config page (every key of the mod's config file, written back on change) on
  the same screen.
- [x] **Presets** (owner, 2026-11-09: "3-4 presets, like a calm one, then aggressive; start off
  balanced"). Done 2026-11-09: `preset` in the config, `calm`, `balanced`, `intense`,
  `relentless` as `Preset` in `config.rs` with the table in [Config](config.md#presets), the
  preset laid under the file's own keys, every preset passing its own checks, `balanced` equal
  to the defaults.
- [x] **The license before the download** (owner, 2026-11-09). Done 2026-11-09: `Resource.license`
  in the SDK (name, URL, text), the lodger's model carrying Qwen's Apache-2.0 notice, the host's
  `LicenseScreen` before the first download of each resource with Accept and Quit, acceptance
  remembered in `config/craftkit/accepted.json`; a dedicated server logs the terms and waits for
  `accepted` in `config/craftkit.json` before it downloads.
- [x] **Three model sizes** (owner, 2026-11-09). Done 2026-11-09: `model.size` of `0.5b`,
  `1.5b` or `3b`, each Qwen's own `q4_k_m` file with its URL, file name and size for the bar,
  `model.url` and `model.path` empty by default and derived from the size; the table in
  [Which model](model.md#which-model). The default moved to `1.5b` the same evening (owner:
  "make the 1.5b model the default"); the dev client's config keeps `0.5b` so no download
  starts there.
- [x] **Seeded picking** (owner, 2026-11-09: "on startup the model would give the same answers
  to the same questions"). Done 2026-11-09: the pick is a draw weighted by
  `softmax(scores / brain.temperature)` from a SplitMix64 seeded per start (the clock and the
  game directory), `0` restoring the engine's always-the-best; the confidence stays the plain
  probability; `Model::reseed` for the world's seed in phase 2; the seed logged with decisions
  (phase 2's log). Tests: temperature zero picks the best, a warm draw follows the scores to
  within a few percent over two thousand draws, the same seed repeats and another differs.
- [x] **`Stay`, the lodger's `World`.** Done 2026-11-09: `core/src/stay.rs`, `bincode` behind a
  format byte, refusing a newer format or another world's save by name, counting ticks,
  players and reports; the model reseeded with the world's seed on open; tests through the
  ABI and direct. Format 3 (the same evening) is JSON with `#[serde(default)]` on every
  struct, so a field added to the memory reads an older save without a migration; formats 1
  and 2 open with their counts and a fresh mind.
- [x] **Memory, first cut.** Done 2026-11-09: `core/src/memory.rs` with the target (the first
  player seen), the home (the bed slept in, or the densest cell they stand still in; doors and
  lights from blocks placed; the floor), the containers opened with their contents and slots,
  the favourites (what is in their hand), the held item, what was done and how it went, the
  target's last state (position, facing, indoors, asleep, walking, creatures near, look-backs),
  chat lines about the lodger, and the placed blocks, each capped with its reason; `in_view`
  and `target_away_from`; `describe` as a handful of template lines. Six tests. Was: every piece
  of [What is kept](memory.md#what-is-kept) with its bound and the size test. Still to do: the
  routes, the hours, the reactions, the figures seen, the surroundings and activity pieces, the
  temper and the chains, and the 256 KiB test.
- [x] **Routes, hours and reactions** (the memory's next pieces). Done 2026-11-09: `Route`
  (a trail of a point every two blocks, cut at a minute's stop, sleep or leaving; kept when
  ten points or more; counted into the route it repeats), `Session` (joined and left in real
  time, the game time at joining; the usual minutes and whether they join after dark),
  `Reaction` (lit, walled, watched, left, ignored, read off what the target does in the day
  after a landmark); `Memory::clock` fed once a tick by the stay; a line of the situation text
  for each; `Need::Route`, `Where::RouteEnd`, `animal_gone`, and `light_path` toward the
  route's end. Tests: a walk repeated becomes a route with its far end known; sessions give
  the usual length; a torch placed after a take reads as `lit`, a day of nothing as `ignored`.
- [x] **The catalogue, third cut, and the group switches.** Done 2026-11-09: twenty-four more
  rows (stairs and banners turned, frames emptied and swapped, the stand dressed, the tool worn
  down, the cold furnace, the chest opening as the target walks up, sorting by colour, a flower
  planted, a crop replanted, an ice path, the occupied bed, waking a block over, the pet
  outside, the player-list entry that stays, the note tune, the piston, the phantom disc, the
  fog closing in, snow and rain indoors, the ore scan and the torch on the vein), over four
  more commands (`rotate_block`, `set_frame_item`, `equip_entity`, `damage_item`) and a player
  moved two blocks at most; `events.sounds`, `events.glitches` and `events.chat` now gate their
  groups in the offer and in forcing (they were read and unused); `Need::InChestAny`,
  `TargetApproachingChest`, `ColouredStacks`, `ToolInContainer`. 126 rows coded.
- [x] **Scripted stays, the other six.** Done 2026-11-09: no event inside its cooldown or past
  its cap over fourteen days; a disabled event never happens and a group switched off stays
  quiet; sleep is only refused with the player in bed and house sounds never play for a player
  at the mine; calm acts less than relentless; the stay is over after stage five and nothing
  happens after; the memory learns the house, the chest, the favourite and the route from the
  script (which now walks to the mine, 77 blocks over 200 ticks, so a route forms) and the save
  stays under 256 KiB. Ten scripted stays in all.
- [ ] **The offer** (`brain/offer.rs`): the six filters of
  [What is offered when](entity.md#what-is-offered-when), with a test per filter (no water, no
  boats; asleep, no `late_steps`; stage one, nothing from stage two) and a test that a hundred
  draws from one situation offer every eligible event at least once.
- [ ] **`Memory::describe`, in full.** The twelve lines of [Describing](memory.md#describing)
  with tests on the exact wording, and the truncation from the bottom at a measured token
  count (the model's tokenizer; without a model, words times 1.4). The first cut writes eight
  of the lines.
- [x] **The catalogue, the second cut** (owner, 2026-11-09: "flesh out the events"). Done
  2026-11-09: 87 rows in `events.rs` over the host's commands, with needs that carry data
  (`Fixture("fence_gate")`, `Decor("painting")`, `Creature("sheep")`, `InChest("sapling")`,
  `Fixtures(kind, n)`) and the first unmet one named for the debug screen: furnaces (`cook_one`,
  `take_output`, `item_on_table`, `anvil_sound`, `page_turn`), the tidy and the two-chest
  events, the house (`window_tap`, `carpet_shift`, `hole`, `cairn`, `sapling`, `sign`,
  `path_worn`), lights and fire (`light_path`, the campfire, the candles, the cauldron), rest
  (`wake`, `egg_on_bed`), decoration (`frame_rotate`, `frames_turn`, `painting_change`,
  `armor_stand_turn`, `stand_watch`, `named_animal`), the farm (`open_gate`, `sheep_dyed`,
  `pet_growl`, `pet_moved`, `animal_indoors`, `villager_at_door`, `golem_turned`), chat and the
  player list (`tab_entry`, `joined`, `left`, `sleepers_wrong`, `says_back`, `says_first`,
  `whisper_from_you`, `repeats_you`), the bell, the note block, the lever, eight more sounds,
  and eight more glitches the client now renders (`client/Glitches.java`). The memory grew
  fixtures, decor, creatures by kind, container kinds, weather and the last chat line for
  them. Tests: every event makes at least one command from a full memory, the tidy algorithm,
  cooking, the kill sign and grave, the worn path and the hole replacing natural ground only,
  frames, stands and the pet. Still to do: the rows that need the figure (phase 5), the
  surroundings (water, fields, villages, mines), routes, redstone, and the endings.
- [x] **What they kill** (owner, 2026-11-09: "keep track of mobs hit/killed and do something with
  that"). Done 2026-11-09: the `killed` event from both loaders (the damage source's attacker),
  `Memory.tally` by mob kind (hits, kills), `most_killed` at three kills, a line of the
  situation text, and `kill_sign`, `grave` and `kill_line` over `brain/text.rs`'s `KillSign`,
  `GraveSign` and `KillLine` banks.
- [x] **Never forgotten** (owner, 2026-11-09: "certain actions are always remembered, where it
  makes sense"). Done 2026-11-09: `Memory.landmarks` (every bold event that landed, kept for
  the stay; the caps per stay count from it when `done` has trimmed) and `Memory.milestones`
  (where they died, the first sleep, the first kill of each kind, a dimension crossed), each
  with a line in `describe` and a section on the debug screen.
- [x] **Text by kind** (owner, 2026-11-09: "it should generate, with examples correlating to
  specific actions"). Done 2026-11-09, the bank half: `brain/text.rs` with a `Kind` per place
  words go (sign word, chat line, kill line, kill sign, whisper, animal name, grave sign), each
  with its examples and its most words, `fill` for `{kind}` and `{n}`; `brain.text` defaults to
  `generate`, which until phase 4 picks from the bank like `bank` does. Phase 4's item below
  wires the model in with these banks as its examples.
- [x] **Events as data, the rules, the arc, the rule brain, the log.** Done 2026-11-09:
  `events.rs` with 22 rows the host's commands can carry out today (containers, the door, the
  lights, rest, animals, the sounds, three glitches), each with its stages, needs, cooldown,
  cap, weight, action and menu words, and a test that every id is a row in the docs' catalogue;
  `rules.rs` (in view within the distance, disabled, the same as last, cooldown, cap, not
  offered) with tests; `arc.rs` (move-in after the configured days, stage conditions and most
  days, the day's budget) with tests; `brain/offer.rs` (needs met, the rules, a weighted draw
  favouring what has not happened, `wait` first) with the hundred-draws test; the rule brain's
  weights (wait more early, small before noticed); `brain/act.rs` turning an event into
  commands at a place from the memory, including a chest's close thirty ticks after its open;
  `brain/log.rs` writing `config/lodger/decisions.log`; the stay's save format 2, opening
  format 1 by carrying its counts over. Was: `events.rs`, `rules.rs`, `arc.rs` as the docs have
  them, the rule brain, the decision log. Still to do: the rest of the catalogue as commands
  arrive, the offer's time-and-weather filter beyond night, the model asking (phase 4).
- [x] **Scripted stays, first four.** Done 2026-11-09: `core/tests/stays.rs` drives a scripted
  player (a house, a bed, a chest opened each morning, a door and torches, a day at the mine)
  through the ABI for days on the rule brain: a twelve-day stay acts within its budget and never
  touches the house while the player is home and awake; nothing happens before move-in and
  stage one never takes or refuses sleep; a saved stay reopens where it was; no event lands
  twice in a row. Twelve days run in about five seconds, with the clock moving five ticks at a
  time. Was: the `Player` script and the first ten tests. Still to do: the other six.
- [x] **Design decisions taken with the owner (2026-11-08):** Rust mind and Java body through
  FFM; `lodger` as the name; the one death on hard kept as the only harm; the denser pace
  (budgets 1, 2, 4, 6 a day, a stay of about two in-game weeks with forty to sixty events) over
  the rarer one (half that over three weeks).

### Phase 3: the body, stages one and two

- [x] **Senses, the host's side.** Done 2026-11-09: `senses/Senses.java` sends every player's
  state each tick (position, facing, velocity, sneaking, sprinting, swimming, sleeping, riding,
  health, hunger, air, level, hands, armour, effects, dimension, biome, light and sky light, what
  they look at within 8 blocks, and the movement reading from `MovementTracker`: pace, stops,
  turns, look-backs, backing away, sprint bursts, standing still), every entity within 64
  blocks once a second (kind, position, name, owner, hostile, whether it faces a player), the
  weather once a second, and the events both loaders' hooks feed: joined, left, used a block
  (a container's contents when it is one), placed (a Fabric mixin; NeoForge's event), broke,
  sleep refused, died, respawned, changed dimension, hit, chat; sleep begun and ended, mounted
  and dismounted and a container closed are read off state changes with no event needed. Six
  JUnit tests for the movement reading. Was: position and facing once a second, containers
  opened with contents, blocks placed and broken near home, sleep and refused sleep, logins and
  logouts, dimension changes, deaths, and what the target is doing read from those. Still to do
  on the mind's side: the activity read from these (mining, farming, building, fishing); the
  moon phase and time of day from 26.3's world clocks (the senses send 0 and game time's
  remainder for now).
- [x] **The surroundings pass, first cut.** Done 2026-11-09: the `surroundings` command starts
  a pass around a point within a radius (at most 160), 64 columns a tick, a column every four
  blocks read at the surface: water, lava, sand, snow, trees, crops, lights, beds, containers,
  doors, each as a feature with a center and a size. Answered as an observation when done.
  Was: once an in-game day within 128 blocks of the home, in slices. Still to do: villages
  (the bell's point of interest), the target's tunnels and unentered caves, pens with their
  animals, and the mind asking for it once a day.
- [x] **The general commands' executors, first cut.** Done 2026-11-09, in `world/Executors.java`:
  `log`, `system_message`, `play_sound` (a sound packet to one player), `hint` (a payload to one
  client, kept in the client's `Hints` list and logged; the renderers are phase 5), `scan` (a
  box of at most 40,000 blocks, counts and positions), `surroundings`, `toggle_block` (doors by
  their own method, levers by theirs, anything with an `open` property by cycling it),
  `move_block` (not a block with contents, only into air), `set_block`, `move_item` (between
  two containers' slots, stacking when the items match), `drop_item` (an item entity that never
  despawns), `set_sign` (the front text), `cancel_sleep` (wake with the vanilla message, or
  refuse the next attempt within the given seconds through both loaders' sleep events),
  `look_at` (a mob's look held for the given seconds by `Looks`). Each reports `done` or why
  not; an unknown command is reported, not dropped. Was: executors for the stage one and two
  events, about fifty, with game tests. Still to do: the game tests (the Fabric gametest setup),
  and the lodger's events themselves, which are the mind's rows over these commands.
- [ ] **Game tests for the executors** (`./gradlew :fabric:runGametest`): one per command
  against a built test world, including `fits` refusing when the target is in view, which is
  the mind's rule but the host's check. Containers first (`open_chest`,
  `shuffle_chest`, `take`, `return_taken`, `place_taken`), then furnaces, doors, lights, sounds,
  then the rest. The lodger's own torch and sign blocks, vanilla-looking.
- [ ] **Budgets, cooldowns, caps and the menu** (`events.rs`, `brain/offer.rs`) with scripted-stay
  tests: a stay of fourteen days on the rule brain holds between forty and seventy events, no
  event twice in a row, none past its cap, none inside its cooldown.
- [x] **The net, first cut.** Done 2026-11-09: sounds go as vanilla's own sound packet to one
  player; `HintPayload` (kind, optional place, seconds, detail) is registered on both loaders
  and lands in the client's `Hints` list, timed and logged. Was: the sound packet, the hint
  packet, the target flag; the client handlers. Still to do: the target flag, the footstep
  sound by block for `late_steps`, and the first two glitch renderers.
- [ ] **Saved data:** `lodger.dat` holding the mind's bytes, saved with the world, loaded on
  world load, the mind closed on unload.
- [ ] **`/lodger status`, `/lodger decisions`, `/lodger reset`, `/lodger leave`.**
- [ ] **Play it.** A fresh world, dev config, through stages one and two on the rule brain. Write
  down everything that read wrong in [Good to know](good-to-know.md) or here.
- [x] **The first play session's fixes** (owner at the keyboard, 2026-11-09, the 1.5B deciding).
  Done 2026-11-09:
  - **Written lines are words:** a sign said `123456`. The prompt ended on `- ` (a trailing
    space leaves Qwen only the tokens without one, digits the likeliest) and the mask kept
    lower case alone; now it ends on `-`, the mask takes either case, digits only for the two
    counting kinds (`Kind::leash`), the decode stops before a word past the kind's, and the
    check refuses digits outside those kinds and a line with no letter. A real-model test
    writes each kind from the session's situation. Was: letters, digits and spaces for every
    kind, lower case only, a prompt ending in a space.
  - **Whose chest:** a village chest opened once was "the chest opened most" (owner: placed, or
    opened often). A container is the target's when they placed it or opened it on three
    different days; only those are used or described. Was: every container opened.
  - **What they took is known:** the host reads a container again on closing
    (`closed_container` with its position and contents). Was: contents read on opening only,
    so `shuffle_chest` failed on the bread they had taken.
  - **Doors counted once:** two doors dug under and put back were four. One placed where one
    is remembered replaces it; a block broken forgets what stood on it. Was: every placement
    appended, and only the broken block itself forgotten.
  - **Blocks stand where a player could put them:** torches floated over a slope. `set_block`
    refuses a block that cannot survive, and with `settle` and `wall` the host looks down and
    up for ground, then a wall (owner: "placed on a wall, just not floating"); `move_block` the
    same. ABI 6. Was: set wherever the mind said.
  - **Out of sight, farther:** a path of torches went up 40 blocks off in view, and an event
    with no spot of its own was never checked. Every block an event changes is checked at its
    moment, a plan in view waits instead of being dropped, and `events.min_distance` is 64
    (`calm` 80, `intense` 48, `relentless` 32; owner: "far, but in sight"). Was: the event's
    own spot within 24, dropped when in view.
  - **Walled means closed in** (owner: enclosed only): building a house around a sign read as
    walling it. Now all four sides closed by placed blocks, the last one placed closing it.
    Was: three blocks placed within 4.
  - **A moment that fits:** `wall_swap` (away or asleep) planned "as they come home" waited
    its whole day. `when_fits` drops a `When?` the event's needs rule out. Was: kept, and
    never met.
  - **The words of the situation:** `6 sheeps`, `a dirt`, `it light a path`, `1 days ago`,
    `the player's an iron pickaxe`. Was: a plain `s`, an article on everything, the menu's
    words after `it`.
  - **A figure's spot on the ground reads as one:** `far_figure at (440,-10000,-32)` was the
    protocol's "on the ground in this column" printed raw; the log and the debug pages say
    `(440,ground,-32)` (`log::spot_words`), and the host stands a figure under the trees, not
    on them (the heightmap without leaves). Was: the sentinel printed, the leaves counted.
  - **Never digging near what they built:** `hole` dug at the lodger's place after the target
    had built up to it, and the two blocks under a crafting table went (the host refused the
    table itself). A command that takes solid ground out within 8 blocks of anything placed
    refuses the whole event, Fire included, and `hole` looks for a clear spot 12 to 20 blocks
    out or is not offered. Was: the catalogue's promise, unchecked.
  - **A plan survives the game restarting:** reopened, a plan due in a minute said 1956
    seconds, since the host's tick count starts again at 0 and the brain's ticks were the old
    session's. The stay keeps the last tick (`Stay::last_tick`) and moves every waiting tick
    back when the clock comes back lower (`Brain::rebase`); a save from before it takes the
    moment from the next planning time. Was: the brain stalled until the count caught up.
  - **The kills line names every kind:** a pig killed seemed not counted, since `show` named
    the most-killed kind alone (`11 sheeps killed, 18 hit; 9 kinds`). Was: the first kind.
  - **It answers chat, sometimes** (owner: "stage 2 and up, often if about it", "may echo a
    word"): `answers_you`, started by the target's line, not the menu; three a day, five
    minutes apart, 60 or 15 in a hundred; the model writes it with their cleaned words in the
    prompt, the new `Reply` bank behind it; exempt from "never twice in a row". Was:
    `says_back` alone, stage three, chosen from the menu whenever, so rarely near a line.
  - **Plans side by side** (owner: "one event at a time kinda gets slow"; a config key, 3;
    ask again sooner; no shared spot, group or event): `brain.most_plans` (calm 2, intense 4,
    relentless 5), each plan run on its own, the budget counting plans held, plans cancelled
    with a reason when they cannot happen (a lasting need or the thing at their spot gone, a
    rule refusing them outright, a reaction at the cap), a save's single plan carried over.
    Was: one plan, and no new decision until it was spent.
  - **The debug panel** (owner: "kinda hard to use ingame, takes up whole screen"; a right-side
    panel): the right third of the screen, three quarters text size, see-through, no blur, not
    paused, wrapped lines, the Mind tab first; plans as rows with Drop and Now (a debug
    section's `items`, ABI 7). Was: the whole screen, darkened, lines cut off.
  - **The monsters stare** (owner: "zombies and skeletons that just stare at you for a night";
    the ones already there, never spawned): `hostiles_stare`, stage three and up, at night with
    three or more hostiles within 48 blocks, holds up to sixteen still and facing the target until
    dawn (`hold`, ABI 8: AI off, turned each tick, a tag and a sweep so none is ever saved
    frozen, released on unload). The Rust is tested; the host's `Holds` compiled and seen in a
    client next session. Was: `stare`, animals and villagers alone.
  - **The second session's silent failures** (the review's B2, B3, B5, B8): `woke_elsewhere`
    moved the player as a made-up entity id and did nothing (now `move_player`, ABI 9);
    `wolf_far`'s sound was gone in 26.3 (now `wolf_sad.whine`, and every sound checked against
    the game's list); `stare` jittered (now `hold`). Was: each failed in the game's log alone.
  - **What it builds lands where it should** (the review's B1, B4): signs, cairns, graves and
    saplings settle to the ground (`rest`, a wall sign when no ground), the sign's text finds
    it where it landed, ice goes only on water, and a failed block drops the rest of its event;
    the sapling is planted before it is taken. ABI 10. Was: placed at the home's floor height
    wherever that fell, the text sent after a sign that was not there, a sapling taken and
    then lost if the planting failed.
  - **Every event's words are a deed** (the review's B10): the eighteen sound events worded as
    noun phrases ("a wolf howling far off") read as "Today it could not a wolf howling far off";
    they are verbs now ("make a wolf whine far off"), and a test keeps every row so. Was: a
    noun phrase after "it did" and "it could not".
  - **The panel explains and goes places** (the review's E1, E2; owner: "a tooltip ... that
    shows description", "hover over coordinates and teleport there"): an event's tooltip is its
    catalogue row (read from `docs/entity.md` at build time), its cooldown and cap and its last
    outcome; a line with a spot in it teleports you there on a click. ABI 11. Was: an id and a
    reason, and coordinates to type by hand.
  - **Lines vary and never repeat** (the review's B11, C1, C4; owner: "the kill_line stuff
    repeats"): Fire asks the model too (it went to the bank alone, round its six lines), the
    model draws among likely words at `brain.text_temperature` 0.8, and no line is used twice
    in a stay. Was: the likeliest word every time, and Fire from the bank in order.
  - **The house they sleep in is the home's** (the review's B6, B7; owner: "the home's, placed
    or not"): the daily surroundings pass reads every block within 8 of the bed, and its doors,
    lights, chests and bed join the home whoever placed them; a chest there is theirs
    (`Container::in_home`). Was: only what the target placed, so a village house had "0 doors
    and 0 lights" and generated torches did not count.
  - **A route on the first day** (the review's B9): the longest single walk of 32 blocks or
    more is the route until one is walked twice. Was: "no route" until the same way was walked
    twice, so `behind_you` and `animal_gone` waited.
  - **The sounds, reviewed** (batch 1 of the review; owner: any house, theirs first; match the
    block; soft knocks early, the heavy one later; three to five ways each): every sound event
    plays one of its ways, never the same twice running; footsteps and placing are the real
    block's sound (`block_sound`, ABI 12); the house's sounds come from a real door or chest of
    the house they are in (a village house by the mind's own scan, `Memory::here`), theirs
    first; `knock` became soft knocks for stages two and three and `knock_heavy` the pounding
    for four and five. The twelve-day stay counts events landed against the budget, not
    commands. Was: one fixed sound each, stone footsteps, a zombie bashing for a knock.
  - **A held mob hit twice wakes** (owner: "a mix of" breaking the spell and staying frozen):
    the first hit it takes unmoved; the second frees that mob alone. Was: frozen through any
    number of hits.
  - **Chests, part one** (batch 2; owner: home only, all four missing, four new): what `take`
    takes the host keeps itself, saved with the world, and gives back components and all
    (ABI 13: `keep`, `kept`; a taken enchanted pickaxe came back plain); `move_item` moves what
    is there into the slot asked or the first that fits; new `renamed_item` (the model writes
    the name, the `ItemName` bank), `returned_used` (worn a little, or named) and
    `chest_answers` (another chest closes as they open theirs, three times in ten). Was: a new
    item made from the id, moves failing on a count or a slot, none of the three.
  - **Chests, part two:** `book_word` (their book and quill signed with one word, by no one;
    the model writes the word) and `map_marked` (a red X at its place on their map), with
    `sign_book` and `mark_map` (ABI 14). Was: in the catalogue alone.
  - **Chests, part three:** `hopper_emptied` (what waited in their hopper is in the chest it
    feeds) and `shulker_moved` (their shulker box across the room, everything still in it);
    hoppers and shulker boxes are fixtures; `move_block` carries a single-block container's
    contents and searches `around` for room (ABI 15). Was: in the catalogue alone.
  - **Chests, part four:** what it digs it holds (`Memory::stores`, from the host's "replaced"
    reports, grass dug as dirt), and `chest_full_of_one` fills a chest's empty slots with one
    dirt each, never more than it holds, spending the store. Was: no count of what it dug,
    so "everything it places it dug" could not be kept.
  - **Eight more sounds** (owner: "add more sound events if you can think of any"):
    `steps_above`, `scratching` (in the wall beside them), `ladder_climb` (rising behind them),
    `pickup_pop`, `cold_crackle` (their cold furnace), `breath_close` (at their ear),
    `cave_air` (a cave's sound in the open) and `door_scratch`, each three or four ways, with
    the new movements above, beside, rising and at the ear. Was: fourteen sound events.
  - **The home read, often** (batch 3; owner: "why only read the house once a day?"): the
    house box (17 by 9 by 17 around the home) is scanned when they come home, half a minute
    after they build or break near it, and every ten minutes while there; inside the box, what
    it finds is the home's (doors, lights, chests, the bed, windows, stairs, carpets, ladders,
    pots, work blocks) and what it does not find is gone; `scan` takes `*` suffix kinds; door
    and window events take the house they are in first. Was: once a day, adding only.
  - **The other reads** (the review's J1; owner: "other reads/scans should be done more
    frequently too"): the village is read every five minutes while they are near it, and its
    scan and the ore's are kept apart; the entity pass (every second) and the surroundings
    pass (daily, the house read on its own) were already right. Was: the village read once,
    and a mining trip's ore scan wiped it.
  - **Something of its own in the loot** (owner: "hook into loot generation"; its own things
    and a short list, mostly add, one in five from stage two): `loot_gift`, armed by the brain
    (what it took of theirs first, their head from stage four now and then, something it dug,
    else a redstone lamp or a candle) and given by the host as fresh loot is unpacked, one item
    added or swapped (the swapped one kept); the host's `loot_touched` report clears what it
    gave. ABI 16. The one exception to "it never spawns anything". Was: nothing in loot.
  - **No ask for a menu of one:** the model was asked every interval with `wait` the only
    choice. Was: asked, and logged.

### Phase 4: the model in the loop

- [x] **Wire the brain to the worker.** Done 2026-11-09: the two planning questions go to the
  worker with the key `plan` and the answer is read on a later tick; `brain.event_threshold`
  and `nothing yet` make `wait`; a refusal, a minute's silence or `brain.force_rules` hands the
  menu to the rule brain; the story and the log say which brain decided. Tests with a fake
  scorer: the answer names the event, `nothing yet` waits, a shut-down model and
  `force_rules` fall back. Was: the asks with their keys, the callbacks in the tick, the
  thresholds, the fall-back. Still to do: the `react` asks on triggers, `When?` and `Where?`.
- [x] **The want and the target asks,** once each at move-in. Done 2026-11-09: `want` asked
  the moment the target is known (the seed's pick without a model), kept in the memory, read
  in the situation text and leaning the rule brain's weights; `witness` asked when
  `multiplayer.pick` is `model` and two players are seen, the first seen after five minutes
  alone or without a model. `When?` and `Where?` the same evening: the plan waits for the
  model's moment (`when_holds`), the model's place stands for the events that leave or make
  something at a spot. The `react` asks on the memory's triggers (slept, opened the chest, died,
  lit, walled, watched, left): the mood kept, a small event now, the far figure or the thing
  given back above `brain.react_threshold`. The `bank` mode's `What does the sign say?` ask.
  Tests with the fake scorer for each. The house-changing events the owner asked for the same
  night (`wall_swap`, `torch_blue`, `window_bars`, `bed_breaks`; `break_block` and
  `memory::family`, every wood and colour as data, not a table).
- [x] **Calibration, round one, first pass.** Done 2026-11-09: `core/tests/calibration.rs`
  (ignored; three scripted days against the real 1.5B with every decision printed) and
  [round one](calibration/round-1.md): seventeen decisions all by the model, thirteen landed,
  the want and the moods read; what to move in round two (the first planning question barely
  discriminates, low-confidence picks still happen, moods at low confidence are noise, no
  written line was exercised). Was: ten stays through stage two by different players, the log
  read end to end, the prompts and thresholds moved. Still to do: round two as the page says,
  on the owner's dev stays with the real model.
- [x] **`brain.text = generate`, the model half** (owner, 2026-11-09). Done 2026-11-09:
  `Scorer::generate` and `Model::generate_async` beside scoring (a writing job on the same
  worker, key `text`), `infer.rs`'s greedy decode under a mask of lower-case letters, digits
  and spaces (built once from the vocabulary), stopping where the model's own best token is a
  stop; the prompt from the situation and the kind's examples (`situation::writing_prompt`);
  `text::check` (lower case, the kind's word count, a 14-letter word cap, the `BLOCKED` list of
  slurs, sexual words and self-harm, which the owner trimmed from a wider list the same night);
  the line asked for when the event is planned and used when it runs, the bank otherwise; a
  `wrote` line in the decision log. Tests with a fake writer: a line on the sign, a bad line
  dropped. Was: the generate function, the prompt, the blocklist, the log line, the fallback,
  the `What does the sign say?` ask. Still to do: that ask for `bank`; calibration round two on
  the real model's lines.
- [ ] **Chat and the player list** (owner, 2026-11-09): the chat sense (the target's lines and
  witnesses' within 32 blocks, matched for lines about the lodger, eight kept) and the events
  of [Chat and the player list](entity.md#chat-and-the-player-list), as host commands: a
  player-list entry to one client, a system or chat line to one client in a given name, an
  advancement toast. All target-only, all behind `events.chat`.
- [ ] **Other mods** (owner, 2026-11-09): `Observations.mods`, the presence checks, and the
  first three hooks (dynamic lights, a minimap marker, Presence Footsteps' sound set), each
  tested with and without the mod in the dev client. The rest of [Other mods](ideas.md#other-mods)
  as people ask.
- [ ] **Chat template or not.** Score the real-model test set with Qwen's chat template around
  the situation and without, and keep whichever separates the cases more. Write the result into
  [Brain](brain.md#the-situation-text).
- [ ] **Qwen3-0.6B.** Same test set, same question. Both are Apache-2.0; if Qwen3 reads better
  and is not slower, change the default URL and [Model](model.md#which-model).

### Phase 5: the figures, stages three and four

- [x] **The figure, first cut** (2026-11-09). `Figures.java`: a figure is packets to the
  target's client alone, a player-list entry with the target's own profile (listed false, the
  name shown or not) and a player entity with an id above the server's, facing the target,
  holding the favourite, re-sent a point every half second along a path, taken back after its
  seconds, when the target comes within its vanishing distance, on `remove_figure`, or when
  they leave; the player entity type looked up by id. The mind's `Action::Figure(where,
  seconds, vanish)` with eight rows: `far_figure` (fifty to seventy blocks off, away from the
  home, on the ground), `behind_you` (the route's far end), `on_the_roof` (at dawn), `across_water`,
  `in_doorway` (0.4 s), `at_the_chest`, `in_your_bed`, `replay` (walking the route); figures are
  exempt from the in-view rule, since being seen is the point; `events.figure` gates them. Not
  yet seen in a client. Still to do: a pose for the one in the bed (it stands), the far
  figure's swing and crouch, the figure stepping behind cover when approached rather than
  vanishing, witnesses never seeing it (true by construction today).
- [ ] **The far presence** (`far_figure`): a server entity in the target's skin with the name
  shown, placed by line of sight from the route or home, gone when approached. The client
  renderer, the skin from the target's profile. Witnesses' clients never receive it.
- [ ] **The replay figure** (`replay`): walks a recorded route at the recorded pace, rounds a
  corner out of sight when approached, leaves the held item. The route's points need a path the
  figure can walk; where the world changed, cut the route there.
- [ ] **Executors for the stage three and four events** of the catalogue, about sixty: the beds,
  the house's block moves, the villages, the water, the mines, the redstone, the animals, the
  late figures, with game tests, and the new general commands they need (dig a line at a
  player's pace, drive an animal to a spot, set a villager's trade, mark a map).
- [ ] **Like a player** (owner, 2026-11-09): `mining_sounds`, `mine_beside`, `tunnel_toward`,
  `tools_in_hand`, `mined_out`, from [Like a player](entity.md#like-a-player). The digging
  executor is one general command, "dig this line of natural blocks at a player's pace with
  these sounds", with the rule checks (nothing placed within 8 blocks, no cave, water or lava
  ahead) in `fits` and re-checked per block, since the target moves while it digs. The memory
  learns the target's tunnel sizes and torch spacing (the senses already see blocks broken and
  placed). `tunnel_toward` is the showpiece: it needs the target's current digging direction and
  a slow approach over minutes, so it is a plan the mind re-checks every few seconds, not one
  command.
- [ ] **The glitches** of [The sky and the light](entity.md#the-sky-and-the-light) and
  [The screen](entity.md#the-screen): the hint packet with a kind, a place and a duration, and a
  client renderer for each: light-level overrides for a room, a painting drawn rotated, the sky's
  sun, moon and stars and colour for a moment, rain and snow particles and sky tint, the
  compass's needle, footprint decals on snow and sand, a reflection drawn wrong, and the HUD's
  hotbar, hunger, hearts, subtitle, crosshair, player list and sleep count, the camera's nudge
  and the field of view. Each ends on its own; none touches the server's world. Tested on the
  client with a screenshot test per glitch against a fixed scene.
- [ ] **Looked-at detection** in the senses: facing within 10 degrees for half a second, how
  long, whether moved toward.
- [ ] **Play it** through stage four, with the model. Calibration round two, same method.

### Phase 6: the ending, multiplayer, release

- [ ] **`settle`** and the three endings as three-day scripts (owner, 2026-11-09: "the endgame
  should be very grand"), each day's beats as data in `arc.rs` with a time, a condition and two
  forms the model picks between, the budget and cooldowns off and the caps on, the chains
  closed inside them, the one death on hard with inventory kept and the vanilla message, and
  the `left` and `stayed` states. Scripted-stay tests for each ending, checking the order of
  beats, that nothing repeats within a day, and that the death happens once or not at all.
- [ ] **Temper** (owner, 2026-11-09: "doing certain things should speed up the progression"):
  `memory/temper.rs` with the move table of [Provocation and temper](entity.md#provocation-and-temper),
  the senses that feed it (the figure chased or hit, its blocks broken, its place walled or
  lit, the thing taken back, chat lines at it, nights slept through), the scaling of the
  budget, the stage lengths, the menu's stage gates, the distance and the ending, each bounded
  by the preset, and the `How does it take this?` ask. Scripted-stay tests: a provoking player
  reaches stage four days sooner than a quiet one; a `calm` preset provoked stays under the
  `balanced` resting pace; temper drifts back.
- [ ] **The other house, a dimension of its own** (owner, 2026-11-09: "custom dimensions you
  can get teleported to by sleeping, walking through portals"): a void dimension in the
  content jar's data pack (`data/lodger/dimension/other.json`, a flat dark sky, no mobs, no
  day), into which the host copies the target's house block for block at stage four, and one
  visit: the target falls asleep and wakes in the copy, in the dark, the doors leading to
  nothing, the figure in the doorway, for one minute, then wakes in their own bed with the door
  open. A second visit only in the `take` ending's last night, where the walk from its place
  goes through its portal frame into the other house and the death, if any, is there. Not a
  place to explore: a set piece, twice at most, so the world stays the player's. Needs the
  host's teleport and block-copy commands and a dimension registered from the content jar;
  judged in the dev client before the design goes further.
- [ ] **Chains** (owner, 2026-11-09: "one that influences the next"): follow-ups in `events.rs`
  (weighted or exclusive, with a carried place or item), `Memory::chains`, the situation line,
  and the weights in `offer.rs`. The eight chains of [Chains](entity.md#chains) as data, with a
  scripted-stay test that walks at least two end to end and one that no step is offered with its
  needs unmet.
- [ ] **Multiplayer:** the `Others` memory, the target ask, witnesses' traces and nothing else,
  retargeting once. Tested with two dev clients on a dev server.
- [ ] **Performance pass:** the per-tick JSON exchange measured (it should be well under 50 µs
  on a quiet tick), the senses' distance checks bounded to the target's chunk neighbourhood,
  the model's unload timer. Nothing the mod does should show in a tick profile on a server
  with the lodger idle.
- [ ] **A proper release:** the jar with three natives, `THIRD_PARTY_NOTICES.txt`, the
  README's install steps checked on a fresh machine on each platform, a Modrinth page that says
  what the mod records and downloads. Version `0.1.0`.

### After every phase

- [x] **A debug UI for events, senses and the brain** (owner, 2026-11-09). Done 2026-11-09,
  earlier than planned, as the debug screen above (phase 2's menu item): the senses as meters,
  the memory's lines, the plan and the last decisions, every event with a Fire button, the
  world's reaction in the Log tab. Was: once the phases are in, a proper in-game screen over the
  menu's debug page, with the open chains and the temper and a want button. Still to do as the
  mind grows: the model's menus and confidences per decision (phase 4), the chains and the
  temper, the want.
- [ ] **A Minecraft version bump, rehearsed:** take the host to the next snapshot once, by the
  process in [Upgrading](upgrading.md) (written 2026-11-09 at the owner's ask), and fix
  whatever it missed; add the bump to its history table.
- [x] **The surroundings in the mind** (owner, 2026-11-09: "you're also detecting structures
  nearby and have code to determine where their house is?"). Done 2026-11-09: the brain asks
  the host's pass once a day 96 blocks around the home, and again when the home moves 16
  blocks; the host's pass gained the bell and the generated structures by tag (village,
  mineshaft, ruined portal, shipwreck, ocean ruin) at their middles; the memory keeps the
  features (`Surround`), the vehicles near the home and the village scan's doors and bell;
  `Need::Around(kind)`, `Vehicle`, `TargetUnderground`, `TargetAte`, `TargetApproachingDoor`,
  `Scanned(kind)`; sixteen rows over them: `boat_moved`, `boat_adrift`, `boat_drifts` and
  `minecart_rolls` (owner: "boats and minecarts move on their own", a `push_entity` command),
  `bucket_floor`, `snow_cleared`, `trample`, `harvest_one`, `mining_sounds`, `echo_steps`,
  `late_eating`, `scan_village`, `village_doors`, `lamps_noon`, `plate_laid`,
  `door_opens_itself`. The debug screen's Story section tells each decision with its menu, its
  situation and its outcome, and the HUD overlay (F9) shows the essentials while playing. Was:
  ask once a day, keep the features, turn `Need::Water`, `Field`, `Route` on, add the structure
  lookup. Still to do: pens, caves and the target's tunnels in the pass; `Need::Route`; the
  harvest reads ripeness (today any crop block of the kind is taken).
- [ ] **A game test for `tab_entry`'s packet** (the player-list entry is set by reflection in
  `Executors.withEntries`; a version bump can break it silently) and for the chest lid's block
  event.

### Later, or maybe never

- [ ] **GPU natives** (`metal`, `vulkan`) as separate downloads rather than in the jar, if anyone
  asks. 0.5B on a CPU is fine.
- [ ] **A second lodger kind** with a different want and events, once the first has been played by
  people other than the author. The data-first events and the staged arc are there to make this
  cheap.
- [ ] **Sound design pass** with a composer: the late steps, the door, the music cut. Everything
  in the first release is vanilla's own sounds re-timed.
- [ ] **Publishing the SDK.** The crate to crates.io under its final name and the host jars to
  Modrinth, once a second mod has been written against it and the protocol has stopped moving
  for it. Until then both live here. The crate name is the owner's to pick; `craftkit` was free
  on 2026-11-08.
- [ ] **The host checks that content jars match across sides.** Today a client without the
  lodger's jar joins a server with it, and sees missing textures where the figure would be. The
  host should compare the content jars' ids and versions in the handshake and say which is
  missing.

## Thought experiments

Things considered and set aside, kept so they are not reconsidered from scratch.

- **Letting the model write the signs.** Set aside at first: three words from a 0.5B model
  about a player's chest are three words of nonsense a third of the time, and once it writes one
  thing a player finds cruel the mod is over. Reopened by the owner (2026-11-09): the model
  writes, with the bank for that kind of line as its examples, under a token mask, a word count,
  a blocklist and a log, with the bank as the fallback, and the default model is the 1.5B. The
  caution stands as the checks; the calibration round reads every generated line.
- **A Rust Minecraft server (Pumpkin, Valence) instead of a Fabric mod,** so the whole thing is
  Rust. Vanilla clients could then play with no mod. But the figure in your skin, the late
  footsteps and the music cut all need the client to do something vanilla does not, and a
  server written from scratch is a project the lodger would be a corner of. The Java body is a
  few thousand lines; it is the right price.
- **JNI instead of FFM.** Minecraft 26.x runs Java 25, where FFM is final, so the body can bind
  the C ABI with no C glue and no `javac -h`. If a target ever runs an older Java, JNI is a day's
  work on the same seven functions.
- **Reading the player's screen, chat or mods for reactions.** The senses read the server's own
  facts. Chat is tempting (a player typing "what the hell" is a reaction) and would make the
  lodger read what players say to each other; no.
- **A chase, even one.** Every playtester of every stalker mod remembers the chase as the moment
  the fear ended. The lodger's last night is bright and still.
- **More than one lodger per world,** or one per player. The want and the ending are singular on
  purpose; two lodgers are a mob type.

## Done

The `[x]` lines above, each dated. Summary by day:

- **2026-11-08:** the docs, from `~/chainv4`'s style and `~/engine`'s AI layer.
- **2026-11-09:** phases 0 to 2 (the SDK, the ABI, the host for both loaders, the content jars,
  the download with the license screen, the model layer), phase 3's host side (the senses, the
  executors), the mind's first and second cuts (the memory, 87 events, the rules, the arc, the
  rule brain, the log, scripted stays), the debug screen, the glitch renderers, the kill tally,
  the landmarks, the text banks, the upgrade guide. First seen in a dev client the same day.

## At the very end

- [ ] **The host in Rust, behind a thin Java wrapper** (owner, 2026-11-09): what decides or
  keeps state in the host (the timers, holds, loot offerings, reads, the debug model) moved
  into Rust beside the SDK, the Java left as the Minecraft-facing edge alone (the loaders'
  hooks, the game's API calls, rendering), since the game's own API is Java.

- [ ] **Split the project into two repositories** (owner, 2026-11-09): one for the SDK
  (`sdk/`, the craftkit host in `host/`, their docs and CI) and one for the lodger (`core/`,
  `mod/`, its docs), the lodger depending on the published SDK crate and host jar.
