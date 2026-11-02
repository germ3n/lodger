//! The hard rules, checked after every decision: nothing in the config, the memory or the
//! situation text reaches these. See `docs/good-to-know.md`, What it never does. Most of the
//! rules are about what the catalogue offers at all; the ones that need the moment are here.

use crate::config::Config;
use crate::events::Event;
use crate::memory::Memory;
use craftkit::world::BlockPos;

/// Why an event may not run now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    InView,
    Disabled,
    SameAsLast,
    Cooldown,
    Capped,
    NotOffered,
}

impl Refused {
    pub fn words(&self) -> &'static str {
        match self {
            Refused::InView => "the spot is in view",
            Refused::Disabled => "disabled in the config",
            Refused::SameAsLast => "the same as the last event",
            Refused::Cooldown => "within its cooldown",
            Refused::Capped => "at its cap for the stay",
            Refused::NotOffered => "not offered in this stage",
        }
    }
}

/// Whether `event` may run at `pos` now.
pub fn check(
    event: &Event,
    pos: Option<BlockPos>,
    memory: &Memory,
    stage: u8,
    day: f32,
    config: &Config,
) -> Result<(), Refused> {
    if event.id == "wait" {
        return Ok(());
    }

    if !event.offered_in(stage) {
        return Err(Refused::NotOffered);
    }

    if config.events.disabled.iter().any(|d| d == event.id) {
        return Err(Refused::Disabled);
    }

    // What the target starts (a line typed, a chest opened, fresh loot found) is paced by its
    // own cap and chance: two replies in a conversation are not the lodger repeating itself.
    let answers = matches!(
        event.action,
        crate::events::Action::Answers
            | crate::events::Action::ChestAnswers
            | crate::events::Action::LootGift
            | crate::events::Action::DoorHeld
            | crate::events::Action::CountUp
    );
    if let Some(last) = memory.recent(1).next() {
        if last.event == event.id && last.ok && !answers {
            return Err(Refused::SameAsLast);
        }
    }

    let cooldown = event.cooldown_days * config.events.cooldown_scale as f32;
    let times = memory.times(event.id);
    for done in &memory.done {
        if done.event != event.id {
            continue;
        }

        if day - done.day < cooldown && (done.ok || done.pos == pos) {
            return Err(Refused::Cooldown);
        }
    }

    if times >= event.most_per_stay as usize {
        return Err(Refused::Capped);
    }

    let exempt = !hidden_from_view(event.action);

    if let Some(pos) = pos {
        if !exempt && memory.in_view(pos, config.events.min_distance as f64) {
            return Err(Refused::InView);
        }
    }

    Ok(())
}

/// Whether an event of this action is kept out of the target's sight: everything that changes
/// something they could see. The sounds, the glitches and the rest exempted are about being
/// near (`docs/entity.md`, The distance rule).
pub fn hidden_from_view(action: crate::events::Action) -> bool {
    !matches!(
        action,
        crate::events::Action::Sound(_)
            | crate::events::Action::SoundAtTarget(_)
            | crate::events::Action::SoundFar(_)
            | crate::events::Action::SoundAtFixture(_, _)
            | crate::events::Action::Taps(_, _, _)
            | crate::events::Action::OpenLid
            | crate::events::Action::Hint(_, _)
            | crate::events::Action::RefuseRest
            | crate::events::Action::Wake
            | crate::events::Action::Stare
            | crate::events::Action::StandWatch
            | crate::events::Action::PetGrowl
            | crate::events::Action::TabEntry
            | crate::events::Action::Message(_)
            | crate::events::Action::Says
            | crate::events::Action::SaysKills
            | crate::events::Action::Whisper
            | crate::events::Action::RepeatsYou
            | crate::events::Action::Answers
            | crate::events::Action::HostilesStare
            | crate::events::Action::ChestAnswers
            | crate::events::Action::MoveCreature(_, crate::events::Where::OutsideDoor)
            | crate::events::Action::MiningSounds
            | crate::events::Action::PushVehicle(_)
            | crate::events::Action::DoorAhead
            | crate::events::Action::OpenOnEnter
            | crate::events::Action::SoundPair(_, _, _)
            | crate::events::Action::BedOccupied
            | crate::events::Action::WokeElsewhere
            | crate::events::Action::ScanOre
            | crate::events::Action::Figure(_, _, _)
            | crate::events::Action::BedBreaks
            | crate::events::Action::ScanVillage
            | crate::events::Action::TunnelToward
            | crate::events::Action::TorchesFarEnd
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::find;

    fn memory_facing_z() -> Memory {
        let mut memory = Memory::default();
        memory.observe(
            &craftkit::world::Observations {
                players: vec![craftkit::world::Player {
                    index: 0,
                    pos: [0.0, 64.0, 0.0],
                    ..Default::default()
                }],
                ..Default::default()
            },
            0.0,
        );
        memory
    }

    #[test]
    fn a_spot_in_view_refuses_a_block_event_but_not_a_sound() {
        let config = Config::default();
        let memory = memory_facing_z();

        assert_eq!(
            check(
                find("open_door").unwrap(),
                Some([0, 64, 10]),
                &memory,
                1,
                0.0,
                &config
            ),
            Err(Refused::InView)
        );
        assert_eq!(
            check(
                find("open_door").unwrap(),
                Some([0, 64, -10]),
                &memory,
                1,
                0.0,
                &config
            ),
            Ok(())
        );
        assert_eq!(
            check(
                find("door_sound").unwrap(),
                Some([0, 64, 10]),
                &memory,
                1,
                0.0,
                &config
            ),
            Ok(())
        );
    }

    #[test]
    fn the_same_event_twice_in_a_row_a_cooldown_and_a_cap_refuse_by_name() {
        let config = Config::default();
        let mut memory = memory_facing_z();
        let door = find("open_door").unwrap();

        memory.record("open_door", Some([0, 64, -10]), 1.0, true, "");
        assert_eq!(
            check(door, Some([0, 64, -10]), &memory, 1, 1.5, &config),
            Err(Refused::SameAsLast)
        );

        memory.record("wait", None, 1.6, true, "");
        assert_eq!(
            check(door, Some([0, 64, -10]), &memory, 1, 1.7, &config),
            Err(Refused::Cooldown)
        );
        assert_eq!(
            check(door, Some([0, 64, -10]), &memory, 1, 3.5, &config),
            Ok(())
        );

        for day in 0..door.most_per_stay {
            memory.record(
                "open_door",
                Some([0, 64, -10]),
                10.0 + day as f32 * 3.0,
                true,
                "",
            );
            memory.record("wait", None, 10.5 + day as f32 * 3.0, true, "");
        }
        assert_eq!(
            check(door, Some([0, 64, -10]), &memory, 1, 40.0, &config),
            Err(Refused::Capped)
        );
    }

    #[test]
    fn disabled_events_and_other_stages_events_are_refused() {
        let mut config = Config::default();
        config.events.disabled.push("knock".to_string());
        let memory = memory_facing_z();

        assert_eq!(
            check(find("knock").unwrap(), None, &memory, 2, 0.0, &config),
            Err(Refused::Disabled)
        );
        assert_eq!(
            check(find("take").unwrap(), None, &memory, 1, 0.0, &config),
            Err(Refused::NotOffered)
        );
        assert_eq!(
            check(find("wait").unwrap(), None, &memory, 0, 0.0, &config),
            Ok(())
        );
    }
}
