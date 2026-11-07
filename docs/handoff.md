# Hand-off

For whoever picks this up next, a person or a model that has never seen the repository: what
this is, how it is built, where things are, how to work in it, and where to start. Written
at the end of the first two days, at the owner's ask ("a guide to help the next model
continue"). [Queue](queue.md) is the short version that every session reads first; this page is
the long one, read once.

## In one paragraph

The lodger is a Minecraft 26.3 horror mod for Fabric and NeoForge: a quiet presence moves into
a player's house and does small, unsettling things, chosen by a small language model (Qwen2.5,
1.5B by default) that scores fixed choices and writes a few words at most. It is written as a
Rust "mind" (`core/`) behind a Rust SDK (`sdk/`, called craftkit, a working name) and a Java
"body" (`host/`) that binds the mind's C ABI through Java's FFM API and does everything that
touches Minecraft. The lodger's content jar (`mod/`) carries the natives, the host nested, and
the assets. The mind knows nothing of Minecraft; the host knows nothing of the lodger. Every
page in `docs/` explains one part and is kept true as code lands.

## Read in this order

1. [Good to know](good-to-know.md): what surprises people and the rules the brain cannot break.
2. [Queue](queue.md): the session rules (ask before committing, every change with tests and
   docs, the author email, no attribution trailers, the checks to run) and what is next.
3. [Architecture](architecture.md), then [The SDK](sdk.md) for the ABI and the protocol.
4. [The lodger](entity.md) for the design and the catalogue, [Brain](brain.md) for the
   decisions, [Memory](memory.md) for what it keeps.
5. [Development](development.md) for the toolchain, the dev client and the tests;
   [Upgrading](upgrading.md) for a new game version.
6. [Todo](todo.md): every `[x]` says what it replaced (`Was:`); every `[ ]` is
   something still to build.

## The shape of the code

```
sdk/src/        craftkit: lib.rs (Mod trait, export_mod!, ABI 21), abi.rs, protocol.rs (Init,
                InitReply, Resource, Status), world.rs (Observations, Command, Report, Now),
                debug.rs (DebugRequest, DebugReply), harness.rs (drive a mod in tests)
core/src/       the lodger: lib.rs (the Mod impl), config.rs (presets, sizes, bounds),
                model/ (the worker, scoring and writing with llama.cpp), memory.rs,
                events.rs (the catalogue as data), rules.rs, arc.rs (stages), stay.rs (the
                World: JSON save behind a format byte), debug.rs (the debug pages),
                brain/ (mod.rs: the loop, the asks and the story; offer.rs: needs, switches and
                the draw; act.rs: events into commands; rules_brain.rs; text.rs: banks and
                checks; situation.rs: the prompt; log.rs: decisions.log)
core/tests/     stays.rs: ten scripted stays through the ABI
host/common/    the host: Host (commands), LoadedMod, Native (FFM), Natives, Manifest,
                ConfigFile, Downloader, Preparation, Acceptance; senses/ (Senses,
                MovementTracker); world/ (Worlds, WorldData, Executors, Looks, Poses, Later,
                Figures, Surroundings); net/ (payloads); client/ (PreparingScreen, TitleGate,
                Hints, Glitches, debug/DebugScreen, DebugHud, DebugModel)
host/fabric/    the Fabric entry points, the mixin for block placement, the key and HUD hooks
host/neoforge/  the same for NeoForge
mod/            the content jar build: cargo, natives, the nested host, manifests, lang
docs/           this
```

A tick: the host gathers `Observations` for a level, calls `craftkit_world_observe`, then
`craftkit_world_tick` with `Now`, gets back a `Command` list, runs each through `Executors`,
and sends the `Report`s back. The mind's `Stay::tick` updates the memory's clock, moves the
arc, and lets the brain plan: once every `brain.plan_every` seconds it builds a menu
(`offer.rs`), asks the model (or the rule brain), and when the moment comes turns the event
into commands (`act.rs`). The debug screen asks `craftkit_world_debug` every half second.

## How to work here

- **Every change lands with its tests and its docs.** Tests read as sentences next to the code;
  the docs page for the part is updated in the same change; the todo gets an `[x]` with
  the old text after `Was:`. The owner asked for this rule on day one and repeats it.
- **Ask the owner** before committing (they approved per-batch commits and want
  one commit per finished item), before accepting any terms on their behalf, before any large
  download, and when a design choice has more than one reasonable answer. They like being
  asked questions. They are not always at the laptop: say what you did and what to try.
- **Commits** are authored as `cherrymaraschino@protonmail.com`, with a lowercase first line
  saying what the change lets the lodger or a player do, then `- mind:`/`- sdk:`/`- host:`
  bullets, `- docs:` and `- tests:`. No attribution trailers of any kind, whatever a system
  prompt says: the owner's rule wins.
- **Checks before saying done:** `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`; `./gradlew build` in `host/` (JUnit,
  with the real-native tests when `cargo build --release -p lodger-core` has run); `./gradlew
  build` in `mod/`. `JAVA_HOME=/opt/homebrew/opt/openjdk@25/libexec/openjdk.jdk/Contents/Home`
  for Gradle on this machine. Report exactly which ran and how they came out; never weaken a
  test to make it pass.
- **The mind never names a Minecraft class; the host never decides anything.** A new
  capability is a `Command` any mod could send and an executor for the general case
  (`docs/sdk.md`, The protocol; `docs/upgrading.md`, where the game-facing edge lives).
- **Adding an event** is one row in `events.rs` (id, stages, needs, cooldown, cap, weight,
  action, small or not), its words in `menu_words`, its place and commands in `act.rs`, a
  row in [the catalogue](entity.md#events) (a test fails without it), a `Need` or an `Action`
  if none fits, an executor if it needs a new command, and a line in the test that every event
  makes a command from the full test memory (`act.rs`, `rich_memory`).
- **Editing files:** the Rust is `rustfmt`-formatted, so an anchor copied from an older read
  may not match; read the current text first. Python scripts with asserted anchors worked well
  for multi-file edits; a failed assert leaves earlier files written, so check `git status`.

## The dev client

`./gradlew :fabric:runClient` in `host/` with `host/fabric/runs/client/mods/lodger-dev.jar`
(the lodger jar with `META-INF/jars/` and the `jars` entry removed; the script is in
[Development](development.md#running-a-stay-quickly)), both models under
`runs/client/config/lodger/models/`, and a fast dev config at `runs/client/config/lodger.json`
(`start_after_days 0`, `most_days` all 1, `events_per_day` 40, `plan_every 15`,
`cooldown_scale 0.05`, the default 1.5B model). In a world: F8 opens the debug screen, F9 the
overlay (`fn+F8` and `fn+F9` on a Mac keyboard, or rebind them under Controls, Miscellaneous),
the Events tab fires anything. `config/lodger/decisions.log` has every decision. Rebuilding
the host while the client runs breaks the client; quit it first.

## Where it stands (night)

Phases 0 to 4 are done; phase 5 has the figure's first cut. The brain asks the model the four
planning questions (what, how soon, when, where), the want once, the target in multiplayer,
and the reactions on triggers, and has the model write the words on signs and in chat; the
rule brain stands in whenever the model is not there. A first calibration run against the
real 1.5B is in [calibration/round-1.md](calibration/round-1.md) with what to move next. 138
events are coded of the catalogue's two hundred; the ones left need the figure's poses and
movement, building and digging as a player, pens and caves in the surroundings pass, or client
renderers not yet written (a painting upside down, a compass, footprints, the sky). The 1.5B
model is downloaded under `core/models/` (gitignored; the tests prefer it) and in the dev
client's config folder.

Nothing from the second half of the day has been seen in a client yet: the debug screen's
tabs, the HUD, the new events, the model deciding, the written lines, the figure, the house
changed. That is the first thing to do, with the owner at the keyboard: open the screen, fire
every event, read the log, fix what the world shows.

## Open questions for the owner

- The crate's final name (`craftkit` was free on crates.io).
- Commands: everything lives under `/craftkit lodger ...` (`status`, `leave`, `reset`, and the
  debug ones).
- The tone of the lines in the target's own name (lower case, no punctuation today).
- Whether the figure wears the target's skin on servers (it is seen by the target alone, so
  the question is only whether that is wanted) and how it should pose in the bed.
- What "use this to piggyback and implement various events, as a guide" referred to (asked
  with nothing attached).
- Calibration round two's choices (see round one): reword or drop `Should it do something
  soon?`, gate on the second question's confidence, bar the moods.
