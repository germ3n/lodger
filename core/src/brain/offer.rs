//! The menu: the catalogue narrowed to what fits the stage, the memory, the moment and the
//! history, then a weighted draw of a few, so the model (or the rule brain) chooses among eight
//! that could all happen now. See `docs/entity.md`, What is offered when.

use crate::config::Config;
use crate::events::{Event, Need};
use crate::memory::Memory;
use crate::model::Rng;
use craftkit::world::Now;

/// Whether the memory and the moment meet a need.
pub fn meets(need: Need, memory: &Memory, now: &Now) -> bool {
    let last = &memory.last;
    let home_pos = memory.home.as_ref().map(|h| h.center);
    let away = |pos| memory.target_away_from(pos, 24.0);
    let night = now.day_time >= 13_000 && now.day_time <= 23_000;

    match need {
        Need::Home => memory.home.is_some(),
        Need::UsedContainer => memory.used_container().is_some(),
        Need::TwoContainers => memory.other_container().is_some(),
        Need::Door => memory
            .home
            .as_ref()
            .map(|h| !h.doors.is_empty())
            .unwrap_or(false),
        Need::Light => memory
            .home
            .as_ref()
            .map(|h| !h.lights.is_empty())
            .unwrap_or(false),
        Need::Candle => memory.has_fixture("candle"),
        Need::Bed => memory
            .home
            .as_ref()
            .map(|h| h.bed.is_some())
            .unwrap_or(false),
        Need::Held => memory.held.is_some(),
        Need::FavouriteInContainer => memory.favourite_in_container().is_some(),
        Need::InChest(suffix) => memory
            .in_used_container(|id| id.ends_with(suffix))
            .is_some(),
        Need::Furnace => memory.furnace().is_some(),
        Need::Smeltable => {
            memory.furnace().is_some()
                && memory
                    .in_used_container(crate::brain::act::smeltable)
                    .is_some()
                && memory.in_used_container(crate::brain::act::fuel).is_some()
        }
        Need::FurnaceOutput => memory
            .furnace()
            .map(|f| f.contents.iter().any(|s| s.slot == 2))
            .unwrap_or(false),
        Need::Fixture(kind) => memory.has_fixture(kind),
        Need::Fixtures(kind, n) => memory.fixtures(kind).count() >= n,
        Need::Decor(kind) => memory.decor(kind).next().is_some(),
        Need::Decors(kind, n) => memory.decor(kind).count() >= n,
        Need::Creature(kind) => memory.creatures(kind).next().is_some(),
        Need::Pet => memory.pets().next().is_some(),
        Need::TargetAwayOrAsleep => home_pos.map(away).unwrap_or(!last.online || last.sleeping),
        Need::TargetOnline => last.online,
        Need::TargetIndoors => last.online && last.indoors,
        Need::TargetOutdoors => last.online && !last.indoors && !last.sleeping,
        Need::TargetWalking => last.online && last.walking,
        Need::TargetInBed => last.online && last.sleeping,
        Need::TargetSaid => !memory.last_said.is_empty(),
        // `answers_you` starts from the chat trigger (`Brain::consider_reply`), never the menu.
        Need::Heard => false,
        Need::Night => night,
        Need::Dawn => now.day_time < 1_500 || now.day_time > 23_000,
        Need::Wall => !memory.walls.is_empty(),
        Need::PlacedLight => memory
            .walls
            .iter()
            .any(|(_, id)| id.ends_with("torch") || id.ends_with("lantern")),
        Need::Pane => memory
            .walls
            .iter()
            .any(|(_, id)| id.ends_with("glass_pane")),
        Need::Day => !night,
        Need::Raining => last.raining,
        Need::NotRaining => !last.raining,
        Need::Route => memory.route().is_some(),
        Need::Field => false,
        Need::Animals => memory.creatures("").count() >= 3,
        Need::Hostiles => memory.last.hostiles_near.len() >= 3,
        Need::Kills => memory.most_killed().is_some(),
        Need::Around(kind) => memory.feature(kind).is_some(),
        Need::Vehicle => !memory.vehicles.is_empty(),
        Need::TargetUnderground => last.online && last.underground,
        Need::TargetAte => {
            last.online && last.ate_day > 0.0 && (memory.today - last.ate_day) < 0.002
        }
        Need::TargetApproachingDoor => memory
            .home
            .as_ref()
            .map(|h| h.doors.iter().any(|d| memory.approaching(*d, 6.0)))
            .unwrap_or(false),
        Need::Scanned(kind) => memory.scanned(kind).next().is_some(),
        Need::InChestAny(suffixes) => memory
            .in_used_container(|id| suffixes.iter().any(|s| id.ends_with(s)))
            .is_some(),
        Need::TargetApproachingChest => memory
            .used_container()
            .map(|c| memory.approaching(c.pos, 6.0))
            .unwrap_or(false),
        Need::ColouredStacks => memory.coloured_stacks() >= 6,
        Need::ToolInContainer => memory.tool_in_container().is_some(),
        Need::Water => false,
    }
}

/// Whether the event's group is on in the config: `events.sounds`, `events.glitches`,
/// `events.chat`. An event in a group that is off is never offered or forced.
pub fn switched_on(event: &Event, config: &Config) -> bool {
    use crate::events::Action;
    match event.action {
        Action::Sound(_)
        | Action::SoundAtTarget(_)
        | Action::SoundFar(_)
        | Action::SoundAtFixture(_, _)
        | Action::SoundPair(_, _, _)
        | Action::Taps(_, _, _)
        | Action::MiningSounds
        | Action::PetGrowl => config.events.sounds,
        Action::Hint(_, _) => config.events.glitches,
        Action::Figure(_, _, _) => config.events.figure,
        Action::TabEntry
        | Action::Message(_)
        | Action::Says
        | Action::SaysKills
        | Action::Whisper
        | Action::RepeatsYou
        | Action::Answers => config.events.chat,
        _ => true,
    }
}

/// The group an event belongs to, of which one plan at a time: the figures, the chat lines,
/// the screen's glitches, the sounds. None for the rest, which are kept apart by spot and id.
pub fn group(event: &Event) -> Option<&'static str> {
    use crate::events::Action;
    match event.action {
        Action::Sound(_)
        | Action::SoundAtTarget(_)
        | Action::SoundFar(_)
        | Action::SoundAtFixture(_, _)
        | Action::SoundPair(_, _, _)
        | Action::Taps(_, _, _)
        | Action::MiningSounds
        | Action::PetGrowl => Some("sound"),
        Action::Hint(_, _) => Some("screen"),
        Action::Figure(_, _, _) => Some("figure"),
        Action::TabEntry
        | Action::Message(_)
        | Action::Says
        | Action::SaysKills
        | Action::Whisper
        | Action::RepeatsYou
        | Action::Answers => Some("chat"),
        _ => None,
    }
}

/// The first need an event lacks right now, in words, or none.
pub fn unmet(event: &Event, memory: &Memory, now: &Now) -> Option<String> {
    event
        .needs
        .iter()
        .find(|need| !meets(**need, memory, now))
        .map(|need| need.words())
}

pub fn offer<'a>(
    catalogue: &'a [Event],
    stage: u8,
    memory: &Memory,
    now: &Now,
    day: f32,
    config: &Config,
    rng: &mut Rng,
) -> Vec<&'a Event> {
    let mut eligible: Vec<(&Event, f64)> = Vec::new();

    for event in catalogue {
        if event.id == "wait" || !event.offered_in(stage) {
            continue;
        }

        if config.events.disabled.iter().any(|d| d == event.id) {
            continue;
        }

        if !switched_on(event, config) {
            continue;
        }

        if !event.needs.iter().all(|need| meets(*need, memory, now)) {
            continue;
        }

        if crate::rules::check(event, None, memory, stage, day, config).is_err() {
            continue;
        }

        let times = memory.times(event.id);
        let weight = 1.0 / (1.0 + times as f64);
        eligible.push((event, weight));
    }

    let mut menu: Vec<&Event> = Vec::new();
    if let Some(wait) = catalogue.iter().find(|e| e.id == "wait") {
        menu.push(wait);
    }

    let want = (config.events.offered as usize).saturating_sub(1);
    while menu.len() < want + 1 && !eligible.is_empty() {
        let total: f64 = eligible.iter().map(|(_, w)| *w).sum();
        let mut draw = rng.next_f64() * total;
        let mut picked = eligible.len() - 1;
        for (index, (_, weight)) in eligible.iter().enumerate() {
            if draw < *weight {
                picked = index;
                break;
            }
            draw -= *weight;
        }
        menu.push(eligible.remove(picked).0);
    }

    menu
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::CATALOGUE;
    use craftkit::world::{Observations, Player};

    fn memory_at_home() -> Memory {
        let mut memory = Memory::default();
        memory.observe(
            &Observations {
                players: vec![Player {
                    index: 0,
                    pos: [0.0, 64.0, 0.0],
                    sky_light: 0,
                    ..Default::default()
                }],
                ..Default::default()
            },
            0.0,
        );
        memory.home = Some(crate::memory::Home {
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![[1, 65, 1]],
        });
        memory
    }

    #[test]
    fn stage_one_offers_only_small_things_and_wait_comes_first() {
        let config = Config::default();
        let memory = memory_at_home();
        let mut rng = Rng::new(1);
        let menu = offer(
            CATALOGUE,
            1,
            &memory,
            &Now::default(),
            0.0,
            &config,
            &mut rng,
        );

        assert_eq!(menu[0].id, "wait");
        assert!(menu.len() <= config.events.offered as usize);
        for event in &menu {
            assert!(event.small, "{} offered in stage one", event.id);
        }
    }

    #[test]
    fn needs_narrow_the_menu_to_the_moment() {
        let config = Config::default();
        let mut memory = memory_at_home();
        let mut rng = Rng::new(2);
        let now = Now {
            day_time: 6000,
            ..Now::default()
        };

        // The target is at home and awake: nothing that needs them away is offered.
        let menu = offer(CATALOGUE, 2, &memory, &now, 0.0, &config, &mut rng);
        assert!(
            menu.iter()
                .all(|e| e.id != "open_door" && e.id != "shuffle_chest"),
            "{:?}",
            menu.iter().map(|e| e.id).collect::<Vec<_>>()
        );

        // The target offline: the indoor sounds go, the away things come.
        memory.last.online = false;
        let menu: Vec<&str> = offer(CATALOGUE, 2, &memory, &now, 0.0, &config, &mut rng)
            .iter()
            .map(|e| e.id)
            .collect();
        assert!(!menu.contains(&"door_sound"), "{menu:?}");
        assert!(
            menu.contains(&"open_door") || menu.contains(&"move_light"),
            "{menu:?}"
        );
    }

    #[test]
    fn a_hundred_draws_offer_every_eligible_event_at_least_once() {
        let config = Config::default();
        let mut memory = memory_at_home();
        memory.last.online = false;
        let mut rng = Rng::new(3);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            for event in offer(
                CATALOGUE,
                2,
                &memory,
                &Now::default(),
                0.0,
                &config,
                &mut rng,
            ) {
                seen.insert(event.id);
            }
        }

        for event in CATALOGUE {
            if event.offered_in(2)
                && event
                    .needs
                    .iter()
                    .all(|n| meets(*n, &memory, &Now::default()))
            {
                assert!(seen.contains(event.id), "{} never offered", event.id);
            }
        }
    }
}
