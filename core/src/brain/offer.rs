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
        Need::Candle => memory
            .walls
            .iter()
            .any(|(_, id)| id.contains("candle") && !id.contains("cake")),
        Need::Bed => memory
            .home
            .as_ref()
            .map(|h| h.bed.is_some())
            .unwrap_or(false),
        Need::Held => memory.held.is_some(),
        Need::FavouriteInContainer => memory.favourite_in_container().is_some(),
        Need::Stored(id, n) => memory.stored(id) >= n,
        Need::Plant(kind) => memory.plants.iter().any(|(id, _)| {
            if kind == "flower" {
                crate::memory::is_flower(id)
            } else {
                id.ends_with(kind)
            }
        }),
        Need::Ladder => crate::brain::act::ladder_column(memory).is_some(),
        // `door_held` starts from the door they open (`Brain::consider_held`), never the menu.
        Need::DoorUsed => false,
        Need::Hole => {
            !memory.holes.is_empty() && crate::brain::act::FILL.iter().any(|b| memory.stored(b) > 0)
        }
        Need::InChest(suffix) => memory
            .in_used_container(|id| id.ends_with(suffix))
            .is_some(),
        Need::Furnace => memory.furnace().is_some(),
        // A furnace whose input and output are free: a full input sent their raw thing to the
        // output slot, where it stopped what was smelting.
        Need::Smeltable => {
            memory
                .furnace()
                .is_some_and(|f| !f.contents.iter().any(|s| s.slot == 0 || s.slot == 2))
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
        Need::UnnamedVillager => memory.unnamed_villager_near().is_some(),
        Need::TheirAnimal(kind) => crate::brain::act::their_animal(memory, kind, false).is_some(),
        Need::TheirUnnamedAnimal => crate::brain::act::their_animal(memory, "", true).is_some(),
        Need::Pet => memory.pets().next().is_some(),
        // On sale three days and not bought: never brought while it is still new on the shelf.
        Need::DiaryUnbought => {
            memory.diary_offered > 0.0
                && !memory.diary_bought
                && memory.today - memory.diary_offered >= 3.0
        }
        Need::TargetAwayOrAsleep => home_pos.map(away).unwrap_or(!last.online || last.sleeping),
        Need::TargetOnline => last.online,
        Need::TargetIndoors => last.online && last.indoors,
        Need::TargetOutdoors => last.online && !last.indoors && !last.sleeping,
        Need::TargetWalking => last.online && last.walking,
        Need::TargetInBed => last.online && last.sleeping,
        Need::TargetSaid => !memory.last_said.is_empty(),
        // `answers_you` starts from the chat trigger (`Brain::consider_reply`), never the menu.
        Need::Heard => false,
        // `chest_answers` starts from the opening (`Brain::consider_echo`), never the menu.
        Need::JustOpened => false,
        // `loot_gift` is armed by the brain and given by the host, never from the menu.
        Need::LootOpened => false,
        Need::Night => night,
        Need::Dawn => now.day_time < 1_500 || now.day_time > 23_000,
        // A block of theirs with a family: the lights among them are there for their ids.
        Need::Wall => memory
            .walls
            .iter()
            .any(|(_, id)| !crate::memory::cousins(id).is_empty()),
        // One soul fire can take (`torch_blue`): a torch, a wall torch, a lantern.
        Need::PlacedLight => memory.walls.iter().any(|(_, id)| {
            matches!(
                id.rsplit(':').next().unwrap_or(id),
                "torch" | "wall_torch" | "lantern"
            )
        }),
        Need::Pane => memory
            .walls
            .iter()
            .any(|(_, id)| id.ends_with("glass_pane")),
        Need::Day => !night,
        Need::Raining => last.raining,
        Need::NotRaining => !last.raining,
        Need::Route => memory.route().is_some(),
        Need::Field => !memory.field_rows().is_empty(),
        Need::Animals => memory.creatures("").count() >= 3,
        Need::Hostiles => memory.last.hostiles_near.len() >= 3,
        Need::Kills => memory.most_killed().is_some(),
        Need::Around(kind) => memory.feature(kind).is_some(),
        Need::Vehicle => !memory.vehicles.is_empty(),
        Need::TargetUnderground => last.online && last.underground,
        Need::Cave => memory.unentered_cave().is_some(),
        Need::UnmarkedDeath => memory.unmarked_death().is_some(),
        Need::LastLight => memory.light_taken_to().is_some(),
        Need::LightsBehind => !memory.lights_behind().is_empty(),
        Need::SoulableLight => memory.soulable_light().is_some(),
        Need::WarpableAnimal => last.online && memory.warp_animal().is_some(),
        Need::JustWoke => false,
        Need::CrouchedStill => false,
        Need::Herd => last.online && memory.animals_near_target().len() >= 3,
        Need::Knocked => false,
        // Met in the other dimension's own stay (`crate::elsewhere`), never here.
        Need::InNether | Need::InEnd => false,
        Need::CameBack => false,
        Need::HoldsNothing => memory.held.is_none(),
        Need::LitUnderground => false,
        Need::TargetSwimming => last.online && last.swimming,
        Need::ItsHouse => memory.its_house().is_some(),
        Need::TargetUnderwater => last.online && last.air < 280,
        Need::InTheWoods => memory.in_the_woods(),
        // `drawn_to_the_light` starts from their running lit (`Brain::tick`), never the menu.
        Need::RanLit => false,
        Need::TakenUpDue => memory.taken_up_due(),
        Need::LookingUp => last.online && !last.indoors && !last.underground && last.pitch <= -35.0,
        Need::ItsHouseNear => memory.its_house().is_some_and(|house| {
            let at = [
                last.pos[0].floor() as i32,
                house.site[1],
                last.pos[2].floor() as i32,
            ];
            let d = crate::memory::distance(at, house.site);
            last.online && (8.0..=48.0).contains(&d)
        }),
        Need::DownTheShaft => last.online && crate::brain::act::shaft_top(memory).is_some(),
        Need::NearTheirDeath => memory.last_death().is_some_and(|died| {
            let at = [
                last.pos[0].floor() as i32,
                last.pos[1].floor() as i32,
                last.pos[2].floor() as i32,
            ];
            let d = crate::memory::distance(at, died);
            last.online && (12.0..=64.0).contains(&d)
        }),
        Need::TargetInBoat => {
            let riding = memory.last.riding.as_str();
            memory.last.online
                && (riding.ends_with("_boat") || riding.ends_with("_raft"))
                && !riding.contains("chest")
        }
        Need::FramesToShow => {
            memory.decor("item_frame").next().is_some()
                && crate::brain::act::frame_shows(memory).is_some()
        }
        Need::Noon => (5_000..7_000).contains(&now.day_time),
        Need::FieldWord => memory
            .field_word()
            .is_some_and(|(_, cells)| !cells.is_empty()),
        Need::ScarecrowParts => {
            memory
                .in_used_container(crate::brain::act::is_armour)
                .is_some()
                && (memory.decor("armor_stand").next().is_some()
                    || memory
                        .in_used_container(|id| id == "minecraft:armor_stand")
                        .is_some())
        }
        Need::Hive => {
            memory.usual_door().is_some()
                && (memory.fixtures("beehive").next().is_some()
                    || memory.fixtures("bee_nest").next().is_some())
        }
        Need::LeashedHorse => crate::brain::act::leashed_horse(memory).is_some(),
        Need::Penned(n) => memory.penned().len() >= n,
        Need::Coop => !memory.egg_line().is_empty(),
        Need::AnimalsNearTarget => last.online && !memory.animals_near_target().is_empty(),
        Need::TheirSign => memory.fixtures("sign").next().is_some(),
        Need::KilledNow => false,
        Need::WeaponAndFrame => {
            crate::brain::act::their_weapon(memory).is_some()
                && memory.decor("item_frame").next().is_some()
        }
        Need::HeadKind => crate::brain::act::head_of_their_kills(memory).is_some(),
        Need::KillKinds => memory.tally.iter().filter(|t| t.kills > 0).count() >= 2,
        Need::TargetAwake => last.online && !last.sleeping,
        Need::Disc => memory.disc_in_chest().is_some(),
        Need::SwapFuel => memory.fuel_swap().is_some(),
        Need::BrewIngredient => memory.brewing_ingredient().is_some(),
        Need::Makings => memory.recipe_at_hand().is_some(),
        Need::TheirBoat => memory.their_boat().is_some(),
        Need::WaterNear(n) => last.online && memory.water_near(n as f64).is_some(),
        Need::Holding(id) => last.online && last.main_hand == id,
        Need::FarFromVillage(n) => memory.far_from_village(n as f64),
        Need::RangBell => false,
        Need::Villagers => memory.villager_near().is_some(),
        Need::BuildLeft(event) => memory.next_build(event).is_some(),
        Need::Walls => {
            memory
                .walls
                .iter()
                .filter(|(_, id)| !crate::memory::cousins(id).is_empty())
                .count()
                >= 20
        }
        Need::LightNear => last.online && crate::brain::act::light_near(memory).is_some(),
        Need::Tunnel => memory.newest_tunnel().is_some(),
        Need::InTunnel => last.online && last.underground && memory.tunnel_at_target().is_some(),
        Need::OreAhead => memory.ore_ahead().is_some(),
        Need::Frame => memory.placed_obsidian.len() >= 10,
        Need::TunnelLights => crate::brain::act::tunnel_lights(memory).len() >= 2,
        Need::BehindThem => memory.behind_in_tunnel().is_some(),
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
        | Action::FishingLine
        | Action::WetSteps
        | Action::PetGrowl
        | Action::Footsteps
        | Action::HandOnShoulder
        | Action::ChestAnswers => config.events.sounds,
        // Their name in the ghasts' cries is a hint too: off with the glitches.
        // The lost hour begins with their screen black: a glitch too.
        Action::Hint(_, _) | Action::GhastName | Action::LostHour => config.events.glitches,
        // The bedside figure too: with the figures off, none comes to the bed (until now the
        // choke and the shot came whatever `events.figure` said).
        Action::Figure(_, _, _)
        | Action::FigureSwing(_)
        | Action::FigureWave
        | Action::DigInto(_)
        | Action::IntoTheDark
        | Action::Choke(_)
        | Action::ShotInBed(_)
        | Action::BurnedInBed
        | Action::LiesBeside
        | Action::RingOfFigures(_)
        | Action::UpsideDown
        | Action::Crawl
        | Action::Hunted
        | Action::BoatPassenger
        | Action::EveryWindow
        | Action::OneMoreEachNight
        | Action::TorchFlash
        | Action::WhereYouDied
        | Action::Rubberband
        | Action::TheOffer
        | Action::TopOfTheLadder
        | Action::FallsFromTheSky
        | Action::PetFollows
        | Action::CrossingAhead
        | Action::InALine
        | Action::PeeksFromGround
        | Action::WaitsAtThePortal
        | Action::AtTheirTable
        | Action::JumpingFar
        | Action::WavesFromInside
        | Action::RidesTheirHorse
        | Action::EatsAtTheirChest
        | Action::MirrorInTheWindow
        | Action::CampNearby
        | Action::FishingFigure
        | Action::ByTheBed
        | Action::SwimsBeneath
        | Action::StandsInTheRain
        | Action::ClimbsAhead
        | Action::OnTheWater
        | Action::InTheTreetops
        | Action::RidesTheCart
        | Action::LanternInTheWoods
        | Action::CrouchedBehindATree
        | Action::PeeksFromTheTrees
        | Action::WhileUnseen(_)
        | Action::DrawnToTheLight
        | Action::TreeToTree
        | Action::RaisesTheirSword
        | Action::Shoot(_) => config.events.figure,
        Action::TabEntry
        | Action::Message(_)
        | Action::Says
        | Action::SaysKills
        | Action::Whisper
        | Action::RepeatsYou
        | Action::DeathMessage
        | Action::TabTwice
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
        | Action::FishingLine
        | Action::WetSteps
        | Action::PetGrowl
        | Action::Footsteps
        | Action::HandOnShoulder => Some("sound"),
        Action::Hint(_, _) => Some("screen"),
        Action::Figure(_, _, _)
        | Action::FigureSwing(_)
        | Action::FigureWave
        | Action::DigInto(_)
        | Action::IntoTheDark
        | Action::RingOfFigures(_)
        | Action::UpsideDown
        | Action::Crawl
        | Action::Hunted
        | Action::BoatPassenger
        | Action::EveryWindow
        | Action::OneMoreEachNight
        | Action::TorchFlash
        | Action::WhereYouDied
        | Action::Rubberband
        | Action::TheOffer
        | Action::TopOfTheLadder
        | Action::FallsFromTheSky
        | Action::PetFollows
        | Action::CrossingAhead
        | Action::InALine
        | Action::PeeksFromGround
        | Action::WaitsAtThePortal
        | Action::AtTheirTable
        | Action::JumpingFar
        | Action::WavesFromInside
        | Action::RidesTheirHorse
        | Action::EatsAtTheirChest
        | Action::MirrorInTheWindow
        | Action::CampNearby
        | Action::FishingFigure
        | Action::ByTheBed
        | Action::SwimsBeneath
        | Action::StandsInTheRain
        | Action::ClimbsAhead
        | Action::OnTheWater
        | Action::InTheTreetops
        | Action::RidesTheCart
        | Action::LanternInTheWoods
        | Action::CrouchedBehindATree
        | Action::PeeksFromTheTrees
        | Action::WhileUnseen(_)
        | Action::DrawnToTheLight
        | Action::TreeToTree
        | Action::RaisesTheirSword
        | Action::Shoot(_) => Some("figure"),
        Action::TabEntry
        | Action::Message(_)
        | Action::Says
        | Action::SaysKills
        | Action::Whisper
        | Action::RepeatsYou
        | Action::DeathMessage
        | Action::TabTwice
        | Action::Answers => Some("chat"),
        Action::ChestAnswers => Some("sound"),
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
    let chain_next = memory.chain_next();
    let chain_starts = memory.chain_starts();

    for event in catalogue {
        if event.id == "wait" || !crate::temper::offered_in(event, stage, &memory.temper) {
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

        if crate::brain::sounds::HOUSE_SOUNDS.contains(&event.id)
            && crate::brain::sounds::house_spot(event.id, memory, 0).is_none()
        {
            continue;
        }

        if crate::rules::check(event, None, memory, stage, day, config).is_err() {
            continue;
        }

        let times = memory.times(event.id);
        let mut weight = 1.0 / (1.0 + times as f64);
        // An open chain's next step (`chains.rs`): likelier, never forced.
        if chain_next.contains(&event.id) || chain_starts.contains(&event.id) {
            weight *= crate::chains::WEIGHT;
        }
        eligible.push((event, weight));
    }

    let mut menu: Vec<&Event> = Vec::new();
    if let Some(wait) = catalogue.iter().find(|e| e.id == "wait") {
        menu.push(wait);
    }

    let want = (config.events.offered as usize).saturating_sub(1);
    // An open chain's next step, when it may happen, always has a place on the menu: its weight
    // alone thinned as the catalogue grew (a hundred and more events), until a chain stalled in
    // a stay for want of being offered. Still never forced: the mind chooses from the menu.
    while let Some(at) = eligible
        .iter()
        .position(|(e, _)| chain_next.contains(&e.id))
    {
        if menu.len() > want {
            break;
        }
        menu.push(eligible.remove(at).0);
    }
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
            outs: Vec::new(),
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![[1, 65, 1]],
        });
        memory
    }

    #[test]
    fn an_open_chain_s_next_step_is_always_on_the_menu_and_a_chain_s_first_step_weighs_more() {
        let memory = memory_at_home();
        // No chain begun: every chain's first step is weighed up.
        let starts = memory.chain_starts();
        assert!(
            starts.contains(&"light_path") && starts.contains(&"far_figure"),
            "{starts:?}"
        );
        let mut begun = memory.clone();
        begun.chains_done.push("the_place".to_string());
        assert!(
            !begun.chain_starts().contains(&"light_path"),
            "a chain runs once a stay"
        );

        // An open chain whose next step may happen: on every menu drawn.
        let mut open = memory_at_home();
        open.chains.push(crate::chains::Open {
            id: "the_place".to_string(),
            landed: vec![0],
            ..Default::default()
        });
        let next = open.chain_next();
        let config = Config::default();
        let mut rng = Rng::new(3);
        for _ in 0..20 {
            let menu = offer(CATALOGUE, 4, &open, &Now::default(), 2.0, &config, &mut rng);
            for step in &next {
                let eligible = crate::events::find(step).is_some_and(|e| {
                    e.needs.iter().all(|n| meets(*n, &open, &Now::default()))
                        && crate::rules::check(e, None, &open, 4, 2.0, &config).is_ok()
                });
                if eligible {
                    assert!(
                        menu.iter().any(|e| e.id == *step),
                        "{step} not on {:?}",
                        menu.iter().map(|e| e.id).collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    #[test]
    fn with_the_figures_off_none_comes_to_the_bed_or_stands_round_the_house() {
        let mut config = Config::default();
        config.events.figure = false;
        for id in [
            "choke",
            "shot_in_bed",
            "burned_in_bed",
            "ring_of_figures",
            "far_figure",
        ] {
            let event = crate::events::find(id).unwrap();
            assert!(!switched_on(event, &config), "{id} with the figures off");
            assert!(switched_on(event, &Config::default()), "{id} with them on");
        }
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
