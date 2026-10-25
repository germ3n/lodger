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
  [Upgrading](upgrading.md) lists.
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

Phases 0 to 2 of [Todo](todo.md) are done, phase 3's host side is in, and the mind has its
second cut (2026-11-09, evening):

- **The Rust side** (`sdk/`, `core/`): the SDK speaks ABI 5 (the per-world calls and the debug
  call); the lodger has its config (presets, three sizes, 1.5B the default), the model layer
  (scoring with seeded picking; generation is phase 4), a memory (home, containers by kind,
  fixtures, decor, creatures, favourites, held, done, landmarks, milestones, the kill tally,
  chat), 87 events as data over the host's commands, the rules, the arc, the rule brain, the
  text banks by kind, the decision log, the debug pages, and a `Stay` that plans, acts and saves
  as JSON (format 3). 85 unit tests, 4 scripted stays, 4 more against the real model.
- **The host** (`host/`): builds for Fabric and NeoForge with 50 JUnit tests. The senses (every
  player's state and movement, entities, weather, events from both loaders' hooks including
  kills, chat, scans, a surroundings pass), 24 executors, the client's glitch renderers for
  eleven hint kinds, the debug screen with its key and commands, the title gate and the
  download screen.
- **The lodger's jars** (`mod/`) build with the host nested, one per loader.
- **Seen working:** a plain Fabric 26.3 server with only the lodger jar (the download, the
  license, the worlds); a dev client through stage two with the first events landing. Not yet
  seen: the debug screen and the new events in a client (built, not yet opened by a player),
  NeoForge at runtime.

**Uncommitted at the cut-off (2026-11-09, late):** everything from the first client run's three
fixes on: the second catalogue cut, the debug screen, the glitch renderers, the kill tally,
the landmarks, the text banks, the new defaults, the JSON save, the upgrade guide. All checks
green (below). Ask the owner, then commit; the owner wanted one commit per phase, so this is
"the catalogue's second cut and the debug screen" or, at their word, two.

## Next, in order

1. **Open the debug screen in the dev client** and fire every event once from the Events tab,
   fixing what the world shows (the chest lid, the frames, the paintings, the stands, the
   player list entry, the chat lines, each glitch). Then NeoForge the same. The owner was
   away from the laptop when this was built; nothing in it has been seen by a player yet.
2. **The catalogue, third cut** (owner, 2026-11-09: "start coding all the events"): the
   surroundings in the mind first (the todo's item), then the rows it unlocks (water, the
   village, the field, the pen, the mine), the routes and the rows over them, redstone, the
   remaining sounds and glitches (fog, the painting flipped, the compass), the reactions and
   hours pieces of the memory, the temper and the chains, the six remaining scripted stays.
3. **The config page** of the debug screen (phase 2's last piece): every key of the mod's
   config, written back.
4. **Phase 4:** the brain on the model, the want and target asks, generation with the banks as
   examples, calibration round one.
5. **Phase 5:** the figure in the target's skin and its renderer, the rows that need it.

## Open questions for the owner

- The crate's final name (`craftkit` is a working name; it was free on crates.io on 2026-11-09).
- Whether the lodger's `/lodger ...` commands stay as the README has them or become
  `/craftkit lodger ...` alone, where the debug commands already are (`show`, `force`, `stage`,
  `action`).
- The tone of the lines in the target's own name (`brain/text.rs`, `ChatLine`): lower case, no
  punctuation, as a player types, is the current choice.
- Whether the figure (phase 5) wears the target's skin by default on a server, where other
  players would see a second copy of someone, or only in singleplayer.
- The moon phase and the time of day: 26.3 moved time to world clocks; the senses send game
  time's remainder and moon phase 0 until the clock API is read properly (a todo line).
