# Queue

Where the work stands and what the next session picks up, written for a session that starts
knowing nothing of how it came about, as `~/chainv4/docs/queue.md` is. Written at the
end of the day the project started; check what has moved since with `git log`.

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

Late on the second day. Seven commits since the mind's first cut, one per finished item:

- **The debug screen and the HUD** (F8, F9; `/craftkit lodger debug|show|force|stage|action`):
  senses as meters, the mind's gauges, a Story section telling each decision with its menu,
  its situation and its outcome, every event with a Fire button, the log. Built but not yet
  opened by a player; the first version's tabs took no clicks (fixed the same evening).
- **The mind:** 134 events coded; the memory has the surroundings (the host's pass once a day,
  with structures by tag), vehicles, the village and ore scans, routes, sessions, reactions,
  landmarks, milestones and the kill tally; `events.sounds/glitches/chat/figure` gate their
  groups; the model decides (the four planning questions, the reactions on triggers, the want
  and the target, asked on the worker and answered on a later tick, the rule brain as the
  fallback) and writes the few words (a masked greedy decode, checked, the bank as the
  fallback, or the bank ask in `bank` mode); the figure in the target's skin as client-only
  packets with eight rows; the house changed (a block for another of its family, the bed
  breaking). 101 unit tests, 10 scripted stays, 46 JUnit, a calibration run; clippy clean.
- **The host:** 30 executors, 14 glitch renderers, the figures, the poses, the timers.

**Not yet seen by a player:** everything above. The dev client was relaunched with the latest
jar at the end of the session and left at the title screen.

## Next, in order

1. **Play it, with the owner at the keyboard.** F8, every tab; fire every event from the Events
   tab and watch the world and the Log tab; read `config/lodger/decisions.log` for the model's
   answers (`brain=model`) and its written lines (`wrote (...)`); the figure (`far_figure`,
   `in_doorway`). Fix what shows. Then NeoForge (`:neoforge:runClient`) the same.
   **Asked during the first session, next after its fixes:** (done the same night:
   parallel plans, cancelling, the side panel; see the todo) plans in parallel
   (`brain.most_plans`, 3 by default, presets scale it; asking again every `plan_every` while
   below it; never two on one spot or container, never two of a group, never one event twice);
   plans cancelled as soon as they cannot happen (the spot gone, the needs unable to return
   within the window, a stronger reaction at the cap) with the reason in the log and on the
   screen; the debug screen as a side panel that neither pauses nor covers the world, smaller
   text, every plan a row with Drop and Now, waiting reasons shown; an audit that every event's
   commands match its row, and each event through a scripted stay. **Atmospheres** (owner: "red mist ... like blood, also just darkness, fog ... not abrupt like other mods,
   make it good and spooky"): the world's own fog driven on the target's client (distance and
   colour through the fog renderer, not a layer over the HUD), easing in over tens of seconds,
   breathing, easing out: a mist that swallows the terrain, a red mist, a darkness that pulls
   the world in; `fog_close` and `red_rain` rebuilt on it. Ask the owner which, how long, which
   stages, and whether they stack.
   **Paused, mid-item (owner: "pause, I'll come back and say continue"):**
   `hostiles_stare` is written and uncommitted: the mind, the SDK's `hold` (ABI 8) and the
   host's `Holds`, `Facing` and the `hold` executor. Rust passes (fmt, clippy, 129 unit, 10
   stays, 28 SDK); the host has not been compiled with it (the client was running). Next: quit
   the client, `cargo build --release -p lodger-core`, `./gradlew build` in `host/` and `mod/`
   (expect `FacingTest`), fix what the compile says, ask to commit, try it at night in the dev
   client. Then the atmospheres, owner's answers: all four (mist from stage 2, red
   mist and darkness stage 3+, dusk at noon stage 2+), the slow breath (in over 40 to 60 s, held
   2 to 4 minutes with the fog edge pulsing, out over 30 to 40 s, starting while they face away
   or are indoors), and they may stack; ask whether `fog_close` and `red_rain` move onto the
   real fog. Then `village_abandoned`, then the rest of the catalogue.
   **After the event review** (owner): a bug check pass over every module and an
   optimization run over every per-tick path.
2. **A dev stay with the real model:** the 1.5B is in the dev client's config folder and the
   default now; read `decisions.log` for `brain=model`, the reactions and the written lines.
3. **Calibration round two** by [round one](calibration/round-1.md)'s list: the first
   planning question, the second question's confidence, the moods' bar, the written lines.
4. **Phase 5, the rest:** the figure's poses (lying in the bed), its swing and crouch, stepping
   behind cover instead of vanishing; the rows that need it (the ending's beats).
5. **The config page** of the debug screen: every key, written back to the file.
6. **The remaining catalogue:** pens, caves and tunnels in the surroundings pass and the rows
   over them; building and digging as a player; the renderers still missing (the painting
   upside down, the compass, footprints, the sky); the endings; chains and the temper.
7. **Game tests** for the executors; **a version bump rehearsal** by [Upgrading](upgrading.md).

## Open questions for the owner

- The crate's final name (`craftkit` is a working name; it was free on crates.io).
- The lodger's commands: decided, all under `/craftkit lodger ...` (`status`, `leave`, `reset`
  beside the debug ones, `show`, `force`, `stage`, `action`).
- The tone of the lines in the target's own name (`brain/text.rs`, `ChatLine`): lower case, no
  punctuation, as a player types, is the current choice.
- Whether the figure (phase 5) wears the target's skin by default on a server, where other
  players would see a second copy of someone, or only in singleplayer.
- The moon phase and the time of day: 26.3 moved time to world clocks; the senses send game
  time's remainder and moon phase 0 until the clock API is read properly (a todo line).
