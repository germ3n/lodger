# Calibration, round one

The first run of the lodger's brain against the real model: Qwen2.5-1.5B-Instruct
(q4_k_m), three scripted in-game days (`core/tests/calibration.rs`: a house with a bed, a
chest opened each morning, a door and two torches, a daily walk to a mine, three cows killed),
`plan_every` 30, `cooldown_scale` 0.2, eight events a day, temperature 0.7, seed 7. Run with
`cargo test -p lodger-core --test calibration -- --ignored --nocapture`; a minute on an M3 Pro
with four threads.

## What happened

22 asks answered, every one by the model (`brain=model`, none fell back to the
rules, none took too long). Seventeen decisions, thirteen events landed. The want was the
model's pick. Two reactions were asked after the player went to bed and two after they opened
the chest.

```
  it wants a place of its own (the model's pick)
  the model: nothing yet (18%), open the chest for a moment (87%)
  the model: something it will notice (73%), open the chest for a moment (87%)
  the model: something it will notice (76%), move things around in the chest (27%)
  the model, after slept: they feel angry (27%), answer by showing itself (43%)
  the model: something it will notice (84%), dripping water indoors (8%)
  the model: something it will notice (83%), knock on the door (36%)
  the model: something it will notice (83%), knock on the door (83%)
  the model: something it will notice (84%), dripping water indoors (71%)
  the model, after opened_chest: they feel uneasy (7%), answer with something small (43%)
  the model: something it will notice (83%), flicker a light (37%)
  the model, after slept: they feel uneasy (85%), answer with something small (45%)
  the model: something it will notice (85%), silence the world's sounds for an evening (33%)
  the model: something small (8%), tell them the bed is occupied (29%)
  the model: something it will notice (83%), wear a path from the door (48%)
  the model: something it will notice (83%), open the door (15%)
  the model: something it will notice (84%), open the chest for a moment (47%)
  the model, after opened_chest: they feel uneasy (83%), answer with something small (44%)
  the model: something it will notice (85%), light a path of torches away from the house (84%)
  the model: something it will notice (85%), stand far off where they can see it (51%)
  the model: something it will notice (87%), close the fog in for a minute (33%)
  the model: something it will notice (88%), change one block of the house for its cousin (34%)
17 decisions, 13 landed; want: a place of its own; mood: uneasy; written: []
```

A decision in full, as the log has it:

```
--- stage=1 day=0.07 brain=model
  The player sleeps in a house with 1 door and 2 lights.
  They reach for an iron pickaxe most.
  The chest in the house is opened most, 1 times so far. It holds an iron pickaxe, a cobblestone, an oak sapling.
  It wants a place of its own.
  Now the player is indoors, walking.
tick 6100: Surroundings { id: 2, around: [82, 40, 2], radius: 96 }
tick 10880: OpenLid { id: 3, pos: [2, 64, 0], seconds: 1.5 }
  the model: something it will notice (76%), move things around in the chest (27%)
```

A reaction:

```

```

## What read right

- The pipeline holds: the want, the planning questions, the reactions, the plans landing on
  their moments, the story and the log all agree, and nothing stalled on the worker thread.
- The `What should it do?` answers are plausible for a stage-one and -two house: the chest,
  the door, a knock, dripping water, a flicker, the torches toward its place once it wanted a
  place of its own, the far figure in stage three.
- The reactions read sensibly when confident: `uneasy` at 83 to 85 percent after a chest
  opening and after going to bed; the `by showing itself` and `with something small` answers
  stayed under `brain.react_threshold` (0.55) and did nothing, which is the bar working.

## What read wrong, and what to move

- **`Should it do something soon?` barely discriminates.** Sixteen of seventeen answers were
  `something it will notice` at 73 to 88 percent, whatever the situation; once `nothing yet`
  at 18 percent (which did make a `wait`), once `something small` at 8 percent. The question
  as worded reads as a yes to a 1.5B model. Options for round two: ask it as a yes or no
  (`Does it do anything tonight?`), or drop the question and gate on the second question's
  confidence alone, or make the choices the stage's own (`nothing`, then the small and the bold
  by name).
- **Low-confidence picks still happen.** `dripping water indoors` at 8 percent, `open the
  door` at 15 percent, `the bed is occupied` at 29 percent were planned, since the threshold
  reads the first question only. Round two: apply `brain.event_threshold` to the second
  question too, or to their product, and read the log for whether that makes the lodger too
  patient.
- **Moods at low confidence are noise.** `angry` at 27 percent after going to bed. Keep a mood
  only above a bar (the react threshold would do), else leave the last one.
- **No written line in three days:** no text-bearing event was chosen, so `generate` was
  never exercised here. A run with `sign`, `says_back` and `kill_sign` offered early (a config
  that disables the rest) is the next calibration step for the words.
- **The situation text is thin on stage one.** Most decisions were made on five or six lines;
  the surroundings line never appeared (the scripted stay has no host pass), so the model
  never saw water or a village. The dev client will give it more.

## Next

Round two after the owner has played a dev stay with the real model: read the decision log
end to end, move the first question or gate on the second's confidence, bar the moods, and
exercise the written lines. Keep this file's shape: the run, the numbers, what read right, what
to move.
