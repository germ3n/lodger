# The lodger

What the entity is, why it is built against the grain of the usual stalker mods, how its stay is
staged, what it can sense and do, and how it ends. The mind that decides all of this is in
[Brain](brain.md); this page is what there is to decide about.

## What it is

The lodger moves into a world that has been played for a few days and attaches itself to one
player. It is not a creature in the woods; it is a presence in the house. Most of its stay it has
no body at all. It shows itself as consequences: a thing of yours gone and found again somewhere
that means something, a torch you did not place, a door open that you closed, your own path
across the snow leading somewhere you did not go. When it does take a shape it is yours: a figure
in your skin, far away, walking the route you walked yesterday, which the mind recorded and
replays.

It learns. What it does is shaped by what you do, judged by a small model reading a plain account
of your habits (see [Brain](brain.md)), so two players' stays diverge from the first day. And it
is finite. The stay has five stages and an end, and when it is over it is over for that world.

## Why not another thing in the fog

The usual stalker mod (From the Fog, Cave Dweller, The Man from the Fog, and their kin) is: a tall
dark shape appears at the edge of render distance at night, stares, vanishes when looked at, later
chases, screams, and jumpscares; it breaks torches and leaves the lights off. Each of those is a
thing the lodger will not do, on purpose:

| The usual | The lodger | Why |
| --- | --- | --- |
| Appears in the dark, at the edge of sight | Events in daylight and indoors; its one far figure is at distance in plain light | Darkness is where you expect it. A moved chest at noon is worse. |
| Chases | Never moves toward you. Everything happens behind you or while you are away | A chase ends; a presence does not. Speed is the opposite of dread. |
| Jumpscares and screams | No sound louder than a door. Your own footsteps a half-second late, a knock, a bell | A jumpscare is a release. Nothing is released here. |
| Things you can prove | Glitches only your client shows: the lights fail, the sun blinks out, the rain runs red, and the world has not changed | What cannot be shown to anyone else cannot be argued away either. |
| Breaks torches | Adds torches, and signs with one word | Taking the light is a monster's move. Lighting your way somewhere you did not mean to go is a host's. |
| Is a stranger | Is you: your skin, your routes, your name at a distance | The one shape that cannot be rationalized away. |
| Loops forever | Five stages and an end, different by what you did | Stakes need an ending. Also a mod you can finish is a mod you recommend. |
| Is the same for everyone | Reads *your* habits and chooses by them | The point of the model. |
| Hates you | Wants something. What, the player works out | A want can be met. Several endings turn on whether it is. |
| Is a monster | Does what you do: mines where you mine, lights the way you light, carries your tools | A thing that copies you is a thing that has been watching. |

## The five stages

Each stage lasts at least `stages.least_days` and at most `stages.most_days` in-game days
(`arc.rs`), ends sooner when its own condition is met, and offers the brain only its own events. A
stage never goes back. The lodger moves in `stages.start_after_days` in-game days after the
world is first played, so a new world is a new world first.

| # | Stage | What the player experiences | Ends when |
| --- | --- | --- | --- |
| 1 | **Noticing** | Nothing, almost. One door open. A torch missing from a dark corner and found two blocks away. The chest's contents in different slots. A chest's sound from the next room. The music does not play one evening. | The memory holds a home, a chest the target uses most and one route, and at least two events landed. |
| 2 | **Residue** | Things are not where they were. One item gone from the chest you open first, found later on your bed, your crafting table, or at the end of your route. A torch you did not place. Footstep sounds after you stop. A sign with one word: `here`, `again`, `yours`. | The target has reacted (lit the house, walled a room, stood watching, came back for the item) and the brain has judged the reaction. |
| 3 | **Habits** | It knows your day. The thing you reach for first is the thing that moves. `You may not rest now` with nothing in sight, on the nights you are tired. Your bed is one block over from where you left it. Crops trampled in a line toward your door. The animals stop and look at you. The lights go out over your head for ten seconds and come back. The sun is gone for a breath at noon. | The target has seen the far figure at least once, or the stage's most days have passed. |
| 4 | **Company** | The figure. Far off, in your skin, with your name over it at the limit of nameplate distance, walking a route you walked, at the hour you walked it. Looked at, it keeps walking; approached, it is gone around the next corner, and in the place it stood is the thing it took. Indoors, a chest opens as you enter the room. | The want is met, refused, or the stage's most days pass. See [What it wants](#what-it-wants). |
| 5 | **Settlement** | The ending, over one to three days. See [How it ends](#how-it-ends). | It is over. |

The stages are gates, not scripts. Within a stage the brain decides what, when and where, and
`wait` is always a choice, so a stage can pass with almost nothing in it if the model judges the
target is not ready.

## What it wants

Every stay has one want, picked by the brain at the start from a short list, and the memory
tracks whether it is being met. The want is never stated; it is shown by what is taken and where
it is put. The player who reads it right can change the ending.

| Want | How it shows | What meets it |
| --- | --- | --- |
| **A place** | Items taken end up in one spot, always the same, somewhere in or by the house. Torches lead to it. A sign there says `here`. | The target builds something there: a room, a bed, a chest left with something in it. |
| **To be seen** | The figure appears in the open, at hours the memory says the target is about, and looks back. Things taken are put where the target will look. Signs say `again`. | The target looks at the figure for a few seconds, several times, without approaching. |
| **The thing** | One item, the one the memory says the target uses most, keeps going. Each time it is found further away. Signs say `yours`. | The target leaves that item in a chest the lodger has used and does not take it back. |

The brain is told, in the situation text, which want is in force and how the memory scores it,
and chooses events that serve it. The rule brain does the same with fixed weights.

## Senses

What the body watches and reports, and the mind keeps (see [Memory](memory.md)). Nothing here
is more than the server knows anyway; the lodger reads no client state.

- **Where the target is and faces,** once a second, downsampled into routes.
- **How the target moves** (owner, 2026-11-09): pace and stops, how often they turn around,
  how often they look behind them, whether they back away from a spot or sprint from it, how
  long they stand still facing one thing, whether they sneak indoors, whether they walk the
  middle of a room or its walls, how fast they leave a room an event happened in. Kept as the
  memory's `Movement` and read by the model as plain lines: `The player keeps looking behind
  them on the way home`, `The player left the chest room at a run`. This is how the lodger
  reads fear without being told.
- **Where the target sleeps,** and whether sleep was refused.
- **Which containers the target opens,** how often, and what is in them at the time, by item id
  and count. Chests, barrels, shulkers. Not the inventory, not the ender chest.
- **What the target places and breaks,** counted by kind around the home: lights, walls, doors.
- **Whether the target looked at a figure:** facing within 10 degrees of it for more than half a
  second, and how long, and whether they moved toward it.
- **The target's hours:** when they log in and out, in real time and in game time, and how long
  they idle.
- **Dimension changes, deaths, respawns.**
- **What the target types in chat,** and what witnesses say within 32 blocks of them (owner,
  2026-11-09). The memory keeps the last few lines about the lodger (lines that mention a
  door, a chest, a torch, the house, the figure, a question) and the situation text quotes one:
  `Yesterday the player said: "something keeps opening my chest"`. It is the server's own chat;
  a server operator should know the mod reads it, and the memory keeps at most eight lines.

Nothing is read from the client or from other mods.

## Events

Everything the lodger does is an event: one small change to the world, or one sound, or one
thing only the target's client shows, chosen by the brain from a catalogue. Every event is data
in `core/src/events.rs`: an id, the stages that offer it, what it needs from the memory and the
surroundings, its cooldown and its cap per stay, and the command it becomes for the host. The
brain is only ever offered events whose needs are met, so a choice is always possible to carry
out when chosen, and the host checks again when the time comes.

The catalogue is long on purpose. A stay is about two in-game weeks, events are spaced out (see
[How often](#how-often)), and the thing that makes a haunting wear thin is the third time the
same door opens. A hundred and some small things, each once or twice, each at a time and place
the model picked for *this* player, do not wear thin. Adding one is a row here, a row in
`events.rs`, an executor in the host if it needs a new command, and a game test (see
[Conventions](development.md#conventions)). [Ideas](ideas.md) is the well: several hundred more,
grouped the same way, to draw from when a stage or a situation feels thin.

### What is offered when

The model never sees the whole catalogue. Before each plan, `brain/offer.rs` narrows it:

1. **The stage.** Each event names the stages that offer it (the numbers below); everything in a
   stage's list stays offered later unless its row says otherwise, so the pool widens as the
   stay goes on. Stage one is a handful of things that could be nothing; stage four is all of it.
2. **What is around.** The memory's surroundings (see [Memory](memory.md#what-is-kept)): water,
   snow or sand, a village, a field, a pen, a mine, a cave, the house's blocks and furniture.
   No pen, no `open_gate`; no water, nothing from [Water](#water-and-boats).
3. **What the target is doing now,** from the last minute of observations: at home, away,
   asleep, mining, farming, building, fishing, travelling, in a boat. `mining_sounds` is offered
   to a miner, `late_steps` to a walker, `open_door` to someone away or asleep.
4. **Time and weather.** Night for the sounds and the figures' first showings, day for the sun,
   rain for the red rain, snowfall for the trail.
5. **What happened lately.** Cooldowns and caps (below), never the same event twice in a row,
   and never the same place twice in a cooldown.
6. **A draw.** From what is left, `events.offered` (eight) are drawn with a weight that favours
   what has not happened in this stay, and the model chooses among those, or `wait`.

So the model's question is always "which of these eight, here, now, for this player", which is
a question a small model can answer. The rule brain answers the same question from weights.

### Containers

Chests, barrels and shulker boxes the target opens. Never the ender chest, never the inventory,
never a witness's container.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `open_chest` | 1 | A chest in the home opens, lid and sound, while the target is in another room or facing away, and closes before they can see it. Nothing inside changes. | A used container out of view. |
| `shuffle_chest` | 1 | The items in the chest the target uses most are in different slots. Same items, same counts. | A used container. |
| `stack_split` | 1 | One stack is two stacks, in two slots, the same total. | A used container with a stack of 2 or more. |
| `tidy_chest` | 2 | The chest is sorted by kind, which the target never did. | A used container of 10 or more stacks. |
| `tidy_by_colour` | 3 | Wool, concrete, glass or beds in a chest are sorted by colour, the way a child would. | A container with 6 or more coloured stacks. |
| `swap_stacks` | 2 | Two stacks trade places between two of the target's chests. | Two used containers. |
| `fill_gap` | 2 | One item the target has many of is in a chest it was never in, in a slot that was empty. | Two used containers. |
| `chest_full_of_one` | 3 | Every empty slot of the chest they use holds a single dirt, as many as it has dug (its holes), never one more. | A chest of theirs, three dirt or more dug. |
| `renamed_item` | 3 | One of their things in the chest they use has a name on it, as an anvil writes one: `mine`, `borrowed`, `not yours`; the model writes it. Their favourite, if it is there. | A used container with something in it. |
| `returned_used` | 3 | What it took comes back to its chest the very same item, but a tool worn a little more (never broken by it), anything else with a name on it. | Something held. |
| `loot_gift` | 2 | Opening a chest of fresh loot for the first time (a dungeon's, a village's, a temple's), one in five holds something of the lodger's among the loot, or in place of one loot item (that one kept, not lost): the thing it took from them, something it dug, now and then from stage four their own head, else a redstone lamp or a candle. Loot is the world's, so this is the one exception to "it never spawns anything" (owner, 2026-11-09). Armed by the brain, given by the host as the loot is made. | Fresh loot, the target opening it. |
| `chest_answers` | 2 | As the target opens a chest of theirs, another chest of the house closes, or opens and closes, at the same moment. Three times in ten, never from the menu. | Two chests of theirs. |
| `take` | 2 | One item of the kind the memory says the target uses most is taken from the container they open most. The lodger holds it. | A used container with a favourite in it, out of view. |
| `return_taken` | 2 | The held item is back where it was taken from, in its slot, or one slot over. | A held item. |
| `place_taken` | 2 | The held item is placed where the want says: on the bed, on the table, at the end of the route, at its place, as a dropped item that does not despawn or into a container there. | A held item. |
| `tool_used` | 3 | The favourite tool in the chest has lost durability overnight, a few dozen uses' worth, as if someone used it. | A damageable favourite in a chest. |
| `book_word` | 3 | The book and quill in the chest they use is a written book: one word on its page, the same word its title, signed by no one (the model writes the word). | A book and quill in a chest of theirs. |
| `map_marked` | 3 | The map in the chest they use shows a red X at the lodger's place, or at the house itself when it has none yet. | A filled map in a chest of theirs. |
| `hopper_emptied` | 3 | What waited in their hopper is in the chest of theirs beside or under it. | A hopper they placed, next to a chest of theirs. |
| `shulker_moved` | 3 | Their shulker box is three to five blocks across the room, on the floor, with everything still in it. | A shulker box they placed. |
| `open_on_enter` | 4 | The chest the target uses most opens as they enter its room, and stays open until they look away. | A used container in a room with a door. |

### Furnaces and workstations

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `cook_one` | 1 | One raw item from the target's chest is in a furnace with one fuel, and comes out cooked. | A furnace, a chest with something smeltable and a fuel. |
| `take_output` | 2 | The finished output of a furnace is gone from the furnace and in the chest the target uses. | A furnace with output. |
| `swap_fuel` | 2 | The fuel in a furnace is a different fuel kind from the target's chests, the counts kept fair in burn time (three coal for sixteen planks). Coded 2026-11-10. | A furnace with fuel, a chest with another fuel. |
| `cold_furnace` | 3 | A furnace mid-smelt has lost its fuel, which is in the chest. The smelting stops. | A lit furnace. |
| `item_on_table` | 2 | An item is laid on the crafting table as a dropped item that does not despawn: the held item, or one of the favourites from a chest. | A crafting table, a held item or a chest. |
| `lectern_page` | 2 | The book on a lectern is open to another page (`turn_page`, ABI 29). Coded 2026-11-10. | A lectern with a book. |
| `anvil_sound` | 2 | An anvil's sound from the workshop, at night, while the target is in bed. | An anvil and the target asleep. |
| `brewing_done` | 3 | A brewing stand that had potions brews one of their ingredients into them, the game's own brewing: one ingredient from their chest into the stand, and a blaze powder too when the stand has no fuel (`brew`, ABI 29: refused unless the stand holds potions and its ingredient slot is empty). Coded 2026-11-10. | A brewing stand and an ingredient in the chest they use. |
| `armor_stand_turn` | 3 | An armour stand's head is turned toward the door, or toward where the target sleeps. | An armour stand in the home. |
| `armor_stand_dress` | 3 | One piece of armour from a chest is on the stand. | An armour stand and a chest with armour. |
| `stand_watch` | 3 | The armour stands in the room turn their heads to follow the target while the target is in it, and hold the pose once the target looks straight at one. Back to normal when the target leaves. | An armour stand in the home. |
| `grindstone_strip` | 4 | An enchanted tool in a chest has lost one enchantment, as a grindstone would take it (`strip`, ABI 29: the host picks the item and the enchantment, never a curse). Coded 2026-11-10. | A grindstone and an enchanted item in a chest. |
| `crafted_for_you` | 3 | Something they make lies finished on their crafting table, the makings gone from their chest: four torches from a stick and a coal, bread from three wheat, planks from a log, sticks from planks. Owner's pick, 2026-11-10. | A crafting table, the makings in their chest. |

### The house

Blocks the target placed, moved within reach, never broken for good.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `open_door` | 1 | A door or trapdoor of the home is opened while the target is away or asleep. | A home with a door. |
| `close_door` | 1 | A door the target left open is closed. | An open door in the home. |
| `furniture_shifted` | 2 | Their crafting table, furnace or barrel is one block over from where it stood, everything still in it; never a chest. | One of those of theirs. |
| `one_room_dark` | 3 | Every torch within five blocks of one of the home's lights is gone in the morning, six at most, and they are in their chest, every one. | Lights in the home, a chest of theirs. |
| `door_held` | 2 | A door they open shuts again a moment later, as if held, one time in four; never from the menu. | The target opening a door. |
| `bed_turned` | 3 | Their bed is turned round where it stands, the head where the foot was. | A bed. |
| `door_hinge` | 3 | A door of the house they are in, theirs first, hangs on the other side of its frame (`set_state`, both halves). | A door. |
| `window_tap` | 2 | Three taps on a window pane from outside, at night. | A window and night. |
| `carpet_shift` | 2 | The carpets in a room are one block over. | 3 or more carpets in a room. |
| `stairs_turned` | 2 | One stair block the target placed faces another way. | A placed stair out of view. |
| `ladder_gap` | 3 | One rung from the middle of the home's tallest ladder is missing, and is set on top of the ladder. | A ladder of four rungs or more. |
| `flowers_potted` | 2 | Up to three flowers around the house are picked and are in the home's empty pots. | Flowers around the home, an empty flower pot. |
| `path_worn` | 3 | A line of dirt path from the door toward the lodger's place, as if walked a hundred times. | Grass between the door and the spot. |
| `cairn` | 3 | A small stack of cobblestone at the end of the target's route, from the lodger's own digging. | A route and held stone. |
| `hole` | 3 | A one-block hole three deep at the lodger's place, with a torch at the bottom. | A spot on natural ground. |
| `alcove` | 4 | One block of natural stone behind a wall of the house, away from it, is hollow: a torch in it, and the thing it took from them if it holds one. Only natural stone; the one dig beside what they built (owner, 2026-11-09). | A wall of the house. |
| `backfill` | 3 | Up to five of the holes they dug by the house (natural ground within 16 blocks, newest first) are filled in again with dirt or stone it dug itself, never more than it holds. | A hole of theirs near the home, something dug to fill it. |
| `sapling` | 2 | A sapling from a chest is planted at the spot. | A sapling in a chest and a spot on grass. |
| `tree_grown` | 3 | The sapling by the house is a tree in the morning: bone meal from their chest (three at most), spent only if it grew. | A sapling around the home, bone meal in a chest of theirs. |
| `pumpkin_face` | 3 | A pumpkin by the house is carved where it grows, its face to the nearest door; the shears in their chest are a little worn. | A pumpkin around the home, shears in a chest of theirs. |

### What they kill

The senses report every mob the target hits and kills, and the memory keeps a tally by kind
(see [Memory](memory.md#what-is-kept)). The lodger has opinions about it (owner's idea,
2026-11-09: "signs saying you like to kill, you killed y of z").

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `kill_sign` | 2 | A sign two blocks out from the door: `14 cows`, or `14 so far`. The count is the memory's. | Three or more kills of one kind, a door. |
| `grave` | 3 | Two cobblestone stacked at the lodger's place with a sign beside them naming the kind they kill most: `cow`, or `rest`. | Three or more kills of one kind, a home. |
| `kill_line` | 3 | A chat line in the target's own name about it, at night: `i saw what you did to the cows`, `the cows remember`, `14 cows`. | Three or more kills of one kind, night. |

### The house, changed

Owner's idea, 2026-11-09: "trying to sleep breaks the bed", "replace certain blocks of the
house". The one place the lodger changes what the target built, and only into the same thing
slightly wrong, never gone: a block becomes another member of its family (`memory::family`:
every wood for planks, logs, fences, slabs, stairs and trapdoors; every colour for wool,
concrete, terracotta, carpets and stained glass; the stones; the bricks; the torches; the
lanterns; panes and iron bars), picked by the stay's seed, and the bed comes back as its item.
Nothing with contents, no door, nothing of the ground. (The owner, 2026-11-09: "shouldn't be
hardcoded, why only spruce"; it was a table at first.)

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `wall_swap` | 3 | One block of the house is another of its family: planks of another wood, cobblestone for stone bricks or mossy cobblestone, wool of another colour, glass tinted. Only blocks the target placed and the memory knows the kind of. | A placed building block with a family, the target away or asleep. |
| `torch_blue` | 3 | One of their torches or lanterns is another kind: a soul torch burning blue, a redstone torch's dim red, a soul lantern. | A placed torch or lantern. |
| `window_bars` | 4 | A window pane is iron bars. | A placed glass pane. |
| `bed_breaks` | 4 | As they lie down, the bed breaks under them and drops as its item where it stood. Once a stay. | The target in bed at night. |

### Lights and fire

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `move_light` | 1 | A torch, lantern or candle the target placed is removed and placed within three blocks, the same light kept. | A placed light not in view. |
| `add_torch` | 2 | One torch where there was none, in the darkest corner of the home. The lodger's own torch, which the memory counts. | A dark spot in the home. |
| `light_path` | 2 | Torches, three to seven, from the home toward the want's spot, placed while the target is away. | A home and a spot. |
| `put_out_campfire` | 2 | A campfire is out. | A lit campfire. |
| `light_campfire` | 2 | A campfire is lit. | An unlit campfire. |
| `candles_lit` | 3 | Every candle in the house is lit at night, or every lit one out. | Candles in the home. |
| `cauldron` | 2 | A cauldron is full that was empty, or empty that was full. | A cauldron. |
| `lantern_ahead` | 3 | Far down a cave the target has not entered, one of their lanterns, lit: set down at the cave's dark place farthest from them, then taken from their chest (a lantern that finds no floor stays in the chest). Coded 2026-11-10. | A cave they have not entered (`Need::Cave`) and a lantern in the chest they use. |
| `warped_animal` | 3 | One of the target's animals (tamed, or by the home), or one outside when they keep none near, 12 to 32 blocks from them, goes wrong for a minute: it freezes with its head always on them, and is one or two of too tall (1.8 times), upside down (the game's Dinnerbone flip, never on an animal with a name of its own) and a step closer along the ground each time they look away; one time in three it stands just outside the door they use first. Walked up to within six blocks, it is at once as it was, where it was (`warp_mob`, ABI 25: a mob saved while warped is put back when next seen). Owner's idea, 2026-11-10. | An animal 12 to 32 blocks from them, they outdoors. |
| `lights_out` | 4 | At night, the target's own lights behind them as they go (within 20 blocks, out of view, nearest first, up to five) go out one by one, three quarters of a second apart, and each is back a minute after it went, as it was (`hide_block`, ABI 24: put back at the time, or at once when the level saves or closes). Owner's idea, 2026-11-10. | Night, and lights they placed behind them. |
| `soul_swap` | 3 | One of the target's torches or lanterns, the farthest out of view, burns with soul fire from then on, the same shape where it hangs (`swap_block`, ABI 24, keeps a wall torch's facing). Owner's idea, 2026-11-10. | A torch, lantern or campfire of theirs out of view. |
| `death_light` | 2 | A candle, lit, on the ground where the target last died, once for each death. Owner's idea, 2026-11-10. | A death of theirs no candle of its own marks. |
| `light_taken` | 3 | The torch or lantern the target placed last is gone, and the same light stands six to ten blocks on along the way they went (their trail, or the route away from the house), against a wall if nothing else holds it. Owner's idea, 2026-11-10. | A light they placed and have not broken, a way on from it out of view. |
| `cave_lit` | 3 | A cave near the home the target has not entered has torches in it, up to six, from its near end inward, at the spacing the target uses in their own tunnels (the middle of their underground lights' gaps, 4 to 14 blocks; 8 before they have lit one). Its own torches. Coded 2026-11-10. | A cave they have not entered. |

The caves come from the brain's cave read (`core/src/caves.rs`): every ten minutes while the
target is above ground within 48 blocks of home, a scan around them of 97 by 41 by 97 blocks
read every third block (`step`, ABI 23) for air with no sky light and no block light (`dark`):
since 1.18 caves are mostly plain air, not `cave_air`. A cave is a crowd of six such places
within 8 blocks; it is unentered when none of it is within 10 blocks of where they walked or
stood, 6 of what they built, 16 of a site of theirs, or the home's radius.

### Beds and rest

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `shift_bed` | 3 | The bed is one block over, to its side, both halves, colour and facing kept, while the target is away (`shift_bed`, ABI 26: never a bed someone lies in, only onto solid floor). Coded 2026-11-10. | A bed and room to move it. |
| `turn_bed` | 3 | The bed faces another wall. Coded as batch 3's `bed_turned`. | A bed and room to turn it. |
| `sheets_changed` | 3 | Their bed is another colour when they come home, both halves, same place and facing (`swap_block` turns a bed's two halves together). Owner's pick, 2026-11-10. | A bed, the target away or asleep. |
| `under_the_bed` | 3 | As they lie down, a slow scratching right under their bed, three or four beats, then nothing: silverfish steps, knocks on wood, or wool and claws, one of three. Owner's pick, 2026-11-10. | A bed, the target in it. |
| `foot_of_bed` | 4 | As they wake, the figure stands at the foot of their bed, across it from them, for half a second. One waking in three, its rules allowing; never offered, the waking starts it. Owner's pick, 2026-11-10. | A bed, the target waking. |
| `second_bed` | 4 | A second bed stands beside theirs when they come home, the same facing, another colour (`bed_beside`, ABI 26). Once a stay. Owner's pick, 2026-11-10. | A bed and room beside it, the target away or asleep. |
| `refuse_rest` | 3 | The next attempt to sleep is refused with vanilla's own `You may not rest now` message. Once per night. | The target trying to sleep. |
| `bed_occupied` | 3 | `This bed is occupied`, vanilla's message, with nobody in it. The next try works. | The target trying to sleep. |
| `wake` | 3 | The target is woken in the middle of the night, sleep cancelled, and the door is open. | The target asleep. |
| `woke_elsewhere` | 4 | The target wakes one block from where they lay down. | The target asleep and room beside the bed. |
| `egg_on_bed` | 2 | An egg from the pen is on the bed. | Chickens and a bed. |

### Decoration and signs

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `frame_rotate` | 1 | An item frame's item is rotated one notch. | An item frame with an item. |
| `frames_turn` | 3 | Every item frame in the room is turned to the same orientation, while the target is away. | Three or more item frames in a room. |
| `frame_swap` | 3 | The item in a frame is swapped with one from a chest. | An item frame and a chest. |
| `frame_emptied` | 3 | An item frame is empty; its item is on the bed. | An item frame with an item and a bed. |
| `painting_change` | 2 | A painting is another painting of the same size. | A painting. |
| `sign` | 2 | A sign with one word from the want's list, at the spot or by the door. | A spot. |
| `sign_word` | 3 | One line of a sign the target wrote reads one word of its own instead: the last line with text (`set_sign`'s `one_line`, ABI 28: a blank sign is refused). The other lines are kept. Coded 2026-11-10. | A sign they placed. |
| `count_up` | 3 | Its kill sign's count rises by itself: each time they kill another of the kind it counts, the sign reads the new number (`15 cows`). Never offered; the kill starts it. Owner's pick, 2026-11-10. | Its kill sign, and a kill of that kind. |
| `weapon_on_wall` | 3 | The weapon they use most hangs in an item frame of theirs, moved straight from its chest slot (`set_frame_item`'s `from`, ABI 28: only into an empty frame, so nothing is lost). Owner's pick, 2026-11-10. | Their weapon in a chest and a frame of theirs. |
| `head_on_post` | 4 | A fence post two blocks out from the door they use, and on it the head of the kind they kill most (a zombie's, a skeleton's, a creeper's, a piglin's), facing the door. Owner's pick, 2026-11-10. | A kill of a kind with a head, a door. |
| `grave_each` | 4 | A row of graves at its place, two blocks apart: for each kind they killed, most first and five at most, a stone and a sign with the count and the kind. Once a stay. Owner's pick, 2026-11-10. | Two kinds killed, a home. |
| `banner_turned` | 2 | A banner faces another way. | A placed banner. |
| `named_animal` | 3 | One of the target's animals has a name, one word from the want's list. | A penned animal without a name. |
| `plant_flower` | 2 | A flower from a chest is planted by the door or on the route. | A chest with a flower. |

### Farm and animals

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `harvest_one` | 2 | One ripe crop is harvested and its yield is in the chest. The soil is left as it was. | A field and a chest. |
| `replant_other` | 3 | One crop is a different crop, from seeds in a chest. | A field and a chest with other seeds. |
| `trample` | 3 | Farmland from the field toward the door, one line, while the target is away. | A field within 32 blocks of the home. |
| `field_word` | 4 | Crops cut back to seedlings so that from above, north at the top, the field reads one word of the want's list in letters three by five: the longest that fits (`MINE`, `LOOK`, `GIVE`; `ME` in a field of 7). Coded 2026-11-10 from the brain's field read. | A field of 7 by 5 or more, four in five planted. |
| `scarecrow` | 3 | An armour stand in the field, wearing one piece of the target's armour (the very piece, from their chest), facing the house: their own stand moved there, or one from the chest set down. Coded 2026-11-10. | A piece of armour and an armour stand (or one in a chest), and a field. |
| `open_gate` | 2 | A pen's gate is open in the morning. The animals wandered. | A fence gate with animals behind it. |
| `animal_indoors` | 3 | One of the target's animals is inside the house in the morning. Moved, not spawned; the gate is closed behind it. | A penned animal and a home with a door. |
| `animal_gone` | 3 | One animal is missing from the pen, and is at the end of the target's route, grazing. | A pen and a route. |
| `sheep_dyed` | 2 | A sheep is another colour, from dye in a chest. | A sheep and dye. |
| `horse_moved` | 2 | The horse is tied to a different post, within eight blocks of the first (`move_leash`, ABI 27). Coded 2026-11-10. | A horse (or donkey, mule, llama, camel) on a lead. |
| `face_the_house` | 4 | At dawn every animal in their pens (up to twelve) stands still facing the house for two minutes (`hold` with `at`, ABI 27). Owner's pick, 2026-11-10. | Three animals in the pens, dawn. |
| `one_more` | 3 | In the morning one more of the kind they keep most stands with the others, and is gone by noon (`place_entity` for a while, ABI 27: gone at once when the level closes). Owner's pick, 2026-11-10. | Two animals in the pens, dawn. |
| `row_withered` | 3 | One row of their crops is back to seedlings; the rest untouched. Owner's pick, 2026-11-10. | A field, the target away or asleep. |
| `eggs_to_door` | 2 | Eggs in a line from the coop to the door they use, two blocks apart, up to six. Owner's pick, 2026-11-10. | Two chickens by the home and a door. |
| `animals_nip` | 3 | The nearest farm animal turns on them: it runs at them and nips, half a heart, twice at most, and never below half a heart (`charge`, ABI 27). The owner's harm stages: a nip at 3. Owner's idea, 2026-11-09. | A farm animal within 16 blocks, the target outdoors. |
| `animals_turn` | 4 | Up to three of the nearest farm animals turn on them for half a minute, a heart and a half a hit, three hits each at most, never below half a heart: real harm from stage 4, never a kill. Owner's idea, 2026-11-09. | Farm animals within 16 blocks, the target outdoors. |
| `hostiles_stare` | 3 | At night every hostile mob within 48 blocks (up to sixteen) stops where it stands and turns to the target, and stays so, facing them as they move, attacking no one, until dawn burns or frees them. Hit once, a held mob takes it unmoved, still staring; hit twice, that one wakes alone and acts as it would (owner, 2026-11-09). None is spawned: a night with few has few. Owner's idea, 2026-11-09. | Three or more hostile mobs near, night. |
| `stare` | 3 | Every animal and villager within 32 blocks stops, turns to the target and holds still looking at them for ten to thirty seconds (`hold`: they stand still rather than look and walk at once, which jittered). Never a hostile mob: that would be a fight. | Three or more animals or villagers nearby. |
| `pet_growl` | 2 | A tamed wolf growls, or a cat hisses, facing an empty spot in the house, for a few seconds. | A tamed pet in the home. |
| `pet_moved` | 2 | A sitting pet is sitting somewhere else, facing a corner. | A sitting pet in the home. |
| `pet_outside` | 3 | The pet is outside the closed door in the morning. | A pet indoors and a door. |
| `bees_out` | 3 | At night the bees of their hive are out, hanging still around the door they use for two minutes, then home (`release_bees`, ABI 27). Coded 2026-11-10. | A beehive of theirs and a door, night. |

### Chat and the player list

Reading: the target's chat is a sense (see [Senses](#senses)). Writing: with `events.chat`,
and always as the target, to the target. Witnesses see none of it. Off, the lodger neither
speaks nor shows in the list, and still reads.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `tab_entry` | 3 | A second entry with the target's name in the player list, for a minute, seen by the target alone. Ping bar empty. | Any. |
| `tab_entry_stays` | 4 | The entry stays all session. | Any. |
| `joined` | 3 | `<your name> joined the game`, in the target's chat, while they are already in. Nobody joined. | Any. |
| `left` | 4 | `<your name> left the game`, in the target's chat, while they are still in. Then nothing for an hour. | Any. |
| `answers_you` | 2 | What the target types is sometimes answered, 5 to 60 seconds later, in their own name, to them alone: often a line about it (the house, a door, `who`, a question, its want's words), rarely anything else (60 and 15 in a hundred), at most three a day and five minutes of play apart. The model writes it with their words in its prompt, cleaned to letters (it may echo one: `house?`), from the reply bank's examples (`i heard you`, `inside`, `not yet`); the bank stands in. Started by the chat line, never from the menu; the one event that may follow itself. Owner, 2026-11-09. | The target typed, `events.chat`. |
| `says_back` | 3 | When the target types something about the lodger (the senses' chat line matches the want's words, `house`, `door`, `chest`, a question), a chat line in the target's own name, to the target alone, from the bank: `i know`, `it was me`, `look behind the door`. Once a day at most. | The target typed. |
| `says_first` | 4 | A chat line in the target's name, unprompted, at night: one word from the want's list, or a bank line. | Night. |
| `whisper_from_you` | 4 | A whisper to the target from the target: `<you> whispers to you: here`. | Any. |
| `repeats_you` | 4 | The target's last chat line, said again, in their name, a minute later. The words are the target's own; nothing is added. | The target typed. |
| `sleep_message` | 3 | `1/2 players sleeping` on the action bar, the game's own words, at night with the target awake and alone. (`sleepers_wrong` says it as they lie down.) Coded 2026-11-10. | Night, the target awake. |
| `advancement` | 4 | The chat line and the toast for one the target has, as if just earned, in their name (`game_message` `advancement`: the host picks one they have that announces itself; the toast is the client's own). Coded 2026-11-10. | The target awake, with an advancement. |
| `death_message` | 4 | One of the game's own death lines about them while they live: `<you> fell from a high place`, `drowned`, `suffocated in a wall`, `fell out of the world`, `died`, `starved to death`, `froze to death`. Owner's pick, 2026-11-10. | The target awake. |
| `words_on_sign` | 3 | What they last typed, in their own words, lower case, on a sign beside the door they use, up to four lines of fifteen. Owner's pick, 2026-11-10. | They typed, a door, the target away or asleep. |
| `ping_fades` | 3 | Their own entry in the player list shows no connection for a minute, then their own again. Owner's pick, 2026-11-10. | The target awake. |
| `empty_message` | 3 | `<you>` in their chat with nothing after it. Owner's pick, 2026-11-10. | The target awake. |

### Villages

Offered only with a village within 128 blocks of the home.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `scan_village` | 2 | Nothing the target sees: the lodger looks the village over once, so it knows its doors and its bell. The host's scan, forty blocks around the village's middle. | A village in the surroundings. |
| `bell` | 2 | A bell rings once, at night. Witnesses hear this one: it is a real bell. | A bell within 48 blocks. |
| `villager_at_door` | 3 | A villager stands outside the target's door at night, facing it, and is back home by morning. | A villager and night. |
| `golem_turned` | 3 | The iron golem faces the target's house all night and all the next day, wherever it stands. | An iron golem. |
| `village_doors` | 3 | Every door in the village is open in the morning. | A village with 4 or more doors. |
| `village_lantern` | 2 | A village lantern is gone, and stands on the target's route where it passes the village (set down there first, so one that finds no ground stays where it hung). Coded 2026-11-10. | A village lantern and a route. |
| `villager_word` | 4 | The villager nearest them sells what the lodger took (the very item, `offer_trade`, ABI 31) for one emerald, once; bought, the trade is gone, and so is the lodger's hold on it. Coded 2026-11-10. | Something it holds, a villager near. |
| `village_abandoned` | 4 | While the target is 64 blocks or more away at night, the village empties: every villager within 48 blocks of its middle walked into the house whose door is nearest the middle, packed inside, facing its back wall, silent, turning to the target as they come within five blocks (`gather_villagers`, ABI 31); every other door open, the lanterns out (`hide_block`). By dawn, everyone is back where they stood, the doors shut, the lights lit. Once a stay. Owner's idea, 2026-11-09; coded 2026-11-10. | The village's doors read, night, the target far from the village. |
| `bell_twice` | 2 | They ring a bell; a second and a half later it rings again, once, swinging, half the times they ring it (`rang_bell`, ABI 31; never offered, the ringing starts it). Owner's idea, 2026-11-09 (G1). | The target ringing a bell. |

### Water and boats

Offered with water within 32 blocks of the home or the route.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `boat_moved` | 2 | The target's boat is on the other bank, or their minecart at the other end of its track. | A boat or minecart the target used. |
| `boat_adrift` | 3 | The boat is in the middle of the water, still, going nowhere. | A boat and open water. |
| `boat_drifts` | 2 | While the target watches from the shore, the boat drifts off on its own, slowly, away from them. (Owner's idea, 2026-11-09: "boats and minecarts move on their own".) | A boat on the water and the target outdoors. |
| `boat_waiting` | 3 | The boat is at the shore the target is walking toward, before they get there: on their side of the water, a block in from its edge. Coded 2026-11-10. | Their boat, water within 48 blocks, the target walking. |
| `bucket_floor` | 3 | One block of water in the house's floor, in a corner. | A bucket in a chest and a floor block out of the way. |
| `ice_path` | 3 | A line of ice across the water from the far shore toward the house, in summer. | Still water of 8 or more blocks across. |
| `ripples` | 3 | Ripples and small splashes cross the water toward the shore on their side, in a line, as under footsteps, for the target alone (the client's `ripples` hint). Coded 2026-11-10. | Water within 32 blocks, the target outdoors. |
| `fishing_line` | 4 | Fishing with no one there, faked (a real bobber needs a player holding the rod; owner, 2026-11-10): a cast, the bobber's splashes and a catch over a minute at one spot on their water, the fishing ripples there for the target alone (the client's `bobber` hint). Coded 2026-11-10. | Water within 32 blocks and a fishing rod in a chest. |
| `boat_in_house` | 4 | Their boat sits in the middle of the house in the morning. Owner's pick, 2026-11-10. | Their boat and a home, the target away or asleep. |
| `under_the_water` | 3 | A dark shape, a squid, hangs still under the water on their side, facing them, for twenty seconds, then is gone (`place_entity` with `face`, ABI 30). Owner's pick, 2026-11-10. | Water within 24 blocks, the target outdoors. |
| `wet_steps` | 3 | Wet steps and a drip with each walk from the door they use to their bed while they are inside. Owner's pick, 2026-11-10. | A bed and a door, the target indoors. |
| `drip` | 2 | Dripping water, indoors, with no water near. | The target indoors. |

### Snow, sand and weather

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `trail` | 3 | Footprints in the snow by the home, from the door toward the lodger's place, that the target did not make: a snow layer taken off at each step, left and right (`prints`, ABI 30; snow only, sand cannot show them). Gone with the next snowfall. Coded 2026-11-10. | Snow around the home and a door. |
| `trail_in` | 4 | Footprints leading *to* the door, from twenty blocks out in the dark, and none leading away. Coded 2026-11-10. | Snow around the home and a door. |
| `snowman` | 3 | Two snow blocks and a carved pumpkin stacked beside the door, facing it, not a golem. Owner's pick, 2026-11-10. | Snow around the home and a door. |
| `snow_cleared` | 2 | The snow is gone from the path between the door and the route, one block wide. | Snow layers on the path. |
| `snow_indoors` | 3 | Snow falls inside the house for a few seconds, for the target alone. | The target indoors, snow outside. |
| `rain_indoors` | 3 | The same with rain. | The target indoors, rain outside. |
| `lightning_silent` | 3 | A flash of lightning with no thunder, in clear weather, for the target alone. | The target outdoors at night. |
| `thunder_clear` | 2 | Thunder, for the target alone, under a clear sky. | The target outdoors. |
| `fog_close` | 3 | The mist rolls in for a minute, then lifts: the real fog of `mist`, since 2026-11-10 (it was a dark ring on the screen). | The target outdoors. |

### Mines and caves

Offered when the memory holds a mine: a tunnel the target dug of 10 or more blocks, or a cave
they use. Digging is what a player digs, at a player's pace, with a player's sounds (see
[Like a player](#like-a-player) and the [rule](good-to-know.md#what-it-never-does)).

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `mining_sounds` | 2 | While the target mines, pickaxe-on-stone sounds from inside the rock, 6 to 12 blocks off, a few minutes at a time, keeping pace with the target's own. Nothing is dug. | The target mining underground. |
| `echo_steps` | 2 | Footsteps that answer the target's from a side tunnel, in the mine. | The target in a tunnel with a branch. |
| `scan_ore` | 2 | Nothing the target sees: while they mine, the lodger looks at the rock around them (the host's scan, 24 blocks across) for ore they have not found, so `ore_marked` has a vein to mark. | The target underground. |
| `ore_marked` | 2 | A torch on an ore vein the target has not found, within sight of their tunnel. | An unfound vein within 12 blocks of a tunnel. |
| `mine_beside` | 3 | While the target is away, a tunnel the size and length of their newest one is dug beside it, three blocks over, lit the way they light theirs (the memory's torch spacing). Its ore is in the chest they use; it keeps the stone. | A tunnel of 10 or more blocks the target dug, and a chest. |
| `mined_out` | 3 | One ore vein the target was heading for, the one their tunnel points at, is gone when they reach it: a hollow with torches in it, as if mined an hour ago. | A vein within 16 blocks ahead of the target's tunnel. |
| `tunnel_sealed` | 3 | The mouth of the target's oldest tunnel is walled with cobblestone, the lodger's own. | An old tunnel and held stone. |
| `ladder_down` | 3 | A ladder from a chest down a one-block shaft the lodger dug, to its place, or to nothing. | A ladder in a chest and natural ground. |
| `minecart_rolls` | 3 | The target's minecart rolls out of the tunnel on its own and stops at the mouth. | A minecart on a track. |
| `tunnel_toward` | 4 | While the target digs a tunnel, a tunnel is dug toward theirs from 20 to 30 blocks off, the sounds coming closer over several minutes, and it breaks through a few blocks ahead of them, or stops one block short so the wall between is thin and torchlit from the other side. Nothing is in it. | The target digging a tunnel of 20 or more blocks, and solid rock on the far side. |
| `portal_sound` | 3 | The sound of a Nether portal in the house, with none near. | The target indoors. |
| `portal_lit` | 3 | A portal frame the target built and never lit is lit, with flint and steel from a chest. The lodger does not go through. | An unlit frame and flint and steel. |

### Redstone

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `lever` | 2 | A lever in the home is flipped, and whatever it powers happens: a lamp, a door. Never a lever with TNT within 16 blocks of anything it powers; the host checks. | A lever. |
| `note` | 2 | One note from a note block, at night, while the target is awake. | A note block within 32 blocks of the home. |
| `note_tune` | 3 | Three notes from the note block, the first three of a disc the target owns. | A note block and a disc in a chest. |
| `jukebox` | 3 | A disc from the target's chest is playing in their jukebox as they come to the door, or the one playing stops, its disc back in the chest (`jukebox`, ABI 29). Coded 2026-11-10. | A jukebox and a disc in the chest they use, the target coming to the door. |
| `door_opens_itself` | 3 | A door opens as the target approaches, with no plate and no one there. | A door in view. |
| `plate_laid` | 3 | A wooden pressure plate from a chest, in front of the chest the target uses. | A plate in a chest. |
| `piston_once` | 3 | A piston in the base extends and retracts once, at night, with its sound. Nothing is hooked to TNT. | A piston. |
| `lamps_noon` | 3 | The redstone lamps are on at noon and off at night, for a day. | Redstone lamps. |
| `lamps_blink` | 3 | At night their redstone lamps, up to six, blink together: three short, three long, three short, once, each ending as it was. Owner's pick, 2026-11-10. | Redstone lamps, night. |
| `composter_fills` | 2 | Their composter is one level fuller in the morning, nothing put in (`set_state`'s `@up`, ABI 29, never past full). Owner's pick, 2026-11-10. | A composter, dawn. |
| `bell_rings` | 3 | A bell of theirs rings once at night, swinging, with no one near it (`toggle_block` rings a bell). Owner's pick, 2026-11-10. | A bell, night. |

### Sounds

Every sound event has three to five ways to sound (`brain/sounds.rs`: the sounds, a pitch, a
rhythm, and whether it stays put, comes from behind, comes closer or goes away), never the same
way twice running; footsteps and placing are the sound of the real block underfoot or in hand
(`block_sound`); the house's sounds come from a real door or chest of the house the target is
in, theirs first (owner, 2026-11-09).

Heard by the target alone. Nothing in the world changes.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `steps_above` | 2 | Footsteps cross the floor above the target, three blocks up, wood or wool underfoot, slow or quick; there is no floor above, or no one on it. | The target indoors. |
| `scratching` | 3 | Something scratches inside the wall beside them at night, small and quick, or slow, a block and a half to their right. | The target indoors, night. |
| `ladder_climb` | 3 | Someone climbs a ladder just behind them, rung by rung, rising five blocks, slow or quick. | The target indoors. |
| `pickup_pop` | 2 | Behind them, the pop of an item picked up, once or twice, or an experience orb taken; nothing on the ground. | The target online. |
| `cold_crackle` | 2 | Their furnace crackles as if lit, once or a few times, and a candle goes out after. It is cold. | A furnace of theirs, the target indoors. |
| `breath_close` | 3 | Breathing at their ear, half a block behind them at head height, once or slowly a few times; in its last way a heartbeat. | The target online. |
| `cave_air` | 2 | The sound of a cave out in the open at night, behind them, going away, or coming closer. | The target outdoors, night. |
| `door_scratch` | 3 | Scratching at a door of the house they are in, at night: quick, or in two bursts, or something small. | A door, the target indoors, night. |
| `door_sound` | 1 | A door's sound from another room. No door moved. | A home with two rooms. |
| `chest_sound` | 1 | A chest's sound from another room. | A home with two rooms. |
| `cut_music` | 1 | The client's music stops and does not start for an in-game evening. | The target online. |
| `late_steps` | 2 | The target's own footstep sounds play once more, half a second after they stop, from their own position. | The target walking. |
| `late_eating` | 2 | The same with eating: one more bite after the target's. | The target eating. |
| `place_sound` | 2 | A block-placed sound behind the target. Nothing placed. | The target indoors. |
| `knock` | 2 | Soft knocks on a door of the house the target is in, at night: three evenly, or two, a pause and one, or four quick, or one; the door shut, no one there. A door-close sound pitched up and quiet (owner, 2026-11-09: soft knocks early, the heavy one later). | A door, the target indoors, night. |
| `knock_heavy` | 4 | Something heavy at the door at night: slow pounding, two, three or five, the sound of a zombie at a wooden door pitched low; nothing there when it is opened. | A door, the target indoors, night. |
| `lever_click` | 2 | A lever's click, in a house with no lever. | The target indoors. |
| `page_turn` | 2 | Pages turning by the lectern. | A lectern. |
| `arrow_hit` | 3 | An arrow hits the wall behind the target. Nothing is there. | The target indoors. |
| `wolf_far` | 2 | A wolf whines far off, long and low, in a biome with none (26.3 has no howl). | Night. |
| `disc_phantom` | 3 | A disc the target does not own plays from the jukebox, which is empty. | A jukebox. |
| `silence` | 3 | Cave sounds and ambient sounds are silenced for an evening. | The target online at night. |
| `your_voice` | 4 | The target's own hurt sound, once, from the next room. | The target indoors. |

### The sky and the light

Only the target's client shows these. Nothing in the world changes and no one else sees them,
which is what makes them worth arguing about on a server. Each is a hint packet (see
[What the client knows](architecture.md#what-the-client-knows)) the client renders for a few
seconds and then stops; `events.glitches` turns the group off, with [The screen](#the-screen).

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `flicker` | 2 | One light near the target flickers for a few seconds, then is steady. | A light within 8 blocks of the target, indoors. |
| `lights_fail` | 3 | Every light in the room gives no light, for five to twenty seconds, and the room is as dark as night. Then they are back. | The target indoors at night. |
| `painting_flip` | 2 | The painting nearest them hangs upside down until they have looked straight at it for two seconds (a render hook turns it half round on the wall). Coded 2026-11-10. | A painting, the target indoors. |
| `sun_blink` | 3 | The sun is gone, and the sky night-dark, for one to three seconds in the middle of the day. Then it is back and the time is right. | The target outdoors by day. |
| `moon_wrong` | 3 | The moon is full when it is not, for a few seconds (the sky's hook; twice its size is beyond the hook). Coded 2026-11-10. | The target outdoors at night. |
| `stars_out` | 3 | The stars fade out for a minute and back. Coded 2026-11-10. | A clear night, the target outdoors. |
| `night_early` | 4 | The sun, moon and stars stand two hours later than they are, the sky dimmed, for a minute, eased in and out, then right. Coded 2026-11-10. | The target outdoors by day. |
| `red_rain` | 4 | While it rains, the drops themselves are red (the rain's colour hook), for two to five minutes; no screen cast since 2026-11-10. The brain may pair it with `red_mist`. | Rain, the target outdoors or at a window. |
| `compass_points` | 3 | A compass the target holds points at the lodger's place for two minutes (a hook on the compass's target). Coded 2026-11-10. | The target holding a compass. |
| `reflection` | 4 | Water reflects nothing in the game, so a face under it instead (owner, 2026-11-10): the figure just under the surface of still water near them, its head below the top, for one second. Coded 2026-11-10. | Water within 24 blocks, the target outdoors. |

#### The atmospheres

Real fog on the target's client (the review's D, owner 2026-11-09: "not abrupt like other mods,
make it good and spooky"): each eases in over most of a minute, holds with its edge breathing a
tenth over nine seconds, and eases out over half a minute; they stack, each pulling the fog as
it stands. A render hook at the end of the game's fog setup closes and tints it
(`craftkit.host.client.Atmosphere`, `Atmospheres`); the sky takes the fog's colour.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `mist` | 2 | Pale grey mist to thirteen blocks, motes drifting near the ground, four minutes. `fog_close` is this mist now, a minute long. Coded 2026-11-10. | The target awake. |
| `dusk_noon` | 3 | At noon the light drains: the fog grey-violet to forty blocks, the sky dimmed, a grey veil, five minutes. Coded 2026-11-10. | Day, the target awake. |
| `red_mist` | 4 | Deep red mist to twenty blocks, a rust sky, red motes, four and a half minutes. Coded 2026-11-10. | Night, the target awake. |
| `darkness` | 4 | The dark closes to seven blocks, black, their own light still showing what is near, three minutes. Coded 2026-11-10. | Night, the target awake. |

### The screen

The target's own HUD and camera, for a moment. Off with `events.glitches`.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `hotbar_slip` | 2 | The selected hotbar slot moves to the favourite tool for one tick. | The favourite in the hotbar. |
| `hunger_shake` | 2 | The hunger bar shakes as if starving, for three seconds, their saturation put back after. Coded 2026-11-10. | The target awake. |
| `heart_skip` | 3 | The hearts blink as if hurt, once, with no damage. Coded 2026-11-10. | The target awake. |
| `subtitle` | 2 | With subtitles on, `Footsteps` appears behind them, with no sound. Coded 2026-11-10. | The target awake. |
| `camera_nudge` | 3 | The camera turns two degrees toward the figure, or the place, once. | A figure or a place. |
| `fov_pull` | 4 | The field of view narrows over ten seconds and snaps back. | The target indoors. |
| `tab_twice` | 3 | The target's own name appears twice in the player list for six seconds. Coded 2026-11-10. | The target awake. |
| `sleepers_wrong` | 3 | `1/2 players sleeping` on a server with one player, or one too many. | The target trying to sleep. |
| `crosshair_gone` | 3 | The crosshair is gone for a few seconds. | Any. |

### Like a player

The lodger wears your skin, and in these it does what you do. Everything here is dug or placed
the way a player would, one block at a time at a player's pace with a player's tools, so the
sounds are the game's own and a witness who looks sees nothing the game could not do. Digging
keeps to natural blocks (see [What it never does](good-to-know.md#what-it-never-does)). The
mining events are in [Mines and caves](#mines-and-caves); these are the rest.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `tools_in_hand` | 3 | The figures hold what the target holds most, the memory's favourite tool, and the far figure swings it at the rock now and then. | A figure and a favourite tool. |
| `your_spacing` | 3 | Torches it places, anywhere, are at the spacing the target uses in their own tunnels; its tunnels are the target's height and width. | A tunnel the target dug. |
| `copy_crouch` | 4 | The far figure crouches when the target crouches, and stands when they stand, at 60 blocks. | A far figure showing. |
| `copy_build` | 4 | A one-block-high outline of the target's house, in cobblestone the lodger dug, at its place, the right size and facing. | A home of 20 or more blocks and held stone. |
| `wave` | 4 | The far figure raises its arm once, as a player swinging, and is gone. | A far figure showing. |
| `through_the_hill` | 4 | It walks into one side of a hill in view and, a minute later, out of the other, the stone closing behind it both times. Approached, both faces are natural rock. | A hill of natural stone 10 to 30 blocks thick in view, the target 40 or more blocks off. |
| `into_the_dark` | 3 | It stands at the edge of your torchlight, steps back one block out of it, the torch flickers, and the light finds nothing. | The target outdoors at night by a torch, open ground beyond. |
| `into_the_mountain` | 4 | Seen from 40 to 80 blocks, the figure digs into a mountainside at a player's pace, a one-by-two tunnel, and the stone closes behind it block by block as it goes, until the face is smooth and it is gone. Walked up to, the face is natural stone; one block in, a torch, and behind it solid rock. Owner's idea, 2026-11-09. | A mountainside of natural stone in view from the route or home, and the target 40 or more blocks from it. |

### Building

What it builds, it builds from what it dug, cut and picked, at a player's pace, out of sight,
over nights (see the [rule](good-to-know.md#what-it-never-does)). Trees it fells it replants.
More in [Ideas](ideas.md#building).

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `pillar` | 3 | A cobblestone pillar at its place, one block higher each night, up to the height of the target's roof. | A spot and held stone. |
| `cave_room` | 3 | A 3x3x3 room in the nearest cave, lit at the target's torch spacing, with a chest holding what it took and a door of logs it cut. | A cave within 64 blocks and a tree. |
| `dock` | 3 | A log dock on the water where the target fishes, one block longer each night. | Water the target fishes in and a tree. |
| `hut_on_route` | 3 | A 3x3 shelter of logs halfway along the target's longest route, a torch inside, nothing else. | A route of 60 or more blocks and a tree. |
| `house_near` | 4 | A house of its own, 40 to 80 blocks from the target's, 5 by 5, its door facing theirs, one window, one torch, built over three nights of cobblestone and logs. Inside, a chest with what it took. It grows a room in stage five. | A spot in sight of the home, held stone and a tree. |
| `bridge` | 4 | A one-block bridge of logs from its place toward the target's shore, a few blocks further each night, finished only in stage five. | Water between its place and the home. |
| `copy_house` | 5 | A copy of the target's house in cobblestone and logs at its place, the same doors and windows, built over the last nights. | A home of 20 or more blocks. |

### The figures

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `far_figure` | 3 | The far presence: a figure in the target's skin, name shown, 48 to 80 blocks away, standing, facing the target, for ten to forty seconds. Approached within 32 blocks, it is gone. | A line of sight from the route or home. |
| `behind_you` | 3 | When the target looks back along their route, the figure is 60 blocks behind, standing. The next time, 50. It is never nearer than 32. | The target travelling a route. |
| `on_the_roof` | 3 | At dawn, from outside, the figure stands on the target's roof, and is gone when they look away. | A home with a roof and the target outdoors at dawn. |
| `across_water` | 3 | The figure stands on the far shore, or sits in a boat in the middle of the water. | Water of 32 or more blocks across. |
| `in_doorway` | 4 | As the target turns, the figure is in the doorway or at the window for less than half a second, then not. No sound. | The target indoors, facing away from a door or window. |
| `at_the_chest` | 4 | Coming into the room, the target sees the figure at their chest, back to them. It closes the lid and is around the corner before they reach it. | A used container in a room with two ways out. |
| `in_your_bed` | 4 | Coming home late at night, the target sees the figure lying in their bed through the door, and the bed is empty when the door opens. | A bed in view of the door, night. |
| `replay` | 4 | The replay figure walks a recorded route, at the recorded pace, starting out of view. Looked at, it keeps walking. Approached within 24 blocks it rounds a corner and is gone, leaving the held item where it stood if it held one. | A recorded route of 20 or more points. |

### The ending

| Event | Stages | What happens |
| --- | --- | --- |
| `settle` | 5 | The ending's own script over one to three days. See [How it ends](#how-it-ends). |
| `wait` | all | Nothing. Always offered, always allowed. |

### Chains

One thing leads to the next. An event can name follow-ups in `events.rs`: events that, once it
has landed, are offered with extra weight, or offered *only* to the exclusion of the rest, for a
few days, and sometimes a place or an item the follow-up inherits. The memory keeps the open
chains (`Memory::chains`), the situation text says what is open (`It took the pickaxe three days
ago and has not given it back.`), and the model chooses with that in front of it. A chain is
never forced: `wait` is always offered, and a follow-up whose needs stopped being met falls out.
Some chains, with the stages they span:

- **The thing.** `take` → (`place_taken` or `return_taken`, favoured for three days) → if placed,
  `figure_leaves_item` or `at_the_chest` with it in hand → if the target takes it back,
  `frame_emptied` everywhere by morning → `want_refused`. [2 to 4]
- **The vein.** `ore_marked` (a torch on a vein) → if the target mines it, nothing; if not,
  `mined_out` a day later → `ore_in_chest`: the ore is in their chest after all. [2 to 3]
- **The tunnel.** `mining_sounds` while they mine → `mine_beside` while they are away →
  `tunnel_toward` the next time they dig → the broken-through tunnel leads to the `cave_room`.
  [2 to 4]
- **The place.** `light_path` → `hole` at its end → `pillar`, a block a night → `sign` on the
  pillar → `house_near` beside it → `house_grows` → the figure `at_its_door` → the ending's
  last night is there. [2 to 5]
- **The watcher.** `far_figure` → if looked at for long, `behind_you` → `on_the_roof` →
  `in_doorway` → `in_your_bed` → `copy_crouch` and `wave` → `want_seen_met`. If chased
  instead, `figure_runs` and a week of nothing at all. [3 to 4]
- **The copy.** `your_spacing` in its torches → `copy_build` outline → `copy_house` over the
  last nights → `woke_in_other_bed` on the last morning. [3 to 5]
- **The house.** `open_door` → `door_hinge` → `second_door` → `boarded` windows, in that
  order over a week, each only after the target has noticed the last (walked up to it, or lit
  it). [1 to 4]
- **The pet.** `pet_growl` at a corner → `pet_watching_spot` → `pet_follows_figure` → the pet
  sits beside the figure, far off, then comes back. [2 to 4]

A chain's steps still obey cooldowns and caps, and the budget; what a chain changes is the
menu's weights and, for an exclusive step, the menu itself for a day or two. The scripted-stay
tests check that a stay with the rule brain walks at least two chains end to end and that no
follow-up is ever offered with its needs unmet.

### Provocation and temper

The lodger answers how it is treated (owner, 2026-11-09). The memory keeps a **temper**, 0 to
1, starting at the preset's resting value (0.3 for `balanced`), moved by what the target does
and read by everything that paces the stay:

| The target... | Temper |
| --- | --- |
| chases the figure, or hits it | up, a lot |
| breaks its torches, its signs, its cairn, its pillar, its house | up |
| takes the thing back after it was placed; empties its chest | up |
| walls, floods or lights its place; puts TNT or lava near it | up, a lot |
| kills an animal it moved, or names one back | up |
| insults it in chat (the chat sense: lines at it with a curse, `leave`, `go away`) | up |
| sleeps every night and plays by day only, as if nothing were there | up, slowly: it is being ignored |
| builds something at its place; leaves the thing where it put it; leaves a chest for it | down |
| looks at the figure without approaching; stands at its sign | down |
| lights the house but not its place; closes its door gently (the memory cannot tell gently: closes it and leaves) | nothing |
| talks to it in chat without insult (`hello`, `what do you want`) | down, a little |
| time passes with nothing | drifts back toward the resting value, a tenth a day |

What temper does, each bounded by the preset so `calm` never becomes `relentless`:

- **The budget.** Events a day are the preset's times `0.5 + temper`, so a stay runs at half
  pace when it is appeased and at one and a half when it is provoked.
- **The stages.** Above 0.7 a stage's least days are halved; below 0.2 its most days are
  stretched by half. Provoke it and the figures come a week sooner.
- **The menu.** Above 0.6 the bolder events (the near figures, the sleep events, `knock`,
  `lights_fail`, `tunnel_toward`) are offered a stage early; below 0.2 they wait a stage.
- **The distance.** Above 0.8, `events.min_distance` is three quarters of the preset's.
- **The ending.** The want's score and temper together pick the ending when `stages.ending` is
  `difficulty`: a met want with low temper leaves; a refused want with high temper takes; the
  rest keeps. Temper above 0.9 at the end of stage four takes on any difficulty.
- **The model** reads it as a line: `It is angry: the player broke its torches twice`, and the
  `react` questions carry it, so the choice of *which* event is coloured too.

The rules still hold at every temper: a furious lodger does everything it does more often and
sooner, never anything it would not do. How much temper moves per act is a table in
`memory/temper.rs` with a scripted-stay test per row.

### How often

Events are rare, and get less rare. Each stage has a budget of events per in-game day
(`stages.events_per_day`, by default one a day in stage one, two in stage two, four in stage
three, six in stage four), and the brain is asked to plan only while the day has budget left.
Sounds and the screen's glitches count half. A day with budget left at its end does not carry it
over; a quiet day is a quiet day. So a whole stay of about two in-game weeks holds some forty to
sixty events, one most evenings from the start and several a day in the last days before the
end. That is the denser of the two paces considered (the other was half of it over three weeks;
the owner picked this one, 2026-11-08), and `events.cooldown_scale` and `stages.events_per_day`
move it either way.

Repetition is held off three ways, all in `events.rs` and `brain/offer.rs`:

- **Cooldowns.** Each event has one (`cooldown_days`, 2 by default; 1 for sounds and
  `open_chest`), and is not offered again until it has passed, nor at the same place for twice
  as long.
- **Caps.** Each event has a most-per-stay (`most_per_stay`, 4 by default; `take` and the
  figures more, `sign_word`, `animal_indoors`, `field_word`, `copy_build` and `in_your_bed`
  once). Past its cap it is gone for the stay. With a hundred and some events and forty to sixty
  in a stay, most happen once or never.
- **A changing menu.** The brain is offered at most `events.offered` events, eight by default,
  drawn from the eligible ones with a weight that favours what has not happened yet in this
  stay, and never the event that happened last. The model picks from a different menu each
  time, so even a model that always liked the same answer would not give it.

The ramp is the stages: the pool widens, the budget rises, the figures arrive in stage three, and
the ending is stage five. Nothing speeds up because the target is afraid or bored; the model's
read of the target shifts *which* events, not how many.

### Events re-check before they happen

An event is decided tens of milliseconds to a day before it runs, and the world moves. So the
host's executor for each event checks `fits` against the world as it is at that tick: the target
is still away or asleep, the spot is not in view, the container still holds the item, the lever
still powers nothing dangerous, the rock ahead is still rock. An event that no longer fits does
nothing and reports why, the memory records the attempt, and the brain does not offer the same
event at the same place for its cooldown. Nothing is ever forced through. An event that runs
over minutes, such as `tunnel_toward`, re-checks every few seconds and stops where it stands.

### Whose chest

A **used container**, in the rows below, is one of the target's own: a chest, a barrel or a
furnace they placed, one in the house they sleep in whoever placed it (the surroundings pass
reads the house; owner, 2026-11-09: "the home's, placed or not"), or one they opened on three
different days.
A chest opened once on the way past, a village's or a dungeon's, is never touched and never
described to the model (owner, 2026-11-09, after a village chest became "the chest opened
most"). What it holds is read when it is opened and again when it is closed, so what the
target took out is known.

### The distance rule

No event but the sounds, the glitches, the screen, `refuse_rest`, `bed_occupied`, `wake`,
`stand_watch`, `stare`, `door_opens_itself`, `open_on_enter` and the figures happens within
`events.min_distance` blocks, 64 by default (24 until the owner watched a path of torches go
up from farther off, 2026-11-09), of the target while the target faces the spot. Every block
the event changes is checked, not only its own spot, and an event whose blocks are in view
waits for the target to look away, within its day. The lodger works behind you, in the rooms you left, and while you sleep. The
exceptions are the events that are *about* being near: a sound in the next room, a light
failing over your head, a stand turning to look at you. They change nothing the target could
check afterwards, and the figures keep their own larger distances.

## Multiplayer

One lodger per world, one target. Everyone else is a witness.

- **Picking the target.** With `multiplayer.pick` at `model`, the brain is asked, with one line
  per candidate from a short memory the senses keep of everyone (hours played, whether they have
  a home, whether they play alone), which one; `first` is whoever has played longest, `random` is
  random, a name is that player. The choice is made at move-in and logged.
- **What a witness experiences.** The traces, if `multiplayer.witness_traces` is on: the open
  door, the moved torch, the sign, the torches leading somewhere, the trampled field, the item
  where it should not be. Never the figure, never a sound, never a cut in the music, never a
  refused rest. The target says there is something in the house; the witnesses see a door open.
  Whether they believe the target is the multiplayer game.
- **The target's absence.** If the target has not played for `multiplayer.absent_days` real days
  and `multiplayer.retarget` is on, the lodger picks a new target at the next stage boundary, from
  stage two, keeping its want. It does this once per stay. Otherwise it waits; a stay can outlast
  a player.
- **The target's own house, not the server's.** The home is where *the target* sleeps and keeps
  things. A shared base is a home with other people in it, and the lodger works in the target's
  rooms, by the chests the target opens. Witnesses' chests are never touched.

## How it ends

Stage five is three days, and it is the point of the stay. Everything before it was quiet;
this is not. Which ending depends on `stages.ending`, the world's difficulty when it reads
`difficulty`, and whether the want was met, and every ending is a script of three days that
ramps from the first evening to the last night, with the chains above coming to their ends
inside it. The script is data in `arc.rs`, each day a list of beats with a time and a
condition; the brain's part is small (which of two ways a beat goes), and the rules still hold
every step but the one death.

| Ending | When | Day one | Day two | The last night |
| --- | --- | --- | --- | --- |
| **Leaves** | The want was met, on any difficulty; or `stages.ending` is `leave` | Everything it took comes back over the day, one thing an hour, each to where it was taken from. The animals it moved come home. Its torches go out one by one along the path, from the far end toward the house. | Its house (if it built one) is emptied: the chest's contents are on your doorstep at dawn. The figure stands at its door at noon, in plain light, for a full minute, and goes in. At dusk its torch goes out. Every pet of yours sits facing its house. | The music plays, the first time in a week. Every light in your house flickers once at midnight and steadies. At the edge of the light, the figure, in your skin, for as long as you look. When you look away and back, nothing. At dawn its house is gone, the ground flat, one torch where its door was. `/lodger status` says `left`. |
| **Takes** | The want was refused, on normal or hard; or `stages.ending` is `take` | You cannot sleep (`You may not rest now`, all night). Every door in the house opens at midnight at once, and the pets go out. The animals stare at the house all day. The sun blinks out three times before noon. | Every frame is empty, the items on your bed, in a line pointing at the door. Every sign in the house says the same word. The figure is behind you on every route, closer each time, to 32 blocks. Your tunnels are all sealed with cobblestone. The village bell rings every hour. | Every torch it ever placed lights at once, the whole path to its place, and every torch of yours goes dark in the client's render. Red rain. The lights of its house are on. At its place, the figure, holding everything it took, which it keeps. It walks toward you for the first time, at a player's pace, and stops at one block. On hard, you die there, once, with your inventory kept, and the death message is vanilla's for falling out of the world. Then the night is ordinary, the house is as it was minus what it took, and it is gone. |
| **Keeps** | The want was refused, on easy or peaceful; or `stages.ending` is `keep` | Its house gets a second room overnight, a copy of your bedroom, your spare bed in it. The far figure is in its window every hour. | A path of cobblestone from your door to its door. A gate in its fence facing your door, open. Your pets go and sit at its door and come back. | At midnight, its door is open and lit, and the figure is in its doorway, facing you, for as long as you look. It does not come. At dawn it stops acting, for good. The torches stay lit, the signs stay, the things stay where they were put, its house stays with its door open. The music comes back. Every few in-game weeks, one door of yours is open. `/lodger status` says `stayed`. |

Within each day the beats are ordered and timed, but which of two forms a beat takes (the figure
at its door or on its roof; the frames emptied or the paintings turned) is the model's, asked
once per day with the day's beats as the choices, so two players' endings differ in detail and
agree in shape. Nothing in stage five repeats a thing from the same day, and nothing in it is
small: the budget is off, the cooldowns are off, and the caps alone hold, so a stay's last night
can spend everything it has left.

After any ending the stay is closed in `lodger.dat` and no lodger moves into that world again
unless `/lodger reset`. The one death the hard ending deals is the only harm the lodger ever does,
it is at the end of a stay of two weeks, and it keeps the inventory: the point is that it happened,
not that it cost anything. `stages.ending` is there for servers that want none of it.
