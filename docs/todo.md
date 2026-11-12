# Notes and todo

What lodger still lacks, in the order to build it, and what got done. The things to keep in mind while running or changing it are in
[Good to know](good-to-know.md), and how each part works is in the [docs](README.md).

## Where it stands

The mind, the host and both jars build and pass their tests, and the lodger has been played
in the dev client: the whole catalogue is coded, the model decides and writes, and ten bug passes are
done. What is next, in order, is in [Queue](queue.md); every open `[ ]` below is something still
to do.

## Todo

The phases are in dependency order and each ends with something that runs. The mind comes first,
because it can be built and tested without a game, and because the model layer is mostly a port.

### Phase 0: the toolchain, and making the docs true

- [x] **Confirm the versions.** Done: Minecraft 26.3 (the current stable; the docs
  had guessed 26.2), Fabric Loader 0.20.1, Java 25 (Mojang's manifest), Fabric API 0.165.2+26.3,
  Loom 1.19, Gradle 9.8.2 and Mojang's mappings, from Fabric's metadata and example mod and
  Mojang's manifest. `VersionsTest` keeps the places agreeing. Was: Minecraft 26.3, Fabric Loader
  0.20.1, Java 25, and the Fabric API and Loom releases for them, from fabricmc.net and the Loom
  template, not the secondary sources the docs were written from.
- [x] **The SDK's shape, and the decision to build it now** (owner): craftkit
  (working name), a crate with the `Mod` trait, the C ABI, the protocol and a harness, and a host
  for Fabric and NeoForge; Rust mods as content jars with no Java. See [The SDK](sdk.md). Was:
  the lodger as a Fabric mod with its own Java body.
- [x] **The host nested in each mod's jar, natives for six platforms** (owner): one
  lodger jar per loader with the craftkit host inside (Fabric `jars`, NeoForge jar-in-jar), the
  CI building the mind on Linux, macOS and Windows runners for x86_64 and arm64 each. Was: the
  host a jar of its own, one content jar for both loaders, three natives.
- [x] **The SDK crate and the mind build and pass**: `sdk/` with `abi`, `protocol`,
  `harness` and `export_mod!`, `core/` as a `craftkit::Mod` with its config schema, 22 tests
  across both (`cargo test --workspace`), clippy and rustfmt clean, `liblodger_core.dylib` built
  in release. The docs-and-schema test caught a rename that had turned `react_threshold` into
  `reevent_threshold` in [Config](config.md).
- [x] **The host builds for both loaders and passes its tests** (`host/`). Done:
  `craftkit-fabric-26.3-0.1.0.jar` and `craftkit-neoforge-26.3-0.1.0.jar`, 18 JUnit tests in
  `common` (manifest, natives, config file, loading failures, versions agreeing). Three things
  the real APIs corrected: 26.3 gates commands with `Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)`,
  not `hasPermission(int)`; Fabric's registry is `PayloadTypeRegistry.clientboundConfiguration()`
  over a plain `FriendlyByteBuf`; NeoForge's `IModFile` gives `JarContents`, not paths, so
  `Platform.ContentJar` now hands the host bytes, which suits both loaders. Was: the MultiLoader
  layout, `Native.java` binding the five phase-0 functions through FFM, `LoadedMod` doing the
  manifest, native, ABI, info and config steps, `/craftkit status`, the handshake on each loader,
  and the JUnit tests in `common`.
- [x] **The lodger's content jar builds** (`mod/`). Done: `./gradlew jar` runs cargo,
  and `lodger-0.1.0.jar` holds `natives/macos-aarch64/liblodger_core.dylib`, `fabric.mod.json`
  and the `neoforge.mods.toml` with no language loader with the versions filled in, `META-INF/craftkit.json`,
  `pack.mcmeta` and the lang file. Was: cargo run by Gradle, the native under
  `natives/<os>-<arch>/`, the three manifests and `pack.mcmeta` filled in.
- [x] **A dev server starts with both jars on each loader.** Done: `:fabric:runServer`
  and `:neoforge:runServer` in `host/` with `lodger-0.1.0.jar` in their `mods/` both log `loaded
  lodger 0.1.0 (sdk 0.1.0) from macos-aarch64` and `wrote lodger.json with every default`, and
  `config/craftkit/natives/lodger/macos-aarch64/liblodger_core.dylib` appears. The whole FFI
  path, exercised once before anything rides on it. Found on the way: f32 fractions were written
  back as `0.4000000059604645`, now f64; NeoForge calls `lowcodefml` deprecated, so the content
  jar's `neoforge.mods.toml` names no loader. Still to see by hand: `/craftkit status` in a
  client. Was: a dev client starts with both jars and `/craftkit status` shows the lodger.
- [ ] **The CI runs green** (run by hand, Actions, ci): the checks on macOS, Ubuntu and
  Windows, the five natives built by `scripts/build_natives.py` (macOS and both Windows on a Mac
  runner, each Linux in Docker on its own chip), then both jars by `scripts/build_jars.py`. Run it
  once to prove it; see [Building](build.md#the-ci). Was: six runners each building its own
  native, CPU only, on every push.
- [ ] **`THIRD_PARTY_NOTICES.txt`** generated and checked in CI (see
  [Development](development.md#third-party-notices)).

### Phase 1: the model layer

- [x] **Port `~/engine/game/base/src/ai/` into `core/src/model/`.** Done: `mod.rs`
  (the `Model` the `Lodger` owns, in place of the engine's static slot; validation; the scoring
  math), `queue.rs` (the worker and keys, verbatim), `download.rs` (paths and `.part` in place of
  the engine's `Fs`), `infer.rs` (the engine's `infer` module; one change, the context's model
  field is private in `llama-cpp-2` 0.1.158 so the vocab is read through the engine's own model
  pointer). Behind a default-on `model` feature, with `metal` and `vulkan` features for the GPU
  experiment. The first build compiled llama.cpp in about six minutes; the library went from
  0.5 MB to 6 MB. Was: the port, with `crate::fs` replaced by a plain path and the Lua library
  left behind.
- [x] **The `Scorer` trait, the fake scorers and the queue tests.** Done, the
  engine's seven queue tests and the model's own: answers without blocking, a loading failure
  returned from `start`, superseding by key, order kept, the queue bounded at 64, stop finishing
  the running one and refusing the rest, the scorer dropped on its own thread. 32 tests in
  `core`. The ABI side (`ask` as a command the host sends) is phase 2's, with the world calls.
- [x] **The download, as the host's, with the title screen gated** (owner: "lock
  them out of the main menu until the model is downloaded and loaded"). Done: the
  SDK's `Resource` in `InitReply`, `warm` and `status` in the ABI (version 3), the lodger
  declaring its model; the host's `Downloader` (JDK `HttpClient`, `.part`, magic, SHA-256,
  progress), `Preparation` (fetch the missing required resources one at a time, then `warm`
  each mod, progress for a screen or a log), `PreparingScreen` over the title screen with a bar,
  Quit and Try again, `TitleGate` hooked on both loaders' screen events, and
  `Host.prepareBlocking` on both loaders' server-starting events. Six JUnit tests for the
  downloader and three for resources. Was: the engine's `download.rs` ported into the mind,
  which was then removed, so one downloader serves every mod.
- [x] **The real-model tests,** written and `#[ignore]`d with the reason: the
  want question on a player who walked toward the figure, and two questions on one situation
  agreeing with each asked alone, and `warm` then `status` through the ABI. Run with
  the downloaded model copied to `core/models/qwen.gguf`: all three pass. A player who walked
  toward the figure makes the model answer `to be looked at`. The tests take turns on one lock,
  since llama.cpp's backend is one per process and three loading at once raced.
- [ ] **Measure.** Time and resident memory for a 300-token situation with four questions of
  four choices on two threads, on a laptop and on a small VPS. Put the numbers in
  [Model](model.md) in place of "tens to a few hundred milliseconds".

### Phase 2: the mind without a model

- [x] **The SDK's world calls.** Done (ABI 4): `Mod::World`, `open`, `observe`,
  `tick`, `report`, `save`, `close` in the trait and `export_mod!` (worlds by handle in a map the
  macro keeps, closed at shutdown); `sdk/src/world.rs` with `WorldInfo`, `Now`, `Observations`
  (players with their state and movement, entities, the world's weather, events, chat, scans,
  surroundings, loaded mods), `Command` (log, scan, sound, hint, message, toggle, move and set a
  block, move and drop items, sign, cancel sleep, look at, figure) and `Report`; saves as
  base64 in JSON; the harness opens a world and drives it; a two-line mod's world is opened,
  observed, ticked, reported, saved and reopened from its bytes in the SDK's tests. The host's
  side (the world loop, saved data, the first executors) is the next item.
- [x] **The host's world loop.** Done: `host/common/.../world/Worlds.java` opens a
  world for each mod when a level loads (`ServerLevelEvents.LOAD` on Fabric, `LevelEvent.Load`
  on NeoForge), with its bytes from saved data (`WorldData`, a `SavedDataType` per mod holding
  the mod's bytes as an NBT byte array, which the game writes to
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
- [x] **The senses, in full** (owner: "a good harness that can get the current
  biome, mobs around, etc."): everything under Observations in [The SDK](sdk.md#the-protocol),
  on the host side as `senses/` with one collector per kind (players, entities within 64,
  world state, the `Scan` command, the daily surroundings pass), each sliced over ticks and
  measured, and on the SDK side as typed structs with a `describe` that turns any of them into
  the plain lines a model can read. The harness gets a `World` builder so a test can say
  "a plains biome, a lake 20 blocks north, three cows, night, raining" in a line. Found done in the audit: `senses/Senses.java` sends players, entities, what each looks at, scans and the surroundings; the one-line `World` builder was never built, `tests/stays.rs` builds its observations itself.
- [x] **The menu, its debug half** (owner: "a ui we can pull up with command/key
  that lets us spawn events", "meters showing different senses, gauges, full debug suite").
  Done as the debug screen: `sdk/src/debug.rs` (`DebugRequest`: snapshot, force,
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
- [x] **Presets** (owner: "3-4 presets, like a calm one, then aggressive; start off
  balanced"). Done: `preset` in the config, `calm`, `balanced`, `intense`,
  `relentless` as `Preset` in `config.rs` with the table in [Config](config.md#presets), the
  preset laid under the file's own keys, every preset passing its own checks, `balanced` equal
  to the defaults.
- [x] **The license before the download** (owner). Done: `Resource.license`
  in the SDK (name, URL, text), the lodger's model carrying Qwen's Apache-2.0 notice, the host's
  `LicenseScreen` before the first download of each resource with Accept and Quit, acceptance
  remembered in `config/craftkit/accepted.json`; a dedicated server logs the terms and waits for
  `accepted` in `config/craftkit.json` before it downloads.
- [x] **Three model sizes** (owner). Done: `model.size` of `0.5b`,
  `1.5b` or `3b`, each Qwen's own `q4_k_m` file with its URL, file name and size for the bar,
  `model.url` and `model.path` empty by default and derived from the size; the table in
  [Which model](model.md#which-model). The default moved to `1.5b` the same evening (owner:
  "make the 1.5b model the default"); the dev client's config keeps `0.5b` so no download
  starts there.
- [x] **Seeded picking** (owner: "on startup the model would give the same answers
  to the same questions"). Done: the pick is a draw weighted by
  `softmax(scores / brain.temperature)` from a SplitMix64 seeded per start (the clock and the
  game directory), `0` restoring the engine's always-the-best; the confidence stays the plain
  probability; `Model::reseed` for the world's seed in phase 2; the seed logged with decisions
  (phase 2's log). Tests: temperature zero picks the best, a warm draw follows the scores to
  within a few percent over two thousand draws, the same seed repeats and another differs.
- [x] **`Stay`, the lodger's `World`.** Done: `core/src/stay.rs`, `bincode` behind a
  format byte, refusing a newer format or another world's save by name, counting ticks,
  players and reports; the model reseeded with the world's seed on open; tests through the
  ABI and direct. Format 3 (the same evening) is JSON with `#[serde(default)]` on every
  struct, so a field added to the memory reads an older save without a migration; formats 1
  and 2 open with their counts and a fresh mind.
- [x] **Memory, first cut.** Done: `core/src/memory.rs` with the target (the first
  player seen), the home (the bed slept in, or the densest cell they stand still in; doors and
  lights from blocks placed; the floor), the containers opened with their contents and slots,
  the favourites (what is in their hand), the held item, what was done and how it went, the
  target's last state (position, facing, indoors, asleep, walking, creatures near, look-backs),
  chat lines about the lodger, and the placed blocks, each capped with its reason; `in_view`
  and `target_away_from`; `describe` as a handful of template lines. Six tests. Was: every piece
  of [What is kept](memory.md#what-is-kept) with its bound and the size test. Still to do: the
  routes, the hours, the reactions, the figures seen, the surroundings and activity pieces, the
  temper and the chains, and the 256 KiB test.
- [x] **Routes, hours and reactions** (the memory's next pieces). Done: `Route`
  (a trail of a point every two blocks, cut at a minute's stop, sleep or leaving; kept when
  ten points or more; counted into the route it repeats), `Session` (joined and left in real
  time, the game time at joining; the usual minutes and whether they join after dark),
  `Reaction` (lit, walled, watched, left, ignored, read off what the target does in the day
  after a landmark); `Memory::clock` fed once a tick by the stay; a line of the situation text
  for each; `Need::Route`, `Where::RouteEnd`, `animal_gone`, and `light_path` toward the
  route's end. Tests: a walk repeated becomes a route with its far end known; sessions give
  the usual length; a torch placed after a take reads as `lit`, a day of nothing as `ignored`.
- [x] **The catalogue, third cut, and the group switches.** Done: twenty-four more
  rows (stairs and banners turned, frames emptied and swapped, the stand dressed, the tool worn
  down, the cold furnace, the chest opening as the target walks up, sorting by colour, a flower
  planted, a crop replanted, an ice path, the occupied bed, waking a block over, the pet
  outside, the player-list entry that stays, the note tune, the piston, the phantom disc, the
  fog closing in, snow and rain indoors, the ore scan and the torch on the vein), over four
  more commands (`rotate_block`, `set_frame_item`, `equip_entity`, `damage_item`) and a player
  moved two blocks at most; `events.sounds`, `events.glitches` and `events.chat` now gate their
  groups in the offer and in forcing (they were read and unused); `Need::InChestAny`,
  `TargetApproachingChest`, `ColouredStacks`, `ToolInContainer`. 126 rows coded.
- [x] **Scripted stays, the other six.** Done: no event inside its cooldown or past
  its cap over fourteen days; a disabled event never happens and a group switched off stays
  quiet; sleep is only refused with the player in bed and house sounds never play for a player
  at the mine; calm acts less than relentless; the stay is over after stage five and nothing
  happens after; the memory learns the house, the chest, the favourite and the route from the
  script (which now walks to the mine, 77 blocks over 200 ticks, so a route forms) and the save
  stays under 256 KiB. Ten scripted stays in all.
- [x] **The offer** (`brain/offer.rs`): the six filters of
  [What is offered when](entity.md#what-is-offered-when), with a test per filter (no water, no
  boats; asleep, no `late_steps`; stage one, nothing from stage two) and a test that a hundred
  draws from one situation offer every eligible event at least once. Found done in the audit: the menu's tests cover stage one, needs narrowing the menu and a hundred draws, not one test per filter.
- [x] **`Memory::describe`, in full.** The twelve lines of [Describing](memory.md#describing)
  with tests on the exact wording, and the truncation from the bottom at a measured token
  count (the model's tokenizer; without a model, words times 1.4). The first cut writes eight
  of the lines. Found done in the audit: `describe` and `describe_within`, truncated at a measured length.
- [x] **The catalogue, the second cut** (owner: "flesh out the events"). Done:
  87 rows in `events.rs` over the host's commands, with needs that carry data
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
- [x] **What they kill** (owner: "keep track of mobs hit/killed and do something with
  that"). Done: the `killed` event from both loaders (the damage source's attacker),
  `Memory.tally` by mob kind (hits, kills), `most_killed` at three kills, a line of the
  situation text, and `kill_sign`, `grave` and `kill_line` over `brain/text.rs`'s `KillSign`,
  `GraveSign` and `KillLine` banks.
- [x] **Never forgotten** (owner: "certain actions are always remembered, where it
  makes sense"). Done: `Memory.landmarks` (every bold event that landed, kept for
  the stay; the caps per stay count from it when `done` has trimmed) and `Memory.milestones`
  (where they died, the first sleep, the first kill of each kind, a dimension crossed), each
  with a line in `describe` and a section on the debug screen.
- [x] **Text by kind** (owner: "it should generate, with examples correlating to
  specific actions"). Done, the bank half: `brain/text.rs` with a `Kind` per place
  words go (sign word, chat line, kill line, kill sign, whisper, animal name, grave sign), each
  with its examples and its most words, `fill` for `{kind}` and `{n}`; `brain.text` defaults to
  `generate`, which until phase 4 picks from the bank like `bank` does. Phase 4's item below
  wires the model in with these banks as its examples.
- [x] **Events as data, the rules, the arc, the rule brain, the log.** Done:
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
- [x] **Scripted stays, first four.** Done: `core/tests/stays.rs` drives a scripted
  player (a house, a bed, a chest opened each morning, a door and torches, a day at the mine)
  through the ABI for days on the rule brain: a twelve-day stay acts within its budget and never
  touches the house while the player is home and awake; nothing happens before move-in and
  stage one never takes or refuses sleep; a saved stay reopens where it was; no event lands
  twice in a row. Twelve days run in about five seconds, with the clock moving five ticks at a
  time. Was: the `Player` script and the first ten tests. Still to do: the other six.
- [x] **Design decisions taken with the owner:** Rust mind and Java body through
  FFM; `lodger` as the name; the one death on hard kept as the only harm; the denser pace
  (budgets 1, 2, 4, 6 a day, a stay of about two in-game weeks with forty to sixty events) over
  the rarer one (half that over three weeks).

### Phase 3: the body, stages one and two

- [x] **Senses, the host's side.** Done: `senses/Senses.java` sends every player's
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
- [x] **The surroundings pass, first cut.** Done: the `surroundings` command starts
  a pass around a point within a radius (at most 160), 64 columns a tick, a column every four
  blocks read at the surface: water, lava, sand, snow, trees, crops, lights, beds, containers,
  doors, each as a feature with a center and a size. Answered as an observation when done.
  Was: once an in-game day within 128 blocks of the home, in slices. Still to do: villages
  (the bell's point of interest), the target's tunnels and unentered caves, pens with their
  animals, and the mind asking for it once a day.
- [x] **The general commands' executors, first cut.** Done, in `world/Executors.java`:
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
- [x] **Budgets, cooldowns, caps and the menu** (`events.rs`, `brain/offer.rs`) with scripted-stay
  tests: a stay of fourteen days on the rule brain holds between forty and seventy events, no
  event twice in a row, none past its cap, none inside its cooldown. Found done in the audit: `tests/stays.rs` checks the budget, no event twice in a row, cooldowns and caps.
- [x] **The net, first cut.** Done: sounds go as vanilla's own sound packet to one
  player; `HintPayload` (kind, optional place, seconds, detail) is registered on both loaders
  and lands in the client's `Hints` list, timed and logged. Was: the sound packet, the hint
  packet, the target flag; the client handlers. Still to do: the target flag, the footstep
  sound by block for `late_steps`, and the first two glitch renderers.
- [x] **Saved data:** `lodger.dat` holding the mind's bytes, saved with the world, loaded on
  world load, the mind closed on unload. Found done in the audit: the mind's bytes are in the level's saved data as `craftkit_<mod id>` (`WorldData`), not a `lodger.dat`.
- [x] **`status`, `reset` and `leave`** (owner: "reset wipes, leave ends gently"): under
  `/craftkit lodger`, operators only, from anywhere and the console, always the overworld's
  stay. `status` says the stage and how far into it, whom it lives with, the things done that
  they would notice and what it holds, and whether the model or the rules decide; `leave` begins
  the leaves ending at the next dawn (before move-in, it never moves in); `reset`, typed twice
  within 30 seconds, gives back what it holds and begins the stay again, keeping the target and
  the mods it saw. `decisions` is the log file and the debug screen, not a command. The docs'
  `/lodger ...` commands, which never existed, now name these. Tests: two in `debug`. Was: **`/lodger status`, `/lodger decisions`, `/lodger reset`, `/lodger leave`.** Partly, found in the audit: `/craftkit status` and `/craftkit lodger debug|show|force|stage|action` exist. Still open: reset and leave, if wanted.
- [x] **Play it.** A fresh world, dev config, through stages one and two on the rule brain. Write
  down everything that read wrong in [Good to know](good-to-know.md) or here. Found done in the audit: the owner has played through the stages and the fight; what read wrong is in the playtest entries.
- [x] **The first play session's fixes** (owner at the keyboard, the 1.5B deciding).
  Done:
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
  - **The house, part two:** `door_hinge` (a door rehung, `set_state`), `ladder_gap` (a middle
    rung set on top), `flowers_potted` (the flowers around the house in its empty pots) and
    `pumpkin_face` (carved where it grows, facing the door, the shears worn); the home read
    keeps the flowers and pumpkins around it; `set_block`'s state applied at last. ABI 17.
    Was: in the catalogue alone, and the state field ignored.
  - **The house, part three:** `tree_grown` (the sapling by the house a tree overnight, their
    bone meal spent only if it grew; `grow`, ABI 18) and `backfill` (their holes by the house
    filled with what it dug); the memory keeps the natural ground they dig near the home.
    Was: in the catalogue alone.
  - **The house, part four:** `alcove` (one block of natural stone hollowed behind a house wall,
    a torch and its things in it; the owner's one exception to the dig rule), and four new:
    `furniture_shifted` (a work block one over, contents and all), `one_room_dark` (a room's
    torches taken down into their chest, one at a time), `door_held` (a door they open shuts
    again, one time in four; `used_door`, ABI 19) and `bed_turned` (`set_state`'s `@turn`).
    Batch 3 is done. Was: in the catalogue or not at all.
  - **Half the GPU for the game** (the review's M2; owner: "limit it to like 50% gpu cuz it
    makes my game stutter"): `model.gpu_share` (0.5) puts half the model's layers on the GPU and
    half on the CPU, and the prompt is decoded in chunks of 64 tokens with a millisecond's rest
    between; the old `model.gpu_layers` was never passed to llama.cpp, whose default put every
    layer on the GPU. Was: the whole model on the GPU, one burst per question.
  - **The model picked on the download screen** (the review's O1; owner: "I want to be able
    to change the model on that screen"): the model resource offers the three sizes as
    `choices` (ABI 20); the screen's button cycles them, and a pick writes `model.size` into
    the config, initialises the mod again and fetches that one instead. Was: the size from
    the config file alone.
  - **A settings page** (the review's O1, the queue's item 5; owner: a title screen button,
    the useful keys grouped): the lodger offers eleven keys (the model, how scary, pacing,
    words) as `settings` (ABI 21); the host's settings screen shows them, Done writes them into
    the config and loads the mod again, fetching a new model size at once. Was: the file alone.
  - **The settings page scrolls** (owner: "looks bad and i cant scroll it"): the rows sit in
    a dark panel between the title and a footer of Done and Cancel; the mouse wheel scrolls
    them, rows out of the panel hide and take no clicks, a thin bar shows where the list is,
    the groups are dim headings (`ListLayout`, with its tests). Was: rows ran past the screen's
    foot and Done sat over the last of them.
  - **A config value of the wrong type no longer stops the mod** (the dev client: `2.0` for
    `brain.most_plans`, written by the settings screen, refused the config and the mod with it,
    so its settings button was gone; owner: "make that more robust when it comes to parsing"):
    the settings screen writes whole steps whole; the mind reads `2.0` as `2`, takes text that
    reads as the key's type, and leaves any other wrong value at the preset's, each named in a
    warning the host logs (`InitReply.warnings`, ABI 22). Tests: the config's three, the
    setting's whole step. Was: the first wrong type refused the whole file.
  - **Habits** (the review's N1; owner: every proposal taken): where each part of the day
    goes, when they leave home and come back, nights slept or stayed up, the door they use
    (the knock and the worn path come to it), what they carry and wear every day, how much they
    light, how often they look back; sampled every hundred ticks, a line each once there is
    enough. Tests: twelve. Was: sessions and the main hand only.
  - **A bigger context, and the lines kept inside it** (owner: "can we increase context window
    ... to 1500?"): `model.context` 1536 (a multiple of 32), and `describe` takes its detail
    lines only while they fit the configured context (`describe_chars`). Real-model tests pass
    at 1536. Was: 512, and nothing kept a long memory from overflowing it, which fails the
    question to the rules.
  - **People** (the review's N4): who comes to the home (by name, once a day each) and who is
    with the target now, as detail lines; `Memory::alone`. Tests: four. Was: the players' names
    for the target ask only.
  - **Qwen3-4B-Instruct-2507 offered** (owner: "how about we offer this too?"): `model.size`
    `4b`, from unsloth's Q4_K_M (Qwen's own GGUF repo answers 401), 36 layers; every size now
    carries its file's exact length and SHA-256, which the host checks; each its own license on
    the screen, and the 3B's is Qwen's research license. Tests: the sizes', the 4B's resource.
    Was: three sizes, approximate lengths, no sum, and the 3B shown as Apache.
  - **Places** (the review's N3): the house in rooms by what stands in them, upstairs and
    under the house; the sites they keep away from home (a mine, a second base, a spot); a
    recent event near an earlier landmark refers back to it; distances along the ground.
    Tests: six. Was: "in the house" or so many blocks off, by the 3D distance.
  - **How they took each event** (the review's N2): every landed event that does something to
    them is watched for a day by its effect, eight at once: startled, looked, fixed (and in how
    many minutes), ran, fought, beside lit, walled, watched and left; the share of each effect
    they notice leans the rule brain (0.6 to 1.4) and is said to the model ("Words get their
    attention; sounds do not."). Tests: six, and the rule brain's. Was: the bold events only,
    one at a time, a new one closing the last as ignored.
  - **A download that stalls goes on** (the dev client: the 4B "got stuck at 2059 out of 2381
    mb"): thirty seconds without a byte closes the connection; the download goes on from where
    it stopped with an HTTP range, five tries; a part file cut off by the network is kept and
    taken up at the next start; the sum is checked over the whole file. Tests: six new against
    a local server (quiet, resumed, range ignored, whole part, always failing, 404 not retried).
    Was: no read limit, so a quiet connection hung forever, and any failure deleted the part.
  - **`lantern_ahead` and `cave_lit`** (batch 4; owner: both, the lantern from their chest):
    a cave read every ten minutes near home finds dark air (scan `step`, `dark`, `most`, ABI
    23); the cave they have not been near gets one of their lanterns far down it, or torches
    at their own tunnel spacing. Tests: five in `caves`, the scan sampling's in JUnit, the
    catalogue's. Was: two rows not coded.
  - **No house sounds in a cave** (the dev client: "a door sound just played while i was
    underground"): a cave is no longer indoors (no sky light below y 56 is underground, not a
    house); away from home and from any house a door, chest or window sound has no spot, so it
    is not offered and, asked anyway, not made. Tests: the cave case end to end; the old test's
    "no other house known" now expects no spot. Was: indoors was any spot without sky light,
    the home's door 200 blocks off stood in, and a sound's moving ways played at their feet.
  - **`death_light` and `light_taken`** (batch 4, two of the owner's four new): a lit candle
    where they last died, once a death; the light they placed last carried six to ten blocks
    on the way they went, away from the house, set down before the old one goes. Tests: four.
  - **`lights_out` and `soul_swap`** (batch 4, the owner's other two): at night their lights
    behind them go out one by one and come back a minute later (`hide_block`, put back at once
    when a level closes); one of their torches or lanterns turns to soul fire, the same shape
    (`swap_block`). ABI 24. Tests: four in `lights`, `Due`'s in JUnit.
  - **Warped animals** (owner: "warped versions of the entity ... a cow or pig"; "3 but also
    mobs outside if they dont own enough"): `warped_animal`, one of their animals or one
    outside, frozen with its head on them and one or two of too tall, upside down, creeping
    closer while they look away, sometimes in their doorway first; put back as it was after a
    minute or when they come within six blocks (`warp_mob`, ABI 25, a tag of how it was so a
    quit never leaves it wrong). Tests: three in `warped`, `WarpMath`'s in JUnit.
  - **Batch 5, beds and rest** (owner: build `shift_bed`, all four new): `shift_bed` (one block
    to the side, both halves, the respawn point moved with it), `sheets_changed` (another
    colour, both halves at once), `under_the_bed` (scratching under it as they lie down),
    `foot_of_bed` (the figure across the bed as they wake, one waking in three), `second_bed`
    (once a stay). ABI 26. Tests: two in `act`. Was: `shift_bed` not coded.
  - **Batch 6, farm and animals** (owner: all four rows built, all four new, and the animals
    turning on them brought here): `scarecrow` (their stand and the very piece of their
    armour), `bees_out`, `horse_moved`, `field_word` (a word cut into the field from a 3 by 5
    font), `face_the_house`, `one_more` (gone by noon), `row_withered`, `eggs_to_door`,
    `animals_nip` (stage 3, half a heart, twice) and `animals_turn` (4 and 5, never below half
    a heart); a field read, `Need::Field` alive again. ABI 27. Tests: six in `farm`, one in
    `act`. Was: four rows not coded, and `Need::Field` always false.
  - **Batch 7, decoration, signs and kills** (owner: `sign_word` built, all four new):
    `sign_word` (the last written line of their sign), `count_up` (the kill sign's count rises
    with each kill of its kind, started by the kill like `door_held` by a door), their weapon in
    their empty frame straight from its slot, a mob head on a post by the door, a grave for each
    kind. ABI 28. The brain's player-started events share one `consider_started`. Tests: two in
    `act`. Was: `sign_word` not coded.
  - **Batch 8, chat and the player list** (owner: both rows, all four new): `sleep_message`
    ("1/2 players sleeping" while they are awake), `advancement` (one of theirs again, line and
    toast), `death_message` (the game's own death line while they live), `words_on_sign` (what
    they typed, on a sign by the door), `ping_fades`, `empty_message`. The stay test caught the
    death message outside `events.chat`; it is in the chat switch and group now. Tests: two in
    `act`.
  - **Batch 9, furnaces, workstations and redstone** (owner: all five rows, all four new):
    `jukebox` (their disc as they come to the door, or the playing stops), `lectern_page`,
    `swap_fuel` (a fair burn), `brewing_done` (the game's own brewing, given their ingredient),
    `grindstone_strip` (never a curse), `lamps_blink` (three short, three long, three short),
    `composter_fills` (`@up`), `crafted_for_you` (torches, bread, planks, sticks from their
    makings), `bell_rings`. ABI 29. Tests: three in `workshop`, one in `act`. Was: five rows not
    coded.
  - **Batch 10, water, boats, snow and weather** (owner: all five rows, all four new): prints in
    the snow out from the door and in to it (`prints`), the boat waiting on their shore,
    ripples crossing toward them, a bobber with no one fishing (sounds and the fishing
    ripples; a real bobber needs a player holding the rod), a snowman by the door, their boat
    in the house, a squid still under the water facing them, wet steps from the door to the
    bed. ABI 30. Tests: three in `water`, one in `act`. Was: five rows not coded.
  - **Batch 11, the sky, the light and the screen, with the atmospheres** (owner: every row,
    `reflection` as a face under the water, real fog with render hooks): mist, dusk at noon,
    red mist and darkness as real fog that eases in, breathes, eases out and stacks; red rain as
    red drops; the stars out, the moon full, the sky two hours on; a painting upside down until
    looked at; a compass turned to its place; the hunger bar, the hearts, a subtitle; their
    name twice; a face just under the water. Five render hooks (fog, sky, rain colour, compass,
    painting) in common code for both loaders, started on each to be sure they apply. Tests:
    four in JUnit (`Atmosphere`), two in `act`. Was: overlays on the screen, nine rows not coded.
  - **Batch 12, villages** (owner: `village_abandoned` as answered, `bell_twice` with it):
    a village lantern on their route, what it took for sale at a villager once, the village
    emptied into one house till dawn (packed, facing the wall, turning to them; the other doors
    open, the lanterns out; all back by dawn), and the bell they ring ringing again. ABI 31.
    Tests: three in `village`, two in `act`. Was: three rows not coded.
  - **Batch 13, mines** (owner: all six rows, all four new): a tunnel memory (what they dig
    underground, grouped as dug), a copy of their tunnel beside it (its ore to their chest),
    their oldest tunnel walled with its own cobblestone, digging toward them that breaks through
    or stops one short, their frame lit, the ore ahead mined out, a shaft with their ladders,
    the tunnel longer, its torches out from the far end, a sign deep down, gravel behind them.
    A dig's block is kept as a pick gives it (stone is cobblestone). ABI 32. And a panic found
    on the way: `tidy_moves` ran past a chest's last slot on two stacks said to share one; it
    sorts what the slots hold now. Tests: four in `tunnels`, two in `act`.
  - **Batch 14, the figures** (owner: the swing by the harm stages, all four new): the figure
    right behind them strikes once (a nip at 3, two hearts at 4 and 5, never below half a
    heart), copies their steps from forty blocks off, stands outside their window looking in,
    stands among the villagers and is gone once they look away, waves from afar. ABI 33. Tests:
    one in `act`.
  - **Batch 15, building** (owner: every one; materials gathered first, topped up when short):
    blueprints laid a night's share at a time (`place_blueprint`, ABI 34): its house near theirs
    over three nights with the chest holding what it took, the room it grows at stage 5, a
    pillar a block a night to their roof, a log hut halfway along their route, a room dug in a
    cave, a dock where they fish, a bridge across their water (finished at stage 5), the
    outline and the copy of their house. Its stone spent first. Two bugs found on the way: a
    stay test caught an overflow in "all at once" (saturating now), and the parallel-plans test
    leaned on chance and now sets its player at home. Tests: eight in `building`.
  - **Batch 15, like a player** (owner: all of them): the figure digs into the rock ahead and
    the stone closes behind it, or walks through the hill and out its far side, all on their
    client alone (`dig_in`); steps back out of their light as it flickers; crouches when they
    crouch; swings their tool at the rock from afar. ABI 35. The parallel-plans test now holds
    one plan and asks for a second, where it had leaned on chance. Tests: one in `act`.
  - **Batch 16, close and quiet** (owner: "the close and quiet ones"): at noon in the open
    their view turned two degrees and a step right behind them (`hand_on_shoulder`); crouched
    still ten seconds, breathing at their ear that stops as they move (`breath`, a new crouch
    sense, the brain dropping a sound's rest when they move); a second set of steps behind
    theirs at night (`footsteps`, ABI 36); their name in the subtitles (`name_called`). Tests:
    one in the brain, two in JUnit (`Stride`). Was: four rows not coded.
  - **Batch 16, the close ones** (owner: `pack`, `hostiles_close`, `shove`): their animals walk
    up to them together and stop four blocks off (`approach`); the monsters hold and stare and
    come nearer whenever they look away, back where they stood when looked at (`hold`'s
    `creep`); a push off the next edge of two or three blocks (`shove`, never harm). ABI 37.
    Tests: one in `act`, two in JUnit (`Edge`). Was: three rows not coded.
  - **Batch 16, more** (owner: `answer_knock`, the frames, the sky, the line): a door they
    knock on (struck twice, a new `knocked` sense on both loaders) knocks back half the time;
    every frame of the house shows the one thing to them alone (`frames_show`); the moon at
    noon for a minute (the sky hook); their bobber pulled under with nothing on it (`bobber`,
    the server's hook kept from a real bite). ABI 38. Tests: two in `act`, one in the brain.
    Was: four rows not coded.
  - **Batch 16, the Nether and the End** (owner: all three): the stays of the other dimensions
    act through `elsewhere` (the overworld's says each tick what its rules allow and what they
    need from home; theirs say what they did, and the overworld's records it, so the caps,
    cooldowns and budget hold): in the Nether the ghasts' cries read as their name in the
    subtitles (a subtitle hook); in the End an enderman holds the block they build with most
    (`carry_block`, ABI 39); back from another dimension after three minutes, one of its bold
    things at home is done at once (`busy_away`, a new sense of being gone without leaving).
    Tests: three in `elsewhere`, two in the brain, one in `act`, one in JUnit (`Subtitles`).
    Was: three rows not coded; the other dimensions' stays did nothing.
  - **A frame's own item was lost** (found reading the code for the endings): `frame_emptied`
    emptied the frame, deleting its item, and dropped a stand-in (their favourite) on the bed;
    `frame_swap` took the chest's item away and made a new one of its id in the frame,
    deleting the frame's and any enchantment or name. Now the host trades the very stacks
    (`set_frame_item`'s `swap`) and drops what a frame held where asked, or at the frame, never
    deleting it (`drop_at`). ABI 40. Tests: one in `act`. Was: two events that lost a player's
    things.
  - **The endings** (owner: "all three as written"): stage five chooses its ending (the config,
    or the want met and the difficulty) and runs it as a script of beats over the three days
    from its next dawn (`core/src/ending.rs`): it leaves (what it holds back, its torches out,
    its chest on their step, its house gone and a torch where its door was), takes (no sleep,
    the doors open, the frames on the bed, the signs one word, the figure closer each time,
    the tunnels walled, the bell hourly, red rain, the figure walking up to them, on hard one
    death keeping everything), or keeps (its second room, the figure in its window, a path to
    its door, its doorway lit, and a door of theirs open every three weeks after). The want is
    read at last; the lights it places are kept; the world's difficulty is sensed. ABI 41. Where
    it differs from the table is written under it in `entity.md`. Tests: seven in `ending`, the
    stay test now plays the ending and checks nothing follows it. Was: stage five a day of
    nothing, then over.
  - **The bug check, the mind's memory** (four reviewers read the code; each finding
    checked before it was fixed): every observation was taken in on day zero (the mod read a
    default clock), so the habits, the adopted chests, the reactions' ages, how long they were
    gone and `ate_day` never moved; it reads the last tick's clock now. A still-cell west or
    north of the origin was rounded toward zero, the home centred up to five blocks off
    (`div_euclid` now). Once full, the sites, tunnels, routes, still-cells and things carried
    dropped the newcomer at once (a thirteenth site's sample even went to another site); the
    least of the others makes room now. After a restart in multiplayer another player could
    take the target's index; the target and the others are found by uuid. The debug screen's
    stage left an old ending's script behind and `forget_all` lost the uuid (pets never
    matched). The intense and relentless presets deal the hard ending's death on normal, and on
    every difficulty. A new overworld clears what the last said to the Nether and the End.
    Tests: one each in `places`, `tunnels`, `arc`, two in `memory`, one in the stays, the
    death's presets in `ending`. Was: all of that.
  - **The bug check, the brain**: a second `take` while it held something lost the
    first (the host keeps one "held"); it holds one thing at a time now (`HoldsNothing`), and
    what it records taking is the favourite `take` chose (it was the first favourite or the
    first stack, not always what was taken), never from a furnace or a chest not theirs. What
    it held was forgotten when a put of it was sent; now when the put is done, so a full chest
    keeps it held. A plan's hour was read off a default clock: noon never came, and the night's
    events ran by day; the real clock now. A restart left a reply and the house's read waiting
    on the old clock. The two needs that turn on the view (a soul-swappable light, the last
    light) cancelled their plans at a glance instead of waiting. `count_up` counted whatever
    they kill most, not the kind on its sign, and wrote while they looked at it. A plan that made
    nothing was recorded as landed; one within its cooldown at its moment was given up, now it
    waits. Sounds "beside" them played on their left. Tests: two in `act`, one in the brain,
    one in `sounds`. Was: all of that.
  - **The bug check, the host's world**: the hard ending's death kept the
    experience in memory alone (a quit on the death screen lost it) and gave a death held in one
    world back in the next opened; the experience is kept in the world's data now, and what the
    host keeps per player (deaths, pushes, footsteps, bobbers, frames shown, approaches,
    charges, loot offerings) is forgotten as a world closes. A sale of what it took was guarded
    in memory alone: after a restart or an unloaded chunk a restock sold the very thing again;
    the villager is marked and a saved sale watched again, taken away once sold, and a villager
    killed before it sold drops the thing. `equip_entity` lost the kept stack when the slot was
    worn; a temporary entity went with what it wore or carried (a saddle, a donkey's chest,
    the player's stack); a spared hit could kill on hard (the game scales it after); gathered
    villagers saved in the house stayed there, and a big village pushed them into the walls; a
    block moved past the world's top was lost with its contents; a warped mob held too froze
    for good; a figure, its tab entry and the rock a figure dug stayed on the client after a
    death or a dimension change; a loot swap with nowhere to keep the item overwrote it; an
    enderman's own block was overwritten; `stand_dress` re-made their armour from its id,
    losing enchantments and names. All fixed. A take without a keep spends the item by design
    (a sapling planted), now said so. Tests: one in JUnit (`Spare`). Was: all of that.
  - **The bug check, the senses and the client**: the JVM's shutdown hook unloaded
    the mods while the server was still to save (a Ctrl-C on a dedicated server could lose the
    stay's last state and skip the world's own saves); it waits for the worlds to close now, and
    a close never throws out of the server's stopping. The senses kept a closed level alive and
    a player's state across leaving; a portal or a teleport read as fleeing at a hundred blocks
    a second (the movement starts afresh after a jump); leaving the End, or respawning in
    another dimension, was never a crossing (the observation reports any change now, and
    leaving the End is no death); a spectator's click unpacked loot, and one use was heard for
    each hand; NeoForge counted a hoe's farmland or a stripped log as their building; one
    looked at behind a wall counted. On the client a field-of-view pull could be saved for good
    by the options, a shake's saturation was put on the next world's player, a settings press
    of the wrong shape threw on the render thread, a refused settings change stayed in the
    file, each download try left an HTTP client open, and red rain turned the snow red. All
    fixed. Tests: one in JUnit (`Setting`). Not fixed: the `mods` observation is sent once a
    run, but nothing reads it yet (fixed below). Was: all of that.
  - **The two left from the bug check** (owner: "do fix these too"): the enderman's block came
    from nothing; now it is one of its own, from what it dug, and spent at home, and with none to
    give no enderman holds one. The other mods loaded were told to one level once a run and read
    by nothing; every level is told once now, the memory keeps them and the debug screen shows
    them (a gauge, `mods`). Tests: `elsewhere`'s enderman test, one in `memory`. Was: both.
  - **The settings group `How scary` is `What it does`** (owner: "rename that"). Was: How scary.
  - **`model.pause_ms`** (owner: "is there a way to limit the amount of compute it uses"): the
    rest between each piece of the model's work, 1 ms by default, up to 100, on the settings
    page; higher leaves more of the GPU and CPU to the game. Tests: the config's refusals. Was:
    a fixed millisecond between chunks.
  - **An icon** (owner: "generate an icon for modrinth", and in the jar too): a door left ajar at
    night, two pale eyes in the gap, pixel art drawn by `mod/art/icon.py` (no image library);
    `mod/art/icon-512.png` for Modrinth, `assets/lodger/icon.png` (128) in both jars, named by
    `fabric.mod.json`'s `icon` and `neoforge.mods.toml`'s `logoFile`. Was: no icon.
  - **The debug panel teleports on Shift+click only** (a stray click put the owner deep in a
    cave). Was: any click on a spot.
  - **No ask for a menu of one:** the model was asked every interval with `wait` the only
    choice. Was: asked, and logged.

### Phase 4: the model in the loop

- [x] **Wire the brain to the worker.** Done: the two planning questions go to the
  worker with the key `plan` and the answer is read on a later tick; `brain.event_threshold`
  and `nothing yet` make `wait`; a refusal, a minute's silence or `brain.force_rules` hands the
  menu to the rule brain; the story and the log say which brain decided. Tests with a fake
  scorer: the answer names the event, `nothing yet` waits, a shut-down model and
  `force_rules` fall back. Was: the asks with their keys, the callbacks in the tick, the
  thresholds, the fall-back. Still to do: the `react` asks on triggers, `When?` and `Where?`.
- [x] **The want and the target asks,** once each at move-in. Done: `want` asked
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
- [x] **Calibration, round one, first pass.** Done: `core/tests/calibration.rs`
  (ignored; three scripted days against the real 1.5B with every decision printed) and
  [round one](calibration/round-1.md): seventeen decisions all by the model, thirteen landed,
  the want and the moods read; what to move in round two (the first planning question barely
  discriminates, low-confidence picks still happen, moods at low confidence are noise, no
  written line was exercised). Was: ten stays through stage two by different players, the log
  read end to end, the prompts and thresholds moved. Still to do: round two as the page says,
  on the owner's dev stays with the real model.
- [x] **`brain.text = generate`, the model half** (owner). Done:
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
- [x] **Chat and the player list** (owner): the chat sense (the target's lines and
  witnesses' within 32 blocks, matched for lines about the lodger, eight kept) and the events
  of [Chat and the player list](entity.md#chat-and-the-player-list), as host commands: a
  player-list entry to one client, a system or chat line to one client in a given name, an
  advancement toast. All target-only, all behind `events.chat`. Found done in the audit: chat lines in `Observations`, `tab_entry`, `name_called` and the advancement toast.
- [x] **Other mods** (owner): `Observations.mods`, the presence checks, and the
  first three hooks (dynamic lights, a minimap marker, Presence Footsteps' sound set), each
  tested with and without the mod in the dev client. The rest of [Other mods](ideas.md#other-mods)
  as people ask. Found done in the audit: `Observations.mods`, the presence checks, and room for other horror mods, their storage and their decorations (owner). The three hooks named here (dynamic lights, a minimap marker, Presence Footsteps) were not built.
- [ ] **Chat template or not.** Score the real-model test set with Qwen's chat template around
  the situation and without, and keep whichever separates the cases more. Write the result into
  [Brain](brain.md#the-situation-text).
- [ ] **Qwen3-0.6B.** Same test set, same question. Both are Apache-2.0; if Qwen3 reads better
  and is not slower, change the default URL and [Model](model.md#which-model).

### Phase 5: the figures, stages three and four

- [x] **The figure, first cut**. `Figures.java`: a figure is packets to the
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
- [x] **The far presence** (`far_figure`): a server entity in the target's skin with the name
  shown, placed by line of sight from the route or home, gone when approached. The client
  renderer, the skin from the target's profile. Witnesses' clients never receive it. Found done in the audit: `far_figure`, a figure in the target's skin seen by the target alone (`Figures.java`).
- [x] **The replay figure** (`replay`): walks a recorded route at the recorded pace, rounds a
  corner out of sight when approached, leaves the held item. The route's points need a path the
  figure can walk; where the world changed, cut the route there. Found done in the audit: `replay`, the figure along a recorded route with the held item.
- [x] **Executors for the stage three and four events** of the catalogue, about sixty: the beds,
  the house's block moves, the villages, the water, the mines, the redstone, the animals, the
  late figures, with game tests, and the new general commands they need (dig a line at a
  player's pace, drive an animal to a spot, set a villager's trade, mark a map). Found done in the audit: the stage three and four events have executors; their game tests are the game-tests item.
- [x] **Like a player** (owner): `mining_sounds`, `mine_beside`, `tunnel_toward`,
  `tools_in_hand`, `mined_out`, from [Like a player](entity.md#like-a-player). The digging
  executor is one general command, "dig this line of natural blocks at a player's pace with
  these sounds", with the rule checks (nothing placed within 8 blocks, no cave, water or lava
  ahead) in `fits` and re-checked per block, since the target moves while it digs. The memory
  learns the target's tunnel sizes and torch spacing (the senses already see blocks broken and
  placed). `tunnel_toward` is the showpiece: it needs the target's current digging direction and
  a slow approach over minutes, so it is a plan the mind re-checks every few seconds, not one
  command. Found done in the audit: all five are events with executors.
- [x] **The glitches** of [The sky and the light](entity.md#the-sky-and-the-light) and
  [The screen](entity.md#the-screen): the hint packet with a kind, a place and a duration, and a
  client renderer for each: light-level overrides for a room, a painting drawn rotated, the sky's
  sun, moon and stars and colour for a moment, rain and snow particles and sky tint, the
  compass's needle, footprint decals on snow and sand, a reflection drawn wrong, and the HUD's
  hotbar, hunger, hearts, subtitle, crosshair, player list and sleep count, the camera's nudge
  and the field of view. Each ends on its own; none touches the server's world. Tested on the
  client with a screenshot test per glitch against a fixed scene. Found done in the audit: `client/Glitches.java` has twenty kinds; there are no screenshot tests.
- [x] **Looked-at detection** (`world/Gaze.java`, ABI 47): each figure shown, while it stands,
  counts the time they face it within ten degrees in looks of half a second or more, how many
  looks, and how far they walk toward it; as it goes the host reports `figure_seen` with that and
  how it ended (approached, looked away from, its time up, they left). The mind keeps the last
  sixteen (`Memory::figure_looks`), with looked at long (two seconds) and chased (eight blocks
  toward it, or near enough that it went) for temper and the chains, and the situation says the
  last. Tests: three in JUnit, one in `memory`. Was: **Looked-at detection** in the senses: facing within 10 degrees for half a second, how   long, whether moved toward. Partly, found in the audit: the senses send what each player faces and which player a mob faces; how long and whether moved toward are not measured.
- [ ] **Play it** through stage four, with the model. Calibration round two, same method.

### Phase 6: the ending, multiplayer, release

- [x] **`settle`** and the three endings as three-day scripts (owner: "the endgame
  should be very grand"), each day's beats as data in `arc.rs` with a time, a condition and two
  forms the model picks between, the budget and cooldowns off and the caps on, the chains
  closed inside them, the one death on hard with inventory kept and the vanilla message, and
  the `left` and `stayed` states. Scripted-stay tests for each ending, checking the order of
  beats, that nothing repeats within a day, and that the death happens once or not at all. Found done in the audit: `ending.rs` holds the three endings as beat scripts, and the confrontation besides.
- [x] **Temper** (owner: "doing certain things should speed up the progression"; "harder fight
  when angry"): `core/src/temper.rs`, kept in the memory, settled each tick toward the preset's
  resting value, moved by the target's acts as the table in [Provocation and
  temper](entity.md#provocation-and-temper) says (what is coded and what is not is there), and read
  by the budget, the stages' lengths, the bold events' stages, the distance, the ending and the
  fight (`fierce`, ABI 48); the situation says it; `events.temper` turns it off. Tests: six in
  `temper` (resting and drift, the daily cap, the pace each way, the words, chat, a provoked
  `calm` under a resting `balanced`) and one scripted stay (a provoking player reaches stage four
  a day or more sooner, measured to the hour; a quiet stay now runs at its least days, so
  halving them gives a day and a half). Was: **Temper** (owner: "doing certain things should speed up the progression"):   `memory/temper.rs` with the move table of [Provocation and temper](entity.md#provocation-and-temper),   the senses that feed it (the figure chased or hit, its blocks broken, its place walled or   lit, the thing taken back, chat lines at it, nights slept through), the scaling of the   budget, the stage lengths, the menu's stage gates, the distance and the ending, each bounded   by the preset, and the `How does it take this?` ask. Scripted-stay tests: a provoking player   reaches stage four days sooner than a quiet one; a `calm` preset provoked stays under the   `balanced` resting pace; temper drifts back.
- [x] **The other house, a dimension of its own** (owner: "Dimension, as designed"):
  `data/lodger/dimension/other.json` and its type in the content jar (a void, a black sky, no
  mobs, no day, no sleep), `world/OtherHouse.java` copying the house around the bed block for
  block with its lights left out, an unseen barrier floor running on outward, the figure in the
  copy's doorway, and home to their own bed, the door open, after the time (ABI 49,
  `other_house`). Once in stage four while they sleep (`other_house`, out of `calm` with the
  other sleep events), and again on the take ending's last night, the figure walking at them; the
  death stays at its place in their own world. A player left there by a quit or a stop is sent
  home. Seen loading on the Fabric dev server. Tests: one in `act`. Was: **The other house, a dimension of its own** (owner: "custom dimensions you   can get teleported to by sleeping, walking through portals"): a void dimension in the   content jar's data pack (`data/lodger/dimension/other.json`, a flat dark sky, no mobs, no   day), into which the host copies the target's house block for block at stage four, and one   visit: the target falls asleep and wakes in the copy, in the dark, the doors leading to   nothing, the figure in the doorway, for one minute, then wakes in their own bed with the door   open. A second visit only in the `take` ending's last night, where the walk from its place   goes through its portal frame into the other house and the death, if any, is there. Not a   place to explore: a set piece, twice at most, so the world stays the player's. Needs the   host's teleport and block-copy commands and a dimension registered from the content jar;   judged in the dev client before the design goes further.
- [x] **Chains** (owner: "one that influences the next"): `core/src/chains.rs`, the eight as
  data with the catalogue's events (what is coded, and how it differs, is in
  [Chains](entity.md#chains)), `Memory::chains`, the situation's line for each open one, the
  next step weighed up four times on the menu and in the rule brain's pick, a step's days held
  while its stage has not come, once a stay each; the debug screen's Chains section. Tests:
  three in `chains` and a balanced scripted stay that walks the place end to end (the scripted
  player notices nothing and keeps no pet, so one chain, not two, can run there). Was: **Chains** (owner: "one that influences the next"): follow-ups in `events.rs`   (weighted or exclusive, with a carried place or item), `Memory::chains`, the situation line,   and the weights in `offer.rs`. The eight chains of [Chains](entity.md#chains) as data, with a   scripted-stay test that walks at least two end to end and one that no step is offered with its   needs unmet.
- [x] **Multiplayer** (`core/src/multiplayer.rs`): the target picked as before (the model among
  several, or the first; `Memory::others`), its last sighting kept by the real clock, a change of
  target once a stay at a stage's beginning when it has been gone `multiplayer.absent_days` and
  someone else is here (its want, place, house and doings kept, the old target's home and habits
  let go), and with `multiplayer.witness_traces` off every change to the world near another
  player held back (`Refused::Witness`, a wait). The debug panel's players line. Not tried with
  two dev clients yet. Tests: two in `multiplayer`. Was: **Multiplayer:** the `Others` memory, the target ask, witnesses' traces and nothing else,   retargeting once. Tested with two dev clients on a dev server.
- [x] **Performance pass:** the per-tick JSON exchange measured (it should be well under 50 µs
  on a quiet tick), the senses' distance checks bounded to the target's chunk neighbourhood,
  the model's unload timer. Nothing the mod does should show in a tick profile on a server
  with the lodger idle. Found done in the audit: `the_tick_costs` in `tests/stays.rs`, and the model unloads when no one plays.
- [ ] **A proper release:** the jar with three natives, `THIRD_PARTY_NOTICES.txt`, the
  README's install steps checked on a fresh machine on each platform, a Modrinth page that says
  what the mod records and downloads. Version `0.1.0`. Partly, found in the audit: `scripts/build_jars.py` makes both jars with six natives and the notices, and `mod/modrinth.md` is written. Still open: the install steps checked on fresh machines.

### After every phase

- [x] **A debug UI for events, senses and the brain** (owner). Done,
  earlier than planned, as the debug screen above (phase 2's menu item): the senses as meters,
  the memory's lines, the plan and the last decisions, every event with a Fire button, the
  world's reaction in the Log tab. Was: once the phases are in, a proper in-game screen over the
  menu's debug page, with the open chains and the temper and a want button. Still to do as the
  mind grows: the model's menus and confidences per decision (phase 4), the chains and the
  temper, the want.
- [ ] **A Minecraft version bump, rehearsed:** take the host to the next snapshot once, by the
  process in [Upgrading](upgrading.md) (written at the owner's ask), and fix
  whatever it missed; add the bump to its history table.
- [x] **The surroundings in the mind** (owner: "you're also detecting structures
  nearby and have code to determine where their house is?"). Done: the brain asks
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

- [x] **GPU natives** (owner: "one native per os per arch", GPU always): in the jar, not
  separate, see the five natives below. Was: `metal` and `vulkan` as separate downloads, if
  anyone asked.
- [ ] **A second lodger kind** with a different want and events, once the first has been played by
  people other than the author. The data-first events and the staged arc are there to make this
  cheap.
- [ ] **Sound design pass** with a composer: the late steps, the door, the music cut. Everything
  in the first release is vanilla's own sounds re-timed.
- [ ] **Publishing the SDK.** The crate to crates.io under its final name and the host jars to
  Modrinth, once a second mod has been written against it and the protocol has stopped moving
  for it. Until then both live here. The crate name is the owner's to pick; `craftkit` was free
 .
- [ ] **The host checks that content jars match across sides.** Today a client without the
  lodger's jar joins a server with it, and sees missing textures where the figure would be. The
  host should compare the content jars' ids and versions in the handshake and say which is
  missing.

## Thought experiments

Things considered and set aside, kept so they are not reconsidered from scratch.

- **Letting the model write the signs.** Set aside at first: three words from a 0.5B model
  about a player's chest are three words of nonsense a third of the time, and once it writes one
  thing a player finds cruel the mod is over. Reopened by the owner: the model
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

The `[x]` lines above, summed up by day:

- **Day one:** the docs, from `~/chainv4`'s style and `~/engine`'s AI layer.
- **Day two:** phases 0 to 2 (the SDK, the ABI, the host for both loaders, the content jars,
  the download with the license screen, the model layer), phase 3's host side (the senses, the
  executors), the mind's first and second cuts (the memory, 87 events, the rules, the arc, the
  rule brain, the log, scripted stays), the debug screen, the glitch renderers, the kill tally,
  the landmarks, the text banks, the upgrade guide. First seen in a dev client the same day.

## Next (owner's answers)

- [x] **Arrows** (owner: "near misses, then hits"): stage 3 a real arrow from the dark into the
  block beside their head, from where the figure stands with a bow; stage 4 it hits by the harm
  stages (a heart at 4, two at 5), never a kill; the arrows stay stuck where they land.
- [x] **Potions** (owner: all four): a splash from the dark at their feet (darkness, slowness,
  blindness, a moment's levitation; never damage, poison or wither); its brew in their chest,
  named `for the night`; one of their potions turned another kind (never a harmful one), its old
  name kept; a lingering cloud at their door at dawn that fades as they step out.
  Done: seven rows (`arrow_near`, `arrow_grazes`, `arrow_hits`, `splash_dark`, `brew_left`,
  `potion_changed`, `door_cloud`), five commands (ABI 42), the host keeping to what does not
  harm whatever it is asked. Tests: one in `act`. Was: the arrow a sound alone, no potions.
- [x] **Other mods** (owner: all three): with another horror mod loaded it acts less often and
  never at the same moment; it takes from other mods' storage too; it moves other mods'
  decorations as it does vanilla ones.
  Done: a mod whose id reads as a horror mod's (`cave_dweller`, `the_man_from_the_fog`, a
  stalker, a mimic, ...: `memory::HORROR_WORDS`) halves the day's budget, and while one of its
  things is within 64 blocks of the target no plan runs and nothing starts; the house read finds
  every mod's chests and barrels and lamps; another mod's lamp is a light, its chairs, tables,
  shelves and cabinets furniture, moved first by `shift_furniture`. A mod's chest the target
  opens was already theirs to the memory. Tests: three in `memory`, one in `arc`. Was: no other
  mod known.
- [x] **The confrontation, a fourth ending** (owner: "a fourth ending: you confront it"): in
  stage 5, at night, at its house (or its place), strike its door or the figure; without that,
  one of the three scripted endings plays. Three phases, one health bar named after them: its
  warped animals in waves while it watches from its roof; the duel in their skin with a copy of
  their best gear (strafes, blocks, sprint-hits, pearls), splitting at half health into three to
  five copies, one real; the copies pulled together into a big thing made of what it took,
  dropping their things as it breaks. Scaled to their gear and the difficulty. Win: everything
  back and all three trophies (its lantern, a soul lantern named `<name>'s light`; its head in
  their skin, named `the lodger`; its weapon, named, with an enchantment it learned). Lose: one
  death on any difficulty, everything kept, it keeps what it took and leaves (no animal waves on
  peaceful). A real beacon in its house from stage 5's first night, lit only at night, to lead
  them there.
  The fight's shape (owner: "rise out of the ground ... deforming and bending its
  limbs, turning grotesque, in between stages"; the sequence below confirmed as is): the rising
  (the ground before its door cracks, it rises lying face up over three seconds and bends
  upright at the waist, the bar filling); stage 1 waves, it watching from its roof; bend 1 (it
  drops onto all fours, crawls at them flat and fast, stands too tall, its head backwards a
  beat); stage 2 the duel (twitching a block aside and back, upside down a second at a time);
  bend 2 at three fifths (it freezes and shakes, copies peel off it and walk out, heads
  backwards); stage 2b the copies; bend 3 at seven twentieths (the copies crawl together and
  sink, its stored blocks rise and snap together into a body, their sword last); stage 3 the
  construct (elbows and knees bending the wrong way, slams and sweeps, at each third an arm
  torn off grown back as two, the torso stretching, the head twisting round); the end (it falls
  apart, its blocks dropping as items, the figure standing where it was, looking at them,
  sinking into the ground; the lantern, the head, the weapon).
  Done: the mind starts it (`ending::confrontation`: stage five, night, a knock on its door or
  standing at its place; the debug screen's `Fight it now` anywhere), pauses the script while
  it is on, and ends the stay on its answer (`beaten`, `kept`; run from, it can be come to
  again); its beacon is built on its roof at stage five's first dusk, capped by day and lit at
  dusk, and goes with its house when it leaves. The host runs the fight (`Fights`, `confront`,
  ABI 43): a 26.3 mannequin in their skin and a copy of their gear, its own health kept apart
  from the mannequin's so it never dies as an entity, the construct's limbs block displays posed
  each tick. Tests: two in `ending`. Not yet seen in the game: the first launch checks it.
- [x] **The fight, from the owner's first try**: the wave's animals gave up after
  forty seconds (a wait in the water and they were let be); they are driven at the player every
  half second now, for as long as they live. The figure could be hit while the animals came; it
  cannot, until it fights (the rising, the waves and the bends are out of reach). Four waves of six
  to ten, with a zombie, a skeleton and a creeper (two in the later waves; the creeper's blast
  hurts and breaks nothing, the hostiles wear helmets against the sun and drop none), of every
  size from small to twice their own, some upside down with no name over them (a team that shows
  none), each with one to three limbs of the wrong stuff and length and some with a face that is
  not theirs, the player's own among them. Run past 112 blocks and it sinks where it stands to
  wait for another night, told as they near 80. From the second review: the construct's shield
  could stay up for good; it rose on the house, not before the door; the lethal check ignored a
  shield, a totem, Protection and Resistance; any harm (a cactus, a fall) came off its health,
  now their blows alone; the trophy weapon was a whole copy of theirs, now a new one with one
  enchantment; a death to anything else during the fight lost their things, now it is the one
  death that keeps everything (both loaders' death hooks); a logout told the mind nothing; the
  arrow's aim was a second stale and it vanished after a reload; a brew could carry levitation
  for ten minutes (brews keep to a short list now); a potion could be "changed" into itself. In
  the mind: its place was never chosen (it is, at move-in); stage five's budget was 0 in every
  preset, so its rows never ran (1, 3, 4, 6 now); the fight's clock was saved across restarts;
  the beacon stayed lit for good after a fight; a knock on stage five's first night was not
  heard; the mods list cut at 64; another horror mod's thing near stayed "near" for good; the
  debug screen's fight and stage left an old script and beacon behind. Tests: the ending's,
  memory's and config's updated. Was: all of that.
- [x] **The warped animals, their own models** (owner: "the blocks look kinda silly
  ... stretching the actual model"): the block limbs are gone; the server names each wave's
  animals in a `warp_models` hint and the client bends their own models as they are posed for
  drawing (two render hooks, `LivingEntityRenderer.extractRenderState` for the entity's id and
  `ModelFeatureRenderer.prepareModel` after `setupAnim`): every part stretched or squashed along
  its length and across, and turned up to 0.8 radians wrong, by a fixed draw so it never
  flickers, the whole body tall and thin or low and wide; a third still wear a face (theirs,
  now and then). Skeletons have their bows (a mob made by the mod gets none from the game), and
  the waves no longer pile up: the next comes once at most two of the last are left and after a
  breath, a wave of a minute and a half has its last sink away (until now one came every fifty
  seconds on top of the last). Tests: one in JUnit (`Warp`). Was: block limbs, bowless
  skeletons, piled waves.
- [x] **Intel Macs too** (owner: "you can include intel macos in the build/ci, i kinda forgot those
  even existed"): `macos-x86_64` back, cross-built on the Apple chip (`x86_64-apple-darwin`, Metal
  linked), so six natives; checked to load and answer under Rosetta. The host's platforms, the
  scripts, the CI and the docs say six. Tests: JUnit's platform count. Was: five, no Intel Mac.
- [x] **The download screen: a note per size and Play without it** (owner: "a small text thing
  explaining the difference between the models, and the recommended vram"; "play without it
  should also be changeable via the ingame settings"): under the size button, how the size
  chosen differs and the VRAM it wants (`Choice.note`); **Play without it**, where a resource
  names the change that does it (`Resource.without`, ABI 46), writes `model.enabled: false`
  (new): nothing downloaded, the part file kept, the rule brain deciding, the status ready. The
  settings' **Use the model** turns it back on and the screen returns. The screen laid out
  again so the progress line no longer sits under the size button. Tests: one in the mind, one
  in JUnit. Was: every client had to download the model to reach the title screen, even one
  that only joins servers, and the sizes said only their names and megabytes.
- [x] **Five natives, the GPU always on, cross-compiled and compressed** (owner: "one native for
  macos, then 2 natives for linux (amd64, arm) and 2 for windows"; "use compression when
  bundling the natives"; "make the ci workflow manually triggered"; "write docs/build.md"):
  `macos-aarch64` with Metal, `windows-x86_64`, `windows-aarch64`, `linux-x86_64` and
  `linux-aarch64` with Vulkan; no Intel Mac. `scripts/build_natives.py` builds any of them from a
  Mac (Windows with cargo-xwin and clang-cl against a Vulkan "SDK" it makes from Khronos's
  headers and an import library from `llvm-dlltool`; Linux in `scripts/cross/linux.Dockerfile`,
  Debian 12) and checks each: the ABI exported; Windows importing only system DLLs and
  `vulkan-1.dll`, the C++ runtime linked in (`.cargo/config.toml`), so no Redistributable; Linux
  asking for glibc 2.34 at most. `scripts/build_jars.py` builds them (`--natives cross`, the
  default) and packs each as `natives/<os>-<arch>/<library>.xz`, about half the jar's own
  deflate; the host unpacks it with XZ for Java (bundled: Fabric's `include`, NeoForge's
  `jarJar`) and takes a plain one too. A native that will not load for want of Vulkan's loader
  says what to install (`apt install libvulkan1` on a headless server). The CI runs by hand and
  builds them by the same scripts, then the jars, then a GitHub release of the five natives
  (named by platform), both jars and `SHA256SUMS`, its notes listing every SHA-256 (owner: "list
  the sha256 of every individual native and bundled jar"; "create a github release"). `model.gpu_share` now defaults to 1, the whole
  model on the GPU (owner). The notices name the Vulkan headers, XZ for Java and Microsoft's
  runtime. Tests: JUnit for the xz unpacking and the Vulkan hint; each native checked as it is
  built. Was: six CPU natives each built on its own CI runner on every push, packed as they
  were, `gpu_share` 0.5, and no `docs/build.md`.
- [x] **The books it writes** (owner: "an event where villager offers a book for sale that has
  been written by the model"; "pay at first, then in later stages they drop it, if not purchased
  already"; "up to you to create more based of that"; and all four of the others asked about):
  `diary_sold` (a villager near them sells its diary of them for an emerald, titled and signed with
  their name: a line of the model's, then what it has seen them do, every fact the memory's own),
  `diary_brought` (not bought: a villager walks up and drops a newer one at their feet; the
  `the_diary` chain), `letter_in_chest`, `villager_named`, `lectern_notes` (a page a day, the
  latest first) and `page_in_your_book`. `core/src/books.rs` writes the pages, `Kind::DiaryLine`
  the model's lines (twelve words at most, the bank's when it has none). The host: `write_book`,
  `add_page`, `put_book`, `approach`'s `give` (kept again if it never arrives) and a `bought`
  event for its sales (ABI 51). Tests: two in `books`, one in `act` for the diary's commands and
  its purchase remembered; the chain stay test now runs three stays and wants the place walked
  in two (one seed's luck had decided it, and the new events moved it). Was: no such events.
- [x] **A spot with no footing skipped, not the whole path; the Windows checks link** (owner: "what
  was the set_block error from?"; "are there other events where the safety rule might be a bit too
  blunt?"; "the ci is erroring during the windows build"): the events whose commands are spots of
  their own (`light_path`, `ice_path`, `village_doors`, `candles_lit`, `torches_far_end`,
  `lights_out`, `face_the_house`, `hostiles_stare`, `frames_turn`) skip a spot that fails and go
  on, done if any spot was (`SPOT_BY_SPOT`), and the path's torches look eight blocks up and down
  for footing; the CI's Windows checks use the dynamic C runtime throughout (llama.cpp's MSVC
  build kept it whatever it was asked, and the static one left its imports unresolved; set through
  `RUSTFLAGS`, which stands in for the config's, where a per-target variable was merged with it). Tests: one
  in `brain`. Was: one torch over water stopped a path of seven, and the Windows checks failed to
  link.
- [x] **A line on the debug screen** (`docs/events_todo.md`, 40): `f3_line`, stage 5, a hint of
  half an hour: one more line on their F3 screen after where they stand, "Lodger: watching"
  (`DebugEntryPositionMixin`). A screen glitch, off with `events.glitches`. Tests: the
  catalogue's own. Was: no such event.
- [x] **Only in third person** (`docs/events_todo.md`, 39): `only_in_third_person`, stages 4 and
  5, a hint of ten minutes: in third person a figure in their skin right behind them, moving as
  they move, nothing in first person (`BehindFigure`, the client's own figure with its own player
  entry, `ClientFigure`, shared with the screenshot). A screen glitch, off with
  `events.glitches`. Tests: the catalogue's own. Was: no such event.
- [x] **In the screenshot** (`docs/events_todo.md`, 38): `in_the_screenshot`, stage 5, a hint of
  half an hour: their next screenshot is held back a tick, a figure in their skin set fourteen
  blocks ahead in their view on their client alone (its own player entry, so their skin), the
  picture taken with it in, and the figure gone; once (`ScreenshotFigure`, `ScreenshotMixin`).
  A screen glitch, off with `events.glitches`. Ahead, not behind them: a first-person picture
  never shows behind. Tests: the catalogue's own. Was: no such event.
- [x] **Handprints** (`docs/events_todo.md`, 37): `handprints`, stages 4 and 5, coming home with a
  window known: a hint of a minute and a half drawing two hands of pale dust on a window's glass,
  across the way they look, renewed as it fades (`HandPrint`). A screen glitch, off with
  `events.glitches`. Tests: one in `act`, `HandPrintTest`. Was: no such event.
- [x] **The chicken lays it** (`docs/events_todo.md`, 36): `chicken_lays`, stages 3 to 5, holding
  something of theirs with a chicken near them: it lays the very thing, with an egg's sound
  (`drop_item` of what it holds; done, it holds it no more). Tests: one in `act`. Was: no such
  event.
- [x] **A firework over its house** (`docs/events_todo.md`, 35): `firework_over_its_house`,
  stages 4 and 5, at night near its house: a firework from its roof bursting high as a pale face,
  the game's creeper shape (`firework`, ABI 54). Not their skin's colours, which the game never
  says. Tests: one in `act`. Was: no such event.
- [x] **It waits at the portal** (`docs/events_todo.md`, 34): `waits_at_the_portal`, stages 4
  and 5, as they come back from another dimension (the brain, `PORTAL_CHANCE`, never from the
  menu): a figure three blocks beside where they arrive, facing the portal, gone once seen and
  looked away from. Tests: one in `act`. Was: no such event.
- [x] **Someone on the map** (`docs/events_todo.md`, 33): `figure_on_map`, stages 4 and 5, away
  from home holding a map: a player's marker at their house on it, five minutes, for them alone
  (`map_mark`, ABI 54: the host's `MapMarks` sends the map's own markers again with it a few times
  a second). Tests: one in `act`, `MapSpotTest`. Was: no such event.
- [x] **All freeze** (`docs/events_todo.md`, 32): `all_freeze`, stages 4 and 5, animals near them
  outdoors: every animal, villager and mob within twenty-four blocks stops at once three seconds,
  facing them (`hold` on each). Tests: one in `act`. Was: no such event.
- [x] **Held breath** (`docs/events_todo.md`, 31): `held_breath`, stages 4 and 5, under water
  (the memory keeps their air now): their air runs out twice as fast for twenty seconds
  (`drain_air`, ABI 54, the host's `Breaths`). Tests: one in `act`. Was: no such event.
- [x] **An animal follows** (`docs/events_todo.md`, 30): `animal_follows`, stages 3 to 5: the
  farm animal of theirs nearest them follows them a whole day, stopping six blocks off when they
  stop (an `approach` renewed every eighty seconds, a walk lasting ninety at most). Tests: one in
  `act`. Was: no such event.
- [x] **It wears your face** (`docs/events_todo.md`, 29): `wears_your_face`, stages 4 and 5, at
  night outdoors: the zombie nearest them wears their own head five minutes, never dropping it,
  and walks at them (`wear_face`, ABI 54: the host's `Masks`, what it wore put back after, and
  taken off as the world closes). Tests: one in `act`. Was: no such event.
- [x] **A head above the ground** (`docs/events_todo.md`, 28): `peeks_from_ground`, stages 4
  and 5, outdoors: twenty-four blocks ahead a figure sunk a block and a half into the ground, its
  head alone above it; once looked at, it sinks slowly out of sight (`figure`'s `sunk`, ABI 54).
  Tests: one in `act`. Was: no such event.
- [x] **In a line** (`docs/events_todo.md`, 27): `in_a_line`, stages 4 and 5, at night outdoors
  with their route known: figures in single file along it ahead of them, eight blocks apart, six
  at most, all facing the house, each gone within four blocks (`figure`'s `faces`, ABI 54).
  Tests: one in `act`. Was: no such event.
- [x] **Crossing ahead** (`docs/events_todo.md`, 26): `crossing_ahead`, stages 3 to 5, walking
  outdoors: thirty blocks ahead a figure walks across their way from their left to their right,
  twenty blocks, facing its way and never them, gone at the far side (`figure`'s `walks`, ABI 54,
  a glide a point a block apart on the ground). Tests: one in `act`. Was: no such event.
- [x] **The eye turns home** (`docs/events_todo.md`, 25): `eye_turns_home`, stage 5, once a
  stay, holding an eye of ender: the next one they throw within ten minutes flies toward their
  house instead of the stronghold (`eye_home`, ABI 54; the host's `Eyes` turns an eye in its
  first ticks near them). Tests: one in `act`. Was: no such event.
- [x] **A note in the ender chest** (`docs/events_todo.md`, 24): `ender_chest_note`, stages 4
  and 5, an ender chest of theirs known (a fixture now): a written book in its words into their
  own ender chest's inventory (`put_ender_chest`, ABI 54, kept again when full). Tests: one in
  `act`. Was: no such event.
- [x] **Respawn in its bed** (`docs/events_todo.md`, 23): `respawn_its_bed`, stage 5, once a
  stay, its house built: a bed laid along its house's left wall, then made their respawn point
  quietly (`set_respawn`, ABI 54); its lanterns (`its_lights_on`) moved clear of it. Tests: one in
  `act`. Was: no such event.
- [x] **A false death** (`docs/events_todo.md`, 22): `false_death`, stage 5, walking outdoors,
  once a stay: a hint of a second and a fifth drawing the death screen over the game as the game
  draws it (its wash, its title twice the size, their score), no buttons, nothing died. A screen
  glitch, off with `events.glitches`. Tests: the catalogue's own. Was: no such event.
- [x] **The inventory stare** (`docs/events_todo.md`, 21): `inventory_stare`, stages 4 and 5, a
  hint of ten minutes: in their inventory their own figure stares straight out at them, drawn as
  if the mouse were at its face (`InventoryScreenMixin`). A screen glitch, off with
  `events.glitches`. Tests: the catalogue's own. Was: no such event.
- [x] **It lies beside the bed** (`docs/events_todo.md`, 20): `lies_beside`, stage 5, asleep at
  night: the bedside walk of `choke`, then it lies down on the floor beside the bed on its side,
  facing them, nothing more, until they wake as it ends (`choke`'s `how` `lie`, ABI 54); in the
  sleep events, so `calm` leaves it out. Tests: one in `act`. Was: no such event.
- [x] **The pet follows** (`docs/events_todo.md`, 19): `pet_follows`, stages 4 and 5, a pet near
  them outdoors: a figure nine blocks ahead, their pet walks to it, the figure gone and the pet
  back to them of itself (`approach`'s `to`, ABI 54; nine blocks, short of the twelve at which
  the game brings a pet back). Tests: one in `act`. Was: no such event.
- [x] **Falls from the sky** (`docs/events_todo.md`, 18): `falls_from_the_sky`, stage 5, at
  night outdoors looking up (the memory keeps their pitch now): a figure falls from forty blocks
  over the ground forty blocks ahead, lands with a heavy fall's thud and is gone there
  (`figure`'s `falls`, ABI 54). Tests: one in `act`. Was: no such event.
- [x] **A face in the chest** (`docs/events_todo.md`, 17): `a_face_in_the_chest`, stages 3 to
  5, the target away or asleep: their chest's grid laid out as two eyes and a mouth of single
  items from their biggest stack, what sat there moved aside first, nothing lost or added
  (`act::face_moves`). Tests: one in `act`. Was: no such event.
- [x] **Its lights on** (`docs/events_todo.md`, 16): `its_lights_on`, stages 4 and 5, at night
  eight to forty-eight blocks from its house built whole (`Memory::its_house`): its wall torch
  taken away a while, three lanterns lit on its floor one by one, then all out, its lanterns
  only (`building::house_lights`). Tests: one in `act`. Was: no such event.
- [x] **Pulled under** (`docs/events_todo.md`, 15): `pulled_under`, stages 4 and 5, swimming
  (the memory keeps it now): pulled down twice a moment apart, a whirl of water and a drowning
  gasp, no harm (`push_player`, ABI 54, a block a tick at most). Tests: one in `act`. Was: no
  such event.
- [x] **At the top of the ladder** (`docs/events_todo.md`, 14): `top_of_the_ladder`, stages 4
  and 5, down a shaft of their ladders six blocks or more below its top (`act::shaft_top`): a
  figure at its mouth looking down at them, gone when they climb within six blocks. Tests: one
  in `act`. Was: no such event.
- [x] **The offer** (`docs/events_todo.md`, 13): `the_offer`, stages 4 and 5, holding something
  of theirs, a door known, the target outdoors: a figure in their doorway holds it out; come
  within three blocks and it is gone and the very thing drops at their feet, given back (the
  memory's `offer_waiting` and `offer_reached_now` from the host's `figure_seen` "approached",
  the brain's `drop_item` of what it holds, done clearing it). What it took, not what they lost
  dying, which it never holds. Tests: one in `brain`. Was: no such event.
- [x] **Set back** (`docs/events_todo.md`, 12): `rubberband`, stages 4 and 5, walking outdoors:
  set back along their trail three blocks or more, as on a bad connection, and a figure stands
  where they had been, gone once seen and looked away from. Tests: one in `act`. Was: no such
  event.
- [x] **Where they died** (`docs/events_todo.md`, 11): `where_you_died`, stages 3 to 5, where
  they last died twelve to sixty-four blocks from them: a figure lies on its side there as a
  body lies, gone when they come within ten blocks (`figure`'s `lies`, ABI 54: the sleeping pose
  with no bed). Tests: one in `act`. Was: no such event.
- [x] **A figure in the light** (`docs/events_todo.md`, 10): `torch_flash`, stages 4 and 5, on
  a light placed underground (the memory's `lit_underground_now`, the brain's next tick, by
  `TORCH_FLASH_CHANCE`; never from the menu): a figure two blocks past it in its light, on the
  floor under a wall torch, facing them, for a fifth of a second. Tests: one in `act`, one in
  `memory`. Was: no such event.
- [x] **Sleepwalking** (`docs/events_todo.md`, 9): `sleepwalk`, stage 5, asleep at night with
  the bed and a door known: they wake standing two blocks out of their front door, the door open
  behind them (set last, so a door gone since loses only itself), their prints in the snow from
  the bed; in the sleep events' group, so `calm` leaves it out. Tests: one in `act`. Was: no such
  event.
- [x] **A door that will not open** (`docs/events_todo.md`, 8): `door_wont_open`, stages 4 and
  5, as they walk to a door of the home: it will not open for six seconds, as if held from the
  other side (`hold_door`, ABI 54: their use refused on both loaders, the door put back on their
  screen, a blow on it heard each try), and three knocks come through it. Named apart from
  `door_held` (stage 2), which shuts a door again once opened. Tests: one in `act`. Was: no such
  event.
- [x] **One more each night** (`docs/events_todo.md`, 7): `one_more_each_night`, stages 4 and
  5, at night with a light of theirs near: at the edge of the light one figure more than the
  nights it came before (`Memory::times`), up to six, side by side, gone together when one is
  seen and looked away from or they come within six blocks; a night apart. Tests: one in `act`.
  Was: no such event.
- [x] **A figure at every window** (`docs/events_todo.md`, 6): `every_window`, stage 5, at night
  with the target indoors and two windows or more (a window a run of panes): a figure outside
  each, a block out, its head at the glass, all gone together when one is seen and looked away
  from (`figure`'s `together`, ABI 54). Tests: one in `act`. Was: no such event.
- [x] **A passenger in their boat** (`docs/events_todo.md`, 5): `boat_passenger`, stages 4 and
  5, rowing alone in a boat or raft with a second seat (the memory keeps what they ride now):
  a figure in the seat behind them, on their client alone (the boat's passengers sent them as
  they and the figure), gone once seen and looked away from, with the boat left, or when anyone
  else gets in (`figure`'s `rides_behind`, ABI 54). Tests: one in `act`. Was: no such event.
- [x] **Hunted** (`docs/events_todo.md`, 4): `hunted`, stage 5, at night outdoors: thirty-six
  blocks ahead of where they look a figure runs at them flat out, after them as they move, and
  is gone two blocks short with a hit's jolt and sound and no harm (`figure`'s `rushes`, ABI
  54). Tests: one in `act`. Was: no such event.
- [x] **The crawl** (`docs/events_todo.md`, 3): `the_crawl`, stages 4 and 5, at night with the
  target indoors and a bed and door known: a figure crawls on its belly from their doorway to the
  bed, head first, a glide slower than a walk, gone at the bed (`figure`'s `crawls`, ABI 54: the
  swimming pose on land; a glide that does not stay now ends the figure). Tests: one in `act`.
  Was: no such event.
- [x] **The figure upside down over the bed** (`docs/events_todo.md`, 2): `upside_down`,
  stages 4 and 5, as they wake (a waking picks it or `foot_of_bed`, of those the stage offers,
  `brain::WOKE_EVENTS`): the figure hangs feet up from the ceiling over the bed, gone once seen
  and looked away from (`figure`'s `upside_down`, ABI 54: its profile named Dinnerbone and its
  cape layer shown, as the game flips a player; the ceiling the first solid block above). And
  every figure now wears its player's skin layers (hat, jacket, sleeves): it was sent none and
  wore the bare skin. Tests: one in `act`. Was: no such event, and figures without their hats.
- [x] **A ring of figures round the house** (owner: "can you add an event where while we are
  inside our house, we get multiple figures standing in a circle around our home";
  `docs/events_todo.md`, 1): `ring_of_figures`, stages 4 and 5, at night with the target
  indoors: eight figures twenty blocks out round the home's middle, on the ground, empty-handed,
  each gone once seen and looked away from, numbered apart from the one figure. With it, the
  figures switch (`events.figure`) holds back the bedside figure too (`choke`, `shot_in_bed`),
  which came whatever it said. Tests: one in `act`, one in `offer`. Was: no ring, and the
  bedside figure with the figures off.
- [x] **At their table** (`docs/events_todo.md`, 41): `at_their_table`, stages 4 and 5, coming
  home: a figure at their crafting table facing it, working at it with their tool, gone within
  five blocks. Tests: one in `act`. Was: no such event.
- [x] **Jumping far off** (`docs/events_todo.md`, 42): `jumping_far`, stages 4 and 5, outdoors:
  thirty blocks ahead a figure jumps in place, a player's jump a second apart, facing them
  (`figure`'s `jumps`, ABI 54, `JumpArc`). Tests: one in `act`, `JumpArcTest`. Was: no such event.
- [x] **It waves from inside** (`docs/events_todo.md`, 43): `waves_from_inside`, stages 4 and 5,
  coming home: a figure just inside a window raises its arm three times, waving, gone within five
  blocks. Tests: one in `act`. Was: no such event.
- [x] **Prints to the cave** (`docs/events_todo.md`, 44): `prints_to_the_cave`, stages 4 and 5,
  the target away or asleep with snow near: prints from their door toward the nearest cave,
  forty-eight blocks at most (`prints`). Tests: one in `act`. Was: no such event.
- [x] **It rides their horse** (`docs/events_todo.md`, 45): `rides_their_horse`, stages 4 and 5:
  their horse with a figure in its saddle for them alone, gone once seen and looked away from or as
  anyone gets on (`figure`'s `rides`, ABI 54). Tests: one in `act`. Was: no such event.
- [x] **It eats at their chest** (`docs/events_todo.md`, 46): `eats_at_their_chest`, stages 4 and
  5, coming home with food in the chest they use: a figure crouched there eating it, its sound, gone
  once seen and looked away from; nothing eaten (`figure`'s `eats`, ABI 54). Tests: one in `act`.
  Was: no such event.
- [x] **A mirror in the window** (`docs/events_todo.md`, 47): `mirror_in_the_window`, stage 5,
  outdoors near a window: a figure just inside it copies their every move (`figure`'s `mirror`),
  twenty seconds or once seen. Tests: one in `act`. Was: no such event.
- [x] **Every door opens** (`docs/events_todo.md`, 48): `every_door_opens`, stage 5, at night
  indoors with two doors or more: every door of the house opens at once. Tests: one in `act`. Was:
  no such event.
- [x] **Wilted flowers** (`docs/events_todo.md`, 49): `flowers_wilted`, stages 3 to 5, the target
  away or asleep: their flowers dead bushes for a day, then flowers again (`swap_block` and back).
  Tests: one in `act`. Was: no such event.
- [x] **A camp nearby** (`docs/events_todo.md`, 50): `camp_nearby`, stages 4 and 5, at night
  outdoors: on a point of their route twenty-four to sixty-four blocks off, a lit campfire with two
  log seats and a figure crouched by it; at dawn its blocks taken away again. On their route, whose
  ground is known, so the dawn's taking finds them. Tests: one in `act`. Was: no such event.
- [x] **A stone circle** (`docs/events_todo.md`, 51): `stone_circle`, stages 4 and 5, once a
  stay: fifty blocks from the house a ring of eight stone-brick pillars, three high, five out
  (`place_blueprint`). Tests: one in `act`. Was: no such event.
- [x] **It slips from their hand** (`docs/events_todo.md`, 52): `slips_from_hand`, stages 4 and 5,
  walking with something held and no monster near: it slips to their feet, picked up again
  (`drop_held`, ABI 54). Tests: one in `act`. Was: no such event.
- [x] **Glass breaks** (`docs/events_todo.md`, 53): `glass_breaks`, stages 4 and 5, indoors: the
  sound of a window breaking at one of their panes, every pane whole (`SoundAtFixture`). Tests: one
  in `act`. Was: no such event.
- [x] **Eyes in the dark** (`docs/events_todo.md`, 75): `eyes_in_the_dark`, stages 4 and 5,
  underground: twelve blocks ahead at eye height two glowing points a hand apart, blinking, gone
  for good within five blocks (a hint; the game's glow particles, lit in the dark). A screen
  glitch, off with `events.glitches`. Tests: one in `act`. Was: no such event.
- [x] **Hearts drop** (`docs/events_todo.md`, 55): `hearts_drop`, stage 5, a hint of a second and a
  half: the right five of their hearts drawn as empty containers over them, no harm. A screen
  glitch, off with `events.glitches`. Tests: the catalogue's own. Was: no such event.
- [x] **The experience slips** (`docs/events_todo.md`, 54): `xp_slips`, stages 4 and 5, a hint of
  four seconds: their experience bar drawn emptying from the right, then as it was; nothing lost
  (`XpSlip`). A screen glitch, off with `events.glitches`. Tests: `XpSlipTest`, the catalogue's
  own. Was: no such event.
- [x] **The pet stares at the wall** (`docs/events_todo.md`, 56): `pet_stares_at_wall`, stages 3
  to 5: their pet held a minute facing the wall of theirs nearest it (`hold`'s `at`). Tests: one in
  `act`. Was: no such event.
- [x] **Bats in the house** (`docs/events_todo.md`, 57): `bats_in_the_house`, stages 4 and 5, at
  night indoors: four bats inside the house a minute, then gone (`place_entity` may place a bat
  now, ABI 54). Tests: one in `act`. Was: no such event.
- [x] **Chains still begin and go on as the catalogue grows** (found as `bats_in_the_house` was
  added: a stay test, "the place walked in 1 of three stays"): the first step of a chain not yet
  begun in the stay is weighed up as an open chain's next step is (`Memory::chain_starts`), and
  an open chain's next step, while it may happen, always has a place on the menu. Never forced:
  the mind still chooses. Tests: one in `offer`; the stay test as it was. Was: with a hundred and
  more events a chain's first step was one weight among them all, and chains often never opened.
- [x] **The lost hour** (`docs/events_todo.md`, 58): `lost_hour`, stage 5, by day: a blink of black,
  and an hour of the day gone (`skip_time`, ABI 54, the overworld's clock, for everyone). With it,
  the host reads the time of day and the day from the overworld's clock, which the sky shows and
  sleeping moves to morning. Tests: one in `act`. Was: no such event, and the mind's time was the
  game's tick count: after a night slept, it thought it still night at dawn.
- [x] **The chest will not open** (`docs/events_todo.md`, 59): `chest_wont_open`, stages 4 and 5,
  walking to their chest: it will not open five seconds, a locked chest's rattle each try, two
  knocks inside (`hold_container`, ABI 54). Tests: one in `act`. Was: no such event.
- [x] **Their name in the snow** (`docs/events_todo.md`, 60): `name_in_the_snow`, stages 4 and 5,
  snow near the home: their name trodden in, letters three by five, the snow taken off only where
  they are (`act::snow_letter`). Tests: one in `act`. Was: no such event.
- [x] **A ring of torches** (`docs/events_todo.md`, 61): `torch_ring`, stages 4 and 5, once a
  stay: sixteen torches twelve blocks out round the house (`set_block` with `settle`). Tests: one in
  `act`. Was: no such event.
- [x] **Breadcrumbs** (`docs/events_todo.md`, 62): `breadcrumbs`, stages 4 and 5, at night while
  they are away or asleep: six of their most plentiful block taken from their chest one at a time,
  each kept, and dropped three blocks apart from their door away from the house. Tests: one in
  `act`. Was: no such event.
- [x] **A figure fishing** (`docs/events_todo.md`, 63): `fishing_figure`, stages 3 to 5: where they
  last fished a figure with a rod, casting now and then, gone once seen. Tests: one in `act`. Was:
  no such event.
- [x] **By the bed** (`docs/events_todo.md`, 64): `by_the_bed`, stages 4 and 5, coming in at night:
  a figure beside their bed facing it, gone once seen. Tests: one in `act`. Was: no such event.
- [x] **It swims beneath** (`docs/events_todo.md`, 65): `swims_beneath`, stages 4 and 5, swimming:
  a figure swims past three blocks under them, side to side (`crawls` in water). Tests: one in
  `act`. Was: no such event.
- [x] **It stands in the rain** (`docs/events_todo.md`, 66): `stands_in_the_rain`, stages 3 to 5,
  raining outdoors: a figure in the open, its face to the sky (`faces` high over it). Tests: one in
  `act`. Was: no such event.
- [x] **Its banner** (`docs/events_todo.md`, 67): `its_banner`, stages 4 and 5, once a stay: a
  black banner beside its house's door, outside (`building::house_banner`). Plain black: a skull
  is the banner's pattern, which a block's state does not carry. Tests: one in `act`. Was: no such
  event.
- [x] **The chest rattles** (`docs/events_todo.md`, 68): `chest_rattles`, stages 3 to 5, indoors:
  their chest's lid open and shut five times fast (`open_lid`). Tests: one in `act`. Was: no such
  event.
- [x] **The trapdoor taps** (`docs/events_todo.md`, 69): `trapdoor_taps`, stages 3 to 5: their
  nearest trapdoor open and shut eight times, ending as it was. Tests: one in `act`. Was: no such
  event.
- [x] **It climbs ahead** (`docs/events_todo.md`, 70): `climbs_ahead`, stages 4 and 5, down their
  shaft: a figure climbs from three blocks over them to the top, gone there (`walks` up). Tests: one
  in `act`. Was: no such event.
- [x] **On the water** (`docs/events_todo.md`, 71): `on_the_water`, stages 4 and 5: a figure on the
  surface of the water near them, gone once seen. Tests: one in `act`. Was: no such event.
- [x] **In the treetops** (`docs/events_todo.md`, 72): `in_the_treetops`, stages 4 and 5,
  outdoors: thirty blocks ahead a figure on top of the trees (`figure`'s `on_leaves`, ABI 54).
  Tests: one in `act`. Was: no such event.
- [x] **A path through the trees** (`docs/events_todo.md`, 73): `path_through_the_trees`, stage 5,
  once a stay: a line two wide cut through the leaves from forty-five blocks out to fifteen from the
  house, pointing at it, leaves only (`place_blueprint` of air over leaves, a column at a time on its
  own ground). Tests: one in `act`. Was: no such event.
- [x] **It rides the cart** (`docs/events_todo.md`, 74): `rides_the_cart`, stages 4 and 5: their
  minecart rolls off with a figure in it for them alone (`rides`, `push_entity`). Tests: one in
  `act`. Was: no such event.
- [x] **A lantern in the woods** (`docs/events_todo.md`, 76): `lantern_in_the_woods`, stages 4 and
  5, at night outdoors: forty blocks ahead a figure walks across with a lantern in hand (`walks`).
  Tests: one in `act`. Was: no such event.
- [x] **Crouched near them** (`docs/events_todo.md`, 77): `crouched_behind_a_tree`, stages 4 and 5,
  outdoors: a figure crouched ten blocks off to one side, gone within four blocks or once seen (the
  crouch of `eats` with an empty hand). Tests: one in `act`. Was: no such event.
- [x] **A snowball thrown** (`docs/events_todo.md`, 78): `snowball_thrown`, stages 3 to 5, outdoors
  in the snow: a snowball from twelve blocks behind them at their back, no one there (`snowball`, ABI
  54). Tests: one in `act`. Was: no such event.
- [x] **An arrow in the door** (`docs/events_todo.md`, 79): `arrow_in_the_door`, stages 4 and 5,
  while they are out: an arrow from ten blocks out struck in their front door (`shoot`). Tests: one
  in `act`. Was: no such event.
- [x] **It raises their sword** (`docs/events_todo.md`, 80): `raises_their_sword`, stages 4 and 5,
  outdoors: far off a figure with their weapon, raising it again and again. Tests: one in `act`. Was:
  no such event.
- [x] **Forty events more** (owner: "once youre done w current 40, commit, then do another 40"):
  `docs/events_todo.md`, The next forty: forty more ideas, none in the catalogue, `ideas.md` or the
  first forty (three that `ideas.md` had were replaced). Was: forty.
- [x] **Forty events to come** (owner: "can you think of more potential events? come up with
  like 20 ... and also write it to docs/events_todo.md"; then "generate 20 more, just make sure
  there are no duplicates"): `docs/events_todo.md`, forty ideas not in the catalogue, in
  `ideas.md` or twice over, each with its stage, needs, what it takes and a size, the ring of
  figures the owner asked for first. Was: none written down.
- [x] **Shot in bed** (owner: "can you also add a variation of this where we get lit on fire,
  maybe shot at with a crossbow"): `shot_in_bed`, as `choke` (stages 4 and 5, asleep, a door and
  the bed known): the same walk, the door opened; at the bed a crossbow out and loaded as a player
  loads one (its draw, the loading sounds), its head down at it, then raised at them, loaded,
  and one shot: the crossbow's sound, two hearts (which wakes them) and up to four seconds
  burning, spared through the fire (`Spare.fireSeconds`); it watches two seconds and goes
  (`choke`'s `how`, ABI 54). Their eyes are kept open by the host with the command now, not a
  hint of the mind's, which `events.glitches` off held back (a stay test caught it). Tests: one
  in `act`, `SpareTest`. Was: the choke alone.
- [x] **A site the home moves to is not away from home** (owner's screenshot: "Away from home
  they keep a second base 9 blocks west of the house"): the sites kept are only those more than
  48 blocks from the home as it is now, and a site the home moves to is let go with the old
  house's doors and lights. Tests: one in `places`. Was: a base made before the bed was moved to
  it stayed "away from home", nine blocks from the house.
- [x] **The figure chokes them in bed** (owner: "can you add an event where figure walks up to
  you while sleeping, then pulls out something and chokes you?"; a lead, held in bed): `choke`,
  stages 4 and 5, as they sleep with a door and the bed known. Their door opens, heard, and is
  left open (owner: "will the door it came in through be left open after waking up"), and the
  figure comes in at it; the host walks it slowly to the bed's side by the pillow (`ChokeWay`: the side nearer
  it with room to stand, then beside the foot, then past it) and holds them asleep meanwhile,
  the night not passing (their sleep timer kept at nothing); there it takes out a lead and
  squeezes once a second for five seconds, a heart each, never below half a heart, the harm taken
  without waking them, each a drowning gasp, the hurt's jolt, the dark closing in (`choke`, ABI
  54). Their client keeps its own fade to sleep off (`eyes_open`), so they see it come. Leave Bed
  breaks free and it is gone; otherwise they wake as it ends. In the sleep events' group, so
  `calm` leaves it out. The owner after a try: "the walking up to the bed is very laggy" (it was
  re-sent a point every half second; it glides now, a block a second, eased by the client, its
  legs moving), "its kinda hard to see the lead" (its rope is drawn from its hand to their neck,
  tied to an unseen silverfish sent to them alone; "i still cant see the lead": tied where they
  lay, at their eyes, the rope ran straight at the camera and was seen end on, so it is tied
  two thirds of a block down the bed and a little lower, across their view; "make it so we see
  it like pulling it out and looking at it and then us and then choking us": at the bed it stands
  a second, pulls the lead out with a swing of its arm, the rope between its hands, looks down at
  it two seconds, then up at them, and the rope's end goes to their chest as it chokes them,
  `ChokeWay.beat`; their eyes kept open eighteen seconds for it), and "verify the open/closed door logic":
  the door is opened by the choke itself, so a door gone since is passed over (a toggle of its
  own that failed stopped the rest of the event, leaving the figure in the doorway). Tests: one in `act`, `ChokeWayTest` in JUnit. Was: no such event.
- [x] **A long stay's memory is saved whole** (owner: "i left my world and rejoined and the
  stage reset"; "it seems like it dropped the whole memory"): the mind's bytes are saved as an
  NBT byte array (`SavedMemory`), as large as they grow, and an older save's base64 string is
  still read. Tests: `SavedMemoryTest` in JUnit. Was: they were one NBT string, which holds 65535
  bytes at most; a memory of about 51 KB (68 KB as base64, half an hour into a world) failed every
  save of the overworld ("Failed to write NBT String ... too long"), the game wrote it empty, and
  the world opened at stage one with nothing remembered.
- [x] **The debug panel in one's own singleplayer world** (owner: "i am in a singleplayer world"
  and it said operators only, in a launcher with an offline profile): a singleplayer world not
  open to LAN lets its one player in whatever the name check says, and a refusal is logged with
  why (the player, the owner's profile, open to LAN or not). Was: "operators only" to the world's
  own owner.
- [x] **A preset picked applies all its values** (owner: "when changing preset it should apply
  all values automatically"): picking another preset takes out of the file every key a preset
  sets (`config::preset_keys`, where any preset differs from `balanced`), those changed by hand
  too, so all of the new preset's values apply; what no preset sets (the model, the log,
  multiplayer) stays. Tests: two in `config`. Was: only the keys still at the old preset's values
  moved, and a value set by hand stayed over the new preset.
- [x] **The last three small ones** (owner: "tend to the small minor issues"): a baby villager is
  never the one that sells (an entity's `baby`, ABI 53; its sale waited until it grew up); the
  lectern notes go on the lectern asked or one of the others the mind names out of their view
  (`put_book`'s `also`), never one the host found within eight blocks, which could be in front of
  them; the scarecrow's armour stand, taken and not placed, goes back to the chest (the put back
  of a failed event's takes already did it). Tests: one in `village`, one in `act`. Was: those.
- [x] **Small events cost a quarter, and `intense` is the default** (owner: "should the amount of
  events per day be scaled up since most events are things the player doesnt notice?"; "make the
  default preset be the intense one"): a small event (the catalogue's `small`, one they may not
  notice) costs at most `events.small_cost` of the day's budget, a quarter by default and a whole
  one on `calm` (`Event::cost_in`), so more of them land while the noticed ones keep their pace; a
  file that names no preset plays `intense` (`DEFAULT_PRESET`; `Config::default()` stays
  `balanced`'s values, the base every preset is laid over). Tests: the budget test counts by the
  config's costs, the empty-file test wants `intense`, one in `events` for the costs; three stay
  tests name `balanced`, whose numbers they check, and the twelve-day one counts every event whole
  so its cap is still the budget's own. The groups test, on `intense`, reached the ending and found
  its script blinking the sun with the glitches off: the ending's beats keep the groups now
  (`Beat::switched_on`: the sun and the flicker with the glitches, the music and the bell with the
  sounds, the figures with the figure). Was: every event cost the same but sounds, an empty file
  was `balanced`, and the ending ignored the groups.
- [x] **Back to the preset, key by key** (owner: "yes back to preset would be good"): each row of
  All keys has a ↺ button; the key is taken out of the file as the change applies
  (`ConfigFile.remove`, `LoadedMod.reconfigure` with removals), and the mod writes the file again
  with the preset's value whenever it lacks a key (`config::lacks_a_key`, on a reload and at
  start). All to preset does it for every key but `preset`, `enabled`, `model` and `log` (owner:
  "a button that resets all values to a preset"). Tests: one in `config`, one in `ConfigFileTest`.
  Was: a key set by hand stayed through
  every preset change, and only typing the preset's value took it back.
- [x] **Tooltips on every key, All keys as it is now, and only the mods it uses** (owner: "add
  tooltips on hover to explain what it does. it should reflect the current setting"; "only have
  it show mods we detect and do something with"): each key's line of the table in `config.md` is
  its tooltip on both tabs (`scripts/gen_config_docs.py` writes it into the content jar, a test
  keeps them agreeing); changing tabs applies the tab left, so All keys shows a preset just
  picked; the debug screen's mods line names only the mods it does something with, a horror mod
  it makes room for and a mod whose furniture or lamps it uses (`Memory::mods_it_uses`). Tests:
  the docs test, one in `memory`. Was: no tooltips, All keys as the screen opened, and every mod
  loaded listed.
- [x] **Every key in the settings, and the F8 screen smaller** (owner: "a full config editor to
  the lodger settings... every key, just in a separate tab"; "make the ingame menu also 0.6
  scale"): the settings screen has two tabs, Settings and All keys; All keys reads the mod's
  config file as it is (`LoadedMod.configFile`) and shows every key by section, a toggle for a yes
  or no and a field for the rest, read back as the kind the key had (`ConfigText`); Done applies
  both tabs, and the mod's own check refuses a bad value with the file put back. The F8 debug
  screen is drawn at 0.6, as the overlay is. Tests: `ConfigTextTest`; `PanelLayoutTest`'s numbers
  for the new scale. Was: a dozen keys on the screen, the rest in the file alone; F8 at 0.75.
- [x] **The overlay fits more, and other mods are only other mods** (owner: "a lot of it is cut
  off"; "the mods list also shows a bunch of fabric stuff"): the F9 overlay is drawn at 0.6 of the
  GUI's size, half the screen wide (230 to 420 of its units), so its values fit; the mods told to
  the mind are those installed in their own right, not Minecraft or Java, nor the modules inside
  another's jar (Fabric API's), nor the loader's or our own. The next decision reads as one
  sentence. Was: values cut off at the GUI's size, and fifty-odd Fabric modules among the mods.
- [x] **The debug screen's next decision before it moves in** (owner: "it keeps saying next a
  decision in about 0s"): before the stay begins (`stages.start_after_days`) the brain decides
  nothing, and the screen says when it moves in; while the model is thinking it says so. Tests: one
  in `debug`. Was: "a decision in about 0s", for days, as if stuck.
- [x] **What is only here a while gives nothing, and no frame turns in view** (owner: "can you
  fix those 2"): a placed-for-a-while animal (`one_more`'s, the squid under the water) cannot be
  hurt, comes shorn, lays no egg and cannot breed, and is never a mooshroom (shorn, one became a
  cow that stayed); the frames and stands an event turns, empties, swaps or dresses are only ones
  out of their view. Tests: a frame in front of them left alone. Was: killed, the animal dropped
  its meat, leather or ink; a frame turned or a stand was dressed as they watched.
- [x] **The tenth bug pass** (five reviewers, the fifth on the events one by one; about forty
  found). A snowman's head made a real snow golem (a pumpkin or a head is set without waking
  what it tops now); a wither skeleton's skull, iron bars and tinted glass came from nothing; a
  pet beside them vanished to the house; their redstone torches were moved and hidden as lights;
  the tool worn was any tool in any chest; a book they wrote in was signed with one word, their
  pages gone; a flower was taken before its pot was known empty; the ladders were taken only at
  the end, so a shaft stopped part way left ladders from nothing; waking elsewhere could stand
  them over a drop or in water; an approach struck a full heart in the nip stage. The brain: the
  new animal needs cancelled a plan the moment they turned; `named_animal` was offered with every
  animal named; a put back that could not go in was kept for good (dropped by the chest now), and
  the debug screen's Clear, Forget and reset left what events had taken kept (`Brain::abandon`).
  The state: another player's far building pushed the target's house out of what it never digs
  beside (kept apart now, near the home, let go when broken); a bed far off kept the old house's
  doors and plants; a retarget left a loot offering armed and its touch unheard; saved entity ids
  named other entities after a restart; the seventeenth visitor was never kept; an empty target
  uuid was never filled. Sales: a cured zombie sold the item again for a few seconds, a restock
  with the screen open sold it twice, and the mark hit a villager beside it. The copy: a dead
  player's drops waited there after a quit (brought to their respawn now), a plant put in a pot
  was spent. The host: a take from a shelf or a jukebox left it showing the item; `ate` read a
  peaceful refill as a meal. The client: two quick switches could delete a download's part file;
  an old Windows kernel, Rosetta or Windows' emulator could be refused for AVX2 wrongly (an
  emulated Java is told to use an arm64 one); the Linux Docker build names its target, so its
  build scripts never run the AVX2 code; the glibc floor is 2.34 on both sides. Tests: the put
  back's drop, abandon, the bed far off, the friend's blocks, the shaft's takes, the pot's order.
  Was: all of that.
- [x] **One x86_64 baseline** (owner: "AVX2 + clear refusal"): the three x86_64 natives are
  built for x86-64-v3 (`.cargo/config.toml`'s `target-cpu`, which llama-cpp-sys-2 hands on to
  llama.cpp), and the host refuses one on a CPU without AVX2 before loading it, naming why
  (`Natives.cpuMissing`: `/proc/cpuinfo`, `hw.optional.avx2_0`, `IsProcessorFeaturePresent`).
  Tests: JUnit for the cpuinfo reading. Was: Linux's native needed AVX2 by accident and ended the
  game on an older CPU with no reason given, and Windows' and the Intel Mac's had plain SSE2, the
  model's CPU path several times slower there.
- [x] **The ninth bug pass** (five reviewers, the fifth on the events one by one; about forty
  found, two in the eighth's own fixes). A failed event no longer runs its own steps to give back
  what it kept (a failed scarecrow put down an armour stand from nothing, a failed alcove dropped
  the held thing where it never dug and forgot it): what its takes took goes back into the chest
  it came from, a spent ingredient as plain items of its kind; and a take holds what its command
  took, not the plan's chest's first favourite. The animal events used any creature near the
  target, wild, another player's or one in front of them, and boats, carts and leash knots passed
  for animals (now their own farm animals by the house, out of their view, and only living kinds);
  `potion_changed` made a real potion of a bottle of water; `replant_other` replaced ripe wheat;
  felling an oak by its house angered it as a broken house. The retarget opened the old target's
  house to its digging (it forgot what was built), kept their death, fishing spot, obsidian and
  frames, and never let a new diary be sold; another player's blocks were never protected; on one
  long run the seventeenth player was never known; `ate` and `fished` were never sent, so the late
  meal and the dock never came. Sales: a villager turned into a zombie or a witch lost what it
  sold, and a cured one sold it again; two sales on one villager lost the second's tag and it
  restocked for ever; a bought offer was taken off while the trading screen was open; the sales
  outlived the world. The other house: a death screen quit kept them from their things, a bed
  upstairs left the doorway out, and a full composter, hive or cauldron handed out items. A chiseled
  bookshelf swallowed what was put in it. The client: Play without it while a size was switching
  kept downloading, a finished download was deleted when its rename failed, a switch that threw
  stuck for good, and a Linux too old for the native said nothing of why (its glibc is named now).
  The harm rule now says what the catalogue does: nips from stage three, real harm from four,
  always spared, the one kill the ending's. Tests: the put back, the spent ingredient, the take's
  hold, the felled tree, the seventeenth player, the retarget's keeps, the glibc hint. Was: all of
  that.
- [x] **The eighth bug pass** (five reviewers, the fifth on the events one by one; about forty
  found, many in the books). Items: a taken thing put back into a slot the chest no longer had (a
  double chest broken in half) was destroyed (any free slot now), and a full chest wore it again at
  every retry; a villager's sale was lost when it took or lost a profession (its offers made
  again: the item drops at its feet), and sold twice after a zombie turned it and it was cured
  (dropped only when killed); a book a villager was bringing was lost when another walk of it
  began; a take now names the item it means (`take_item`'s `expect`, ABI 52), so a slot holding
  something else since is not spent; a failed event dropped what would have given a kept stack
  back (the scarecrow's piece when the stand would not move; those are sent all the same, the
  event still a failure). The books: bought by a witness, the diary was still brought (bought by
  anyone now, its chain closed), it named a visiting player, a line longer than a page overflowed
  it, its own line was never kept as written (so written again), the lectern notes were refused
  by a lectern holding a book (the nearest free one within eight blocks now), and a retarget kept
  the old target's diary. `villager_named` renamed villagers the player had named, with the item
  names' words. The other house: a second player's visit built a house around the first (one at
  a time now); a death there ended the visit on the death screen, and what they carried was lost
  with the respawn; crops trampled and a plant out of a pot came home (left out of the copy); a
  book on a copied lectern, a trident or arrows stayed there. The mind: the villager at the door
  kept its night unrecorded for half a day (a move back by morning is a tail now); the ending's
  tick ran with the target gone; the log never rotated over many starts; the diary's unbought
  need ignored the days; the house's most-built block counted lights and far blocks; the
  seventeenth player could evict the target; a retarget kept the old target's words, places and
  people; `text_temperature` was unchecked. The client: the choice showed while switching, a
  singleplayer world waited on the server's preparation, a license accepted for one model counted
  for another of the same mod. The rule on making things is amended (owner): what it makes of its
  own is its own work and worth no more than paper (its books, `brew_left`, `second_bed`,
  `composter_fills`), never ore, a tool or anything worth more. Tests: a failed event giving back,
  the tail, the log's rotation, the hopper's slots, the long line, the unnamed villager, the diary
  bought by another. Was: all of that.
- [x] **The seventh bug pass** (five reviewers; about fifty found, many in the sixth's own
  fixes). The other house again: a kept stack past 99 threw at every save and stopped the world
  saving (a full stack now spills beside it); its walls ran four high, so a flight, a pearl or a
  swim down found the void (walls the whole height now, and anyone out of the box is sent home
  at once); TNT and respawn anchors were copied and their blast dropped the copy's blocks; what
  they left in its chests or dropped there was lost (it comes home with them). A figure up as the
  world closed held the old level for good. The wall swap rebuilt blocks in their default state
  (a wall torch fell, stairs turned, a double slab was half one; `SwapBlock` keeps the state now)
  and could make a redstone torch (none in the family now); candles were still never recorded;
  one swapped torch was marked as its own light, which the leave ending would put out. Toggles
  went the other way half the time: `close_door` opened a shut door, and the gates, campfires,
  `wake`, `door_opens_itself` and `village_doors` likewise (`toggle_block`'s `to`, ABI 50). A
  pet could be set into a wall (the host finds room within two blocks, or refuses); the villager
  and the pet stood in the doorway itself (two blocks out now, and the villager back by morning);
  `door_opens_itself` opened a random door; the doorway, chest and bed figures stood a block high;
  `one_room_dark` lost a torch to a full chest; `head_on_post` never rested a head on its post;
  `cook_one` could jam a full furnace; the jukebox waited out of sight it was never in; `busy_away`
  set its work in front of them as they came back. The mind: the first observation after a reopen
  was day 0; the retarget missed a friend at the old index and forgot too little; its lights were
  kept after they broke theirs there; "thanks" and "fucking" stopped counting; the chest was its
  own for three days only; the house's corners and glass were not its; the usual door could be
  evicted; `multiplayer.pick` took anything; house sounds where they stand lapsed for most of
  every half minute; a chain's noticing looked at the wrong step; boats were offered with no boat.
  The download screen: one license accepted counted for every size (the 3B's was never shown),
  the model unloaded at every quit to the title and loaded again behind "Getting ready" (it stays
  loaded now, the idle unload alone), the size button vanished after a failure, and a size changed
  mid-download showed a failure for a moment. `calm` now keeps the glitches off and two sleep
  events out, as it promises. Tests: the swap's, the acceptance's, the catalogue's. Was: all of
  that.
- [x] **The sixth bug pass** (five reviewers, the fifth on the events one by one; about
  fifty found). The other house could kill and duplicate: its floor ended in the dark, so a walk
  out of the door dropped them into the void, and its copy could be mined in survival and carried
  home; now one fixed place in the dimension, cleared before each copy, a barrier floor with walls
  at its edge and a layer under the copy, adventure mode while there (put back after, a quit
  too), everything checked before anything is written, and a refused move said. A figure's path
  skipped its first point (a path of one ended at once, the other house's walker and the dig
  exit's among them); a figure at its path's end said nothing of how it was looked at, and every
  sighting went to the dimension they were in, not the one it was shown in. The target was matched
  by index in three places (after a restart a friend at that index was taken for them: their
  bed, their blocks, their curses); a change with no spot, or checked without one, passed the
  witness rule. Temper: its chest angered it once a stay, calm stuck a hair under rest and kept
  its stages slow, grass by its house and their own door counted as its things, the daily cap
  reset when the words list was full, a campfire was a fire set, "whatever" a curse. Chains: a
  step waiting on its stage lost the noticing before it, a chase before the watcher ended it, the
  bold steps' stages were not temper's. A reply held back in the fight was spent; dropping the
  plans left their waits; `absent_days` took 0. The download screen hid Play without it after a
  failure, leaving only Quit; a last mirror of a download could land over the failure; the Vulkan
  hint never showed (Java drops the system's reason: the loader is now tried by itself); the
  console could not use show, force, stage and action. The events: `lever` had no TNT check (none
  within 16 blocks now); a stand already wearing that slot kept the piece for good (dropped at
  its feet now, and a store never overwrites one item with another); `bucket_floor` set water in
  the room's air; `village_doors` opened and shut each door; `candles_lit` looked for a candle
  among the fixtures, where none is; `fish_on_line` compared a short id; `cauldron` filled and
  emptied a tick apart (the host toggles it now); the tunnel digs were refused for their own
  torches; the house's door events while away went to the last village house; `harvest_one` cut
  seedlings (ripe only now, any of the four crops, its own item to the chest); boats moved
  minecarts; `torch_blue` missed lanterns and `window_bars` turned panes another colour;
  `tab_entry_stays` lasted a minute; `chest_answers` was no sound to the switches. Tests: several
  in `temper`, `chains` and `memory`; the catalogue's every-event test with the candle where
  the game puts it. Was: all of that.
- [x] **The fifth bug pass** (forty found, four reviewers): **picking a preset on the settings
  screen changed nothing** (the file holds every key, written at the first start, so balanced's
  values stayed; now the old preset's values the player never changed are taken out and the file
  written again, `Init.config_before` and `InitReply.rewrite`, ABI 45). The mind: a sorted chest
  with two equal stacks was tidied out of order; an event that made nothing spent the day's
  budget and counted toward the next stage, and a plan run early on a new day was charged to the
  day before; "while they sleep" was planned for events that need them awake (and "away" for
  ones by the door), waiting out the whole window; `replant_other` took golden carrots and baked
  potatoes for seed; `harvest_one` cleared carrots into wheat; one ladder in the chest made four;
  a `want` or `witness` question the model was too slow for was asked again forever; the
  reactions (a door held, a knock answered, a breath) went on through the fight and beside
  another mod's horror; a light moved, a lantern swapped, a flower potted or a crop harvested
  added the block to its store from nothing; flowers, plates, seeds and water were taken before
  they were placed, so a failed placing lost theirs; a sound's next way could be its last again.
  Its memory: the first eight things ever held stayed the favourites for good; the Nether and
  the End stuck to whoever they saw first in multiplayer; a mine lit far off filled the home's
  lights and a far base's door became its usual door; the home read dropped lamps and candles;
  the ending's returned thing stayed "held" and its grown room could be sent again, and the room
  had no doorway; `brain.plan_every` took 0; Forget everything lost the mods list, the target
  pick and what it held. The host: a temporary animal saved with its chunk dropped its saddle
  twice; a fight whose figure's chunk unloaded could not be won (it ends as a flight now); a
  village gathered twice stayed in the house; entities discarded while the level's live list was
  walked; a moved bed's respawn was updated only for players in its dimension; a spared arrow
  falling in fast could still kill; a second swapped loot item replaced the first kept one. The
  client and the rest: a download whose server never answered hung with no Try again, and Quit
  threw away a part file (it is kept now); `model.idle_minutes` was read by nothing (the model
  unloads after it); a key misspelt inside a section was ignored
  silently (reported now); NeoForge counted tilling with torches in the off hand as building;
  the debug overlay kept the last world's picture on the next server. The CI builds the Linux
  natives on Ubuntu 22.04 (24.04's glibc did not load on Debian 12) and Windows on ARM with
  clang-cl (llama.cpp refuses MSVC there). The docs: `/lodger status` named on the Modrinth
  page (it is `/craftkit status`), when the config is read, the presets' distances, and that the
  GPU is used on Macs alone. Tests: 7 in the mind, 2 in JUnit, one pinned test turned to report
  what it pinned. Was: all of that. Left for their items: the docs' other `/lodger` commands
  (item 9), the `multiplayer.*` keys read by nothing (item 16), and two for the owner (a client
  that only joins servers still downloads the model; Windows and Linux have no GPU build).
- [x] **The fourth bug pass** (seven found): the release script failed on Windows
  (gradlew.bat by its full path now), packed the CI's natives of the commit beside uncommitted
  mind changes (refused unless `--allow-dirty`), and picked up an old version's jars (cleared
  first). The place's want counted blocks by an index into a list that drops its oldest and its
  broken (a snapshot of what stood there now). A fight the host refused froze every plan for a
  day (a refusal ends it at once). A totem in hand let a void death through on Fabric, and on
  NeoForge a totem already failed counted (the source and the loader's order are asked now). The
  ground search no longer looked three blocks up, and wave mobs could be set in a cave under a
  hill (the surface now). Dead code of the old lethal estimate gone. Tests: one in the brain.
  Was: all of that.
- [x] **Warped animals that are never still, and animals that come at them** (owner:
  "seems too stiff ... should be spookier"; "a mob comes towards you very slow ... the opposite
  too, quick"): every `warp_animal` now takes two of its three ways at once (tall, upside down,
  creeping), its height its own, its model bent on their screen as the fight's are, its head
  snapping aside and back, a shudder now and then, its own voice slowed. `slow_approach` and
  `quick_approach`: one animal near them, half the time bent, comes at them very slowly or at a
  run, stops a block and a half off, and stares, turns away, or strikes once never below half a
  heart (`approach`'s `pace`, `then`, `damage`, ABI 44). Tests: `warped`'s, one in `act`. Was:
  one way or two, frozen; no such event.
- [x] **The third bug pass** (twelve found): on Fabric a totem never saved them in
  the fight (its death hook runs before the totem), now a totem in hand is the game's own way;
  the fight's own estimate of a lethal hit ignored the hurt cooldown and more, now the game deals
  every hit and any death is caught and kept; a refused kept death left them at half a heart
  for good, now the game's own goes ahead; a death in its last seconds counted as a loss, now
  the reward comes first; the catalogue's own events (an arrow, a splash) ran in the middle of
  the fight, now they wait; walking past its place (with no house) started the fight, now
  crouching there calls it; what they had built near its place before it chose it met its want;
  the rising took a porch roof for the ground; the render hook redirected a call another mod
  may also hook (an inject after it now); the warped set was read every model every frame (a
  quarter second's cache now); wave mobs picked up loot and bred; a creeper gone another way
  counted as a blast. Tests: two in `ending`. Was: all of that.
- [x] **A script for release jars, every platform's native in each** (owner):
  `scripts/build_jars.py <minecraft>`, with `versions/<minecraft>.properties` holding the pins
  (`26.3` today), the natives from the CI run of the commit, from this computer (`--natives
  local`, both macOS chips on a Mac) or a directory, each jar checked to carry them all; at boot
  a screen names a mod that could not load and why (`FailedScreen`), the server's log an error;
  a CPU the host does not know is named as it is (`riscv64`), where it passed for x86_64 and
  failed to load obscurely. Tests: `Natives`, `LoadedMod` in JUnit. Was: the CI alone made the
  six-native jars, and a missing native was silent but for `/craftkit status`.
- [x] **The creepers' blast** (owner: "creepers aren't dealing any damage"): 26.3 marks
  an exploding creeper dead before it goes, so the fight took its blast for a kill and dealt
  nothing; a creeper gone with its health left went off, and the blast is dealt by how near they
  stood (nine at its heart to two at five blocks, scaled as a mob's hit), with a push away. Was:
  no harm at all.
- [x] **No split** (owner: "nevermind on the splitting the repo"): one repository;
  `gen_notices.py` comes here instead.

## At the very end

- [x] **A pass over the docs for anything stale** (owner). Done: every page under `docs/`, the
  README, `mod/modrinth.md` and the config's comments read against the code and put right: the
  status lines (design only, phases still to come) now say where it stands, the paths and
  modules as they are, ABI 52, the four model sizes, the GPU by default, six natives, the stage
  table and the counts from `arc.rs` and the catalogue, the hint kinds the client renders, the
  tests as they are (no game tests), the seed and `brain.queue` as the code has them, and no
  time words. Was: every page under `docs/`, the README, `mod/modrinth.md` and the config's
  comments read against the code as it is: features, numbers, commands and paths that changed,
  dates that crept back in, promises the mod no longer keeps. After the last bug pass.

- [x] **Out of the door, its place by the home, a held door only shuts** (owner, playing): a
  scan tells each door's way out, the side of it the sky is over (`Scan::outs`, ABI 54), and every
  event that stands or puts things before a door goes by it (`sleepwalk`, `kill_sign`,
  `head_on_post`, `snowman`, `words_on_sign`, `arrow_in_the_door`, `door_cloud`, the trails and
  the rest), the old guess kept for a door that does not tell; `move_player` looks one block down
  too; its place is chosen again once the home moves more than 32 blocks from it; `door_held` only
  shuts. Was: the way out was the side away from the home's middle, which is where they stand
  still, and beside a door in a long wall it pointed along the wall: `sleepwalk` put them into
  the planks beside the door and failed with no room to stand there. The place stayed by the
  first home, a hundred blocks off at that floor's height, where a sign found nothing to stand on.
  `door_held`, forced on a shut door, opened it.

- [x] **Peeking from behind the trees** (`docs/events_todo.md`, 81, owner's): `peeks_from_the_trees`,
  stages 4 and 5, in the woods and walking: a figure half out from behind a trunk ahead, slipping
  back behind it once looked at (`figure`'s `peeks`, ABI 54), a second farther in five seconds on.
  The tree read: logs scanned around them every twenty seconds in a wooded biome, columns of three
  or more the trunks (`crate::trees`), the biome kept in the memory's snapshot. Tests: `trees`'s
  three, one in `act`. Was: the mind did not know where any tree was.

- [x] **A warp that stretches and shakes** (owner, playing: "more like a stretching that makes sense
  and like shaking"): a warped model stretches whole, tall and thin or low and wide with its volume
  kept and its feet on the ground, grown in over two and a half seconds and breathing; every part
  trembles; the head tilts over to an angle no neck holds and twitches every few seconds
  (`client/Warp.java`). `warped_animal`'s height is the stretch's now (the hint's `id:height`), not
  the server's scale as well. Tests: `WarpTest` rewritten for the new warp (the old one checked each
  part's random bend, which is gone), one in `act`. Was: every part of the model was stretched,
  squashed and turned its own random way at once, which read as a broken model, and the server's
  1.8 times on top of the stretch made it twice as tall again, at once.

- [x] **Moves only while unseen** (`docs/events_todo.md`, 101, owner's): `while_unseen`, stages 4 and
  5, at night in the open: a figure behind them that holds still while on their screen with nothing
  between and comes a step nearer each fifth of a second it is not; at arm's length a real hit by the
  harm stages (two hearts, as `figure_swing`, owner: "a real hit by stage"), never a kill
  (`figure`'s `while_unseen`, ABI 54; `Unseen.java`). Tests: `UnseenTest`'s two, one in `act`.
  Was: no such event.

- [x] **The choke's looking seen from the bed** (owner, playing: "they should stand in front of the
  bed and look at the lead, then at the player, walk over to bedside, then do the thing"): the
  figure walks to the foot of the bed first, and there, in their sight down the bed, stands, takes
  the lead (or the crossbow) out and looks at it, then at them; then it walks round to the pillow,
  the rope kept between its hands, and chokes or shoots (`ChokeWay.Beat.TO_THE_BED`,
  `ChokeWay.foot`); no room at the foot, straight to the side as before. Their eyes are held open
  up to 22 seconds now, the walk round counted. Tests: one more in `ChokeWayTest`. Was: it walked
  straight to the bedside and did its looking there, beside their head, where it could not be
  seen.

- [x] **Three bed events: the choke, the fire, the crossbow** (owner: "a choke event, a fire event,
  and a crossbow event. crossbow should just shoot you from the end of the bed, others look and walk
  around ... use flint and steel for the fire one"): `burned_in_bed`, new, looks at a flint and
  steel at the bed's foot, walks round to the pillow and strikes it at them, and they burn five
  seconds, spared (`choke`'s `how` `light`); `shot_in_bed` shoots from the bed's foot, no walk round
  and no fire now. Calm leaves the new one out with the other sleep events. Tests: one in `act`,
  the figures switch's test names it. Was: one fire-and-crossbow event, the crossbow's bolt setting
  them burning, shot from the pillow.

- [x] **Drawn to the light** (`docs/events_todo.md`, 102, owner's): `drawn_to_the_light`, stages 4
  and 5, never from the menu: five seconds of running at night in the open with a light in either
  hand (`Memory::running_lit`; their pace kept in the snapshot now), once a run and a chance in
  three, a figure far off walks at their light; the light put away, it has lost them for good
  (owner: "when putting torch away it no longer sees you"); at twenty blocks it rushes them
  (`figure`'s `drawn_to_light`, ABI 54; `Drawn.java`). Tests: `DrawnTest`'s two, one in `act`.
  Was: no such event.

- [x] **Taken up into the sky** (`docs/events_todo.md`, 103, owner's): `taken_up`, stages 4 and 5,
  outdoors: one to four of their most valuable stacks (`Worth.java`: netherite, elytra, totems,
  diamonds, enchanted things first; never under the worth of a stack of iron's tenth), from their
  inventory but what they wear and hold, and from their chests (owner: "consider chests as well,
  and a random number of items"), kept with the world and shown to them alone hanging in front of
  them and flying up into the sky (`take_up`, ABI 54; the picture is on their client, so no hopper
  can catch a copy). `sky_gives_back`, half a day on at the earliest: all of it falls out of the sky
  before their door (`drop_kept`, ABI 54, which does nothing when nothing is kept). Every ending's
  start and a reset give it back too (`crate::taken_up`). Tests: `WorthTest`, `taken_up`'s, one in
  `act`, one in `ending`. Was: the lodger took one thing at a time, from a chest alone.

- [x] **From tree to tree** (`docs/events_todo.md`, 82): `tree_to_tree`, stages 4 and 5, in the
  woods: a figure behind a trunk behind them that is a tree nearer each time they look back and away
  (the trunks behind them, `Memory::trunks_behind`, four blocks nearer each), gone after the nearest
  with a branch's crack (`figure`'s `then_at`, ABI 54). Tests: one in `trees`, one in `act`. Was: no
  such event.

- [x] **Knocks on the trees** (`docs/events_todo.md`, 83): `knocks_on_wood`, stages 3 to 5, in the
  woods: knocks in threes on a trunk ahead, then on one farther in, then a third; the last tree's
  bark stripped at eye height (the tree read keeps each trunk's log now). Tests: `trees`'s, one in
  `act`. Was: no such event.

- [x] **A trail blazed** (`docs/events_todo.md`, 84): `blazed_trees`, stages 3 to 5, in the woods:
  three to five trunks in a line straight ahead of them, a strip of bark gone from each at eye
  height (`trees::blaze_line`). Leads straight on, not to its place: the trees read are within
  twenty blocks of them. Tests: one in `trees`, one in `act`. Was: no such event.

- [x] **A branch snaps** (`docs/events_todo.md`, 85): `branch_snaps`, stages 3 to 5, walking in the
  woods: a crack in the tree just passed and up to four of its own leaves torn out, falling
  (`snap_branch`, ABI 54; never persistent leaves, which someone placed). Tests: one in `act`. Was:
  no such event.

- [x] **The tree line** (`docs/events_todo.md`, 86): `the_tree_line`, stage 5, walking out of the
  woods (`Memory::leaving_the_woods`): figures between the trees of the edge behind them
  (`trees::between_trunks`), gone together once seen and looked away from. Tests: one in `trees`,
  one in `act`. Was: no such event.

- [x] **The trees close behind** (`docs/events_todo.md`, 87): `trees_close_behind`, stages 4 and 5,
  walking in the woods: the woods' own sapling set on their trail behind them and grown at once
  (`trees::sapling_of`, `Memory::way_they_came`). Tests: one in `trees`, one in `act`. Was: no such
  event.

- [x] **A tree comes down** (`docs/events_todo.md`, 88): `tree_comes_down`, stages 4 and 5, walking
  in the woods: a tree ahead of them falls across their way, its logs laid along the ground as falling
  blocks, the stump left (`fell_tree`, ABI 54; `Felled.java`, never a log within two blocks of them,
  never one landed on anything but air). Tests: `FelledTest`, one in `act`. Was: no such event.

- [x] **The animals flee the woods** (`docs/events_todo.md`, 89): `animals_flee_the_woods`, stages 4
  and 5: the animals near them run past them away from behind, and far back a figure walks across
  (`Memory::animals_to_flee`, never pets or named ones). Tests: one in `act`. Was: no such event.

- [x] **Round and round a tree** (`docs/events_todo.md`, 90): `circles_a_tree`, stages 3 to 5, in
  snowy woods (the biome they stand in, `trees::snowy`): prints round a trunk behind them, two square
  rings (`trees::round_a_trunk`, each side long enough for prints). Tests: one in `trees`, one in
  `act`. Was: no such event.

- [x] **Higher falls by the stage** (`docs/events_todo.md`, 114, owner: "for the ledge event,
  depending on the stage higher falls are okay"): `shove` is stage four's, three blocks at most;
  `shove_far`, new, stage five's, up to six, a fall that harms; the host's push needs the health
  for the fall and a heart over (`Edge.healthFor`), its highest edge six now. Tests: one more in
  `EdgeTest`, the batch-16 test checks `shove_far`'s six. Was: one push for stages four and five,
  three blocks at most.

- [x] **The villagers flee** (`docs/events_todo.md`, 91): `villagers_flee_you`, stages 4 and 5,
  walking into the village (`Memory::entering_village`): every villager near runs from them a
  minute (`panic`, ABI 54: the memory of a hurt alone, no gossip, no golem) and the bell rings, last.
  Tests: one in `act`. Was: no such event.

- [x] **The villagers refuse them, or one stabs** (`docs/events_todo.md`, 92 and 111, 111 the
  owner's): `trades_refused`, stages 3 to 5, a day of every villager shaking its head with a no as
  they go to trade; `villager_stabs`, stages 4 and 5, the next villager they go to trade with shows
  a blade and stabs them, two hearts by the harm stages, never a kill (`hold_trades`, ABI 54: the
  loaders refuse the villager's use, Fabric's `UseEntityCallback`, NeoForge's `EntityInteract`).
  The trades are refused, not shown crossed out: the game's own head shake says it. Tests: one in
  `act`. Was: no such events.

- [x] **A villager spins** (`docs/events_todo.md`, 93): `villager_spins`, stages 4 and 5: a villager
  off a way turns round and round where it stands until they come near (`hold`'s `spin`, ABI 54).
  Tests: one in `act`. Was: no such event.

- [x] **The villagers walk out** (`docs/events_todo.md`, 94): `villagers_walk_out`, stages 4 and 5,
  at dusk (a new need, 11,500 to 13,500): the villagers near them single file out of the village
  toward its place, home to bed of themselves after. Tests: one in `act`. Was: no such event.

- [x] **A villager collapses** (`docs/events_todo.md`, 95): `villager_collapses`, stages 4 and 5: a
  villager falls dead on their screen alone, the game's death event sent to them, held still on the
  server; a minute on, when they look away, it is sent to them again whole and let go (`fake_death`,
  ABI 54; the held mobs' tag and sweep keep one from being left frozen). Tests: one in `act`. Was:
  no such event.

- [x] **The wall bursts** (`docs/events_todo.md`, 96): `wall_bursts`, stages 4 and 5, indoors: a
  block of their wall breaks (hidden ten minutes, the most a hide lasts, not till morning) and five
  silverfish come out at them. The spared hostiles, new (`place_entity`'s `targets` and `harm`, ABI
  54; `Temporaries.addHostile`): set on a player for a while, hurt at the harm asked, never a kill
  (a death they would cause is refused, the player left at half a heart, `Worlds.allowDeath`), and
  killed they vanish with nothing dropped (both loaders' death hooks). Tests: one in `act`. Was: a
  placed entity was a farm animal or a stand, unhurtable; no hostile could be set on anyone.

- [x] **A phantom at noon** (`docs/events_todo.md`, 97): `daylight_phantom`, stages 4 and 5: a spared
  phantom set on them out of the noon sun, burning, a nip, gone in fifteen seconds. Tests: one in
  `act`. Was: no such event.

- [x] **Down from the ceiling** (`docs/events_todo.md`, 98): `drops_from_the_ceiling`, stages 4 and 5,
  underground: a spared cave spider drops before their face. Anywhere underground, not their tunnel
  alone. Tests: one in `act`. Was: no such event.

- [x] **Wolves circle** (`docs/events_todo.md`, 99): `wolves_circle`, stages 4 and 5, at night in the
  open: five wolves circling them nearer, growling, then running off (`circle`, ABI 54; `Ring.java`,
  the wolves placed for a while and unhurt). Tests: `RingTest`, one in `act`. Was: no such event.

- [x] **A zombie in their gear** (`docs/events_todo.md`, 100): `zombie_in_your_gear`, stage 5, at night:
  a spared zombie in copies of their armour and tool walks at them, two hearts a hit, never a kill
  (`place_entity`'s `wears`, ABI 54; the memory keeps what they wear now). Tests: one in `act`. Was:
  no such event.

- [x] **Vexes** (`docs/events_todo.md`, 113, owner's): `vex_single` (stage 4, two hearts a hit) and
  `vex_strikes` (stage 5, three), one vex at them; `vex_wave` (stages 4 and 5), twelve, half a heart
  each; all spared, never a kill, nothing dropped, gone in half a minute. Tests: one in `act`. Was:
  no such events.

- [x] **A creeper behind them** (`docs/events_todo.md`, 104, owner's): `creeper_behind`, stages 3 to 5:
  a creeper sent to their client alone, two blocks behind them, swelling and hissing, then the
  explosion's flash, sound and jolt, no harm and nothing broken (`fake_creeper`, ABI 54). Tests: one
  in `act`. Was: no such event.

- [x] **A villager flees in and dies** (`docs/events_todo.md`, 105, owner's): `villager_flees_in`,
  stages 4 and 5, once a stay, no village near the home: a villager made before their door bangs on
  it, the door opens, it runs in and drops dead inside (`villager_flees_in`, ABI 54: the host walks
  it in and kills it with no killer, so no gossip and no drops). Tests: one in `act`. Was: no such
  event.

- [x] **The sound goes wrong** (`docs/events_todo.md`, 106, owner's): `sound_warp`, stages 3 to 5:
  every sound on their client slowed, in a hall's long echo, muffled or crunched a while, a way of
  four in turn (the `sound_warp` hint; `ChannelMixin` sets each sound's pitch and its source's
  OpenAL effects as it starts, `SoundWarps`, a reverb, a distortion and a low-pass filter; a device
  without EFX keeps the slowed way). `creeper_behind` muffles their hearing after its bang. Tests:
  `SoundWarpTest`, one in `act`. Was: no such event.

- [x] **Dead redstone** (`docs/events_todo.md`, 107, owner's): `dead_redstone`, stages 3 to 5: a minute
  and a half of every plate, button and lever by the house giving no power (`dead_redstone`, ABI 54:
  `DeadRedstone.java` and the signal mixins on the three, `ownSignal` and `getDirectSignal`; their
  neighbours told as it begins and ends). Tests: one in `act`. Was: no such event.

- [x] **The debug panel can pause** (owner: "an option to the ingame debug menu where it pauses
  the game when its open, should be toggleable"): a Pause button beside HUD in the panel's header,
  off at first; on, the open panel pauses a singleplayer world as the game's own pause does
  (`DebugPause`, kept in `config/craftkit/debug.properties`). Tests: `DebugPauseTest`. Was: the
  panel never paused.

- [x] **Prints round the house that stay** (`docs/events_todo.md`, 108, owner's):
  `prints_around_the_house`, stages 3 to 5, away or asleep: a few more footprints each night, by
  turns in to a window, round the walls, waiting before the door (`crate::prints`); kept in the
  memory and drawn on their client as the handprints are, dark dust on any ground, sent again near
  home every nine minutes (the `footprints` hint; `FootPrint.java`). Off with the glitches. Decals
  of dust rather than a block, so nothing is left in the world. Tests: `prints`'s two,
  `FootPrintTest`, one in `act`. Was: prints were snow's alone, gone with the next fall.

- [x] **The sound warp on top of Sound Physics** (owner: "apply it on top more carefully"): with Sound
  Physics Remastered loaded, `sound_warp` is set after it (the channel hook at the tail of play and of
  each move, a later priority) and takes only what it needs: the muffled way's filter over its own
  while the warp lasts, the hall by lengthening its four reverbs and putting them back after, the
  broken way's distortion on its first send alone. Was: the warp set the sends and the filter as
  the sound started, and Sound Physics set its own over them, so the hall, the muffled and the broken
  ways were lost with it.

- [x] **Their cave torches flicker** (`docs/events_todo.md`, 109, owner's): `cave_torches_flicker`,
  stages 3 to 5, underground: their lights near them blink down the line toward them, the nearest
  dark a moment last (`hide_block` a few ticks at a time; `Memory::cave_lights_near`). Tests: one in
  `act`. Was: `flicker` was a light indoors alone.

- [x] **Things out of place** (`docs/events_todo.md`, 110, owner's: "not just limited to caves"):
  `out_of_place`, stages 3 to 5: a small scene by turns (a bed, a table laid with a candle cake, a
  chair, a potted flower, a carpet and a lantern) on a cave floor they have not entered or on their
  way back, taken away ten minutes on, each of its own blocks alone (`crate::out_of_place`). Tests:
  one in `out_of_place`, one in `act`. Was: no such event.

- [x] **Trees bare and skinned** (`docs/events_todo.md`, 112, owner's): `bare_trees`, stages 4 and 5,
  in the woods: a stand of trees off to one side bare of leaves, stripped, or both by turns
  (`bare_tree`, ABI 54; `Memory::stand_of_trees`). In view or not; overnight was not needed, since the
  stand is off to the side. Tests: one in `act`. Was: no such event.

- [x] **The menu freaks out** (`docs/events_todo.md`, 115, owner's): `menu_freaks`, stages 4 and 5:
  armed ten minutes, the next pause menu's buttons shake and settle over four seconds, the first
  reading `stay` a moment (the `menu_freaks` hint; `PauseScreenMixin`, `MenuFreaks`). The pause menu,
  not the title screen: the mod is in a world. Tests: `MenuFreakTest`, one in `act`. Was: no such event.

- [x] **Chains for the newer events** (owner: "remember the chain functionality, so chain events,
  where possible"): seven chains in `chains.rs`, the woods, the marked trees, the village wrong,
  the sky, the night, the visitor and the glitches (docs/entity.md, Chains). Tests: the chains test
  counts sixteen now (it counted nine) and checks each id is its own. Was: none of the newer events
  led to another.

- [x] **Chains over the whole catalogue** (owner: "make new chains out of all the events where it
  make sense ... where it doesnt conflict/duplicate another chain"): nineteen more in `chains.rs`,
  the knock, the bed, the hands, the voice, the lights, the sky wrong, the screen, the village, the
  kills, the water, the cave, the figures, the mirror, the death, the animals, the field, the chest,
  the steps and the bow (docs/entity.md, Chains); `knocks_on_wood` left to the woods alone. Tests:
  the chains test counts thirty-five and checks no event is in two chains. Was: sixteen chains,
  `knocks_on_wood` in two of them.

- [x] **The CI's checks behind a box** (owner: "make the checks matrix a checkbox, else it eats up
  too many minutes of actions"): `ci.yml` has a third input, *Run the checks*, off at first; the
  checks job on three systems runs only when it is ticked, and the jars build after the natives
  whether the checks ran or not (stopped only by a check that failed). Was: every run ran fmt,
  clippy and the tests on macOS, Ubuntu and Windows first.

- [x] **Version 0.1.1** (owner): the lodger (`mod/gradle.properties`, `core/Cargo.toml`) and craftkit
  (`host/gradle.properties`, `sdk/Cargo.toml`) at 0.1.1, the lodger needing craftkit 0.1.1 or later
  (ABI 54). Was: 0.1.0.

- [x] **The mod's build finds the host jar of its version** (`mod/build.gradle`): the craftkit jar
  of `craftkit_version`, so an older one left in `host/*/build/libs` after a version moves is
  passed over. Was: the build stopped with "more than one file" until the old jar was removed.

- [x] **The jars' manifests at the build's version, and checked** (owner: the game said "lodger
  0.1.0 requires craftkit 0.1.0, which is missing" after the move to 0.1.1): the mod's manifest
  tasks take the versions and the nested host jar's name as inputs, so a version moved fills them
  in again; and each lodger jar checks itself as it is made, its manifest at the build's version
  and every jar it nests inside it (`mod/build.gradle`). The host's manifests carry the project's
  description, not its resources task's. Was: Gradle kept the 0.1.0 manifest, which named a host
  jar no longer in the jar, and Fabric found no craftkit; craftkit read "Processes main resources."
  in the mod lists.

- [x] **The handprints shorter** (owner: "the handprints dont seem to stop"): `handprints` shows half a
  minute. Was: a minute and a half, still on the glass long after they had looked at them.

- [x] **Breathing in the wall** (`docs/events_todo.md`, 116): `breathing_in_the_wall`, stages 3 to 5, indoors: slow breaths from inside the wall beside them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A scream in the house** (`docs/events_todo.md`, 117): `scream_indoors`, stages 4 and 5, indoors at night: a goat's scream from above or through the wall. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A tool snaps behind them** (`docs/events_todo.md`, 118): `item_break_behind`, stages 3 to 5: a tool's break heard behind them, nothing broken. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Armour behind them** (`docs/events_todo.md`, 119): `armour_behind`, stages 4 and 5, walking at night: armour's clanks a few steps behind. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A breath in the chest** (`docs/events_todo.md`, 120): `breath_in_the_chest`, stages 3 to 5, indoors: a breath from the chest they use (a house sound, from the chest). Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Rain on the roof** (`docs/events_todo.md`, 121): `rain_on_the_roof`, stages 3 to 5, indoors with no rain: rain heard on the roof over them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Its builds follow the home** (owner: "the pillar only generated 1 block"): when the home moves,
  every build of its more than 96 blocks from the new home is let go (`Memory::home_moved`), so the
  pillar, its house and its copies begin again by the new home. Was: a build kept its first site
  for good; the pillar went on rising, a block a night as it should, three hundred blocks off by the
  old home, and the one by the new home never began. Tests: one in `memory`.

- [x] **Sniffing** (`docs/events_todo.md`, 122): `sniffing`, stages 4 and 5, at night outdoors: sniffing behind them, nearer or moving off. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A gasp behind them** (`docs/events_todo.md`, 123): `gasp_when_turning`, stages 4 and 5, walking at night: a sharp breath in behind them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Water where there is none** (`docs/events_todo.md`, 124): `water_where_none`, stages 3 to 5, indoors: swimming and splashes beside them, water under the floor. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Doors slam far off** (`docs/events_todo.md`, 125): `doors_slam_far`, stages 4 and 5, at night, a village known: doors slamming at once, far off. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **The count** (`docs/events_todo.md`, 126): `the_count`, stages 4 and 5, at dusk: a bell at the house rung once more each time, nine at most. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **The face under the water, the worn path** (owner: "nothing happens when i go to water";
  "path_worn ... set_block not done, the place holds jungle_planks"): `reflection` stays up to six
  seconds, gone once seen and looked away from; `path_worn` passes over the blocks they built (a
  porch's planks refused stopped the whole path, a refused block ending its event) and finds the
  grass two blocks up or down, only ever over grass. Tests: the reflection test checks its time
  and its going once seen too; one more for the path. Was: the face a second long, gone before it
  was found; the path stopped at their porch.

- [x] **Bones round the house** (`docs/events_todo.md`, 128): `rattling_round_the_house`, stages 4 and 5, indoors at night: skeleton steps all round the outside walls. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Chains dragging** (`docs/events_todo.md`, 129): `chains_dragging`, stages 4 and 5, underground: chains dragged behind them, nearer or going off. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A shriek below** (`docs/events_todo.md`, 130): `shriek_below`, stage 5, indoors: a shrieker's shriek sixteen blocks under them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A witch laughs** (`docs/events_todo.md`, 131): `witch_laughs`, stages 3 to 5, at night outdoors: a witch's laugh far off. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **The dog whines at the door** (`docs/events_todo.md`, 132): `pet_whines_at_door`, stages 3 to 5, at night: their dog held facing the door, whining. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Glass rubbed** (`docs/events_todo.md`, 134): `glass_rubbed`, stages 3 to 5, indoors at night: a slow squeak across a window (a house sound, from a pane). Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Bone underfoot** (`docs/events_todo.md`, 135): `bones_underfoot`, stages 4 and 5, walking indoors: bone's crunch with their steps a few seconds. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Bubbling under the floor** (`docs/events_todo.md`, 136): `bubbling_under`, stages 3 to 5, indoors: bubbling water under their floor. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A bell in the chest** (`docs/events_todo.md`, 137): `the_bell_in_the_chest`, stages 4 and 5, indoors: a small bell inside the chest they use. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Panting behind them** (`docs/events_todo.md`, 138): `panting_behind`, stages 4 and 5, running at night: panting behind them (the running need, new). Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **An iron door below** (`docs/events_todo.md`, 139): `iron_door_below`, stages 4 and 5, underground: an iron door slams sixteen blocks under them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Ice cracks** (`docs/events_todo.md`, 140): `ice_cracks`, stages 3 to 5, walking in a snowy biome: a crack under them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A bush rustles** (`docs/events_todo.md`, 141): `bush_rustles`, stages 3 to 5, walking outdoors: a hard rustle beside them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Snoring beside them** (`docs/events_todo.md`, 142): `snoring_beside`, stages 4 and 5, in bed: slow snoring beside them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **More than the music at the start** (owner: "it seems to ALWAYS pick cut_music, no matter what,
  at the start"; owner's pick): `pickup_pop`, `drip`, `place_sound` and `subtitle`, four of the
  faintest, come from stage one too. Not the seed: at the very start, nothing learned yet, the menu
  was `wait` and `cut_music` alone, every other stage-one event wanting a home, a chest or a door.
  Tests: one in `offer`. Was: the start's one choice was cutting the music.
- [x] **Every plan in the overlay** (owner: "make the debug ui show all the plans"): the F9 overlay
  lists every plan the brain holds, each with what it waits for, under its NEXT line
  (`DebugModel.planLines`). The F8 panel's Mind tab listed them already. They wait side by side,
  each sent the tick its own moment comes (`Brain::tick`, every plan on its own). Tests: one more in
  `DebugModelTest`. Was: the overlay showed the next plan alone.

- [x] **The mists seen with a shader pack on** (owner: "mist doesnt seem to do anything"): with Iris'
  shader pack on, the pack draws its own fog and leaves the game's unused, which is what `mist`,
  `fog_close` and `red_mist` move; so with one on they are drawn as a veil over the screen too, pale
  (red for the red mist), as thick as they have grown (`MistVeil`); and every mist has banks of big
  soft drifts low over the ground, seen with a pack or without. Tests: `MistVeilTest`. Was: with a
  pack on, a mist was a few motes of ash and nothing else.

- [x] **Red rain seen at once** (owner: "idk if the red rain is working"): it comes in over eight
  seconds (`Atmosphere.weightQuick`), not most of a minute; the hint came, but the drops were still
  barely pink when each new firing began it again. Tests: one more in `AtmosphereTest`.

- [x] **The footprints no longer cut the client's hints short** (found by the performance review):
  the `footprints` case left the whole client tick with a `return` seven ticks in eight, so every
  hint after it in the list and the restores at the tick's end were skipped; it draws every eighth
  tick now without leaving, and the prints read for a hint are let go when it ends. The mist veil's
  check for a shader pack is asked every two seconds, not by reflection every frame.

- [x] **A performance pass** (owner: "do a performance scan to see where we can speed things up,
  and if we even need to"): [Performance](performance.md) has the numbers. Was: nothing measured
  the mind's time; block scans made a string for every block's id and every wildcard; six
  entity sweeps and the saves all fell on one tick in a hundred; the hints were copied at every call
  from the render, hundreds a frame in red rain; dead redstone's box had no limit. Now the mind is
  timed by an ignored test (3.3 µs a tick, p99 5 µs), ids are kept per block, wildcards match in
  place, `Sweeps` spreads the sweeps over their own ticks (with a test), the hints are a snapshot,
  red rain's weight is read once in 30 ms, and a dead redstone box is no bigger than a scan. The
  host does not need Rust for speed.

- [x] **What it builds at its place stands on the ground there** (owner: "when i teleported to
  'hole', there was just a torch on the ground"; "sign not done, nothing holds it there"). Was:
  its place's height was the home's floor, and on lower ground the hole dug the air over the grass
  and left only its torch, the sign found nothing to stand on within three blocks, and a cairn or
  grave on a hill went into it. Now `hole`, `cairn`, `sign`, `grave`, `grave_each` and `sapling`
  send their blocks counted from the ground in their column (ABI 55: `set_block` and `set_sign`
  with a y below -1000), the host finding the ground; the view rule and the distance from what
  they built take those blocks at the home's floor.

- [x] **Every spot in the debug panel is a link** (owner: "make EVERY place with coordinates in
  the ui clickable", "like in the log page", "and also let me click when i fire, the little text
  that appears above"). Was: a line linked to its first spot alone, the Log tab's commands named
  no place, its sensed events wrote theirs as `x, y, z`, which no link read, the entity lines and
  the line at the top after Fire were never links. Now each spot of a line is its own link over
  its own characters, every command carried out names where it acted, sensed events and the
  nearest entity of each kind are written `(x,y,z)`, the mind's "forced" note says where, and the
  line at the top links its spot (`Coordinates.findAll`, `words`, `placeOf`, with tests).

- [x] **Eight more of the fifty** (`docs/events_todo.md` 127, 133, 143, 145, 151, 157, 164, 165).
  Was: no such events. Now `rumble` (a deep boom under them, the view shaking and dying away,
  `Rumble`, dust from the ceiling indoors), `boom_far` (at night a burst seen sixty-four blocks off
  toward its place, the boom from that way, a glow on the screen), `torches_turn` (the house's
  wall torches pointed at the bed, `turn_torches`, ABI 55), `your_grave` (a stone and a sign with
  their name and a day not come, at its place on the ground), `cobwebs` (up to six inside their
  upper walls, gone a day on), `false_door` (a door against an inside wall, gone a day on),
  `static` (a second of noise and a hiss) and `elder_face` (the elder guardian's face and cry, no
  curse). Tests: in `act`, `RumbleTest`, `FacingTest`.

- [x] **Eight more of the fifty** (`docs/events_todo.md` 144, 146, 147, 149, 150, 152, 153, 162).
  Was: no such events. Now `red_trail` (red specks from the door to the bed as they come home),
  `snow_shape` (a body's hollow in the snow by the house), `clock_wrong` (a clock showing night at
  noon, `ClockTimeMixin`), `cold_house` (their breath showing, frost on the floor a minute),
  `the_killer_rabbit` (the game's killer rabbit at them, spared; `minecraft:killer_bunny`, ABI 55),
  `phantom_item` (a paper named "behind you" in an empty slot for three seconds),
  `border_closes` (the world border close round them on their client a moment) and
  `frost_on_the_glass` (their panes white from inside a minute and a half). Tests: in `act`.

- [x] **Never a kill, made a setting** (owner: "its also kinda silly to have the never a death
  rule, maybe we should make that toggleable in the lodger settings"; "make the death setting
  true by default"). Was: every hit, burn, arrow and squeeze it dealt, and every hostile it set on
  them, left them half a heart, always. Now `events.kills`, on by default, lets them land in full,
  a death from one the game's own; off, as before. The host learns it with the opened world
  (`Mod::spares`, the open answer's `spare`, ABI 55) and from a `spare` command whenever the
  setting moves, and its one `Spare` gate lifts every clamp and the refused death of a spared
  hostile's hit. The stages still decide what may harm, and the fight keeps its own death. Tests:
  `SpareTest`, in `stay`.

- [x] **An `extreme` preset** (owner: "please add another preset, EXTREME for many many events and
  such per day"). Was: `relentless` the most, four to twelve events a day. Now `extreme`: 10, 16,
  24, 32 and 16 a day by stage, moving in at once, stages a day or two, cooldowns a fifth, eight
  plans held, asking every fifteen seconds, twelve offered (the most the menu holds), acting at 0.15, keeping 24 blocks
  off, temper resting at 0.6, the death at the end on every difficulty. In the settings' preset
  choice. Tests: in `config`.

- [x] **Five more of the fifty** (`docs/events_todo.md` 148, 158, 159, 161, 163). Was: no such
  events. Now `shadow_on_the_wall` (a person's shadow behind them while they hold a light,
  `Silhouette`), `stairs_up_to_nowhere` (oak stairs up an inside wall into the ceiling, gone a day
  on), `face_in_the_canopy` (eyes and a mouth cut in the leaves of the tree nearest the house,
  facing the door), `boots_at_the_door` (boots like theirs on an unseen locked stand inside the
  door, ABI 55) and `nightmare` (a frame of their own face over the screen as they wake). Tests:
  in `act`, `SilhouetteTest`.

- [x] **The last four of the fifty** (`docs/events_todo.md` 154, 155, 156, 160). Was: no such
  events. Now `ceiling_crawl` (upside down across the ceiling over them, the path hung under the
  ceiling), `in_the_lava` (in the nearest lava, unburnt), `over_the_edge` (a head over the cliff
  edge above them) and `rider_on_the_ridge` (on a skeleton horse far off at dusk); the figure's
  `finds` (`lava`, `cliff_top`) and `mount`, ABI 55, the host finding what the mind cannot know.
  All fifty of 116 to 165 are built. Tests: in `act`.

- [x] **Recommended addons and shader on the Modrinth page** (owner: "put these mods into
  mod/modrinth.md under a recommended addon/shaders sections"; "take out the celestials one").
  Was: none named. Now `mod/modrinth.md` lists the owner's set, by what each is for, and
  Hysteria with Iris.

- [x] **craftkit as a crate for crates.io** (owner: "can you create a seperate crate for craftkit
  that i can upload to crates.io"). Was: `sdk/` marked `publish = false`, no readme, keywords or
  license file of its own, its docs pointing at paths in this repository, and the workspace's
  `repository` naming another address than the real one. Now it is publishable as `craftkit`
  (the name is free): `sdk/README.md` (a whole mod and its harness test, run as a doctest) is its
  crates.io page and its docs' front, `sdk/LICENSE`, keywords and categories, `include` keeping
  the upload to the crate, and the repository `github.com/germ3n/lodger`. How to publish is in
  [The SDK](sdk.md#versions-and-publishing). Not published: the owner uploads it.

- [x] **craftkit's own repository** (owner: "make a git project folder kind of thing for the host,
  upload to github, update the craftkit docs to point there"; the SDK and the host together,
  owner's pick). Was: the host lived only here, and the crate named the lodger's repository. Now
  `scripts/export_craftkit.py` writes `sdk/`, `host/`, the SDK guide (its links to the lodger's
  pages made absolute), the license, the toolchain, a workspace, a README and a .gitignore to
  github.com/germ3n/craftkit, which builds and tests on its own; the crate's `repository`,
  `homepage` and README point there. The host's five tests about the lodger skip where it is not
  beside the host (`Repo.assumeLodger`); here they run as ever.

- [x] **Keeping craftkit's repository up to date** (owner: "when something changes here is it
  possible to update it in the repo as well? (on the sdk side)"; a script, the owner's pick over
  a GitHub Action or a standing rule). Was: re-exporting, committing and pushing by hand. Now
  `scripts/sync_craftkit.sh` does all three, with lodger's last SDK or host commit's subject,
  cloning craftkit first where there is no clone, and committing nothing when nothing changed.

- [x] **The 7B and 8B models** (owner: "please add the 7b and 8b models? also wire it up to the
  model selection, download screen, all that"). Was: four sizes, the largest the 4B. Now
  `model.size` `7b`, Qwen2.5-7B-Instruct (Qwen's own `q4_k_m`, which Qwen publishes in two
  parts: two resources, saved side by side under the split's names, each checked by its own
  length and SHA-256, the license asked once), and `8b`, Qwen3-8B (Qwen's own `Q4_K_M`, one file),
  both Apache-2.0, in the preparing screen's choices with their notes, the settings page, the
  config's checks and the docs. Tests: in `config` and the crate's. Tests changed (owner told):
  the counts of sizes, 4 to 6; the unknown size refused is `14b` (it was `7b`); the sizes are
  ordered by every part together.

- [x] **The model reads the situation: the frame and the scoring** (owner: "should this mention
  the never chase, never hurt thing?", "maybe theres a better prompt here", "do look into the
  menu wording and scoring", "you should calibrate for both 4b and 1.5b"). Was: the frame told
  the model, as "you", that it never chased or hurt and acted only while they were away or
  asleep, beside lines that say "it"; and the question ended `The answer is: ` with the choices
  read without their space, so the 1.5B and the 4B both answered "while they are away" to a
  player asleep in bed. Now the frame is told of "it", with a sentence for its stage
  (`situation::frame`), and choices are read as Qwen's tokens carry them (`spaced`), which got
  every obvious answer right on both models; four ways of scoring were measured
  (`core/tests/frames.rs`, `MODEL`, `ONLY`, `GPU=1` with `--features metal`), the table in
  [Model](model.md#scoring). Left: the menu's words (one to fifteen words; short, common ones
  win). Tests changed (owner told): the situation's frame text and the brain's "What it knows".

- [x] **The menu picked to fit the situation** (owner: "do this please", of shortening the menu's
  words). Was: one common phrase (`knock on the door`) won every menu, on the 1.5B and the 4B,
  whatever the situation. Measured first: shortening the words changed nothing (0 of 18 fits
  either way), so the words were kept; the cause was scoring a choice by the mean over its
  tokens, an idiom's tail all but certain. Now a menu (more than five choices) is lettered: the
  letter answered is scored, the list asked in three orders and averaged; the short questions
  stay `spaced` (`mixed`, the default). Fits went from 0 to 6 of 18 on the 1.5B and 0 to 12 on the
  4B, every obvious `When?` right on both, and a three-day stay waits when waiting fits. Numbers in
  [Model](model.md#scoring); the half mode measured, correctly named (it was also listed).

- [x] **Phi-4 mini and the Granites** (owner: "can you add these 2"; "can u also add 7 or 8b
  variants of those"). Was: Qwen alone. Now `model.size` `phi4-mini` (Microsoft's
  Phi-4-mini-instruct, MIT, from unsloth's repository as Microsoft publishes no GGUF),
  `granite4-micro` (IBM's Granite 4.0 Micro) and `granite4-tiny` (IBM's Granite 4.0 H Tiny, 7B,
  about 1B at work a word), both from IBM's own repositories, Apache-2.0; each file's length and
  SHA-256 from Hugging Face, each architecture checked in the vendored llama.cpp. Phi-4 has no
  7-8B. In the preparing screen's choices, smallest first, the settings, the docs. Phi-4 mini and
  Granite Micro measured against the two Qwens (owner's pick; [Model](model.md#scoring)): the
  Qwen3 4B still the best reader; neither the default. Granite H Tiny not measured (a download). Tests: in `config` and the crate's; the counts of sizes, 6
  to 9, and the labels looked up by name, not place (owner told).

- [x] **The Modrinth page caught up** (owner: "you updated the modrinth desc ye?"). Was: it said
  it never kills before the very end (untrue since `events.kills`, on by default) and never
  chases, and its spoilers had none of the fifty newest events. Now the rule reads as it is, the
  chasing waits "not for a long time", the spoilers name the woods, the villagers, what it sets
  on you, the senses going wrong and the things taken into the sky, and the model section says
  the Qwen3 4B reads best and names the other makers'.

- [x] **The mist no whiteout** (owner: "the fog got really dense and i can hardly see anything",
  "weird smoke/steam particles all over the ground", "i am not using shaders rn"). Was: the mist's
  fog closed to thirteen blocks, and banks of big dust drifted over the ground with or without a
  shader pack, though they were there for a shader pack's sake (whose own fog hides the game's):
  without one, fog and banks together left nothing to see. Now the mist's fog ends at twenty-two
  blocks, and the banks drift only with a shader pack on (`MistVeil.banks`, with a test).

- [x] **A double chest read whole** (owner: "its saying my chest holds nothing even tho theres
  stuff in there", "i have 2 chests tho, 1 empty and 1 not", a double chest). Was: the host read
  the half clicked alone, 27 slots, so what lay in the other half was never seen (after a stack
  moved, the chest "held nothing"), each half was its own record, and the lodger's moves and
  takes reached one half. Now the host reads a double chest whole, 54 slots as the screen shows
  them (`Senses.containerAt`), reports it always at the same half (`ChestHalves`, with a test)
  with its slots (`size`, ABI 55), and every command on a container uses the whole; the mind keeps
  the size (`Container::slots`), its empty slots and tidying over all of them. Tests: in `memory`,
  `ChestHalvesTest`.

- [x] **A sigh at the door** (`docs/events_todo.md`, 166): `sigh_at_the_door`, stages 3 to 5: in the house, a slow sigh just outside a door of theirs. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Humming ahead** (`docs/events_todo.md`, 167): `humming_ahead`, stages 3 to 5: in a tunnel, a few low notes hummed ahead of them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **Their steps counted** (`docs/events_todo.md`, 168): `steps_counted`, stages 4 and 5,
  walking at night: eight notes behind them a step apart, each higher up the scale. Tests: one in
  `act`. Was: no such event.

- [x] **A cart overhead** (`docs/events_todo.md`, 169): `cart_overhead`, stages 3 to 5: underground, a minecart rolling across the rock above them. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [x] **A ram on the wall** (`docs/events_todo.md`, 170): `ram_on_the_wall`, stages 4 to 5: at night, three heavy thuds against a wall of the house. Tests: one in `act`, its ways in
  the sound variants test. Was: no such event.

- [ ] **The host in Rust, behind a thin Java wrapper** (owner): what decides or
  keeps state in the host (the timers, holds, loot offerings, reads, the debug model) moved
  into Rust beside the SDK, the Java left as the Minecraft-facing edge alone (the loaders'
  hooks, the game's API calls, rendering), since the game's own API is Java.

- [x] ~~**Split the project into two repositories**~~ (owner: cancelled, "nevermind on the splitting the repo") (owner): one for the SDK
  (`sdk/`, the craftkit host in `host/`, their docs and CI) and one for the lodger (`core/`,
  `mod/`, its docs), the lodger depending on the published SDK crate and host jar.


- [x] **A license and notices** (owner; no split now): the MIT license was already
  here (`LICENSE`); `gen_notices.py`, adapted from `~/chainv4`'s, writes
  `THIRD_PARTY_NOTICES.txt` from `Cargo.lock` (every crate, the Apache text once, the other texts
  the lock requires), llama.cpp's license from the crate that bundles it, and the model's
  licenses named, since it is downloaded and not shipped. Run it after a dependency changes.