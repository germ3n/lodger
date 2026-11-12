//! Chains (`docs/entity.md`, Chains): one thing leads to the next. A chain opens when its first
//! event lands; its next step then always has a place on the menu while it may happen, and more
//! weight in the mind's choosing, for a few days, and the situation
//! says what is open. A chain is never forced: `wait` is always offered, every step still
//! passes the rules, its cooldown, its cap and the day's budget, and a step whose moment passed
//! falls out and closes the chain. Each chain runs once a stay.

use serde::{Deserialize, Serialize};

use crate::memory::Memory;

/// When a step may come: always, once they noticed the last one, or once they looked long at
/// the figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    Always,
    Noticed,
    LookedLong,
}

/// One step: its event (or any of several), the days it waits for it, and when.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub events: &'static [&'static str],
    pub within_days: f32,
    pub when: When,
}

#[derive(Clone, Copy, Debug)]
pub struct Chain {
    pub id: &'static str,
    pub steps: &'static [Step],
    /// A scene: once a step lands, the next is rolled at once ([`SCENE_CHANCE`]) and, the roll
    /// won, comes ten seconds to a minute on, still passing its rules, at half its cost; lost, the
    /// chain goes on at its own pace.
    pub scene: bool,
}

/// Which of a trigger's branches opens: the event, the day and how much has happened mixed, so
/// the same trigger goes down different branches in different stays and moments.
fn branch_pick(event: &str, day: f32, done: usize) -> usize {
    let mut h: u64 = (day.to_bits() as u64) ^ (done as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for b in event.bytes() {
        h = h.wrapping_mul(0x0100_0000_01B3).wrapping_add(b as u64);
    }
    (h ^ (h >> 29)) as usize
}

/// The chance a scene's next step is rolled straight on, once a step of it lands.
pub const SCENE_CHANCE: f64 = 0.6;
/// How soon a scene's next step comes, the roll won: ten seconds to a minute, ticks.
pub const SCENE_LEAST_TICKS: u64 = 200;
pub const SCENE_MOST_TICKS: u64 = 1_200;
/// What a scene's step costs of the day's budget, as a share of its own: a scene is one thing.
pub const SCENE_COST: f32 = 0.5;

const fn step(events: &'static [&'static str], within_days: f32, when: When) -> Step {
    Step {
        events,
        within_days,
        when,
    }
}

use When::*;

/// Seventy-five (owner: "make 75 total chains then but redo all of them with the new rules"): the
/// five the code names kept as they were (the thing, the place, the watcher, the house, the
/// diary) and seventy drawn again over the whole catalogue, the Nether's and the End's among them.
/// An event may be in several where it fits; no two chains hold the same events; an event only a
/// moment brings comes only first.
pub const CHAINS: &[Chain] = &[
    Chain {
        id: "the_thing",
        steps: &[
            step(&["take"], 0.0, Always),
            step(
                &[
                    "place_taken",
                    "return_taken",
                    "sand_gives_back",
                    "pot_of_returns",
                ],
                3.0,
                Always,
            ),
            step(&["at_the_chest"], 3.0, Always),
            step(&["frame_emptied"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_place",
        steps: &[
            step(&["light_path"], 0.0, Always),
            step(&["hole"], 3.0, Always),
            step(&["pillar", "empty_lead"], 3.0, Always),
            step(&["sign", "sign_with_coords"], 3.0, Always),
            // Its house grows in the ending's script, not as an event: the chain ends at the house.
            step(&["house_near", "anchor_charged"], 4.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_watcher",
        steps: &[
            step(
                &["far_figure", "on_the_peak", "spyglass_figure"],
                0.0,
                Always,
            ),
            step(&["behind_you"], 3.0, LookedLong),
            step(&["on_the_roof"], 3.0, Always),
            step(&["in_doorway"], 3.0, Always),
            step(&["in_your_bed"], 3.0, Always),
            step(&["copy_crouch"], 3.0, Always),
            step(&["figure_waves", "figure_bows"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_house",
        steps: &[
            step(&["open_door"], 0.0, Always),
            step(
                &["door_hinge", "false_door", "trapdoor_to_nothing"],
                3.0,
                Noticed,
            ),
            step(&["door_held"], 3.0, Noticed),
            step(&["window_bars"], 4.0, Noticed),
        ],
        scene: false,
    },
    Chain {
        id: "the_diary",
        steps: &[
            step(&["diary_sold"], 0.0, Always),
            step(&["diary_brought"], 8.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_vein",
        steps: &[
            step(&["ore_marked", "ore_counted", "scan_ore"], 0.0, Always),
            step(&["late_drop"], 3.0, Always),
            step(&["mined_out"], 3.0, Always),
            step(&["dripstone_falls"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_tunnel",
        steps: &[
            step(&["mining_sounds"], 0.0, Always),
            step(&["mine_beside", "cart_overhead"], 3.0, Always),
            step(
                &["tunnel_toward", "iron_door_below", "dripstone_in_tunnel"],
                3.0,
                Always,
            ),
            step(&["cave_room", "skull_in_the_tunnel"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_way_back",
        steps: &[
            step(&["way_back_closes"], 0.0, Always),
            step(&["tunnel_sealed"], 3.0, Always),
            step(&["tunnel_goes_on"], 3.0, Always),
            step(&["torches_far_end"], 3.0, Always),
            step(&["sign_deep"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_copy",
        steps: &[
            step(&["copy_build"], 0.0, Always),
            step(&["copy_house"], 3.0, Always),
            step(&["second_bed"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_pet",
        steps: &[
            step(&["pet_growl"], 0.0, Always),
            step(&["pet_moved", "cat_gift"], 3.0, Always),
            step(&["bark_from_home"], 3.0, Always),
            step(&["pet_outside", "pet_whines_at_door"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_woods",
        steps: &[
            step(&["peeks_from_the_trees"], 0.0, Always),
            step(&["tree_to_tree"], 3.0, Always),
            step(&["branch_snaps"], 3.0, Always),
            step(&["knocks_on_wood"], 3.0, Always),
            step(&["the_tree_line"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_marked_trees",
        steps: &[
            step(&["blazed_trees"], 0.0, Always),
            step(&["circles_a_tree"], 3.0, Always),
            step(&["bare_trees"], 3.0, Always),
            step(&["trees_close_behind"], 3.0, Always),
            step(&["tree_comes_down"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_village_wrong",
        steps: &[
            step(&["trades_refused"], 0.0, Always),
            step(&["villager_spins"], 3.0, Always),
            step(&["villager_collapses"], 3.0, Always),
            step(&["villagers_walk_out"], 3.0, Always),
            step(&["villager_stabs"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_village_night",
        steps: &[
            step(&["village_lantern"], 0.0, Always),
            step(&["villager_word"], 3.0, Always),
            step(&["villager_at_door"], 3.0, Always),
            step(&["villager_flees_in"], 3.0, Always),
            step(&["villagers_flee_you"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_sky",
        steps: &[
            step(&["taken_up"], 0.0, Always),
            step(&["sky_gives_back"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_night",
        steps: &[
            step(&["drawn_to_the_light"], 0.0, Always),
            step(&["lantern_in_the_woods"], 3.0, Always),
            step(&["while_unseen"], 3.0, Always),
            step(&["wolves_circle"], 3.0, Always),
            step(&["zombie_in_your_gear"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_visitor",
        steps: &[
            step(&["prints_around_the_house", "trial_key_left"], 0.0, Always),
            step(&["red_trail"], 3.0, Always),
            step(
                &["boots_at_the_door", "dried_ghast_on_doorstep"],
                3.0,
                Always,
            ),
            step(&["wall_bursts"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_glitches",
        steps: &[
            step(&["dead_redstone"], 0.0, Always),
            step(&["crafter_clicks", "enchant_six"], 3.0, Always),
            step(&["sound_warp", "false_glint"], 3.0, Always),
            step(&["menu_freaks"], 3.0, Always),
            step(&["title_splash"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_knock",
        steps: &[
            step(&["knock"], 0.0, Always),
            step(&["door_scratch"], 3.0, Always),
            step(&["knock_heavy"], 3.0, Always),
            step(&["door_wont_open"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_bed",
        steps: &[
            step(&["egg_on_bed", "egg_on_the_pillow"], 0.0, Always),
            step(&["sheets_changed", "bed_fenced"], 3.0, Always),
            step(&["under_the_bed"], 3.0, Always),
            step(&["lies_beside"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_hands",
        steps: &[
            step(&["by_the_bed"], 0.0, Always),
            step(&["choke"], 3.0, Always),
            step(&["burned_in_bed"], 3.0, Always),
            step(&["shot_in_bed"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_voice",
        steps: &[
            step(&["says_back"], 0.0, Always),
            step(&["repeats_you"], 3.0, Always),
            step(&["your_voice"], 3.0, Always),
            step(&["says_first"], 3.0, Always),
            step(&["whisper_from_you"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_lights",
        steps: &[
            step(&["flicker", "fireflies_go_out"], 0.0, Always),
            step(&["lights_fail"], 3.0, Always),
            step(&["candles_lit"], 3.0, Always),
            step(&["torches_turn"], 3.0, Always),
            step(&["lights_out"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_sky_wrong",
        steps: &[
            step(&["thunder_clear"], 0.0, Always),
            step(&["moon_wrong"], 3.0, Always),
            step(&["clock_wrong", "compass_spins"], 3.0, Always),
            step(&["stars_out", "red_petals"], 3.0, Always),
            step(&["red_rain", "ash_on_the_wind"], 3.0, Always),
            step(&["night_early"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_screen",
        steps: &[
            step(&["hotbar_slip", "count_shows_zero"], 0.0, Always),
            step(&["crosshair_gone", "held_name_wrong"], 3.0, Always),
            step(&["inventory_stare", "recipe_toast"], 3.0, Always),
            step(&["phantom_item", "slot_barred"], 3.0, Always),
            step(&["static"], 3.0, Always),
            step(&["border_closes"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_village",
        steps: &[
            step(&["villager_at_door"], 0.0, Always),
            step(&["golem_turned"], 3.0, Always),
            step(&["village_doors"], 3.0, Always),
            step(&["village_abandoned"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_kills",
        steps: &[
            step(&["kill_sign"], 0.0, Always),
            step(&["kill_line"], 3.0, Always),
            step(&["head_on_post"], 3.0, Always),
            step(&["grave_each"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_water",
        steps: &[
            step(&["ripples"], 0.0, Always),
            step(&["boat_waiting"], 3.0, Always),
            step(&["on_the_water"], 3.0, Always),
            step(&["swims_beneath", "beam_from_the_water"], 3.0, Always),
            step(&["bell_underwater"], 3.0, Always),
            step(&["pulled_under"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_cave",
        steps: &[
            step(&["lantern_ahead"], 0.0, Always),
            step(&["cave_lit"], 3.0, Always),
            step(&["humming_ahead"], 3.0, Always),
            step(&["cave_torches_flicker"], 3.0, Always),
            step(&["drops_from_the_ceiling", "dripstone_falls"], 3.0, Always),
            step(&["in_the_lava"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_figures",
        steps: &[
            step(&["crossing_ahead"], 0.0, Always),
            step(&["in_a_line"], 3.0, Always),
            step(&["one_more_each_night"], 3.0, Always),
            step(&["ring_of_figures"], 3.0, Always),
            step(&["every_window"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_mirror",
        steps: &[
            step(&["figure_mirror"], 0.0, Always),
            step(&["reflection"], 3.0, Always),
            step(&["shadow_on_the_wall"], 3.0, Always),
            step(&["wears_your_face"], 3.0, Always),
            step(&["mirror_in_the_window"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_death",
        steps: &[
            step(&["death_light"], 0.0, Always),
            step(&["where_you_died"], 3.0, Always),
            step(&["death_message"], 3.0, Always),
            step(&["false_death"], 3.0, Always),
            step(&["your_grave"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_animals",
        steps: &[
            step(&["stare", "cattle_turn_away"], 0.0, Always),
            step(&["slow_approach", "the_child_stares"], 3.0, Always),
            step(&["warped_animal"], 3.0, Always),
            step(&["axolotls_play_dead", "armadillos_curl"], 3.0, Always),
            step(&["animals_turn", "refuses_food"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_field",
        steps: &[
            step(&["scarecrow"], 0.0, Always),
            step(&["row_withered"], 3.0, Always),
            step(&["field_word"], 3.0, Always),
            step(&["face_the_house"], 3.0, Always),
            step(&["pale_horse_in_your_pen"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_chest",
        steps: &[
            step(&["shuffle_chest", "shelf_shuffled"], 0.0, Always),
            step(&["tidy_chest", "tools_laid_out"], 3.0, Always),
            step(&["chest_rattles", "shulker_lid_peeks"], 3.0, Always),
            step(&["the_bell_in_the_chest", "bundle_gains"], 3.0, Always),
            step(&["chest_wont_open"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_steps",
        steps: &[
            step(&["late_steps"], 0.0, Always),
            step(&["footsteps_follow"], 3.0, Always),
            step(&["steps_counted"], 3.0, Always),
            step(&["armour_behind"], 3.0, Always),
            step(&["panting_behind"], 3.0, Always),
            step(&["hand_on_shoulder"], 3.0, Always),
            step(&["breath_close"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_bow",
        steps: &[
            step(&["arrow_near", "arrow_returned"], 0.0, Always),
            step(&["crossbow_click"], 3.0, Always),
            step(&["arrow_in_the_door"], 3.0, Always),
            step(&["arrow_grazes"], 3.0, Always),
            step(&["arrow_hits"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_walls",
        steps: &[
            step(&["breathing_in_the_wall"], 0.0, Always),
            step(&["eating_in_the_wall"], 3.0, Always),
            step(&["lava_in_the_wall"], 3.0, Always),
            step(&["scream_indoors"], 3.0, Always),
            step(&["cracks_in_the_wall"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_below",
        steps: &[
            step(&["bubbling_under"], 0.0, Always),
            step(&["digging_under_the_floor"], 3.0, Always),
            step(&["ominous_hum"], 3.0, Always),
            step(&["shriek_below"], 3.0, Always),
            step(&["sculk_creeps"], 3.0, Always),
            step(&["sculk_trail"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_pale_garden",
        steps: &[
            step(&["eyeblossoms_open"], 0.0, Always),
            step(&["creaking_behind"], 3.0, Always),
            step(&["pale_moss_creeps"], 3.0, Always),
            step(&["creaking_in_the_field"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_neglect",
        steps: &[
            step(&["cobwebs", "mushrooms_in_corners"], 0.0, Always),
            step(&["cold_house"], 3.0, Always),
            step(&["frost_on_the_glass"], 3.0, Always),
            step(&["window_tinted", "copper_ages"], 3.0, Always),
            step(&["stairs_up_to_nowhere"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_round",
        steps: &[
            step(&["rattling_round_the_house"], 0.0, Always),
            step(&["hoofbeats_circling"], 3.0, Always),
            step(&["ram_on_the_wall"], 3.0, Always),
            step(&["sigh_at_the_door"], 3.0, Always),
            step(&["climbing_the_house"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_night_in_bed",
        steps: &[
            step(&["vault_opens_by_the_bed"], 0.0, Always),
            step(&["snoring_beside"], 3.0, Always),
            step(&["bed_occupied"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_ceiling",
        steps: &[
            step(&["steps_above"], 0.0, Always),
            step(&["wings_in_the_room"], 3.0, Always),
            step(&["prints_on_the_ceiling"], 3.0, Always),
            step(&["ceiling_crawl"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_far_off",
        steps: &[
            step(&["witch_laughs"], 0.0, Always),
            step(&["doors_slam_far"], 3.0, Always),
            step(&["boom_far", "slam_crater"], 3.0, Always),
            step(&["rider_on_the_ridge"], 3.0, Always),
            step(&["happy_ghast_overhead"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_set_on_you",
        steps: &[
            step(&["creeper_behind", "gust_from_nowhere"], 0.0, Always),
            step(&["the_killer_rabbit", "teleport_beside"], 3.0, Always),
            step(&["vex_single", "fangs_in_a_line"], 3.0, Always),
            step(&["vex_wave"], 3.0, Always),
            step(&["vex_strikes"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_quiet_house",
        steps: &[
            step(&["sweeping_next_room"], 0.0, Always),
            step(&["pot_shatters"], 3.0, Always),
            step(&["bulbs_clicking"], 3.0, Always),
            step(&["water_where_none"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_hunt",
        steps: &[
            step(&["bush_rustles"], 0.0, Always),
            step(&["sniffing"], 3.0, Always),
            step(&["gasp_when_turning"], 3.0, Always),
            step(&["hunted"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_snow",
        steps: &[
            step(&["ice_cracks"], 0.0, Always),
            step(&["snowball_thrown"], 3.0, Always),
            step(&["snowball_comes_back"], 3.0, Always),
            step(&["name_in_the_snow"], 3.0, Always),
            step(&["snow_shape"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_kitchen",
        steps: &[
            step(&["cook_one"], 0.0, Always),
            step(&["take_output"], 3.0, Always),
            step(&["plate_laid"], 3.0, Always),
            step(&["at_their_table"], 3.0, Always),
            step(&["eats_at_their_chest"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_garden",
        steps: &[
            step(&["plant_flower"], 0.0, Always),
            step(&["flowers_potted"], 3.0, Always),
            step(&["replant_other"], 3.0, Always),
            step(&["flowers_wilted"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_flock",
        steps: &[
            step(&["sheep_dyed"], 0.0, Always),
            step(&["snipping_in_the_dark"], 3.0, Always),
            step(&["named_animal"], 3.0, Always),
            step(&["animal_gone"], 3.0, Always),
            step(&["animal_indoors"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_boat",
        steps: &[
            step(&["boat_moved"], 0.0, Always),
            step(&["boat_adrift", "boat_drifts"], 3.0, Always),
            step(&["boat_in_house"], 3.0, Always),
            step(&["boat_passenger"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_fisher",
        steps: &[
            step(&["fish_on_line"], 0.0, Always),
            step(&["fishing_line"], 3.0, Always),
            step(&["fishing_figure"], 3.0, Always),
            step(&["bucket_fish_changed"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_fog",
        steps: &[
            step(&["mist"], 0.0, Always),
            step(&["fog_close"], 3.0, Always),
            step(&["red_mist"], 3.0, Always),
            step(&["darkness"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_compass",
        steps: &[
            step(&["compass_points"], 0.0, Always),
            step(&["recovery_compass_lies"], 3.0, Always),
            step(&["lodestone_wrong"], 3.0, Always),
            step(&["eye_turns_home"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_books",
        steps: &[
            step(&["lectern_page"], 0.0, Always),
            step(&["book_word"], 3.0, Always),
            step(&["lectern_notes"], 3.0, Always),
            step(&["page_in_your_book"], 3.0, Always),
            step(&["letter_in_chest"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_signs",
        steps: &[
            step(&["sign_word"], 0.0, Always),
            step(&["words_on_sign"], 3.0, Always),
            step(&["sign_with_coords"], 3.0, Always),
            step(&["sign_deep"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_names",
        steps: &[
            step(&["named_animal"], 0.0, Always),
            step(&["villager_named"], 3.0, Always),
            step(&["name_called"], 3.0, Always),
            step(&["title_your_name"], 3.0, Always),
            step(&["name_in_the_snow"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_name_everywhere",
        steps: &[
            step(&["name_in_the_snow"], 0.0, Always),
            step(&["ghast_name"], 3.0, Always),
            step(&["name_in_netherrack"], 3.0, Always),
            step(&["name_from_the_void"], 3.0, Always),
            step(&["the_poem_knows"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_list",
        steps: &[
            step(&["tab_entry"], 0.0, Always),
            step(&["tab_twice"], 3.0, Always),
            step(&["joined"], 3.0, Always),
            step(&["tab_entry_stays"], 3.0, Always),
            step(&["left"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_music",
        steps: &[
            step(&["disc_phantom"], 0.0, Always),
            step(&["jukebox"], 3.0, Always),
            step(&["note_tune"], 3.0, Always),
            step(&["cut_music"], 3.0, Always),
            step(&["silence"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_redstone",
        steps: &[
            step(&["lever_click"], 0.0, Always),
            step(&["piston_once"], 3.0, Always),
            step(&["lamps_blink"], 3.0, Always),
            step(&["bulbs_clicking"], 3.0, Always),
            step(&["dead_redstone"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_brewing",
        steps: &[
            step(&["brewing_done"], 0.0, Always),
            step(&["potion_changed"], 3.0, Always),
            step(&["brew_left"], 3.0, Always),
            step(&["their_own_potion"], 3.0, Always),
            step(&["splash_dark"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_frames",
        steps: &[
            step(&["frame_rotate"], 0.0, Always),
            step(&["frames_turn"], 3.0, Always),
            step(&["item_frames_face"], 3.0, Always),
            step(&["frame_swap"], 3.0, Always),
            step(&["frame_emptied"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_windows",
        steps: &[
            step(&["window_tap"], 0.0, Always),
            step(&["glass_rubbed"], 3.0, Always),
            step(&["handprints"], 3.0, Always),
            step(&["waves_from_inside"], 3.0, Always),
            step(&["figure_window"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_portal",
        steps: &[
            step(&["portal_sound"], 0.0, Always),
            step(&["portal_lit"], 3.0, Always),
            step(
                &["someone_went_through", "ruined_portal_glows"],
                3.0,
                Always,
            ),
            step(&["standing_in_the_portal"], 3.0, Always),
            step(&["portal_goes_out"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_crossing",
        steps: &[
            step(
                &[
                    "prints_from_the_portal",
                    "came_back_wrong_hour",
                    "chorus_lands_home",
                    "waits_at_the_portal",
                ],
                0.0,
                Always,
            ),
            step(&["someone_went_through"], 3.0, Always),
            step(&["lodestone_wrong"], 3.0, Always),
            step(&["firework_over_its_house"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_nether",
        steps: &[
            step(&["piglins_stare", "ghast_watches"], 0.0, Always),
            step(&["their_door_behind", "rain_in_the_nether"], 3.0, Always),
            step(&["name_in_netherrack", "nether_roof_steps"], 3.0, Always),
            step(&["standing_in_the_portal"], 3.0, Always),
            step(&["portal_goes_out"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_fortress",
        steps: &[
            step(&["back_to_the_wall"], 0.0, Always),
            step(&["their_torch_ahead"], 3.0, Always),
            step(&["blaze_ring", "zombified_follow"], 3.0, Always),
            step(&["far_end_of_the_bridge"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_heat",
        steps: &[
            step(&["lava_goes_still", "breath_in_the_nether"], 0.0, Always),
            step(&["souls_follow", "prints_in_soul_sand"], 3.0, Always),
            step(
                &["vines_reach", "hoglins_part", "magma_cubes_settle"],
                3.0,
                Always,
            ),
            step(&["lightning_below", "splash_in_the_nether"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_way_home",
        steps: &[
            step(
                &[
                    "home_through_the_portal",
                    "compass_steadies",
                    "strider_turns_home",
                ],
                0.0,
                Always,
            ),
            step(&["portal_weeps"], 3.0, Always),
            step(&["barter_from_home"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_end",
        steps: &[
            step(&["endermen_turn_away"], 0.0, Always),
            step(&["name_from_the_void", "second_wings"], 3.0, Always),
            step(
                &["endermen_at_the_edge", "prints_off_the_edge"],
                3.0,
                Always,
            ),
            step(&["standing_on_the_void"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_city",
        steps: &[
            step(&["shulkers_open", "gateway_flashes"], 0.0, Always),
            step(
                &["purpur_knock", "ship_creaks", "chorus_blooms"],
                3.0,
                Always,
            ),
            step(&["elytra_at_home"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_island",
        steps: &[
            step(&["the_dragon_growls", "on_the_exit_portal"], 0.0, Always),
            step(&["end_goes_quiet", "end_sky_face"], 3.0, Always),
            step(&["the_poem_knows"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_glass_knock",
        steps: &[
            step(&["window_tap"], 0.0, Always),
            step(&["knock"], 3.0, Always),
            step(&["knock_heavy"], 3.0, Always),
            step(&["door_wont_open"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_steps_behind",
        steps: &[
            step(&["late_steps"], 0.0, Always),
            step(&["item_break_behind"], 3.0, Always),
            step(&["armour_behind"], 3.0, Always),
            step(&["panting_behind"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_dragging",
        steps: &[
            step(&["rattling_round_the_house"], 0.0, Always),
            step(&["chains_dragging"], 3.0, Always),
            step(&["scratching"], 3.0, Always),
            step(&["door_scratch"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_archer",
        steps: &[
            step(&["arrow_near"], 0.0, Always),
            step(&["arrow_hit"], 3.0, Always),
            step(&["arrow_in_the_door"], 3.0, Always),
            step(&["raises_their_sword"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_stalker",
        steps: &[
            step(&["bush_rustles"], 0.0, Always),
            step(&["snipping_in_the_dark"], 3.0, Always),
            step(&["sniffing"], 3.0, Always),
            step(&["hunted"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_wall_breaks",
        steps: &[
            step(&["breathing_in_the_wall"], 0.0, Always),
            step(&["digging_under_the_floor"], 3.0, Always),
            step(&["bubbling_under"], 3.0, Always),
            step(&["cracks_in_the_wall"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_flooding",
        steps: &[
            step(&["bubbling_under"], 0.0, Always),
            step(&["water_where_none"], 3.0, Always),
            step(&["ominous_hum"], 3.0, Always),
            step(&["sculk_trail"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_loft",
        steps: &[
            step(&["steps_above"], 0.0, Always),
            step(&["scratching"], 3.0, Always),
            step(&["trapdoor_taps"], 3.0, Always),
            step(&["ceiling_crawl"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_ambush",
        steps: &[
            step(&["creeper_behind"], 0.0, Always),
            step(&["arrow_near"], 3.0, Always),
            step(&["vex_single"], 3.0, Always),
            step(&["vex_strikes"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_failing",
        steps: &[
            step(&["hotbar_slip"], 0.0, Always),
            step(&["hearts_drop"], 3.0, Always),
            step(&["xp_slips"], 3.0, Always),
            step(&["false_death"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_nether_watchers",
        steps: &[
            step(&["piglins_stare"], 0.0, Always),
            step(&["nether_roof_steps"], 3.0, Always),
            step(&["zombified_follow"], 3.0, Always),
            step(&["far_end_of_the_bridge"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_end_closes",
        steps: &[
            step(&["endermen_turn_away"], 0.0, Always),
            step(&["the_dragon_growls"], 3.0, Always),
            step(&["end_goes_quiet"], 3.0, Always),
            step(&["standing_on_the_void"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_dark_circuit",
        steps: &[
            step(&["lever_click"], 0.0, Always),
            step(&["dead_redstone"], 3.0, Always),
            step(&["lights_fail"], 3.0, Always),
            step(&["lights_out"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_deep_tunnel",
        steps: &[
            step(&["mining_sounds"], 0.0, Always),
            step(&["echo_steps"], 3.0, Always),
            step(&["humming_ahead"], 3.0, Always),
            step(&["shriek_below"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_burning",
        steps: &[
            step(&["lava_goes_still"], 0.0, Always),
            step(&["lightning_below"], 3.0, Always),
            step(&["blaze_ring"], 3.0, Always),
            step(&["portal_goes_out"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_mist_figure",
        steps: &[
            step(&["mist"], 0.0, Always),
            step(&["fog_close"], 3.0, Always),
            step(&["figure_window"], 3.0, Always),
            step(&["in_doorway"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_storm",
        steps: &[
            step(&["thunder_clear"], 0.0, Always),
            step(&["lightning_silent"], 3.0, Always),
            step(&["boom_far"], 3.0, Always),
            step(&["falls_from_the_sky"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_meal",
        steps: &[
            step(&["cook_one"], 0.0, Always),
            step(&["late_eating"], 3.0, Always),
            step(&["eats_at_their_chest"], 3.0, Always),
            step(&["at_their_table"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_wilting",
        steps: &[
            step(&["plant_flower"], 0.0, Always),
            step(&["flowers_wilted"], 3.0, Always),
            step(&["pumpkin_face"], 3.0, Always),
            step(&["scarecrow"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_marked_flock",
        steps: &[
            step(&["sheep_dyed"], 0.0, Always),
            step(&["named_animal"], 3.0, Always),
            step(&["warped_animal"], 3.0, Always),
            step(&["animals_turn"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_drift",
        steps: &[
            step(&["boat_moved"], 0.0, Always),
            step(&["boat_waiting"], 3.0, Always),
            step(&["on_the_water"], 3.0, Always),
            step(&["swims_beneath"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_still_water",
        steps: &[
            step(&["fish_on_line"], 0.0, Always),
            step(&["ripples"], 3.0, Always),
            step(&["across_water"], 3.0, Always),
            step(&["reflection"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_reader",
        steps: &[
            step(&["lectern_page"], 0.0, Always),
            step(&["page_turn"], 3.0, Always),
            step(&["lectern_notes"], 3.0, Always),
            step(&["figure_at_the_lectern"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_counting",
        steps: &[
            step(&["sign_word"], 0.0, Always),
            step(&["sign_with_coords"], 3.0, Always),
            step(&["kill_sign"], 3.0, Always),
            step(&["head_on_post"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_guest",
        steps: &[
            step(&["tab_entry"], 0.0, Always),
            step(&["joined"], 3.0, Always),
            step(&["says_first"], 3.0, Always),
            step(&["your_voice"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_bells",
        steps: &[
            step(&["disc_phantom"], 0.0, Always),
            step(&["note_tune"], 3.0, Always),
            step(&["bell_rings"], 3.0, Always),
            step(&["the_count"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_thrown",
        steps: &[
            step(&["brewing_done"], 0.0, Always),
            step(&["splash_dark"], 3.0, Always),
            step(&["potion_at_the_window"], 3.0, Always),
            step(&["their_own_potion"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_paintings",
        steps: &[
            step(&["frame_rotate"], 0.0, Always),
            step(&["painting_flip"], 3.0, Always),
            step(&["painting_change"], 3.0, Always),
            step(&["frame_emptied"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_old_portal",
        steps: &[
            step(&["portal_sound"], 0.0, Always),
            step(&["someone_went_through"], 3.0, Always),
            step(&["ruined_portal_glows"], 3.0, Always),
            step(&["portal_lit"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_snow_prints",
        steps: &[
            step(&["ice_cracks"], 0.0, Always),
            step(&["snow_cleared"], 3.0, Always),
            step(&["prints_around_the_house"], 3.0, Always),
            step(&["snow_shape"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_canopy",
        steps: &[
            step(&["peeks_from_the_trees"], 0.0, Always),
            step(&["crouched_behind_a_tree"], 3.0, Always),
            step(&["face_in_the_canopy"], 3.0, Always),
            step(&["path_through_the_trees"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_blazed_way",
        steps: &[
            step(&["blazed_trees"], 0.0, Always),
            step(&["trail"], 3.0, Always),
            step(&["trail_in"], 3.0, Always),
            step(&["prints_to_the_cave"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_shunned",
        steps: &[
            step(&["trades_refused"], 0.0, Always),
            step(&["villager_word"], 3.0, Always),
            step(&["villager_named"], 3.0, Always),
            step(&["villagers_flee_you"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_empty_village",
        steps: &[
            step(&["village_lantern"], 0.0, Always),
            step(&["village_doors"], 3.0, Always),
            step(&["village_abandoned"], 3.0, Always),
            step(&["villager_collapses"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_far_sky",
        steps: &[
            step(&["witch_laughs"], 0.0, Always),
            step(&["boom_far"], 3.0, Always),
            step(&["happy_ghast_overhead"], 3.0, Always),
            step(&["falls_from_the_sky"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_bloom",
        steps: &[
            step(&["eyeblossoms_open"], 0.0, Always),
            step(&["fireflies_go_out"], 3.0, Always),
            step(&["red_petals"], 3.0, Always),
            step(&["creaking_behind"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_dust",
        steps: &[
            step(&["cobwebs"], 0.0, Always),
            step(&["bats_in_the_house"], 3.0, Always),
            step(&["pet_stares_at_wall"], 3.0, Always),
            step(&["cold_house"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_vault",
        steps: &[
            step(&["vault_opens_by_the_bed"], 0.0, Always),
            step(&["ominous_hum"], 3.0, Always),
            step(&["bed_breaks"], 3.0, Always),
            step(&["shot_in_bed"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_other_room",
        steps: &[
            step(&["sweeping_next_room"], 0.0, Always),
            step(&["furniture_shifted"], 3.0, Always),
            step(&["carpet_shift"], 3.0, Always),
            step(&["water_where_none"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_wrong_way",
        steps: &[
            step(&["compass_points"], 0.0, Always),
            step(&["figure_on_map"], 3.0, Always),
            step(&["breadcrumbs"], 3.0, Always),
            step(&["eye_turns_home"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_souls",
        steps: &[
            step(&["home_through_the_portal"], 0.0, Always),
            step(&["souls_follow"], 3.0, Always),
            step(&["portal_weeps"], 3.0, Always),
            step(&["standing_in_the_portal"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_ship",
        steps: &[
            step(&["shulkers_open"], 0.0, Always),
            step(&["chorus_blooms"], 3.0, Always),
            step(&["end_sky_face"], 3.0, Always),
            step(&["elytra_at_home"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_fortress_halls",
        steps: &[
            step(&["back_to_the_wall"], 0.0, Always),
            step(&["zombified_follow"], 3.0, Always),
            step(&["nether_roof_steps"], 3.0, Always),
            step(&["standing_in_the_portal"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_long_night",
        steps: &[
            step(&["drawn_to_the_light"], 0.0, Always),
            step(&["torch_ring"], 3.0, Always),
            step(&["stone_circle"], 3.0, Always),
            step(&["ring_of_figures"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_climber",
        steps: &[
            step(&["window_tap"], 0.0, Always),
            step(&["handprints"], 3.0, Always),
            step(&["eyes_in_the_dark"], 3.0, Always),
            step(&["climbing_the_house"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_turn",
        steps: &[
            step(&["late_steps"], 0.0, Always),
            step(&["footsteps_follow"], 3.0, Always),
            step(&["hand_on_shoulder"], 3.0, Always),
            step(&["gasp_when_turning"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_pack",
        steps: &[
            step(&["creeper_behind"], 0.0, Always),
            step(&["the_killer_rabbit"], 3.0, Always),
            step(&["wolves_circle"], 3.0, Always),
            step(&["daylight_phantom"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_breakdown",
        steps: &[
            step(&["hotbar_slip"], 0.0, Always),
            step(&["static"], 3.0, Always),
            step(&["border_closes"], 3.0, Always),
            step(&["in_the_screenshot"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_red_fog",
        steps: &[
            step(&["mist"], 0.0, Always),
            step(&["red_mist"], 3.0, Always),
            step(&["darkness"], 3.0, Always),
            step(&["eyes_in_the_dark"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_cold_hearth",
        steps: &[
            step(&["cook_one"], 0.0, Always),
            step(&["cold_furnace"], 3.0, Always),
            step(&["cold_crackle"], 3.0, Always),
            step(&["put_out_campfire"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_seedlings",
        steps: &[
            step(&["plant_flower"], 0.0, Always),
            step(&["replant_other"], 3.0, Always),
            step(&["tree_grown"], 3.0, Always),
            step(&["sapling"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_lost_sheep",
        steps: &[
            step(&["sheep_dyed"], 0.0, Always),
            step(&["snipping_in_the_dark"], 3.0, Always),
            step(&["animal_gone"], 3.0, Always),
            step(&["pale_horse_in_your_pen"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_line_pulled",
        steps: &[
            step(&["fish_on_line"], 0.0, Always),
            step(&["fishing_figure"], 3.0, Always),
            step(&["bucket_fish_changed"], 3.0, Always),
            step(&["beam_from_the_water"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_written",
        steps: &[
            step(&["lectern_page"], 0.0, Always),
            step(&["book_word"], 3.0, Always),
            step(&["page_in_your_book"], 3.0, Always),
            step(&["letter_in_chest"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_connection",
        steps: &[
            step(&["tab_entry"], 0.0, Always),
            step(&["tab_entry_stays"], 3.0, Always),
            step(&["ping_fades"], 3.0, Always),
            step(&["left"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_bats",
        steps: &[
            step(&["steps_above"], 0.0, Always),
            step(&["wings_in_the_room"], 3.0, Always),
            step(&["bats_in_the_house"], 3.0, Always),
            step(&["prints_on_the_ceiling"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_wrong_stars",
        steps: &[
            step(&["thunder_clear"], 0.0, Always),
            step(&["red_rain"], 3.0, Always),
            step(&["stars_out"], 3.0, Always),
            step(&["moon_wrong"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_howls",
        steps: &[
            step(&["witch_laughs"], 0.0, Always),
            step(&["doors_slam_far"], 3.0, Always),
            step(&["wolf_far"], 3.0, Always),
            step(&["rider_on_the_ridge"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_pale_field",
        steps: &[
            step(&["eyeblossoms_open"], 0.0, Always),
            step(&["pale_moss_creeps"], 3.0, Always),
            step(&["creaking_in_the_field"], 3.0, Always),
            step(&["face_in_the_canopy"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_sleeper",
        steps: &[
            step(&["vault_opens_by_the_bed"], 0.0, Always),
            step(&["under_the_bed"], 3.0, Always),
            step(&["snoring_beside"], 3.0, Always),
            step(&["lies_beside"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_crossing_over",
        steps: &[
            step(&["portal_sound"], 0.0, Always),
            step(&["portal_lit"], 3.0, Always),
            step(&["standing_in_the_portal"], 3.0, Always),
            step(&["name_in_netherrack"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_traded_back",
        steps: &[
            step(&["piglins_stare"], 0.0, Always),
            step(&["their_door_behind"], 3.0, Always),
            step(&["home_through_the_portal"], 3.0, Always),
            step(&["barter_from_home"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_wings",
        steps: &[
            step(&["endermen_turn_away"], 0.0, Always),
            step(&["second_wings"], 3.0, Always),
            step(&["prints_off_the_edge"], 3.0, Always),
            step(&["elytra_at_home"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_slow_lava",
        steps: &[
            step(&["lava_goes_still"], 0.0, Always),
            step(&["souls_follow"], 3.0, Always),
            step(&["vines_reach"], 3.0, Always),
            step(&["magma_cubes_settle"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_fortress_fire",
        steps: &[
            step(&["back_to_the_wall"], 0.0, Always),
            step(&["their_torch_ahead"], 3.0, Always),
            step(&["blaze_ring"], 3.0, Always),
            step(&["lightning_below"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_stones",
        steps: &[
            step(&["out_of_place"], 0.0, Always),
            step(&["stone_circle"], 3.0, Always),
            step(&["cairn"], 3.0, Always),
            step(&["grave"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_frozen",
        steps: &[
            step(&["stare"], 0.0, Always),
            step(&["hostiles_stare"], 3.0, Always),
            step(&["all_freeze"], 3.0, Always),
            step(&["held_breath"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_eggs",
        steps: &[
            step(&["egg_on_bed"], 0.0, Always),
            step(&["chicken_lays"], 3.0, Always),
            step(&["eggs_to_door"], 3.0, Always),
            step(&["animal_indoors"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_last_death",
        steps: &[
            step(&["death_light"], 0.0, Always),
            step(&["death_message"], 3.0, Always),
            step(&["where_you_died"], 3.0, Always),
            step(&["respawn_its_bed"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_wet",
        steps: &[
            step(&["ripples"], 0.0, Always),
            step(&["wet_steps"], 3.0, Always),
            step(&["water_where_none"], 3.0, Always),
            step(&["swims_beneath"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_old_cave",
        steps: &[
            step(&["lantern_ahead"], 0.0, Always),
            step(&["cave_air"], 3.0, Always),
            step(&["bones_underfoot"], 3.0, Always),
            step(&["drops_from_the_ceiling"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_crossers",
        steps: &[
            step(&["crossing_ahead"], 0.0, Always),
            step(&["jumping_far"], 3.0, Always),
            step(&["across_water"], 3.0, Always),
            step(&["on_the_peak"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_copy_moves",
        steps: &[
            step(&["figure_mirror"], 0.0, Always),
            step(&["copy_crouch"], 3.0, Always),
            step(&["only_in_third_person"], 3.0, Always),
            step(&["wears_your_face"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_crowd",
        steps: &[
            step(&["scarecrow"], 0.0, Always),
            step(&["pumpkin_face"], 3.0, Always),
            step(&["animals_turn"], 3.0, Always),
            step(&["figure_crowd"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_stacks",
        steps: &[
            step(&["shuffle_chest"], 0.0, Always),
            step(&["swap_stacks"], 3.0, Always),
            step(&["stack_split"], 3.0, Always),
            step(&["fill_gap"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_shrieking",
        steps: &[
            step(&["bubbling_under"], 0.0, Always),
            step(&["bones_underfoot"], 3.0, Always),
            step(&["shriek_below"], 3.0, Always),
            step(&["sculk_creeps"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_dead_wires",
        steps: &[
            step(&["dead_redstone"], 0.0, Always),
            step(&["piston_once"], 3.0, Always),
            step(&["lamps_blink"], 3.0, Always),
            step(&["bulbs_clicking"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_your_voice",
        steps: &[
            step(&["says_back"], 0.0, Always),
            step(&["whisper_from_you"], 3.0, Always),
            step(&["your_voice"], 3.0, Always),
            step(&["tab_entry_stays"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_blinking",
        steps: &[
            step(&["flicker"], 0.0, Always),
            step(&["sun_blink"], 3.0, Always),
            step(&["lightning_silent"], 3.0, Always),
            step(&["lights_fail"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_villager_dying",
        steps: &[
            step(&["villager_at_door"], 0.0, Always),
            step(&["villager_word"], 3.0, Always),
            step(&["villager_flees_in"], 3.0, Always),
            step(&["villager_collapses"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_graves",
        steps: &[
            step(&["kill_sign"], 0.0, Always),
            step(&["grave"], 3.0, Always),
            step(&["grave_each"], 3.0, Always),
            step(&["your_grave"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_pet_knows",
        steps: &[
            step(&["pet_growl"], 0.0, Always),
            step(&["pet_stares_at_wall"], 3.0, Always),
            step(&["pet_follows"], 3.0, Always),
            step(&["pet_whines_at_door"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_unwatched",
        steps: &[
            step(&["peeks_from_the_trees"], 0.0, Always),
            step(&["while_unseen"], 3.0, Always),
            step(&["tree_to_tree"], 3.0, Always),
            step(&["the_tree_line"], 3.0, Always),
        ],
        scene: true,
    },
    Chain {
        id: "the_turned_away",
        steps: &[
            step(&["trades_refused"], 0.0, Always),
            step(&["villager_spins"], 3.0, Always),
            step(&["villagers_flee_you"], 3.0, Always),
            step(&["villager_stabs"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_one_inside",
        steps: &[
            step(&["prints_around_the_house"], 0.0, Always),
            step(&["footsteps_follow"], 3.0, Always),
            step(&["door_scratch"], 3.0, Always),
            step(&["in_doorway"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_named",
        steps: &[
            step(&["named_animal"], 0.0, Always),
            step(&["animal_follows"], 3.0, Always),
            step(&["animal_indoors"], 3.0, Always),
            step(&["the_child_stares"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_dug",
        steps: &[
            step(&["hole"], 0.0, Always),
            step(&["cairn"], 3.0, Always),
            step(&["pillar"], 3.0, Always),
            step(&["stone_circle"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_filled",
        steps: &[
            step(&["hole"], 0.0, Always),
            step(&["backfill"], 3.0, Always),
            step(&["digging_under_the_floor"], 3.0, Always),
            step(&["bones_underfoot"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_torches",
        steps: &[
            step(&["add_torch"], 0.0, Always),
            step(&["move_light"], 3.0, Always),
            step(&["torches_turn"], 3.0, Always),
            step(&["torch_blue"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_torch_ring",
        steps: &[
            step(&["add_torch"], 0.0, Always),
            step(&["torch_ring"], 3.0, Always),
            step(&["light_taken"], 3.0, Always),
            step(&["lights_out"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_chest_kept",
        steps: &[
            step(&["open_chest"], 0.0, Always),
            step(&["shuffle_chest"], 3.0, Always),
            step(&["chest_rattles"], 3.0, Always),
            step(&["chest_wont_open"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_chest_alive",
        steps: &[
            step(&["open_chest"], 0.0, Always),
            step(&["breath_in_the_chest"], 3.0, Always),
            step(&["the_bell_in_the_chest"], 3.0, Always),
            step(&["a_face_in_the_chest"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_noon",
        steps: &[
            step(&["dusk_noon"], 0.0, Always),
            step(&["lamps_noon"], 3.0, Always),
            step(&["sun_blink"], 3.0, Always),
            step(&["night_early"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_lost_day",
        steps: &[
            step(&["dusk_noon"], 0.0, Always),
            step(&["stars_out"], 3.0, Always),
            step(&["moon_wrong"], 3.0, Always),
            step(&["lost_hour"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_unrest",
        steps: &[
            step(&["refuse_rest"], 0.0, Always),
            step(&["sleepers_wrong"], 3.0, Always),
            step(&["bed_occupied"], 3.0, Always),
            step(&["sleepwalk"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_woken",
        steps: &[
            step(&["refuse_rest"], 0.0, Always),
            step(&["wake"], 3.0, Always),
            step(&["woke_elsewhere"], 3.0, Always),
            step(&["in_your_bed"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_body",
        steps: &[
            step(&["hunger_shake"], 0.0, Always),
            step(&["heart_skip"], 3.0, Always),
            step(&["hearts_drop"], 3.0, Always),
            step(&["xp_slips"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_eye",
        steps: &[
            step(&["hunger_shake"], 0.0, Always),
            step(&["camera_nudge"], 3.0, Always),
            step(&["fov_pull"], 3.0, Always),
            step(&["only_in_third_person"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_frost",
        steps: &[
            step(&["frost_on_the_glass"], 0.0, Always),
            step(&["snow_indoors"], 3.0, Always),
            step(&["ice_path"], 3.0, Always),
            step(&["snowman"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_breath_on_glass",
        steps: &[
            step(&["frost_on_the_glass"], 0.0, Always),
            step(&["glass_rubbed"], 3.0, Always),
            step(&["handprints"], 3.0, Always),
            step(&["figure_window"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_copper",
        steps: &[
            step(&["copper_golem_at_door"], 0.0, Always),
            step(&["copper_ages"], 3.0, Always),
            step(&["golem_statue_turns"], 3.0, Always),
            step(&["bulbs_clicking"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_golem",
        steps: &[
            step(&["copper_golem_at_door"], 0.0, Always),
            step(&["golem_turned"], 3.0, Always),
            step(&["village_doors"], 3.0, Always),
            step(&["crafter_clicks"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_harvest",
        steps: &[
            step(&["harvest_one"], 0.0, Always),
            step(&["row_withered"], 3.0, Always),
            step(&["trample"], 3.0, Always),
            step(&["field_word"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_supper",
        steps: &[
            step(&["harvest_one"], 0.0, Always),
            step(&["plate_laid"], 3.0, Always),
            step(&["at_their_table"], 3.0, Always),
            step(&["eats_at_their_chest"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_horse",
        steps: &[
            step(&["horse_moved"], 0.0, Always),
            step(&["rides_their_horse"], 3.0, Always),
            step(&["horse_balks"], 3.0, Always),
            step(&["pale_horse_in_your_pen"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_open_gate",
        steps: &[
            step(&["horse_moved"], 0.0, Always),
            step(&["open_gate"], 3.0, Always),
            step(&["animal_gone"], 3.0, Always),
            step(&["hoofbeats_circling"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_map",
        steps: &[
            step(&["map_marked"], 0.0, Always),
            step(&["figure_on_map"], 3.0, Always),
            step(&["breadcrumbs"], 3.0, Always),
            step(&["prints_to_the_cave"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_ore_map",
        steps: &[
            step(&["map_marked"], 0.0, Always),
            step(&["ore_marked"], 3.0, Always),
            step(&["mined_out"], 3.0, Always),
            step(&["cave_room"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_trees",
        steps: &[
            step(&["tree_grown"], 0.0, Always),
            step(&["sapling"], 3.0, Always),
            step(&["circles_a_tree"], 3.0, Always),
            step(&["tree_comes_down"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_treetops",
        steps: &[
            step(&["tree_grown"], 0.0, Always),
            step(&["in_the_treetops"], 3.0, Always),
            step(&["face_in_the_canopy"], 3.0, Always),
            step(&["branch_snaps"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_echoes",
        steps: &[
            step(&["cave_air"], 0.0, Always),
            step(&["echo_steps"], 3.0, Always),
            step(&["bones_underfoot"], 3.0, Always),
            step(&["shriek_below"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_dimming",
        steps: &[
            step(&["cave_air"], 0.0, Always),
            step(&["torches_far_end"], 3.0, Always),
            step(&["cave_torches_flicker"], 3.0, Always),
            step(&["lights_out"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_bell",
        steps: &[
            step(&["bell"], 0.0, Always),
            step(&["bell_rings"], 3.0, Always),
            step(&["the_count"], 3.0, Always),
            step(&["bell_underwater"], 3.0, Always),
        ],
        scene: false,
    },
    Chain {
        id: "the_bell_calls",
        steps: &[
            step(&["bell"], 0.0, Always),
            step(&["villager_at_door"], 3.0, Always),
            step(&["village_abandoned"], 3.0, Always),
            step(&["villagers_walk_out"], 3.0, Always),
        ],
        scene: false,
    },
];

/// How much likelier an open chain's next step is on the menu.
pub const WEIGHT: f64 = 4.0;

/// A chain under way: the steps landed, and the day the last one did.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Open {
    pub id: String,
    /// Which of its steps have landed, by index: any order counts, the first step always.
    pub landed: Vec<usize>,
    /// The day its waiting began: the last step's landing, held on while its stage has not come.
    pub since: f32,
    /// The day its last step landed: what a noticing or a look must come after.
    pub landed_day: f32,
    /// Which step landed last: the one a `Noticed` step needs them to have noticed (steps land
    /// in any order, and the one before by index may be older).
    pub last: usize,
}

impl Open {
    /// The first step not landed yet: the one the menu weighs up.
    pub fn next(&self, chain: &Chain) -> Option<usize> {
        (0..chain.steps.len()).find(|i| !self.landed.contains(i))
    }
}

pub fn find(id: &str) -> Option<&'static Chain> {
    CHAINS.iter().find(|c| c.id == id)
}

impl Memory {
    /// An event landed: the chain it begins opens, a chain it is a step of counts it (in any
    /// order: a pillar before the hole is still the place), and a chain whose every step has
    /// landed is walked. Once a stay each.
    pub fn chain_landed(&mut self, event: &str, day: f32) {
        for open in &mut self.chains {
            let Some(chain) = find(&open.id) else {
                continue;
            };
            for (i, step) in chain.steps.iter().enumerate() {
                if step.events.contains(&event) && !open.landed.contains(&i) {
                    open.landed.push(i);
                    open.since = day;
                    open.landed_day = day;
                    open.last = i;
                    if chain.scene {
                        self.scene_moved.push(chain.id);
                    }
                    break;
                }
            }
        }
        let walked: Vec<String> = self
            .chains
            .iter()
            .filter(|o| find(&o.id).is_none_or(|c| o.next(c).is_none()))
            .map(|o| o.id.clone())
            .collect();
        for id in walked {
            self.chains.retain(|o| o.id != id);
            if !self.chains_walked.contains(&id) {
                self.chains_walked.push(id.clone());
            }
            if !self.chains_done.contains(&id) {
                self.chains_done.push(id);
            }
        }
        self.open_branch(event, day, CHAINS);
    }

    /// The chain an event begins. The chains whose first step it is are branches of one trigger
    /// (owner: "multiple possible chains to go down per trigger"): one of them opens, picked from
    /// those not yet run this stay, and none while one of them is open; the trigger landing again
    /// after, another may.
    fn open_branch(&mut self, event: &str, day: f32, chains: &'static [Chain]) {
        let begins_here = |c: &Chain| c.steps[0].events.contains(&event);
        let branch_open = self.chains.iter().any(|o| {
            chains
                .iter()
                .find(|c| c.id == o.id)
                .is_some_and(&begins_here)
        });
        if branch_open {
            return;
        }
        let branches: Vec<&Chain> = chains
            .iter()
            .filter(|c| {
                begins_here(c)
                    && !self.chains.iter().any(|o| o.id == c.id)
                    && !self.chains_done.iter().any(|d| d == c.id)
            })
            .collect();
        if branches.is_empty() {
            return;
        }
        let chain = branches[branch_pick(event, day, self.done.len()) % branches.len()];
        if chain.scene {
            self.scene_moved.push(chain.id);
        }
        self.chains.push(Open {
            id: chain.id.to_string(),
            landed: vec![0],
            since: day,
            landed_day: day,
            last: 0,
        });
    }

    /// The chains whose next step's moment has passed fall out; the watcher ends when they
    /// chased the figure instead.
    pub fn chains_tick(&mut self, day: f32, stage: u8) {
        // A next step not offered in this stage yet waits for its stage, its days not counted;
        // the bold ones by temper's stage, as the menu offers them.
        let temper = self.temper.clone();
        for open in &mut self.chains {
            let Some(chain) = find(&open.id) else {
                continue;
            };
            let waiting = open.next(chain).is_some_and(|i| {
                chain.steps[i]
                    .events
                    .iter()
                    .filter_map(|e| crate::events::find(e))
                    .all(|e| {
                        !crate::temper::offered_in(e, stage, &temper)
                            && (stage + 1..=5).any(|s| crate::temper::offered_in(e, s, &temper))
                    })
            });
            if waiting {
                open.since = day;
            }
        }
        // Chased since the watcher began, not before it: an older chase ended it at once.
        let chased_since = |since: f32| {
            self.figure_looks
                .last()
                .is_some_and(|l| l.chased() && l.day >= day - 0.5 && l.day >= since)
        };
        let mut gone = Vec::new();
        for open in &self.chains {
            let Some(chain) = find(&open.id) else {
                gone.push(open.id.clone());
                continue;
            };
            let late = open
                .next(chain)
                .and_then(|i| chain.steps.get(i))
                .is_none_or(|s| day - open.since > s.within_days);
            if late || (chain.id == "the_watcher" && chased_since(open.landed_day)) {
                gone.push(open.id.clone());
            }
        }
        for id in gone {
            self.chains.retain(|o| o.id != id);
            if !self.chains_done.contains(&id) {
                self.chains_done.push(id);
            }
        }
    }

    /// The open scenes' next steps, ready: the other worlds take them at a scene's pace.
    pub fn scene_next(&self) -> Vec<&'static str> {
        self.chains
            .iter()
            .filter(|o| find(&o.id).is_some_and(|c| c.scene))
            .flat_map(|o| self.chain_next_of(&o.id))
            .collect()
    }

    /// One open chain's next step, its condition met: a scene's, rolled straight on.
    pub fn chain_next_of(&self, id: &str) -> Vec<&'static str> {
        let mut next = self.chain_next_where(|o| o.id == id);
        next.dedup();
        next
    }

    /// The events an open chain offers next, its condition met: the menu weighs them up.
    pub fn chain_next(&self) -> Vec<&'static str> {
        self.chain_next_where(|_| true)
    }

    fn chain_next_where(&self, which: impl Fn(&Open) -> bool) -> Vec<&'static str> {
        let mut next = Vec::new();
        for open in self.chains.iter().filter(|o| which(o)) {
            let Some(chain) = find(&open.id) else {
                continue;
            };
            let Some(at) = open.next(chain) else { continue };
            let step = &chain.steps[at];
            let ready = match step.when {
                When::Always => true,
                When::LookedLong => self
                    .figure_looks
                    .iter()
                    .any(|l| l.looked_long() && l.day >= open.landed_day),
                When::Noticed => {
                    let last = chain.steps.get(open.last).map(|s| s.events).unwrap_or(&[]);
                    self.reactions.iter().any(|r| {
                        last.contains(&r.event.as_str())
                            && r.day >= open.landed_day - 0.01
                            && r.noticed()
                    })
                }
            };
            if ready {
                next.extend_from_slice(step.events);
            }
        }

        next
    }

    /// The first steps of the chains not yet begun in this stay: the menu weighs them up too,
    /// so a chain still begins as the catalogue grows (its first step was one among a hundred
    /// and more, and a chain often never opened).
    pub fn chain_starts(&self) -> Vec<&'static str> {
        CHAINS
            .iter()
            .filter(|c| {
                !self.chains.iter().any(|o| o.id == c.id)
                    && !self.chains_done.iter().any(|d| d == c.id)
            })
            .flat_map(|c| {
                c.steps
                    .first()
                    .map(|s| s.events)
                    .unwrap_or(&[])
                    .iter()
                    .copied()
            })
            .collect()
    }

    /// The situation's line for each open chain: what it began, and what may come.
    pub fn chain_words(&self, day: f32) -> Vec<String> {
        fn words(id: &str) -> String {
            crate::events::find(id)
                .map(|e| e.menu_words().to_string())
                .unwrap_or_else(|| id.to_string())
        }
        self.chains
            .iter()
            .filter_map(|open| {
                let chain = find(&open.id)?;
                let did = self.done.iter().rev().find(|d| {
                    d.ok && open
                        .landed
                        .iter()
                        .any(|i| chain.steps[*i].events.contains(&d.event.as_str()))
                })?;
                let next = chain.steps.get(open.next(chain)?)?;
                Some(format!(
                    "It has begun something: it chose to {} {}; next it may {}.",
                    words(&did.event),
                    crate::memory::ago_words(day - did.day, "today").to_lowercase(),
                    words(next.events[0])
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_step_is_an_event_of_the_catalogue() {
        for chain in CHAINS {
            for step in chain.steps {
                for event in step.events {
                    assert!(
                        crate::events::find(event).is_some(),
                        "{}: {event}",
                        chain.id
                    );
                }
            }
        }
        // Five the code names (the thing, the place, the watcher, the house, the diary),
        // seventy drawn again over the whole catalogue (owner: "make 75 total chains"), and the
        // branches down the triggers' other ways (owner: "fork/branches yes").
        assert_eq!(CHAINS.len(), 189);
        // No two chains alike (owner: "no duplicate chains, but an event that makes sense to be a
        // step in a chain if it already exists in another one is fine"): an event may be in
        // several, never two chains of the same events.
        let mut seen: Vec<Vec<&str>> = Vec::new();
        for chain in CHAINS {
            let mut events: Vec<&str> = chain
                .steps
                .iter()
                .flat_map(|s| s.events.iter().copied())
                .collect();
            events.sort_unstable();
            assert!(
                !seen.contains(&events),
                "{} is another chain again",
                chain.id
            );
            seen.push(events);
        }
        let ids: std::collections::HashSet<&str> = CHAINS.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), CHAINS.len(), "each chain its own id");
    }

    #[test]
    fn the_scenes_are_the_short_beats_and_a_step_of_one_is_noted() {
        let scenes: Vec<&str> = CHAINS.iter().filter(|c| c.scene).map(|c| c.id).collect();
        assert_eq!(scenes.len(), 37, "{scenes:?}");
        assert!(CHAINS
            .iter()
            .filter(|c| c.scene)
            .all(|c| c.steps.len() >= 3));
        assert!(
            !find("the_house").unwrap().scene,
            "the slow ones at their pace"
        );
        let mut memory = Memory::default();
        memory.chain_landed("knock", 1.0);
        assert_eq!(memory.scene_moved, vec!["the_knock"], "opened, noted");
        assert_eq!(memory.chain_next_of("the_knock"), vec!["door_scratch"]);
        memory.scene_moved.clear();
        memory.chain_landed("door_scratch", 1.0);
        assert_eq!(memory.scene_moved, vec!["the_knock"], "moved, noted again");
        memory.scene_moved.clear();
        memory.chain_landed("take", 1.0);
        assert!(
            memory.scene_moved.is_empty(),
            "a chain at its own pace is not a scene"
        );
    }

    #[test]
    fn a_trigger_opens_one_of_its_branches_and_another_when_it_comes_again() {
        static BRANCHES: &[Chain] = &[
            Chain {
                id: "branch_a",
                steps: &[
                    step(&["knock"], 0.0, Always),
                    step(&["door_scratch"], 3.0, Always),
                ],
                scene: false,
            },
            Chain {
                id: "branch_b",
                steps: &[
                    step(&["knock"], 0.0, Always),
                    step(&["knock_heavy"], 3.0, Always),
                ],
                scene: false,
            },
        ];
        let mut memory = Memory::default();
        memory.open_branch("knock", 1.0, BRANCHES);
        assert_eq!(memory.chains.len(), 1, "one branch, not both");
        let first = memory.chains[0].id.clone();
        memory.open_branch("knock", 1.5, BRANCHES);
        assert_eq!(memory.chains.len(), 1, "none while one is open");
        memory.chains.clear();
        memory.chains_done.push(first.clone());
        memory.open_branch("knock", 2.0, BRANCHES);
        assert_eq!(memory.chains.len(), 1);
        assert_ne!(
            memory.chains[0].id, first,
            "the other, the trigger come again"
        );
        // Over many stays the trigger goes down both.
        let picked: std::collections::HashSet<String> = (0..20)
            .map(|d| {
                let mut m = Memory::default();
                m.open_branch("knock", d as f32 * 0.37, BRANCHES);
                m.chains[0].id.clone()
            })
            .collect();
        assert_eq!(picked.len(), 2, "{picked:?}");
    }

    #[test]
    fn a_chain_opens_goes_on_and_ends_once_a_stay() {
        let mut memory = Memory::default();
        memory.chain_landed("take", 1.0);
        assert_eq!(memory.chains.len(), 1);
        assert_eq!(
            memory.chain_next(),
            vec![
                "place_taken",
                "return_taken",
                "sand_gives_back",
                "pot_of_returns"
            ]
        );

        memory.chain_landed("return_taken", 2.0);
        assert_eq!(memory.chain_next(), vec!["at_the_chest"]);
        memory.chain_landed("at_the_chest", 3.0);
        memory.chain_landed("frame_emptied", 4.0);
        assert!(memory.chains.is_empty());
        assert_eq!(memory.chains_done, vec!["the_thing".to_string()]);

        memory.chain_landed("take", 6.0);
        assert!(memory.chains.is_empty(), "once a stay");
    }

    #[test]
    fn a_step_waiting_for_its_stage_still_sees_what_was_noticed_before() {
        let mut memory = Memory::default();
        memory.chain_landed("open_door", 1.0);
        memory.reactions.push(crate::memory::Reaction {
            event: "open_door".to_string(),
            day: 1.0,
            kind: "watched".to_string(),
            ..Default::default()
        });
        // door_hinge from stage three: through stages one and two the chain waits.
        memory.chains_tick(3.0, 1);
        memory.chains_tick(6.0, 2);
        assert_eq!(memory.chains.len(), 1, "it waited for its stage");
        assert_eq!(
            memory.chain_next(),
            vec!["door_hinge", "false_door", "trapdoor_to_nothing"],
            "the noticing before still counts"
        );
    }

    #[test]
    fn a_chase_before_the_watcher_began_does_not_end_it() {
        let mut memory = Memory::default();
        memory.figure_looks.push(crate::memory::FigureLook {
            day: 5.0,
            toward: 12.0,
            end: "approached".to_string(),
            ..Default::default()
        });
        memory.chain_landed("far_figure", 5.2);
        memory.chains_tick(5.3, 3);
        assert_eq!(memory.chains.len(), 1);
    }

    #[test]
    fn a_step_whose_moment_passed_falls_out_and_the_watcher_waits_for_a_long_look() {
        let mut memory = Memory::default();
        memory.chain_landed("ore_marked", 1.0);
        memory.chains_tick(3.5, 3);
        assert_eq!(memory.chains.len(), 1);
        memory.chains_tick(4.5, 3);
        assert!(
            memory.chains.is_empty(),
            "three days and no mined_out: it fell out"
        );

        memory.chain_landed("far_figure", 5.0);
        assert!(
            memory.chain_next().is_empty(),
            "behind_you waits for a long look"
        );
        memory.figure_looks.push(crate::memory::FigureLook {
            day: 5.2,
            looked_ticks: 60,
            looks: 1,
            ..Default::default()
        });
        assert_eq!(memory.chain_next(), vec!["behind_you"]);

        memory.figure_looks.push(crate::memory::FigureLook {
            day: 5.4,
            toward: 12.0,
            end: "approached".to_string(),
            ..Default::default()
        });
        memory.chains_tick(5.5, 3);
        assert!(memory.chains.is_empty(), "chased: the watcher ends");
    }
}
