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
- [ ] **The menu** (owner, 2026-11-09): `Menu` in the protocol (pages, fields, actions), the
  host rendering it as `/craftkit <id> ...` commands and as a client screen, operator-only,
  with the config write-back; `MenuAction` commands back to the mod. The lodger's three pages
  (status, config, debug) declared in `core/`, with the debug actions (force stage, force event,
  dump memory, rule brain on, replay last decision) wired as the stay grows. Tests: the command
  tree generated from a menu matches the pages; a field change round-trips to the config file.
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
  [Which model](model.md#which-model).
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
  ABI and direct.
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
- [ ] **The offer** (`brain/offer.rs`): the six filters of
  [What is offered when](entity.md#what-is-offered-when), with a test per filter (no water, no
  boats; asleep, no `late_steps`; stage one, nothing from stage two) and a test that a hundred
  draws from one situation offer every eligible event at least once.
- [ ] **`Memory::describe`, in full.** The twelve lines of [Describing](memory.md#describing)
  with tests on the exact wording, and the truncation from the bottom at a measured token
  count (the model's tokenizer; without a model, words times 1.4). The first cut writes eight
  of the lines.
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

### Phase 4: the model in the loop

- [ ] **Wire the brain to the worker:** the asks with their keys, the callbacks in the tick, the
  thresholds, the fall-back to the rule brain on refusal or no model.
- [ ] **The want and the target asks,** once each at move-in.
- [ ] **Calibration, round one.** Ten stays through stage two by different players (or the same
  one playing differently: a tidy player, a careless one, one who chases), the decision log read
  end to end, and the prompts, choices' wording and thresholds moved where the choices read
  wrong. Read them for repetition above all: anything that happened three times in a stay, or
  twice in a row, is a cooldown or a cap to move. Keep the before-and-after logs under
  `docs/calibration/` with a paragraph on what moved and why.
- [ ] **The phrase bank and `brain.text`** (owner, 2026-11-09: "use the model for sign text"):
  `brain/text.rs` with a few hundred lines by want, stage and place; the `What does the sign
  say?` ask over eight drawn lines; and the `generate` experiment: a letters-and-spaces token
  mask in `infer.rs`, three words at most, a blocklist, a log line, the bank as the fallback.
  Judged in calibration round two: does `generate` read better than the bank over ten stays?
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

- [ ] **A debug UI for events, senses and the brain** (owner, 2026-11-09): once the phases are
  in, a proper in-game screen over the menu's debug page: the live observations as the mind
  sees them (the players' lines, the entities, the movement reading), the memory's lines, the
  last decisions with their menus and confidences, the open chains and the temper, and buttons
  to force an event, a stage or a want, with the world's reaction shown as it lands. Built on the
  menu system, so any Rust mod gets the frame and the lodger fills it.
- [ ] **A Minecraft version bump, rehearsed:** take the host to the next snapshot once, by the
  checklist in [Updating Minecraft](development.md#updating-minecraft), and fix whatever the
  checklist missed.

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

- **Letting the model write the signs.** Three words from a 0.5B model about a player's chest are
  three words of nonsense a third of the time, and once it writes one thing a player finds
  cruel the mod is over. The fixed word lists are the design, not a limitation. The model picks
  *which*; it never writes.
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

Nothing yet. The docs were written 2026-11-08 from `~/chainv4`'s style and `~/engine`'s AI
layer; the first `[x]` is the first build.
