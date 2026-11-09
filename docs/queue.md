# Queue

Where the work stands and what the next session picks up, written for a session that starts
knowing nothing of how it came about, as `~/chainv4/docs/queue.md` is. Check what has moved
since with `git log`.

## Rules for every session

- **Read first:** [Hand-off](handoff.md) if this is your first session here, then
  [Good to know](good-to-know.md), [The SDK](sdk.md), the conventions in
  [Development](development.md), and the top of [Todo](todo.md). If a rule here and a habit of
  yours differ, this page wins.
- **Ask before:** committing (every time), accepting any terms on the owner's behalf (the model's
  license, Mojang's EULA for a dev server), downloading anything large, and any real design
  choice, with the option you recommend first.
- **Every change** lands with its tests and its docs: the page for the part,
  [Good to know](good-to-know.md) for what would surprise someone, and an `[x]` in
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
  `core/models/qwen-1.5b.gguf` (or `qwen.gguf`) is there; `./gradlew build` in `host/` (JUnit, including the
  real-native tests when `target/release/liblodger_core.dylib` is built); `./gradlew jar` in
  `mod/`. The plain-server smoke test below for anything that touches the host's edge.
- **Machine notes:** Gradle needs `JAVA_HOME=/opt/homebrew/opt/openjdk@25/libexec/openjdk.jdk/Contents/Home`.
  The downloaded models are at `core/models/qwen-1.5b.gguf` (the default size, 1.1 GB) and
  `core/models/qwen.gguf` (the 0.5B), both gitignored; the tests prefer the 1.5B. The dev
  client has both under `runs/client/config/lodger/models/` and runs the default. The scratch Fabric server used
  for smoke tests was in the session's scratchpad and is gone with it; make one again with the
  Fabric server launcher for 26.3, Fabric API in `mods/`, `eula.txt` (ask), the lodger jar, and
  `config/craftkit/accepted.json` holding `lodger:model` (ask).

## Where it stands

The lodger is whole and has been played in the dev client:

- **The mind:** the whole catalogue coded (299 rows in `core/src/events.rs`), with the five
  stages, the three endings and the fight, the chains, the temper, the other house, the books,
  multiplayer (the pick, retargeting, witness traces) and the Nether's and the End's two events.
  The model decides (the planning questions, the reactions, the want and the target, on its
  worker, the rule brain as the fallback) and writes its few words; the four sizes and the
  presets are in the config.
- **The host:** an executor for every command of the protocol (ABI 52; `core/tests/host_carries_out.rs`
  checks none is missing), the glitch renderers and the atmospheres, the figures, the
  download screen with the license, the title screen's settings page, the debug panel and the
  HUD (F8, F9; `/craftkit lodger debug|show|force|stage|action|status|leave|reset`).
- **The build:** both jars with six natives (Windows and Linux on Vulkan, macOS on Metal,
  x86_64 for AVX2), by `scripts/build_jars.py` on a Mac or by the CI, run by hand
  ([Building](build.md)).
- **Bug passes:** ten so far, each over every module, each with its fixes and tests in the todo.

## Next, in order

Bug passes first, then the open `[ ]` entries of [Todo](todo.md), in short:

1. **Bug passes until one finds nothing,** each with its fixes and tests in the todo.
2. **The CI run green once,** by hand; `THIRD_PARTY_NOTICES.txt` checked there.
3. **Play it** through stage four with the model; **calibration round two** by
   [round one](calibration/round-1.md)'s list; **measure** time and memory; the chat template
   and Qwen3-0.6B questions.
4. **A proper release:** the install steps checked on fresh machines on each platform.
5. **The config page** of the debug screen: every key, written back to the file.
6. **Game tests** for the executors and for `tab_entry`'s packet; **a version bump rehearsal** by
   [Upgrading](upgrading.md); the host comparing content jars across sides in the handshake.

## Open questions for the owner

- The crate's final name (`craftkit` is a working name; it was free on crates.io).
- The lodger's commands: decided, all under `/craftkit lodger ...` (`status`, `leave`, `reset`
  beside the debug ones, `show`, `force`, `stage`, `action`).
- The tone of the lines in the target's own name (`brain/text.rs`, `ChatLine`): lower case, no
  punctuation, as a player types, is the current choice.
- Whether the figure wears the target's skin by default on a server, where other
  players would see a second copy of someone, or only in singleplayer.
- The moon phase and the time of day: 26.3 moved time to world clocks; the senses send game
  time's remainder and moon phase 0 until the clock API is read properly (a todo line).
