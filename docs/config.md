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
| `log.decisions` | `true` | Writes every decision to `config/lodger/decisions.log`. Keep it on while developing; see [The decision log](brain.md#the-decision-log). |
| `log.keep_mb` | `16` | The log rotates past this size, keeping one older file. |

## Model

| Key | Default | What it changes |
| --- | --- | --- |
| `model.url` | Qwen's `qwen2.5-0.5b-instruct-q4_k_m.gguf` on Hugging Face | Where `/lodger model download` fetches from without a URL. |
| `model.path` | `config/lodger/models/qwen.gguf` | Where the model is kept. Must be a loose file: llama.cpp maps it. |
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
| `events.glitches` | `true` | Whether the target's client is ever sent a glitch: the lights failing, the sun blinking out, red rain and the rest of [Glitches](entity.md#the-sky-and-the-light). |

## Multiplayer

| Key | Default | What it changes |
| --- | --- | --- |
| `multiplayer.pick` | `"model"` | `"model"`, `"first"`, `"random"` or a player name. How the target is picked when several players are there. See [Multiplayer](entity.md#multiplayer). |
| `multiplayer.retarget` | `true` | Whether the lodger may change target once, at a stage boundary, if the current one has not played for `multiplayer.absent_days` real days. |
| `multiplayer.absent_days` | `7` | See above. |
| `multiplayer.witness_traces` | `true` | Whether witnesses see traces at all. `false` makes the stay invisible to everyone but the target, which is a different and lonelier mod. |
