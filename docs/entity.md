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
| 1 | **Noticing** | Nothing, almost. One door open. A torch missing from a dark corner and found two blocks away. The chest's contents in different slots. A chest's sound from the next room. The music does not play one evening. | The memory holds a home and a chest the target uses, and at least two events landed. |
| 2 | **Residue** | Things are not where they were. One item gone from the chest you open first, found later on your bed, your crafting table, or at the end of your route. A torch you did not place. Footstep sounds after you stop. A sign with one word: `here`, `again`, `yours`. | Six events landed in it, or the stage's most days have passed. |
| 3 | **Habits** | It knows your day. The thing you reach for first is the thing that moves. `You may not rest now` with nothing in sight, on the nights you are tired. Your bed is one block over from where you left it. Crops trampled in a line toward your door. The animals stop and look at you. The lights go out over your head for ten seconds and come back. The sun is gone for a breath at noon. | Ten events landed in it, or the stage's most days have passed. |
| 4 | **Company** | The figure. Far off, in your skin, with your name over it at the limit of nameplate distance, walking a route you walked, at the hour you walked it. Looked at, it keeps walking; approached, it is gone around the next corner, and in the place it stood is the thing it took. Indoors, a chest opens as you enter the room. | The stage's most days pass. Whether the want was met is read as stage five begins: see [What it wants](#what-it-wants). |
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
- **How the target moves** (owner): pace and stops, how often they turn around,
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
- **What the target types in chat,** and what witnesses say within 32 blocks of them (owner). The memory keeps the last few lines about the lodger (lines that mention a
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
same door opens. Nearly three hundred small things, each once or twice, each at a time and place
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
| `chest_rattles` | 3 | While they are in the house, the lid of the chest they use opens and shuts five times fast, with its sounds, then is still; nothing inside changes (`open_lid`). | The chest they use, the target indoors. |
| `shuffle_chest` | 1 | The items in the chest the target uses most are in different slots. Same items, same counts. | A used container. |
| `stack_split` | 1 | One stack is two stacks, in two slots, the same total. | A used container with a stack of 2 or more. |
| `tidy_chest` | 2 | The chest is sorted by kind, which the target never did. | A used container of 10 or more stacks. |
| `tidy_by_colour` | 3 | Wool, concrete, glass or beds in a chest are sorted by colour, the way a child would. | A container with 6 or more coloured stacks. |
| `swap_stacks` | 2 | Two stacks trade places between two of the target's chests. | Two used containers. |
| `fill_gap` | 2 | One item the target has many of is in a chest it was never in, in a slot that was empty. | Two used containers. |
| `chest_full_of_one` | 3 | Every empty slot of the chest they use holds a single dirt, as many as it has dug (its holes), never one more. | A chest of theirs, three dirt or more dug. |
| `renamed_item` | 3 | One of their things in the chest they use has a name on it, as an anvil writes one: `mine`, `borrowed`, `not yours`; the model writes it. Their favourite, if it is there. | A used container with something in it. |
| `returned_used` | 3 | What it took comes back to its chest the very same item, but a tool worn a little more (never broken by it), anything else with a name on it. | Something held. |
| `loot_gift` | 2 | Opening a chest of fresh loot for the first time (a dungeon's, a village's, a temple's), one in five holds something of the lodger's among the loot, or in place of one loot item (that one kept, not lost): the thing it took from them, something it dug, now and then from stage four their own head, else a redstone lamp or a candle. Loot is the world's, so this is the one exception to "it never spawns anything" (owner). Armed by the brain, given by the host as the loot is made. | Fresh loot, the target opening it. |
| `chest_answers` | 2 | As the target opens a chest of theirs, another chest of the house closes, or opens and closes, at the same moment. Three times in ten, never from the menu. | Two chests of theirs. |
| `take` | 2 | One item of the kind the memory says the target uses most is taken from the container they open most. The lodger holds it. | A used container with a favourite in it, out of view. |
| `return_taken` | 2 | The held item is back where it was taken from, in its slot, or one slot over. | A held item. |
| `place_taken` | 2 | The held item is placed where the want says: on the bed, on the table, at the end of the route, at its place, as a dropped item that does not despawn or into a container there. | A held item. |
| `tool_used` | 3 | The favourite tool in the chest has lost durability overnight, a few dozen uses' worth, as if someone used it. | A damageable favourite in a chest. |
| `book_word` | 3 | The book and quill in the chest they use is a written book: one word on its page, the same word its title, signed by no one (the model writes the word). Never one they wrote in. | An empty book and quill in a chest of theirs. |
| `diary_sold` | 3 | A villager near them (never a baby, who cannot trade) sells a written book for one emerald, once: its diary of them, titled and signed with their own name. Its first page is a line the model writes in its quiet voice; the pages after are what it has seen them do, every one true (where they sleep, the chest they open first and how often, the way home, what they carry most, the door they use, their other places, how many things it did for them to find; never another player's name), and it ends "i am still watching." Bought by anyone, it is never brought. A villager that takes or loses a trade clears its offers: the book drops at its feet. Owner's idea. | A villager near them, outdoors. |
| `diary_brought` | 4 | The diary was on sale and never bought: a villager near them walks up, slowly, stops a step away, drops a newer copy at their feet and goes its way. Owner's idea. | The diary unbought, a villager near them, outdoors. |
| `letter_in_chest` | 2 | A written book is in the chest they use, titled "for" and their name, unsigned: a line from it, the model's, about something it did or saw. | The chest they use with a free slot, they away or asleep. |
| `ender_chest_note` | 4 | A note in its words, a written book "for" them, put into their own ender chest's inventory: the one place nothing else can reach (`write_book`, `put_ender_chest`). | An ender chest of theirs, the target in the game. |
| `eye_turns_home` | 5 | As they hold an eye of ender, the next one they throw (within ten minutes) flies toward their house instead of the stronghold, once (`eye_home`). Once a stay. | The target holding an eye of ender, the home known. |
| `villager_named` | 3 | The villager nearest them that had no name has one now, a word or two it chose (the model's), shown over its head as a name tag's is. One they named keeps theirs. | A villager near them. |
| `lectern_notes` | 4 | A lectern with no book holds one now (theirs, or another of theirs out of their view): its notes of them, a page for each day it did something they would find, the latest first, under a line of its own. | A lectern, they away or asleep. |
| `page_in_your_book` | 4 | A book and quill of theirs in the chest they use has one page more after their own, in its words; theirs stay as they wrote them. | A book and quill in the chest they use, they away or asleep. |
| `map_marked` | 3 | The map in the chest they use shows a red X at the lodger's place, or at the house itself when it has none yet. | A filled map in a chest of theirs. |
| `hopper_emptied` | 3 | What waited in their hopper is in the chest of theirs beside or under it. | A hopper they placed, next to a chest of theirs. |
| `shulker_moved` | 3 | Their shulker box is three to five blocks across the room, on the floor, with everything still in it. | A shulker box they placed. |
| `open_on_enter` | 4 | The chest the target uses most opens as they enter its room, and stays open until they look away. | A used container in a room with a door. |

### Furnaces and workstations

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `cook_one` | 1 | One raw item from the target's chest is in a furnace with one fuel, and comes out cooked. | A furnace, a chest with something smeltable and a fuel. |
| `take_output` | 2 | The finished output of a furnace is gone from the furnace and in the chest the target uses. | A furnace with output. |
| `swap_fuel` | 2 | The fuel in a furnace is a different fuel kind from the target's chests, the counts kept fair in burn time (three coal for sixteen planks). | A furnace with fuel, a chest with another fuel. |
| `cold_furnace` | 3 | A furnace mid-smelt has lost its fuel, which is in the chest. The smelting stops. | A lit furnace. |
| `item_on_table` | 2 | An item is laid on the crafting table as a dropped item that does not despawn: the held item, or one of the favourites from a chest. | A crafting table, a held item or a chest. |
| `lectern_page` | 2 | The book on a lectern is open to another page (`turn_page`, ABI 29). | A lectern with a book. |
| `anvil_sound` | 2 | An anvil's sound from the workshop, at night, while the target is in bed. | An anvil and the target asleep. |
| `glass_breaks` | 4 | While they are in the house, the sound of a window breaking, at one of its panes; every pane is whole (a sound). | The target indoors, a pane. |
| `brewing_done` | 3 | A brewing stand that had potions brews one of their ingredients into them, the game's own brewing: one ingredient from their chest into the stand, and a blaze powder too when the stand has no fuel (`brew`, ABI 29: refused unless the stand holds potions and its ingredient slot is empty). | A brewing stand and an ingredient in the chest they use. |
| `armor_stand_turn` | 3 | An armour stand's head is turned toward the door, or toward where the target sleeps. | An armour stand in the home. |
| `armor_stand_dress` | 3 | One piece of armour from a chest is on the stand. | An armour stand and a chest with armour. |
| `stand_watch` | 3 | The armour stands in the room turn their heads to follow the target while the target is in it, and hold the pose once the target looks straight at one. Back to normal when the target leaves. | An armour stand in the home. |
| `grindstone_strip` | 4 | An enchanted tool in a chest has lost one enchantment, as a grindstone would take it (`strip`, ABI 29: the host picks the item and the enchantment, never a curse). | A grindstone and an enchanted item in a chest. |
| `crafted_for_you` | 3 | Something they make lies finished on their crafting table, the makings gone from their chest: four torches from a stick and a coal, bread from three wheat, planks from a log, sticks from planks. Owner's pick. | A crafting table, the makings in their chest. |

### The house

Blocks the target placed, moved within reach, never broken for good.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `open_door` | 1 | A door or trapdoor of the home is opened while the target is away or asleep. | A home with a door. |
| `close_door` | 1 | A door the target left open is closed. | An open door in the home. |
| `furniture_shifted` | 2 | Their crafting table, furnace or barrel is one block over from where it stood, everything still in it; never a chest. | One of those of theirs. |
| `one_room_dark` | 3 | Every torch within five blocks of one of the home's lights is gone in the morning, six at most, and they are in their chest, every one. | Lights in the home, a chest of theirs. |
| `door_held` | 2 | A door they open shuts again a moment later, as if held, one time in four; never from the menu. It only ever shuts a door: forced with the door shut, nothing moves. | The target opening a door. |
| `door_wont_open` | 4 | Past `door_held`: the door of the house they walk to will not open at all for six seconds, as if held from the other side; each try rattles it shut on their screen with a blow on it, and three knocks come through it as they stand there (`hold_door`, ABI 54). | The target walking to a door of the home. |
| `trapdoor_taps` | 3 | A trapdoor of theirs near them taps open and shut eight times, a third of a second apart, and is as it was (`toggle_block`). | A trapdoor of theirs. |
| `chest_wont_open` | 4 | As they walk to the chest they use, it will not open for five seconds (a double chest both halves), each try a locked chest's rattle, and two knocks from inside it (`hold_container`). | The target walking to the chest they use. |
| `every_door_opens` | 5 | At night, while they are in the house, every door of it opens at once, each with its sound (`toggle_block` on each, left open). | Night, the target indoors, two doors or more. |
| `bed_turned` | 3 | Their bed is turned round where it stands, the head where the foot was. | A bed. |
| `door_hinge` | 3 | A door of the house they are in, theirs first, hangs on the other side of its frame (`set_state`, both halves). | A door. |
| `window_tap` | 2 | Three taps on a window pane from outside, at night. | A window and night. |
| `carpet_shift` | 2 | The carpets in a room are one block over. | 3 or more carpets in a room. |
| `stairs_turned` | 2 | One stair block the target placed faces another way. | A placed stair out of view. |
| `ladder_gap` | 3 | One rung from the middle of the home's tallest ladder is missing, and is set on top of the ladder. | A ladder of four rungs or more. |
| `flowers_potted` | 2 | Up to three flowers around the house are picked and are in the home's empty pots. | Flowers around the home, an empty flower pot. |
| `flowers_wilted` | 3 | While they are away or asleep, each flower they planted (six at most) is a dead bush; a day on, each is the flower it was again (`swap_block` there and back, each over its own kind only). | Flowers of theirs, the target away or asleep. |
| `path_worn` | 3 | A line of dirt path from the door toward the lodger's place, as if walked a hundred times: on the grass alone, found a block or two up or down where the ground slopes, passing over anything they built (a porch). | Grass between the door and the spot. |
| `cairn` | 3 | A small stack of cobblestone at the end of the target's route, from the lodger's own digging. | A route and held stone. |
| `stone_circle` | 4 | While they are away or asleep, fifty blocks from the house, any way, a ring of eight stone-brick pillars three high, five blocks out, stands on the ground; only into air. Once a stay. | The home known, the target away or asleep. |
| `path_through_the_trees` | 5 | While they are away or asleep, a straight line two blocks wide is cut through the leaves from forty-five blocks out to fifteen from the house, pointing at it: every leaf three to fourteen blocks over the ground along it taken away, leaves only, nothing else. Once a stay. | The home known, the target away or asleep. |
| `hole` | 3 | A one-block hole three deep at the lodger's place, with a torch at the bottom. | A spot on natural ground. |
| `alcove` | 4 | One block of natural stone behind a wall of the house, away from it, is hollow: a torch in it, and the thing it took from them if it holds one. Only natural stone; the one dig beside what they built (owner). | A wall of the house. |
| `backfill` | 3 | Up to five of the holes they dug by the house (natural ground within 16 blocks, newest first) are filled in again with dirt or stone it dug itself, never more than it holds. | A hole of theirs near the home, something dug to fill it. |
| `sapling` | 2 | A sapling from a chest is planted at the spot. | A sapling in a chest and a spot on grass. |
| `tree_grown` | 3 | The sapling by the house is a tree in the morning: bone meal from their chest (three at most), spent only if it grew. | A sapling around the home, bone meal in a chest of theirs. |
| `pumpkin_face` | 3 | A pumpkin by the house is carved where it grows, its face to the nearest door; the shears in their chest are a little worn. | A pumpkin around the home, shears in a chest of theirs. |

### What they kill

The senses report every mob the target hits and kills, and the memory keeps a tally by kind
(see [Memory](memory.md#what-is-kept)). The lodger has opinions about it (owner's idea:
"signs saying you like to kill, you killed y of z").

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `kill_sign` | 2 | A sign two blocks out from the door: `14 cows`, or `14 so far`. The count is the memory's. | Three or more kills of one kind, a door. |
| `grave` | 3 | Two cobblestone stacked at the lodger's place with a sign beside them naming the kind they kill most: `cow`, or `rest`. | Three or more kills of one kind, a home. |
| `your_grave` | 5 | At its place, on the ground there, a mossy stone and a sign beside it: "here lies", their own name, and a day three days on, not come yet. | A home, the target away or asleep. |
| `torches_turn` | 4 | Overnight every wall torch of the house is on another wall of its block, each pointing as nearly at their bed as a wall there lets it, lit as it was (`turn_torches`, ABI 55). | A home with a bed and lights, the target away or asleep. |
| `pale_moss_creeps` | 4 | Overnight pale moss creeps across their floor from one corner of the room: three blocks the first night it comes, three more each night after, onto the floor alone, as a house given over to the pale garden. | A home, the target away or asleep. |
| `sand_gives_back` | 4 | By the house, a patch of suspicious sand in the ground; brushed, it gives back the thing it took from them (`keep_in`, ABI 55). | Something of theirs held by it, a home, the target away or asleep. |
| `pot_of_returns` | 4 | A decorated pot stands at its place; broken, it gives back the thing it took from them (`keep_in`, ABI 55). | Something of theirs held by it, a home, the target away or asleep. |
| `crafter_clicks` | 3 | Their crafter clicks and crafts once by itself, its item dropped out of its face, as if a pulse no one sent had reached it (`toggle_block` on a crafter, ABI 55). | A crafter of theirs. |
| `cobwebs` | 3 | Overnight the upper corners of their house are hung with cobwebs, up to six, in the air just inside the walls they built, as if no one had lived there for years; a day on they are gone. | A home, their building, the target away or asleep. |
| `false_door` | 4 | Overnight, a spruce door on an inside wall of their house where there was none, flat against the wall they built; open it and there is their wall behind it. A day on it is gone. | A home, their building, the target away or asleep. |
| `red_trail` | 4 | As they come home, a trail of red specks on the floor from the door to their bed, there a minute and gone (`red_trail`, a client hint). | A home with a bed and a door, the target coming to it. |
| `snow_shape` | 4 | In the snow by the house, a hollow the shape of a person lying with their arms out, the snow layer taken off where it lay, as if one lay there all night. | A home in snow, the target away or asleep. |
| `cold_house` | 4 | The house goes cold a minute: their breath shows in puffs before their face, and frost lies on a few blocks of the floor, gone with it (`cold_breath`, a client hint, and snow it lays and takes away again; off with `events.glitches`). | A home, the target indoors. |
| `frost_on_the_glass` | 3 | Their windows frost over from inside, every pane white, a minute and a half, though it is not cold; then clear again, the very panes as they were. | A home with windows, the target indoors. |
| `stairs_up_to_nowhere` | 4 | Overnight, a flight of oak stairs up from their floor along an inside wall of theirs, a step a block, into the ceiling, going nowhere; a day on, gone. | A home, their building, the target away or asleep. |
| `face_in_the_canopy` | 4 | In the tree nearest the house, the leaves cut away three deep in a face, two eyes and a wide mouth, looking at their door; leaves alone, never the trunk. | A home with a door, a tree within twenty-four blocks, the target away or asleep. |
| `boots_at_the_door` | 4 | As they come home, just inside the door, a pair of boots like the ones they wear, standing as if someone had just taken them off and come in; five minutes on they are gone. An unseen stand wears them, its things never theirs to take (`place_entity`'s `wears` on a stand, ABI 55). | A home with a door, the target coming to it. |
| `kill_line` | 3 | A chat line in the target's own name about it, at night: `i saw what you did to the cows`, `the cows remember`, `14 cows`. | Three or more kills of one kind, night. |

### The house, changed

Owner's idea: "trying to sleep breaks the bed", "replace certain blocks of the
house". The one place the lodger changes what the target built, and only into the same thing
slightly wrong, never gone: a block becomes another member of its family (`memory::family`:
every wood for planks, logs, fences, slabs, stairs and trapdoors; every colour for wool,
concrete, terracotta, carpets and stained glass; the stones; the bricks; the torches; the
lanterns; panes and iron bars), picked by the stay's seed, and the bed comes back as its item.
Nothing with contents, no door, nothing of the ground. (The owner: "shouldn't be
hardcoded, why only spruce"; it was a table at first.)

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `wall_swap` | 3 | One block of the house is another of its family: planks of another wood, cobblestone for stone bricks or mossy cobblestone, wool of another colour, glass tinted. Only blocks the target placed and the memory knows the kind of. | A placed building block with a family, the target away or asleep. |
| `torch_blue` | 3 | One of their torches or lanterns is another kind: a soul torch burning blue, a redstone torch's dim red, a soul lantern. | A placed torch or lantern. |
| `window_bars` | 4 | A window pane is iron bars. | A placed glass pane. |
| `bed_breaks` | 4 | As they lie down, the bed breaks under them and drops as its item where it stood. Once a stay. | The target in bed at night. |
| `choke` | 4 | As they sleep, the figure comes in at their door, which opens (heard, and left open for them to find; passed over if it is no door now), and glides slowly to the foot of the bed, its legs moving; there, in their sight down the bed, it stands a moment, takes out a lead, its rope between its hands, and looks down at it, then up at them; then it walks round to the bed's side by the pillow, the rope still between its hands (straight to the side when the foot has no room); the rope's end goes from its hand to their chest, across their view, and it chokes them for five seconds, a heart a squeeze, never below half a heart: they stay lying, the night held, their eyes kept open (no fade to sleep), each squeeze a drowning gasp, the hurt's jolt and the dark closing in. Leave Bed breaks free and it is gone; otherwise they wake as it ends (`choke`, ABI 54, which keeps their eyes open itself with the `eyes_open` hint, whatever `events.glitches` says). Twice a stay at most. | The target in bed at night, the bed and a door known. |
| `shot_in_bed` | 4 | The same walk to the bed, the door opened and left so, their eyes kept open; at the bed's foot it takes out a crossbow and loads it as a player would, looking down at it, raises it at them and shoots once from there: two hearts, which wakes them, never below half a heart. It watches them two seconds and is gone (`choke`'s `how` `shoot`). Twice a stay at most. The owner: "crossbow should just shoot you from the end of the bed". | The target in bed at night, the bed and a door known. |
| `burned_in_bed` | 4 | The same walk to the bed's foot, the door opened and left so, their eyes kept open; there it takes out a flint and steel and looks down at it, then up at them; then it walks round to the pillow and strikes it at them: they burn five seconds and wake, never below half a heart through the fire. It watches them two seconds and is gone (`choke`'s `how` `light`). Twice a stay at most. The owner's: "a fire event ... use flint and steel for the fire one". | The target in bed at night, the bed and a door known. |
| `lies_beside` | 5 | The same walk to the bed as `choke`, the door opened and left so, their eyes kept open and the night held; at the bed it lies down on the floor beside it, on its side, facing them, and does nothing more; a few seconds on they wake and it is gone (`choke`'s `how` `lie`). | The target in bed at night, the bed and a door known. |
| `sleepwalk` | 5 | Asleep at night, they wake standing two blocks out of their front door (out by the side of it the sky is over, as the host reads each door), the door open behind them, and where there is snow their prints come from the bed (`cancel_sleep`, `move_player`, `prints`, the door last). Once a stay. | The target in bed at night, the bed and a door known. |

### Lights and fire

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `move_light` | 1 | A torch, lantern or candle the target placed is removed and placed within three blocks, the same light kept. | A placed light not in view. |
| `add_torch` | 2 | One torch where there was none, in the darkest corner of the home. The lodger's own torch, which the memory counts. | A dark spot in the home. |
| `light_path` | 2 | Torches, three to seven, from the home toward the want's spot, placed while the target is away; a spot with no footing within eight blocks up or down is skipped, the rest still laid. | A home and a spot. |
| `torch_ring` | 4 | While they are away or asleep, sixteen torches in a ring twelve blocks out round the house, each finding its footing near the floor's height. Once a stay. | The home known, the target away or asleep. |
| `put_out_campfire` | 2 | A campfire is out. | A lit campfire. |
| `light_campfire` | 2 | A campfire is lit. | An unlit campfire. |
| `candles_lit` | 3 | Every candle in the house is lit at night, or every lit one out. | Candles in the home. |
| `cauldron` | 2 | A cauldron is full that was empty, or empty that was full. | A cauldron. |
| `lantern_ahead` | 3 | Far down a cave the target has not entered, one of their lanterns, lit: set down at the cave's dark place farthest from them, then taken from their chest (a lantern that finds no floor stays in the chest). | A cave they have not entered (`Need::Cave`) and a lantern in the chest they use. |
| `warped_animal` | 3 | One of the target's animals (tamed, or by the home), or one outside when they keep none near, 12 to 32 blocks from them, goes wrong for a minute: it freezes with its head always on them, its model bent on their screen (the whole body stretching tall and thin, or squashing low and wide, its volume kept, grown in over a few seconds and breathing slowly, every part trembling, the head tilted over and twitching now and then: `warp_models`), and is one or two of too tall (1.5 to 2.2 times, the height the stretch grows to), upside down (the game's Dinnerbone flip, never on an animal with a name of its own) and a step closer along the ground each time they look away; one time in three it stands just outside the door they use first. Walked up to within six blocks, it is at once as it was, where it was (`warp_mob`, ABI 25: a mob saved while warped is put back when next seen). Owner's idea. | An animal 12 to 32 blocks from them, they outdoors. |
| `lights_out` | 4 | At night, the target's own lights behind them as they go (within 20 blocks, out of view, nearest first, up to five) go out one by one, three quarters of a second apart, and each is back a minute after it went, as it was (`hide_block`, ABI 24: put back at the time, or at once when the level saves or closes). Owner's idea. | Night, and lights they placed behind them. |
| `soul_swap` | 3 | One of the target's torches or lanterns, the farthest out of view, burns with soul fire from then on, the same shape where it hangs (`swap_block`, ABI 24, keeps a wall torch's facing). Owner's idea. | A torch, lantern or campfire of theirs out of view. |
| `death_light` | 2 | A candle, lit, on the ground where the target last died, once for each death. Owner's idea. | A death of theirs no candle of its own marks. |
| `light_taken` | 3 | The torch or lantern the target placed last is gone, and the same light stands six to ten blocks on along the way they went (their trail, or the route away from the house), against a wall if nothing else holds it. Owner's idea. | A light they placed and have not broken, a way on from it out of view. |
| `cave_lit` | 3 | A cave near the home the target has not entered has torches in it, up to six, from its near end inward, at the spacing the target uses in their own tunnels (the middle of their underground lights' gaps, 4 to 14 blocks; 8 before they have lit one). Its own torches. | A cave they have not entered. |

The caves come from the brain's cave read (`core/src/caves.rs`): every ten minutes while the
target is above ground within 48 blocks of home, a scan around them of 97 by 41 by 97 blocks
read every third block (`step`, ABI 23) for air with no sky light and no block light (`dark`):
since 1.18 caves are mostly plain air, not `cave_air`. A cave is a crowd of six such places
within 8 blocks; it is unentered when none of it is within 10 blocks of where they walked or
stood, 6 of what they built, 16 of a site of theirs, or the home's radius.

### Beds and rest

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `shift_bed` | 3 | The bed is one block over, to its side, both halves, colour and facing kept, while the target is away (`shift_bed`, ABI 26: never a bed someone lies in, only onto solid floor). | A bed and room to move it. |
| `turn_bed` | 3 | The bed faces another wall. Coded as batch 3's `bed_turned`. | A bed and room to turn it. |
| `sheets_changed` | 3 | Their bed is another colour when they come home, both halves, same place and facing (`swap_block` turns a bed's two halves together). Owner's pick. | A bed, the target away or asleep. |
| `under_the_bed` | 3 | As they lie down, a slow scratching right under their bed, three or four beats, then nothing: silverfish steps, knocks on wood, or wool and claws, one of three. Owner's pick. | A bed, the target in it. |
| `foot_of_bed` | 4 | As they wake, the figure stands at the foot of their bed, across it from them, for half a second. One waking in three, its rules allowing; never offered, the waking starts it. Owner's pick. | A bed, the target waking. |
| `second_bed` | 4 | A second bed stands beside theirs when they come home, the same facing, another colour (`bed_beside`, ABI 26). Once a stay. Owner's pick. | A bed and room beside it, the target away or asleep. |
| `refuse_rest` | 3 | The next attempt to sleep is refused with vanilla's own `You may not rest now` message. Once per night. | The target trying to sleep. |
| `bed_occupied` | 3 | `This bed is occupied`, vanilla's message, with nobody in it. The next try works. | The target trying to sleep. |
| `wake` | 3 | The target is woken in the middle of the night, sleep cancelled, and the door is open. | The target asleep. |
| `woke_elsewhere` | 4 | The target wakes one block from where they lay down, on ground, never over a drop or in water. | The target asleep and room beside the bed. |
| `other_house` | 4 only | Once a stay, asleep in stage four, they wake in a copy of their house in a void dimension of its own (`lodger:other`): block for block, its lights left out, a black sky, the doors opening onto nothing (an unseen floor runs on, so no one falls), the figure in the doorway. After a minute they wake in their own bed, their door open, with what they left there (in its chests, on a lectern, dropped, thrown or shot; a death there waits for the respawn, its drops come home). Its crops and the plants in its pots are left out, so nothing of the copy comes home; one player there at a time. The second and last visit is the take ending's last night, the figure walking at them from the copy's doorway; the death that night stays at its place, in their own world. Owner's design. The world keeps the dimension: a world opened again without the mod warns of it. | The target asleep, a bed, a door and a home. |
| `egg_on_bed` | 2 | An egg from the pen is on the bed. | Chickens and a bed. |

### Decoration and signs

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `frame_rotate` | 1 | An item frame's item is rotated one notch. | An item frame with an item. |
| `frames_turn` | 3 | Every item frame in the room out of their view is turned to the same orientation, while the target is away (a frame or a stand it turns or dresses is never one in front of them). | Three or more item frames in a room. |
| `frame_swap` | 3 | The item in a frame is traded with a thing of one from their chest: the very stacks, the frame's into that slot (`set_frame_item`'s `swap`, ABI 40). | An item frame and a chest. |
| `item_frames_face` | 3 | While they are indoors, every item frame of the house shows the same thing, to them alone, for three minutes: what it took, or else what they hold most; then what each really holds again (`frames_show`, ABI 38: the frame's item as their client sees it; nothing in a frame changes). | An item frame, indoors. |
| `frame_emptied` | 3 | An item frame is empty; its own item is on the bed, the very stack (`set_frame_item`'s `drop_at`, ABI 40). | An item frame with an item and a bed. |
| `painting_change` | 2 | A painting is another painting of the same size. | A painting. |
| `sign` | 2 | A sign with one word from the want's list, at the spot or by the door. | A spot. |
| `sign_word` | 3 | One line of a sign the target wrote reads one word of its own instead: the last line with text (`set_sign`'s `one_line`, ABI 28: a blank sign is refused). The other lines are kept. | A sign they placed. |
| `count_up` | 3 | Its kill sign's count rises by itself: each time they kill another of the kind it counts, the sign reads the new number (`15 cows`). Never offered; the kill starts it. Owner's pick. | Its kill sign, and a kill of that kind. |
| `weapon_on_wall` | 3 | The weapon they use most hangs in an item frame of theirs, moved straight from its chest slot (`set_frame_item`'s `from`, ABI 28: only into an empty frame, so nothing is lost). Owner's pick. | Their weapon in a chest and a frame of theirs. |
| `raises_their_sword` | 4 | Outdoors, thirty-five blocks ahead of them, a figure holds their weapon (a plain one of its kind), raising it again and again, facing them; gone once seen and looked away from (`figure`'s `swinging`). | Their weapon known, the target outdoors. |
| `head_on_post` | 4 | A fence post two blocks out from the door they use, and on it the head of the kind they kill most (a zombie's, a skeleton's, a creeper's, a piglin's), facing the door. Owner's pick. | A kill of a kind with a head, a door. |
| `grave_each` | 4 | A row of graves at its place, two blocks apart: for each kind they killed, most first and five at most, a stone and a sign with the count and the kind. Once a stay. Owner's pick. | Two kinds killed, a home. |
| `banner_turned` | 2 | A banner faces another way. | A placed banner. |
| `named_animal` | 3 | One of the target's animals has a name, one word from the want's list. | A penned animal without a name, out of their view. |
| `plant_flower` | 2 | A flower from a chest is planted by the door or on the route. | A chest with a flower. |

### Farm and animals

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `harvest_one` | 2 | One ripe crop is harvested and its yield is in the chest: wheat, a carrot, a potato or a beetroot, whichever it was (the host says what it cleared). A crop not yet ripe is never cut. The soil is left as it was. | A field and a chest. |
| `replant_other` | 3 | One young crop of wheat is a different crop, from seeds in a chest (never a ripe one). | A field and a chest with other seeds. |
| `trample` | 3 | Farmland from the field toward the door, one line, while the target is away. | A field within 32 blocks of the home. |
| `field_word` | 4 | Crops cut back to seedlings so that from above, north at the top, the field reads one word of the want's list in letters three by five: the longest that fits (`MINE`, `LOOK`, `GIVE`; `ME` in a field of 7). Coded from the brain's field read. | A field of 7 by 5 or more, four in five planted. |
| `scarecrow` | 3 | An armour stand in the field, wearing one piece of the target's armour (the very piece, from their chest), facing the house: their own stand moved there, or one from the chest set down. | A piece of armour and an armour stand (or one in a chest), and a field. |
| `open_gate` | 2 | A pen's gate is open in the morning. The animals wandered. | A fence gate with animals behind it. |
| `animal_indoors` | 3 | One of the target's animals is inside the house in the morning. Moved, not spawned; the gate is closed behind it. | A penned animal and a home with a door. |
| `bats_in_the_house` | 4 | At night, while they are in the house, four bats flutter about inside it, two blocks over the floor round its middle, a minute; then gone, leaving nothing (`place_entity` of bats for a while). | Night, the target indoors. |
| `animal_gone` | 3 | One animal is missing from the pen, and is at the end of the target's route, grazing. | A pen and a route. |
| `sheep_dyed` | 2 | One of their sheep by the house is another colour (no dye spent: a sheep's wool is not an item). | A sheep of theirs by the house, out of their view. |
| `horse_moved` | 2 | The horse is tied to a different post, within eight blocks of the first (`move_leash`, ABI 27). | A horse (or donkey, mule, llama, camel) on a lead. |
| `face_the_house` | 4 | At dawn every animal in their pens (up to twelve) stands still facing the house for two minutes (`hold` with `at`, ABI 27). Owner's pick. | Three animals in the pens, dawn. |
| `one_more` | 3 | In the morning one more of the kind they keep most stands with the others, and is gone by noon (`place_entity` for a while, ABI 27: gone at once when the level closes). It gives nothing while it is here: it cannot be hurt, a sheep comes shorn, a chicken lays no egg, it cannot breed, and never a mooshroom. Owner's pick. | Two animals in the pens, dawn. |
| `row_withered` | 3 | One row of their crops is back to seedlings; the rest untouched. Owner's pick. | A field, the target away or asleep. |
| `eggs_to_door` | 2 | Eggs in a line from the coop to the door they use, two blocks apart, up to six. Owner's pick. | Two chickens by the home and a door. |
| `breadcrumbs` | 4 | At night while they are away or asleep, six of the most plentiful block in their chest, taken one at a time and each kept, lie one by one three blocks apart from their door away from the house: a line to follow, theirs to pick up (`take_item` with `keep`, `drop_item` of each). Once a stay. | A block of theirs in their chest (six or more), a door, night, the target away or asleep. |
| `animals_nip` | 3 only | The nearest farm animal turns on them: it runs at them and nips, half a heart, twice at most, and never below half a heart (`charge`, ABI 27). The owner's harm stages: a nip at 3. Owner's idea. | A farm animal within 16 blocks, the target outdoors. |
| `animals_turn` | 4 | Up to three of the nearest farm animals turn on them for half a minute, a heart and a half a hit, three hits each at most, never below half a heart: real harm from stage 4, never a kill. Owner's idea. | Farm animals within 16 blocks, the target outdoors. |
| `hostiles_stare` | 3 | At night every hostile mob within 48 blocks (up to sixteen) stops where it stands and turns to the target, and stays so, facing them as they move, attacking no one, until dawn burns or frees them. Hit once, a held mob takes it unmoved, still staring; hit twice, that one wakes alone and acts as it would (owner). None is spawned: a night with few has few. Owner's idea. | Three or more hostile mobs near, night. |
| `hostiles_close` | 4 | At night in the open the hostile mobs near (up to sixteen) hold and stare as in `hostiles_stare`, and each moment the target looks away, one comes a step nearer along the ground, to six blocks; the moment they look at it, it is back where it stood (`hold`'s `creep`, ABI 37), so what they glimpse turning is never there when they face it. Three minutes, then every one back where it stood and as it was; one saved mid-creep goes back when next seen. | Three or more hostile mobs near, night, outdoors. |
| `pack` | 3 | Their farm animals near them (three or more, up to twelve) turn and walk toward them together, at a walk, and stop four blocks off, facing them, for most of a minute; then they graze as before (`approach`, ABI 37: only their path and look are steered, nothing changed). | Three of their farm animals within sixteen blocks, outdoors. |
| `slow_approach` | 3 | One animal near them (8 to 28 blocks), half the time with its model bent on their screen (stretched whole, trembling, its head tilted and twitching, as `warped_animal`'s), walks at them very slowly, its eyes on them, and stops a block and a half off; then it stares five seconds and goes its way, or turns and walks off, or strikes once (half a heart, a nip, never below half a heart) and walks off (`approach`'s `pace`, `then`, `damage`, ABI 44). Owner's idea. | Three animals near, outdoors. |
| `quick_approach` | 3 | The same, at a run: one animal near them comes at them fast, its eyes on them, and stops a block and a half off; then it stares, turns away or strikes once (owner: "the opposite event too"). | Three animals near, outdoors. |
| `stare` | 3 | Every animal and villager within 32 blocks stops, turns to the target and holds still looking at them for ten to thirty seconds (`hold`: they stand still rather than look and walk at once, which jittered). Never a hostile mob: that would be a fight. | Three or more animals or villagers nearby. |
| `pet_growl` | 2 | A tamed wolf growls, or a cat hisses, facing an empty spot in the house, for a few seconds. | A tamed pet in the home. |
| `pet_moved` | 2 | A sitting pet is sitting somewhere else, facing a corner. | A sitting pet in the home. |
| `pet_outside` | 3 | The pet is outside the closed door in the morning. | A pet indoors and a door. |
| `bees_out` | 3 | At night the bees of their hive are out, hanging still around the door they use for two minutes, then home (`release_bees`, ABI 27). | A beehive of theirs and a door, night. |

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
| `answers_you` | 2 | What the target types is sometimes answered, 5 to 60 seconds later, in their own name, to them alone: often a line about it (the house, a door, `who`, a question, its want's words), rarely anything else (60 and 15 in a hundred), at most three a day and five minutes of play apart. The model writes it with their words in its prompt, cleaned to letters (it may echo one: `house?`), from the reply bank's examples (`i heard you`, `inside`, `not yet`); the bank stands in. Started by the chat line, never from the menu; it may follow itself. Owner. | The target typed, `events.chat`. |
| `says_back` | 3 | When the target types something about the lodger (the senses' chat line matches the want's words, `house`, `door`, `chest`, a question), a chat line in the target's own name, to the target alone, from the bank: `i know`, `it was me`, `look behind the door`. Once a day at most. | The target typed. |
| `says_first` | 4 | A chat line in the target's name, unprompted, at night: one word from the want's list, or a bank line. | Night. |
| `whisper_from_you` | 4 | A whisper to the target from the target: `<you> whispers to you: here`. | Any. |
| `repeats_you` | 4 | The target's last chat line, said again, in their name, a minute later. The words are the target's own; nothing is added. | The target typed. |
| `sleep_message` | 3 | `1/2 players sleeping` on the action bar, the game's own words, at night with the target awake and alone. (`sleepers_wrong` says it as they lie down.) | Night, the target awake. |
| `advancement` | 4 | The chat line and the toast for one the target has, as if just earned, in their name (`game_message` `advancement`: the host picks one they have that announces itself; the toast is the client's own). | The target awake, with an advancement. |
| `death_message` | 4 | One of the game's own death lines about them while they live: `<you> fell from a high place`, `drowned`, `suffocated in a wall`, `fell out of the world`, `died`, `starved to death`, `froze to death`. Owner's pick. | The target awake. |
| `words_on_sign` | 3 | What they last typed, in their own words, lower case, on a sign beside the door they use, up to four lines of fifteen. Owner's pick. | They typed, a door, the target away or asleep. |
| `ping_fades` | 3 | Their own entry in the player list shows no connection for a minute, then their own again. Owner's pick. | The target awake. |
| `empty_message` | 3 | `<you>` in their chat with nothing after it. Owner's pick. | The target awake. |

### Villages

Offered only with a village within 128 blocks of the home.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `scan_village` | 2 | Nothing the target sees: the lodger looks the village over once, so it knows its doors and its bell. The host's scan, forty blocks around the village's middle. | A village in the surroundings. |
| `bell` | 2 | A bell rings once, at night. Witnesses hear this one: it is a real bell. | A bell within 48 blocks. |
| `villager_at_door` | 3 | A villager stands outside the target's door at night, facing it, and is back home by morning. | A villager and night. |
| `golem_turned` | 3 | The iron golem faces the target's house all night and all the next day, wherever it stands. | An iron golem. |
| `village_doors` | 3 | Every door in the village is open in the morning. | A village with 4 or more doors. |
| `village_lantern` | 2 | A village lantern is gone, and stands on the target's route where it passes the village (set down there first, so one that finds no ground stays where it hung). | A village lantern and a route. |
| `villager_word` | 4 | The villager nearest them sells what the lodger took (the very item, `offer_trade`, ABI 31) for one emerald, once; bought, the trade is gone, and so is the lodger's hold on it. | Something it holds, a villager near. |
| `village_abandoned` | 4 | While the target is 64 blocks or more away at night, the village empties: every villager within 48 blocks of its middle walked into the house whose door is nearest the middle, packed inside, facing its back wall, silent, turning to the target as they come within five blocks (`gather_villagers`, ABI 31); every other door open, the lanterns out (`hide_block`). By dawn, everyone is back where they stood, the doors shut, the lights lit. Once a stay. Owner's idea. | The village's doors read, night, the target far from the village. |
| `bell_twice` | 2 | They ring a bell; a second and a half later it rings again, once, swinging, half the times they ring it (`rang_bell`, ABI 31; never offered, the ringing starts it). Owner's idea (G1). | The target ringing a bell. |

### Water and boats

Offered with water within 32 blocks of the home or the route.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `boat_moved` | 2 | The target's boat is on the other bank, or their minecart at the other end of its track. | A boat or minecart the target used. |
| `prints_to_the_cave` | 4 | While they are away or asleep, prints in the snow from their door toward the nearest cave they know of, forty-eight blocks at most (`prints`). Once a stay. | Snow near the home, a door, a cave known, the target away or asleep. |
| `boat_adrift` | 3 | The boat is in the middle of the water, still, going nowhere. | A boat and open water. |
| `boat_drifts` | 2 | While the target watches from the shore, the boat drifts off on its own, slowly, away from them. (Owner's idea: "boats and minecarts move on their own".) | A boat on the water and the target outdoors. |
| `boat_waiting` | 3 | The boat is at the shore the target is walking toward, before they get there: on their side of the water, a block in from its edge. | Their boat, water within 48 blocks, the target walking. |
| `bucket_floor` | 3 | One block of water in the house's floor, in a corner. | A bucket in a chest and a floor block out of the way. |
| `ice_path` | 3 | A line of ice across the water from the far shore toward the house, in summer. | Still water of 8 or more blocks across. |
| `ripples` | 3 | Ripples and small splashes cross the water toward the shore on their side, in a line, as under footsteps, for the target alone (the client's `ripples` hint). | Water within 32 blocks, the target outdoors. |
| `fishing_line` | 4 | Fishing with no one there, faked (a real bobber needs a player holding the rod; owner): a cast, the bobber's splashes and a catch over a minute at one spot on their water, the fishing ripples there for the target alone (the client's `bobber` hint). | Water within 32 blocks and a fishing rod in a chest. |
| `fish_on_line` | 2 | They fish: the next time their bobber sits in the water, within two minutes, it splashes and goes under and stays under for ten seconds, on their screen; the server's own hook is kept from any real bite meanwhile, so reeled in it brings nothing (`bobber`, ABI 38). | A fishing rod in hand, water within sixteen blocks. |
| `fishing_figure` | 3 | Where they last fished (the water near them, if they never have), a figure stands with a fishing rod, casting now and then; gone once seen and looked away from. | Water near them, outdoors. |
| `ghast_name` | 3 | In the Nether, for five minutes, every ghast's cry reads in the subtitles as their own name, and one cries now, far off, so it is heard (the subtitle hook). Done by the Nether's stay on the overworld's word, counted at home (`elsewhere`). | The target in the Nether. |
| `ender_holds` | 3 | In the End, the enderman nearest them holds the block they build with most at home (`carry_block`, ABI 39); it may set it down as endermen do. Done by the End's stay on the overworld's word. | The target in the End, an enderman near, a block they build with. |
| `busy_away` | 3 | Coming back from another dimension after three minutes or more, one of its bold things happened at home while they were gone: the first of its house's share, its pillar, the hut on their route, the dock, the bridge, a cairn, a light path, a worn path, a grave, a sapling whose rules and lasting needs hold, run at once, so it was there before they could see it. | They just came back from another dimension. |
| `boat_in_house` | 4 | Their boat sits in the middle of the house in the morning. Owner's pick. | Their boat and a home, the target away or asleep. |
| `under_the_water` | 3 | A dark shape, a squid, hangs still under the water on their side, facing them, for twenty seconds, then is gone (`place_entity` with `face`, ABI 30). Owner's pick. | Water within 24 blocks, the target outdoors. |
| `wet_steps` | 3 | Wet steps and a drip with each walk from the door they use to their bed while they are inside. Owner's pick. | A bed and a door, the target indoors. |
| `drip` | 1 | Dripping water, indoors, with no water near. | The target indoors. |

### Snow, sand and weather

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `trail` | 3 | Footprints in the snow by the home, from the door toward the lodger's place, that the target did not make: a snow layer taken off at each step, left and right (`prints`, ABI 30; snow only, sand cannot show them). Gone with the next snowfall. | Snow around the home and a door. |
| `trail_in` | 4 | Footprints leading *to* the door, from twenty blocks out in the dark, and none leading away. | Snow around the home and a door. |
| `snowman` | 3 | Two snow blocks and a carved pumpkin stacked beside the door, facing it, not a golem. Owner's pick. | Snow around the home and a door. |
| `name_in_the_snow` | 4 | While they are away or asleep, ten blocks south of the house, their name ("you" before it is known) trodden into the snow in letters three blocks wide and five long (ten letters at most): the snow taken off where the letters are, snow only (`place_blueprint` of air over snow). | Snow near the home, the target away or asleep. |
| `snow_cleared` | 2 | The snow is gone from the path between the door and the route, one block wide. | Snow layers on the path. |
| `snow_indoors` | 3 | Snow falls inside the house for a few seconds, for the target alone. | The target indoors, snow outside. |
| `rain_indoors` | 3 | The same with rain. | The target indoors, rain outside. |
| `lightning_silent` | 3 | A flash of lightning with no thunder, in clear weather, for the target alone. | The target outdoors at night. |
| `thunder_clear` | 2 | Thunder, for the target alone, under a clear sky. | The target outdoors. |
| `fog_close` | 3 | The mist rolls in for a minute, then lifts: the real fog of `mist`. | The target outdoors. |

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
| `mine_beside` | 3 | While the target is away, a copy of their newest tunnel is dug three blocks beside it, two high, lit at their torch spacing; the ore it meets goes to the chest they use as it is dug, the stone to the lodger's store. | A tunnel of theirs of ten blocks, a chest they use. |
| `mined_out` | 3 | The ore vein ahead of their newest tunnel (within 16 blocks past its end, from the ore read) is dug out while they are away, a torch where it was. | Ore ahead of a tunnel of theirs. |
| `tunnel_sealed` | 3 | The mouth of their oldest tunnel is walled, floor and head, with cobblestone from the lodger's store (digging stone gives it cobblestone, as a pick does). | A tunnel of theirs and two cobblestone dug. |
| `ladder_down` | 3 | A one-block shaft dug down from its place, as deep as their ladders (four to twelve), the ladders up its side, each taken from their chest as it goes up. | Ladders in the chest they use, a home. |
| `minecart_rolls` | 2 | The target's minecart rolls out of the tunnel on its own and stops at the mouth. | A minecart on a track. |
| `rides_the_cart` | 4 | A minecart of theirs rolls off away from them with a figure sitting in it, on their client alone; the figure gone once anyone gets in or its time is up (`figure`'s `rides`, `push_entity`). | A minecart of theirs near them, outdoors. |
| `tunnel_toward` | 4 | While the target is in their tunnel, digging comes toward them from 25 blocks past its end, a column every four seconds, each heard as it breaks, and it breaks through just ahead or stops one block short. Built to be heard and found: out of the view rule. | The target in a tunnel of theirs. |
| `tunnel_goes_on` | 3 | Their newest tunnel is five blocks longer when they come back to it, their size, unlit. Owner's pick. | A tunnel of theirs, the target away or asleep. |
| `torches_far_end` | 3 | While they stand in their tunnel, its lights go out one by one from the far end toward them, a second apart, back in five minutes. Owner's pick. | The target in a tunnel of theirs with two of their lights. |
| `sign_deep` | 3 | A sign at the far end of their newest tunnel, one word of its own. Owner's pick. | A tunnel of theirs, the target away or asleep. |
| `way_back_closes` | 4 | Gravel falls into their tunnel five blocks behind them, floor and head, closing the way they came (they can dig it). Owner's pick. | The target in a tunnel of theirs with room behind them. |
| `portal_sound` | 3 | The sound of a Nether portal in the house, with none near. | The target indoors. |
| `portal_lit` | 3 | A portal frame they built (ten obsidian or more placed) is lit while they are away (`light_portal`, ABI 32), and their flint and steel is a use the worse. The lodger does not go through. | A frame and flint and steel in the chest they use. |

### Redstone

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `lever` | 2 | A lever in the home is flipped, and whatever it powers happens: a lamp, a door. Never a lever with TNT within 16 blocks of anything it powers; the host checks. | A lever. |
| `note` | 2 | One note from a note block, at night, while the target is awake. | A note block within 32 blocks of the home. |
| `note_tune` | 3 | Three notes from the note block, the first three of a disc the target owns. | A note block and a disc in a chest. |
| `jukebox` | 3 | A disc from the target's chest is playing in their jukebox as they come to the door, or the one playing stops, its disc back in the chest (`jukebox`, ABI 29). | A jukebox and a disc in the chest they use, the target coming to the door. |
| `door_opens_itself` | 3 | A door opens as the target approaches, with no plate and no one there. | A door in view. |
| `plate_laid` | 3 | A wooden pressure plate from a chest, in front of the chest the target uses. | A plate in a chest. |
| `piston_once` | 3 | A piston in the base extends and retracts once, at night, with its sound. Nothing is hooked to TNT. | A piston. |
| `lamps_noon` | 3 | The redstone lamps are on at noon and off at night, for a day. | Redstone lamps. |
| `lamps_blink` | 3 | At night their redstone lamps, up to six, blink together: three short, three long, three short, once, each ending as it was. Owner's pick. | Redstone lamps, night. |
| `composter_fills` | 2 | Their composter is one level fuller in the morning, nothing put in (`set_state`'s `@up`, ABI 29, never past full). Owner's pick. | A composter, dawn. |
| `bell_rings` | 3 | A bell of theirs rings once at night, swinging, with no one near it (`toggle_block` rings a bell). Owner's pick. | A bell, night. |

### Sounds

Every sound event has three to five ways to sound (`brain/sounds.rs`: the sounds, a pitch, a
rhythm, and whether it stays put, comes from behind, comes closer or goes away), never the same
way twice running; footsteps and placing are the sound of the real block underfoot or in hand
(`block_sound`); the house's sounds come from a real door or chest of the house the target is
in, theirs first (owner).

Heard by the target alone. Nothing in the world changes.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `steps_above` | 2 | Footsteps cross the floor above the target, three blocks up, wood or wool underfoot, slow or quick; there is no floor above, or no one on it. | The target indoors. |
| `scratching` | 3 | Something scratches inside the wall beside them at night, small and quick, or slow, a block and a half to their right. | The target indoors, night. |
| `ladder_climb` | 3 | Someone climbs a ladder just behind them, rung by rung, rising five blocks, slow or quick. | The target indoors. |
| `pickup_pop` | 1 | Behind them, the pop of an item picked up, once or twice, or an experience orb taken; nothing on the ground. | The target online. |
| `cold_crackle` | 2 | Their furnace crackles as if lit, once or a few times, and a candle goes out after. It is cold. | A furnace of theirs, the target indoors. |
| `breath_close` | 3 | Breathing at their ear, half a block behind them at head height, once or slowly a few times; in its last way a heartbeat. | The target online. |
| `breath` | 3 | When they crouch still for ten seconds, now and then, slow breathing at their ear, a little longer each way, one way with a heartbeat in it; it stops the moment they move (the brain drops what is left). Started by the crouch, never planned. | Crouched still ten seconds. |
| `footsteps_follow` | 3 | Walking in the open at night, a second set of steps a moment behind theirs, three blocks back along their way, in the sound of the ground there, stopping when they stop, for most of a minute (`footsteps`, ABI 36). | Night, outdoors, walking. |
| `cave_air` | 2 | The sound of a cave out in the open at night, behind them, going away, or coming closer. | The target outdoors, night. |
| `door_scratch` | 3 | Scratching at a door of the house they are in, at night: quick, or in two bursts, or something small. | A door, the target indoors, night. |
| `door_sound` | 1 | A door's sound from another room. No door moved. | A home with two rooms. |
| `chest_sound` | 1 | A chest's sound from another room. | A home with two rooms. |
| `cut_music` | 1 | The client's music stops and does not start for an in-game evening. | The target online. |
| `late_steps` | 2 | The target's own footstep sounds play once more, half a second after they stop, from their own position. | The target walking. |
| `late_eating` | 2 | The same with eating: one more bite after the target's. | The target eating. |
| `place_sound` | 1 | A block-placed sound behind the target. Nothing placed. | The target indoors. |
| `knock` | 2 to 3 | Soft knocks on a door of the house the target is in, at night: three evenly, or two, a pause and one, or four quick, or one; the door shut, no one there. A door-close sound pitched up and quiet (owner: soft knocks early, the heavy one later). | A door, the target indoors, night. |
| `answer_knock` | 2 | They knock on a door (strike it twice within a second and a half, the `knocked` sense, ABI 38); half the time it knocks back from that door a second later: two knocks, or three, or one slow one. Started by the knock, never planned. | They just knocked. |
| `knock_heavy` | 4 | Something heavy at the door at night: slow pounding, two, three or five, the sound of a zombie at a wooden door pitched low; nothing there when it is opened. | A door, the target indoors, night. |
| `lever_click` | 2 | A lever's click, in a house with no lever. | The target indoors. |
| `page_turn` | 2 | Pages turning by the lectern. | A lectern. |
| `arrow_hit` | 3 | An arrow hits the wall behind the target. Nothing is there. | The target indoors. |
| `arrow_near` | 3 | At night in the open, the figure stands a moment in the dark behind them and to one side, a bow in its hand; then a real arrow flies past their head and sticks where it lands, harmless (`shoot`, ABI 42: no harm, never picked up, never gone). | Night, outdoors. |
| `arrow_in_the_door` | 4 | While they are away or asleep, an arrow shot from ten blocks out in front of their door strikes its middle and stays there, harming no one (`shoot`). Once a stay. | A door of the home, the target away or asleep. |
| `snowball_thrown` | 3 | Outdoors in the snow, a snowball thrown from twelve blocks behind them hits their back; no one is there, and it harms no one (`snowball`, a real one). | Snow near them, outdoors. |
| `arrow_grazes` | 4 only | The same, at them: a heart, never below half a heart. | Night, outdoors. |
| `arrow_hits` | 5 | The same: two hearts, never below half a heart. | Night, outdoors. |
| `splash_dark` | 3 | A splash potion from the dark lands at their feet: darkness, slowness, blindness, nausea, or levitation for a moment and a half (a fall of no harm). Never damage, poison or wither: the host throws those five alone (`splash_potion`, ABI 42). | Night, outdoors. |
| `brew_left` | 3 | A potion of its brewing in the chest they use, named `for the night`: night vision and darkness at once (`put_potion`, ABI 42: nothing that harms). | A chest they use, the target away or asleep. |
| `potion_changed` | 3 | One of their potions (one with an effect, never a bottle of water or an awkward, mundane or thick one) is another kind, its name as it was: night vision, invisibility, water breathing, fire resistance, leaping, swiftness or slow falling, never one that harms (`change_potion`, ABI 42). | A potion in the chest they use, the target away or asleep. |
| `door_cloud` | 3 | At dawn a lingering cloud of slowness or weakness on the step outside the door they use, gone the moment it touches them (`effect_cloud`, ABI 42). | Dawn, a door. |
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
| `lost_hour` | 5 | By day, their screen goes black a moment, and as it comes back an hour of the day is gone, the sun moved on (`lights_fail`, then `skip_time` of a thousand ticks: the world's clock, for everyone in it). Off with `events.glitches`. | By day, the target in the game. |
| `painting_flip` | 2 | The painting nearest them hangs upside down until they have looked straight at it for two seconds (a render hook turns it half round on the wall). | A painting, the target indoors. |
| `sun_blink` | 3 | The sun is gone, and the sky night-dark, for one to three seconds in the middle of the day. Then it is back and the time is right. | The target outdoors by day. |
| `moon_wrong` | 3 | The moon is full when it is not, for a few seconds (the sky's hook; twice its size is beyond the hook). | The target outdoors at night. |
| `stars_out` | 3 | The stars fade out for a minute and back. | A clear night, the target outdoors. |
| `static` | 4 | A second of grey static over their whole screen, new each frame, a hiss with it; then nothing (`static`, a client hint). | The target awake. |
| `elder_face` | 5 | Wherever they are, far from any ocean, the elder guardian's face fills their screen with its cry, as the game shows its curse, and no curse after (`elder_face`, a client hint). | The target awake. |
| `clock_wrong` | 3 | The clock in their hand shows night at noon, a minute and a half (a hook on the clock's face). | Holding a clock, by day. |
| `phantom_item` | 4 | The first time they open their inventory in the next two minutes, a paper named "behind you" sits in an empty slot, its name under the mouse; three seconds on it is gone, as if it never was (a hook on the inventory screen). | The target awake. |
| `border_closes` | 5 | The world border's striped red wall stands close round them, seven blocks off on every side, two seconds and a half, as if the world were ending; then it is where it was (on their client alone). | The target awake. |
| `shadow_on_the_wall` | 4 | Indoors at night, while they hold a torch or a lantern, a person's dark shadow on the wall behind them, as if cast by their light; once they turn to it it is gone for good (`shadow_on_the_wall`, a client hint). | Indoors, night. |
| `nightmare` | 5 | As they wake, a single frame of the figure's face, their own skin's, fills their screen over black. | Just woken. |
| `rustle_in_the_pack` | 3 | The next time they open their inventory, within two minutes, something rustles in it as a bundle does, once, though they carry none (`rustle_in_the_pack`, a client hint). | The target awake. |
| `spyglass_figure` | 4 | For five minutes, whenever they look through a spyglass, far off on the ground where they look (seventy-two blocks), the figure in their skin looks back; lowered, it is gone, and once seen it does not come again (`spyglass_figure`, a client hint). | The target outdoors, awake. |
| `recovery_compass_lies` | 3 | The recovery compass they hold points not at where they died but at its place, two minutes (the compass's hook, a recovery compass alone). | Holding a recovery compass. |
| `boss_bar_it` | 5 | A boss bar at the top of their screen, reading only "it", fills slowly over a minute, then is gone (`boss_bar`, ABI 55, their screen alone). | The target awake. |
| `rumble` | 4 | A deep rumble under them, three seconds: the view shakes, hard at first and dying away, and indoors dust falls from the ceiling over them; nothing breaks (`rumble`, a client hint). | The target awake. |
| `boom_far` | 4 | At night in the open, a far explosion from the way of its place: a flash low on the sky there, the burst seen sixty-four blocks off, the boom coming to them muffled; nothing is harmed (`boom_far`, a client hint at the place it is seen). | Night, the target outdoors, a home. |
| `night_early` | 4 | The sun, moon and stars stand two hours later than they are, the sky dimmed, for a minute, eased in and out, then right. | The target outdoors by day. |
| `red_rain` | 4 | While it rains, the drops themselves are red (the rain's colour hook), red within eight seconds, for two to five minutes; no screen cast. The brain may pair it with `red_mist`. | Rain, the target outdoors or at a window. |
| `compass_points` | 3 | A compass the target holds points at the lodger's place for two minutes (a hook on the compass's target). | The target holding a compass. |
| `reflection` | 4 | Water reflects nothing in the game, so a face under it instead (owner): the figure just under the surface of still water near them, its head below the top, up to six seconds, gone once seen and looked away from. | Water within 24 blocks, the target outdoors. |

#### The atmospheres

Real fog on the target's client (the review's D, owner: "not abrupt like other mods,
make it good and spooky"): each eases in over most of a minute, holds with its edge breathing a
tenth over nine seconds, and eases out over half a minute; they stack, each pulling the fog as
it stands. A render hook at the end of the game's fog setup closes and tints it
(`craftkit.host.client.Atmosphere`, `Atmospheres`); the sky takes the fog's colour.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `mist` | 2 | Pale grey mist to twenty-two blocks, motes in it, four minutes. `fog_close` is this mist, a minute long. With a shader pack on (which draws its own fog), a pale veil over the screen and banks of it drifting low over the ground instead. | The target awake. |
| `dusk_noon` | 3 | At noon the light drains: the fog grey-violet to forty blocks, the sky dimmed, a grey veil, five minutes. | Day, the target awake. |
| `red_mist` | 4 | Deep red mist to twenty blocks, a rust sky, red motes, four and a half minutes. | Night, the target awake. |
| `darkness` | 4 | The dark closes to seven blocks, black, their own light still showing what is near, three minutes. | Night, the target awake. |

### The screen

The target's own HUD and camera, for a moment. Off with `events.glitches`.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `hotbar_slip` | 2 | The selected hotbar slot moves to the favourite tool for one tick. | The favourite in the hotbar. |
| `slips_from_hand` | 4 | Walking, what they hold in their main hand slips from it and lies still at their feet, to be picked up again; never with a monster near, never thrown, never lost (`drop_held`). | The target walking, holding something, no monster near. |
| `hunger_shake` | 2 | The hunger bar shakes as if starving, for three seconds, their saturation put back after. | The target awake. |
| `heart_skip` | 3 | The hearts blink as if hurt, once, with no damage. | The target awake. |
| `subtitle` | 1 | With subtitles on, `Footsteps` appears behind them, with no sound. | The target awake. |
| `wrong_sky` | 3 | At noon in the open, for a minute, the moon is where the sun should be and the sun is under the world, on their screen alone (the sky hook). | Noon, outdoors. |
| `name_called` | 3 | With subtitles on, their own name appears in the subtitles, four blocks behind them, with no sound. | The target awake. |
| `hand_on_shoulder` | 3 | At noon in the open, their view turns two degrees, as if a hand were on their shoulder, and a step sounds on the ground right behind them. | Noon, outdoors. |
| `camera_nudge` | 3 | The camera turns two degrees toward the figure, or the place, once. | A figure or a place. |
| `fov_pull` | 4 | The field of view narrows over ten seconds and snaps back. | The target indoors. |
| `tab_twice` | 3 | The target's own name appears twice in the player list for six seconds. | The target awake. |
| `sleepers_wrong` | 3 | `1/2 players sleeping` on a server with one player, or one too many. | The target trying to sleep. |
| `eyes_in_the_dark` | 4 | Underground, twelve blocks ahead of where they look at the height of their eyes, two small glowing lights a hand apart, blinking now and then; gone for good once they come within five blocks (a hint, the game's glow). | The target underground. |
| `hearts_drop` | 5 | For a second and a half the right five of their hearts show empty, drawn as empty containers over them; then as they are, no harm (a hint). | The target in the game. |
| `xp_slips` | 4 | Their experience bar empties from the right over two seconds, stays empty two more, then is as it was: the empty bar drawn over it on their screen, their experience untouched (a hint, `XpSlip`). | The target in the game. |
| `f3_line` | 5 | For half an hour, their debug screen (F3) has one more line among its own, after where they stand: "Lodger: watching" (a hint, `DebugEntryPositionMixin`). | The target in the game. |
| `only_in_third_person` | 4 | For ten minutes, whenever their camera is out of their head (F5), a figure in their skin stands right behind them, facing the way they face, moving as they move; in first person nothing is there (a hint, `BehindFigure`). | The target in the game. |
| `in_the_screenshot` | 5 | The next screenshot they take (F2) within half an hour has a figure in their skin in it, fourteen blocks ahead, facing them, though it was on their screen a frame at most: the game's screenshot held back a tick, the figure set there on their client, the picture taken, the figure gone (a hint, `ScreenshotFigure`, `ScreenshotMixin`). | The target in the game. |
| `handprints` | 4 | As they come home, two handprints of pale dust show on a window of the house, on the glass as if pressed from inside while they were out, half a minute (a hint, `HandPrint`). | The target walking to a door of the home, a window. |
| `false_death` | 5 | Walking outdoors, for a second the death screen over the game, its red wash, "You Died!" and their score, no buttons; then the game as it was, nothing lost (a hint). Once a stay. | The target walking outdoors. |
| `inventory_stare` | 4 | For ten minutes, in their inventory, their own figure stares straight out at them instead of following the mouse (a hint, `InventoryScreenMixin`). | The target in the game. |
| `crosshair_gone` | 3 | The crosshair is gone for a few seconds. | Any. |

### Like a player

The lodger wears your skin, and in these it does what you do. Everything here is dug or placed
the way a player would, one block at a time at a player's pace with a player's tools, so the
sounds are the game's own and a witness who looks sees nothing the game could not do. Digging
keeps to natural blocks (see [What it never does](good-to-know.md#what-it-never-does)). The
mining events are in [Mines and caves](#mines-and-caves); these are the rest.

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `tools_in_hand` | 3 | The far figure holds what the target holds most and swings it at the rock now and then (`figure`'s `swinging`, ABI 35). | The target outdoors. |
| `your_spacing` | 3 | The torches it places in caves and tunnels (`cave_lit`, `mine_beside`) are at the spacing the target uses in their own; its tunnels their height and width. | A tunnel the target dug. |
| `copy_crouch` | 4 | The far figure crouches when the target crouches, and stands when they stand (`figure`'s `copy_crouch`, ABI 35). | The target outdoors. |
| `copy_build` | 4 | A one-block-high outline of the target's house, in cobblestone, at its place, the size of their walls' footprint. | Twenty building blocks of theirs, the target away or asleep. |
| `wave` | 4 | Built as `figure_waves` (batch 14). | A far figure showing. |
| `through_the_hill` | 4 | Thirty blocks ahead of them, the figure walks to the first rock face, digs in a step at a time with the stone closing behind it, and a minute later steps out of the hill's far side. All of it block updates to their client (`dig_in`, ABI 35): approached, both faces are natural rock, nothing in the world changed. | The target outdoors. |
| `into_the_dark` | 3 | At night by a light of theirs, the figure stands at the edge of its reach, steps back one block out of it, the light flickers, and it is gone. | A light of theirs within ten blocks, night, the target outdoors. |
| `into_the_mountain` | 4 | Thirty blocks ahead of them, the figure walks to the rock face and digs into it, a one-by-two tunnel, the stone closing behind it block by block until the face is smooth and it is gone; on their client alone (`dig_in`, ABI 35). Owner's idea. | The target outdoors. |

### Building

What it builds, it builds from what it gathered: stone it dug (as cobblestone, as a pick gives
it) and whatever else its store holds; what it lacks it makes up so a build never stalls (owner: "gathered, topped up when short"). A night's share at a time, at a player's pace,
out of sight, only into air or over grass, flowers, snow and the ground's own top (and water for
a dock or bridge), never over anything a player built (see the
[rule](good-to-know.md#what-it-never-does)). More in [Ideas](ideas.md#building).

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `pillar` | 3 | A cobblestone pillar at its place, one block higher each night, up to the height of the target's roof (five above their floor). | A home, the target away or asleep. |
| `cave_room` | 3 | A 3x3x3 room dug in the cave near the home they have not been in, lit, an oak door at its way in, and a chest with what it took. | A cave they have not entered. |
| `dock` | 3 | A plank dock out over the water from where they stood fishing, one block longer each night, eight at most. | A place they fished, the target away or asleep. |
| `hut_on_route` | 3 | A 3x3 shelter of logs, two high, a log roof, an opening along the way, halfway along their route, a torch inside. | A route, the target away or asleep. |
| `camp_nearby` | 4 | At night, at a point of their route twenty-four to sixty-four blocks from them, a lit campfire with a log seat either side, and a figure crouched by it facing the fire; at dawn every block of it is taken away again (its own blocks only). | Night, outdoors, their route known. |
| `house_near` | 4 | A house of its own, 56 blocks from theirs on the side away from their route's far end, 5 by 5: a plank floor, cobblestone walls three high, a log roof, its door facing theirs, a pane in the wall across, a torch and a chest inside, built a third a night over three nights (`place_blueprint`, ABI 34: the ground found on the first night and kept). The chest takes what it holds once the ground is known. | A home, the target away or asleep. |
| `house_grows` | 5 | Its house grows a second room, five by four, behind it, through the back wall. | Its house standing, the target away or asleep. |
| `bridge` | 4 only | A plank bridge from the far shore of the water toward the target's, three blocks a night, stopping two short of their shore. | Water by the home, the target away or asleep. |
| `bridge_done` | 5 | The bridge finished, to their shore. | Its bridge, the target away or asleep. |
| `copy_house` | 5 | A copy of the target's house at its place from their building blocks: stone as cobblestone, wood as logs, glass as glass, forty blocks a night. | Twenty building blocks of theirs, the target away or asleep. |

### The figures

| Event | Stages | What happens | Needs |
| --- | --- | --- | --- |
| `crossing_ahead` | 3 | Walking outdoors: thirty blocks ahead of them a figure walks across their way, from their left to their right, twenty blocks at an unhurried walk, facing its way and never them, empty-handed; gone at the far side (`figure`'s `walks`). | The target walking outdoors. |
| `lantern_in_the_woods` | 4 | At night, forty blocks ahead of them, a figure walks across their way thirty blocks with a lantern in its hand, facing its way; gone at the far end (`figure`'s `walks`). | Night, the target outdoors. |
| `taken_up` | 4 | Outdoors, one to four of their most valuable things, from what they carry (never what they wear or hold) and from their chests, the rarest first (netherite, an elytra, a totem, diamonds, enchanted things, emeralds, gold...), one after another hang in the air in front of them, turning, on their screen alone, then fly up into the sky faster and faster and are gone, a chime and a rush as each goes. The very stacks are kept, never lost (`take_up`, ABI 54). Owner's idea: "consider chests as well, and a random number of items". | The target outdoors, something of worth on them or in their chests. |
| `sky_gives_back` | 4 | Half a day or more after `taken_up`, all it took falls out of the sky in front of the door they use, one stack after another, as items that never despawn (`drop_kept`, ABI 54). An ending's start and a reset give them back too, so nothing is kept for good. | Things of theirs up there half a day, a home. |
| `drawn_to_the_light` | 4 | Running about at night in the open with a light in hand (a torch, a lantern, a candle, in either hand), five seconds of it, a run in three or so draws it: forty-five blocks off to one side, out of the dark, a figure turns to them and walks at their light, a quick walk. The moment they hold no light it has lost them, for good: it stops, looks this way and that, and is gone. Kept lit, at twenty blocks it runs at them, flat out, and is gone two blocks short with a hit's jolt, no harm (`figure`'s `drawn_to_light`, ABI 54). Never from the menu. Owner's idea. | Night, outdoors, running with a light in hand. |
| `ceiling_crawl` | 5 | At night in the house, the figure crawls upside down across the ceiling over them, from five blocks to one side to five to the other, slow, gliding, and is gone at the end as if through the wall (`figure`'s `upside_down` with `crawls`, its path under the ceiling, ABI 55). | Indoors at night, a home. |
| `in_the_lava` | 4 | Underground, the figure stands in a lava lake six to sixteen blocks off, unburnt, facing them; once seen and looked away from it is gone. The host finds the lava (`figure`'s `finds` `lava`, ABI 55); none near, nothing. | Underground. |
| `over_the_edge` | 4 | At a cliff's foot, a head peeks over the top edge high above them, eight blocks up or more, straight ahead, watching; seen and looked away from, gone (`figure`'s `finds` `cliff_top` with `sunk`, ABI 55); no cliff ahead, nothing. | Outdoors, awake. |
| `rider_on_the_ridge` | 4 | At dusk, forty blocks off to one side, the figure sits a skeleton horse, still, watching them; once seen and looked away from it is gone (`figure`'s `mount`, ABI 55). | Dusk, outdoors. |
| `while_unseen` | 4 | At night in the open, a figure eighteen blocks behind them that moves only while it is not seen: on their screen with nothing between it holds still, facing them; off their screen or behind a wall it comes a step nearer along the ground every fifth of a second. At arm's length it strikes, two hearts, never a kill, and is gone; three minutes at most, then it is gone where it stands (`figure`'s `while_unseen`, ABI 54). Owner's idea. | Night, the target outdoors. |
| `menu_freaks` | 4 | Armed ten minutes: the next time they open the game's menu (Escape), every button shivers and jitters in place, hardest at first and settling over four seconds, and the first one reads `stay` a moment; then it is the menu as ever (the `menu_freaks` hint). Owner's idea. | The target online. |
| `bare_trees` | 4 | In the woods, a stand of trees off to one side of them, fourteen to twenty-four blocks, up to six together, made wrong, by turns: every leaf of their own gone, bare trunks and branches among the green; every log of them stripped, pale and skinned; or both (`bare_tree`, ABI 54: the tree's own leaves alone, never placed ones). Owner's idea. | In the woods. |
| `out_of_place` | 3 | Out and about, something that does not belong, set out where they will come on it: underground on the floor of a cave they have not entered, above on their own way back (their trail sixteen to thirty blocks back). By turns: a made bed; a table laid, a cake with a lit candle beside it; a chair facing nothing; a flower in a pot; a carpet and a lit lantern. Ten minutes on it is taken away again, block by block, so it gives nothing (`place_blueprint`, then `swap_block` to air over each of its own blocks alone). Owner's idea. | The target online, a cave unentered or a trail behind them. |
| `cave_torches_flicker` | 3 | Underground, their own lights within twenty-four blocks (up to eight) flicker one after another down the line toward them, farthest first, three blinks each as if something passed it; the nearest goes dark a second and a half last of all, then all are steady (`hide_block`, each put back). Owner's idea. | Underground, three lights of theirs or more near. |
| `prints_around_the_house` | 3 | While they are away or asleep, more footprints round their house, dark on any ground, that stay: by turns a trail in from eighteen blocks out to a window, stopping under it; a loop round the walls a block and a half out; or three pairs standing before the door they use, facing it, as if someone waited there. Kept by the memory (160 at most, the oldest let go) and drawn on their client whenever they are near home (the `footprints` hint, sent again every nine minutes within 48 blocks). Owner's idea. | A home, the target away or asleep. |
| `dead_redstone` | 3 | For a minute and a half their redstone by the house is dead: every pressure plate, button and lever within sixteen blocks of the home presses, clicks and flips as ever and powers nothing, the door shut, the lamp dark, the piston still; what was powered goes dark at once, and comes back after (`dead_redstone`, ABI 54). Owner's idea. | A plate, button or lever of theirs at the home. |
| `sound_warp` | 3 | For half a minute to a minute every sound they hear goes wrong, a way of four in turn: slowed and deep, as a tape dragging; a long echo, as in a hall far too big; muffled, as through a wall; or crunched and broken. Then it is as it was. Their client's own sound, through OpenAL's effects (the `sound_warp` hint; a device without them keeps the slowed way; with Sound Physics Remastered set on top of its own, taking only what each way needs). Also after `creeper_behind`'s bang, their hearing muffled a moment. Owner's idea. | The target awake. |
| `villager_flees_in` | 4 | Once a stay, while they are in the house and no village is near it, a villager is suddenly before the door they use, banging on it three times with a cry, as if something were after it; the door opens, it runs in, and two blocks in it drops dead. No one is to blame (no killer, no gossip, no golem), it drops nothing, and the door is left open (`villager_flees_in`, ABI 54). Owner's idea. | No village within the home's surroundings, a door, the target indoors. |
| `breathing_in_the_wall` | 3 | In the house, slow breathing from inside the wall beside them, a block and a half to their right, deep and drawn out, two to four breaths. | The target indoors. |
| `scream_indoors` | 4 | At night in the house, a goat's scream from the floor above or through the wall, where no goat is; once, or twice. | The target indoors at night. |
| `item_break_behind` | 3 | The snap of a tool breaking right behind them, at their ear or a few blocks back; nothing of theirs is broken. | The target awake. |
| `armour_behind` | 4 | Walking at night, the clank of armour a few steps behind them, six steps' worth, iron, chain or something heavier; no one there. | Night, the target walking. |
| `breath_in_the_chest` | 3 | In the house, a breath from inside the chest they use, close and slow; once a lid shuts on it. | A chest they use, the target indoors. |
| `rain_on_the_roof` | 3 | Indoors under a clear sky, rain drums on the roof over them for a quarter to half a minute; outside it is dry. | The target indoors, no rain. |
| `sniffing` | 4 | At night in the open, sniffing close behind them, as of something hunting by smell; nearer, or moving off. | Night, the target outdoors. |
| `gasp_when_turning` | 4 | Walking at night, a sharp breath in right behind them, as if something there were startled. | Night, the target walking. |
| `water_where_none` | 3 | In the house, the slosh and splash of something moving in water beside them, or water running under the floor; there is no water. | The target indoors. |
| `doors_slam_far` | 4 | At night, far off, door after door slamming at once, one great crash. | Night, a village known. |
| `the_count` | 4 | At dusk a bell rings at their house: the first time once, the next twice, each time once more, nine at most, counting. | Dusk, a home. |
| `rattling_round_the_house` | 4 | At night while they are in the house, a skeleton's rattling steps go all the way round the outside of it, a block and a half from the walls, and a rattle at the last corner; no skeleton. | The target indoors at night, a home. |
| `chains_dragging` | 4 | Underground, chains dragged along the stone behind them, coming nearer, or going off. | The target underground. |
| `shriek_below` | 5 | Once a stay, in the house, a sculk shrieker's shriek from far under it; or a heartbeat down there, then the shriek. | A home, the target indoors. |
| `witch_laughs` | 3 | At night in the open, a witch's laugh far off; no witch. | Night, the target outdoors. |
| `pet_whines_at_door` | 3 | At night their dog sits facing the door they use and whines at it, at nothing outside, now and then for a minute and a half (`hold` facing the door). | A tamed wolf of theirs near, a door, night. |
| `glass_rubbed` | 3 | At night while they are in the house, a long squeak of something dragged slowly across a window's glass from outside. | A pane, the target indoors at night. |
| `bones_underfoot` | 4 | Walking in the house, a few seconds of every step crunching as on bone. | The target indoors, walking. |
| `bubbling_under` | 3 | In the house, water bubbling under the floor they stand on, as from a deep spring; there is no water. | The target indoors. |
| `the_bell_in_the_chest` | 4 | In the house, a small bell rings inside the chest they use, once, or twice. | A chest they use, the target indoors. |
| `panting_behind` | 4 | Running at night, panting right behind them, breath for breath. | Night, the target running. |
| `iron_door_below` | 4 | Underground, an iron door slams somewhere far under them, where there are no doors. | The target underground. |
| `ice_cracks` | 3 | Walking in a frozen biome, something cracks under them, as ice splitting; nothing gives. | A snowy biome, the target walking outdoors. |
| `bush_rustles` | 3 | Walking outdoors, something rustles hard in the green beside them, as if something in it moved; nothing comes out. | The target walking outdoors. |
| `snoring_beside` | 4 | Lying in bed, someone snoring softly and slowly beside them; no one there. | The target in bed. |
| `the_killer_rabbit` | 4 | Out in the open, a white rabbit eight blocks ahead of them, red-eyed, the game's own killer rabbit, comes at them fast, half a minute: each bite a heart, never a kill; killed it leaves nothing (`place_entity`'s `minecraft:killer_bunny`, ABI 55). | The target outdoors, awake. |
| `sigh_at_the_door` | 3 | In the house, a long sigh just outside a door of theirs, slow, as of someone waiting there; no one there. | A home with a door, the target indoors. |
| `humming_ahead` | 3 | In a tunnel, a few notes hummed somewhere ahead of them, low, a tune, then quiet. | The target underground, walking. |
| `steps_counted` | 4 | Walking at night, behind them a note at each step's time, each one higher up the scale, eight of them, as if something counted their steps; then nothing. | Night, the target walking. |
| `cart_overhead` | 3 | Underground, a minecart rolls across the rock over their head and away; there are no rails. | The target underground. |
| `ram_on_the_wall` | 4 | At night, three heavy thuds against a wall of the house, as of a goat ramming it, then nothing. | Night, a home, the target indoors. |
| `bell_underwater` | 4 | Swimming, a bell rings once below them, muffled by the water. | The target swimming. |
| `bark_from_home` | 3 | Out with their dog, a dog barks from inside their house, where no dog is. | A pet of theirs, a home, the target outdoors. |
| `hoofbeats_circling` | 4 | At night indoors, a horse's hoofbeats gallop a loop round the outside of the house, a block and a half from the walls, and away into the dark; no horse. | Night, a home, the target indoors. |
| `snipping_in_the_dark` | 3 | Near their sheep at night, the snip of shears, again and again; the sheep keep their wool. | Sheep near, night. |
| `eating_in_the_wall` | 3 | Indoors, chewing, wet and slow, from inside the wall beside them. | The target indoors. |
| `ominous_hum` | 4 | At home, from under the floor, the low hum of a trial spawner turning ominous; there is no spawner. | A home, the target indoors. |
| `vault_opens_by_the_bed` | 4 | Lying down, the clunk and chime of a vault unlocking beside the bed. | The target in bed. |
| `creaking_behind` | 4 | In the woods, the groan and steps of a creaking behind them, close, gone when they turn. | In the woods, the target walking. |
| `bulbs_clicking` | 3 | Indoors, the click of copper bulbs switching, one after another round the room; there are none. | The target indoors. |
| `gust_from_nowhere` | 4 | A wind charge's burst right beside them and a soft push away from it, as from a breeze no one sent; no harm (`push_player`, a nudge). | The target awake. |
| `digging_under_the_floor` | 3 | Indoors, under the floor, the snuffle and scrape of something digging, as a sniffer digs. | The target indoors. |
| `wings_in_the_room` | 4 | Awake at night indoors, wings beat close round them, circling, and stop. | Night, the target indoors and awake. |
| `lava_in_the_wall` | 4 | Indoors, from inside the wall beside them, the pop and gloop of lava; the wall is cold. | The target indoors. |
| `crossbow_click` | 4 | The click of a crossbow being loaded right behind them; nothing follows. | The target awake. |
| `teleport_beside` | 4 | Twice, the warp of a chorus fruit right beside them, the second nearer, as if something kept stepping closer. | The target awake. |
| `sweeping_next_room` | 3 | In the house, from the next room, the soft strokes of a brush sweeping, as at a dig; then quiet. | A home, the target indoors. |
| `pot_shatters` | 3 | In the house, a decorated pot shatters loudly; nothing of theirs is broken. | The target indoors. |
| `blow_on_the_shield` | 4 | With a shield in hand, a blow lands on it from nothing: the block's sound in front of them and a shove back, as a hit taken on it; no harm. | A shield in either hand. |
| `copper_golem_at_door` | 4 | At night a copper golem stands still two blocks outside the door they use, facing them in the house; five minutes on it is gone, nothing left (`place_entity`'s `minecraft:copper_golem`, ABI 55). | Night, a home with a door. |
| `creaking_in_the_field` | 5 | At night a creaking stands in their field, the game's own: frozen while they watch it, coming a step at a time whenever they look away. If it reaches them, a nick of half a heart; it is gone in two minutes, and killed it leaves nothing (`place_entity`'s `minecraft:creaking`, spared, ABI 55). | Night, a field of theirs. |
| `creeper_behind` | 3 | A creeper suddenly right behind them, two blocks back, on their screen alone: it hisses and swells as a real one does and, a moment and a half on, always goes off: the flash, the bang, the jolt as of a hit, and nothing harmed, not a block broken, nothing there after (`fake_creeper`, ABI 54). Owner's idea. | The target awake. |
| `vex_single` | 4 | Out of nowhere, a vex a few blocks up behind them comes at them through walls and all, half a minute: its hits two hearts, never a kill; killed it leaves nothing (`place_entity`'s `targets` and `harm`). Stage four. Owner's idea. | The target awake. |
| `vex_strikes` | 5 | `vex_single` in stage five: its hits three hearts, never a kill. | The target awake. |
| `vex_wave` | 4 | A swarm of twelve vexes out of nowhere all round them, coming at them through walls and all, half a minute: each hit half a heart, never a kill; killed they leave nothing. Owner's idea: "a bunch coming at the player but dealing little damage". | The target awake. |
| `zombie_in_your_gear` | 5 | At night in the open, twenty blocks behind them, a zombie walks at them wearing armour like theirs, piece for piece, and holding a tool like the one in their hand: new and plain, never dropped. Its hits two hearts by the harm stages, never a kill; killed it leaves nothing, and two minutes on it is gone (`place_entity`'s `wears`, ABI 54). | Night, the target outdoors. |
| `wolves_circle` | 4 | At night in the open, five wild wolves come round them twelve blocks out and circle, nearer each round, to four blocks, a growl now and then; half a minute on they break off all at once and run into the dark, and are gone. They hurt no one (`circle`, ABI 54). | Night, the target outdoors. |
| `drops_from_the_ceiling` | 4 | Underground, a cave spider drops from the ceiling a block and a half before their face, with its hiss, and goes for them a minute: a nip and its poison, never a kill, killed it leaves nothing (`place_entity`'s `targets`). | The target underground. |
| `daylight_phantom` | 4 | At noon in the open, a phantom out of the sun, twenty blocks over them, swoops at them, burning as phantoms burn by day: a nip at most, never a kill, and fifteen seconds on it is gone, nothing dropped (`place_entity`'s `targets`). | Noon, the target outdoors. |
| `wall_bursts` | 4 | Indoors, a block of the wall they built two to six blocks from them (at their height or a block over; never glass, a door or a light) breaks with its sound and is gone, and five silverfish pour out of the hole at them, real and weak: a nip each, never a kill, killed they leave nothing. A minute on they are gone; the block is back ten minutes on (`hide_block`, `place_entity`'s `targets` and `harm`, ABI 54). | The target indoors, a home. |
| `villager_collapses` | 4 | A villager six to twenty-four blocks from them drops dead before them: the game's own death, its fall and its sound, on their screen alone, held still meanwhile. A minute on, the first moment they are not looking at it, it is up and about again as if nothing had been. No villager is harmed (`fake_death`, ABI 54). | A villager near them. |
| `villagers_walk_out` | 4 | At dusk, the villagers near them (eight at most) walk out of the village one after another, a second and a half apart, single file, to a place forty blocks out toward its place (else away from their home); let go there, they go home to their beds of themselves (`approach`'s `to`). | Dusk, a village, villagers near. |
| `villager_spins` | 4 | A villager twelve to thirty blocks from them turns round and round where it stands, fast and silent, for two minutes at most; when they come within five blocks it stops, facing them, a few seconds, and is a villager again (`hold`'s `spin`, ABI 54). | A villager twelve to thirty blocks off. |
| `trades_refused` | 3 | For a day of the game's, every villager they go to trade with refuses them: no trade opens, it shakes its head with a no, as one with nothing to sell (`hold_trades` `refuse`, ABI 54). | A villager near them. |
| `villager_stabs` | 4 | Within the day, the next villager they go to trade with opens no trade: a blade is in its crossed hands, on their screen, and they are stabbed, two hearts, never a kill; a breath later the blade is gone, and it is a villager again, its trades as ever (`hold_trades` `stab`, ABI 54). Owner's idea: "when trying to interact with a villager you get stabbed". | A villager near them, stage four or five. |
| `villagers_flee_you` | 4 | Walking into the village (its middle within forty blocks and nearer), every villager near them runs from them in a panic, as from a zombie, for a minute, and the bell rings, so they go in to hide and shut the doors; then they are as they were. No reputation lost, no golem set on them: they only think they were hurt (`panic`, ABI 54). | Walking into a village, villagers near. |
| `circles_a_tree` | 3 | In snowy woods, round the nearest trunk behind them (eight to twenty blocks), footprints go round and round it, two rings, three and four blocks out, coming from nowhere and going nowhere (`prints`). | In the woods, a snowy biome. |
| `animals_flee_the_woods` | 4 | In the woods, every animal near them (up to four, never a pet or a named one) runs past them, a run, away from behind them, as from something; thirty blocks back among the trees a figure walks across, its back to them, and is gone (`approach`'s `to`, `figure`'s `walks`). | In the woods, animals near them. |
| `tree_comes_down` | 4 | Walking in the woods, a tree five to nine blocks ahead of them comes down across their way with a crash: each log over its stump falls as a block laid along the ground, the stump left standing, its leaves left to fall as leaves do. Never a log on them: none lands within two blocks of them (`fell_tree`, ABI 54). | In the woods, walking. |
| `trees_close_behind` | 4 | Walking deep in the woods, behind them where they came in (their trail eight to fourteen blocks back, clear of the trunks), a sapling of the nearest tree's own kind is set and grown at once with bone meal's growth: a full tree stands in the way they came (`set_block`, `grow`). A sapling left there when it has no room to grow. | In the woods, walking, a trail behind them. |
| `the_tree_line` | 5 | Walking out of the woods (five trunks or more behind them, one at most ahead), a figure stands between each two trees of the edge behind them, up to six, still, facing them, empty-handed; when they look back and away again, all are gone at once (`figure`'s `together`). | Walking out of the woods. |
| `branch_snaps` | 3 | Walking in the woods, a crack of breaking wood right behind them in the tree they just passed (the nearest trunk behind them, three to ten blocks), and up to four of its leaves torn out of its crown, falling as leaves break; no one there. Never a leaf anyone placed (`snap_branch`, ABI 54). | In the woods, walking. |
| `blazed_trees` | 3 | In the woods, the trunks in a line straight ahead of them, ten to thirty blocks off and five apart or more, three to five of them, each have a strip of bark gone at eye height: a trail blazed, leading on (`swap_block` to the stripped log). | In the woods, three trunks in a line ahead. |
| `knocks_on_wood` | 3 | In the woods, three knocks on a trunk eight blocks or more ahead of them, at head height, then three on one farther in, then on a third, a few seconds apart, leading them deeper; on the last tree a strip of its bark is gone at eye height (`swap_block` to its stripped log). | In the woods (trunks of the tree read ahead of them). |
| `tree_to_tree` | 4 | In the woods, a figure half out from behind a trunk eight to thirty blocks behind them, facing them; each time they look back at it and away again it is behind a trunk nearer, four blocks nearer or more, four trunks at most; after the nearest it is gone, and a branch cracks where it stood (`figure`'s `then_at`, ABI 54). | In the woods (trunks of the tree read behind them). |
| `peeks_from_the_trees` | 4 | Walking in the woods, a figure leans out from behind a trunk ten to twenty-two blocks ahead of where they look, half of it past the bark, empty-handed, facing them; once looked at it slips back behind the trunk and is gone. Five seconds on, a second from a trunk farther in, when there is one. Gone as they come within four blocks, or after twelve seconds (`figure`'s `peeks`, ABI 54; the trunks from the tree read). Owner's idea. | In the woods (trunks of the tree read around them), walking. |
| `crouched_behind_a_tree` | 4 | Outdoors, ten blocks ahead and well to one side, a figure crouches as if hiding, empty-handed, facing them; gone as they come within four blocks or once seen and looked away from. | The target outdoors. |
| `in_a_line` | 4 | At night, outdoors on their route: figures stand in single file along it ahead of them, one every eight blocks, six at most, all facing the house, empty-handed; each gone as they come within four blocks of it (`figure`'s `faces`). | Their route known, night, the target outdoors. |
| `peeks_from_ground` | 4 | Outdoors, twenty-four blocks ahead and a little aside, a figure stands a block and a half into the ground, its head alone above it, facing them; once they have looked at it, it sinks slowly out of sight and is gone (`figure`'s `sunk`). | The target outdoors. |
| `waits_at_the_portal` | 4 | As they come back through a portal (three times in ten; never from the menu), a figure stands three blocks beside where they arrive, facing the portal, empty-handed, as if it had waited; gone once seen and looked away from. | Coming back from another dimension. |
| `wears_your_face` | 4 | At night outdoors, the zombie nearest them (or a husk, a drowned) wears their own head, five minutes, never dropping it, and walks at them; then what it wore before (`wear_face`, `approach`). | Night, a zombie near them outdoors. |
| `animal_follows` | 3 | The farm animal of theirs nearest them follows them a whole day at a distance, wherever they go, stopping six blocks off when they stop (a walk after them renewed every eighty seconds). | An animal of theirs near them. |
| `chicken_lays` | 3 | A chicken near them lays, with an egg's sound, what it took from them instead of an egg: the very thing, where the chicken stands. Once a stay. | It holds something of theirs, a chicken near them. |
| `all_freeze` | 4 | Outdoors, every animal, villager and mob within twenty-four blocks of them stops at once for three seconds, facing them, then carries on (`hold` on each). | Animals near them outdoors. |
| `figure_on_map` | 4 | Away from home (thirty-two blocks or more) with a map in hand: a player's marker stands at their house on it, five minutes, for them alone, the map itself unchanged (`map_mark`). | The target holding a map, away from home. |
| `far_figure` | 3 | The far presence: a figure in the target's skin, name shown, 48 to 80 blocks away, standing, facing the target, for ten to forty seconds. Approached within 32 blocks, it is gone. | A line of sight from the route or home. |
| `stands_in_the_rain` | 3 | In the rain, twenty-five blocks ahead of them, a figure stands still in the open, empty-handed, its face turned up to the sky; gone once seen and looked away from (`figure`'s `faces` high over it). | Raining, the target outdoors. |
| `behind_you` | 3 | When the target looks back along their route, the figure is 60 blocks behind, standing. The next time, 50. It is never nearer than 32. | The target travelling a route. |
| `on_the_roof` | 3 | At dawn, from outside, the figure stands on the target's roof, and is gone when they look away. | A home with a roof and the target outdoors at dawn. |
| `across_water` | 3 | The figure stands on the far shore, or sits in a boat in the middle of the water. | Water of 32 or more blocks across. |
| `on_the_water` | 4 | Far off, a figure stands on the surface of the water near them, still, facing them, empty-handed; gone once seen and looked away from. | Water near them, outdoors. |
| `in_the_treetops` | 4 | Outdoors, thirty blocks ahead of them, a figure stands on top of the trees (the leaves its ground; the ground where there are no trees), facing them, empty-handed; gone once seen and looked away from (`figure`'s `on_leaves`). | The target outdoors. |
| `in_doorway` | 4 | As the target turns, the figure is in the doorway or at the window for less than half a second, then not. No sound. | The target indoors, facing away from a door or window. |
| `at_the_chest` | 4 | Coming into the room, the target sees the figure at their chest, back to them. It closes the lid and is around the corner before they reach it. | A used container in a room with two ways out. |
| `in_your_bed` | 4 | Coming home late at night, the target sees the figure lying in their bed through the door, and the bed is empty when the door opens. | A bed in view of the door, night. |
| `by_the_bed` | 4 | Coming in at night, a figure stands a block from their bed toward their door, facing the bed, empty-handed, still; gone once seen and looked away from. | Night, coming home, a bed. |
| `replay` | 4 | The replay figure walks a recorded route, at the recorded pace, starting out of view. Looked at, it keeps walking. Approached within 24 blocks it rounds a corner and is gone, leaving the held item where it stood if it held one. | A recorded route of 20 or more points. |
| `figure_nip` | 3 only | The figure right behind them for a second, two blocks back: it swings once, a hit, half a heart, the hurt's jolt, never below half a heart (`figure_swing`, ABI 33). The owner's harm stages: a nip at 3. | The target awake. |
| `figure_swing` | 4 | The same, hard: two hearts, still never below half a heart, never a kill. | The target awake. |
| `shove` | 4 | Armed for two minutes: the first moment they stand at the edge of a drop of two or three blocks onto solid ground (or on a ladder with ground that near below), not crouching, in no water, with more than two hearts, they are pushed off it, and a step sounds right behind them (`shove`, ABI 37). Never a fall of more than three blocks, so never any harm. Stage four. | The target awake. |
| `shove_far` | 5 | `shove` in stage five, an edge up to six blocks high: a fall that harms, up to a heart and a half, and only when they have that and a heart over, so never a kill (`shove`'s `most_drop`, ABI 54). The owner: "depending on the stage higher falls are okay". | The target awake. |
| `figure_mirror` | 3 | The figure forty blocks to their side moves as they move, step for step, stopping when they stop, forty seconds, gone if they come within twenty (`figure`'s `mirror`, ABI 33). Owner's pick. | The target outdoors. |
| `figure_window` | 4 | At night, while they are inside, the figure stands outside a window of theirs, its head at the glass, looking in, half a minute, gone when they come within six. Owner's pick. | A pane, night, the target indoors. |
| `figure_crowd` | 3 | In the village, the figure stands still among the villagers, facing them, and is not there once they look away from it (`figure`'s `once_seen`, ABI 33). Owner's pick. | A villager near, the target outdoors. |
| `figure_waves` | 3 | The far figure, its name shown, raises its arm once and is gone. Owner's pick. | The target outdoors. |
| `ring_of_figures` | 4 | At night, while they are in the house, eight figures in their skin stand in a ring round it, twenty blocks out, on the ground, empty-handed, facing in; each is gone once they have seen it and looked away. (The owner: "multiple figures standing in a circle around our home".) | The home known, night, the target indoors. |
| `upside_down` | 4 | As they wake, the figure hangs feet up from the ceiling over the bed, facing them, empty-handed (the host finds the ceiling, the first solid block above the bed; the game draws it so as it draws a player named Dinnerbone); gone once seen and looked away from. A waking starts it or `foot_of_bed`, one of those the stage offers. | The target waking, a bed. |
| `the_crawl` | 4 | At night, while they are in the house, a figure crawls on its belly from their doorway across the floor to the bed, head first, slower than a walk, and is gone at the bed (`figure`'s `crawls`: the game's swimming pose on land, a glide eased by their client). | Night, the target indoors, a bed and a door known. |
| `hunted` | 5 | At night, outdoors: thirty-six blocks ahead of where they look, a figure starts running at them, flat out (a sprint and a half), after them as they move, and is gone two blocks short with a hit's jolt and sound and no harm (`figure`'s `rushes`). | Night, the target outdoors. |
| `falls_from_the_sky` | 5 | At night, outdoors, as they look up at the sky, a figure falls from forty blocks over the ground forty blocks ahead of them, fast, lands with a heavy fall's thud, and is gone there (`figure`'s `falls`). | Night, the target outdoors looking up. |
| `pet_follows` | 4 | Outdoors, nine blocks ahead of them, a figure stands empty-handed; their pet gets up and walks to it; the figure is gone after a few seconds and the pet comes back to them of itself (`approach`'s `to`; nine blocks, never the twelve at which the game brings a pet back at once). | A pet of theirs near them, outdoors. |
| `pet_stares_at_wall` | 3 | Their pet stands still a minute facing the wall of theirs nearest it, as animals do at what no one else sees (`hold` with `at`). | A pet of theirs, their walls known. |
| `at_their_table` | 4 | As they come home, a figure stands at their crafting table, facing it, working at it with their own tool, seen through the door or a window; gone as they come within five blocks. | Coming home, a crafting table in the house. |
| `eats_at_their_chest` | 4 | As they come home, a figure crouches at the chest they use, facing it, eating their food (the first food the chest holds, in its hand; nothing is eaten), the sound of eating; gone once seen and looked away from (`figure`'s `eats`). | Coming home, food in the chest they use. |
| `jumping_far` | 4 | Outdoors, thirty blocks ahead of them and a little aside, a figure jumps in place, again and again, a player's jump a second apart, facing them, empty-handed; gone once seen and looked away from (`figure`'s `jumps`). | The target outdoors. |
| `waves_from_inside` | 4 | As they come home, a figure inside the house a block in from a window, its head at the glass, raises its arm three times, waving at them, empty-handed; gone as they come within five blocks. | Coming home, a window. |
| `boat_passenger` | 4 | Rowing alone, a figure sits in the seat behind them, empty-handed, on their client alone; gone once they have turned round to it and looked away, when they leave the boat, or when anyone else gets in (`figure`'s `rides_behind`). | The target in a boat or raft with a second seat (no chest boat). |
| `rides_their_horse` | 4 | Their horse stands with a figure in its saddle, empty-handed, on their client alone; gone once seen and looked away from, or as anyone gets on it (`figure`'s `rides`). | Their horse near them, outdoors. |
| `every_window` | 5 | At night, while they are in the house, a figure stands outside every window at once (one for each run of panes, eight at most), a block out, its head at the glass, looking in; all gone together the moment one has been seen and looked away from (`figure`'s `together`). | Night, the target indoors, two windows or more. |
| `mirror_in_the_window` | 5 | Outdoors near the house, in one of its windows, just inside the glass, a figure copies their every move, as a reflection would; twenty seconds, or gone once seen and looked away from (`figure`'s `mirror`). | The target outdoors within twenty blocks of a window. |
| `one_more_each_night` | 4 | At night, at the edge of their light (seven blocks past it from them), one figure the first night it comes, two the next, three the next, up to six, side by side across the way out, still, empty-handed, never closer; all gone together when one is seen and looked away from, or when they come within six blocks. A night apart at least. | Night, a light of theirs near them. |
| `torch_flash` | 4 | Underground, as they place a light (one time in seven or so; never from the menu), a figure stands two blocks past it in its light, facing them, for a blink (a fifth of a second). | A light placed underground, that moment. |
| `where_you_died` | 3 | Where they last died, a figure lies on the ground on its side as a body lies, empty-handed, seen from twelve to sixty-four blocks off; gone when they come within ten blocks (`figure`'s `lies`). | Where they last died, twelve to sixty-four blocks from them. |
| `rubberband` | 4 | Walking outdoors, they are set back along their trail three blocks or more, as on a bad connection (`move_player`), and where they had been a figure stands facing them, empty-handed, gone once seen and looked away from. | The target walking outdoors. |
| `the_offer` | 4 | In their doorway, as they are outside, a figure stands holding out what it took from them, facing them; come within three blocks and it is gone and the very thing lies at their feet, given back (the host's word that the figure was come to, then `drop_item` of what it keeps). Gone any other way, it keeps it. Once a stay. | It holds something of theirs, a door known, the target outdoors. |
| `top_of_the_ladder` | 4 | Down a shaft of their ladders, six blocks or more below its top, a figure stands at its mouth over the top ladder, looking down at them, empty-handed; gone when they climb within six blocks of it. | The target down a shaft of their ladders. |
| `climbs_ahead` | 4 | Down a shaft of their ladders, a figure climbs ahead of them, from three blocks over them up to the top, at a climb, empty-handed; gone at the top (`figure`'s `walks`, up). | The target down a shaft of their ladders. |
| `pulled_under` | 4 | Swimming, they are pulled down twice, a moment apart, as if taken by the ankle, a whirl of water and a drowning gasp, no harm, and let go (`push_player`, a block a tick at most). | The target swimming. |
| `swims_beneath` | 4 | Swimming, a figure swims past three blocks beneath them, from eight blocks to one side to eight to the other, across the way they face; gone at the far end (`figure`'s `crawls`, a swim in water). | The target swimming. |
| `held_breath` | 4 | Under water, their air runs out twice as fast this dive (twenty seconds), and nothing is there; the drowning, if it comes, is the game's (`drain_air`). | The target under water. |
| `its_lights_on` | 4 | At night, eight to forty-eight blocks from the house it built (whole), its house goes dark (its wall torch taken away a while), then a lantern on its floor lights, and another, and another, three seconds apart, seen through its door and pane; half a minute on, all out again, its lanterns only, and its torch back. | Night, its house built, the target near it. |
| `respawn_its_bed` | 5 | A red bed laid along the left wall of the house it built, then quietly made their respawn point (no message): their next death wakes them in its house (`place_blueprint`, `set_respawn`). Once a stay. | Its house built. |
| `its_banner` | 4 | A black banner of its own hangs on the front of the house it built, beside the door, outside, facing out (`place_blueprint`, only into air). Once a stay. | Its house built. |
| `firework_over_its_house` | 4 | At night, near the house it built, a firework goes up from its roof and bursts high as a pale face (the game's creeper shape) (`firework`). | Night, its house built, the target near it. |
| `a_face_in_the_chest` | 3 | While they are away or asleep, their chest's grid is laid out as a face: what sits where the face goes moved aside into empty slots, then one item of their biggest stack into each of two eyes in the top row and a mouth of five along the third, nothing lost or added (`act::face_moves`). | The chest they use, a stack of seven or more in it and room to clear the face, the target away or asleep. |

### The ending

| Event | Stages | What happens |
| --- | --- | --- |
| `settle` | 5 | The ending's own script over three days, not a catalogue row: `core/src/ending.rs`. See [How it ends](#how-it-ends). |
| `wait` | all | Nothing. Always offered, always allowed. |

### Chains

One thing leads to the next. An event can name follow-ups in `events.rs`: events that, once it
has landed, are offered with extra weight, or offered *only* to the exclusion of the rest, for a
few days, and sometimes a place or an item the follow-up inherits. The memory keeps the open
chains (`Memory::chains`), the situation text says what is open (`It took the pickaxe three days
ago and has not given it back.`), and the model chooses with that in front of it. An open
chain's next step, while it may happen, always has a place on the menu as well as its weight,
and the first step of a chain not yet begun in the stay is weighed up the same: with a hundred
and more events, a weight alone left chains unopened and stalled. A chain is never forced:
`wait` is always offered, and a follow-up whose needs stopped being met falls out.
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
- **The woods.** `peeks_from_the_trees` → `tree_to_tree` → `branch_snaps` or `knocks_on_wood` →
  `the_tree_line` as they leave. [4 to 5]
- **The marked trees.** `blazed_trees` → `bare_trees` → `trees_close_behind`. [3 to 5]
- **The village wrong.** `trades_refused` → `villager_spins` → `villager_collapses` →
  `villagers_walk_out` → `villagers_flee_you` or `villager_stabs`. [3 to 5]
- **The sky.** `taken_up` → `sky_gives_back`. [4 to 5]
- **The night.** `drawn_to_the_light` → `while_unseen` → `wolves_circle` → `zombie_in_your_gear`.
  [4 to 5]
- **The visitor.** `villager_flees_in` → `prints_around_the_house` → `wall_bursts`. [4 to 5]
- **The glitches.** `dead_redstone` → `sound_warp`, once they noticed the redstone → `menu_freaks`.
  [3 to 5]
- **The knock.** `knock` → `door_scratch` → `knock_heavy` → `door_wont_open`.
- **The bed.** `egg_on_bed` → `sheets_changed` → `under_the_bed` → `foot_of_bed` → `lies_beside`.
- **The hands.** `by_the_bed` → `choke` → `burned_in_bed` or `shot_in_bed`.
- **The voice.** `says_back` → `repeats_you` → `says_first` → `whisper_from_you`.
- **The lights.** `flicker` → `lights_fail` → `candles_lit` → `lights_out`.
- **The sky wrong.** `thunder_clear` → `moon_wrong` → `stars_out` → `red_rain` → `night_early`.
- **The screen.** `hotbar_slip` → `crosshair_gone` → `inventory_stare` → `f3_line`.
- **The village.** `villager_at_door` → `golem_turned` → `village_doors` → `village_abandoned`.
- **The kills.** `kill_sign` → `count_up` → `head_on_post` → `grave_each`.
- **The water.** `ripples` → `boat_waiting` → `on_the_water` → `swims_beneath` → `pulled_under`.
- **The cave.** `lantern_ahead` → `cave_lit` → `cave_torches_flicker` → `drops_from_the_ceiling`.
- **The figures.** `crossing_ahead` → `in_a_line` → `ring_of_figures` → `every_window`.
- **The mirror.** `figure_mirror` → `reflection` → `wears_your_face` → `mirror_in_the_window`.
- **The death.** `death_light` → `where_you_died` → `death_message` → `false_death`.
- **The animals.** `stare` → `slow_approach` → `warped_animal` → `animals_turn`.
- **The field.** `scarecrow` → `row_withered` → `field_word` → `face_the_house`.
- **The chest.** `shuffle_chest` → `tidy_chest` → `chest_answers` → `chest_rattles` → `chest_wont_open`.
- **The steps.** `late_steps` → `footsteps_follow` → `breath` → `hand_on_shoulder` → `breath_close`.
- **The bow.** `arrow_near` → `arrow_in_the_door` → `arrow_grazes` → `arrow_hits`.

A chain's steps still obey cooldowns and caps, and the budget; what a chain changes is the
menu's weights and, for an exclusive step, the menu itself for a day or two. The scripted-stay
tests check that a stay with the rule brain walks at least two chains end to end and that no
follow-up is ever offered with its needs unmet.

**As coded (`core/src/chains.rs`).** Thirty-five, no event in two: the seven of the newer events
and the nineteen over the rest of the catalogue, as listed above, and the nine of the design, with
the catalogue's events: the thing (`take`,
`place_taken` or `return_taken`, `at_the_chest`, `frame_emptied`), the vein (`ore_marked`,
`mined_out`), the tunnel (`mining_sounds`, `mine_beside`, `tunnel_toward`, `cave_room`), the
place (`light_path`, `hole`, `pillar`, `sign`, `house_near`; its house grows in the ending's
script), the watcher (`far_figure`, then `behind_you` once they looked at a figure two seconds
or more, `on_the_roof`, `in_doorway`, `in_your_bed`, `copy_crouch`, `figure_waves`; chased, it
ends), the copy (`copy_build`, `copy_house`, `second_bed`), the house (`open_door`, then
`door_hinge`, `door_held` and `window_bars`, each once they noticed the last), the diary
(`diary_sold`, `diary_brought`), the pet (`pet_growl`, `pet_moved`, `pet_outside`). A chain opens when its first event lands; its steps
count in any order; its next step is four times likelier on the menu and in the rule brain's
pick for the days it waits (three to five; the diary's eight), its days held while its stage has not come; once
every step has landed it is walked; once a stay each. Not exclusive, and nothing carried from
one step to the next yet. The scripted stay walks the place end to end: the scripted player
notices nothing and keeps no pet, so the rest wait on what a player does.

### Provocation and temper

The lodger answers how it is treated (owner). The memory keeps a **temper**, 0 to
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
`temper.rs`, with its tests.

**As coded (`core/src/temper.rs`, `events.temper`).** It rests at 0.2 on `calm`, 0.3 on
`balanced`, 0.4 on `intense`, 0.5 on `relentless`, and one cause moves it three times a day at
most. Up: chasing the figure (eight blocks toward it, or near enough that it goes; +0.15),
lighting its place (+0.12), TNT, lava, water or fire by it (+0.15), breaking its lights (+0.08),
its house (+0.1) or a thing it made (+0.08), opening its house's chest (+0.05), cursing it in
chat (+0.1). Down: building at its place (-0.03), leaving a chest there (-0.1), watching the
figure two seconds or more from afar (-0.05), kind words in chat (-0.03). Each day it drifts a
tenth of the way back. The pace: the day's events times `(0.5 + temper) / (0.5 + resting)`,
between half and half again, so a stay at rest runs as its preset says; the stages, the menu and
the distance as above; the ending, when `stages.ending` is `difficulty`, takes on any difficulty
above 0.9 and keeps rather than leaves above 0.7 though the want was met; and the fight (owner:
"harder fight when angry"): its health up to a quarter more and a fifth wave from halfway between
resting and furious (`confront`'s `fierce`, ABI 48). The situation says it far from rest: `It is
angry: the player broke its lights and chased the figure.` Not yet: the kills of an animal it
moved, a name given back, being ignored night after night, the thing left where it put it,
standing at its sign, and the `How does it take this?` ask.

### How often

Events are rare, and get less rare. Each stage has a budget of events per in-game day
(`stages.events_per_day`, by default one a day in stage one, two in stage two, four in stage
three, six in stage four), and the brain is asked to plan only while the day has budget left.
Sounds and the screen's glitches count half, and a small event (one they may not notice) a quarter (`events.small_cost`; `calm` counts it whole), so the house drifts more while the noticed things keep their pace. A day with budget left at its end does not carry it
over; a quiet day is a quiet day. So a whole stay of about two in-game weeks holds some forty to
sixty events, one most evenings from the start and several a day in the last days before the
end. That is the denser of the two paces considered (the other was half of it over three weeks;
the owner picked this one), and `events.cooldown_scale` and `stages.events_per_day`
move it either way.

Repetition is held off three ways, all in `events.rs` and `brain/offer.rs`:

- **Cooldowns.** Each event has one (`cooldown_days`: two days for most small things, three for
  most the target would notice, a day or less for many sounds and for `open_chest`), and is not
  offered again until it has passed; one that did not land waits it out at that place.
- **Caps.** Each event has a most-per-stay (`most_per_stay`: two for most things the target
  would notice, three or four for most small ones; `take`, the far figure and the night-by-night
  builds more; some thirty once, `field_word`, `copy_build`, `in_your_bed` and `other_house`
  among them). Past its cap it is gone for the stay. With nearly three hundred events and forty
  to sixty in a stay, most happen once or never.
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
reads the house; owner: "the home's, placed or not"), or one they opened on three
different days.
A chest opened once on the way past, a village's or a dungeon's, is never touched and never
described to the model (owner, after a village chest became "the chest opened
most"). What it holds is read when it is opened and again when it is closed, so what the
target took out is known.

### The distance rule

No event but the sounds, the glitches, the screen, the chat and the player list, the sleep
events (`refuse_rest`, `bed_occupied`, `wake`, `woke_elsewhere`, `bed_breaks`), the creatures
that stare or come at them (`stand_watch`, `stare`, `pack`, `hostiles_stare`, `hostiles_close`,
the approaches), the figures, and what is meant to be noticed as it happens (`open_chest`,
`open_on_enter`, `door_opens_itself`, `chest_answers`, `tunnel_toward`, `torches_far_end`, the
diary's villager) happens within
`events.min_distance` blocks, 64 by default (24 until the owner watched a path of torches go
up from farther off), of the target while the target faces the spot. Every block
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
- **As coded (`core/src/multiplayer.rs`).** The pick as above. The target's last sighting is kept
  by the real clock (`Memory::target_seen_unix`); at a stage's beginning from stage two, gone
  `multiplayer.absent_days` and someone else here, the first other here it knows longest is the
  target, once a stay, its own things kept and the old target's let go. With
  `multiplayer.witness_traces` off, a change to the world within 48 blocks of another player
  waits until they go. The debug panel's players line says who it lives with and who else is
  here.
- **The target's own house, not the server's.** The home is where *the target* sleeps and keeps
  things. A shared base is a home with other people in it, and the lodger works in the target's
  rooms, by the chests the target opens. Witnesses' chests are never touched.

## How it ends

Stage five is three days, and it is the point of the stay. Everything before it was quiet;
this is not. Which ending depends on `stages.ending`, the world's difficulty when it reads
`difficulty`, and whether the want was met, and every ending is a script of three days that
ramps from the first evening to the last night, with the chains above coming to their ends
inside it. The script is data in `ending.rs`, each day a list of beats with a time and a
condition; the brain's part is small (which of two ways a beat goes), and the rules still hold
every step but the one death.

| Ending | When | Day one | Day two | The last night |
| --- | --- | --- | --- | --- |
| **Leaves** | The want was met, on any difficulty; or `stages.ending` is `leave` | Everything it took comes back over the day, one thing an hour, each to where it was taken from. The animals it moved come home. Its torches go out one by one along the path, from the far end toward the house. | Its house (if it built one) is emptied: the chest's contents are on your doorstep at dawn. The figure stands at its door at noon, in plain light, for a full minute, and goes in. At dusk its torch goes out. Every pet of yours sits facing its house. | The music plays, the first time in a week. Every light in your house flickers once at midnight and steadies. At the edge of the light, the figure, in your skin, for as long as you look. When you look away and back, nothing. At dawn its house is gone, the ground flat, one torch where its door was. `/craftkit lodger status` says `left`. |
| **Takes** | The want was refused, on normal or hard; or `stages.ending` is `take` | You cannot sleep (`You may not rest now`, all night). Every door in the house opens at midnight at once, and the pets go out. The animals stare at the house all day. The sun blinks out three times before noon. | Every frame is empty, the items on your bed, in a line pointing at the door. Every sign in the house says the same word. The figure is behind you on every route, closer each time, to 32 blocks. Your tunnels are all sealed with cobblestone. The village bell rings every hour. | Every torch it ever placed lights at once, the whole path to its place, and every torch of yours goes dark in the client's render. Red rain. The lights of its house are on. At its place, the figure, holding everything it took, which it keeps. It walks toward you for the first time, at a player's pace, and stops at one block. On hard, you die there, once, with your inventory kept, and the death message is vanilla's for falling out of the world. Then the night is ordinary, the house is as it was minus what it took, and it is gone. |
| **Keeps** | The want was refused, on easy or peaceful; or `stages.ending` is `keep` | Its house gets a second room overnight, a copy of your bedroom, your spare bed in it. The far figure is in its window every hour. | A path of cobblestone from your door to its door. A gate in its fence facing your door, open. Your pets go and sit at its door and come back. | At midnight, its door is open and lit, and the figure is in its doorway, facing you, for as long as you look. It does not come. At dawn it stops acting, for good. The torches stay lit, the signs stay, the things stay where they were put, its house stays with its door open. The music comes back. Every few in-game weeks, one door of yours is open. `/craftkit lodger status` says `stayed`. |

Within each day the beats are ordered and timed, but which of two forms a beat takes (the figure
at its door or on its roof; the frames emptied or the paintings turned) is the model's, asked
once per day with the day's beats as the choices, so two players' endings differ in detail and
agree in shape. Nothing in stage five repeats a thing from the same day, and nothing in it is
small: the budget is off, the cooldowns are off, and the caps alone hold, so a stay's last night
can spend everything it has left.

**As coded (`core/src/ending.rs`).** The ending is chosen on stage five's first
tick (`stages.ending`, or with `difficulty` the want met leaves, refused takes on normal and hard
and keeps on easy and peaceful; the want read as: three blocks of theirs within five of its
place, or a chest of theirs there with something in it; three things it showed them looked at
and never run from; their favourite left in the chest it uses). The script starts at the next
dawn and its beats run in order as the clock reaches them; a moment (a sound, a sight, the
screen) that comes more than two thousand ticks late is skipped, a change to the world is done
late. Where it differs from the table:

- **Leaves.** What it holds goes back (it holds one thing at a time, so one, not one an hour);
  its torches go out one by one, the farthest from the house first (the lights it placed itself
  are kept); the chest of its house is spilled on their doorstep
  (`spill_container`, ABI 41); the figure at its door a full minute, then in; their pets held
  facing its house ten minutes (held, not sitting); its torch out at dusk; the music (the
  game's music as a sound); the flicker; the figure at the edge of the light, gone once looked
  away from; at dawn its house to air, its floor to grass, its chest taken, a torch where its
  door was. The animals it moved are not walked home: what it moved is not kept.
- **Takes.** Their animals stare at the house (twice that day); the sun out three times before
  noon; no sleep that night; every door of the house open at midnight and their pets put out;
  every frame emptied onto the bed in a line toward the door, the very items; one line of every
  sign of theirs says the want's word (`here`, `again`, `yours`; the line it replaced is not
  given back); the figure behind them at 64, 48 and 32 blocks; their tunnels walled at the mouth
  with cobblestone; the village bell every hour of the day; on the last night its torches lit,
  theirs dark on their screen alone (`hide_block`, five minutes), red rain; the second visit to
  the other house (see `other_house`); the figure from its place, holding what it holds, walks
  up to a block from them at a walk; on hard (on normal too with `intense`, on any difficulty
  with `relentless`), one death as if fallen out of the world (`kill_player`, ABI 41: what they carry held in the world's data
  and given back as they respawn, their experience with it); at dawn the doors shut again.
- **Keeps.** The second room of its house (an empty room, no copy of their bedroom); the figure
  in its window every two hours of the night; a cobblestone path from their door to its door
  over the ground's own top; their pets at its door at noon and home two hours on; at midnight
  its door open, a torch beside it, the figure in the doorway until they look away; at dawn it
  stops for good, and every three in-game weeks after, while they are away or asleep, the door
  they use is found open. It has no fence, so no gate; the music is not brought back.
- The model does not yet choose between two forms of a beat. The stay's status line (the debug screen) says `the stay is over: it left`,
  `took` or `stayed`; `/craftkit lodger status` says so.

**The fourth ending: you confront it** (owner). In stage five, at night, knock on its
door (strike it twice), or with no house stand at its place, and it rises out of the ground
before you. One boss bar in your name over three stages, with a bend between each: warped animals
in waves while it watches from its roof (none on peaceful); it drops onto all fours and crawls at
you, stands too tall with its head backwards, and duels you in your skin and a copy of your best
gear, strafing, sprint-hitting, pearling behind you, twitching, upside down a second at a time; at
three fifths it shakes and copies of you peel off it, one real; at seven twentieths they crawl
together and sink, and its blocks rise out of the ground into a construct with your sword, its
knees and elbows bending the wrong way, slamming the ground, an arm torn off and grown back as two
at its first third. Win and it falls apart into its blocks, the figure looks at you and sinks, and
you have its lantern (`<you>'s light`), its head (`the lodger`) and its weapon (`what it
learned`, an enchantment more), with what it held; the stay ends `beaten`. Lose and you die once
keeping everything, and it leaves with what it took (`kept`). Run, and it can be come to again
the next night. Its health and its hits are scaled to your armour, your weapon and the
difficulty. A beacon on its roof, lit only at night from stage five's first dusk, shows the way.

After any ending the stay is closed in `lodger.dat` and no lodger moves into that world again
unless `/craftkit lodger reset`. The one death the hard ending deals is the only kill the lodger ever makes
(the harm before it is spared, never below half a heart), it is at the end of a stay of two weeks, and it keeps the inventory: the point is that it happened,
not that it cost anything. `stages.ending` is there for servers that want none of it.
