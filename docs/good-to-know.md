# Good to know

The things that surprise people, and the rules that hold whatever the model decides. The other
pages explain each part in full; this one is the short list to keep in mind.

## The model

- **The model decides by scoring, and writes only a few words.** Every event is a fixed choice
  the model scores; it cannot invent an event and cannot be talked to. The one place it writes
  is the words on a sign, a name tag or a chat line in your own name (`brain.text`, `generate`
  by default): a few words, letters and spaces only (digits only on the lines
  that count kills), shown handwritten examples
  of that kind of line first, through a blocklist, logged, and replaced by a handwritten line if
  any check fails. The blocklist (`BLOCKED` in `core/src/brain/text.rs`) holds slurs, sexual
  words and self-harm and nothing else; horror words are the mod's subject. See
  [Brain](brain.md#why-scoring-and-not-generation) and [Text](brain.md#text).
- **The model is downloaded, not shipped, and the game waits for it.** It is 1.1 GB, Qwen's
  own `q4_k_m` GGUF of Qwen2.5-1.5B-Instruct (`model.size` picks 0.5B at 491 MB or 3B at 2 GB
  instead). The first start shows Qwen's terms, then a progress bar over the title screen,
  fetches it to `config/lodger/models/<size>.gguf`,
  loads it, and only then lets the player in; a dedicated server does the same at startup with
  progress in its log. A file that does not start with the GGUF magic is refused and nothing is
  left behind. `model.url` in the config says where from. The host does the fetching, for any
  Rust mod's declared resources. See [Model](model.md#downloading).
- **Without the model the mod still runs.** The rule brain picks from the same choices with fixed
  weights. The lodger is duller, not absent. `/lodger status` says which brain is in charge.
- **Thinking takes tens to hundreds of milliseconds and never stalls the server.** The model runs
  on a thread of its own; an answer comes back on a later tick. By then the situation may have
  changed, so every event re-checks its preconditions as it starts and quietly does nothing if they
  no longer hold. See [Events re-check](entity.md#events-re-check-before-they-happen).
- **A newer question replaces an older one still waiting.** Each purpose has a key (`plan`,
  `react`, `witness`), and a queued question with the same key is dropped in favour of the new
  one, so the lodger never goes on stale news. A question already being scored cannot be
  recalled. The queue holds 64; past that a question is refused and the rule brain answers it.
- **The model's answer is not the last word.** The rules in [What it never does](#what-it-never-does)
  are checked after every decision, and an event they forbid is replaced by `wait`. Nothing in the
  situation text, the config or the memory can turn a rule off.
- **There is a debug panel, for operators.** F8 in a world (or `/craftkit lodger debug`)
  opens a strip down the right of the screen, the world still in view and running beside it,
  showing what the host senses of you as meters, the plans it holds with why each waits, what the mind holds, every event with whether it
  could happen right now and why not, a Fire button on each, and what landed. Players without
  operator level on a server cannot open it; the owner of a singleplayer world can, cheats or
  not. See [Running a stay quickly](development.md#running-a-stay-quickly).
- **It shares the GPU with the game.** Half the model's layers run on the GPU by default (`model.gpu_share`), and a question is fed in small chunks with rests between, so the frames do not wait; `0` keeps it off the GPU entirely.
- **It costs CPU on the server.** Two threads while scoring, about 1.6 GB of RAM while the
  default model is loaded (700 MB for the small one).
  It loads when a target is online and unloads after `model.idle_minutes` with no one to think
  about. A client connected to someone else's server pays nothing.
- **Decisions are logged.** `config/lodger/decisions.log`, and `/lodger decisions`, show every
  situation text, question, choice and confidence, so a strange event can always be traced to what
  the lodger believed at the time. The log is the only way to tune the prompts; keep it on while
  developing.
- **Runs are reproducible from the log, and otherwise not.** Scoring is deterministic, the
  pick from the scores is a seeded draw (`brain.temperature`), and the seed is in every decision
  log entry, so a strange run can be replayed from its seed and situations. Two worlds never go
  the same way on their own: the situation text carries times, positions and what the player
  did, and the seed differs per start.

## Where things run

- **The mind is Rust, the body is Java, and the body is not the lodger's.** Everything the
  lodger knows and decides is in the `core/` crate, loaded as a shared library through Java's
  FFM API by craftkit, the host (`host/`), and tested with `cargo test` without a game. The host
  only watches the world and carries out commands, for any Rust mod. A bug in what the lodger
  *did* is the host's; in what it *chose* is the mind's, and the decision log says which. See
  [The SDK](sdk.md) and [Architecture](architecture.md).
- **One jar per loader, on both sides.** `lodger-fabric-…` or `lodger-neoforge-…`, with the
  craftkit host nested inside (Fabric's `jars`, NeoForge's jar-in-jar). Two Rust mods on one
  server each nest a host and the loader keeps the newest once. The lodger's own part has no
  Java in it; the host is what runs.
- **Everything that decides runs on the server.** The client renders what it is told, plays
  sounds it is told to, and reports nothing but what vanilla reports already. Both sides need
  the mod: the client for the entity, the replay figure and the sounds, the server for all the
  rest. A vanilla client is refused at login with a message saying so.
- **One lodger per world, one target at a time.** It attaches to one player and the stay is that
  player's. Others in the world are witnesses: they see the traces, never the figure, and they
  hear nothing. Who is the target is the lodger's choice and it can change once, at a stage
  boundary. See [Multiplayer](entity.md#multiplayer).
- **State is in the world save.** `data/lodger.dat` next to the level's other saved data holds
  the mind's bytes: the stage, the target, the memory and the open plan. Copying a world copies
  its lodger. Deleting the file is `/lodger reset` by hand. The body keeps nothing of its own.
- **Peaceful does not remove it.** The lodger is not a hostile mob and does not despawn with
  them. `enabled: false` in the config, or `/lodger leave`, does. Difficulty changes how far it
  goes at the end (see [How it ends](entity.md#how-it-ends)).
- **Nothing leaves the machine but the model download.** The situation text names the player and
  what they did; it is written to the log and read by the model in the same process, and goes
  nowhere else. The mod opens one outbound connection in its life, to `model.url`.

## What it never does

These hold in every stage, under every brain, whatever the config says. They are what make the
lodger a presence rather than a mob, and they are also what makes it safe to put on a server.

- **It never breaks a block a player placed.** What it removes of theirs it puts back within
  reach the same night: a torch moved three blocks, a painting swapped for another, a bed
  shifted one block. It never opens a wall, never touches anything in view, and never fires
  anything that explodes: a lever with TNT within 16 blocks of what it powers is not a lever to
  the lodger. What it does dig is what a player digs: natural stone, dirt and ore, one block at a
  time at a player's pace, never within 8 blocks of anything placed (checked on every command, the debug screen's Fire too) but `alcove`, which may take one block of natural stone from behind a house wall (owner) and never into a cave, water
  or lava it can see coming (see [Like a player](entity.md#like-a-player)).
- **What it takes is the very thing it gives back.** The host keeps the taken stack with the
  world, enchantments, wear and name and all (until now a taken enchanted tool came
  back plain).
- **It never takes more than one item at a time, and never anything from the inventory, a hand,
  an armour slot or the ender chest.** From a chest the player opens often, one item, one that
  the memory says they use. What it takes it holds, and it always comes back, somewhere. Items
  it moves between the player's own chests are all still the player's.
- **It never harms a player before the last stage, and never a witness.** Up to then it does not
  touch anyone. In the last stage, what it does depends on the difficulty, and once it is done
  the stay is over.
- **It never pretends to be anyone but you, and never speaks to anyone but you.** It wears
  your skin and your name, and with `events.chat` it shows in your player list, joins and leaves
  in your name, and answers you in chat in your own name, to you alone. Never another player's
  name, never a line a witness sees. Its words come from the mod's phrase bank by the model's
  choice, or, with `brain.text: generate`, from the model under a letters-only grammar, a length
  cap and a blocklist (see [Text](brain.md#text)).
- **It reads your chat, and sometimes answers.** What you type is a sense: from stage two a
  line about it is often answered, in your own name to you alone, a few seconds later, at most
  three times a day; your words, cleaned to letters, reach the model's prompt for that answer. On a
  server, say so in the server's rules; it is the server's own chat and the memory keeps at most
  eight lines.
- **Nothing changes within 64 blocks of the target while the target faces it.** Every block an
  event would change is checked, and an event in view waits for them to look away. It works
  behind you, in rooms you left, and while you sleep. A player who stands and watches a chest
  watches a chest.
- **It answers how it is treated, within the preset's bounds.** Chase it, break its things,
  take the thing back, wall its place, insult it, and its temper rises: more a day (up to one
  and a half times the preset), stages ending sooner, bolder events earlier, the ending tilted
  toward taking. Build at its place, leave the thing, look without chasing, and it eases. It
  never does anything it would not do; it does what it does sooner and more. The preset is the
  ceiling: `calm` provoked is still calm. See [Provocation and temper](entity.md#provocation-and-temper).
- **It never does the same thing twice in a row,** and no event more than a few times in a stay.
  Cooldowns, caps and a menu of eight drawn from a hundred and some keep a stay from wearing
  thin, and the menu is filtered by what is around and what the target is doing before the model
  sees it, so a player with no lake never hears of boats. See
  [What is offered when](entity.md#what-is-offered-when).
- **It never changes the world's time or weather.** The sun blinking out and the red rain are
  the target's client rendering a hint for a few seconds. The server's clock and sky are the
  same for everyone, and a witness looking up sees noon.
- **It never follows the target into another dimension.** The Nether and the End are the one
  place it is not. Coming back, it has been busy.
- **It never spawns anything** (what it gives without taking it first comes out of what it dug, counted in `Memory::stores`), with one exception: in a chest of fresh loot, which is the world's, a redstone lamp, a candle or the player's own head may turn up (`loot_gift`, owner). No mobs, no items that did not exist. Everything it places it
  dug, cut, picked, cooked, crafted from those or took first, as a player would have to: its
  cobblestone is from its tunnels, its logs from trees it felled and replanted, its torches from
  both. A mob it uses is one that was already there, redirected. The memory counts all of it.

## Things that bite

- **A chest is the lodger's to use only if it is the home's.** Placed by them, standing in the
  house they sleep in (a village house counts once they sleep there), or opened on three
  different days. A village chest opened once on the way past is never touched. The house's
  doors and torches count the same way, whoever placed them; what it breaks is still natural
  ground alone.
- **Blocks the lodger places stand where a player could place them.** `set_block` refuses a
  torch over air; with `settle` the host looks a few blocks down and up for ground, then for a
  wall (`wall_torch`, `oak_wall_sign`); a cairn's cobblestone sits on something solid (`rest`)
  and ice goes only on water (`only`). The mind knows positions, not terrain. When a block of
  an event fails, the rest of that event is not sent (no text for a sign that is not there),
  and a sapling is planted before it is taken from the chest, so a failed planting loses
  nothing.

- **The replay figure needs the target's skin on the client.** In singleplayer the client has
  it. On a server every client already fetches skins by profile, so the witnesses' clients would
  have it too, but witnesses never see the figure, by design.
- **`You may not rest now` with nothing nearby is the lodger**, in stage three and up: it leans
  on the vanilla check, which is why it reads exactly like vanilla. There is no other message.
- **The recorded routes are the target's own movements.** They are kept in the world save,
  downsampled to at most 64 routes of 256 points, and never leave it. A server operator should
  know they are there; a player on a public server should be told the mod records paths.
- **The lodger's own torches and signs are the mod's blocks.** They look like vanilla's and drop
  vanilla's, and the memory knows which they are, so a mod that strips unknown blocks strips
  them.
- **The native must match the machine.** The lodger's jar carries its mind built for Linux,
  macOS and Windows, x86_64 and arm64 each. On anything else the host logs `no native for
  <os>-<arch>` and the lodger is not there at all: its assets load, nothing else.
  `/craftkit status` says so. Build the native yourself (see
  [Development](development.md#building-the-native)) and put it in the jar, or in
  `config/craftkit/natives/lodger/<os>-<arch>/` where the host would have extracted it.
- **The glitches are the target's client lying to the target,** on the server's instruction.
  A screenshot shows them; a witness standing beside the target sees nothing. F3 shows the true
  time and light. That is the design; a player who reads F3 to check is playing the game.
- **A model file in a wrong place is a silent fallback.** If `qwen.gguf` is missing or unreadable
  the rule brain takes over and the only sign is `/lodger status` and one line in the log at
  load. Check both when the lodger seems dumb.
- **Changing `brain.*` thresholds mid-stay changes behaviour at once.** Nothing is recomputed,
  but the next decision uses the new numbers. The stage and the memory stay as they were.
