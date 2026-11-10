# Config

`config/lodger.json`, written with every default on first run. Every key is listed here; a key
missing from the file takes its default, and an unknown key is logged and ignored. A value of
the wrong type does not stop the mod: a whole number written with a point (`2.0`) is read as
whole; text that reads as what the key wants (`"2"`, `"0.25"`, `"true"`, `"off"`) is taken as
that; anything else leaves that key at its preset's value; each is logged as a warning naming
the key. Only a file that is not JSON at all, or a value out of its bounds, refuses to load,
with the reason in the log. The file is read when the game starts and again when the settings
screen's Done saves it; a change made by hand applies at the next start.

The hard rules in [What it never does](good-to-know.md#what-it-never-does) are not here. There
is no key that turns one off.


**In the game:** the title screen's "Lodger settings" button (top left) changes the useful keys
without opening the file: whether the model is used, its size, its GPU share and its pause; the preset and the event groups
(sounds, glitches, chat, the figure); plans at once and the seconds between decisions; the
words (generated or the bank) and their variety. Its **All keys** tab has every key of this
file, by section: a toggle for a yes or no, a field for the rest (a list as JSON, `[40.0, 40.0]`),
a changed key's name in yellow, and each key's line here as its tooltip. A key set in this file
wins over the preset, so a key changed by hand stays when another preset is picked; its ↺
button sends it back to the preset (the key taken out of the file, the preset's value written
in its place), and All to preset does so for every key but `preset`, `enabled` and the `model` and
`log` sections (a model size sent back would fetch the default's). Changing
tabs applies the tab left, so All keys shows the file as it is now. Done writes the changes of
both tabs here and loads the lodger again, which checks them: a value it refuses is said under the list, and the file is put back. A
new model size is fetched at once.

## General

| Key | Default | What it changes |
| --- | --- | --- |
| `enabled` | `true` | Whether a lodger moves into new worlds and existing ones keep theirs. `false` ends the stay in every world, where `/craftkit lodger leave` ends one gently. |
| `preset` | `"intense"` | `calm`, `balanced`, `intense` or `relentless`: a set of values for the keys below (see [Presets](#presets)). A key the file names wins over the preset; the file written back shows the preset's values filled in. Picking another preset on the settings screen takes the old preset's values out of the file, the ones you never changed, so the new preset's apply, and writes the file again with them. |
| `log.decisions` | `true` | Writes every decision to `config/lodger/decisions.log`. Keep it on while developing; see [The decision log](brain.md#the-decision-log). |
| `log.keep_mb` | `16` | The log rotates past this size, keeping one older file. |

## Presets

Four ways to play, as the owner asked: how much happens, how bold it is, and what
is left out. `balanced` is the design as the docs describe it, and `intense` the default (owner: "make the
default preset be the intense one"), denser and sooner; the others change
the keys below and nothing else. The hard rules in [What it never does](good-to-know.md#what-it-never-does)
hold under every preset; a preset turns events off, never rules.

| | `calm` | `balanced` | `intense` | `relentless` |
| --- | --- | --- | --- | --- |
| Events a day by stage | 0.5, 1, 1.5, 2, 1 | 1, 2, 4, 6, 3 | 2, 3, 5, 8, 4 | 4, 6, 8, 12, 6 |
| Stage lengths, least to most days | 2 to 5, up to 4 to 9 | 1 to 3, up to 3 to 7 | 1 to 2, up to 2 to 6 | 1 to 1, up to 1 to 3 |
| Move-in after | 3 days | 3 days | 3 days | the first night |
| Cooldowns | doubled | as written | three quarters | halved |
| Keeps its distance (`events.min_distance`) | 80 blocks | 64 | 48 | 32 |
| Menu offered | 8 | 8 | 8 | 12 |
| A small event costs (`events.small_cost`) | a whole one | a quarter | a quarter | a quarter |
| Acts at confidence | 0.5 | 0.4 | 0.3 | 0.25 |
| Chat and player list | off | on | on | on |
| Left out | the sleep events (`bed_breaks` and `foot_of_bed` among them), the near figures, every screen glitch (`events.glitches` off), `lights_fail`, `red_rain`, `your_voice`, `knock`, `stare`, `sign_word`, `tunnel_toward` | nothing | nothing | nothing |
| Ending | `leave`, always | by difficulty | `take` on every difficulty | `take` on every difficulty |

So `calm` is a presence that never touches your sleep, your screen or your chat, keeps 80 blocks
off, and leaves at the end; `relentless` is a week of something every evening and the death at
the end whatever the difficulty. The settings page picks the preset; the file written back shows what it set.

## Model

| Key | Default | What it changes |
| --- | --- | --- |
| `model.enabled` | `true` | Whether the model is used at all. `false` downloads nothing, and the simpler built-in rules decide what it does; a player who only joins servers (where the server's model decides) needs nothing more. The download screen's **Play without it** sets it, and the settings' **Use the model** turns it back on, which brings the download screen up. |
| `model.size` | `"1.5b"` | Which model to run: `1.5b` (1.1 GB, about 1.6 GB of RAM, a few hundred milliseconds a question; the default, the owner's call, for sharper judgment and better words), `0.5b` (491 MB, about 700 MB of RAM, tens of milliseconds) , `3b` (2.1 GB, about 2.6 GB of memory, slower again; Qwen's research license) or `4b`, Qwen3-4B-Instruct-2507 (2.5 GB, about 3.2 GB of memory, the best reader). Changing it downloads the new file at the next start; the old one stays until deleted. See [Which model](model.md#which-model). |
| `model.url` | empty | Where to fetch from. Empty means Qwen's own `q4_k_m` file for `model.size` on Hugging Face. |
| `model.path` | empty | Where the model is kept. Empty means `config/lodger/models/<the size's file name>`. Relative to the game directory unless absolute. Must be a loose file: llama.cpp maps it. |
| `model.threads` | `2` | Threads llama.cpp uses while scoring. More is faster and takes more from the game. |
| `model.context` | `1536` | The context window, in tokens; llama.cpp wants a multiple of 32. The situation text plus one question and its longest choice must fit: the memory's detail lines (habits, places, people) are taken only while they fit, at 3.5 characters a token with 200 tokens kept for the frame and the question (`memory::describe_chars`). Larger costs memory (about 30 KB a token for the 1.5B), not speed. It was 512 (owner: "can we increase context window ... to 1500?"). |
| `model.idle_minutes` | `10` | Unloads the model after this long with no target online. Loading again takes a second or two. |
| `model.gpu_share` | `1` | The share of the model's layers run on the GPU (Vulkan on Windows and Linux, Metal on a Mac), the rest on the CPU: `0` the CPU alone, `1` all of it, the default (owner: "by default the full model should run on the gpu"; it was half before, after "limit it to like 50% gpu cuz it makes my game stutter"). With no GPU, all of it runs on the CPU whatever this says. The prompt is decoded in chunks of 64 tokens with a rest between (`model.pause_ms`), so no frame waits on one long burst; if the game still stutters, lower this. The old `model.gpu_layers` it replaced was never passed on, and llama.cpp put every layer on the GPU. |
| `model.pause_ms` | `1` | Milliseconds the model rests between each piece of its work while it thinks (every 64 tokens of a question, and before each piece when above 1): higher leaves more of the GPU and the CPU to the game at any moment, and its answers come slower; the game never waits on them either way. 0 to 100; the settings page steps it by 5. Owner: "is there a way to limit the amount of compute it uses". |

## Brain

| Key | Default | What it changes |
| --- | --- | --- |
| `brain.plan_every` | `90` | Seconds of the target's play between planning asks, on top of the trigger asks; 15 at least. |
| `brain.text_temperature` | `0.8` | How freely the model writes a sign or a line: 0 is the likeliest word every time (the same moment, the same line), higher draws among the likely ones (top 90 per cent). Lowered after lines repeated. |
| `brain.most_plans` | `3` | How many plans it holds at once; it asks again every `plan_every` while below it (`calm` 2, `intense` 4, `relentless` 5; one before, owner: "one event at a time kinda gets slow"). |
| `brain.event_threshold` | `0.4` | The confidence of the model's `Should it do something soon?` answer below which the decision is `wait`. Higher is a more patient lodger. Live. |
| `brain.react_threshold` | `0.55` | The same for the model's `Should it answer this?` after a trigger (live): below it, no answer. Reactions are cheaper to get wrong than plans, so the bar is higher. |
| `brain.queue` | `64` | Checked (1 at least) but not read: the queue holds 64 questions whatever this says (`MAX_QUEUED` in `model/queue.rs`). A full queue refuses; the rule brain answers. |
| `brain.force_rules` | `false` | Uses the rule brain even with a model loaded. For comparing the two from the log, whose `brain=` says which decided. |
| `brain.temperature` | `0.7` | How the answer is picked from the model's scores. `0`: always the best choice, so the same situation gets the same answer every time, as the engine did. Above `0`: a seeded draw weighted by the scores, sharper near `0`, the model's own odds at `1`, flatter above. The draws start from a seed that differs per start, joined by the world's seed. See [Picking](brain.md#picking). |
| `brain.text` | `"generate"` | How signs, chat lines and names get their words. `generate` (the default, the owner's call): the model writes the line itself, shown the bank for that kind of line as examples, a few words at most, letters and spaces only, through a blocklist, every line logged, and the bank used when a check fails. `bank`: the model picks among eight of the bank's lines for that kind, as it picks a choice (`brain/text.rs`). See [Text](brain.md#text). |

## Stages

| Key | Default | What it changes |
| --- | --- | --- |
| `stages.least_days` | `[1, 2, 2, 3, 1]` | The least in-game days each stage lasts, in order. See [The five stages](entity.md#the-five-stages). |
| `stages.most_days` | `[3, 5, 5, 7, 3]` | The most. A stage that has not ended on its own ends here. |
| `stages.events_per_day` | `[1, 2, 4, 6, 3]` | The budget of events per in-game day in each stage. Sounds count half, a small event a quarter (`events.small_cost`). Stage five has its ending's script besides (its budget was 0 before, and its own rows, `arrow_hits` among them, could never run). See [How often](entity.md#how-often). |
| `stages.start_after_days` | `3` | In-game days a world is played before the lodger moves in. A fresh world is a fresh world first. |
| `stages.ending` | `"difficulty"` | `"difficulty"`, `"leave"`, `"take"` or `"keep"`. Which ending the last stage works toward; `"difficulty"` reads the world's. See [How it ends](entity.md#how-it-ends). |

## Events

| Key | Default | What it changes |
| --- | --- | --- |
| `events.temper` | `true` | Whether how they treat it moves its temper (chasing the figure, breaking its lights or house, lighting or flooding its place, cursing it in chat up; building or leaving a chest at its place, watching the figure from afar, kind words down; a tenth of the way back each day), and its temper the pace: the day's events half again at the top and half at the bottom, a stage's least days halved above 0.7 and most stretched by half below 0.2, the bolder events a stage early above 0.6 and late below 0.2, three quarters of `events.min_distance` above 0.8, a fiercer fight, and the ending. It rests at 0.2 on `calm`, 0.3 on `balanced`, 0.4 on `intense`, 0.5 on `relentless`. `false` keeps it at rest. See [Provocation and temper](entity.md#provocation-and-temper). |
| `events.min_distance` | `64` | Blocks from the target within which nothing that changes a block happens while it is in view (`calm` 80, `intense` 48, `relentless` 32; 24 before, when the owner watched torches go up from farther off). Lowering it is the one way to make the lodger bolder, and it is still bounded by the rules. |
| `events.disabled` | `[]` | Event ids the brain never offers as choices. For servers that want, say, no `take` or no `sign_word`. See [Events](entity.md#events). |
| `events.offered` | `8` | How many eligible events the brain is offered per plan. Fewer is a more predictable lodger; more is a longer prompt. |
| `events.cooldown_scale` | `1.0` | Multiplies every event's cooldown. `2.0` is a lodger that repeats itself half as often and so does less. |
| `events.small_cost` | `0.25` | What a small event costs of the day's budget, at most: the ones they may not notice (a chest tidied, a tool worn, a frame turned, a sound behind a wall; the catalogue marks them). A quarter lets four of them fit where one noticed thing would, so the house drifts more while the things they will notice keep their pace. `1` counts them as any event (`calm`). 0.05 to 1. Owner: "should the amount of events per day be scaled up since most events are things the player doesnt notice?" |
| `events.sounds` | `true` | Whether the sounds (the door and chest sounds, the knocks, the late steps, the mining sounds, the pet's growl) may happen. Off, nothing in the group is offered or forced. |
| `events.figure` | `true` | Whether the far presence and the replay figure are ever shown. |
| `events.chat` | `true` | Whether the lodger may touch the target's chat and player list (a line in their own name, a whisper from themselves, their name twice in the list, the joined and left lines, the sleepers count). |
| `events.glitches` | `true` | Whether the client-rendered glitches (the flicker, the lights failing, the sun blinking, the red rain, the fog, the crosshair gone, the camera nudge, the field of view pulled, the music cut, the silence) may happen. |

## Multiplayer

| Key | Default | What it changes |
| --- | --- | --- |
| `multiplayer.pick` | `"model"` | Who the lodger moves in with when several players are on: `model` asks the brain once two are seen (`Which player does it move in with?`, their ordinals as the choices; live), `first` takes the first player seen. Alone for five minutes, the first seen is it either way. |
| `multiplayer.retarget` | `true` | Whether the lodger may change target once, at a stage's beginning from stage two, if the current one has not been seen for `multiplayer.absent_days` real days (by the computer's clock) and someone else is here. It keeps its want, its place, its house and what it did, and lets the old target's home, chests, habits and favourites go. |
| `multiplayer.absent_days` | `7` | See above. |
| `multiplayer.witness_traces` | `true` | Whether witnesses see traces at all. `false` holds back every change to the world (the house, their things, a sign) within 48 blocks of another player until they are gone, so only the target sees its work: a different and lonelier mod. Sounds, figures, the screen and chat go to the target alone either way. |
