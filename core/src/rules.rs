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
    /// Another player is near, and witness traces are off (`multiplayer.rs`).
    Witness,
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
            Refused::Witness => "another player is near",
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

    if !crate::temper::offered_in(event, stage, &memory.temper) {
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
    ) || matches!(event.id, "answer_knock" | "breath");
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
        if !exempt && memory.in_view(pos, crate::temper::min_distance(config, memory)) {
            return Err(Refused::InView);
        }
    }

    // A change to the world (the house, their things, a sign) near another player, with
    // witness traces off: it waits. What only the target sees goes on.
    if matches!(event.effect(), "house" | "things" | "words") && memory.witness_near(config, pos) {
        return Err(Refused::Witness);
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
            | crate::events::Action::ShulkerLidPeeks
            | crate::events::Action::Hint(_, _)
            | crate::events::Action::BossBarIt
            | crate::events::Action::CracksInTheWall
            | crate::events::Action::FangsInALine
            | crate::events::Action::SmokeFromTheRoof
            | crate::events::Action::PortalGoesOut
            | crate::events::Action::TheirOwnPotion
            | crate::events::Action::PearlTurned
            | crate::events::Action::ChorusLandsHome
            | crate::events::Action::ChorusBlooms
            | crate::events::Action::EndGoesQuiet
            | crate::events::Action::EndermenAtTheEdge
            | crate::events::Action::BlazeRing
            | crate::events::Action::GatewayFlashes
            | crate::events::Action::ShulkersOpen
            | crate::events::Action::LightningBelow
            | crate::events::Action::PotionAtTheWindow
            | crate::events::Action::RuinedPortalGlows
            | crate::events::Action::SomeoneWentThrough
            | crate::events::Action::SplashInTheNether
            | crate::events::Action::NameFromTheVoid
            | crate::events::Action::PortalWeeps
            | crate::events::Action::PrintsFromThePortal
            | crate::events::Action::PrintsOffTheEdge
            | crate::events::Action::PrintsInSoulSand
            | crate::events::Action::ShipCreaks
            | crate::events::Action::PurpurKnock
            | crate::events::Action::SecondWings
            | crate::events::Action::TheDragonGrowls
            | crate::events::Action::HomeThroughThePortal
            | crate::events::Action::NetherRoofSteps
            | crate::events::Action::TheirDoorBehind
            | crate::events::Action::RainInTheNether
            | crate::events::Action::ZombifiedFollow
            | crate::events::Action::EndermenTurnAway
            | crate::events::Action::BackToTheWall
            | crate::events::Action::MagmaCubesSettle
            | crate::events::Action::HoglinsPart
            | crate::events::Action::GhastWatches
            | crate::events::Action::StriderTurnsHome
            | crate::events::Action::PiglinsStare
            | crate::events::Action::EdgeTug
            | crate::events::Action::ArrowReturned
            | crate::events::Action::SnowballComesBack
            | crate::events::Action::OreCounted
            | crate::events::Action::LateDrop
            | crate::events::Action::TheChildStares
            | crate::events::Action::RefusesFood
            | crate::events::Action::ArmadillosCurl
            | crate::events::Action::HorseBalks
            | crate::events::Action::CattleTurnAway
            | crate::events::Action::TitleYourName
            | crate::events::Action::EyeblossomsOpen
            | crate::events::Action::FirefliesGoOut
            | crate::events::Action::BeamFromTheWater
            | crate::events::Action::DripstoneFalls
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
            | crate::events::Action::FigureSwing(_)
            | crate::events::Action::FigureWave
            | crate::events::Action::OnTheExitPortal
            | crate::events::Action::StandingOnTheVoid
            | crate::events::Action::FarEndOfTheBridge
            | crate::events::Action::StandingInThePortal
            | crate::events::Action::OnThePeak
            | crate::events::Action::FigureInYourArmour
            | crate::events::Action::FigureAtTheLectern
            | crate::events::Action::ClimbingTheHouse
            | crate::events::Action::FigureBows
            | crate::events::Action::FigureDigging
            | crate::events::Action::DigInto(_)
            | crate::events::Action::IntoTheDark
            | crate::events::Action::Pack
            | crate::events::Action::MobComes(_)
            | crate::events::Action::HostilesClose
            | crate::events::Action::Shove(_)
            | crate::events::Action::FramesShow
            | crate::events::Action::BobberUnder
            | crate::events::Action::Shoot(_)
            | crate::events::Action::SplashDark
            | crate::events::Action::DoorCloud
            | crate::events::Action::BedBreaks
            | crate::events::Action::Choke(_)
            | crate::events::Action::RaisesTheirSword
            | crate::events::Action::SnowballThrown
            | crate::events::Action::CrouchedBehindATree
            | crate::events::Action::PeeksFromTheTrees
            | crate::events::Action::WhileUnseen(_)
            | crate::events::Action::DrawnToTheLight
            | crate::events::Action::TreeToTree
            | crate::events::Action::CeilingCrawl
            | crate::events::Action::InTheLava
            | crate::events::Action::OverTheEdge
            | crate::events::Action::RiderOnTheRidge
            | crate::events::Action::HappyGhastOverhead
            | crate::events::Action::TheTreeLine
            | crate::events::Action::AnimalsFleeTheWoods
            | crate::events::Action::LanternInTheWoods
            | crate::events::Action::RidesTheCart
            | crate::events::Action::InTheTreetops
            | crate::events::Action::OnTheWater
            | crate::events::Action::ClimbsAhead
            | crate::events::Action::TrapdoorTaps
            | crate::events::Action::ChestRattles
            | crate::events::Action::ItsBanner
            | crate::events::Action::StandsInTheRain
            | crate::events::Action::SwimsBeneath
            | crate::events::Action::ByTheBed
            | crate::events::Action::FishingFigure
            | crate::events::Action::ChestWontOpen
            | crate::events::Action::LostHour
            | crate::events::Action::BatsInTheHouse
            | crate::events::Action::PetStaresAtWall
            | crate::events::Action::SlipsFromHand
            | crate::events::Action::CampNearby
            | crate::events::Action::EveryDoorOpens
            | crate::events::Action::MirrorInTheWindow
            | crate::events::Action::EatsAtTheirChest
            | crate::events::Action::RidesTheirHorse
            | crate::events::Action::WavesFromInside
            | crate::events::Action::JumpingFar
            | crate::events::Action::AtTheirTable
            | crate::events::Action::ShotInBed(_)
            | crate::events::Action::BurnedInBed
            | crate::events::Action::LiesBeside
            | crate::events::Action::RingOfFigures(_)
            | crate::events::Action::UpsideDown
            | crate::events::Action::Crawl
            | crate::events::Action::Hunted
            | crate::events::Action::BoatPassenger
            | crate::events::Action::EveryWindow
            | crate::events::Action::OneMoreEachNight
            | crate::events::Action::DoorWontOpen
            | crate::events::Action::Sleepwalk
            | crate::events::Action::TorchFlash
            | crate::events::Action::WhereYouDied
            | crate::events::Action::Rubberband
            | crate::events::Action::TheOffer
            | crate::events::Action::TopOfTheLadder
            | crate::events::Action::PulledUnder
            | crate::events::Action::EyeTurnsHome
            | crate::events::Action::WearsYourFace
            | crate::events::Action::AnimalFollows
            | crate::events::Action::HeldBreath
            | crate::events::Action::AllFreeze
            | crate::events::Action::FigureOnMap
            | crate::events::Action::FireworkOverItsHouse
            | crate::events::Action::ChickenLays
            | crate::events::Action::ItsLightsOn
            | crate::events::Action::FallsFromTheSky
            | crate::events::Action::PetFollows
            | crate::events::Action::CrossingAhead
            | crate::events::Action::InALine
            | crate::events::Action::PeeksFromGround
            | crate::events::Action::WaitsAtThePortal
            | crate::events::Action::ScanVillage
            | crate::events::Action::TunnelToward
            | crate::events::Action::TorchesFarEnd
            // A disc put on inside the house as they come to the door: heard, not seen, and
            // waiting until they looked away the approach was over and it never came.
            | crate::events::Action::JukeboxDisc
            // A villager's trade or its walk up: meant to be seen.
            | crate::events::Action::DiarySold
            | crate::events::Action::DiaryBrought
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
