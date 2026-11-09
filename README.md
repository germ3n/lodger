# lodger

A horror mod for Minecraft with one entity in it, the lodger, which moves into your world and
learns you. It does not chase, it does not scream, and for most of its stay you never see it. It
notices where you sleep, which chest you open first, which way you walk home, and it uses what
it noticed: a thing of yours is gone and turns up somewhere that means something, a torch you did
not place lights the way to somewhere, and a figure far off, wearing your skin, walks the path you
walked yesterday.

What it does next is decided by a small language model, Qwen2.5 1.5B, running on the server's own
machine, on its GPU by default. The model never writes text and never invents an action. It reads a short plain-English
account of what the lodger knows and picks from a fixed list of choices, the way the AI in
`~/engine` scores NPC decisions. Every choice the model can make is one the mod wrote and bounded
first.

The lodger is a Rust mod. Its mind, a Rust crate, holds everything it knows and decides
everything it does, and runs and is tested without Minecraft. It is the first mod written on
craftkit (a working name), a Rust SDK for Minecraft mods that this repository also holds: a crate
mods are written against and a host, a Java mod for Fabric and NeoForge, that loads them and
carries out what they decide. The Java is there because the loaders load Java; everything that
can be Rust is Rust (see [The SDK](docs/sdk.md) and [Architecture](docs/architecture.md)).

**Status:** playable, before its first release. See [Todo](docs/todo.md) for what is left.

`lodger` is a working name.

## What it will take to run

- Minecraft Java 26.3 on Java 25, with Fabric Loader 0.20.1 and the Fabric API, or NeoForge
  26.3, on both the client and the server, on Linux, Windows or macOS, each on x86_64 (with
  AVX2) or arm64, which the lodger's jar carries natives for, each with its GPU on (Vulkan, Metal). A
  Linux server without a GPU needs Vulkan's loader all the same (`apt install libvulkan1`).
  Building them: [Building](docs/build.md). The versions were confirmed against
  Fabric's, NeoForged's and Mojang's own metadata (see [Development](docs/development.md#toolchain)).
- One jar for your loader, `lodger-fabric-26.3-<version>.jar` or
  `lodger-neoforge-26.3-<version>.jar`, with the craftkit host nested inside it. Nothing else.
- **The first start downloads the model.** The title screen shows the model's license, then
  waits behind a progress bar for the 1.1 GB model (from Hugging Face, Qwen's own file;
  `model.size` picks another, 491 MB to 2.5 GB), then loads it, then lets you in. Once.
  **Play without it** skips the download. A
  dedicated server does the same at its first startup, with progress in its log.
- About 1.6 GB of memory on the server for the model, nearly all of it VRAM with the model on
  the GPU (the default, `model.gpu_share` 1) and RAM without a GPU, two CPU threads while it
  thinks, and 1.1 GB of disk for the model file, which the first start downloads (see
  [Model](docs/model.md)).
- Without the model the mod still runs, on a rule-based brain. It is less interesting, not
  broken.

## Commands

All of them are for operators, since the point of the mod is not knowing.

| Command | What it does |
| --- | --- |
| `/craftkit status` | The host's: every Rust mod loaded, its version and native, or why it did not load. |
| `/craftkit lodger debug` | Opens the debug screen (also F8 in a world): the host's senses of you as meters, the lodger's gauges, memory and plan, every event with whether it could happen now and a Fire button, and what landed. `/craftkit lodger show`, `force <event>`, `stage <n>` and `action <id>` do the same from chat or a console. |
| `/craftkit lodger status` | Which stage the stay is in and how far, whom it lives with, how many things it has done that they would notice and what it holds, and whether the model or the rules decide. From anywhere, the console too. |
| `/craftkit lodger leave` | Ends the stay gently: its leaves ending (everything it holds goes back, its last three days) begins at the next dawn. Before it has moved in, it never will. |
| `/craftkit lodger reset` | Forgets everything it knows of this world and starts the stay over, as a fresh world would; what it holds goes back first. Typed twice within 30 seconds. |

All of them are for operators. Every decision is also written to `config/lodger/decisions.log`
(the situation text, each question, the choice and its confidence), and the debug screen shows
the last ones.

## Docs

How each part works and why it is built that way is in [docs/](docs/README.md). Start with
[Good to know](docs/good-to-know.md), then [The lodger](docs/entity.md) for the design and
[Brain](docs/brain.md) for how the model decides.

## License

MIT for the mod. The model, Qwen2.5-Instruct (1.5B by default), is Apache-2.0, as are the 0.5B
and Qwen3 4B; the 3B is under Qwen's research license (non-commercial). Each is downloaded by
the player, not distributed with the mod. llama.cpp, which runs it, is MIT, and its notice ships
with the natives in the jar (see [Development](docs/development.md#third-party-notices)).
