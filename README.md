# lodger

A horror mod for Minecraft with one entity in it, the lodger, which moves into your world and
learns you. It does not chase, it does not scream, and for most of its stay you never see it. It
notices where you sleep, which chest you open first, which way you walk home, and it uses what
it noticed: a thing of yours is gone and turns up somewhere that means something, a torch you did
not place lights the way to somewhere, and a figure far off, wearing your skin, walks the path you
walked yesterday.

**Status:** playable, version 0.1.3, for Minecraft Java 26.3 on Fabric and NeoForge. What is left
is in [Todo](docs/todo.md).

## What is in it

- **A stay in five stages,** from things you could not swear to, through things moved and sounds
  where there should be none, to the figure, and an ending that depends on how you treated it: it
  leaves, it takes, or it stays.
- **About six hundred events**, each with the stages it fits, what it needs (your bed, your chest,
  night, rain, you underground), how often it may come and what it costs of the day's budget.
  The catalogue is [The lodger](docs/entity.md).
- **A hundred and eighty-nine chains**: one thing leading to the next over days. Chains that start
  from the same event are **forks**, so the same event goes down different ways in different
  stays; thirty-seven are **scenes**, whose beats come one straight after another. At most three
  run at once, so the story stays legible.
- **The Nether and the End** have events of their own, done there and counted at home.
- **A temper**: how you treat it (chasing it, breaking what it built, ignoring it) changes how it
  treats you.
- **Rules it never breaks**: it never destroys what you built or takes anything for good, never
  makes anything of value out of nothing (what would be loot appears on your screen alone), never
  harms another player, and what it does to your house it does while you look away.
- **Multiplayer**: it chooses one player to live with; the others may witness, never be the target.

## How it decides

What the lodger does next is decided by a small language model running on the server's own
machine, on its GPU by default. The model never invents an action. It reads a short plain-English
account of what the lodger knows about you and picks from a menu of events the mod wrote and
bounded first, and every rule (what it may never do, when, how often, the day's budget) is
checked after every choice. It writes a few words now and then (a sign, a book, a line of chat),
kept short and filtered. Without the model, a rule-based brain decides: less interesting, not
broken. See [Brain](docs/brain.md) and [Model](docs/model.md).

The model is chosen in the settings (`model.size`): Qwen2.5 1.5B by default (1.1 GB); Qwen2.5
0.5B, 3B and 7B; Qwen3 4B, which reads a situation best of all of them, and 8B; Microsoft's
Phi-4 mini; and IBM's Granite 4.0 Micro and H Tiny, from 491 MB to 5 GB.

## What it takes to run

- Minecraft Java 26.3 on Java 25, with Fabric Loader 0.20.1 and the Fabric API, or NeoForge
  26.3.0.101, on both the client and the server.
- Linux, Windows or macOS, each on x86_64 (with AVX2: Intel since 2013, AMD since 2015) or arm64.
  The jar carries a native for each, running the model on the GPU (Vulkan, Metal) or the CPU. A
  Linux server without a GPU needs Vulkan's loader all the same (`apt install libvulkan1`).
- One jar for your loader, `lodger-fabric-26.3-<version>.jar` or
  `lodger-neoforge-26.3-<version>.jar`, with the craftkit host nested inside it. Nothing else.
- **The first start downloads the model.** The title screen shows its license, then a progress
  bar, once. **Play without it** skips it, and a player who only joins a server needs nothing. A
  dedicated server downloads at its first start, with progress in its log.
- About 1.6 GB of memory for the default model, nearly all of it VRAM with the model on the GPU
  (`model.gpu_share` 1, the default) and RAM without a GPU; two CPU threads while it thinks, on
  its own thread so the game never waits for it.

## Commands

All of them are for operators, since the point of the mod is not knowing.

| Command | What it does |
| --- | --- |
| `/craftkit status` | The host's: every Rust mod loaded, its version and native, or why it did not load. |
| `/craftkit lodger debug` | Opens the debug screen (also F8 in a world): the host's senses of you as meters, the lodger's gauges, memory and plan, the open chains, every event with whether it could happen now and a Fire button, and what landed. `/craftkit lodger show`, `force <event>`, `stage <n>` and `action <id>` do the same from chat or a console. |
| `/craftkit lodger status` | Which stage the stay is in and how far, whom it lives with, how many things it has done that they would notice and what it holds, and whether the model or the rules decide. |
| `/craftkit lodger leave` | Ends the stay gently: its leaves ending (everything it holds goes back, its last three days) begins at the next dawn. |
| `/craftkit lodger reset` | Forgets everything it knows of this world and starts over; what it holds goes back first. Typed twice within 30 seconds. |

Every decision is written to `config/lodger/decisions.log` (the situation, each question, the
choice and its confidence).

## The repository

The lodger is a Rust mod. Its mind holds everything it knows and decides everything it does, and
runs and is tested without Minecraft. It is written on **craftkit**, a Rust SDK for Minecraft mods
that this repository also holds: a crate mods are written against, and a host, a Java mod for
Fabric and NeoForge, that loads them and carries out what they decide. The Java is there because
the loaders load Java; everything that can be Rust is Rust.

| Folder | What is in it |
| --- | --- |
| `core/` | The lodger's mind: memory, the stages, the event catalogue, chains, the brain, the model's runner (llama.cpp). |
| `sdk/` | craftkit: the protocol (`Command`, `Observations`), the `Mod` trait and the C ABI. |
| `host/` | craftkit's host for Fabric and NeoForge: the senses, every command's executor, the client's effects. |
| `mod/` | The lodger's content jar: the natives, the assets, the manifests. |
| `scripts/` | Building the natives and the jars, the third-party notices, the config docs, the craftkit export. |
| `docs/` | How every part works and why. |

craftkit also lives in its own repository, [germ3n/craftkit](https://github.com/germ3n/craftkit),
which `scripts/sync_craftkit.sh` brings up to date from here. See [The SDK](docs/sdk.md) and
[Architecture](docs/architecture.md).

### Building and checking

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace` for the mind and the SDK.
- `./gradlew build` in `host/` for the host and its JUnit tests (Java 25).
- `python3 scripts/build_jars.py 26.3 --natives local` for both jars with this computer's native,
  in `dist/26.3/`; `--natives cross` builds every platform's on a Mac.
- CI (`.github/workflows/ci.yml`) is run by hand: the checks on three systems when ticked, each
  native on a runner of its own, both jars, and a GitHub release.

The toolchain, the conventions and how to add an event are in [Development](docs/development.md)
and [Building](docs/build.md); start with [Good to know](docs/good-to-know.md).

## License

MIT for the mod and craftkit. The models are downloaded by the player, not distributed with the
mod: the Qwen models are Apache-2.0 but the Qwen2.5 3B, under Qwen's research license
(non-commercial); Phi-4 mini is MIT; the Granites are Apache-2.0. llama.cpp, which runs them, is
MIT, and its notice ships with the natives in the jar (see
[Development](docs/development.md#third-party-notices) and `THIRD_PARTY_NOTICES.txt`).
