# Memory

What the lodger knows about its target (`core/src/memory/`), how it is kept small, and how it
becomes the lines of the situation text the brain reads. The memory is the whole of what makes
one stay different from another, and it is also what a reader of the decision log needs in
order to understand a choice, so it is plain data, plainly described.

## What is kept

| Piece | What | Bound |
| --- | --- | --- |
| **Home** | Where the target sleeps and where they spend most of their time standing still: a bed position, a center and a radius, and the blocks around it counted by kind (doors, lights, walls, containers). Re-estimated daily. | One. |
| **Routes** | Paths the target walked: a point every two blocks while they move, cut where they stand still for a minute, sleep or leave; a trail of ten points or more becomes a route, or is counted into the route it repeats (starting and ending within 8 blocks of it). The route walked twice or more is *the* route (before one is, the longest single walk of 32 blocks or more, so a first day has one); its far end from the home is where things get left. | 16 routes of at most 128 points (two blocks apart: 256 blocks). The least-walked route is dropped first. |
| **Containers** | Every container the target opened or placed: position, its kind (a chest, a barrel, a furnace), how many times, on how many different days, when last, whether they placed it, and what is in it, by item id, count and slot, read on opening and again on closing. Only the target's own are used (`Container::theirs`: placed, or opened on three days); the used container is the one of those opened most; the furnace has its own events. Never an ender chest. | 32 containers, 27 stacks each. The least-opened of those not theirs dropped first. |
| **Doors and lights** | Of the home, from the placed-block event and from the home read (`Memory::read_home`: the house box scanned when the target comes home, half a minute after they build or break near it, and every ten minutes there; inside its box what it does not find is gone), and the daily surroundings pass's read of the house around the bed (`house_door`, `house_light`, `house_chest`, `house_bed`, every block within 8), so a village house or generated torches count. One placed where one is remembered replaces it, and a block broken forgets what stood on it (a door, a torch on the floor), which the game drops without a break event of its own: two doors dug under and put back were four until 2026-11-09. | 16 each. |
| **Fixtures** | Blocks the target placed that an event can use, by plain kind: fence gates, glass panes, levers, campfires, cauldrons, crafting tables, note blocks, bells, jukeboxes, lecterns, anvils, carpets, signs, trapdoors, plates, buttons, pots, composters. Learned from the placed-block event, forgotten when broken. | 128, oldest dropped. |
| **Decor** | Item frames, paintings and armour stands within the home's radius, with the server's entity ids. Rewritten by every entity pass, since the ids last only while the world is loaded. | 32. |
| **Creatures near** | Animals and villagers within 32 blocks of the target as of the last entity pass: id, kind, position, whether named, whether tamed by the target. | 48. |
| **Tally** | What the target has hit and killed, by mob kind, most killed first. The kill events (`kill_sign`, `grave`, `kill_line`) read it, and `describe` names the kind they kill most once it reaches three. | 32 kinds, the least-hit dropped first. |
| **Landmarks** | Every event that landed which the target would have noticed (the catalogue's bold rows: a take, a grave, a sign), kept for the whole stay where `done` trims, so what it took and where it left things is never forgotten and the caps per stay hold. | 512, which a stay never reaches. |
| **Milestones** | What happened to the target: where they died, the first night they slept, the first of each kind they killed, a dimension crossed. One line of the situation text is the latest. | 128. |
| **Favourites** | Item kinds ranked by how often they are in the container opened first after login and in the target's main hand when they place or break: the target's tool, their block, their food. | 8 kinds. |
| **Hours** | The target's sessions: when they joined and left in real time, and the game's time of day when they joined. From these: the usual length in minutes (after two sessions) and whether they join after dark. | 30 sessions. |
| **Reactions** | What the target did in the day after each bold event landed: `lit` (a light placed within 16 blocks), `walled` (closed in on all four sides by blocks the target placed, one or two out, level or one above, the last block placed closing it; building near it is not walling it), `watched` (stood a minute facing the spot within 8 blocks), `left` (left the game within a minute), `ignored` (a day passed). One row per landmark. Chasing a figure is phase 5's. | 128 rows, oldest dropped. |
| **Stores** | What it dug and holds, by block id and count, from the host's reports of what a dig replaced (grass dug is dirt): the only blocks and items it gives without taking them first, spent as it gives. | By kind. |
| **Scanned** | The village's doors, bell and lanterns (read every five minutes while the target is within 64 blocks of it) and the ore around the target (`scan_ore`), each list from its own scan. | 64 each. |
| **Held** | The item the lodger holds, if any, where it took it from and when. | One. |
| **Done** | Every event landed or refused: id, place, time, `done` or why not. The brain uses it to not repeat. | 256 rows, oldest dropped. |
| **Want** | Which want (`a place of its own`, `to be looked at`, `one thing of theirs`), the model's pick at move-in or the seed's; a line of the situation text. The score for how far it is met is the todo's. | One. |
| **Mood** | The lodger's reading of how the target feels, from the model's `How does the player feel?` after a trigger: `calm`, `curious`, `uneasy`, `afraid`, `angry`. A line of the situation text. | One. |
| **Walls** | The building blocks the target placed with their ids (planks, logs, stone, bricks, glass, torches, lanterns, wool and the rest with a family), for the events that swap one for another of its family. | 512, oldest dropped. |
| **Temper** | 0 to 1, how provoked it is, moved by the target's acts as [Provocation and temper](entity.md#provocation-and-temper) lists them, drifting back toward the preset's resting value a tenth a day; and the last three things that moved it, for the situation text. | One, plus 3 causes. |
| **Chains** | The open chains (see [Chains](entity.md#chains)): which chain, which step landed last and when, the place or item it carries, and the steps still ahead. | 8 open chains. |
| **Figures seen** | Each time the target looked at a figure: for how long, from how far, whether they moved toward it. | 32 rows. |
| **Others** | Every player seen, by index, name and uuid, for the target ask in multiplayer. Hours and homes per player are the todo's. | 16 players. |
| **Surroundings** | What is around the home, from the host's pass once an in-game day within 96 blocks, and again when the home moves: water and how wide, snow, sand, trees, crops, a village, a mineshaft, each with where it is. `Need::Around(kind)` reads it; `describe` names up to three. Pens, caves and the target's tunnels are the todo's. | One, rewritten daily; the host caps a pass at 160 blocks. |
| **Vehicles** | Boats and minecarts within 64 blocks of the home, from the entity pass, for the events that move them. | 8. |
| **Scanned** | What the village scan found (its doors, its bell), by plain kind and position, so `village_doors` knows what to open. | 64. |
| **Activity** | What the target is doing now, from the last minute: at home, away, asleep, mining, farming, building, fishing, travelling, in a boat; and the tunnel they are digging, if any, with its direction. | One. |
| **Movement** | How the target moves, from the last five minutes and as a running average over the stay: pace, stops per minute, turns of more than 120 degrees per minute, looks behind (a turn and back within two seconds), backing away (moving opposite to facing), sprint bursts and what they sprinted from, time standing still and what they faced, sneaking indoors, the path hugging walls or not, how fast they left the last event's room. Compared to the stay's average, so "more than usual" is a line the model can read. | One, plus 7 daily averages. |

Positions are block positions. Times are in-game ticks since the world began and, for hours, the
real wall clock the server reports, which the body passes in `now_json` each tick, so the mind
never reads a clock itself and a test can set the time.

## Bounds

Every list has a cap, every cap is a constant in `memory/mod.rs` with its reason beside it, and
`cargo test` checks that a million observations leave the memory under 256 KiB serialized. The
whole memory goes into `lodger.dat` on every world save, so it must stay the size of a large
NBT compound, not a database. Routes are the largest piece and the reason for the downsampling.

Forgetting is by the caps alone. Nothing decays with time, because a lodger that remembers your
first week in your third is the design. The landmarks and the milestones are the pieces that
hold the rest to that (owner, 2026-11-09: "certain actions are always remembered, where it makes
sense"): `done` is a ring of the last 256 outcomes for the rules, the landmarks are the ones
that mattered, for good.

## Serialization

The mind serializes the stay, memory included, as JSON (`serde_json`) behind a one-byte format
version (3 since 2026-11-09; 1 and 2 were `bincode` from development builds and only their
counts are read), and the body stores the bytes as base64 in the world's saved data
(`<world>/dimensions/<ns>/<dim>/data/craftkit/lodger.dat`). JSON because every struct carries
`#[serde(default)]`: a field added to the memory reads an older save with its default and
costs no migration, which `bincode`'s positional layout could not give. A newer mind reads
older formats and writes the newest; an older mind refuses a newer format, logs it, and runs
a fresh stay without overwriting the bytes, so downgrading loses nothing but the lodger until
the mod is upgraded again. There is no NBT in the mind, so the memory's layout is the mind's
business and the body never has to change for it.

## Describing

`Memory::describe` writes the memory as at most twelve short lines of plain English for the
situation text (see [The situation text](brain.md#the-situation-text)). It is a template, not
prose generation: each line is a fixed sentence with the numbers and names filled in, so the
model reads the same shapes every time and its scoring stays comparable across stays, and so a
line can never carry anything a player typed. Player names are reduced to `the player`; item ids
to their path with underscores as spaces (`iron pickaxe`); positions to directions and
distances from the home (`40 blocks north of the house`).

The lines, in order, each left out when there is nothing to say:

1. The stage and the day within it.
2. The want and how met it is: `It wants a place. The player has not built there.`
3. The home: `The player sleeps in a small house with two doors and many lights.`
4. The favourites: `They reach for an iron pickaxe first most days.`
5. The most used container: `The chest by the door is opened first most days. It holds ...`
6. The hours: `They usually play in the evening for about an hour, and do not play at night.`
7. The last three events and their reactions: `Two days ago a torch was moved. The player lit
   the room.`, and the open chains: `It took the pickaxe three days ago and has not given it
   back.`, and the temper: `It is angry: the player broke its torches twice.`
8. The held item: `It holds the player's iron pickaxe, taken from the chest by the door.`
9. The figure: `The player has seen the figure twice and walked toward it once.`
10. The routes: `The player walks the same way to the mine most days, 60 blocks east.`
11. What the player is doing now, how they move, and what is around: `Now the player is digging
    a tunnel east, 40 blocks from the house. They keep looking behind them, more than usual.
    There is a lake north of the house and a village to the west.`
12. The time: `It is evening on the player's fifth day.`

Twelve lines are about 250 tokens, which leaves room in a 512-token context for the question
and the longest choice with margin. The brain measures and drops lines from the bottom of the
list, never the top, when the measured count would not fit.
