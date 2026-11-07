# Modrinth texts

Kept with the project so the page and the mod agree. The summary goes in Modrinth's
**Summary** field (one line, under 256 characters); the rest is the **Description**.

## Summary

Something moves into your world and learns how you play. Not quite horror. Play Minecraft as
you always do, and come find out.

Shorter, if it has to be:

Something moves into your world and learns how you play. Not quite horror. Come find out.

## Description

**Play Minecraft as you always do.** Build, mine, farm, sleep. Something has moved in. You will
not see it for a long time. It notices where you sleep, which chest you open first, the way you
walk home, and it uses what it noticed.

It does not chase you. It does not scream. For most of its stay, nothing happens that you could
swear to. And then, slowly, more does.

This is a not-quite-horror mod: unsettling rather than jump-scary, and different every time,
because what it does comes from what *you* did.

### How to play it

- **Use your existing world.** It works best in a world you already care about. It never
  breaks what you built and never takes anything for good.
- **Starting fresh?** Pick a calmer preset on the title screen's settings (Lodger, then
  `calm` or `balanced`).
- **Don't read the spoilers below** if you want to find out for yourself.

### Presets

| Preset | What it is like |
| --- | --- |
| `calm` | Rare and quiet. Nothing touches your screen, your chat or your sleep, and the stay ends gently. |
| `balanced` | The default: a stay of a few in-game weeks that builds slowly. |
| `intense` | Denser, bolder, sooner. |
| `relentless` | Something every evening from the first day. For people who asked. |

You can also switch whole groups off in the settings (sounds, screen effects, the figure, chat
lines), or turn off single events by name in `config/lodger.json`.

### Requirements

- Minecraft Java **26.3**, Java 25.
- **Fabric** (with Fabric API) or **NeoForge**. One jar for your loader; nothing else to
  install.
- Needed on **both the client and the server**.
- **It adds a dimension of its own** to your world (an empty, dark one it uses twice in a stay
  at most). If you remove the mod later, Minecraft warns about that missing dimension when you
  open the world; the world is fine.
- Windows or Linux on x86_64 or arm64, or macOS on Apple silicon. On Linux, Vulkan's loader
  (`libvulkan1`), which any desktop with a GPU driver has; a headless server may need
  `apt install libvulkan1`. No GPU is needed.
- **A one-time model download** on first start (see below), with a progress bar on the title
  screen and a note on each size. **Play without it** skips it: simpler built-in rules decide
  then, and a player who only joins servers needs nothing more. Turn it back on in the
  settings (Use the model).

### The model, and what it needs

What the lodger does next is decided by a small language model that runs **on your own
machine** (the server's, in multiplayer). It is downloaded once from Hugging Face, from the
model's publisher, after you accept its license on the title screen. It never connects
anywhere else.

Pick the size in the settings (`model.size`). By default the whole model runs on the GPU
(Vulkan on Windows and Linux, Metal on a Mac). If the game stutters, lower `model.gpu_share`
(0 to 1) to put part of it on the CPU, or set it to `0` to keep the GPU free for the game. With
no GPU it all runs on the CPU.

| Size | Model | Download | Memory in all | VRAM at the default (all on the GPU) | License |
| --- | --- | --- | --- | --- | --- |
| `0.5b` | Qwen2.5 0.5B Instruct | 491 MB | about 0.7 GB | about 0.6 GB | Apache 2.0 |
| `1.5b` (default) | Qwen2.5 1.5B Instruct | 1.1 GB | about 1.6 GB | about 1.3 GB | Apache 2.0 |
| `3b` | Qwen2.5 3B Instruct | 2.1 GB | about 2.6 GB | about 2.4 GB | Qwen Research License (non-commercial) |
| `4b` | Qwen3 4B Instruct 2507 | 2.5 GB | about 3.2 GB | about 2.9 GB | Apache 2.0 |

"Memory in all" is RAM and VRAM together. At the default nearly all of it is VRAM; with
`model.gpu_share` at `0`, or no GPU, all of it is RAM. The figures are estimates and vary with your system. The model
uses two CPU threads while it thinks and runs on its own thread, so the game never waits for
it. It loads when someone is playing and unloads when no one is.

On a server, the model runs on the server; a client that only plays there can press **Play
without it** and download nothing.

### Generative AI notice

- **A language model drives the lodger's decisions while you play.** It reads a short,
  plain-English account of what the lodger knows about you and picks among choices the mod
  defines. It cannot invent actions. Every rule the mod sets (what it may never do, when, how
  often) is checked after every choice. It also writes a few words now and then (on a sign, or
  a chat line in your name). These are kept short, letters only, and filtered, with a
  hand-written line used instead if a check fails.
- **The lodger reads your server's chat** for lines about it, and keeps at most the last
  eight. Everything it learns stays in your world's save, on your machine.
- **This mod was made with the help of generative AI** (Anthropic's Claude), working on
  the code, the tests and the documentation alongside its author.

### Operators

`/craftkit status`, `/craftkit lodger show`, and a debug screen (F8, with an overlay on F9,
operators only) that shows what it knows, what it plans and why. Every decision is logged in `config/lodger/decisions.log`.

<details>
<summary>What it can do (spoilers)</summary>

- Your things move. A chest is not quite how you left it. Something of yours is gone, and
  turns up somewhere that means something.
- Torches you did not place. A path you did not walk. A sign with one word.
- Sounds where there should be none: a door, steps behind yours, breathing when you crouch
  still.
- A figure, far off, wearing your skin, walking the path you walked yesterday.
- It builds. Slowly. Not far from you.
- In the end, it leaves, it takes, or it stays, depending on how you treated it.

</details>
