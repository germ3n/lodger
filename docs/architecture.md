# Architecture

lodger is a Rust mod on craftkit (see [The SDK](sdk.md)). The **mind** is a Rust crate,
`core/`, built as a shared library and shipped in a content jar (`mod/`) with the lodger's
assets, which keeps everything the lodger knows and decides everything it does, and has no idea
what Minecraft is. The **body** is the craftkit host, `host/`, a Java mod for Fabric and
NeoForge, which watches the world, tells the mind what it saw, and carries out what the mind
decided. They talk through the SDK's C ABI, with JSON in both directions, called from Java
through the FFM API (`java.lang.foreign`, final since Java 22; Minecraft 26.x runs Java 25).
There is no JNI, no second process, and no Java of the lodger's own.

Why the split this way round: the model layer already exists in Rust (`~/engine`'s `ai/`), the
mind is the part that needs tests by the hundred and is pure logic over data the body hands it,
and a mind that knows nothing of the world can be run in `cargo test` against a fake scorer and
a scripted player, in milliseconds, with no game open. The body is as thin as the game allows,
and generic: nothing in it is the lodger's, so a second Rust mod runs on the same host.

## The parts

### The mind, `core/`

| Part | Where | What it does |
| --- | --- | --- |
| Stay | `core/src/stay.rs` | One lodger in one world: the stage, the target, the memory and the open plan. `Stay::observe` takes the body's observations, `Stay::tick` advances time and asks, `Stay::commands` hands out what to do. Serializes itself to bytes the body stores. |
| Memory | `core/src/memory.rs` | What the lodger knows about its target, bounded, and `describe`, which turns it into the lines of the situation text. See [Memory](memory.md). |
| Brain | `core/src/brain/` | Builds the situation text, picks the questions for the moment, asks the model or the rule brain, applies the rules to the answer, logs it. See [Brain](brain.md). |
| Model | `core/src/model/` | The worker thread, the queue with its keys, llama.cpp through `llama-cpp-2`, and the scoring. Ported from `~/engine/game/base/src/ai/` with the game taken out. See [Model](model.md). |
| Events | `core/src/events.rs` | Every event as data: its id, which stages offer it, what it needs (a chest the target uses, a route, darkness), and the command it becomes. See [Events](entity.md#events). |
| Rules | `core/src/rules.rs` | The hard rules, applied to every decided event before it becomes a command. See [What it never does](good-to-know.md#what-it-never-does). |
| Arc | `core/src/arc.rs` | When a stage ends and what the next one is, and the endings. See [The five stages](entity.md#the-five-stages). |
| Log | `core/src/brain/log.rs` | The decision log. |
| Mod | `core/src/lib.rs` | `Lodger`, the `craftkit::Mod`: `init`, `info` and `warm`, and the per-world calls; `craftkit::export_mod!` makes the C ABI from it. |

### The content jar, `mod/`

No Java. `mod/build.gradle` runs cargo, puts the mind's library under `natives/<os>-<arch>/`
(and the other platforms' from `natives-extra/`, compressed, which `scripts/build_jars.py`
fills; see [Building](build.md)), and packs it with
`META-INF/craftkit.json`, `fabric.mod.json`, a `META-INF/neoforge.mods.toml` with no language loader,
`pack.mcmeta`, `assets/lodger/` (the lang file and the icon) and `data/lodger/` (its dimension).
One jar per loader, from one tree.

### The body, `host/`

The craftkit host, which [The SDK](sdk.md#what-the-host-is) describes. The parts the lodger
leans on, all generic:

| Part | Where | What it does |
| --- | --- | --- |
| Native | `host/common/.../Native.java` | Binds one mod's library through FFM. The only Java that knows there is a native. |
| LoadedMod, Host | `host/common/.../LoadedMod.java`, `Host.java` | Finds content jars by their manifest, extracts the native to `config/craftkit/natives/<id>/<os>-<arch>/`, checks the ABI, hands the mod its config, answers `/craftkit status`. |
| Worlds | `host/common/.../world/Worlds.java`, `WorldData.java` | Per loaded world: the mind's handle, the per-tick exchange, the mod's bytes in the world's saved data. |
| Senses | `host/common/.../senses/` | Turns server events and distance checks into observations: positions, chests opened, blocks placed, sleep, death, looking toward a figure, dimension changes. |
| Executors | `host/common/.../world/Executors.java`, and the rest of `world/` | One per command of the protocol. Each re-checks the world (`fits`), runs, and reports `done` or `did not fit`. |
| Figure | `host/common/.../world/Figures.java` | A figure in a player's skin that stands or walks a path, sent as packets to the target's client alone (see [What the client knows](#what-the-client-knows)). |
| Net | `host/common/.../net/` | The host's own packets to a player's client: a hint, and the debug screen's asks and answers. Sounds and the figure go as the game's own packets. |
| Download | `host/common/.../Preparation.java`, `Downloader.java` | Before play, fetches a missing model on a thread of its own (the magic checked, written under a temporary name, the SHA-256 checked, a cut-off download resumed), then has the mind load it (`warm`). |
| Platform | `host/fabric/`, `host/neoforge/` | Each loader's entry point, events, config directory, loaded jars, and the handshake that refuses clients without the host. |

Each part has one owner, passed down rather than shared:

- `Lodger` (the `Mod`) owns the `Model` worker and the config; each `Stay` (its `World`) owns the
  `Memory`, the `Brain`, the `Arc` and the open plan.
- `Brain` owns its `Situation` builder and its decision log, and borrows the `Model` worker.
- `Model` owns the worker thread, the queue and the loaded llama.cpp context.

A test can build a `Stay` around a scripted player and a fake scorer and run it tick by tick,
through the SDK's harness, the same path the host takes.

## The C ABI

The SDK's, in [The SDK](sdk.md#the-c-abi): `craftkit_init` once per process, and per world
`open`, `observe`, `tick`, `report`, `save` and `close`, all UTF-8 JSON in caller-owned buffers,
so the FFM side is a few `MethodHandle`s and no struct layouts. The JSON is small (a tick's
observations are a few hundred bytes) and the exchange is once a tick, so its cost is nothing
next to a block update. `tick` is also where the mind drains model replies and finished
downloads, so every callback runs on the server thread, in a tick, never inside the call that
asked. Errors come back as JSON, never as a panic across the boundary.

## Threads

- **The server thread** runs the game. Every call into the mind happens here, once per tick,
  from the host's world loop. The mind's own code runs on this thread too, except the two below, so it never
  needs a lock the body can see.
- **The model worker** is one thread per process, `lodger-model`, started when the model loads
  (`warm`, before play, or else the first question) and stopped by `craftkit_shutdown`. It takes `Job`s off a channel, scores them, sends `Reply`s
  back. It is the only thread that touches llama.cpp. One per process, not per world, because the
  model is a gigabyte or more in memory and a client that opens two worlds in a row must not load two.
- **The host's preparation thread** fetches a missing model before play, one download at a time,
  and has the mind load it.
- **The client's render thread** draws the figure and plays sounds. It gets packets and asks for
  nothing.

Replies cross back into the server thread inside `tick`, which
drains them first, so a model's answer is acted on in a tick, never inside the
call that asked. This is the engine's `ai.pump`, moved inside the mind.

## A tick's way through

1. **Senses** gather what the target did since last tick and the host sends it as observations
   (and, once an in-game day, a scan of the surroundings within 96 blocks of the home, which
   the memory keeps as [Surroundings](memory.md#what-is-kept)):
   position and facing, a chest opened (which one, what is in it, by id and count), a block
   placed, sleep begun or refused, a figure looked at (within 10 degrees), a dimension change, a
   death, a login or logout.
2. **The mind drains** replies that arrived: each is matched to its pending question and handed
   to the brain, which turns the choices into an event at a time and place, runs the rules over it,
   and puts it in the plan, or `wait`.
3. **Memory** takes the observations, bounded (see [Memory](memory.md#bounds)).
4. **The arc** checks whether the stage should change: enough in-game days, enough events landed,
   the target's reactions. A change is logged and may pick a new target (see
   [Multiplayer](entity.md#multiplayer)).
5. **The brain** decides whether to ask: on a timer (`brain.plan_every`) and on triggers (the
   target slept, opened a chest, saw a figure, came home). An ask builds the situation text, picks
   this stage's and moment's questions, and submits them with a key. It returns at once.
6. **The plan** is checked: an event whose time has come becomes a command in this tick's return.
7. **The body carries it out.** Each executor's `fits` runs against the world as it is now. If it
   still fits, `run`, and next tick's `report` says `done`; if not, `did not fit` and why, and the
   memory records that too, so the lodger does not try the same thing in the same place twice.

## What the client knows

As little as possible, because the mod is about not knowing:

- the figure: not an entity the server spawns but packets to the target's client alone (a
  player-list entry carrying the target's own profile, so the skin is theirs, and a player
  entity with an id far above the server's; `world/Figures.java`), so no other
  player and no other mod ever sees it, and the server's world never holds it;
- a sound to play at a position, by id, once;
- a hint, with a duration and a place, which the client renders and then stops; the world the
  server holds never changed. The kinds the client renders (`client/Glitches.java`):
  `cut_music` (the music stops and stays stopped), `silence` (ambient, weather and
  hostile sounds stopped as they start), `flicker` (the screen darkens in flickers),
  `lights_fail` (dark, then back), `sun_blink` (black for a blink, twice), `lightning_silent`
  (a white flash), `red_rain` (the drops red), `mist`, `red_mist`, `darkness`, `dusk_noon` and
  `fog_close` (real fog), `stars_out`, `moon_wrong`, `night_early` and `wrong_sky` (the sky),
  `compass_points` (a compass turned) and `painting_flip` (a painting upside down until looked
  at), `hunger_shake`, `heart_skip`, `subtitle`, `name_called` and `ghast_name` (the HUD and the
  subtitles), `crosshair_gone`, `hotbar_slip` (the selected slot jumps to an item once),
  `camera_nudge` (two degrees toward the place, once), `fov_pull` (the field of view narrows
  over the hint's time and snaps back), `snow_indoors` and `rain_indoors` (snow or rain falls
  around the player), `ripples` and `bobber` (on water at a place), `advancement_toast`,
  `eyes_open` (lying in bed, the game's fade to sleep held off: `choke`);
- whether this player is the target, so a witness's client drops sounds and figures aimed at
  someone else. The server never sends them to a witness anyway; the flag is a second guard.

The client never knows the stage, the memory or the plan, so a client-side reading of the mod's
state shows nothing worth knowing. The mind's bytes live on the server only. The one exception
is the debug screen (`client/debug/`), which asks the server every half second and gets an
answer only if the server says the player may: an operator, or the owner of a singleplayer
world. A player on someone else's server sees nothing through it.
