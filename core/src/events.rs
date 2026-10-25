//! The catalogue as data: every event the lodger can do, what stage offers it, what it needs,
//! how often it may happen, and what it becomes for the host.
//!
//! The rows here are the catalogue in `docs/entity.md` as code; the docs test at the bottom
//! keeps the ids agreeing. Adding an event is a row here, a row there, and if it needs a new
//! command, an executor in the host. The brain never sees the catalogue whole: `brain::offer`
//! narrows it by stage, needs, time, cooldowns and caps before the model is asked.

use serde::{Deserialize, Serialize};

/// What an event needs from the memory and the moment before it is offered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Need {
    /// The memory knows where the target sleeps and keeps things.
    Home,
    /// A container the target opens, with contents.
    UsedContainer,
    /// Two containers the target opens.
    TwoContainers,
    /// A door of the home.
    Door,
    /// A torch, lantern or candle the target placed.
    Light,
    /// A bed.
    Bed,
    /// The lodger holds something it took.
    Held,
    /// A favourite item sits in a used container.
    FavouriteInContainer,
    /// The target is away from home (more than 24 blocks) or asleep.
    TargetAwayOrAsleep,
    /// The target is online right now.
    TargetOnline,
    /// The target is indoors.
    TargetIndoors,
    /// The target is walking.
    TargetWalking,
    /// The target is trying to sleep or in bed.
    TargetInBed,
    /// Night.
    Night,
    /// A route of twenty or more points.
    Route,
    /// A field within reach of the home.
    Field,
    /// Three or more animals or villagers near the target.
    Animals,
    /// Water near the home.
    Water,
}

/// How an event counts against the day's budget: a sound or a screen glitch is half an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weight {
    Full,
    Half,
}

/// How an event is carried out: which command family, resolved against the memory at run time
/// (`brain::act`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Nothing.
    Wait,
    /// Toggle the home's door, or a given door.
    ToggleDoor,
    /// A sound at a place, by id.
    Sound(&'static str),
    /// A sound at the target's own position.
    SoundAtTarget(&'static str),
    /// A chest's open then close sounds at the used container.
    ChestSounds,
    /// Move a placed light up to three blocks.
    MoveLight,
    /// A torch where there was none, near the home.
    AddTorch,
    /// Swap two stacks' slots in the used container.
    ShuffleContainer,
    /// Split one stack of the used container into two slots.
    SplitStack,
    /// Take one favourite from the used container and hold it.
    Take,
    /// Put the held item back in its slot.
    ReturnTaken,
    /// Drop the held item at the want's place.
    PlaceTaken,
    /// A client hint by kind, for the given seconds.
    Hint(&'static str, f32),
    /// Refuse the target's next sleep.
    RefuseRest,
    /// The animals and villagers near the target look at them.
    Stare,
}

/// One row of the catalogue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub id: &'static str,
    /// The first stage that offers it; it stays offered after.
    pub from_stage: u8,
    /// The last stage that offers it.
    pub to_stage: u8,
    pub needs: &'static [Need],
    pub cooldown_days: f32,
    pub most_per_stay: u32,
    pub weight: Weight,
    pub action: Action,
    /// The size class the `Should it do something soon?` answer maps to: small things, or
    /// things the target will notice.
    pub small: bool,
}

#[allow(clippy::too_many_arguments)]
const fn row(
    id: &'static str,
    from_stage: u8,
    to_stage: u8,
    needs: &'static [Need],
    cooldown_days: f32,
    most_per_stay: u32,
    weight: Weight,
    action: Action,
    small: bool,
) -> Event {
    Event {
        id,
        from_stage,
        to_stage,
        needs,
        cooldown_days,
        most_per_stay,
        weight,
        action,
        small,
    }
}

use Need::*;

/// The events the mind can carry out with the host's commands today. The catalogue in the docs
/// is longer; a row joins here when its command exists (see `docs/todo.md`).
pub const CATALOGUE: &[Event] = &[
    row(
        "wait",
        1,
        5,
        &[],
        0.0,
        u32::MAX,
        Weight::Half,
        Action::Wait,
        true,
    ),
    // Containers
    row(
        "open_chest",
        1,
        5,
        &[UsedContainer, TargetIndoors, TargetOnline],
        1.0,
        6,
        Weight::Half,
        Action::ChestSounds,
        true,
    ),
    row(
        "shuffle_chest",
        1,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        2.0,
        4,
        Weight::Full,
        Action::ShuffleContainer,
        true,
    ),
    row(
        "stack_split",
        1,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        2.0,
        4,
        Weight::Full,
        Action::SplitStack,
        true,
    ),
    row(
        "take",
        2,
        5,
        &[FavouriteInContainer, TargetAwayOrAsleep],
        2.0,
        6,
        Weight::Full,
        Action::Take,
        false,
    ),
    row(
        "return_taken",
        2,
        5,
        &[Held, TargetAwayOrAsleep],
        1.0,
        6,
        Weight::Full,
        Action::ReturnTaken,
        false,
    ),
    row(
        "place_taken",
        2,
        5,
        &[Held, Home, TargetAwayOrAsleep],
        1.0,
        6,
        Weight::Full,
        Action::PlaceTaken,
        false,
    ),
    // The house
    row(
        "open_door",
        1,
        5,
        &[Door, TargetAwayOrAsleep],
        2.0,
        4,
        Weight::Full,
        Action::ToggleDoor,
        true,
    ),
    row(
        "close_door",
        1,
        5,
        &[Door, TargetAwayOrAsleep],
        2.0,
        4,
        Weight::Full,
        Action::ToggleDoor,
        true,
    ),
    // Lights
    row(
        "move_light",
        1,
        5,
        &[Light, TargetAwayOrAsleep],
        2.0,
        4,
        Weight::Full,
        Action::MoveLight,
        true,
    ),
    row(
        "add_torch",
        2,
        5,
        &[Home, TargetAwayOrAsleep],
        2.0,
        4,
        Weight::Full,
        Action::AddTorch,
        false,
    ),
    // Beds and rest
    row(
        "refuse_rest",
        3,
        5,
        &[TargetInBed, Night],
        2.0,
        4,
        Weight::Full,
        Action::RefuseRest,
        false,
    ),
    // Animals
    row(
        "stare",
        3,
        5,
        &[Animals, TargetOnline],
        2.0,
        4,
        Weight::Full,
        Action::Stare,
        false,
    ),
    // Sounds
    row(
        "door_sound",
        1,
        5,
        &[Home, TargetIndoors, TargetOnline],
        1.0,
        6,
        Weight::Half,
        Action::Sound("minecraft:block.wooden_door.open"),
        true,
    ),
    row(
        "chest_sound",
        1,
        5,
        &[Home, TargetIndoors, TargetOnline],
        1.0,
        6,
        Weight::Half,
        Action::Sound("minecraft:block.chest.open"),
        true,
    ),
    row(
        "cut_music",
        1,
        5,
        &[TargetOnline],
        1.0,
        6,
        Weight::Half,
        Action::Hint("cut_music", 600.0),
        true,
    ),
    row(
        "late_steps",
        2,
        5,
        &[TargetWalking, TargetOnline],
        1.0,
        6,
        Weight::Half,
        Action::SoundAtTarget("minecraft:block.stone.step"),
        true,
    ),
    row(
        "knock",
        2,
        5,
        &[Home, TargetIndoors, Night],
        1.0,
        4,
        Weight::Half,
        Action::Sound("minecraft:entity.zombie.attack_wooden_door"),
        true,
    ),
    row(
        "place_sound",
        2,
        5,
        &[TargetIndoors, TargetOnline],
        1.0,
        6,
        Weight::Half,
        Action::SoundAtTarget("minecraft:block.stone.place"),
        true,
    ),
    // Glitches the client renders when it can; today it logs them
    row(
        "flicker",
        2,
        5,
        &[TargetIndoors, TargetOnline],
        1.0,
        6,
        Weight::Half,
        Action::Hint("flicker", 4.0),
        true,
    ),
    row(
        "lights_fail",
        3,
        5,
        &[TargetIndoors, Night],
        2.0,
        4,
        Weight::Full,
        Action::Hint("lights_fail", 12.0),
        false,
    ),
    row(
        "sun_blink",
        3,
        5,
        &[TargetOnline],
        2.0,
        4,
        Weight::Full,
        Action::Hint("sun_blink", 2.0),
        false,
    ),
];

/// The row with this id, if any.
pub fn find(id: &str) -> Option<&'static Event> {
    CATALOGUE.iter().find(|event| event.id == id)
}

impl Event {
    pub fn offered_in(&self, stage: u8) -> bool {
        self.from_stage <= stage && stage <= self.to_stage
    }

    /// What the event counts against the day's budget.
    pub fn cost(&self) -> f32 {
        match self.weight {
            Weight::Full => 1.0,
            Weight::Half => 0.5,
        }
    }

    /// The words the model sees for this event in the `What should it do?` menu.
    pub fn menu_words(&self) -> &'static str {
        match self.id {
            "wait" => "nothing",
            "open_chest" => "open the chest for a moment",
            "shuffle_chest" => "move things around in the chest",
            "stack_split" => "split a stack in the chest",
            "take" => "take one thing they use",
            "return_taken" => "put the thing back",
            "place_taken" => "leave the thing where they will find it",
            "open_door" => "open the door",
            "close_door" => "close the door",
            "move_light" => "move a torch",
            "add_torch" => "add a torch",
            "refuse_rest" => "keep them from sleeping",
            "stare" => "have the animals look at them",
            "door_sound" => "a door sound from another room",
            "chest_sound" => "a chest sound from another room",
            "cut_music" => "stop the music",
            "late_steps" => "their own footsteps, a moment late",
            "knock" => "knock on the door",
            "place_sound" => "a block placed behind them",
            "flicker" => "flicker a light",
            "lights_fail" => "put the lights out for a while",
            "sun_blink" => "make the sun blink out",
            _ => self.id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique_and_wait_is_always_offered() {
        let mut seen = HashSet::new();
        for event in CATALOGUE {
            assert!(seen.insert(event.id), "{} twice", event.id);
            assert!(
                event.from_stage >= 1 && event.to_stage <= 5 && event.from_stage <= event.to_stage,
                "{}",
                event.id
            );
        }

        let wait = find("wait").unwrap();
        for stage in 1..=5 {
            assert!(wait.offered_in(stage));
        }
        assert!(wait.needs.is_empty());
    }

    #[test]
    fn nothing_bold_is_offered_in_stage_one() {
        for event in CATALOGUE {
            if event.offered_in(1) {
                assert!(event.small, "{} in stage one must be small", event.id);
                assert_ne!(event.id, "take");
            }
        }
    }

    #[test]
    fn every_event_has_words_for_the_menu() {
        for event in CATALOGUE {
            assert_ne!(
                event.menu_words(),
                event.id,
                "{} has no menu words",
                event.id
            );
        }
    }

    /// Every id here is a row in docs/entity.md's catalogue, so the two never drift apart.
    #[test]
    fn every_event_is_in_the_docs_catalogue() {
        let docs =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/entity.md"))
                .expect("docs/entity.md beside the crate");
        for event in CATALOGUE {
            assert!(
                docs.contains(&format!("| `{}` |", event.id)),
                "{} is not a row in docs/entity.md",
                event.id
            );
        }
    }
}
