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
        // Their words and their places too: the new target was told the old one's last line,
        // and its signs wrote it.
        self.last_said.clear();
        self.sites.clear();
        self.sessions.clear();
        self.walls.clear();
        self.fixtures.clear();
        self.tunnels.clear();
        self.holes.clear();
        // `placed` is kept: it is what it never digs beside, whoever built it (cleared, the old
        // target's house was open to its digging).
        // What happened to them and what was around them: their death, where they fished, their
        // obsidian, their frames and boats, plants and field, the land around them.
        self.milestones.clear();
        self.last_fished = None;
        self.placed_obsidian.clear();
        self.decor.clear();
        self.vehicles.clear();
        self.plants.clear();
        self.field.clear();
        self.surroundings = None;
        self.elsewhere_since = None;
        self.back_after = None;
        self.people = Default::default();
        // The diary was of the old target: a new one is written for the new.
        self.diary_offered = 0.0;
        self.diary_bought = false;
        self.chains.retain(|o| o.id != "the_diary");
        self.chains_done.retain(|c| c != "the_diary");
        // Its once-a-stay count too, or the new one's was never written.
        let diary = |d: &crate::memory::Done| d.event == "diary_sold" || d.event == "diary_brought";
        self.done.retain(|d| !diary(d));
        self.landmarks.retain(|d| !diary(d));
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
            placed: vec![[3, 64, 3]],
            milestones: vec![crate::memory::Milestone::default()],
            diary_offered: 4.0,
            done: vec![crate::memory::Done {
                event: "diary_sold".to_string(),
                ok: true,
                ..Default::default()
            }],
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
            memory.placed.contains(&[3, 64, 3]),
            "what was built stays where it never digs"
        );
        assert!(
            memory.milestones.is_empty(),
            "their death was not the new one's"
        );
        assert_eq!(memory.diary_offered, 0.0, "the old target's diary too");
        assert_eq!(memory.times("diary_sold"), 0, "and a new one may be sold");
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
