# Modrinth texts

Kept with the project so the page and the mod agree. The summary goes in Modrinth's
**Summary** field (one line, under 256 characters); the rest is the **Description**.

## Summary

Something moves into your world and learns how you play. Not quite horror. Play Minecraft as
you always do, and come find out.

Shorter, if it has to be:

Something moves into your world and learns how you play. Not quite horror. Come find out.

## Description

### Generative AI notice

- **A language model drives the lodger's decisions while you play.** It reads a short,
  plain-English account of what the lodger knows about you and picks among choices the mod
  defines. It cannot invent actions. Every rule the mod sets (what it may never do, when, how
  often) is checked after every choice. It also writes a few words now and then (on a sign, in a
  book, or a chat line in your name). These are kept short, letters only, and filtered, with a
  hand-written line used instead if a check fails.
- **The lodger reads your server's chat** for lines about it, and keeps at most the last
  eight. Everything it learns stays in your world's save, on your machine.
- **This mod was made with the help of generative AI** (Anthropic's Claude), working on
  the code, the tests and the documentation alongside its author.

**Play Minecraft as you always do.** Build, mine, farm, sleep. Something has moved in. You will
not see it for a long time. It notices where you sleep, which chest you open first, the way you
walk home, and it uses what it noticed.

It does not chase you, not for a long time. It does not scream. For most of its stay, nothing
happens that you could swear to. And then, slowly, more does.

This is a not-quite-horror mod: unsettling rather than jump-scary, and different every time,
because what it does comes from what *you* did.

### How to play it

- **Use your existing world.** It works best in a world you already care about. It never
  breaks what you built and never takes anything for good.
- **Starting fresh?** Pick a calmer preset on the title screen's settings (Lodger, then
  `calm` or `balanced`): the default is `intense`.
- **Don't read the spoilers below** if you want to find out for yourself.

### What it never does

- It never breaks what you built, and never takes anything for good: what it moves comes back,
  somewhere.
- It never makes anything of value out of nothing. What it makes is its own work: the books it
  writes, a potion of its brewing, a little more in your composter.
- It never harms another player. It can nip at you from the middle of a stay, and hit harder
  later. By default what it does can kill you, late in a stay; turn `events.kills` off in the
  settings and it never takes you below half a heart before the very end.
- What it does to your house, it does while you look away.
- It lives in the overworld. Past a portal, you are mostly on your own.

### Presets

| Preset | What it is like |
| --- | --- |
| `calm` | Rare and quiet. Nothing touches your screen, your chat or your sleep, and the stay ends gently. |
| `balanced` | A stay of a few in-game weeks that builds slowly. |
| `intense` | The default: denser, bolder, sooner. |
| `relentless` | It moves in the moment you open the world, and something happens every evening. For people who asked. |
| `extreme` | Something nearly every few minutes from the moment you open the world, a stage a day or two, and the death at the end. Barely a moment's peace. |

You can also switch whole groups off in the settings (sounds, screen effects, the figure, chat
lines). The settings' **All keys** tab has every option, each explained when you hover it, with a
button to put one (or all) back to the preset; single events can be turned off by name there too.

### Requirements

- Minecraft Java **26.3**, Java 25.
- **Fabric** (with Fabric API) or **NeoForge**. One jar for your loader; nothing else to
  install.
- Needed on **both the client and the server**.
- **It adds a dimension of its own** to your world (an empty, dark one it uses twice in a stay
  at most). If you remove the mod later, Minecraft warns about that missing dimension when you
  open the world; the world is fine.
- Windows, Linux or macOS, on x86_64 or arm64 (Intel Macs too). An x86_64 processor needs AVX2
  (Intel since 2013, AMD since 2015). On Linux, Vulkan's loader
  (`libvulkan1`), which any desktop with a GPU driver has; a headless server may need
  `apt install libvulkan1`. No GPU is needed.
- **A one-time model download** on first start (see below), with a progress bar on the title
  screen and a note on each size. **Play without it** skips it: simpler built-in rules decide
  then, and a player who only joins servers needs nothing more. Turn it back on in the
  settings (Use the model).

### Recommended addons

The set the lodger is played and tested with on Fabric 26.3. None is needed; each works beside
it. Grouped by what it is for:

- **Sound:** AmbientSounds, Presence Footsteps, Sound Physics Remastered (its echoes carry the
  lodger's sounds too, and sounds that go wrong go wrong with it on).
- **The look of the world:** Subtle Effects, Visuality, Fancy World Animations, LambDynamicLights,
  Leaves Be Gone, ItemPhysic, Inventory Particles, NotEnoughAnimations, FirstPerson.
- **Performance:** Sodium, Sodium Extra, Lithium, FerriteCore, ImmediatelyFast, EntityCulling,
  More Culling, and Iris for shaders.
- **Interface:** AppleSkin, Auto HUD, EffectTimerPlus, Just Enough Items, Mod Menu, Ok Zoomer,
  Liteminer.
- **Libraries** the mods above need: Fabric API, Fabric Language Kotlin, Cloth Config,
  YetAnotherConfigLib, Forge Config API Port, Fzzy Config, MezzConfig, Konfig, Amber, CreativeCore,
  Data Anchor, MossyLib, Placeholder API, Puzzles Lib.

### Recommended shader

**Hysteria**, with Iris. The lodger's mists and fog show through a shader pack too: with one on,
they are drawn over the screen instead of as the game's own fog.

### The model, and what it needs

What the lodger does next is decided by a small language model that runs **on your own
machine** (the server's, in multiplayer). It is downloaded once from Hugging Face, from the
model's publisher, after you accept its license on the title screen. It never connects
anywhere else.

Pick the size in the settings (`model.size`). The 1.5B is the default and light; the **Qwen3
4B reads you best** of all of them, if your GPU has about 3 GB to spare. Microsoft's Phi-4 mini
and IBM's Granites are there too, if you would rather run one of theirs. By default the whole model runs on the GPU
(Vulkan on Windows and Linux, Metal on a Mac). If the game stutters, lower `model.gpu_share`
(0 to 1) to put part of it on the CPU, or set it to `0` to keep the GPU free for the game. With
no GPU it all runs on the CPU.

| Size | Model | Download | Memory in all | VRAM at the default (all on the GPU) | License |
| --- | --- | --- | --- | --- | --- |
| `0.5b` | Qwen2.5 0.5B Instruct | 491 MB | about 0.7 GB | about 0.6 GB | Apache 2.0 |
| `1.5b` (default) | Qwen2.5 1.5B Instruct | 1.1 GB | about 1.6 GB | about 1.3 GB | Apache 2.0 |
| `3b` | Qwen2.5 3B Instruct | 2.1 GB | about 2.6 GB | about 2.4 GB | Qwen Research License (non-commercial) |
| `4b` | Qwen3 4B Instruct 2507 | 2.5 GB | about 3.2 GB | about 2.9 GB | Apache 2.0 |
| `7b` | Qwen2.5 7B Instruct | 4.7 GB (two files, fetched together) | about 5.5 GB | about 5.2 GB | Apache 2.0 |
| `8b` | Qwen3 8B | 5.0 GB | about 6 GB | about 5.6 GB | Apache 2.0 |
| `phi4-mini` | Microsoft Phi-4 mini Instruct | 2.5 GB | about 3.2 GB | about 2.9 GB | MIT |
| `granite4-micro` | IBM Granite 4.0 Micro | 2.1 GB | about 2.6 GB | about 2.4 GB | Apache 2.0 |
| `granite4-tiny` | IBM Granite 4.0 H Tiny | 4.2 GB | about 5 GB | about 4.6 GB | Apache 2.0 |

"Memory in all" is RAM and VRAM together. At the default nearly all of it is VRAM; with
`model.gpu_share` at `0`, or no GPU, all of it is RAM. The figures are estimates and vary with your system. The model
uses two CPU threads while it thinks and runs on its own thread, so the game never waits for
it. It loads before you play and unloads after ten minutes with no one playing.

On a server, the model runs on the server; a client that only plays there can press **Play
without it** and download nothing.

### Operators

`/craftkit status`, `/craftkit lodger status`, `leave` and `reset`, `/craftkit lodger show`, and a debug screen (F8, with an overlay on F9,
operators, and you in your own singleplayer world) that shows what it knows, what it plans and why. Every decision is logged in `config/lodger/decisions.log`.

<details>
<summary>What it can do (spoilers)</summary>

- Your things move. A chest is not quite how you left it. Something of yours is gone, and
  turns up somewhere that means something.
- Torches you did not place. A path you did not walk. A sign with one word.
- Sounds where there should be none: a door, steps behind yours, breathing when you crouch
  still.
- A figure, far off, wearing your skin, walking the path you walked yesterday.
- It builds. Slowly. Not far from you.
- It writes. A villager has a book for sale with your name on it. Read it. If you do not buy
  it, someone brings it to you. A letter in your chest. Its notes on your lectern. A page in
  your own book.
- One night you wake in your house, and it is not your house.
- One night you cannot sleep, because something is standing by your bed. It has brought
  something with it.
- Out walking, the woods are not empty. Something peeks from behind the trees, and the trees
  behind you are closer than they were.
- The villagers stop being themselves. One will not trade. One stabs you. One comes running to
  your door, as if something were after it.
- It sets things on you: a vex, a swarm of them, a rabbit that should not be, a creeper right
  behind you.
- Your senses go wrong. Sounds drag and echo. The screen fills with static. Something on the
  ceiling. A face as you wake.
- Your most valuable things rise into the sky in front of you. Later, they come back down.
- It has a temper. How you treat it changes how it treats you.
- In the end, it leaves, it takes, or it stays, depending on how you treated it.

</details>
