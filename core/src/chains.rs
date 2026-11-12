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
}

const fn step(events: &'static [&'static str], within_days: f32, when: When) -> Step {
    Step {
        events,
        within_days,
        when,
    }
}

use When::*;

/// The nine chains of the design, with the catalogue's events, and seven of the newer events'
/// (owner: "remember the chain functionality, so chain events, where possible").
pub const CHAINS: &[Chain] = &[
    Chain {
        id: "the_thing",
        steps: &[
            step(&["take"], 0.0, Always),
            step(&["place_taken", "return_taken"], 3.0, Always),
            step(&["at_the_chest"], 3.0, Always),
            step(&["frame_emptied"], 3.0, Always),
        ],
    },
    Chain {
        id: "the_vein",
        steps: &[
            step(&["ore_marked"], 0.0, Always),
            step(&["mined_out"], 3.0, Always),
        ],
    },
    Chain {
        id: "the_tunnel",
        steps: &[
            step(&["mining_sounds"], 0.0, Always),
            step(&["mine_beside"], 3.0, Always),
            step(&["tunnel_toward"], 4.0, Always),
            step(&["cave_room"], 4.0, Always),
        ],
    },
    Chain {
        id: "the_place",
        steps: &[
            step(&["light_path"], 0.0, Always),
            step(&["hole"], 3.0, Always),
            step(&["pillar"], 3.0, Always),
            step(&["sign"], 3.0, Always),
            // Its house grows in the ending's script, not as an event: the chain ends at the house.
            step(&["house_near"], 4.0, Always),
        ],
    },
    Chain {
        id: "the_watcher",
        steps: &[
            step(&["far_figure"], 0.0, Always),
            step(&["behind_you"], 3.0, LookedLong),
            step(&["on_the_roof"], 3.0, Always),
            step(&["in_doorway"], 3.0, Always),
            step(&["in_your_bed"], 3.0, Always),
            step(&["copy_crouch"], 3.0, Always),
            step(&["figure_waves"], 3.0, Always),
        ],
    },
    Chain {
        id: "the_copy",
        steps: &[
            step(&["copy_build"], 0.0, Always),
            step(&["copy_house"], 5.0, Always),
            step(&["second_bed"], 4.0, Always),
        ],
    },
    Chain {
        id: "the_house",
        steps: &[
            step(&["open_door"], 0.0, Always),
            step(&["door_hinge"], 3.0, Noticed),
            step(&["door_held"], 3.0, Noticed),
            step(&["window_bars"], 4.0, Noticed),
        ],
    },
    Chain {
        id: "the_diary",
        steps: &[
            step(&["diary_sold"], 0.0, Always),
            step(&["diary_brought"], 8.0, Always),
        ],
    },
    Chain {
        id: "the_pet",
        steps: &[
            step(&["pet_growl"], 0.0, Always),
            step(&["pet_moved"], 3.0, Always),
            step(&["pet_outside"], 3.0, Always),
        ],
    },
    // The woods: it peeks, it comes nearer, it is heard, and as they leave it is all along the
    // edge.
    Chain {
        id: "the_woods",
        steps: &[
            step(&["peeks_from_the_trees"], 0.0, Always),
            step(&["tree_to_tree"], 3.0, Always),
            step(&["branch_snaps", "knocks_on_wood"], 3.0, Always),
            step(&["the_tree_line"], 4.0, Always),
        ],
    },
    // The trees marked: a trail, knocks along it, a stand gone wrong, and the way out shut.
    Chain {
        id: "the_marked_trees",
        steps: &[
            step(&["blazed_trees"], 0.0, Always),
            step(&["bare_trees"], 3.0, Always),
            step(&["trees_close_behind"], 4.0, Always),
        ],
    },
    // The village going wrong, by degrees, to its fleeing them, or its stab.
    Chain {
        id: "the_village_wrong",
        steps: &[
            step(&["trades_refused"], 0.0, Always),
            step(&["villager_spins"], 3.0, Always),
            step(&["villager_collapses"], 3.0, Always),
            step(&["villagers_walk_out"], 3.0, Always),
            step(&["villagers_flee_you", "villager_stabs"], 4.0, Always),
        ],
    },
    // Their things up into the sky, and back out of it.
    Chain {
        id: "the_sky",
        steps: &[
            step(&["taken_up"], 0.0, Always),
            step(&["sky_gives_back"], 3.0, Always),
        ],
    },
    // The night after them: drawn to their light, then nearer each time unseen, the pack, and at
    // the last something in their own gear.
    Chain {
        id: "the_night",
        steps: &[
            step(&["drawn_to_the_light"], 0.0, Always),
            step(&["while_unseen"], 3.0, Always),
            step(&["wolves_circle"], 3.0, Always),
            step(&["zombie_in_your_gear"], 4.0, Always),
        ],
    },
    // The visitor: one that ran in and died, then prints round the house, then the wall gives.
    Chain {
        id: "the_visitor",
        steps: &[
            step(&["villager_flees_in"], 0.0, Always),
            step(&["prints_around_the_house"], 3.0, Always),
            step(&["wall_bursts"], 4.0, Always),
        ],
    },
    // The house's own things wrong: the redstone dead, the sound, the menu.
    Chain {
        id: "the_glitches",
        steps: &[
            step(&["dead_redstone"], 0.0, Always),
            step(&["sound_warp"], 3.0, Noticed),
            step(&["menu_freaks"], 3.0, Always),
        ],
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
        for chain in CHAINS {
            let begun = self.chains.iter().any(|o| o.id == chain.id)
                || self.chains_done.iter().any(|d| d == chain.id);
            if !begun && chain.steps[0].events.contains(&event) {
                self.chains.push(Open {
                    id: chain.id.to_string(),
                    landed: vec![0],
                    since: day,
                    landed_day: day,
                    last: 0,
                });
            }
        }
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

    /// The events an open chain offers next, its condition met: the menu weighs them up.
    pub fn chain_next(&self) -> Vec<&'static str> {
        let mut next = Vec::new();
        for open in &self.chains {
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
        // The design's eight, the diary (sold, then brought if not bought), and the newer
        // events' seven.
        assert_eq!(CHAINS.len(), 16);
        let ids: std::collections::HashSet<&str> = CHAINS.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), CHAINS.len(), "each chain its own id");
    }

    #[test]
    fn a_chain_opens_goes_on_and_ends_once_a_stay() {
        let mut memory = Memory::default();
        memory.chain_landed("take", 1.0);
        assert_eq!(memory.chains.len(), 1);
        assert_eq!(memory.chain_next(), vec!["place_taken", "return_taken"]);

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
            vec!["door_hinge"],
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
