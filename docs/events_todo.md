# Events to come

Twenty event ideas not yet in the catalogue (`core/src/events.rs`, `docs/entity.md`), none of
them a copy of one there or of a line in `docs/ideas.md`. Each says the stage it fits, what the
target must be doing, what it would take (the commands there are today, or what the host would
need), and a rough size: **S** (the mind alone, today's commands), **M** (a small host change),
**L** (a new host system). The owner picks which to build; picked ones move to `docs/todo.md`.

| # | Event | Stage | What happens | Needs | Takes | Size |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `ring_of_figures` | 4 | At night, while they are in the house, six to eight figures in their skin stand in a ring around it, twenty blocks out, facing it, still. Each is gone the moment they look straight at it; step outside and they are all gone at once. (Asked for by the owner.) | Home, night, the target indoors. | `figure` with several numbers, each on the ground, `once_seen`; the ring's spots from the home's center. | S |
| 2 | `upside_down` | 4 | They wake and the figure hangs from the ceiling over the bed, upside down, facing them; gone when they get up. | The target waking in bed. | The figure's profile named as the game flips a player (Dinnerbone), standing on the ceiling block's underside. | M |
| 3 | `the_crawl` | 4 | Through the window at night, a figure crawls across the floor of their house toward the bed, flat on its belly, and is gone at the bed. | Night, the target indoors, a bed. | A figure in the swimming pose on land (the crawl), gliding along a path. | M |
| 4 | `hunted` | 5 | Far off in the dark a figure starts running at them, flat out, and is gone two blocks short with a hit's jolt and no harm. | Night, the target outdoors. | The new glide at a sprint, then `figure_swing` with no damage. | S |
| 5 | `boat_passenger` | 4 | Rowing alone, they feel the boat sit lower: a figure in the seat behind them, for as long as they do not turn round. | The target in a boat alone. | The figure set as a passenger of their boat for their client alone (the set-passengers packet). | M |
| 6 | `every_window` | 5 | At night every window of the house has a figure outside it at once, looking in, all gone together when they look at any one. | Night, the target indoors, two or more windows. | `figure` per pane from the home read, one shared vanish. | S |
| 7 | `one_more_each_night` | 4 | At the edge of their light one figure stands the first night, two the next, three the next, never closer. | Night, a light near them, three nights. | `figure` at the light's edge; a counter in the memory. | S |
| 8 | `door_held` | 4 | Their own door will not open for a few seconds, as if held from the other side, and a knock comes through it. | The target at the home's door from inside. | The door kept shut a few seconds against their use, then a knock sound. | M |
| 9 | `sleepwalk` | 5 | They wake standing outside their front door at night, the door open behind them, footprints in the snow or a worn path from the bed. | The target in bed. | `cancel_sleep`, `move_player` out of the door, `toggle_block`, `prints` or a worn path. | S |
| 10 | `torch_flash` | 4 | In a cave, the torch they just placed lights a figure right in front of it, for a blink. | A torch placed underground. | `figure` for a tenth of a second at the new light, on the senses' placed-block report. | S |
| 11 | `where_you_died` | 3 | At the spot they last died a figure lies on the ground as a body would, and is gone when they come within ten blocks. | A death remembered, the target near it. | A figure in the sleeping pose on the ground. | M |
| 12 | `rubberband` | 4 | As they walk they are set back three blocks, as on a bad connection, and a figure stands where they had been. | The target walking outdoors. | `move_player` back along the trail, `figure` at the spot. | S |
| 13 | `the_offer` | 4 | A figure in their doorway holds out what they lost when they last died; walk to it and it drops at their feet, and it is gone. | A death with items lost, the target near home. | `figure` holding the item, a drop kept under a name, vanish within. | M |
| 14 | `top_of_the_ladder` | 4 | Going down their mine shaft, they look up: a figure at the top looking down at them. Gone when they climb. | The target on a ladder underground. | `figure` at the shaft's mouth, `vanish_within`. | S |
| 15 | `pulled_under` | 4 | Swimming, they are pulled down two blocks, bubbles and the drowning sound, once, and let go. | The target swimming in deep water. | A short downward push of the player, a sound. | M |
| 16 | `its_lights_on` | 4 | The house it built near theirs: when they look at it at night, its windows are lit, one by one; look away and they are dark. | `house_near` built, night. | Light blocks set and hidden in its house (`hide_block` the other way). | M |
| 17 | `a_face_in_the_chest` | 3 | Their chest's stacks are laid out in the grid as two eyes and a mouth. | A chest they use with enough stacks. | `move_item` within the chest to a face pattern. | S |
| 18 | `falls_from_the_sky` | 5 | Looking up at night, they see a figure falling from high above, landing far off with a thud; gone where it fell. | Night, the target outdoors, looking up. | A figure gliding straight down, a landing sound there. | M |
| 19 | `pet_follows` | 4 | Their dog or cat gets up and walks off toward a figure standing at the tree line; the figure is gone, and the pet comes back. | A pet near them, outdoors. | `figure` far, `approach` of the pet to it and back. | M |
| 20 | `lies_beside` | 5 | Asleep, their eyes held open as in `choke`: a figure lies down on the floor beside the bed, facing them, and lies there until they get up. | The target in bed. | The bedside walk of `choke`, then the sleeping pose on the floor. | M |
