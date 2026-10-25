# Queue

Where the work stands and what the next session picks up, written for a session that starts
knowing nothing of how it came about, as `~/chainv4/docs/queue.md` is. Written 2026-11-09 at the
end of the day the project started; check what has moved since with `git log`.

## Rules for every session

- **Read first:** [Good to know](good-to-know.md), [The SDK](sdk.md), the conventions in
  [Development](development.md), and the top of [Todo](todo.md). If a rule here and a habit of
  yours differ, this page wins.
- **Ask before:** committing (every time), accepting any terms on the owner's behalf (the model's
  license, Mojang's EULA for a dev server), downloading anything large, and any real design
  choice, with the option you recommend first.
- **Every change** lands with its tests and its docs: the page for the part,
  [Good to know](good-to-know.md) for what would surprise someone, and a dated `[x]` in
  [Todo](todo.md) with the old text kept after `Was:`. Report exactly which checks ran and how
  they came out. Never weaken a test to make it pass.
- **Commits:** author `cherrymaraschino@protonmail.com` (the repo's `git config` has it), a
  lowercase first line saying what it lets the lodger or a player do, then `- area:` bullets,
  then `- docs:` and `- tests:`. No attribution trailers of any kind.
- **The mind knows nothing of Minecraft; the host knows nothing of the lodger.** A new
  capability is a protocol command any mod could send and an executor written for the general
  case. Every Minecraft-facing line of the host lives in the places
  [Updating Minecraft](development.md#updating-minecraft) lists.
- **Checks before reporting:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  -- -D warnings`, `cargo test --workspace`, and `cargo test -p lodger-core -- --ignored` when
  `core/models/qwen.gguf` is there; `./gradlew build` in `host/` (JUnit, including the
  real-native tests when `target/release/liblodger_core.dylib` is built); `./gradlew jar` in
  `mod/`. The plain-server smoke test below for anything that touches the host's edge.
- **Machine notes:** Gradle needs `JAVA_HOME=/opt/homebrew/opt/openjdk@25/libexec/openjdk.jdk/Contents/Home`.
  The downloaded model is at `core/models/qwen.gguf` (gitignored). The scratch Fabric server used
  for smoke tests was in the session's scratchpad and is gone with it; make one again with the
  Fabric server launcher for 26.3, Fabric API in `mods/`, `eula.txt` (ask), the lodger jar, and
  `config/craftkit/accepted.json` holding `lodger:model` (ask).

## Where it stands

Phases 0 to 2 of [Todo](todo.md) are done and phase 3's host side is in:

- **The Rust side** (`sdk/`, `core/`) builds, with 64 tests and 4 more against the real model.
  The SDK speaks ABI 4; the lodger's mind has its config with presets and sizes, the model
  layer (scoring with seeded picking), and a `Stay` that opens, ticks and saves. It has no
  memory, brain, events or arc yet: a stay today only counts.
- **The host** (`host/`) builds for Fabric and NeoForge with 42 JUnit tests: loading content
  jars, the download with the license screen, the per-world loop, the senses (players' full
  state and movement, entities, weather, events from both loaders' hooks, chat, scans, a
  surroundings pass in slices), executors for log, message, sound, hint, scan, surroundings,
  toggle, move and set block, move and drop items, sign, sleep, look. Not yet: the figure, the
  glitch renderers (hints reach the client and are logged), the menu, `/lodger` commands.
- **The lodger's jars** (`mod/`) build with the host nested, one per loader. Six natives come
  from the CI; a developer's jar has the host machine's.
- **Seen working on a plain Fabric 26.3 server** with only the lodger jar and the Fabric API:
  the nested host loads, Qwen's terms are logged and then accepted by the owner, the 491 MB model
  downloads with progress, loads, three worlds open from saved data and save every five seconds.
  Not yet seen: a client (the title screen, the license screen, the hints), NeoForge at runtime,
  a player's observations (the server had no players).

## Next, in order

1. **A client run.** `./gradlew :fabric:runClient` in `host/` with the lodger jar in the run's
   `mods/`: the license screen, the download bar, the title screen released, `/craftkit status`
   in a world, the observations of a real player (log them once behind a debug flag), the hint
   path with a `hint` command from a test mod or a temporary lodger line. Fix what the client
   shows that the server could not. Then NeoForge the same.
2. **Phase 3's mind side:** `events.rs` as data (the catalogue's rows with needs, cooldowns,
   caps), the `Memory` with its bounds and `describe`, the surroundings and activity and
   movement pieces fed from the observations, `brain/offer.rs` with the six filters, the rule
   brain, the decision log, and `Stay` doing the per-tick work. Scripted-stay tests in
   `core/tests/stays.rs`. This is where the lodger starts to exist.
3. **The menu** (phase 2's last item): `Menu` in the protocol, the host's command tree and
   screen, the lodger's three pages.
4. **Phase 4:** the brain on the model, the want and target asks, calibration round one.

## Open questions for the owner

- The crate's final name (`craftkit` is a working name; it was free on crates.io on 2026-11-09).
- Whether the lodger's `/lodger ...` commands stay as the README has them or become
  `/craftkit lodger ...` under the menu alone.
- The moon phase and the time of day: 26.3 moved time to world clocks; the senses send game
  time's remainder and moon phase 0 until the clock API is read properly (a todo line).
