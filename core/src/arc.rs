//! The stages: when the lodger moves in, when a stage ends, and the day's budget of events. The
//! endings' three-day scripts are a later item; today stage five ends the stay quietly.

use crate::config::Config;
use crate::memory::Memory;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Arc {
    /// 0 before it moves in, 1 to 5 during the stay, 6 when it is over.
    pub stage: u8,
    /// The day the current stage began.
    pub stage_began_day: f32,
    /// Events landed in this stage.
    pub landed_this_stage: u32,
    /// What the day's budget has spent, and which day that was.
    pub spent_today: f32,
    pub budget_day: u64,
}

impl Arc {
    pub fn over(&self) -> bool {
        self.stage > 5
    }

    pub fn moved_in(&self) -> bool {
        (1..=5).contains(&self.stage)
    }

    /// Advances the stage when its time has come. Answers the new stage when it changed.
    pub fn tick(&mut self, day: f32, memory: &Memory, config: &Config) -> Option<u8> {
        if self.stage == 0 {
            if day >= config.stages.start_after_days as f32 {
                self.stage = 1;
                self.stage_began_day = day;
                self.landed_this_stage = 0;

                return Some(1);
            }

            return None;
        }

        if self.over() {
            return None;
        }

        let index = (self.stage - 1) as usize;
        let in_stage = day - self.stage_began_day;
        let least = config.stages.least_days[index] as f32;
        let most = config.stages.most_days[index] as f32;

        let condition_met = match self.stage {
            1 => {
                memory.home.is_some()
                    && memory.used_container().is_some()
                    && self.landed_this_stage >= 2
            }
            2 => self.landed_this_stage >= 6,
            3 => self.landed_this_stage >= 10,
            4 => false,
            _ => false,
        };

        if in_stage >= least && (condition_met || in_stage >= most) {
            self.stage += 1;
            self.stage_began_day = day;
            self.landed_this_stage = 0;

            return Some(self.stage);
        }

        None
    }

    /// The day's budget of events, from the config's stage table; 0 before move-in and after.
    pub fn budget(&self, config: &Config) -> f32 {
        if !self.moved_in() {
            return 0.0;
        }

        config.stages.events_per_day[(self.stage - 1) as usize] as f32
    }

    /// Whether the day has room for an event of this cost; resets when the day turns.
    pub fn affords(&mut self, cost: f32, day: u64, config: &Config) -> bool {
        if day != self.budget_day {
            self.budget_day = day;
            self.spent_today = 0.0;
        }

        self.spent_today + cost <= self.budget(config) + 1e-6
    }

    pub fn spend(&mut self, cost: f32) {
        self.spent_today += cost;
        self.landed_this_stage += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_moves_in_after_the_configured_days_and_leaves_stage_one_on_its_condition() {
        let config = Config::default();
        let mut arc = Arc::default();
        let mut memory = Memory::default();

        assert_eq!(arc.tick(2.9, &memory, &config), None);
        assert_eq!(arc.tick(3.0, &memory, &config), Some(1));
        assert!(arc.moved_in());

        // Nothing known yet: the stage waits for its most days.
        assert_eq!(arc.tick(4.5, &memory, &config), None);
        memory.home = Some(crate::memory::Home::default());
        memory.containers.push(crate::memory::Container {
            pos: [0, 0, 0],
            opens: 3,
            placed: true,
            ..Default::default()
        });
        arc.landed_this_stage = 2;
        assert_eq!(
            arc.tick(3.5, &memory, &config),
            None,
            "under the least days"
        );
        assert_eq!(arc.tick(4.0, &memory, &config), Some(2));
    }

    #[test]
    fn a_stage_ends_at_its_most_days_whatever_happened() {
        let config = Config::default();
        let mut arc = Arc {
            stage: 2,
            stage_began_day: 10.0,
            ..Arc::default()
        };
        let memory = Memory::default();

        assert_eq!(arc.tick(14.9, &memory, &config), None);
        assert_eq!(arc.tick(15.0, &memory, &config), Some(3));
    }

    #[test]
    fn the_budget_is_the_stage_s_and_resets_when_the_day_turns() {
        let config = Config::default();
        let mut arc = Arc {
            stage: 2,
            ..Arc::default()
        };

        assert_eq!(arc.budget(&config), 2.0);
        assert!(arc.affords(1.0, 5, &config));
        arc.spend(1.0);
        assert!(arc.affords(1.0, 5, &config));
        arc.spend(1.0);
        assert!(!arc.affords(0.5, 5, &config));
        assert!(arc.affords(0.5, 6, &config), "a new day");

        let mut before = Arc::default();
        assert_eq!(before.budget(&config), 0.0);
        assert!(!before.affords(0.5, 0, &config));
    }
}
