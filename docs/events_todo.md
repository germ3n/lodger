# Events to come

Eighty event ideas not yet in the catalogue (forty at first, forty more after) (`core/src/events.rs`, `docs/entity.md`), none of
them a copy of one there or of a line in `docs/ideas.md`. Each says the stage it fits, what the
target must be doing, what it would take (the commands there are today, or what the host would
need), and a rough size: **S** (the mind alone, today's commands), **M** (a small host change),
**L** (a new host system). The owner picks which to build; picked ones move to `docs/todo.md`.

| # | Event | Stage | What happens | Needs | Takes | Size |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `ring_of_figures` | 4 | **Built.** At night, while they are in the house, six to eight figures in their skin stand in a ring around it, twenty blocks out, facing it, still. Each is gone the moment they look straight at it; step outside and they are all gone at once. (Asked for by the owner.) | Home, night, the target indoors. | `figure` with several numbers, each on the ground, `once_seen`; the ring's spots from the home's center. | S |
| 2 | `upside_down` | 4 | **Built.** They wake and the figure hangs from the ceiling over the bed, upside down, facing them; gone when they get up. | The target waking in bed. | The figure's profile named as the game flips a player (Dinnerbone), standing on the ceiling block's underside. | M |
| 3 | `the_crawl` | 4 | **Built** (from their door, seen from inside). Through the window at night, a figure crawls across the floor of their house toward the bed, flat on its belly, and is gone at the bed. | Night, the target indoors, a bed. | A figure in the swimming pose on land (the crawl), gliding along a path. | M |
| 4 | `hunted` | 5 | **Built.** Far off in the dark a figure starts running at them, flat out, and is gone two blocks short with a hit's jolt and no harm. | Night, the target outdoors. | The new glide at a sprint, then `figure_swing` with no damage. | S |
| 5 | `boat_passenger` | 4 | **Built.** Rowing alone, they feel the boat sit lower: a figure in the seat behind them, for as long as they do not turn round. | The target in a boat alone. | The figure set as a passenger of their boat for their client alone (the set-passengers packet). | M |
| 6 | `every_window` | 5 | **Built.** At night every window of the house has a figure outside it at once, looking in, all gone together when they look at any one. | Night, the target indoors, two or more windows. | `figure` per pane from the home read, one shared vanish. | S |
| 7 | `one_more_each_night` | 4 | **Built.** At the edge of their light one figure stands the first night, two the next, three the next, never closer. | Night, a light near them, three nights. | `figure` at the light's edge; a counter in the memory. | S |
| 8 | `door_wont_open` | 4 | **Built** (named apart from `door_held`, which shuts a door again once open). Their own door will not open for a few seconds, as if held from the other side, and a knock comes through it. | The target at the home's door from inside. | The door kept shut a few seconds against their use, then a knock sound. | M |
| 9 | `sleepwalk` | 5 | **Built.** They wake standing outside their front door at night, the door open behind them, footprints in the snow or a worn path from the bed. | The target in bed. | `cancel_sleep`, `move_player` out of the door, `toggle_block`, `prints` or a worn path. | S |
| 10 | `torch_flash` | 4 | **Built.** In a cave, the torch they just placed lights a figure right in front of it, for a blink. | A torch placed underground. | `figure` for a tenth of a second at the new light, on the senses' placed-block report. | S |
| 11 | `where_you_died` | 3 | **Built.** At the spot they last died a figure lies on the ground as a body would, and is gone when they come within ten blocks. | A death remembered, the target near it. | A figure in the sleeping pose on the ground. | M |
| 12 | `rubberband` | 4 | **Built.** As they walk they are set back three blocks, as on a bad connection, and a figure stands where they had been. | The target walking outdoors. | `move_player` back along the trail, `figure` at the spot. | S |
| 13 | `the_offer` | 4 | **Built** (what it took and keeps: it never holds what they lose dying). A figure in their doorway holds out what they lost when they last died; walk to it and it drops at their feet, and it is gone. | A death with items lost, the target near home. | `figure` holding the item, a drop kept under a name, vanish within. | M |
| 14 | `top_of_the_ladder` | 4 | **Built.** Going down their mine shaft, they look up: a figure at the top looking down at them. Gone when they climb. | The target on a ladder underground. | `figure` at the shaft's mouth, `vanish_within`. | S |
| 15 | `pulled_under` | 4 | **Built.** Swimming, they are pulled down two blocks, bubbles and the drowning sound, once, and let go. | The target swimming in deep water. | A short downward push of the player, a sound. | M |
| 16 | `its_lights_on` | 4 | **Built** (dark, then lit one by one, then out, on a timer rather than their gaze). The house it built near theirs: when they look at it at night, its windows are lit, one by one; look away and they are dark. | `house_near` built, night. | Light blocks set and hidden in its house (`hide_block` the other way). | M |
| 17 | `a_face_in_the_chest` | 3 | **Built.** Their chest's stacks are laid out in the grid as two eyes and a mouth. | A chest they use with enough stacks. | `move_item` within the chest to a face pattern. | S |
| 18 | `falls_from_the_sky` | 5 | **Built.** Looking up at night, they see a figure falling from high above, landing far off with a thud; gone where it fell. | Night, the target outdoors, looking up. | A figure gliding straight down, a landing sound there. | M |
| 19 | `pet_follows` | 4 | **Built.** Their dog or cat gets up and walks off toward a figure standing at the tree line; the figure is gone, and the pet comes back. | A pet near them, outdoors. | `figure` far, `approach` of the pet to it and back. | M |
| 20 | `lies_beside` | 5 | **Built.** Asleep, their eyes held open as in `choke`: a figure lies down on the floor beside the bed, facing them, and lies there until they get up. | The target in bed. | The bedside walk of `choke`, then the sleeping pose on the floor. | M |
| 21 | `inventory_stare` | 4 | **Built.** In the inventory screen their character stops following the mouse and stares straight out at them, for a few openings. | Any time. | A client hook on the inventory's paper doll. | M |
| 22 | `false_death` | 5 | **Built.** For a second the death screen, "You died!", then the game as it was, nothing lost. | The target walking, no danger near. | A client hint drawing the death screen over the game. | M |
| 23 | `respawn_its_bed` | 5 | **Built** (its house had no bed: it lays one first). Their respawn point is quietly moved to the bed in the house it built; the next death, they wake there. | `house_near` built with a bed. | Their respawn position set to its bed. | M |
| 24 | `ender_chest_note` | 4 | **Built.** A note in their ender chest, the one place nothing else can reach. | They own an ender chest. | A written book put into their ender chest's own inventory. | M |
| 25 | `eye_turns_home` | 5 | **Built.** An eye of ender they throw flies toward their house instead of the stronghold, once. | The target throwing an eye of ender. | The thrown eye's target changed for that throw. | M |
| 26 | `crossing_ahead` | 3 | **Built** (ahead of where they look; any walk, not only their route). Far ahead on their route a figure walks across their path, left to right, never looking at them. | The target walking their route. | `figure` with a path across the route ahead. | S |
| 27 | `in_a_line` | 4 | **Built.** Along their route, figures stand single file, all facing the house; each is gone as they pass it. | The target on their route, dusk or night. | `figure` per spot along the route, `vanish_within`. | S |
| 28 | `peeks_from_ground` | 4 | **Built.** Far off, only a figure's head shows above the grass, as if it stood in a hole; it sinks out of sight as they look. | The target outdoors on open ground. | A figure stood a block and a half into the ground, a short path down. | S |
| 29 | `wears_your_face` | 4 | **Built.** At night a zombie walks toward them wearing their own head; killed or gone at dawn, the head goes with it. | Night, a zombie near them. | A real zombie equipped with a head of their skin, taken back after. | M |
| 30 | `animal_follows` | 3 | **Built.** One of their animals follows them all day at a distance, stopping when they stop, wherever they go. | An animal of theirs near them. | `approach` of the animal, again and again, for a day. | M |
| 31 | `held_breath` | 4 | **Built.** Underwater, their air runs out twice as fast, once, and nothing is there. | The target underwater. | Their air supply drawn down faster for one dive. | M |
| 32 | `all_freeze` | 4 | **Built** (facing them as they stop). Every animal and mob around them stops mid-step at once for three seconds, then carries on. | Mobs near them. | `hold` on each, all together. | S |
| 33 | `figure_on_map` | 4 | **Built.** On a map they carry, a player marker stands at their house while they are away from it. | A map of the home in their hands, the target away. | A marker on their map's decorations, for them alone. | M |
| 34 | `waits_at_the_portal` | 4 | **Built.** Coming back from the Nether, a figure stands beside their portal, facing it, as if it had waited. | The target back through their portal. | `figure` beside the portal on their return (`CameBack`). | S |
| 35 | `firework_over_its_house` | 4 | **Built** (a pale face: the game never says their skin's colours). At night a firework goes up over the house it built, bursting in the colours of their skin. | `house_near` built, night, the target outdoors. | A firework launched there, its colours from their skin. | M |
| 36 | `chicken_lays` | 3 | **Built.** One of the chickens near their house lays something of theirs instead of an egg. | Chickens near the home, something taken and kept. | A drop of the kept stack at the chicken, with the laying sound. | S |
| 37 | `handprints` | 4 | **Built** (client dust, facing them). On the inside of a window, handprints, as if pressed from inside the house while they were out. | The target coming home, a pane. | Client particles or a decal on the pane, for them alone. | M |
| 38 | `in_the_screenshot` | 5 | **Built** (ahead in their view, not behind: a first-person picture never shows behind). A screenshot they take (F2) shows the figure behind them, though it was never on their screen. | The target taking a screenshot. | A client hook drawing the figure into the screenshot's frame alone. | M |
| 39 | `only_in_third_person` | 4 | **Built.** In third person (F5) a figure stands right behind them; in first person there is nothing. | The target switching to third person. | A figure drawn only while the camera is detached. | M |
| 40 | `f3_line` | 5 | **Built.** The debug screen (F3) has one more line among the others: "Lodger: watching". | The target opening F3. | A client hook on the debug screen's lines. | M |

## The next forty

The owner, once the first forty were built: "do another 40". Checked against the catalogue,
`docs/ideas.md` and the first forty; three that `ideas.md` already had were replaced.

| # | Event | Stage | What happens | Needs | Takes | Size |
| --- | --- | --- | --- | --- | --- | --- |
| 41 | `at_their_table` | 4 | **Built.** Coming home, through the window: a figure at their crafting table, working at it; gone as they reach the door. | Coming home, a crafting table in the house. | `figure` swinging, `vanish_within`. | S |
| 42 | `jumping_far` | 4 | Far off in the open a figure jumps, again and again, in place, facing them. | Outdoors. | A figure gliding up and down. | M |
| 43 | `waves_from_inside` | 4 | As they come home, a figure inside the house at a window waves at them; gone when they open the door. | Coming home, a window. | `figure` inside the pane, `figure_swing`. | S |
| 44 | `prints_to_the_cave` | 4 | In the snow, prints from their door to the mouth of the nearest cave. | Snow, a cave known. | `prints`. | S |
| 45 | `rides_their_horse` | 4 | Far off, their horse stands with a figure in its saddle, on their client alone. | Their horse in sight. | The figure set as the horse's passenger for them. | M |
| 46 | `eats_at_their_chest` | 4 | Coming in, a figure crouched at their chest eating their food, the sound of it; gone when looked at. | Coming home, food in their chest. | `figure` using a food item. | M |
| 47 | `mirror_in_the_window` | 5 | From outside, in their own window, a figure copies their every move. | Outdoors near the house, a window. | `figure` inside the pane with `mirror`. | S |
| 48 | `every_door_opens` | 5 | At night, inside, every door of the house opens at once. | Night, the target indoors, two doors or more. | `toggle_block` on each. | S |
| 49 | `flowers_wilted` | 3 | The flowers they planted are dead bushes for a day, then flowers again. | Flowers of theirs. | `swap_block` and back. | S |
| 50 | `camp_nearby` | 4 | At night, forty blocks off, a campfire with log seats and a figure sat by it; gone by morning. | Night, outdoors. | `place_blueprint`, a figure, the blocks taken away at dawn. | M |
| 51 | `stone_circle` | 4 | Overnight, a ring of stone pillars stands in a clearing far off. | The target asleep or away. | `place_blueprint`. | S |
| 52 | `slips_from_hand` | 4 | What they hold slips from their hand and lies at their feet. | Holding something. | The held stack dropped at their feet. | M |
| 53 | `glass_breaks` | 4 | Inside, the sound of a window breaking; every pane is whole. | Indoors, a pane. | `play_sound`. | S |
| 54 | `xp_slips` | 4 | Their experience bar drains to nothing on their screen, then is back. | Experience. | A client hint. | M |
| 55 | `hearts_drop` | 5 | For a second their hearts show half empty, then whole. | Any time. | A client hint. | M |
| 56 | `pet_stares_at_wall` | 3 | Their pet stands facing a blank wall of the house, still, for a minute. | A pet indoors. | `hold` facing a wall. | S |
| 57 | `bats_in_the_house` | 4 | At night bats flutter about inside the house for a minute, then are gone. | Night, the target indoors. | Bats for a while. | M |
| 58 | `lost_hour` | 5 | A blink of black, and an hour of the day is gone. | Daytime. | The time moved on, a hint. | M |
| 59 | `chest_wont_open` | 4 | Their chest will not open for a few seconds, and something knocks inside it. | Walking to the chest they use. | A container held shut, a sound. | M |
| 60 | `name_in_the_snow` | 4 | Near the house, their name trodden into the snow. | Snow near the home. | Snow layers taken away in letters. | M |
| 61 | `torch_ring` | 4 | Overnight, a ring of torches round the house, twelve blocks out. | The target asleep or away. | `set_block` torches. | S |
| 62 | `breadcrumbs` | 4 | A line of single blocks of theirs leads from their door off into the dark. | A stack of theirs in the chest. | `take_item`, `drop_item` one at a time. | S |
| 63 | `fishing_figure` | 3 | Where they fish, a figure sits fishing, its line in the water. | Where they fished known. | `figure` holding a rod, swinging. | S |
| 64 | `by_the_bed` | 4 | Coming in at night, a figure stands beside their bed, facing it; gone when looked at. | Night, coming home, a bed. | `figure` beside the bed, `once_seen`. | S |
| 65 | `swims_beneath` | 4 | Swimming, a figure swims past beneath them. | Swimming in deep water. | A figure crawling (swimming) along under them. | S |
| 66 | `stands_in_the_rain` | 3 | In the rain, a figure stands in the open with its face to the sky. | Raining, outdoors. | `figure`, facing up. | M |
| 67 | `its_banner` | 4 | A banner of its own hangs on its house's door, black, a skull on it. | Its house built. | `set_block` a banner with patterns. | M |
| 68 | `chest_rattles` | 3 | Their chest's lid opens and shuts five times fast, then is still. | A chest of theirs near them. | `open_lid` in a row. | S |
| 69 | `trapdoor_taps` | 3 | A trapdoor of theirs taps open and shut, again and again. | A trapdoor of theirs. | `toggle_block` in a row. | S |
| 70 | `climbs_ahead` | 4 | Climbing their shaft, a figure climbs ahead of them, above, and is gone at the top. | On a ladder of theirs. | A figure gliding up the shaft. | S |
| 71 | `on_the_water` | 4 | Far off, a figure stands on the surface of the water. | Water near them. | `figure` on the water's top. | S |
| 72 | `in_the_treetops` | 4 | Far off, a figure stands on top of the trees. | Outdoors, trees near. | `figure` on the canopy. | S |
| 73 | `path_through_the_trees` | 5 | Overnight, a straight line cut through the leaves of a forest, pointing at their house. | Trees near the home. | `set_block` air over leaves. | M |
| 74 | `rides_the_cart` | 4 | A minecart of theirs rolls past with a figure sitting in it. | A minecart of theirs. | The figure the cart's passenger for them, a push. | M |
| 75 | `eyes_in_the_dark` | 4 | In a dark cave, two small lights at eye height ahead, blinking; gone when they come near. | Underground, dark. | A client hint of particles. | M |
| 76 | `lantern_in_the_woods` | 4 | At night, far off in the trees, a lantern moving, held by a figure walking. | Night, outdoors. | `figure` walking, holding a lantern. | S |
| 77 | `crouched_behind_a_tree` | 4 | A figure crouched behind a tree near them; gone when they come round it. | Outdoors near trees. | `figure` crouching, `vanish_within`. | S |
| 78 | `snowball_thrown` | 3 | A snowball hits their back; no one is there. | Snow, outdoors. | A snowball thrown at them from behind. | M |
| 79 | `arrow_in_the_door` | 4 | While they were out, an arrow struck in their front door. | Away, a door. | `shoot` at the door. | S |
| 80 | `raises_their_sword` | 4 | Far off, a figure holds their own sword up high, still. | Their weapon known. | `figure` holding it, arm up. | S |

