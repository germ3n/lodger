# Brain

How the lodger decides (`core/src/brain/`): a short plain-English account of what it knows, a few
questions each with fixed choices, a model that scores the choices, and rules that have the last
word. The model layer under it is in [Model](model.md); the events it chooses between are in
[Events](entity.md#events).

## Why scoring and not generation

A 0.5B model asked to *write* what a monster should do next writes nonsense a good part of the
time, and every bit of it would have to be parsed, checked and translated into something the mod
can do. The engine in `~/engine` does not ask the model to write. It gives the model a situation
and a question with fixed choices, decodes the situation once, and for each choice measures how
probable the model finds that choice as the continuation: the mean log-probability of the
choice's tokens after `The answer is: `. Softmax over the choices gives a probability each, the
best wins, and its probability is the confidence.

So:

- **Nothing to parse.** The answer is one of the strings the mod wrote. There is no output text.
- **Bounded.** The model cannot choose an event that is not offered, cannot speak, and cannot be
  steered by anything in the situation text: the text can shift which choice scores best, which
  is the point, but every choice was the mod's.
- **Cheap.** One decode of the situation, then a handful of tokens per choice. Tens to a few
  hundred milliseconds on two CPU threads at 512 context.
- **Deterministic where it counts.** The same text and choices *score* the same every time.
  The pick from those scores is a seeded draw (see [Picking](#picking)), so the log can replay a
  run from its seed, and the same situation does not get the same answer on every start.
- **Calibrated enough.** A small model's judgment on `Should it event tonight?` given `The player
  lit every room yesterday and logged off early` is not deep, but it is not random either, and
  it is different from its judgment given `The player walked toward the figure twice`. That
  difference is all the lodger needs, and the confidence says how sure it was.

What a 0.5B model is bad at, the brain does not ask: no arithmetic, no spatial reasoning, no
long chains. Positions become `near the door`; counts become `many`; the question is always
`which of these`, never `where exactly`.

## Picking

The engine always took the best-scoring choice, and the owner noticed what that does: the same
situation on every start gets the same answer (2026-11-09). So the pick is a draw: the scores
are divided by `brain.temperature` (0.7 by default), softmaxed, and a seeded random number
lands on one. At 0 the best wins every time, as the engine; at 1 the model's own odds; below 1
the best most of the time; above 1 flatter. The confidence logged is always the chosen choice's
plain probability, so logs read the same way at any temperature, and the thresholds still
compare against the *best* choice's probability, so a low-confidence question is still `wait`.

The generator is a SplitMix64 in `model/mod.rs`, seeded at start from the clock and the game
directory, reseeded with the world's seed when a world opens, and the seed is written at the top
of each decision log entry: a stay replayed from the same seed and the same situations draws the
same way, which is how a strange run is reproduced.

## Does the model remember?

No. Every ask is a fresh 512-token context: the frame, the memory's lines, the question. The
model has no state from one ask to the next, on purpose: a stateful conversation would grow past
the context in a day, cost more each turn, and drift. What remembers is the lodger's
[Memory](memory.md), in Rust, bounded and saved with the world, and `Memory::describe` is what
hands the model its past each time: the home, the habits, the last events and their reactions,
the open chains, the temper, the want. The model reads a summary of everything that happened
and decides once; the memory is the context.

## The situation text

Built by `brain/situation.rs` from `Memory::describe` (see [Describing](memory.md#describing))
with a fixed frame around it:

```
You are a quiet presence living unseen in a player's house in a game. You never chase, never
hurt, and never hurry. You learn the player's habits and event on them when they are away or
asleep, to make them feel known. You want one thing from them.

What you know:
<the memory's lines>
```

Then, per question, appended and scored, then rewound:

```

<question>
The answer is: <choice>
```

The frame is the same every time, the memory's lines are templates, and the question and
choices are from the tables below, so the model only ever reads text the mod wrote. The whole
thing is logged with the decision.

Qwen2.5-Instruct is a chat model, and the engine feeds it plain text with no chat template,
which works for scoring: we are measuring continuations, not holding a conversation. The todo
has an item to measure whether the template helps.

## The questions

A decision is a small set of questions asked together on one situation (one decode, several
questions). Which set depends on why the brain is asking.

### Planning (`plan`)

On the timer, `brain.plan_every` seconds of the target's play, and when the target leaves or
sleeps (the lodger's working hours).

| Question | Choices | Becomes |
| --- | --- | --- |
| `Should it do something soon?` | `nothing yet`, `something small`, `something it will notice` | `wait`, or the event's size class: small is the sounds, `flicker`, `pet_growl`, `open_door`, `close_door`, `open_chest`, `shuffle_chest`, `move_light`, `frame_rotate`; noticed is the rest the stage offers. |
| `What should it do?` | The menu: at most `events.offered` events the stage offers, the memory can supply, and cooldowns and caps allow, drawn to favour what has not happened yet (see [How often](entity.md#how-often)), as words: `open the chest`, `move a torch`, `take the pickaxe`, `put the pickaxe on the bed`, `cook something`, `turn the painting`, `show itself far away`, `walk the player's path`; `wait` always among them | The event. |
| `When?` | `while they are away`, `while they sleep`, `as they come home`, `now` | The event's time: the next matching moment, within a day. |
| `Where?` | Up to five places the memory names: `by the door`, `on the bed`, `in the chest they use`, `at the end of their path`, `at its place` | The event's place. |

### Reacting (`react`)

On a trigger: the target opened a container, saw a figure, came home to a landed event, lit or
walled a spot, went to sleep.

| Question | Choices | Becomes |
| --- | --- | --- |
| `How does the player feel?` | `calm`, `curious`, `uneasy`, `afraid`, `angry` | A row in the memory's reactions, which later situations describe. Nothing else; it is the lodger's reading, not the truth. |
| `How does it take this?` (after a provocation or an offering) | `it does not mind`, `it is hurt`, `it is angry`, `it is pleased` | Scales the temper move the table in [Provocation and temper](entity.md#provocation-and-temper) prescribes: `does not mind` halves it, `angry` doubles it, `pleased` makes a down move larger. The direction is the table's, never the model's. |
| `Should it answer this?` | `no`, `with something small`, `by showing itself`, `by giving back` | `wait`, a small event now, `far_figure`, or `place_taken` at the target's place. The bar is `brain.react_threshold`. |

### The want (`want`)

Once, at move-in.

| Question | Choices |
| --- | --- |
| `What does it want from this player?` | `a place of its own`, `to be looked at`, `one thing of theirs` |

### The target (`witness`)

Once, at move-in, in multiplayer with `multiplayer.pick` at `model`. The situation text is the
`Others` lines, one per player, and the question is `Which player does it move in with?` with
their numbers as choices, `the first`, `the second`, up to eight. Names never appear.

### The ending (`settle`)

At the start of stage five, when `stages.ending` is `difficulty`, the arc decides the ending from
the want's score and the difficulty, not the model. The ending's weight is a design decision and
stays out of the model's hands. Each of its three days, the model is asked once, `How does
today go?`, with the day's beats in their two forms as the choices (`the figure at its door` or
`the figure on its roof`; `the frames emptied` or `the paintings turned`), so endings differ in
detail and agree in shape (see [How it ends](entity.md#how-it-ends)).

### Chains

An open chain (see [Chains](entity.md#chains)) reaches the brain two ways: the situation text
says what is open and for how long, and `offer.rs` weights the chain's next steps up, or offers
them alone when the step is exclusive. The model still chooses, and `wait` is still there; what
a chain adds is that the lodger's next move is *about* its last one, which is what makes a stay
read as one story rather than a list.

## Text

Signs, books, named animals and, with `events.chat`, chat lines in the target's name need words.
Two ways, by `brain.text`:

- **`bank`, the default.** The mod's phrase bank (`brain/text.rs`): hundreds of short lines,
  one to four words, written by hand and grouped by want, stage and place (`for a sign by the
  door in stage two when it wants a place`). The model is asked `What does the sign say?` with
  eight lines drawn from the group as the choices, and scores them as it scores everything. The
  lodger never writes, and still every sign is its choice among many, for this player, now.
- **`generate`, an experiment.** The model writes the words itself: up to three, decoded one
  token at a time with every token outside letters and spaces masked out, stopped at a newline,
  passed through a blocklist and a length check, logged with the situation, and dropped (the
  bank used instead) if any check fails. A 0.5B model writes three plausible words most of the
  time and something wrong some of the time; the calibration pass (todo) reads the log of a few
  stays and decides whether `generate` becomes the default or stays an experiment.

Either way the words go only where the catalogue puts words: signs, books, a name tag, a chat
line to the target in the target's own name. Never a message to a witness, never another
player's name.

## From choices to an event

`brain/decide.rs` turns a reply into a plan entry:

1. **Confidence.** The first question's best probability below `brain.event_threshold`
   (`brain.react_threshold` for reactions) is `wait`, whatever the other answers. A model that is
   not sure is a lodger that waits, and waiting is free.
2. **Event.** The `What` choice maps back to an event id. `nothing yet` on the first question is
   `wait` regardless.
3. **Time and place.** From `When` and `Where`, resolved against the memory into a tick and a
   block position. A place the memory cannot resolve, because the chest is gone, falls back to
   the home's door.
4. **Rules.** `rules.rs` checks the event against [What it never does](good-to-know.md#what-it-never-does)
   and the stage: not in view within `events.min_distance`, not a witness's container, not twice at
   the same place in a day, not an event the stage does not offer, not an event in `events.disabled`,
   not within the stage's least days of a stage change. A failed rule is `wait`, and the log
   says which rule.
5. **Plan.** One entry at most per lodger. A new plan replaces an unstarted one.

## Cadence and keys

The brain asks to plan only while the day's budget has room (`stages.events_per_day`; see
[How often](entity.md#how-often)). With the budget spent, triggers still fire `react` asks, but
their answers can only be `no` or a sound, which count half, until the day turns. This is what
keeps a stay to a few dozen events whatever the model would like.

Every ask has a key: `plan`, `react`, `want`, `witness` or `settle`. A queued ask with the same
key is superseded by a newer one and its callback gets `superseded`, as in the engine, so a
`react` fired three times in a second costs one scoring and goes by the newest. An ask already
being scored runs to the end. The queue holds `brain.queue` asks; past that the ask is refused
and the rule brain answers it at once.

Asks are rare by design: one `plan` every `brain.plan_every` seconds of play, a `react` on a
trigger with at least ten seconds between triggers, and the three others once. A server with one
lodger scores a few times a minute at most, each in tens to hundreds of milliseconds on the
worker thread.

## The rule brain

`brain/rules_brain.rs` answers the same questions with the same choices from fixed weights per
stage and want, with a little randomness from a per-stay seed, when the model is not there,
when the queue is full, or when `brain.force_rules` is on. Its weights are tables in the file,
with the reasoning beside them. It is also the baseline the model is judged against: a stay on
the rule brain is a competent haunted house; the model's job is to be *that player's*.

## The decision log

`config/lodger/decisions.log`, one entry per ask, when `log.decisions` is on:

```
2026-11-08T21:40:12Z world=New World stage=2 day=4 key=plan brain=model took=140ms
--- situation
<the whole text>
--- answers
Should it do something soon? -> something small (0.52) [nothing yet 0.31, something it will notice 0.17]
What should it do? -> move a torch (0.44) [...]
When? -> while they sleep (0.61) [...]
Where? -> by the door (0.48) [...]
--- result
event=move_light at=(12,64,-7) when=sleep rule=ok
```

`/lodger decisions [n]` prints the last n to the operator. The log is how the prompts and
thresholds get tuned: the todo's calibration item is to play ten stays, read the log, and move
what reads wrong. It is also how a player's report of a strange event is traced to what the lodger
believed.
