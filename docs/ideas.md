# Ideas

A brainstorm, not a plan. Everything the lodger could do, written down so the catalogue in
[The lodger](entity.md#events) has a well to draw from when a stage feels thin or a situation has
nothing offered for it. An idea moves to the catalogue by getting a row there (id, stages, what
happens, needs, cooldown, cap), a row in `events.rs`, an executor if it needs a new command, and
a test. Until then it is an idea.

Each line: a working id, what the player experiences, and in brackets the stage it might first be
offered in and what it needs. The rules in [What it never does](good-to-know.md#what-it-never-does)
hold for all of them: nothing a player placed is destroyed, nothing is spawned (everything placed
was dug, cut, picked or taken first), no one is harmed before the ending, nothing is written but
the want's words, nothing happens in view within 24 blocks, and the world's time and weather are
never touched (the sky things are the target's client alone). Ideas that would break a rule are
in [Set aside](#set-aside) with why.

## Containers and items

- `open_chest` a chest opens behind you and shuts before you turn [1; a used chest]
- `shuffle_chest` same items, different slots [1]
- `stack_split` one stack is two [1]
- `stack_merged` two half stacks are one [1]
- `tidy_chest` sorted by kind [2]
- `tidy_by_colour` wool, glass, beds sorted by colour [3]
- `tidy_alphabetical` sorted by name, which no player does [3]
- `mirror_chest` the slots mirrored left to right [2]
- `one_slot_over` every stack shifted one slot right, the last wrapping to the first [2]
- `swap_stacks` two stacks trade chests [2]
- `fill_gap` an item is in a chest it never was [2]
- `take` one favourite item is gone; it holds it [2]
- `return_taken` the item is back, exactly [2]
- `return_one_over` back, one slot over [2]
- `place_taken` the item is placed where the want says [2]
- `take_two_return_one` two of a kind gone, one back the next day [3]
- `tool_used` a tool lost durability overnight [3; a damageable favourite]
- `tool_mended` a damaged tool is at full durability, as a mending enchantment would do over a day of use [3]
- `enchant_gone` one enchantment is gone from a tool [4; a grindstone]
- `renamed_tool` the favourite tool has a name from the want's list, as from an anvil [3; an anvil and levels... taken from nothing, so see Set aside]
- `book_word` a book and quill has one word in it [3]
- `book_pages` a written book's pages are in another order [3]
- `map_marked` a map shows a marker at its place [3]
- `map_turned` a map is rotated in its frame [2]
- `compass_in_chest` a compass is in the chest that was not there... (from nothing: Set aside) 
- `bundle_emptied` a bundle's contents are loose in the chest around it [2]
- `hopper_emptied` hopper contents moved to the chest it feeds [3]
- `hopper_reversed` the chest's contents are in the hopper [3]
- `shulker_moved` a shulker box is on another wall [3]
- `shulker_colour` a shulker box is dyed another colour [3; dye]
- `barrel_turned` a barrel faces another way [2]
- `locked_chest` nothing visible: the chest refuses to open once, with the lock sound, then opens [3; client-side sound and a one-tick cancel]
- `chest_minecart_arrives` a chest minecart the target owns rolls to the mouth of the mine holding what was in the mine's chest [3]
- `armor_swapped` two armour pieces in a chest swapped slots with each other [1]
- `food_cooked_in_chest` the raw meat in the chest is cooked, the furnace fuel one fewer [2]
- `arrows_counted` an odd number of arrows is even [1]
- `potions_lined` potions in a chest arranged by colour [3]
- `dye_sorted_rainbow` dyes in a chest in rainbow order [3]
- `eggs_hatched` the eggs in a chest are gone; one more chicken in the pen (from an egg, so not spawned) [3]
- `seeds_planted` the seeds in a chest are gone; the field has a new row [3]
- `saplings_planted` saplings gone; a line of saplings toward the place [3]
- `one_of_everything` one of every stack in the main chest is in the chest by the bed [3]
- `nothing_moved_but_the_lid` the chest's lid is left half open in the client's render until looked at [2; glitch]

## Furnaces, workstations and crafting

- `cook_one` one raw thing is cooking [1]
- `take_output` the output is gone to the chest [2]
- `swap_fuel` another fuel is burning [2]
- `cold_furnace` the fuel is gone mid-smelt [3]
- `furnace_lit_empty` a furnace burns with nothing in it, one coal's worth, then stops [2]
- `blast_vs_smoker` the thing in the furnace is in the blast furnace, or the smoker, whichever fits it [2]
- `campfire_cooking` raw food on the campfire, cooked by morning [2]
- `anvil_sound` the anvil, at night [2]
- `anvil_moved` the anvil is one block over, dented side toward the door [3]
- `grindstone_used` the grindstone sound, and a tool's enchantment is gone [4]
- `smithing_sound` the smithing table at night [2]
- `loom_pattern` a banner in a chest has one more pattern [3; dye]
- `stonecutter_cut` a stack of stone in a chest is stairs now [2]
- `brewing_done` a potion brewed from the target's ingredients [3]
- `brewing_wrong` the potion brewed is a different one than the ingredients made before [4]
- `cartography_copy` a map in a chest has a copy beside it [3; paper]
- `lectern_page` the book is open to another page [2]
- `lectern_book_swapped` the lectern's book is another book from the chest [3]
- `enchanting_glyphs` the enchanting table's book flips pages on its own while you are in the room [2; client]
- `composter_full` the composter is full in the morning [2; crops]
- `crafting_remains` a half-finished crafting layout on the table as dropped items in a 3x3 above it: pickaxe shape, one stick missing [3]
- `item_on_table` an item laid on the crafting table [2]
- `bell_at_table` ... (no bell: Set aside)
- `armor_stand_turn` a stand's head turns to the door [3]
- `armor_stand_dress` a stand wears a piece [3]
- `armor_stand_undress` the stand's armour is in the chest [3]
- `armor_stand_pose` a stand's arms raised, as if waving [3]
- `stand_watch` the stands follow you with their heads [3]
- `stand_in_doorway` the armour stand is in the doorway in the morning [3]
- `stand_at_bed` the stand is beside the bed, facing it [4]

## The house

- `open_door` a door is open [1]
- `close_door` a door is shut [1]
- `door_hinge` a door is hung on the other side [3]
- `door_type` a door is another wood's door, the old one in the chest [3; a door in a chest]
- `trapdoor_open` the trapdoor is open [1]
- `gate_open` the garden gate is open [1]
- `window_tap` taps on the glass [2]
- `window_fogged` the window pane is frosted glass for the target for a minute [3; glitch]
- `window_gone` a window pane is in the chest; the hole is open [3]
- `curtain` a banner hung over a window, from the chest [3]
- `carpet_shift` the carpets moved one block [2]
- `carpet_colour` one carpet is another colour [2; dye]
- `stairs_turned` a stair faces another way [2]
- `slab_flipped` a slab is on the top half [2]
- `fence_gap` a fence post is gone and is on the roof [3]
- `ladder_gap` a rung is missing, on top [3]
- `ladder_longer` the ladder goes one rung higher, from the chest [2]
- `roof_block` one roof block is a different block, the old one inside [3]
- `wall_block` one block in a wall is another kind, the old one in the chest [3]
- `wall_brick_count` one block in a wall is a block the target does not own: cobblestone, from its digging [3]
- `floor_hollow` one floor block is gone and the hole is a one-block alcove with a torch [4]
- `alcove` an alcove dug behind a wall, hidden [4]
- `secret_room` a 3x3 room dug into stone behind the house, reachable through a one-block gap behind a chest [4]
- `chimney_cold` the campfire chimney is out [2]
- `chimney_smoke` smoke from a chimney with no fire [2; client particles]
- `flowers_potted` the outdoor flowers are in pots inside [2]
- `pots_emptied` the flower pots are empty, the flowers outside in a ring [3]
- `path_worn` a dirt path to its place [3]
- `path_to_nowhere` a dirt path that stops at a wall of rock [3]
- `cairn` a stack of its stone at the route's end [3]
- `hole` a hole with a torch at the bottom [3]
- `backfill` the target's hole is filled in [3]
- `sapling` a sapling planted at the spot [2]
- `tree_grown` the sapling is a tree [3]
- `tree_cut` a tree by the house is cut, the logs in the chest, the sapling replanted [3]
- `leaves_trimmed` the leaves over the door are gone, as if for a better view [2]
- `pumpkin_face` a carved pumpkin faces the door [3]
- `pumpkin_lit` the carved pumpkin is a jack o'lantern [3; a torch]
- `scaffold` a scaffold tower beside the house, two blocks higher than the roof [3; bamboo in a chest]
- `stairs_outside` dirt steps up to the roof, from the back [3]
- `second_door` a door in the back wall where there was none, the wall block in the chest [4; a door in a chest]
- `boarded` the windows are wood planks from the inside, the glass in the chest [4]
- `rug_under_bed` a carpet under the bed [2]
- `bell_hung` ... (Set aside: needs a bell)
- `painting_change` a painting is another painting [2]
- `painting_flip` the painting is upside down for the target [2; glitch]
- `painting_moved` the painting is on another wall [2]
- `painting_added` a painting from the chest hangs by the bed [2]
- `frame_rotate` an item frame rotated one notch [1]
- `frames_turn` all frames the same way [3]
- `frame_swap` the frame's item swapped with the chest's [3]
- `frame_emptied` the frame is empty; the item is on the bed [3]
- `frame_favourite` the frame holds the favourite tool [3]
- `frame_glowing` the frame is a glow frame, the old one in the chest [3; glow ink in a chest]
- `banner_turned` the banner faces another way [2]
- `banner_word` the banner's pattern spells a letter of the want's word [4; dye]
- `bookshelf_books` a bookshelf is a chiseled one with the target's books in it [3]
- `house_mirrored` for one second, for the target, the house's interior renders mirrored [4; glitch]
- `house_lit_from_outside` every window glows at night from outside when the target comes home and no lights are on [3; client]
- `copy_build` a one-block outline of the house at its place [4]
- `copy_house` see [Building](#building)

## Lights and fire

- `move_light` a torch moved three blocks [1]
- `add_torch` a torch where there was none [2]
- `torch_gone_to_bed` the torch by the door is on the bed [2]
- `light_path` torches toward the spot [2]
- `torch_spiral` torches in a spiral around the house, one a night [3]
- `torch_every_other` every other torch in the house is a soul torch, the normal ones in the chest [3; soul torches in a chest]
- `candles_lit` every candle lit, or out [3]
- `candle_counts` the candles on the cake are a different number [2; a cake]
- `cake_eaten` one slice of the cake is gone [2]
- `put_out_campfire` the campfire is out [2]
- `light_campfire` the campfire is lit [2]
- `campfire_colour` the campfire is a soul campfire [3; soul soil in a chest]
- `lantern_moved` the lantern hangs from the other hook [1]
- `lantern_ahead` a lantern far down a cave [3]
- `cave_lit` a cave is lit at the target's spacing [3]
- `lamp_colour` a sea lantern where a glowstone was, the glowstone in the chest [3]
- `redstone_lamp_pulse` a redstone lamp pulses once a night, no power [2; client]
- `fire_in_hearth` a fire burns on the netherrack hearth that was cold [2; flint and steel]
- `light_level_low` for the target, every light in the house is one level dimmer for a day [3; glitch]
- `shadow_wrong` the target's own shadow points the wrong way for a second [4; glitch]
- `glow_in_dark` the target's own skin glows faintly in the dark for a second, like a glow squid [4; glitch]
- `cauldron` a cauldron filled or emptied [2]
- `cauldron_lava` the cauldron holds lava [3; a lava bucket in a chest]
- `cauldron_snow` the cauldron holds powder snow [2; snowing]

## Beds and sleep

- `shift_bed` the bed is one block over [3]
- `turn_bed` the bed faces another wall [3]
- `bed_colour` the bed is another colour [2; dye]
- `bed_unmade` ... (no state: Set aside)
- `refuse_rest` you may not rest now [3]
- `bed_occupied` this bed is occupied [3]
- `bed_too_far` you can only sleep at night, in the middle of the night [3; glitch message]
- `wake` woken, door open [3]
- `woke_elsewhere` woke one block over [4]
- `woke_outside` woke on the roof, your bed below you [4; a roof]
- `woke_in_other_bed` woke in a bed in the lodger's house [5; see Building]
- `second_bed` a second bed beside yours, from the chest [3; a bed in a chest]
- `bed_moved_to_spot` the bed is at its place, outside [4]
- `pillow_item` the held item on the pillow [2]
- `egg_on_bed` an egg on the bed [2]
- `sleep_sound` breathing from the other side of the bed while you lie in it [4; sound, Set aside? kept: it is soft]
- `dream_room` on waking, for one second the room is rendered as its place, then right [4; glitch]
- `sleep_skipped` the night passes but it is still night when you wake: your client shows night for a minute [3; glitch]
- `night_long` the night is twice as long for the target's client, then snaps [4; glitch]
- `phantoms_wrong` phantom sounds on a night you slept [3; sound]
- `spawn_point_message` "Respawn point set" with no bed touched [2; glitch message]

## Signs, words, maps

- `sign` a sign with one word [2]
- `sign_word` a line of your sign replaced [3]
- `sign_erased` your sign is blank; the words return tomorrow [3]
- `sign_turned` your sign faces the wall [2]
- `sign_dyed` your sign's text is another colour [2; dye]
- `sign_glow` your sign glows [3; glow ink]
- `sign_moved_to_spot` your sign is at its place [3]
- `hanging_sign` a hanging sign under the roof edge, one word [3; a hanging sign in a chest]
- `word_in_torches` torches laid out to spell a letter, seen from the roof [4]
- `word_in_crops` the field reads a word from above [4]
- `word_in_snow` the word walked into the snow [3; glitch decals]
- `word_in_path` dirt path blocks spelling a word, from above [4]
- `map_marked` a map marker at its place [3]
- `map_banner` a banner at its place, and your map shows its name as the word [4]
- `map_house` on the map, a second house drawn beside yours where there is none [4; map pixel edit]
- `book_word` a book has a word [3]
- `book_diary` a book and quill holds the memory's lines about you, as plain sentences, signed with no author [5; see Set aside for the text rule: these are the mod's own templates, so allowed]
- `book_routes` a book with your routes drawn as ascii art [5]
- `chat_none` never: see Set aside

## Farm, crops and food

- `harvest_one` one crop harvested, the yield in the chest [2]
- `harvest_row` one row harvested, replanted [3]
- `harvest_all` the whole field harvested and replanted overnight, the yield in the chest [4]
- `replant_other` one crop is another crop [3]
- `crop_grown` one crop is fully grown overnight [2; bone meal]
- `crop_stunted` one crop is a seedling again (replanted) [3]
- `trample` farmland trampled toward the door [3]
- `trample_circle` a circle trampled in the field [3]
- `field_word` the field spells a word [4]
- `field_watered` a water block in the field's center where there was none [3; bucket]
- `field_dry` the field's water is gone, in the bucket, in the chest [3]
- `scarecrow` an armour stand in the field [3]
- `scarecrow_moved` the scarecrow is a row closer to the house each morning [4]
- `melon_carved` ... (Set aside: can't)
- `pumpkin_pie` the pumpkins, sugar and eggs are a pie in the chest [3]
- `bread_baked` the wheat is bread [2]
- `cake_baked` the eggs, milk, sugar and wheat are a cake, on the table [3]
- `cookies` ... fold into bread
- `sweet_berries` the berry bush by the door is picked, the berries in the chest [2]
- `berries_planted` berry bushes along the path, from the chest [3]
- `bamboo_cut` the bamboo is cut to one height [2]
- `sugar_cane` the cane is harvested, replanted [2]
- `cactus_gone` the cactus by the door is in the chest [2]
- `cocoa` the cocoa is picked [2; jungle]
- `nether_wart` the wart is harvested, replanted [2; a wart farm]
- `mushroom_ring` mushrooms in a ring around the house, from the chest [3; mushrooms]
- `flowers_row` a row of flowers of one colour along the path [3]
- `hay_bale` the wheat is hay bales, stacked by the door [3]
- `honey_taken` the hive is emptied into bottles in the chest [3; bottles]
- `compost_bonemeal` the composter is empty, bone meal in the chest [2]
- `food_eaten` a stack of food in the chest is one short each morning [2]
- `food_rotten` ... (Set aside: a transform into rotten flesh is making an item)
- `fish_cooked` the raw fish is cooked [2]

## Animals and pets

- `open_gate` the pen is open [2]
- `animal_indoors` an animal in the house [3]
- `animal_gone` one animal grazing at the route's end [3]
- `animal_on_roof` a sheep on the roof [3]
- `animal_in_mine` a cow at the bottom of the mine [3]
- `animals_lined` the animals in a row facing the house at dawn [3]
- `animals_in_circle` the animals in a ring around the spot [4]
- `animals_silent` no animal sounds for a day, for the target [3; client]
- `sheep_dyed` a sheep is another colour [2]
- `sheep_sheared` a sheep is shorn, the wool in the chest [2; shears]
- `sheep_word` sheep dyed so that from the roof they spell a letter [4]
- `cow_milked` ... (nothing visible: Set aside)
- `horse_moved` the horse is on another post [2]
- `horse_saddled` the saddle from the chest is on the horse [2]
- `horse_unsaddled` the saddle is in the chest [2]
- `horse_at_door` the horse is at the door in the morning, facing in [3]
- `horse_far` the horse is at the route's far end [3]
- `donkey_chest` the donkey carries your chest's contents [3]
- `pig_saddle` the pig has the saddle [2]
- `chicken_in_bed` a chicken on the bed [2]
- `chickens_perched` the chickens are on the roof [3]
- `egg_on_bed` an egg on the bed [2]
- `eggs_in_a_row` eggs in a line toward the spot [2]
- `rabbit_in_house` a rabbit indoors [2]
- `named_animal` an animal named with a word [3]
- `all_named` every animal named with the same word [4]
- `pet_growl` the dog growls at nothing [2]
- `pet_moved` the sitting dog is elsewhere, facing a corner [2]
- `pet_outside` the pet is outside the closed door [3]
- `pet_on_bed` the cat is on the bed... (cats do that: fine, stage 1 as a false alarm) [1]
- `pet_watching_spot` the dog sits facing its place all day [3]
- `pet_follows_figure` the dog runs toward the far figure, stops at 32 blocks, sits [4]
- `cat_hiss_at_bed` the cat hisses at the bed [3]
- `parrot_mimic` the parrot makes the lodger's door sound [3; a parrot]
- `parrot_on_stand` the parrot sits on the armour stand [2]
- `wolf_collar` the dog's collar is another colour [2; dye]
- `fish_moved` the fish in the tank (water block) is in the pond [2; a bucketed fish placed]
- `axolotl_colour` ... (Set aside: can't recolour)
- `bees_out` the bees are out at night [3]
- `beehive_moved` the hive is on another tree [3; silk touch, Set aside unless the player has it]
- `frog_on_lilypad` the frogs sit in a row [2]
- `turtle_eggs` the turtle eggs are by the door [3]
- `villager_pet`... see Villages
- `golem_pet` the iron golem stands at your door all night [3]
- `allay_gift` the allay brings the held item to you... (an allay is tamed: its own behaviour; fine) [3]
- `fox_steals` a fox carries the favourite item past the window [3]
- `stare` animals and villagers stare [3]
- `stare_one` one animal stares, all day, wherever you go, turning its head [3]
- `herd_moves` the whole herd is at the spot [4]
- `wandering_trader` the trader's llamas face your house [2; a trader near]
- `llama_spit_wall` the llama spits at the wall behind you [2]
- `bat_indoors` a bat flies out of the chest room when you open the door [2; a cave near]
- `squid_ink` ink clouds in the pond with no squid [2; client particles]
- `dolphin_leads` a dolphin leads you to its place instead of a treasure [3; ocean]

## Villages and villagers

- `bell` the bell rings once at night [2]
- `bell_morning` the bell rings at the hour you usually log in [3]
- `villager_at_door` a villager at your door at night [3]
- `villager_in_house` a villager inside in the morning [4]
- `villager_at_window` a villager looks in the window all night [3]
- `villager_follows` a villager walks your route behind you [3]
- `villager_word` a trade of the held item for an emerald [4]
- `villager_free` a trade of one favourite for nothing [4]
- `villager_gossip` the villagers hum the note block's tune... (no: Set aside)
- `villagers_face_house` every villager faces your house at noon [3]
- `villagers_in_beds` every villager in bed at noon [3]
- `villagers_in_one_house` all the villagers are in one house [4]
- `golem_turned` the golem faces your house [3]
- `golem_at_spot` the golem stands at its place [4]
- `golem_flower` the golem offers a poppy to the air at your door [3]
- `village_doors` every door open [3]
- `village_doors_shut` every door shut at noon [3]
- `village_lantern` a lantern gone, on your path [2]
- `village_torches_out` the village's torches are in a chest at its place [4]
- `village_bell_moved` the bell is at your door [4]
- `village_beds_turned` the villagers' beds face your house [4]
- `village_crops` the village's crops are harvested, in your chest [3]
- `cat_from_village` the village's cat is in your house [2]
- `raid_horn_sound` the raid horn, for you alone, no raid [3; sound]
- `pillager_outpost_banner` the outpost's banner is at your door [4; an outpost near]

## Water, boats, fishing and ice

- `boat_moved` the boat is on the other bank [2]
- `boat_adrift` the boat is in the middle [3]
- `boat_waiting` the boat is at the shore you walk toward [3]
- `boat_sunk` the boat is on the lake bed, as an item frame-less wreck: the boat item in a chest at the bottom [4]
- `boat_in_house` the boat is in the house [3]
- `boat_on_roof` the boat on the roof [3]
- `boat_upstream` the boat is upriver, where it could not drift [3]
- `boat_doubled` ... (Set aside)
- `minecart_at_end` the minecart at the far end of the track [2]
- `bucket_floor` one water block in the floor [3]
- `moat` a one-block channel of water around the house, dug and filled from the lake [4]
- `well` a one-block well with water by the door, dug [3]
- `ice_path` a line of ice across the lake [3]
- `ice_circle` a ring of ice around the boat [3]
- `pond_frozen` the pond is ice in summer, for the target's client [3; glitch]
- `ripples` ripples in a line toward shore [3; glitch]
- `fishing_line` a bobber with no one at the rod [4]
- `fish_on_dock` fish laid on the dock in a row, from the chest [2]
- `fish_in_boat` the chest's fish are in the boat [2]
- `fishing_rod_gone` the rod is at the end of the dock, cast [3]
- `drip` dripping indoors [2]
- `drip_from_ceiling` dripping water particles from the ceiling over the bed [2; client]
- `water_sound_dry` splashing from the dry well [2; sound]
- `waterfall_quiet` the waterfall by the house is silent for an evening [3; client]
- `underwater_breath` the bubble bar appears for a second on land [3; glitch]
- `lily_pads` lily pads in a line to the far shore, from the chest [3]
- `kelp_cut` the kelp is cut to one height [2]
- `coral_moved` ... (Set aside: coral dies)
- `conduit_hum` the conduit's sound in a house with none [2; sound]
- `shipwreck_chest` the shipwreck's chest is empty; its loot in your chest [3; a wreck near]
- `drowned_trident_sound` a trident thrown, hitting your wall [3; sound]
- `reflection` your reflection faces the wrong way [4; glitch]
- `reflection_absent` no reflection for a second [4; glitch]
- `across_water` the figure on the far shore [3]
- `figure_in_boat` the figure in a boat in the middle [3]
- `figure_under_water` the figure stands on the lake bed, looking up, seen from the boat [4]

## Weather, sky and time (the target's client alone)

- `sun_blink` the sun gone for seconds [3]
- `sun_wrong_place` the sun sets in the east for a minute [4]
- `two_moons` two moons for a second [4]
- `moon_wrong` the moon full, or huge [3]
- `moon_red` a red moon [3]
- `stars_out` no stars for a minute [3]
- `stars_moving` the stars move for a few seconds [4]
- `night_early` the sky two hours ahead [4]
- `day_frozen` the sun stands still for ten minutes [4]
- `red_rain` red rain [4]
- `black_rain` black rain [4]
- `rain_one_block` rain on one block, indoors, over the bed [3]
- `snow_indoors` snow indoors [3]
- `rain_indoors` rain indoors [3]
- `lightning_silent` lightning, no thunder [3]
- `thunder_clear` thunder, clear sky [2]
- `lightning_on_spot` lightning strikes its place, for the target only, no fire [4]
- `fog_close` fog closes in [3]
- `fog_colour` the fog is the wrong colour for a minute [3]
- `clouds_low` the clouds are at roof height for a minute [3]
- `clouds_gone` no clouds for a day [2]
- `wind_sound` wind, which the game has no sound for [2; sound]
- `aurora` an aurora, in no biome that has one [4]
- `eclipse` the sun dark in the middle, a ring, for a minute [4]
- `sky_void` the sky is the void's colour for a second [4]
- `rain_sound_dry` rain sounds on a clear night [2; sound]
- `weather_lag` the rain keeps falling for a minute after it stopped for everyone else [3]

## Biomes and places

- Snow: `trail` footprints you did not make [3]; `trail_in` prints to the door, none away [4]; `snow_cleared` the path cleared [2]; `snow_piled` snow piled against the door, two layers [2]; `snowman_parts` a pumpkin and snow blocks stacked by the door, not a golem [3]; `igloo` a snow block hut at its place [4]; `frost_window` frost on the window glass for the target [3; glitch]; `snow_angel` a shape in the snow decals [3; glitch]
- Desert: `sand_fallen` sand fallen over the door at night, one block [2]; `sand_cleared` the sand cleared from the road [2]; `cactus_ring` cacti planted in a ring [3]; `dead_bush_line` dead bushes in a line [2]; `sandstone_pillar` a pillar of sandstone it cut, at the spot [3]; `oasis` a one-block pool dug and filled at the spot [4]; `mirage_house` your house, far off across the sand, for a second [4; glitch]; `husk_stare` husks stand still facing the house, no attack [3]
- Jungle: `vines_cut` the vines off the house [2]; `vines_grown` vines over the door [2; vines in a chest]; `cocoa_picked` [2]; `melon_split` melons harvested [2]; `parrot_on_roof` parrots line the roof ridge [3]; `ocelot_stare` [3]; `temple_lever` the temple's levers are flipped [3]; `bamboo_wall` a bamboo fence at the spot [3]
- Swamp: `lilypad_path` lily pads to the shore [3]; `frog_choir` frogs croak in a rhythm, for the target [2; sound]; `slime_sound_dry` slime sounds, no slimes [2]; `mangrove_prop` a propagule planted at the spot [2]; `witch_hut_cauldron` the witch's cauldron is empty [3]; `fog_swamp` the swamp's fog thicker for a minute [3; glitch]
- Mushroom: `mushroom_ring` [3]; `mooshroom_sheared` the mooshroom is a cow [3; shears]; `mycelium_path` a path of grass across the mycelium [3; dirt]
- Mountains: `goat_stare` the goats stare [3]; `goat_horn_sound` a horn from the peak at dawn [2]; `snow_slide` a layer of snow gone from the roof in the night [2]; `cairn_peak` a cairn on the nearest peak [3]; `echo_louder` the mountain echo answers twice [3; sound]; `powder_snow_warning` the freezing overlay for a second indoors [3; glitch]
- Ocean: `dolphin_leads` [3]; `sea_lantern_gift` a sea lantern on the dock, from the monument... (from nothing unless it takes: it takes one from the monument) [4]; `conduit_hum` [2]; `drowned_stare` drowned stand in the shallows facing the house, no attack [3]; `turtle_eggs_at_door` [3]; `message_in_bottle`... (Set aside: no such item)
- Caves and deep dark: `sculk_sound` sculk sensor clicks, none near [3; sound]; `warden_heartbeat` the deep dark's heartbeat under the house, once [4; sound]; `glow_lichen_arrow` glow lichen placed to point down a tunnel [3]; `amethyst_chime` an amethyst chime from the wall [2; sound]; `dripstone_drip` dripstone dripping over the bed [2; client]; `spore_blossom` spores falling indoors [2; client]
- Nether-adjacent (never through): `portal_sound` [3]; `portal_lit` [3]; `portal_unlit` your lit portal is unlit at night [3]; `portal_frame_moved` the portal frame is one block over [4]; `netherrack_hearth` [2]; `ghast_cry_far` a ghast, far, overworld [3; sound]; `piglin_at_portal` a piglin stands at your portal, overworld side, not zombifying, for a minute, for the target's client [4; glitch entity]
- End-adjacent: `endermen_face` the endermen all face your house at night [3]; `enderman_holds_your_block` an enderman carries a block of your house's kind past the window [3]; `end_portal_sound` the end portal's hum under the house [4; sound]; `dragon_breath_bottle` ... (Set aside)

## Mines and caves

- `mining_sounds` pickaxe in the rock keeping pace with yours [2]
- `mining_sounds_stop` when you stop, it stops; when you start, it starts [2]
- `mining_sounds_ahead` the sounds are ahead of you, where you will dig next [3]
- `echo_steps` footsteps answering from a side tunnel [2]
- `ore_marked` a torch on an unfound vein [2]
- `ore_gone` the vein you saw yesterday is gone, mined clean [3]
- `mined_out` the vein ahead is a torchlit hollow [3]
- `ore_moved` ... (Set aside)
- `ore_in_chest` ore you never mined in your chest; a hollow in the rock somewhere [3]
- `mine_beside` a parallel tunnel, lit your way [3]
- `mine_above` a tunnel one block above yours, your footsteps overhead... [3]
- `mine_below` a tunnel one block below, you fall through a thin floor into it (no fall damage: one block) [4]
- `tunnel_toward` a tunnel dug toward you over minutes [4]
- `tunnel_breaks_through` ... part of above
- `tunnel_stops_short` ... part of above
- `tunnel_sealed` your old tunnel walled with cobble [3]
- `tunnel_lit` your unlit tunnel is lit at your spacing [2]
- `tunnel_unlit` your tunnel's torches are in a chest at its mouth [3]
- `tunnel_extended` your tunnel is ten blocks longer in the morning [3]
- `tunnel_branch` a branch off your tunnel you did not dig, lit, ending in a dead end with a torch [3]
- `tunnel_loop` your tunnel loops back to its own start [4]
- `stairs_down` a staircase down from your tunnel to its place [4]
- `ladder_down` a ladder down a shaft [3]
- `shaft_to_bedrock` a 1x1 shaft to bedrock with torches every 8, by the house [4]
- `minecart_rolls` your minecart rolls out of the mine [3]
- `rails_extended` your rails go ten blocks further [3; rails in a chest]
- `rails_curve` your straight rail curves toward the spot [4]
- `lantern_ahead` a lantern far down an unentered cave [3]
- `cave_lit` an unentered cave lit your way [3]
- `cave_sealed` a cave mouth near the house walled with cobble [3]
- `cave_room` a room in the cave: see [Building](#building)
- `cave_sound_near` a cave sound from inside the wall of the house [2; sound]
- `cave_sound_direction` cave sounds always from the direction of its place [3]
- `gravel_falls` gravel falls behind you in the tunnel, blocking it (natural gravel from above) [3]
- `water_in_tunnel` a water source at the end of your tunnel, from the bucket [3]
- `lava_light` lava light flickering around a corner with no lava [3; glitch]
- `torch_pulled` the last torch in your tunnel is gone when you turn: it is in your chest [3]
- `pickaxe_left` your spare pickaxe is leaning (as an item frame on the wall) at the tunnel's end [3; a frame]
- `cobble_stack` its cobblestone stacked at your tunnel's mouth, a full stack as an item [2]
- `diamonds_shown` a torch on a diamond vein, far down [4]
- `bedrock_message` nothing: Set aside

## Building

What it builds, it builds from what it dug, cut and picked: cobblestone, dirt, logs from trees it
felled (replanted), stone it mined. At a player's pace, out of sight, over nights.

- `copy_build` a one-block outline of your house at its place [4]
- `copy_house_rough` a cobblestone copy of your house at its place, your size, no roof, over three nights [4]
- `copy_house_exact` a copy of your house in cobblestone and logs, same doors and windows, a cobble bed-shaped block where your bed is [5]
- `house_near` a small house of its own 40 to 80 blocks from yours: 5x5, a door facing your door, one window, a torch, nothing inside but a chest holding what it took [4]
- `house_grows` its house gets one room bigger each stage [4, 5]
- `house_furnished` its house gets a copy of your furniture, one piece a night: a table, a bed (your spare), a frame with your tool [5]
- `cave_room` a 3x3x3 room in the nearest cave, lit, a chest, a door of logs it cut [3]
- `cave_room_near_mine` the room is off your mine's tunnel [4]
- `cave_room_with_bed` the room has a bed... (a bed is crafted from wool and planks: it has sheep and trees... allowed if it shears and cuts) [4]
- `pillar` a cobble pillar at its place, up to the height of your roof [3]
- `pillar_taller` the pillar grows one block a night [3]
- `tower` a 3x3 cobble tower at the spot with a ladder, to twice your roof's height [4]
- `bridge` a one-block bridge of logs across the water from its place toward your shore, not finished [4]
- `bridge_finished` the bridge reaches your shore [5]
- `wall_segment` a two-high cobble wall, ten long, between your house and the village [4]
- `fence_line` a fence of logs it cut from your door to its place [3]
- `gate_in_fence` the fence has a gate, facing your door [3]
- `well` a well by your door [3]
- `moat` a channel around the house [4]
- `path_cobbled` your dirt path is cobblestone now [3]
- `steps_to_water` cobble steps down to the lake [3]
- `dock` a log dock at the lake, where you fish [3]
- `dock_longer` the dock reaches further each night [3]
- `hut_on_route` a 3x3 shelter of logs halfway along your route, a torch inside [3]
- `graveyard` ... (Set aside: too far)
- `mirror_room` a room dug into the hill facing your bedroom window, the same size, a torch where your torch is [4]
- `platform_in_tree` a log platform in the tree outside your window [3]
- `ladder_to_roof` a ladder up your house's outer wall [3]
- `roof_garden` dirt and flowers on your roof [3]
- `greenhouse` glass... (glass needs a furnace and sand: it cooks: allowed) a 3x3 glass hut over the field's corner [4]
- `pen_for_animals` a fence pen at its place with one of your animals in it [4]
- `second_field` a field at its place with your crops, watered [4]
- `cairn_row` cairns every 20 blocks along your route [3]
- `arch` a cobble arch over your route [3]
- `doorway_in_hill` a 2-high doorway dug into the hill with a log door, leading nowhere [3]
- `stairs_in_hill` stairs dug into the hill to the top, lit [3]
- `your_initial` ... (text rule: Set aside)
- `nest` a circle of hay bales with the held item in the middle [4]
- `altar` a cobble block with the held item in a frame on it at the spot, torches at the corners [4]
- `house_burned` ... (Set aside: fire)
- `house_lit_at_night` its house's torch is lit only when yours are [4]
- `smoke_from_its_house` its campfire smokes at the hour you cook [4]
- `its_door_open` its door is open every morning you look [4]
- `its_bed_slept` ... (no state)
- `its_chest_grows` its chest holds one more of your things each stage [4]

## Redstone and mechanisms

- `lever` a lever flipped [2]
- `lever_flipped_back` flipped back an hour later [2]
- `note` one note at night [2]
- `note_tune` three notes of your disc [3]
- `note_your_steps` note block plays a note on each of your steps for a minute [3; sound]
- `jukebox` your disc plays, or stops [3]
- `disc_phantom` a disc you do not own plays from an empty jukebox [3; sound]
- `disc_swapped` the disc in the jukebox is another of yours [2]
- `door_opens_itself` a door opens as you approach [3]
- `door_closes_behind` the door closes behind you every time for a day [3]
- `plate_laid` a pressure plate before the chest [3]
- `plate_gone` your plate is in the chest [2]
- `button_pressed` a button clicks with no one there, and the door opens [2]
- `piston_once` a piston cycles once at night [3]
- `piston_blocks_door` a piston pushes a block into your doorway (your own piston, your block, pulled back by morning) [4]
- `lamps_noon` lamps on at noon [3]
- `lamp_morse` a lamp blinks a pattern (the want's word in Morse) [4]
- `daylight_sensor_inverted` the sensor is inverted [2]
- `hopper_clock` a hopper clock ticks faster for a day [3]
- `dispenser_fires` your dispenser fires one arrow into the wall at night [3; arrows in it]
- `dropper_drops` your dropper drops the held item at your feet as you pass [3]
- `observer_watches` the observer faces you wherever you stand in the room [3]
- `comparator_sound` the comparator's click, no comparator [2; sound]
- `tripwire_snapped` the tripwire string is gone, in the chest [2]
- `target_block_hit` the target block rings as you enter [3; sound]
- `bell_rings_itself` your bell rings when you enter [3]
- `iron_door_open` your iron door is open with no power [3]
- `rail_powered` the powered rail is on, the cart moving [3]
- `redstone_dust_line` a line of dust from your chest to the door, from your chest's dust [3]
- `torch_redstone_off` your redstone torch is off for a day, on the ground [2]

## Sounds (the target alone)

- `door_sound` a door in another room [1]
- `chest_sound` a chest in another room [1]
- `cut_music` no music tonight [1]
- `music_wrong` the Nether's music, overworld [3]
- `music_loud` the music is suddenly loud, then normal [3]
- `music_slow` the music plays at half speed for a bar [4]
- `late_steps` your steps, late [2]
- `late_eating` your bite, late [2]
- `late_jump` your landing, again [2]
- `late_swing` your tool swing, again, behind you [2]
- `late_door` the door you closed closes again, in sound [2]
- `place_sound` a block placed behind you [2]
- `break_sound` a block broken behind you, nothing broken [2]
- `knock` three knocks on the door [2]
- `knock_window` knocks on the window [2]
- `knock_floor` knocks from under the floor [3]
- `knock_bed` knocks from under the bed [4]
- `lever_click` a lever, no lever [2]
- `page_turn` pages turning [2]
- `arrow_hit` an arrow in the wall behind you [3]
- `arrow_fly` an arrow passing your ear [3]
- `wolf_far` a wolf far off [2]
- `cat_purr_empty` a cat purring under the bed, no cat [2]
- `chicken_in_wall` a chicken clucking from inside the wall [2]
- `villager_hm` a villager's hm from the next room [2]
- `anvil_sound` the anvil at night [2]
- `furnace_crackle_cold` the cold furnace crackles [2]
- `water_sound_dry` splashing from dry ground [2]
- `drip` dripping [2]
- `drip_faster` the dripping speeds up as you approach the wall [3]
- `wind_sound` wind [2]
- `silence` ambient silenced [3]
- `silence_sudden` every sound stops for two seconds, then returns [3]
- `your_voice` your own hurt sound from the next room [4]
- `your_death_sound` your own death sound, far off [5]
- `breathing_bed` breathing beside the bed [4]
- `whistle` a single note whistled, the first of your disc [3]
- `sculk_sound` sculk clicks [3]
- `warden_heartbeat` the heartbeat, once [4]
- `portal_sound` a portal hum [3]
- `end_portal_sound` the end portal's hum [4]
- `amethyst_chime` a chime from the wall [2]
- `bell_sound_far` a bell with no village [2]
- `goat_horn_sound` a horn at dawn [2]
- `raid_horn_sound` the raid horn [3]
- `elytra_pass` elytra wings passing over the roof [3]
- `trident_hit` a trident in the wall [3]
- `fishing_splash` a bobber splash from the dry well [2]
- `totem_pop` the totem's sound, no totem [4]
- `xp_pickup` experience pickup sounds while standing still [2]
- `level_up` the level-up chime, no level [3]
- `achievement_sound` the advancement chime, no toast [2]
- `tnt_fuse` a fuse hiss behind the wall, nothing [4; borderline: see Set aside, kept as sound only]
- `creeper_hiss_short` half a hiss, cut off [3]
- `enderman_stare_sound` the enderman's scream, no enderman [3]
- `ghast_cry_far` a ghast, far [3]
- `your_name_said` nothing: see Set aside

## Visual glitches (the target's client alone)

- `flicker` a light flickers [2]
- `lights_fail` the room's lights fail [3]
- `light_level_low` dimmer for a day [3]
- `shadow_wrong` your shadow wrong [4]
- `painting_flip` a painting upside down [2]
- `painting_face` the painting's image is replaced by a plain dark rectangle for a second [4]
- `frame_item_spins` the item in a frame spins slowly [3]
- `block_texture_shift` one block of the wall has the wrong texture until looked at [3]
- `block_missing` one block renders missing (see-through) until looked at [3]
- `door_render_open` a closed door renders open [2]
- `chest_lid_open` the chest lid renders open [2]
- `torch_unlit` a torch renders unlit [2]
- `water_still` the river's water stops moving for a minute [3]
- `grass_colour` the grass is the swamp's colour for a minute [3]
- `leaves_colour` the leaves are autumn-coloured for a minute [3]
- `sky_colour` the sky green for a second [3]
- `fog_colour` the fog red at the edges [3]
- `particles_reverse` smoke falls instead of rising [3]
- `particles_gather` dust particles gather at its place [3]
- `footprint_decals` prints on snow and sand [3]
- `handprints` handprints on the window decals [4]
- `reflection` your reflection wrong [4]
- `reflection_absent` no reflection [4]
- `your_skin_wrong` your own arm in first person is the figure's... it is your skin: your sleeve is dirty for a minute [4]
- `house_mirrored` the interior mirrored for a second [4]
- `mirage_house` your house across the sand [4]
- `second_house_far` a second copy of your house on the horizon for a second [4]
- `nameplate_far` your own name over nothing at 60 blocks [3]
- `item_frame_your_face` the frame shows your head for a second [4]
- `bed_render_occupied` the bed renders with someone's legs under the blanket until looked at [4]
- `window_face` a face at the window for one frame (the figure's: yours) [4]
- `fov_pull` the field of view narrows [4]
- `camera_nudge` the camera nudges toward the place [3]
- `camera_behind` the camera is in third person behind you for one frame [4]
- `camera_front` the camera is in front, looking at your face, for one frame [5]
- `screen_shake` a tremor, once [3]
- `vignette_dark` the darkness vignette for a second in full light [3]
- `portal_overlay` the Nether portal overlay for a frame, no portal [4]
- `pumpkin_overlay` the pumpkin blur for a frame [3]
- `freeze_overlay` the freezing frost for a second [3]
- `underwater_overlay` the water overlay on land for a frame [3]
- `blindness_fade` blindness for one second [4]
- `darkness_pulse` the darkness effect's pulse once [4]
- `night_vision_flash` night vision for a frame [3]

## HUD and controls (the target's client alone)

- `hotbar_slip` the selected slot moves to the favourite tool for a tick [2]
- `hotbar_order` the hotbar renders in another order for a second, items unchanged [3]
- `hunger_shake` the hunger bar shakes [2]
- `heart_skip` the hearts flicker [3]
- `hearts_one_more` eleven hearts for a second [3]
- `armor_bar_gone` the armour bar is empty for a second [3]
- `xp_bar_drain` the experience bar drains and refills [3]
- `subtitle` `Footsteps` with no sound [2]
- `subtitle_name` `Door opens` repeated three times [2]
- `crosshair_gone` no crosshair [3]
- `crosshair_tilted` the crosshair rotated 45 degrees [3]
- `tab_twice` your name twice in the list [3]
- `tab_ping` your ping bar is empty for a second [2]
- `sleepers_wrong` the sleep count is off by one [3]
- `actionbar_word` the action bar shows one word of the want's list for a second [4]
- `title_blank` a title fades in and out, blank [4]
- `scoreboard_flicker` a sidebar appears for a frame with your name and a number: the days of the stay [4]
- `inventory_cursor_moves` in the inventory screen, the cursor's held item is your favourite for a frame [3]
- `tooltip_word` the favourite tool's tooltip has one extra line: the word [4]
- `coordinates_wrong` the F3 coordinates are its place's for a second [3]
- `fps_zero` the F3 fps reads 0 for a second [2]
- `chat_scroll` the chat scrolls up one line and back [2]
- `mouse_drift` the view drifts one degree a second toward the place for five seconds [4]
- `sprint_cancel` sprint stops once, as if you had let go [3]
- `sneak_held` you stay sneaking one second after letting go [3]
- `jump_delayed` ... (input: Set aside beyond the three above)

## The figures (yours, in your skin)

- `far_figure` far off, facing you, gone when approached [3]
- `behind_you` behind on your route, nearer each time, never under 32 [3]
- `on_the_roof` on your roof at dawn [3]
- `across_water` on the far shore [3]
- `figure_in_boat` in a boat [3]
- `figure_under_water` on the lake bed looking up [4]
- `in_doorway` in the doorway for half a second [4]
- `at_the_chest` at your chest, back to you [4]
- `in_your_bed` in your bed, gone when the door opens [4]
- `at_the_window` at the window from outside while you are in bed [4]
- `on_the_route_ahead` ahead of you on your route, walking away at your pace [3]
- `replay` walks your recorded route [4]
- `replay_yesterday` walks exactly what you did yesterday at this hour, in your house, through walls if you moved them [5]
- `copy_crouch` crouches when you crouch [4]
- `wave` waves once [4]
- `points` points toward its place (arm raised) [4]
- `tools_in_hand` holds your tool [3]
- `mines_far` mines at the rock at 60 blocks, with sound [3]
- `fishes_far` fishes from the far shore [3]
- `plants_far` plants at its field [4]
- `stands_in_field` stands in your field at night, among the crops [3]
- `stands_in_pen` stands among your animals, they are calm [3]
- `sits_at_campfire` sits (crouches) at your campfire when you come home, gone at 32 [4]
- `in_village` stands among the villagers at noon, seen from your route [3]
- `in_your_mine` at the end of your tunnel, facing the wall, when you arrive; gone around the branch [4]
- `two_figures` never: Set aside
- `figure_sleeps_in_its_house` seen through its house's window, lying in its bed [5]
- `figure_at_its_door` stands at its door when you approach its house, goes in, the door shuts [5]
- `figure_follows_dog` your dog leads you to it, it walks away [4]
- `figure_leaves_item` it stood here; the item is here [3]
- `figure_leaves_torch` a lit torch where it stood [3]
- `figure_in_rain` only visible in the rain, at 60 blocks [3]
- `figure_in_lightning` only visible in a lightning flash [4]
- `figure_in_reflection` only visible in the water's reflection [4]
- `figure_in_map` a player marker on your map 50 blocks off, with your name, for a minute [3; glitch]
- `figure_on_scaffold` on its scaffold tower at dawn [4]
- `figure_mirrors_you` at 60 blocks, moves as you move, mirrored, for ten seconds [4]
- `figure_runs` runs away when seen, the only time it is fast [4]
- `figure_sits_in_your_boat` in your boat at the dock [3]
- `figure_rides_your_horse` on your horse at the far post, dismounts, gone [4]
- `figure_with_your_pet` your dog sits beside it, far off, then runs back to you [4]
- `nameplate_only` your name over empty air [3]
- `into_the_mountain` see [Where it goes](#where-it-goes-and-comes-from)
- `figure_lit_by_its_torch` at night, holding a torch, far off, the torch the only light [3]

## Where it goes and comes from

The owner's mountain (2026-11-09) made a family: the world opens for it and closes behind it,
always seen from a distance, always leaving a natural face or a thing of yours back in place,
so walking up finds nothing to prove. The host does these as the figure plus a block script:
each block removed is put back a second later, so nothing is ever destroyed, and a block of
yours it passes through is yours again before you arrive.

- `into_the_mountain` it digs into the mountainside at a player's pace; the stone closes behind it block by block until the face is smooth; one block in, a torch, then rock [4]
- `out_of_the_mountain` the mountainside opens one block at a time, it steps out, looks at you, and the stone closes [4]
- `through_the_hill` it walks into one side of a hill and, a minute later, out of the other [4]
- `into_the_ground` it stands at its place and sinks one block at a time, the dirt closing over; a dirt path block marks the spot [4]
- `out_of_the_ground` the ground at its place rises as a column of dirt, it is standing on it, the column sinks back [4]
- `into_the_lake` it walks into the lake and does not come up; the water is still; the next day a boat is where it went in [4]
- `out_of_the_lake` it walks out of the lake at dawn, dry, up the bank, toward your door, gone at 32 [4]
- `under_the_ice` seen through the ice from the surface, walking below, then not [4; glitch]
- `into_the_tree` it steps into the trunk of the big oak and the leaves shake once [4]
- `into_the_leaves` it climbs into the canopy and the leaves close; the tree drops one sapling [3]
- `into_the_snow` it lies down in the snow and sinks; the snow is smooth; footprints lead there and stop [4; glitch decals]
- `into_the_sand` the desert sand takes it like water [4]
- `into_your_wall` it walks into the wall of your house from outside; the wall block is gone a second and back; nothing inside [4; a block of yours moved and restored]
- `out_of_your_wall` from inside, a block of your wall is missing for a second and it is in the gap, then the block is back and it is not [5]
- `into_the_chest` seen through the window, it climbs into your double chest and the lid shuts; the chest is as it was, one slot shuffled [4]
- `into_the_bed` it lies down in your bed and sinks through it; the bed is one block over [4]
- `into_the_painting` it steps into the big painting; the painting is another painting [4; a figure walk plus `painting_change`]
- `into_the_well` it climbs down the well it dug; the well is dry; at the bottom, a torch [4]
- `into_the_portal_frame` it walks through your unlit portal frame as if lit, and is not on the other side [4]
- `up_the_ladder` it climbs your ladder to the roof and is not on the roof [3]
- `down_the_shaft` it drops down its own one-block shaft; the shaft fills in from the bottom up [4]
- `into_the_cave` it walks into the cave mouth; the cave is lit behind it one torch at a time as it goes deeper, then the torches go out from the far end [4]
- `into_the_mine` it walks into your mine ahead of you; the tunnel's torches go out one by one ahead of you; at the end, nothing, and they are lit again behind you [4]
- `into_the_village` it walks into a village house at dusk and the door shuts; the house has one more bed in the morning [4]
- `into_the_fog` it walks into fog that closes around it, on a clear day, for the target alone [4; glitch]
- `into_the_rain` it is there in the rain and not when it stops [3]
- `into_the_dark` it stands at the edge of your torchlight and steps back one block out of it; the torch flickers; it is not there [3]
- `out_of_the_dark` the reverse: it steps into the light from the dark, one step, and back [3]
- `into_its_house` it goes into its house and the door shuts; its window is dark; the next night it is lit [4]
- `into_your_house` the last night: it goes into your house while you are outside; every window lights; when you open the door, nothing is different but one thing [5]
- `into_the_sky` ... no: see Set aside, it never flies
- `through_the_floor` from the roof you see it walk into your house's floor as into water [5]
- `into_the_mirror_room` it walks into the room dug facing your bedroom and the room's torch goes out [4]
- `into_the_waterfall` it steps through the waterfall; behind it, a one-block alcove with a torch [4]
- `closes_the_door_from_inside` the hill's doorway it dug: it goes in, and the doorway is stone from the inside out, the log door left standing in rock [4]

- `stare` animals and villagers stare [3]
- `zombies_face_house` every zombie within 64 faces your house, standing still, until dawn [3]
- `skeleton_no_shot` a skeleton stands at 30 blocks and does not shoot [3]
- `creeper_still` a creeper stands still at 20 blocks, never approaching, until you look away [3]
- `spider_on_roof` a spider sits on the roof all night [2]
- `endermen_face` the endermen face the house [3]
- `enderman_holds_your_block` an enderman carries your block's kind past [3]
- `enderman_at_spot` an enderman stands at its place, teleports when approached [3]
- `phantoms_circle_spot` phantoms circle its place, not you [3]
- `slimes_in_line` slimes hop in a line toward the house [2]
- `witch_watches` a witch stands at the swamp's edge watching, no potions [3]
- `pillagers_kneel` ... (no pose: Set aside)
- `husk_stare` husks face the house [3]
- `drowned_stare` drowned in the shallows face the house [3]
- `zombie_at_door_knocks` a zombie at the door that does not break it (hard mode off for it) [3]
- `mobs_avoid_spot` no mob ever walks on its place [3]
- `mobs_avoid_you` mobs give you a 16-block berth for a night [3]
- `bats_from_chimney` bats fly from the chimney at dusk [2]
- `silverfish_sound` silverfish in the wall, sound [2]
- `wandering_trader_stares` the trader faces you from the road [2]
- `iron_golem_follows` the golem follows you for a day at 10 blocks [3]
- `cat_sits_on_chest` a village cat sits on your chest and will not move [2]
- `chicken_jockey_far` ... (spawning: Set aside)
- `mob_with_your_name` a zombie with your name over it, far off, for a second [4; glitch, not a tag]
- `mob_heads_turn` every mob in view turns its head to you at once [3]
- `mobs_quiet` no mob sounds for a night [3; client]

## Multiplayer and witnesses

- `tab_twice` [3]
- `sleepers_wrong` [3]
- `witness_sees_door` a witness sees the door open; the target did not [2; the normal trace]
- `witness_asked` the model asks: should a witness see this one? [any]
- `witness_blind_spot` an event in a witness's room, which the target gets blamed for [3]
- `target_doubt` the target's own events happen only when no witness is within 32 blocks [all; a rule, not an event]
- `witness_message_none` never a message to a witness [rule]
- `retarget` a new target once [stage boundary]
- `two_homes` on a shared base, only the target's rooms [rule]
- `witness_trace_delayed` a witness finds a trace a day after the target did [3]
- `sign_for_witness` a sign only a witness reads (placed in their room: no, their room is off limits) [Set aside]
- `shared_chest_untouched` the shared chest is never touched; the target's own always [rule]
- `figure_seen_by_witness_once` at the very end, a witness sees the figure once [5]
- `witness_hears_bell` the bell is for everyone [2]
- `op_status_hidden` `/lodger status` says nothing to a non-op [rule]

## Travel and routes

- `behind_you` [3]
- `on_the_route_ahead` [3]
- `route_lit` your route has torches at your spacing [3]
- `route_marked` a cobble marker every 20 blocks [3]
- `route_shortened` a tunnel through the hill your route goes around, lit [4]
- `route_blocked` a cobble wall across your route, one high, step over it [3]
- `route_signposted` a sign at the fork with the word [3]
- `route_flowers` flowers along the route [3]
- `route_path` dirt path along the route [3]
- `route_trees_cut` the trees along your route are cut, replanted [3]
- `route_bridge` a log bridge where you jump the stream [3]
- `route_stairs` stairs where you climb the slope [3]
- `route_gate` a fence gate where you enter the field [2]
- `route_echo` your footsteps echo differently on one stretch [3; sound]
- `route_away` a second path leads from your route to its place [3]
- `horse_waiting` your horse at the route's start [3]
- `boat_waiting` [3]
- `minecart_waiting` your cart at the station [2]
- `compass_points` [3]
- `lodestone` a lodestone at its place, your compass bound... (a lodestone needs netherite: Set aside)
- `far_lands_house` ... no

## Reading how you move

Events keyed to the movement reading, which is the lodger noticing fear or its absence.

- `when_you_look_back` the figure is there only on the third look back, never the first [3]
- `when_you_stop` every time you stand still more than ten seconds, a door sound somewhere [2]
- `when_you_run` when you sprint from a room, the door shuts behind you [3]
- `when_you_sneak` when you sneak indoors, it is quieter than you: no sounds that night [2]
- `when_you_back_away` the thing you backed away from is in your bed [3]
- `when_you_hug_walls` the torches on the walls you walk along go out one by one ahead of you, client-side [3]
- `when_you_face_it` the longer you face the spot, the longer the far figure stays [3]
- `when_you_turn_fast` a fast turn shows the figure in the doorway for one frame [4]
- `when_you_are_calm` a player who never looks back gets the bold events sooner; one who always does gets the slow ones [all; a weight]
- `when_you_leave_fast` leaving a room at a run marks it: the next event is in it [3]
- `when_you_linger` standing at the chest a long time opens the chest one more time [2]
- `when_you_jump` at a jump indoors, late landing sound from the room above [2]
- `when_you_crouch_at_window` crouching at a window shows the figure far off, crouching too [4]
- `pace_match` the far figure's pace is yours, from the memory [3]
- `route_speed` it walks your route at your average pace, not a player's default [4]

## Provocations and offerings

What moves the temper, beyond the table in the catalogue, and what the lodger does about it.

- `torch_broken` you break its torch: two more appear by morning [2]
- `sign_broken` you break its sign: the word is on three signs [3]
- `pillar_broken` you break its pillar: the pillar is back, one taller, overnight [3]
- `house_broken` you break into its house: its door is gone and the doorway walled; a new door on the side facing away from you [4]
- `place_walled` you wall its place: the wall has a doorway in the morning; your wall blocks are stacked beside it [3]
- `place_lit` you light its place with torches: they are moved to your bed [3]
- `place_flooded` you flood its place: the water is gone and the place is one block higher [4]
- `place_tnt` you put TNT near it: the TNT is in your chest, the fuse sound every night for a week [4]
- `figure_hit` you hit the figure: it does not react; that night, every door opens at once [4]
- `figure_chased` you chase the figure: it runs, the only time it is fast; a week of nothing [4]
- `figure_shot` you shoot it: the arrow is on your bed in the morning [4]
- `animal_killed` you kill an animal it moved: the pen is empty the next morning, the animals at its place [4]
- `thing_taken_back` you take the thing back: every frame emptied [3]
- `chest_emptied` you empty its chest: your chest is empty, the contents in its [4]
- `insult` you curse at it in chat: your name in the player list goes grey for an hour [3]
- `ignore` you sleep every night: it starts waking you [3]
- `offering_chest` you leave a chest for it: it fills it with one of everything it took [3]
- `offering_build` you build at its place: its temper drops; a torch of thanks on your doorstep [3]
- `offering_item` you leave the thing: the want is met [4]
- `greeting` you say hello in chat: a door opens and closes, softly [2]
- `question` you ask what it wants in chat: a sign appears with the want's word [3]
- `apology` you say sorry after a provocation: half the temper move undone [3]

## The other house

A dimension of the lodger's own (owner, 2026-11-09), used twice at most, as the todo has it:
a void with a dark sky where your house stands copied block for block. Things that could happen
in it, in the one minute you are there:

- `other_wake` you wake in the copy's bed; the door is open onto nothing [4]
- `other_doorway` the figure in the doorway, facing in, not moving [4]
- `other_frames` every frame holds the thing [4]
- `other_signs` every sign reads the want's word [4]
- `other_chest` the chest holds everything it ever took [4]
- `other_window` through the window, your real house, far off, lit [4]
- `other_sounds` your own footsteps from the next room, walking your route [4]
- `other_bed` a second bed beside yours, slept in [4]
- `other_mirror` the copy is mirrored [5]
- `other_portal` in the `take` ending: its portal frame leads here; the walk ends at the copy's door [5]
- `other_return` you wake in your own bed, the door open, one thing moved [4]
- `other_witness` in multiplayer, a witness sees you asleep the whole minute [4]
- `other_memory` the copy is your house as it was on the first day of the stay, not as it is now [5]

## Time, habits and rituals

- `login_hour_event` something happens at the hour you usually log in, before you do, found at once [2]
- `logout_event` the door opens the moment you log out, seen by witnesses [2]
- `afk_watch` when you are idle a minute, the far figure appears; when you move, it is gone [3]
- `afk_items` when you are idle five minutes, the item in your main hand is... no inventory: the item on the table moves [3]
- `anniversary` on the seventh day of the stay, every torch it ever placed is lit at once for a minute, then as before [4]
- `your_first_day` on the first day of the stay, nothing at all [1]
- `your_birthday` nothing: no personal data
- `midnight_bell` the bell at midnight, every night of stage four [4]
- `dawn_figure` the figure at dawn, every day of stage four, somewhere different [4]
- `same_time` an event at the same in-game minute three days running [3]
- `countdown_torches` a torch fewer by its house each night of stage five [5]
- `death_memorial` where you died, a cairn with a torch and your dropped... (items you lost are gone; a cairn only) [3]
- `respawn_event` on respawn, the door is open [2]
- `rainy_day_quiet` nothing happens on a rainy day, ever [rule?]
- `full_moon_figure` the figure only on full-moon nights in stage three [3]
- `new_moon_sounds` the sounds only on new-moon nights [2]
- `weekend` ... no real-world data beyond the hour

## The want and gifts

- `gift_flower` a flower on the bed [2]
- `gift_food` cooked food on the table from your raw [2]
- `gift_ore` ore you never mined in your chest [3]
- `gift_diamond` one diamond in your chest, with a hollow in the rock somewhere [4]
- `gift_tool_mended` [3]
- `gift_map` a map of your routes (drawn) in a frame [4]
- `gift_house` its house, with a chest of your things, the door open [5]
- `gift_sapling_row` [3]
- `gift_back_everything` everything it took, back, the last morning [5]
- `gift_torch_at_grave` ... no
- `want_place_met` when you build at its place, its torches lead there and stop [4]
- `want_seen_met` when you have looked at it long enough, it waves, once [4]
- `want_thing_met` when you leave the thing, the thing is in a frame in its house [5]
- `want_refused` when you take the thing back, every frame in your house is empty by morning, the items on the bed [4]
- `want_shown` the figure points at its place [4]

## Endings and after

- `leaves_quiet` everything back, the figure at the edge of the light, gone at dawn [5]
- `leaves_bright` every torch lit at once, then dark, then nothing [5]
- `leaves_house` its house stays, empty, door open, as a memorial [5]
- `leaves_house_gone` its house is gone, the ground flat, one torch [5]
- `takes_hard` the one death, inventory kept, the fall-out-of-the-world message [5]
- `takes_normal` everything it took is gone for good, with it [5]
- `takes_witness` a witness sees it once, at the end [5]
- `keeps` it stays, stops acting, the lights stay on [5]
- `keeps_door` one door open every few weeks [after]
- `keeps_figure` the figure once a month at dawn [after]
- `keeps_house` its house gets one block a month [after]
- `reset_new_want` on `/lodger reset`, a new want, a new place [after]
- `second_stay` a second lodger never; the world is marked [rule]
- `your_copy_stays` the copy of your house stays [5]
- `book_left` a book with the memory's lines, on your bed, the last morning [5]

## Other mods

The host can see which mods are loaded, and for a few well-known ones offer hooks the lodger
uses when the mod is there and does nothing otherwise (owner, 2026-11-09). Each is a hint kind
the host implements against that mod's API, behind a check that the mod is present, so the
lodger's jar never depends on any of them.

- dynamic lights (LambDynamicLights, Sodium Dynamic Lights): the held torch gives no light for a minute during `lights_fail`; the figure carries a light the mod renders [3]
- shaders (Iris, Oculus): a shadow with no caster for a second; the figure's shadow arrives before it [4]
- minimaps (Xaero's, JourneyMap): your own marker twice, the second at its place; a waypoint named with the want's word appears and is gone; the figure as a player dot that fades [3]
- Distant Horizons: its house on the horizon, in low detail, before it exists [4]
- Simple Voice Chat: your own last words played back from behind you, half a second late, once; breathing from the next room in proximity chat; its footsteps in your group's channel [4]
- Sodium / Iris performance mods: nothing; they are the reason to keep the glitches cheap
- Fabric Skyboxes: a different sky for one second [4]
- Presence Footsteps: the late steps use its sound set so they match yours exactly [2]
- Ambient Sounds mods: the silence event silences those too [3]
- inventory and storage mods (Sophisticated Storage, Tom's Storage): the chest events reach their containers, through their APIs, so a player with no vanilla chests still has `open_chest` [2]
- Create: a contraption's gears turn one notch; a train horn far off at night [3]
- Farmer's Delight: the cooking pot is cooking what you had, overnight [2]
- Waystones: a waystone named with the word, at its place [4]
- Replay Mod: nothing it records is hidden; the glitches are in the recording, which is the point
- Essential / in-game friends list (vanilla 26.x): your own name shown online in the friends list for a minute, for you alone [3; client render only]
- Discord Rich Presence mods: nothing; nothing leaves the machine

## Chains

Sequences where one event sets the next up; the catalogue's [Chains](entity.md#chains) has the
eight that are in. More:

- the boat: `boat_moved` → `boat_adrift` → `figure_in_boat` → `boat_sunk` with the held item in its chest at the bottom → `boat_waiting` at the shore on the last day
- the field: `harvest_one` → `harvest_row` → `scarecrow` → `scarecrow_moved` nightly toward the house → `field_word` → the scarecrow in the doorway on the last night
- the bed: `egg_on_bed` → `pillow_item` → `shift_bed` → `turn_bed` → `second_bed` → `in_your_bed` → `woke_in_other_bed`
- the village: `bell` → `villager_at_door` → `golem_turned` → `village_doors` → `villagers_in_one_house` → `village_bell_moved` to your door
- the lights: `flicker` → `lights_fail` → `candles_lit` → `torch_every_other` → every light out but its path, the last night
- the water: `drip` → `bucket_floor` → `well` → `moat` → `bridge` → `bridge_finished` the last night, the figure crossing it
- the mine: `ore_marked` → `mined_out` → `mine_beside` → `mine_below` → `tunnel_toward` → `cave_room` → `stairs_down` to its place
- the sky: `sun_blink` → `moon_wrong` → `stars_out` → `night_early` → `red_rain` → `eclipse` on the last day
- the sounds: `late_steps` → `late_eating` → `knock` → `knock_floor` → `knock_bed` → `breathing_bed` → `your_voice`
- the pet: `pet_growl` → `pet_moved` → `pet_outside` → `pet_watching_spot` → `pet_follows_figure` → the pet at its door on the last night
- the frames: `frame_rotate` → `frames_turn` → `frame_swap` → `frame_favourite` → `frame_emptied` → `item_frame_your_face`
- the route: `late_steps` on the route → `route_marked` → `behind_you` → `on_the_route_ahead` → `route_away` → `hut_on_route` → the figure in the hut
- the redstone: `lever` → `lever_flipped_back` → `plate_laid` → `door_opens_itself` → `lamp_morse` → `piston_blocks_door` → every door open at once the last night
- the want of a place: `light_path` → `hole` → `pillar` → `sign` → `cairn_row` → `house_near` → `house_furnished` → `figure_at_its_door` → the last night at its house
- the want to be seen: `far_figure` → `behind_you` → `on_the_roof` → `in_doorway` → `stands_in_field` → `at_the_chest` → `wave` → `copy_crouch` → the last night, it does not leave your sight
- the want of a thing: `take` → `place_taken` far → `return_one_over` → `take` again → `tool_used` → `frame_favourite` in its house → `villager_word` trading it back → the last night, it holds it out at one block

Chains cross: the pet chain joins the watcher chain when the pet follows the figure; the mine
chain ends in the place chain's house if it is underground. Which chain opens depends on the
want and the surroundings, which is the brain's choice from the first week's menus.

## Set aside

Ideas that break a rule, or the design, kept so they are not reconsidered from scratch.

- **Free prose** beyond the phrase bank and, as an experiment, three constrained words from
  the model: signs with another player's name, books in free prose, chat to witnesses. Chat and
  the player list in the target's own name, to the target, moved into the catalogue on
  2026-11-09 (the owner's call); the model writing words is `brain.text: generate`, judged in
  the calibration pass.
- **Anything that makes an item from nothing:** a compass, a bell, a lodestone, rotten flesh in
  place of food, a message in a bottle. It digs, cuts, picks, cooks, crafts from what it has, and
  takes. A compass can appear if the player has iron and redstone in a chest and it crafts one.
- **Anything that spawns a mob:** a chicken jockey, a second lodger, a zombie with the player's
  skin. It redirects mobs that exist.
- **Harm before the ending:** TNT, fire on the house, lava, a trap, a fall. The fuse hiss is kept
  as a sound only, since nothing follows it.
- **Input control** beyond a nudge: moving the player, opening their inventory, dropping their
  items. The screen glitches render; they never act for the player.
- **Reading the client or other mods.** The senses read the server's own facts, which since
  2026-11-09 include the server's chat (the owner's call); the client's state and other mods'
  stay off limits.
- **Real-world data** beyond the clock: no birthdays, no weekends, no names.
- **Destroying what a player built,** even to rebuild it elsewhere. Moved within reach, same
  night, is the limit.
- **Following into the Nether or the End.** The portal things happen on this side.
- **Two figures at once,** or a figure in any skin but the target's. One of you is the design.
