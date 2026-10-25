# lodger docs

How the lodger works, part by part, and what to keep in mind while running it or changing it.
Nothing is built yet (2026-11-08): these pages are the design, written so that building can start
from them and so that every page is kept true as code lands.

- The [README](../README.md) covers installing, the commands and the licenses at a glance.
- These pages explain how each part works and why it is built that way.

| Page | What it covers |
| --- | --- |
| [Good to know](good-to-know.md) | What surprises people, the hard rules the brain cannot break, and what runs where. Read this first. |
| [Architecture](architecture.md) | The parts, the threads, and how a tick and a decision travel through them. |
| [The SDK](sdk.md) | craftkit, the Rust SDK the lodger is the first mod of: what a Rust mod is (a crate and a content jar), the host for Fabric and NeoForge, the C ABI, the `Mod` trait, the protocol, and testing without a game. |
| [The lodger](entity.md) | The design: what it is, why it is not another thing in the fog, the five stages of its stay, its senses, every event it can do, multiplayer, and how its stay ends. |
| [Brain](brain.md) | How decisions are made: the situation text, the questions and their choices, scoring with the model, confidence, cadence, the rule brain that stands in without a model, and the decision log. |
| [Model](model.md) | The model file: Qwen2.5 1.5B (or 0.5B or 3B) as a GGUF, downloading and checking it, loading it through llama.cpp from Java, threads, memory and unloading. |
| [Memory](memory.md) | What the lodger remembers about its target: home, routes, habits, reactions, what it took, and how all of it is kept small and turned into text. |
| [Ideas](ideas.md) | The brainstorm the catalogue draws from: several hundred events by what is around and what the player is doing, what it could build, and what was set aside and why. |
| [Config](config.md) | Every key in `config/lodger.json`, its default and what it changes. |
| [Development](development.md) | Toolchain, building the mod and the natives, running, tests, and the conventions the code follows. |
| [Upgrading](upgrading.md) | Taking the host to a new Minecraft, Fabric, NeoForge or Java version: where the game-facing edge is, the versions and where they are pinned, the step-by-step process, what usually breaks, and the history of bumps. |
| [Todo](todo.md) | What is done, what is next, in order, and known gaps. |
| [Queue](queue.md) | Where the work stands and what the next session picks up, with the rules every session follows. |
