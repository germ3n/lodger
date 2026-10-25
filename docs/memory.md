# Memory

What the lodger knows about its target (`core/src/memory/`), how it is kept small, and how it
becomes the lines of the situation text the brain reads. The memory is the whole of what makes
one stay different from another, and it is also what a reader of the decision log needs in
order to understand a choice, so it is plain data, plainly described.

## What is kept

| Piece | What | Bound |
| --- | --- | --- |
| **Home** | Where the target sleeps and where they spend most of their time standing still: a bed position, a center and a radius, and the blocks around it counted by kind (doors, lights, walls, containers). Re-estimated daily. | One. |
| **Routes** | Paths the target walked, as position lists at one point a second, downsampled to a point every two blocks, cut where the target stands still for a minute. Kept are the ones walked most alike more than once: the way home, the way to the mine, the way to the field. | 64 routes of at most 256 points. The least-repeated route is dropped first. |
| **Containers** | Every container the target opened: position, how many times, when last, and what was in it the last three times, by item id and count. | 32 containers, 27 stacks each. Least-opened dropped first. |
| **Favourites** | Item kinds ranked by how often they are in the container opened first after login and in the target's main hand when they place or break: the target's tool, their block, their food. | 8 kinds. |
| **Hours** | When the target logs in and out, in real and game time, the last 30 sessions, and how long they idle. From these: the usual hour, the usual length, whether they play at night. | 30 sessions. |
| **Reactions** | What the target did after each event landed: lit the area (lights placed within 16 blocks within a day), walled (solid blocks placed around the spot), watched (stood facing the spot for a minute), chased (moved toward a figure), ignored, returned the item, left the world within a minute. One row per event. | 128 rows, oldest dropped. |
| **Held** | The item the lodger holds, if any, where it took it from and when. | One. |
| **Done** | Every event landed or refused: id, place, time, `done` or why not. The brain uses it to not repeat. | 256 rows, oldest dropped. |
| **Want** | Which want, and a score from 0 to 1 for how far the target has met it. | One. |
| **Figures seen** | Each time the target looked at a figure: for how long, from how far, whether they moved toward it. | 32 rows. |
| **Others** | For multiplayer: per player, hours played, whether they have a home, whether they play alone. Used only to pick a target. | 16 players. |
| **Surroundings** | What is around the home and the routes, from a scan the host does once an in-game day within 128 blocks: the nearest water and how wide, snow or sand, a village and its bell, fields, pens and what is in them, a mine (the target's tunnels, their size and torch spacing) and caves not entered, and the house's own furniture and blocks by kind (doors, windows, carpets, frames, paintings, stands, lecterns, jukeboxes, levers, pistons, campfires, candles, ladders, portals). This is what [What is offered when](entity.md#what-is-offered-when) filters on. | One, rewritten daily; at most 64 points of interest. |
| **Activity** | What the target is doing now, from the last minute: at home, away, asleep, mining, farming, building, fishing, travelling, in a boat; and the tunnel they are digging, if any, with its direction. | One. |

Positions are block positions. Times are in-game ticks since the world began and, for hours, the
real wall clock the server reports, which the body passes in `now_json` each tick, so the mind
never reads a clock itself and a test can set the time.

## Bounds

Every list has a cap, every cap is a constant in `memory/mod.rs` with its reason beside it, and
`cargo test` checks that a million observations leave the memory under 256 KiB serialized. The
whole memory goes into `lodger.dat` on every world save, so it must stay the size of a large
NBT compound, not a database. Routes are the largest piece and the reason for the downsampling.

Forgetting is by the caps alone. Nothing decays with time, because a lodger that remembers your
first week in your third is the design.

## Serialization

The mind serializes the stay, memory included, with `bincode` behind a one-byte format version,
and the body stores the bytes as a byte array in `lodger.dat`. A newer mind reads older formats
and writes the newest; an older mind refuses a newer format, logs it, and runs a fresh stay
without overwriting the bytes, so downgrading loses nothing but the lodger until the mod is
upgraded again. There is no NBT in the mind, so the memory's layout is the mind's business and
the body never has to change for it.

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
7. The last three events and their reactions: `Two days ago a torch was moved. The player lit the
   room.`
8. The held item: `It holds the player's iron pickaxe, taken from the chest by the door.`
9. The figure: `The player has seen the figure twice and walked toward it once.`
10. The routes: `The player walks the same way to the mine most days, 60 blocks east.`
11. What the player is doing now, and what is around: `Now the player is digging a tunnel east,
    40 blocks from the house. There is a lake north of the house and a village to the west.`
12. The time: `It is evening on the player's fifth day.`

Twelve lines are about 250 tokens, which leaves room in a 512-token context for the question
and the longest choice with margin. The brain measures and drops lines from the bottom of the
list, never the top, when the measured count would not fit.
