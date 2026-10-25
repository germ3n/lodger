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
  for Fabric and NeoForge; Rust mods as content jars with no Java, one jar for both loaders; the
  host a mod of its own. See [The SDK](sdk.md). Was: the lodger as a Fabric mod with its own
  Java body.
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
- [ ] **The CI runs green:** rustfmt, clippy and `cargo test` on three runners with the natives
  uploaded, the host built for both loaders with its tests, the content jar assembled with all
  three natives. Written (`.github/workflows/ci.yml`); the first push proves it.
- [ ] **`THIRD_PARTY_NOTICES.txt`** generated and checked in CI (see
  [Development](development.md#third-party-notices)).

### Phase 1: the model layer

- [ ] **Port `~/engine/game/base/src/ai/` into `core/src/model/`:** `mod.rs`'s validation and
  scoring math, `queue.rs`'s worker and keys, `download.rs`, and `infer.rs` from the `infer`
  module, with `crate::fs` replaced by a plain path and the Lua library left behind. The engine's
  unit tests come along. `llama-cpp-2` at the engine's version (0.1.158) to start.
- [ ] **The `Scorer` trait, the `Echo` scorer, and the queue tests:** returns at once, calls back
  from the tick, supersedes by key, refuses when full, reports a missing model without calling
  back. All from the engine's `script/libs/ai.rs` tests, rewritten against the C ABI.
- [ ] **The download through the ABI:** `/lodger model download`, the GGUF check, the `.part`
  file, one at a time, the finished message back on a tick. Test against a local HTTP server as
  the engine does.
- [ ] **The real-model tests,** ignored by default, with the model at `core/models/qwen.gguf`
  (`.gitignore`d): a known situation scores the expected way; two questions on one situation
  agree with the same two asked alone (the rewind is right).
- [ ] **Measure.** Time and resident memory for a 300-token situation with four questions of
  four choices on two threads, on a laptop and on a small VPS. Put the numbers in
  [Model](model.md) in place of "tens to a few hundred milliseconds".

### Phase 2: the mind without a model

- [ ] **The SDK's world calls.** `Mod::World`, `open`, `observe`, `tick`, `report`, `save`,
  `close` in the trait and `export_mod!`; the JSON shapes for observations, `now`, commands and
  reports in `protocol.rs`, from the lodger's needs; the host's `world/` running them once a
  tick and keeping the bytes in saved data; the harness driving a world with scripted
  observations. ABI version 2.
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
- [ ] **`Stay`, the lodger's `World`.** `bincode` behind a format byte. A stay that only ever
  waits, saved and reopened.
- [ ] **Memory.** Every piece in [What is kept](memory.md#what-is-kept) with its bound, and the
  test that a million observations stay under 256 KiB. Routes' downsampling and matching
  (which walks are "the same way"). The surroundings and the activity, from the host's daily
  scan and the last minute of observations.
- [ ] **The offer** (`brain/offer.rs`): the six filters of
  [What is offered when](entity.md#what-is-offered-when), with a test per filter (no water, no
  boats; asleep, no `late_steps`; stage one, nothing from stage two) and a test that a hundred
  draws from one situation offer every eligible event at least once.
- [ ] **`Memory::describe`.** The twelve lines, with tests on the exact wording, and the
  truncation from the bottom at a measured token count. Token counting needs the model's
  tokenizer; without a model, count words times 1.4 and say so in the log.
- [ ] **Events as data, the rules, the arc.** `events.rs`, `rules.rs`, `arc.rs` as the docs have
  them. The rule brain with its weight tables. The decision log.
- [ ] **Scripted stays** (`core/tests/stays.rs`): the `Player` script and the first ten tests in
  [Tests](development.md#tests), run on the rule brain. A whole stay in under a second.
- [x] **Design decisions taken with the owner (2026-11-08):** Rust mind and Java body through
  FFM; `lodger` as the name; the one death on hard kept as the only harm; the denser pace
  (budgets 1, 2, 4, 6 a day, a stay of about two in-game weeks with forty to sixty events) over
  the rarer one (half that over three weeks).

### Phase 3: the body, stages one and two

- [ ] **Senses:** position and facing once a second, containers opened with contents, blocks
  placed and broken near home, sleep and refused sleep, logins and logouts, dimension changes,
  deaths, and what the target is doing (mining, farming, building, fishing, in a boat) read from
  those. Sent as observations each tick; a tick with nothing new sends nothing.
- [ ] **The surroundings scan:** once an in-game day, within 128 blocks of the home, on the
  server thread in slices of a few chunks a tick so it never shows in a tick profile: water,
  snow and sand, a village, fields and pens, the target's tunnels and unentered caves, and the
  house's furniture by kind. Sent as one observation.
- [ ] **Executors for the stage one and two events** of the [catalogue](entity.md#events), about
  fifty, each with `fits`, each with a game test. Most are one of a dozen general commands
  (move an item, move a block within reach, toggle a block, set an entity's look, play a sound
  to a player, send a hint), so the count of executors is far below the count of events; the
  catalogue grows by rows in `events.rs`, not by Java. Containers first (`open_chest`,
  `shuffle_chest`, `take`, `return_taken`, `place_taken`), then furnaces, doors, lights, sounds,
  then the rest. The lodger's own torch and sign blocks, vanilla-looking.
- [ ] **Budgets, cooldowns, caps and the menu** (`events.rs`, `brain/offer.rs`) with scripted-stay
  tests: a stay of fourteen days on the rule brain holds between forty and seventy events, no
  event twice in a row, none past its cap, none inside its cooldown.
- [ ] **The net:** the sound packet, the hint packet (kind, place, duration), the target flag;
  the client handlers. `late_steps` plays the right footstep sound for the block; `flicker` and
  `painting_flip` are the first two glitches, so the hint path is built here and the rest come
  in phase 5.
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

- [ ] **`settle`** and the three endings, including the one death on hard with inventory kept and
  the vanilla message, and the `left` and `stayed` states. Scripted-stay tests for each.
- [ ] **Multiplayer:** the `Others` memory, the target ask, witnesses' traces and nothing else,
  retargeting once. Tested with two dev clients on a dev server.
- [ ] **Performance pass:** the per-tick JSON exchange measured (it should be well under 50 µs
  on a quiet tick), the senses' distance checks bounded to the target's chunk neighbourhood,
  the model's unload timer. Nothing the mod does should show in a tick profile on a server
  with the lodger idle.
- [ ] **A proper release:** the jar with three natives, `THIRD_PARTY_NOTICES.txt`, the
  README's install steps checked on a fresh machine on each platform, a Modrinth page that says
  what the mod records and downloads. Version `0.1.0`.

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
