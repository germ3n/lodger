# Config

`config/lodger.json`, written with every default on first run. Every key is listed here; a key
missing from the file takes its default, and an unknown key is logged and ignored. Keys under
`brain` and `stages` are read at each decision, so a change applies to the next one; the rest are
read at world load.

The hard rules in [What it never does](good-to-know.md#what-it-never-does) are not here. There
is no key that turns one off.

## General

| Key | Default | What it changes |
| --- | --- | --- |
| `enabled` | `true` | Whether a lodger moves into new worlds and existing ones keep theirs. `false` ends the stay in every world as `/lodger leave` would. |
| `preset` | `"balanced"` | `calm`, `balanced`, `intense` or `relentless`: a set of values for the keys below (see [Presets](#presets)). A key the file names wins over the preset; the file written back shows the preset's values filled in. |
| `log.decisions` | `true` | Writes every decision to `config/lodger/decisions.log`. Keep it on while developing; see [The decision log](brain.md#the-decision-log). |
| `log.keep_mb` | `16` | The log rotates past this size, keeping one older file. |

## Presets

Four ways to play, as the owner asked (2026-11-09): how much happens, how bold it is, and what
is left out. `balanced` is the design as the docs describe it and the default; the others change
the keys below and nothing else. The hard rules in [What it never does](good-to-know.md#what-it-never-does)
hold under every preset; a preset turns events off, never rules.

| | `calm` | `balanced` | `intense` | `relentless` |
| --- | --- | --- | --- | --- |
| Events a day by stage | 0.5, 1, 1.5, 2 | 1, 2, 4, 6 | 2, 3, 5, 8 | 4, 6, 8, 12 |
| Stage lengths, least to most days | 2 to 5, up to 4 to 9 | 1 to 3, up to 3 to 7 | 1 to 2, up to 2 to 6 | 1 to 1, up to 1 to 3 |
| Move-in after | 3 days | 3 days | 3 days | the first night |
| Cooldowns | doubled | as written | three quarters | halved |
| Keeps its distance | 32 blocks | 24 | 16 | 12 |
| Menu offered | 8 | 8 | 8 | 12 |
| Acts at confidence | 0.5 | 0.4 | 0.3 | 0.25 |
| Chat and player list | off | on | on | on |
| Left out | the sleep events, the near figures, `lights_fail`, `red_rain`, `your_voice`, `knock`, `stare`, `sign_word`, `tunnel_toward` | nothing | nothing | nothing |
| Ending | `leave`, always | by difficulty | `take` on every difficulty | `take` on every difficulty |

So `calm` is a presence that never touches your sleep, your screen or your chat, keeps 32 blocks
off, and leaves at the end; `relentless` is a week of something every evening and the death at
the end whatever the difficulty. The menu's config page shows the preset and what it set.

## Model

| Key | Default | What it changes |
| --- | --- | --- |
| `model.size` | `"0.5b"` | Which Qwen2.5-Instruct to run: `0.5b` (491 MB, about 700 MB of RAM, tens of milliseconds a question), `1.5b` (1.1 GB, about 1.6 GB of RAM, a few times slower) or `3b` (2.0 GB, about 3 GB of RAM, slower again). Changing it downloads the new file at the next start; the old one stays until deleted. See [Which model](model.md#which-model). |
| `model.url` | empty | Where to fetch from. Empty means Qwen's own `q4_k_m` file for `model.size` on Hugging Face. |
| `model.path` | empty | Where the model is kept. Empty means `config/lodger/models/<the size's file name>`. Relative to the game directory unless absolute. Must be a loose file: llama.cpp maps it. |
| `model.threads` | `2` | Threads llama.cpp uses while scoring. More is faster and takes more from the game. |
| `model.context` | `512` | The context window. The situation text plus one question and its longest choice must fit; the brain truncates the memory's lines to make sure. |
| `model.idle_minutes` | `10` | Unloads the model after this long with no target online. Loading again takes a second or two. |
| `model.gpu_layers` | `0` | Layers offloaded to Metal or Vulkan. `0` is CPU only, the default, so every server behaves alike. Changing it is an experiment, not a setting. |

## Brain

| Key | Default | What it changes |
| --- | --- | --- |
| `brain.plan_every` | `90` | Seconds of the target's play between planning asks, on top of the trigger asks. |
| `brain.event_threshold` | `0.4` | The confidence below which an answer is `wait`. Higher is a more patient lodger. |
| `brain.react_threshold` | `0.55` | The same for reactions to what the target just did, which are cheaper to get wrong than plans, so the bar is higher. |
| `brain.queue` | `64` | The most questions waiting. A full queue refuses; the rule brain answers. |
| `brain.force_rules` | `false` | Uses the rule brain even with a model loaded. For comparing the two from the log. |
| `brain.temperature` | `0.7` | How the answer is picked from the model's scores. `0`: always the best choice, so the same situation gets the same answer every time, as the engine did. Above `0`: a seeded draw weighted by the scores, sharper near `0`, the model's own odds at `1`, flatter above. The seed is logged with every decision so a run can be replayed. See [Picking](brain.md#picking). |
| `brain.text` | `"bank"` | How signs, books and chat lines get their words. `bank`: the model scores phrases from the mod's bank, hundreds of short handwritten lines by want and stage. `generate`: the model writes up to three words itself, letters and spaces only, through a blocklist, every line logged. An experiment; see [Text](brain.md#text). |

## Stages

| Key | Default | What it changes |
| --- | --- | --- |
| `stages.least_days` | `[1, 2, 2, 3, 1]` | The least in-game days each stage lasts, in order. See [The five stages](entity.md#the-five-stages). |
| `stages.most_days` | `[3, 5, 5, 7, 3]` | The most. A stage that has not ended on its own ends here. |
| `stages.events_per_day` | `[1, 2, 4, 6, 0]` | The budget of events per in-game day in each stage. Sounds count half. Stage five's is its own script. See [How often](entity.md#how-often). |
| `stages.start_after_days` | `3` | In-game days a world is played before the lodger moves in. A fresh world is a fresh world first. |
| `stages.ending` | `"difficulty"` | `"difficulty"`, `"leave"`, `"take"` or `"keep"`. Which ending the last stage works toward; `"difficulty"` reads the world's. See [How it ends](entity.md#how-it-ends). |

## Events

| Key | Default | What it changes |
| --- | --- | --- |
| `events.min_distance` | `24` | Blocks from the target within which nothing happens while the spot is in view. Lowering it is the one way to make the lodger bolder, and it is still bounded by the rules. |
| `events.disabled` | `[]` | Event ids the brain never offers as choices. For servers that want, say, no `take` or no `sign_word`. See [Events](entity.md#events). |
| `events.offered` | `8` | How many eligible events the brain is offered per plan. Fewer is a more predictable lodger; more is a longer prompt. |
| `events.cooldown_scale` | `1.0` | Multiplies every event's cooldown. `2.0` is a lodger that repeats itself half as often and so does less. |
| `events.sounds` | `true` | Whether the target hears anything at all. |
| `events.figure` | `true` | Whether the far presence and the replay figure are ever shown. |
| `events.chat` | `true` | Whether it may appear in the player list, in join and leave messages and in chat, always as the target and only to the target. `false` on servers that want the chat untouched. See [Chat and the player list](entity.md#chat-and-the-player-list). |
| `events.glitches` | `true` | Whether the target's client is ever sent a glitch: the lights failing, the sun blinking out, red rain and the rest of [Glitches](entity.md#the-sky-and-the-light). |

## Multiplayer

| Key | Default | What it changes |
| --- | --- | --- |
| `multiplayer.pick` | `"model"` | `"model"`, `"first"`, `"random"` or a player name. How the target is picked when several players are there. See [Multiplayer](entity.md#multiplayer). |
| `multiplayer.retarget` | `true` | Whether the lodger may change target once, at a stage boundary, if the current one has not played for `multiplayer.absent_days` real days. |
| `multiplayer.absent_days` | `7` | See above. |
| `multiplayer.witness_traces` | `true` | Whether witnesses see traces at all. `false` makes the stay invisible to everyone but the target, which is a different and lonelier mod. |
