# Memory

What the lodger knows about its target (`core/src/memory.rs`), how it is kept small, and how it
becomes the lines of the situation text the brain reads. The memory is the whole of what makes
one stay different from another, and it is also what a reader of the decision log needs in
order to understand a choice, so it is plain data, plainly described.

## What is kept

| Piece | What | Bound |
| --- | --- | --- |
| **Home** | Where the target sleeps and where they spend most of their time standing still: a bed position, a center and a radius, and the blocks around it counted by kind (doors, lights, walls, containers). Re-estimated daily. | One. |
| **Routes** | Paths the target walked: a point every two blocks while they move, cut where they stand still for a minute, sleep or leave; a trail of ten points or more becomes a route, or is counted into the route it repeats (starting and ending within 8 blocks of it). The route walked twice or more is *the* route (before one is, the longest single walk of 32 blocks or more, so a first day has one); its far end from the home is where things get left. | 16 routes of at most 128 points (two blocks apart: 256 blocks). The least-walked route is dropped first. |
| **Containers** | Every container the target opened or placed: position, its kind (a chest, a barrel, a furnace), how many times, on how many different days, when last, whether they placed it, and what is in it, by item id, count and slot, read on opening and again on closing. Only the target's own are used (`Container::theirs`: placed, or opened on three days); the used container is the one of those opened most; the furnace has its own events. Never an ender chest. | 32 containers, 27 stacks each. The least-opened of those not theirs dropped first. |
| **Doors and lights** | Of the home, from the placed-block event and from the home read (`Memory::read_home`: the house box scanned when the target comes home, half a minute after they build or break near it, and every ten minutes there; inside its box what it does not find is gone), and the daily surroundings pass's read of the house around the bed (`house_door`, `house_light`, `house_chest`, `house_bed`, every block within 8), so a village house or generated torches count. One placed where one is remembered replaces it, and a block broken forgets what stood on it (a door, a torch on the floor), which the game drops without a break event of its own, so a door dug under and put back is not counted twice. The home read also gives each door's way out, the side of it the sky is over (`Memory::door_out`), for the events that stand or put things before the door; a door the host could not tell goes by the side away from the home's middle. | 16 doors, 64 lights, 16 ways out. |
| **Fixtures** | Blocks the target placed that an event can use, by plain kind: fence gates, glass panes, levers, campfires, cauldrons, crafting tables, note blocks, bells, jukeboxes, lecterns, anvils, carpets, signs, trapdoors, plates, buttons, pots, composters. Learned from the placed-block event, forgotten when broken. | 128, oldest dropped. |
| **Decor** | Item frames, paintings and armour stands within the home's radius, with the server's entity ids. Rewritten by every entity pass, since the ids last only while the world is loaded. | 32. |
| **Creatures near** | Animals and villagers within 32 blocks of the target as of the last entity pass: id, kind, position, whether named, whether tamed by the target. | 48. |
| **Tally** | What the target has hit and killed, by mob kind, most killed first. The kill events (`kill_sign`, `grave`, `kill_line`) read it, and `describe` names the kind they kill most once it reaches three. | 32 kinds, the least-hit dropped first. |
| **Landmarks** | Every event that landed which the target would have noticed (the catalogue's bold rows: a take, a grave, a sign), kept for the whole stay where `done` trims, so what it took and where it left things is never forgotten and the caps per stay hold. | 512, which a stay never reaches. |
| **Milestones** | What happened to the target: where they died, the first night they slept, the first of each kind they killed, a dimension crossed. One line of the situation text is the latest. | 128. |
| **Favourites** | Item kinds ranked by how often they are in the container opened first after login and in the target's main hand when they place or break: the target's tool, their block, their food. | 8 kinds. |
| **Habits** | (`core/src/habits.rs`, the review's N1.) Sampled every hundred ticks: where each part of the day goes (mornings, afternoons, evenings, nights at home, underground, at the field, in the village or away), the time of day they leave home and come back (in within 16 blocks, out past 32, so walking the edge is neither), nights slept and nights stayed up through, the home's door they use most (the knock, `door_scratch` and `path_worn` come to it: `Memory::usual_door`), what they hold or wear on each day they play, the lights they place, and how often they look behind them. Each says a line only once there is enough to say it (a part of the day after a minute and a half of samples, leaving and coming back after three of each, nights after two, carrying after two days). | 16 leaves, returns and sleeps; 32 items carried; 16 doors. |
| **Hours** | The target's sessions: when they joined and left in real time, and the game's time of day when they joined. From these: the usual length in minutes (after two sessions) and whether they join after dark. | 30 sessions. |
| **Reactions** | (Widened, the review's N2.) What the target did in the day after **every** event that landed and does something to them, sorted by `Event::effect` (`sounds`, `words`, `things`, `house`, `seen`, `sky`, `night`), up to eight watched at once, each on its own: `lit` (a light placed within 16 blocks of a thing left in a place, not a sound's spot), `walled` (closed in on all four sides by blocks the target placed, one or two out, level or one above, the last block placed closing it; building near it is not walling it), `watched` (stood a minute facing the spot within 8 blocks), `left` (left the game within a minute), `ignored` (a day passed); and beside the kind: startled (more turns or look-backs within ten seconds), looked (three seconds facing it, or opening the chest it was done to), fixed and after how many minutes (a block placed or broken where a `house` event changed one), ran (in the first minute, 12 blocks farther than the nearest they came, at a run), fought (hit something in the minute after a `seen` one). `Memory::attention` gives each effect's share noticed once three have settled; the rule brain leans each event's weight by it, times 0.6 for an effect never noticed to 1.4 for one always noticed (`rules_brain::choose_with`), and the situation text says it: `Words get their attention; sounds do not.`, `They put things right within about 5 minutes.`, `When something shows itself, they run.` | 128 rows, oldest dropped; 8 open. |
| **Chains** | The chains under way (`chains.rs`): each with the steps landed and the day the last did; those run this stay, and of them those walked to the end. The situation says each open one: `It has begun something: it chose to put out a path of torches yesterday; next it may dig a hole at the path's end.` | Eight, once a stay each. |
| **Figure looks** | How each figure shown them was looked at as it went (the host's `figure_seen`, ABI 47): the time spent facing it within ten degrees, in looks of half a second or more; how many looks; how far they walked toward it from where they first stood; and how it ended (`approached`, `looked_away`, `timed`, `gone`). Two seconds or more is looked at long (`FigureLook::looked_long`); eight blocks toward it, or near enough that it went, is chased (`chased`). The situation says the last within three days: `Today they looked at the figure for 3 seconds and went 12 blocks toward it.` | 16, oldest dropped. |
| **Stores** | What it dug and holds, by block id and count, from the host's reports of what a dig replaced (grass dug is dirt): the only blocks and items it gives without taking them first, spent as it gives. | By kind. |
| **Scanned** | The village's doors, bell and lanterns (read every five minutes while the target is within 64 blocks of it) and the ore around the target (`scan_ore`), each list from its own scan. | 64 each. |
| **Holes and plants** | Natural ground the target dug out within 16 blocks of the home (not what they placed), for `backfill`; the flowers, pumpkins and saplings the home read finds. | 32 each. |
| **Held** | The item the lodger holds, if any, where it took it from and when. | One. |
| **Done** | Every event landed or refused: id, place, time, `done` or why not. The brain uses it to not repeat. | 256 rows, oldest dropped. |
| **Taken up** | The day it first took their things up into the sky (`taken_up`) while any are up there: the host keeps the stacks under `taken_up_<n>_<i>`, the memory only that some are up and since when. Cleared when they fall back (`sky_gives_back`), as an ending begins, and on a reset. | One day. |
| **Want** | Which want (`a place of its own`, `to be looked at`, `one thing of theirs`), the model's pick at move-in or the seed's; a line of the situation text. Its place is chosen by the home once it has moved in (8 to 12 blocks out, at the floor's height) and chosen again when the home moves more than 32 blocks from it. Whether it is met is `ending::want_met`, read when the ending is chosen. | One. |
| **Mood** | The lodger's reading of how the target feels, from the model's `How does the player feel?` after a trigger: `calm`, `curious`, `uneasy`, `afraid`, `angry`. A line of the situation text. | One. |
| **Walls** | The building blocks the target placed with their ids (planks, logs, stone, bricks, glass, torches, lanterns, wool and the rest with a family), for the events that swap one for another of its family. | 512, oldest dropped. |
| **Temper** | 0 to 1, how provoked it is, moved by the target's acts as [Provocation and temper](entity.md#provocation-and-temper) lists them, drifting back toward the preset's resting value a tenth a day; and the last eight things that moved it, of which the situation text names up to two. | One, plus 8 causes. |
| **Others** | Every player seen, by index, name and uuid, for the target ask in multiplayer. Hours and homes per player are the todo's. | 16 players. |
| **Surroundings** | What is around the home, from the host's pass once an in-game day within 96 blocks, and again when the home moves: water and how wide, snow, sand, trees, crops, a village, a mineshaft, each with where it is. `Need::Around(kind)` reads it; `describe` names up to three. Pens, caves and the target's tunnels are the todo's. | One, rewritten daily; the host caps a pass at 160 blocks. |
| **Vehicles** | Boats and minecarts within 64 blocks of the home, from the entity pass, for the events that move them. | 8. |
| **People** | (`core/src/people.rs`, the review's N4.) Other players seen within 24 blocks of the home, by name and on how many days (once a day each), and who is within 24 blocks of the target now; `Memory::alone` reads it. Lines: `Now they are with alex.` (or `Now they are alone.` once other players have been seen at all), `Others come to the house: alex on 3 days and sam once.` | 16 visitors, the least seen dropped first. |
| **Places** | (`core/src/places.rs`, the review's N3.) The house in rooms, named by what stands nearest on the same floor within 5 blocks: the bed, a furnace, a crafting table, a lectern, a jukebox, three chests or more together (the storeroom); above or below the floor is upstairs or under the house (`Memory::room_of`, `Memory::where_words`, which every place in `describe` goes through). Rooms are read from what the memory has, not walls: the home read finds blocks by kind, never air. The sites they stay at more than 48 blocks from home, sampled with the habits: a `mine` (underground), a `base` (40 blocks or more they placed within 12) or a `spot`, said once kept (two and a half minutes there): `Away from home they keep a mine 198 blocks east of the house.` A recent event near an earlier landmark of another event refers back to it: `Today it did a cairn, where it once did a sign.` Distances from the house are along the ground. | 12 sites, the least stayed at dropped first. |
| **Trees** | The trunks around the target, read every twenty seconds while they are outdoors in a wooded biome (a forest, taiga, jungle, grove or swamp): a scan for logs within twenty blocks, each column of three logs or more a trunk at its lowest log, with its log's kind, a column with a log they placed left out as a wall of theirs (`crate::trees`). Six trunks within twenty-four blocks are the woods (`Memory::in_the_woods`), for the forest's events. Not saved. | 2048 logs. |
| **Caves** | (batch 4.) The dark places of the last cave read, air with no light around the target near home, every third block; `Memory::unentered_cave` is the crowd of them they have not been near, for `lantern_ahead` and `cave_lit`. Beside it, the habits keep where they placed lights underground, for their tunnel spacing. | 512 places; 32 lights. |
| **Field** | (batch 6.) The crop blocks of the last field read: every wheat, carrot, potato and beetroot within 12 blocks of the crops the surroundings saw, read every ten minutes while the target is within 64 blocks; `crate::farm` makes rows, the field's box and the word it can be cut into. Beside it, the ids of the animals near tied to a post. | 256 crops. |
| **Tunnels** | (batch 13.) What the target digs underground of natural rock, grouped as it is dug (a dig within three blocks of a tunnel's last block carries it on), each with its blocks, its mouth, end and way, and its floor by column; ten columns or more is a tunnel. The obsidian they placed, for a frame. | 16 tunnels of 256 blocks; 64 obsidian. |
| **Builds** | (batch 15.) What it builds, each a blueprint laid a night's share at a time: its kind, its site (the ground's height once the host has found it), the way it faces, how many parts are laid, whether its house has grown its room. Where they last stood fishing, for the dock. | One of each kind. |
| **Elsewhere** | (batch 16.) The day the target went into another dimension (gone from the world with no word of leaving the game), and on their return how long they were gone, for `busy_away`; whether they crouch still now, for `breath`; a door they just knocked on, for `answer_knock`. The lights it placed itself and the world's difficulty, for the endings. The other dimensions' stays keep a memory of their own for the moment (who is near, what hostile mobs), and what they do is recorded here (`crate::elsewhere`). | One each. |
| **The budget** | `describe` gives the core lines, then the detail lines (people, how they take each kind of event, places, habits) only while the lines stay under `memory::describe_chars(model.context)`: 3.5 characters a token with 200 tokens kept for the frame, the question and its longest choice. A prompt past the context fails and the question falls to the rules. Then the `Now the player is` line, always. | 4,676 characters at 1536 tokens. |
| **Activity** | The target as last seen (`Snapshot`): where, facing, online, asleep, indoors, walking, underground, crouched still, sneaking, what they hold, whether it rains, the animals, villagers and hostile mobs near, and another horror mod's mob near. The `Now the player is` line reads it: away from the game, asleep, indoors or outdoors, walking. | One. |
| **Movement** | How the target moves: turns of more than 120 degrees in the last minute and looks behind (a turn and back), on the snapshot; the reactions read them for startled, the habits for how often they look behind, and the `Now the player is` line says `keeps looking behind them`. Pace, sprints, standing still and hugging walls are not kept. | Two counts. |

Positions are block positions. Times are in-game ticks since the world began and, for hours, the
real wall clock the server reports, which the body passes in `now_json` each tick, so the mind
never reads a clock itself and a test can set the time.

## Bounds

Every list has a cap, every cap is a constant in `memory.rs` (or the module beside it that keeps
the list: `habits.rs`, `places.rs`, `tunnels.rs` and the rest) with its reason beside it, and
the tests push lists past their caps and check they hold. The whole memory goes into `lodger.dat` on every world save, so it must stay the size of a large
NBT compound, not a database. Routes are the largest piece and the reason for the downsampling.

Forgetting is by the caps alone. Nothing decays with time, because a lodger that remembers your
first week in your third is the design. The landmarks and the milestones are the pieces that
hold the rest to that (owner: "certain actions are always remembered, where it makes
sense"): `done` is a ring of the last 256 outcomes for the rules, the landmarks are the ones
that mattered, for good.

## Serialization

The mind serializes the stay, memory included, as JSON (`serde_json`) behind a one-byte format
version (3; 1 and 2 were `bincode` from development builds and only their
counts are read), and the body stores the bytes as an NBT byte array in the world's saved data
(`<world>/dimensions/<ns>/<dim>/data/craftkit/lodger.dat`, `memory`; `host/.../SavedMemory.java`). Not
as a string: an NBT string holds 65535 bytes at most, which a memory passes within an hour, and
the game then saved it empty. An older save's base64 string (`bytes`) is still read. JSON because every struct carries
`#[serde(default)]`: a field added to the memory reads an older save with its default and
costs no migration, which `bincode`'s positional layout could not give. A newer mind reads
older formats and writes the newest; an older mind refuses a newer format, logs it, and runs
a fresh stay without overwriting the bytes, so downgrading loses nothing but the lodger until
the mod is upgraded again. There is no NBT in the mind, so the memory's layout is the mind's
business and the body never has to change for it.

## Describing

`Memory::describe` writes the memory as short lines of plain English for the situation text
(see [The situation text](brain.md#the-situation-text)), within the budget above. It is a
template, not prose generation: each line is a fixed sentence with the numbers and names filled
in, so the model reads the same shapes every time and its scoring stays comparable across stays.
The one line that carries what a player typed is their last chat line about the lodger or its
want, quoted (`The player said: "..."`). The target's name is reduced to `the player`; item ids
to their path with underscores as spaces (`an iron pickaxe`); positions to directions and
distances from the home (`40 blocks north of the house`).

The lines, in order, each left out when there is nothing to say:

1. The home: `The player sleeps in a house with 2 doors and 9 lights.`
2. The favourite: `They reach for an iron pickaxe most.`
3. The most used container: `The chest in the house is opened most, 14 times so far. It holds ...`
4. The last three events: `Yesterday it did open the door.`, and the latest landmark before
   them: `3 days ago it did put up a sign with one word, in the house.`
5. The open chains, and the temper when it is far from resting, with why.
6. The last figure within three days: `Today they looked at the figure for 3 seconds and went 12
   blocks toward it.`
7. The latest milestone: `Yesterday they died, 40 blocks north of the house.`
8. The route: `They walk a way of 60 blocks to 60 blocks east of the house, 4 times so far.`
9. The hours: `They usually play for about 50 minutes, and join after dark.`
10. The last reaction: `After it did move a torch, they lit the place up.`
11. What is around: `Around the house: water 12 blocks across north of the house; a village
    to the west.`
12. The want, the mood, the kind they kill most, the held item, and their last chat line about
    it: `It wants to be looked at.`, `The player seems uneasy.`, `They have killed 7 zombies so
    far.`, `It holds the player's iron pickaxe, taken yesterday.`
13. The detail lines while they fit: the people, how they take each kind of event, the places,
    the habits.
14. Now: `Now the player is indoors, walking, and keeps looking behind them.`, always last.

The brain passes `describe_chars(model.context)` (4,676 characters at the default 1536 tokens),
and only the detail lines are dropped to fit; the core lines and the last are always there.
