# Development

Building the three parts, running them, the tests, and the conventions the code follows. The
layout:

| Where | What | Built with |
| --- | --- | --- |
| `sdk/` | craftkit, the Rust SDK: the `Mod` trait, the C ABI, the protocol, the harness. See [The SDK](sdk.md). | cargo, in the root workspace |
| `core/` | The lodger's mind, a craftkit mod, built as a shared library. | cargo, in the root workspace |
| `host/` | The craftkit host, a Java mod for Fabric and NeoForge from one `common` tree (`host/common`, `host/fabric`, `host/neoforge`, the MultiLoader layout). | Gradle, Loom for Fabric, ModDevGradle for NeoForge and for `common` |
| `mod/` | The lodger's two jars, one per loader: natives for six platforms, assets, manifests, the host nested, no Java of its own. | Gradle, which runs cargo and takes the host jars from `host/`'s build |

## Toolchain

| What | Pinned to | Where |
| --- | --- | --- |
| Rust | 1.98.1, with clippy and rustfmt, the same as `~/chainv4` and `~/engine` | `rust-toolchain.toml` at the root; the CI's `toolchain:` |
| Java | 25 | `host/build-logic`'s `toolchain.languageVersion`, which the foojay resolver in `host/settings.gradle` finds or downloads; the CI's `setup-java` |
| Minecraft | 26.3 | `host/gradle.properties` and `mod/gradle.properties`, and the manifests' `~26.3` and `[26.3, 26.4)` |
| Fabric Loader | 0.20.1 | the same |
| Fabric API | 0.165.2+26.3 | the same |
| Fabric Loom | 1.19.4 | the same |
| NeoForge | 26.3.0.101 | the same |
| ModDevGradle | 2.0.148, which builds `host/neoforge` and compiles `host/common` against vanilla | the same |
| NeoForm | 26.3-1, the vanilla jar `host/common` compiles against | the same |
| Gradle | 9.8.2 | `host/gradle/wrapper/gradle-wrapper.properties`, and `mod/`'s copy |
| Mappings | Mojang's official ones, on both loaders. Yarn ended at 1.21.11, so there is no `mappings` line | nothing to pin |
| llama.cpp | the commit `llama-cpp-2`'s pinned `llama-cpp-sys-2` builds, so the crate's version pins it | `core/Cargo.toml`, from phase 1 |
| cmake and a C++ compiler | whatever the machine has; the CI's runners' own | for `llama-cpp-sys-2`, from phase 1 |

Confirmed against meta.fabricmc.net (game, loader), Mojang's version manifest (26.3
and 26.2 both run Java 25), Modrinth (the Fabric API build for 26.3), maven.neoforged.net
(NeoForge, ModDevGradle) and the 26.3 branches of Fabric's example mod and the MultiLoader
template. `VersionsTest` in `host/common` checks that this table, the README, both
`gradle.properties`, the manifests, the `Cargo.toml`s, `rust-toolchain.toml`, the ABI version
in `sdk/src/lib.rs` and `Native.java`, and the CI agree. Minecraft 26.x needs Java 25, which is what makes the FFM
API available without flags; a Minecraft that still ran Java 21 would need JNI.

## Updating Minecraft

A new Minecraft, Fabric, NeoForge or Java version changes the host alone, and inside the host
only its Minecraft-facing edge. The whole process, the table of where that edge is, what a
version is most likely to break and the history of bumps are in [Upgrading](upgrading.md).

## Building the native

```sh
cargo build --release -p lodger-core
```

`./gradlew jar` in `mod/` runs this itself when `cargo` is on `PATH` (`cargoBuild` in
`mod/build.gradle`; set `CARGO` to another binary, or to a path that does not exist to skip it,
as the CI does so its jar holds only the three natives it built).

That builds `liblodger_core.dylib`, `.so` or `.dll` in `target/release/`, with llama.cpp
compiled in from phase 1 on. The first build then compiles llama.cpp, which takes a few minutes;
later ones do not. `--features metal` on macOS and `--features vulkan` elsewhere build the
GPU-capable native that `model.gpu_layers` needs; the default native is CPU only.

`mod/build.gradle` puts this machine's native into each jar as `natives/<os>-<arch>/`, and
anything in `mod/natives-extra/<os>-<arch>/` beside it, which is how the CI puts all six
together: `linux-x86_64`, `linux-aarch64`, `macos-x86_64`, `macos-aarch64`, `windows-x86_64`,
`windows-aarch64`, each built on a runner of that platform (llama.cpp is not cross-compiled).
At runtime the host's
`Natives.java` extracts the one for the machine to `config/craftkit/natives/lodger/<os>-<arch>/`
and `Native.java` loads it from there, because a shared library cannot be loaded from inside a
jar. A native already there with the same SHA-256 is not rewritten; a new one is written beside
and moved into place.

Cross-compiling llama.cpp is not attempted: each native is built on its own platform's runner.

## Building the host and the content jar

```sh
cd host && ./gradlew build        # craftkit-fabric-26.3-*.jar and craftkit-neoforge-26.3-*.jar, and the tests
cd mod && ./gradlew jar           # lodger-fabric-26.3-*.jar and lodger-neoforge-26.3-*.jar, with cargo run first
```

The host's jars land in `host/fabric/build/libs/` and `host/neoforge/build/libs/`, and the
lodger's build nests them into its own, in `mod/build/libs/` (a missing host jar is an error
naming the step). A developer runs the two builds in that order; the CI passes the host jars as
artifacts into `mod/host-jars/`. `./gradlew :fabric:runClient` and `:neoforge:runClient` in `host/` start a
development client with the host; to see the lodger in it, put `mod/build/libs/lodger-*.jar` in
that run's `mods/` folder (`host/fabric/runs/client/mods/`), since the host finds content jars
through the loader, not a path. Without a lodger jar, the host starts, loads nothing, and
`/craftkit status` says so.

The icon (a door left ajar at night, two eyes in the gap) is drawn by `mod/art/icon.py`, pixel
art on a 32 by 32 grid written as PNG by hand: `python3 mod/art/icon.py` writes the Modrinth
upload, `mod/art/icon-512.png`, and the jars' `assets/lodger/icon.png` (128), which
`fabric.mod.json` (`icon`) and `neoforge.mods.toml` (`logoFile`) name.

## Building the jars for a release

`python3 scripts/build_jars.py <minecraft>` builds both lodger jars, Fabric and NeoForge, for one
Minecraft version, with the mind's native library for all six platforms inside (windows, linux
and macos, each on x86_64 and aarch64; owner), and puts them in `dist/<minecraft>/`.
At startup the host loads the one for the computer it runs on; a jar without it says so on a
screen at boot (`FailedScreen`), naming the platforms it carries, and the server's log says it
as an error.

- **The natives.** By default they come from the CI's run of the commit checked out
  (`.github/workflows/ci.yml` builds each platform on its own runner; push first, `gh` signed
  in). `--natives local` builds what this computer can (a Mac both macOS ones, cross-compiling
  the other chip); `--natives DIR` takes `DIR/<os>-<arch>/<library>`. A jar missing any of the
  six is an error unless `--allow-missing`. Windows and Linux natives are not cross-compiled from
  a Mac: llama.cpp needs each platform's own C++ toolchain, which the CI's runners have.
- **The version.** `versions/<minecraft>.properties` holds every pin that changes with it
  (Minecraft, NeoForm, Fabric API and loader, Loom, NeoForge, ModDevGradle, Java), laid over the
  gradle.properties files for that build only. A version with no file is refused: the host is
  written against one version's code, and another needs its file and usually a port (see
  [Upgrading](upgrading.md)). Today: 26.3.

## Running a stay quickly

A stay takes weeks of play by design. For development:

- `stages.start_after_days: 0`, `stages.least_days: [0, 0, 0, 0, 0]`, `most_days: [1, 1, 1, 1,
  1]`, `events_per_day: [40, 40, 40, 40, 0]`, `brain.plan_every: 15` and
  `events.cooldown_scale: 0.05` in the dev config, so stages move as soon as their conditions
  allow and the lodger acts every few minutes; no `model.size`, so the dev client runs the
  default 1.5B like a player would (`"0.5b"` for the small file on a slow machine);
- **the debug panel**, F8 in a world (`fn+F8` on a Mac keyboard, where F8 is a media key;
  rebindable under Controls, Miscellaneous), or
  `/craftkit lodger debug`: a strip down the right third of the screen at three quarters of the
  GUI's text size, see-through, that neither pauses the game nor blurs it, so the world stays
  in view beside it (owner: the full screen hid the events it was there to watch);
  it opens on the Mind tab. Hovering an event shows what it does and needs (the catalogue's
  own row, built into the mind from `docs/entity.md`), its cooldown and cap, and how it went
  last; any line or plan with a spot in it, `(x,y,z)` or `(x,ground,z)`, is a link that
  teleports you there. The Senses tab is the host's reading of you (pace, stops, turns,
  look-backs, backing away, a fear meter, light, health, the clock), the Mind tab the lodger's
  gauges (stage, days in stage, the day's budget, the plans held of how many, decisions, what
  the memory holds, kills), its situation lines, its plans as rows with Drop and Now buttons
  and why each waits, what it did lately, what it never forgets, the last decision log
  entries, and buttons (move in, next stage, decide now, drop the plans, forget cooldowns,
  forget everything); the Events tab every event with whether it is offered right now
  and the first reason it is not, a filter, and a Fire button that forces it on the next tick
  past every rule but the ones that protect what the player built (no digging within 8 blocks
  of it); the Log tab the last events the senses took in and the last commands carried
  out with their reports. Operators and the owner of a singleplayer world (cheats on or off).
  The same from chat: `/craftkit lodger show`, `force <event>`, `stage <n>`, `action <id>`,
  which a dedicated server's console can use;
- `-Dcraftkit.debug=true` (on in the dev client's run config) logs every event the senses take
  in and every command carried out;
- `/time add 24000` to move days;
- the mind's scripted-player tests, which run a whole stay in under a second (below).

The dev client: `./gradlew :fabric:runClient` in `host/`, with `mod/build/libs/lodger-fabric-*.jar`
copied to `host/fabric/runs/client/mods/` and its `META-INF/jars/` and the `jars` entry of its
`fabric.mod.json` removed (the dev run has the host from its classes; a second host inside the
jar would load twice), the model under `runs/client/config/lodger/models/`, and the dev config
above as `runs/client/config/lodger.json`. From the repository's root, after `./gradlew build`
in `mod/`, with the client stopped:

```sh
python3 - <<'EOF'
import zipfile, json
src = 'mod/build/libs/lodger-fabric-26.3-0.1.0.jar'
dst = 'host/fabric/runs/client/mods/lodger-dev.jar'
zi, zo = zipfile.ZipFile(src), zipfile.ZipFile(dst, 'w', zipfile.ZIP_DEFLATED)
for i in zi.infolist():
    if i.filename.startswith('META-INF/jars/'):
        continue
    data = zi.read(i.filename)
    if i.filename == 'fabric.mod.json':
        manifest = json.loads(data)
        manifest.pop('jars', None)
        data = json.dumps(manifest, indent=2).encode()
    zo.writestr(i, data)
zo.close()
EOF
```

`decisions.log` keeps growing across runs; move it aside before a session you want to read
from the start.

## Tests

**The SDK and the mind** are where most tests are, and they need no game:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo test -p lodger-core -- --ignored model::     # needs the real model at core/models/qwen-1.5b.gguf (or qwen.gguf
```

- **The SDK's tests:** the ABI helpers (`abi.rs`), the harness refusing another ABI version, and
  `tests/exported.rs`, a two-line mod exported with `export_mod!` and driven through the harness:
  info, init, a refused init, a panicking init, shutdown.

- **Unit tests** of the pure parts: softmax and scoring math, `Memory::describe`'s lines, the
  memory's bounds, the rules, the arc's stage conditions, serialization across format versions.
- **Scripted stays** (`core/tests/stays.rs`): a `Player` script (a home, a daily route, a chest
  opened each morning, reactions to events) drives a `Stay` tick by tick for fourteen in-game days
  against an `Echo` or a table-driven fake scorer, and the test checks what the lodger did: that
  stage one never offered `take`, that nothing happened within reach in view, that the hard
  ending dealt exactly one death, that a `superseded` reply did not event. These are the tests that
  prove the rules.
- **Model tests**, ignored by default, load the real model and check that known situations score
  the expected way: `The player walked toward the figure twice` makes `to be looked at` win over
  the other wants, and an empty-feeling situation leaves `nothing yet` with the most probability.
  They are the regression tests for prompt changes.
- **Harness tests** in the mind (`core/src/lib.rs`) call `info` and `init` through the exported
  functions, as the host would, with good and bad config, and check that nothing panics across
  the boundary.

- **What a tick costs** (`the_tick_costs` in `core/tests/stays.rs`, ignored, run in release:
  `cargo test --release -p lodger-core --test stays -- --ignored the_tick_costs --nocapture`):
  twelve scripted days through the harness, every game tick observed and ticked as the host
  does it, JSON in and out, timed. On an M-series Mac: 288,000 ticks, a mean of
  2 µs, p99 5 µs, p99.9 11 µs, the worst 296 µs (the first tick of a day: the day's surveys and
  reads); the save 44 KB. A game tick is 50 ms, so the mind's own part is not where the time
  goes; the model's questions run on their own thread. It fails when p99 passes 2 ms.

**The host** has fewer, because it is thin:

```sh
cd host
./gradlew test               # JUnit in common: the manifest, the native's names and extraction, the config file, the versions agreeing
./gradlew :fabric:runGametest  # game tests (phase 3): each command's executor against a built test world
```

Each command's executor gets one game test: the world is set up (a house, a chest with an iron
pickaxe, a torch), the command is run, and the world is checked, including that `fits` refuses
when the target is standing in view. These are the tests that prove the body does what the mind
said and nothing more.

As in `~/chainv4`: each module keeps its tests next to it, as `tests.rs` or an inline
`mod tests`, and tests read as sentences: `a_queued_plan_is_superseded_by_a_newer_one`,
`nothing_happens_within_24_blocks_while_in_view`. Every change adds tests, updates the ones it
changes the truth of, and removes those with nothing left to prove. A test is never weakened to
make it pass. A test that needs the real model is `#[ignore]` with a sentence saying why, not
skipped in silence.

## Conventions

- **The mind knows nothing of Minecraft.** No block ids, no entity classes, no NBT: positions are
  `(i32, i32, i32)`, items are strings, players are indices. If the mind needs a new fact, the
  body observes it and sends it. This is what keeps the tests fast and the body thin.
- **The body decides nothing.** An executor checks `fits` and runs. If a choice is wrong, fix the
  mind. A body that quietly does something smarter than it was told is a bug.
- **Every event is data first.** A new event is a row in `events.rs` (id, stages, needs,
  cooldown, cap, command), an executor in the body, a game test for the executor, a scripted-stay
  test if it has a rule of its own, a row in the [catalogue](entity.md#events) and a line in
  [Config](config.md) if it adds a key. All of it in one change; a row without its test and its
  docs is not done.
- **Every rule is in `rules.rs` and in [What it never does](good-to-know.md#what-it-never-does),**
  with a scripted-stay test that would catch its removal. Nothing in config reaches a rule.
- **The host is nobody's mod.** Nothing in `host/` names the lodger. A capability the lodger
  needs is a command of the protocol any mod could send, with an executor written for the
  general case.
- **Text the player sees is written once, in `mod/src/main/resources/assets/lodger/lang/`.** The
  signs' words, the status lines, the native's error. Nothing the model chooses is shown as text;
  the model chooses events.
- **The situation text and the choices are in `brain/text.rs`,** nowhere else, so a prompt change
  is one diff and the model tests cover it.
- **No `static mut`, one `OnceLock`:** the model worker, which is one per process on purpose. All
  else is owned through `Stay`.
- **No panics across the FFI,** and no `unwrap` in the mind outside tests. Errors are strings in
  JSON the body logs.
- **Comments say why.** Full sentences. A cap has its reason beside it.
- **Docs and tests land with the change, every change,** as `~/chainv4` does it: the page for
  the part, [Good to know](good-to-know.md) for what would surprise someone, a `[x]` in
  [Todo](todo.md) saying what was done and when with the old text kept after `Was:`, and the tests
  above. A change is reported with exactly which checks ran and how they came out.
- **Commit messages:** a lowercase first line saying what it lets the lodger or a player do, then
  `- area: ...` bullets, then `- docs:` and `- tests:`.

## Third-party notices

`python3 gen_notices.py` writes `THIRD_PARTY_NOTICES.txt` at the root from `Cargo.lock`, the
crates' own license files and llama.cpp's (adapted from `~/chainv4`'s). Run it after
any dependency changes; it stops with a message when a crate no longer carries what it expects. Both
jars carry it at their root, beside `LICENSE_lodger`.

The jar carries llama.cpp (MIT) and ggml (MIT) compiled into the native, plus every Rust crate
in `core/Cargo.lock` and the Java dependencies Loom bundles. `scripts/gen_notices.py`, adapted
from `~/engine`'s, writes `THIRD_PARTY_NOTICES.txt` from the lockfile and Gradle's dependency
report, with the licence text of each, and stops naming what to update when a crate it expects
is gone. Run it after changing dependencies; it ships inside the jar at `META-INF/`. The model
is not a dependency: the player downloads it, and its Apache-2.0 notice is Qwen's to give.
