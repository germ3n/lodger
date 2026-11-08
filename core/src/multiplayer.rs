//! More than one player (`docs/entity.md`, Multiplayer): one lodger, one target, everyone else a
//! witness. The target is picked at move-in (`multiplayer.pick`, in the brain); here, the target
//! changed once a stay when they have been gone too long (`multiplayer.retarget`,
//! `multiplayer.absent_days`), and the world's changes held back near a witness when
//! `multiplayer.witness_traces` is off. Sounds, figures, the screen and the chat go to the target
//! alone whatever this says: every such command names its player.

use crate::config::Config;
use crate::memory::{distance, Memory};
use craftkit::world::BlockPos;

/// How near a witness holds a change to the world back, with traces off.
pub const WITNESS_BLOCKS: f64 = 48.0;

/// Seconds since the Unix epoch, by the computer's clock: absence is counted in real days.
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Memory {
    /// Whether the target has been gone long enough, by the real clock, to be given up: the
    /// config allows it, it has not happened this stay, from stage two, and someone else is here.
    pub fn retarget_due(&self, config: &Config, stage: u8, now: u64) -> bool {
        config.multiplayer.retarget
            && !self.retargeted
            && stage >= 2
            && self.target.is_some()
            && !self.last.online
            && self.target_seen_unix > 0
            && now.saturating_sub(self.target_seen_unix)
                >= config.multiplayer.absent_days as u64 * 86_400
            && !self.others_online.is_empty()
    }

    /// The new target, the first other here of those it knows longest, with what was the old
    /// target's let go (their home, their chests, their habits and favourites, how they took
    /// things, how they looked at the figure) and what is its own kept (its want, its place, its
    /// house, lights and store, what it did, its temper and chains). Answers their name.
    pub fn retarget(&mut self, now: u64) -> Option<String> {
        // Another player by uuid: one at the old target's index is someone else, and is a
        // candidate (by index alone it was skipped, and the change never came).
        let old = self.target_uuid.clone();
        let (index, name, uuid) = self
            .others
            .iter()
            .find(|(i, _, u)| self.others_online.contains(i) && (old.is_empty() || *u != old))
            .cloned()?;
        self.set_target(index, &name, &uuid);
        self.home = None;
        self.containers.clear();
        self.favourites.clear();
        self.habits = Default::default();
        self.reactions.clear();
        self.figure_looks.clear();
        self.tally.clear();
        self.chat.clear();
        self.routes.clear();
        self.trail.clear();
        self.last = Default::default();
        // Where the old target stood still (the home was read again from it), when they were
        // last elsewhere, and who visited their house: theirs.
        self.still_cells.clear();
        self.elsewhere_since = None;
        self.back_after = None;
        self.people = Default::default();
        self.target_seen_unix = now;
        self.retargeted = true;

        Some(name)
    }

    /// With witness traces off: whether a change to the world at `pos` would be seen near
    /// another player.
    pub fn witness_near(&self, config: &Config, pos: Option<BlockPos>) -> bool {
        // With no place of its own, where it would be: by the house, or by the target.
        let pos = pos
            .or(self.home.as_ref().map(|h| h.center))
            .or(Some(crate::memory::block_of(self.last.pos)));
        !config.multiplayer.witness_traces
            && pos.is_some_and(|p| {
                self.others_at
                    .iter()
                    .any(|o| distance(*o, p) <= WITNESS_BLOCKS)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_players() -> Memory {
        let mut memory = Memory {
            others: vec![
                (0, "razor".to_string(), "u0".to_string()),
                (1, "friend".to_string(), "u1".to_string()),
            ],
            target_seen_unix: 1_000_000,
            others_online: vec![1],
            want: "a place of its own".to_string(),
            place: Some([40, 64, 40]),
            home: Some(crate::memory::Home::default()),
            ..Memory::default()
        };
        memory.set_target(0, "razor", "u0");
        memory
    }

    #[test]
    fn a_target_gone_seven_real_days_is_given_up_once_for_someone_here() {
        let config = Config::default();
        let mut memory = two_players();
        let week = 7 * 86_400;

        assert!(
            !memory.retarget_due(&config, 2, 1_000_000 + week - 1),
            "a day short"
        );
        assert!(
            !memory.retarget_due(&config, 1, 1_000_000 + week),
            "not in stage one"
        );
        assert!(memory.retarget_due(&config, 2, 1_000_000 + week));

        assert_eq!(memory.retarget(1_000_000 + week).as_deref(), Some("friend"));
        assert_eq!(memory.target, Some(1));
        assert_eq!(memory.want, "a place of its own", "the want is kept");
        assert_eq!(memory.place, Some([40, 64, 40]), "its place is kept");
        assert!(memory.home.is_none(), "the old target's home is let go");
        assert!(
            !memory.retarget_due(&config, 3, 1_000_000 + 3 * week),
            "once a stay"
        );
    }

    #[test]
    fn with_witness_traces_off_a_change_near_another_player_waits() {
        let mut config = Config::default();
        let mut memory = two_players();
        memory.others_at = vec![[0, 64, 0]];

        assert!(
            !memory.witness_near(&config, Some([10, 64, 0])),
            "traces on: no matter"
        );
        config.multiplayer.witness_traces = false;
        assert!(memory.witness_near(&config, Some([10, 64, 0])));
        assert!(!memory.witness_near(&config, Some([100, 64, 0])));
    }
}
