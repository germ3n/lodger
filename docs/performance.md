# Performance

What the lodger costs a game, measured where it can be, where the time goes, what was made cheaper,
and whether the host needs to move to Rust for speed (it does not).

## The numbers

The mind, timed by `the_mind_s_time_per_tick` in `core/tests/stays.rs` (ignored by default; run it
with `cargo test --release -p lodger --test stays -- --ignored --nocapture the_mind_s_time_per_tick`).
It plays four days of a stay through the same JSON over the C ABI the game uses, and times every
tick, the FFI and the JSON included:

| Run | Mean | Median | p99 | p99.9 | Worst |
| --- | --- | --- | --- | --- | --- |
| One player, the usual world | 3.3 µs | 3 µs | 5 µs | 14 µs | 442 µs |
| `CROWD=1`, 300 entities in every tick's senses | 117.5 µs | | 138 µs | | 1061 µs |

A Minecraft tick has 50 ms. The mind's share is a ten-thousandth of it on a usual tick and a
quarter of a percent with a crowd. The test fails if p99 reaches 5 ms. The model's scoring runs on
its own thread and never in the tick (see [Brain](brain.md)).

## Where the host's time goes

The host was read through for cost (there is no profiler in the game yet). Nothing it does each
tick is expensive: reading the senses, sending them, and acting on a handful of commands are a few
hundred microseconds at most, nearly all of it the game's own API. The costs that showed were
spikes, not a steady drain:

- **Block scans.** A scan reads up to 40,000 blocks (`SCAN_LIMIT`) in one tick. Each block's id was
  made fresh from the registry, and each wildcard kind (`*_door`) made a new substring per block.
- **Sweeps on one tick.** Six sweeps read every entity of every level once in five seconds (holds,
  deaths, warps, temporaries, fights, gatherings), and all six fell on the same tick, with the
  world saves.
- **Hints in the render.** `Hints.active()` copied its list at every call, and red rain asked for
  its weight for every rain column, a few hundred calls a frame while it rained.
- **Dead redstone.** Its box was walked whole as it began and ended, and had no limit.

## What was made cheaper

- Block ids are made once for each block (`Senses.blockId` keeps them), and wildcards are matched in
  place (`Kinds.wanted`), so a scan makes no strings.
- `Sweeps` gives each sweep its own tick of the five seconds; `SweepsTest` checks that each runs
  once in a hundred ticks and that no two share one.
- `Hints.active()` hands back a list made again only when a hint comes or one runs out, and the rain
  reads red rain's weight once in 30 ms.
- `dead_redstone` refuses a box bigger than a scan may read.

## Does the host need Rust?

Not for speed. The host's time is spent in Minecraft's own calls (reading blocks and entities,
setting blocks, spawning, rendering), which are Java and stay Java whatever calls them. What a port
could move, the decisions and the bookkeeping, is already a small share, and the FFI and JSON
between the two are inside the 3.3 µs above. A port would add FFI calls for every game read without
saving any of them. Moving host logic to Rust is still open in [Todo](todo.md) for other reasons
(one language for the logic, tests without a game), not for speed.

## When to look again

- A tick timing in the debug HUD would show the host's real share in a game; there is none yet.
- If scans grow past `SCAN_LIMIT`, slice them over ticks instead of raising the limit.
- If a new sweep is added, give it a `Sweeps.Sweep` of its own.
