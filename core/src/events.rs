//! The catalogue as data: every event the lodger can do, what stage offers it, what it needs,
//! how often it may happen, and what it becomes for the host.
//!
//! The rows here are the catalogue in `docs/entity.md` as code; the docs test at the bottom
//! keeps the ids agreeing. Adding an event is a row here, a row there, and if it needs a new
//! command, an executor in the host. The brain never sees the catalogue whole: `brain::offer`
//! narrows it by stage, needs, time, cooldowns and caps before the model is asked.

use serde::{Deserialize, Serialize};

/// What an event needs from the memory and the moment before it is offered. The debug screen
/// shows the first unmet need beside an event, in [`Need::words`].
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
    /// A candle the target placed.
    Candle,
    /// A bed.
    Bed,
    /// The lodger holds something it took.
    Held,
    /// A favourite item sits in a used container.
    FavouriteInContainer,
    /// The used container holds an item whose id ends with this.
    InChest(&'static str),
    /// A furnace the target used.
    Furnace,
    /// A furnace, and a chest with something to smelt and something to burn.
    Smeltable,
    /// A furnace with something finished in its output slot.
    FurnaceOutput,
    /// A placed block of this plain kind (`fence_gate`, `glass_pane`).
    Fixture(&'static str),
    /// At least this many placed blocks of the kind.
    Fixtures(&'static str, usize),
    /// A frame, painting or stand of this kind near the home.
    Decor(&'static str),
    /// At least this many.
    Decors(&'static str, usize),
    /// An animal or villager of this kind near the target.
    Creature(&'static str),
    /// A tamed animal of the target's near them.
    Pet,
    /// The target is away from home (more than 24 blocks) or asleep.
    TargetAwayOrAsleep,
    /// The target is online right now.
    TargetOnline,
    /// The target is indoors.
    TargetIndoors,
    /// The target is outdoors.
    TargetOutdoors,
    /// The target is walking.
    TargetWalking,
    /// The target is trying to sleep or in bed.
    TargetInBed,
    /// The target typed something in chat this stay.
    TargetSaid,
    /// Night.
    Night,
    /// Day.
    Day,
    /// It is raining where the target is.
    Raining,
    /// It is not.
    NotRaining,
    /// A route of twenty or more points.
    Route,
    /// A field within reach of the home.
    Field,
    /// Three or more animals or villagers near the target.
    Animals,
    /// The target has killed three or more of one kind.
    Kills,
    /// Water near the home.
    Water,
}

impl Need {
    /// The need in words, for the debug screen's `why` column.
    pub fn words(&self) -> String {
        match self {
            Need::Home => "no home known".to_string(),
            Need::UsedContainer => "no chest they use".to_string(),
            Need::TwoContainers => "fewer than two chests".to_string(),
            Need::Door => "no door".to_string(),
            Need::Light => "no placed light".to_string(),
            Need::Candle => "no candle".to_string(),
            Need::Bed => "no bed".to_string(),
            Need::Held => "holds nothing".to_string(),
            Need::FavouriteInContainer => "no favourite in a chest".to_string(),
            Need::InChest(suffix) => format!("no {} in the chest", suffix.replace('_', " ")),
            Need::Furnace => "no furnace used".to_string(),
            Need::Smeltable => "nothing to smelt with fuel".to_string(),
            Need::FurnaceOutput => "nothing finished in the furnace".to_string(),
            Need::Fixture(kind) => format!("no {} placed", kind.replace('_', " ")),
            Need::Fixtures(kind, n) => format!("fewer than {n} {}s", kind.replace('_', " ")),
            Need::Decor(kind) => format!("no {} near the home", kind.replace('_', " ")),
            Need::Decors(kind, n) => format!("fewer than {n} {}s", kind.replace('_', " ")),
            Need::Creature(kind) => format!("no {} near", kind.replace('_', " ")),
            Need::Pet => "no pet near".to_string(),
            Need::TargetAwayOrAsleep => "the target is home and awake".to_string(),
            Need::TargetOnline => "the target is offline".to_string(),
            Need::TargetIndoors => "the target is not indoors".to_string(),
            Need::TargetOutdoors => "the target is not outdoors".to_string(),
            Need::TargetWalking => "the target is not walking".to_string(),
            Need::TargetInBed => "the target is not in bed".to_string(),
            Need::TargetSaid => "the target has not typed".to_string(),
            Need::Night => "not night".to_string(),
            Need::Day => "not day".to_string(),
            Need::Raining => "not raining".to_string(),
            Need::NotRaining => "raining".to_string(),
            Need::Route => "no route".to_string(),
            Need::Field => "no field".to_string(),
            Need::Animals => "fewer than three animals near".to_string(),
            Need::Kills => "fewer than three kills of one kind".to_string(),
            Need::Water => "no water".to_string(),
        }
    }
}

/// How an event counts against the day's budget: a sound or a screen glitch is half an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weight {
    Full,
    Half,
}

/// Where a creature is moved to.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Where {
    /// Inside the home, by its center.
    Home,
    /// A block outside the home's door.
    OutsideDoor,
    /// A corner of the home, facing the wall.
    Corner,
}

/// How an event is carried out: which command family, resolved against the memory at run time
/// (`brain::act`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Nothing.
    Wait,
    /// Toggle the home's door.
    ToggleDoor,
    /// Toggle a placed fixture of this kind: a gate, a lever, a campfire, a trapdoor.
    ToggleFixture(&'static str),
    /// Light or put out every candle of the home.
    ToggleCandles,
    /// A sound at a place off the home's center, by id.
    Sound(&'static str),
    /// A sound at the target's own position.
    SoundAtTarget(&'static str),
    /// A sound far from the target, 30 to 40 blocks off.
    SoundFar(&'static str),
    /// A sound at a placed fixture of this kind.
    SoundAtFixture(&'static str, &'static str),
    /// This many of a sound at a fixture of this kind, half a second apart.
    Taps(&'static str, &'static str, u32),
    /// The used chest's lid opens for a moment.
    OpenLid,
    /// Move a placed light up to three blocks.
    MoveLight,
    /// A torch where there was none, near the home.
    AddTorch,
    /// Torches from the home toward the place, three to seven.
    LightPath,
    /// Swap two stacks' slots in the used container.
    ShuffleContainer,
    /// Split one stack of the used container into two slots.
    SplitStack,
    /// Sort the used container by item id.
    TidyContainer,
    /// Two stacks trade places between two chests.
    SwapStacks,
    /// Half of the biggest stack goes to the other chest.
    FillGap,
    /// One raw thing and one fuel from the chest into the furnace.
    CookOne,
    /// The furnace's output goes to the chest.
    TakeOutput,
    /// A favourite from the chest is laid on the crafting table.
    ItemOnTable,
    /// An egg on the bed.
    EggOnBed,
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
    /// Wake the target and open the door.
    Wake,
    /// The animals and villagers near the target look at them.
    Stare,
    /// A hole three deep at the place, a torch at the bottom.
    Hole,
    /// Three cobblestone stacked at the place.
    Cairn,
    /// A sapling from the chest, planted at the place.
    Sapling,
    /// A sign at the place with one word.
    Sign,
    /// A sign by the door with what they killed and how many.
    KillSign,
    /// A cairn with a sign naming what they kill most.
    Grave,
    /// A dirt path from the door toward the place.
    PathWorn,
    /// The carpets one block over.
    CarpetShift,
    /// A cauldron filled or emptied.
    Cauldron,
    /// An item frame turned one notch.
    RotateFrame,
    /// Every frame turned the same way.
    FramesTurn,
    /// A painting swapped for another of its size.
    SwapPainting,
    /// An armour stand's head toward the door or the bed.
    StandTurn,
    /// The stands follow the target while they are in the room.
    StandWatch,
    /// An unnamed animal gets a name.
    NameAnimal,
    /// A sheep is another colour.
    DyeSheep,
    /// The pet growls at nothing.
    PetGrowl,
    /// A creature of this kind is moved somewhere.
    MoveCreature(&'static str, Where),
    /// The iron golem faces the house.
    GolemTurned,
    /// A second entry with the target's name in their player list.
    TabEntry,
    /// A game message in the target's name: `joined`, `left`, `sleepers`.
    Message(&'static str),
    /// A chat line in the target's own name, from the bank.
    Says,
    /// A chat line in their own name about what they kill.
    SaysKills,
    /// A whisper to the target from the target.
    Whisper,
    /// The target's last line, said again in their name.
    RepeatsYou,
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
use Weight::{Full, Half};

/// The events the mind can carry out with the host's commands today. The catalogue in the docs
/// is longer; a row joins here when its command exists (see `docs/todo.md`).
pub const CATALOGUE: &[Event] = &[
    row("wait", 1, 5, &[], 0.0, u32::MAX, Half, Action::Wait, true),
    // Containers
    row(
        "open_chest",
        1,
        5,
        &[UsedContainer, TargetIndoors, TargetOnline],
        1.0,
        6,
        Half,
        Action::OpenLid,
        true,
    ),
    row(
        "shuffle_chest",
        1,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        2.0,
        4,
        Full,
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
        Full,
        Action::SplitStack,
        true,
    ),
    row(
        "tidy_chest",
        2,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::TidyContainer,
        false,
    ),
    row(
        "swap_stacks",
        2,
        5,
        &[TwoContainers, TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::SwapStacks,
        true,
    ),
    row(
        "fill_gap",
        2,
        5,
        &[TwoContainers, TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::FillGap,
        true,
    ),
    row(
        "take",
        2,
        5,
        &[FavouriteInContainer, TargetAwayOrAsleep],
        2.0,
        6,
        Full,
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
        Full,
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
        Full,
        Action::PlaceTaken,
        false,
    ),
    // Furnaces and workstations
    row(
        "cook_one",
        1,
        5,
        &[Smeltable, TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::CookOne,
        true,
    ),
    row(
        "take_output",
        2,
        5,
        &[FurnaceOutput, UsedContainer, TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::TakeOutput,
        true,
    ),
    row(
        "item_on_table",
        2,
        5,
        &[
            Fixture("crafting_table"),
            FavouriteInContainer,
            TargetAwayOrAsleep,
        ],
        2.0,
        3,
        Full,
        Action::ItemOnTable,
        false,
    ),
    row(
        "anvil_sound",
        2,
        5,
        &[Fixture("anvil"), TargetInBed],
        2.0,
        4,
        Half,
        Action::SoundAtFixture("anvil", "minecraft:block.anvil.use"),
        true,
    ),
    row(
        "page_turn",
        2,
        5,
        &[Fixture("lectern"), TargetIndoors, TargetOnline],
        2.0,
        4,
        Half,
        Action::SoundAtFixture("lectern", "minecraft:item.book.page_turn"),
        true,
    ),
    row(
        "armor_stand_turn",
        3,
        5,
        &[Decor("armor_stand"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::StandTurn,
        false,
    ),
    row(
        "stand_watch",
        3,
        5,
        &[Decor("armor_stand"), TargetIndoors, TargetOnline],
        3.0,
        3,
        Full,
        Action::StandWatch,
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
        Full,
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
        Full,
        Action::ToggleDoor,
        true,
    ),
    row(
        "window_tap",
        2,
        5,
        &[Fixture("glass_pane"), TargetIndoors, Night],
        2.0,
        4,
        Half,
        Action::Taps("glass_pane", "minecraft:block.glass.hit", 3),
        true,
    ),
    row(
        "carpet_shift",
        2,
        5,
        &[Fixtures("carpet", 3), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::CarpetShift,
        false,
    ),
    row(
        "hole",
        3,
        5,
        &[Home, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::Hole,
        false,
    ),
    row(
        "cairn",
        3,
        5,
        &[Home, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::Cairn,
        false,
    ),
    row(
        "sapling",
        2,
        5,
        &[Home, InChest("sapling"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::Sapling,
        false,
    ),
    row(
        "sign",
        2,
        5,
        &[Home, TargetAwayOrAsleep],
        3.0,
        3,
        Full,
        Action::Sign,
        false,
    ),
    row(
        "path_worn",
        3,
        5,
        &[Door, TargetAwayOrAsleep],
        4.0,
        2,
        Full,
        Action::PathWorn,
        false,
    ),
    // What they kill
    row(
        "kill_sign",
        2,
        5,
        &[Door, Kills, TargetAwayOrAsleep],
        3.0,
        3,
        Full,
        Action::KillSign,
        false,
    ),
    row(
        "grave",
        3,
        5,
        &[Home, Kills, TargetAwayOrAsleep],
        4.0,
        2,
        Full,
        Action::Grave,
        false,
    ),
    row(
        "kill_line",
        3,
        5,
        &[Kills, TargetOnline, Night],
        3.0,
        3,
        Full,
        Action::SaysKills,
        false,
    ),
    // Lights and fire
    row(
        "move_light",
        1,
        5,
        &[Light, TargetAwayOrAsleep],
        2.0,
        4,
        Full,
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
        Full,
        Action::AddTorch,
        false,
    ),
    row(
        "light_path",
        2,
        5,
        &[Home, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::LightPath,
        false,
    ),
    row(
        "put_out_campfire",
        2,
        5,
        &[Fixture("campfire"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::ToggleFixture("campfire"),
        true,
    ),
    row(
        "light_campfire",
        2,
        5,
        &[Fixture("campfire"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::ToggleFixture("campfire"),
        true,
    ),
    row(
        "candles_lit",
        3,
        5,
        &[Candle, TargetAwayOrAsleep, Night],
        2.0,
        3,
        Full,
        Action::ToggleCandles,
        false,
    ),
    row(
        "cauldron",
        2,
        5,
        &[Fixture("cauldron"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::Cauldron,
        true,
    ),
    // Beds and rest
    row(
        "refuse_rest",
        3,
        5,
        &[TargetInBed, Night],
        2.0,
        4,
        Full,
        Action::RefuseRest,
        false,
    ),
    row(
        "wake",
        3,
        5,
        &[TargetInBed, Door, Night],
        3.0,
        2,
        Full,
        Action::Wake,
        false,
    ),
    row(
        "egg_on_bed",
        2,
        5,
        &[Bed, Creature("chicken"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::EggOnBed,
        false,
    ),
    // Decoration and signs
    row(
        "frame_rotate",
        1,
        5,
        &[Decor("item_frame"), TargetAwayOrAsleep],
        2.0,
        6,
        Full,
        Action::RotateFrame,
        true,
    ),
    row(
        "frames_turn",
        3,
        5,
        &[Decors("item_frame", 3), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::FramesTurn,
        false,
    ),
    row(
        "painting_change",
        2,
        5,
        &[Decor("painting"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::SwapPainting,
        true,
    ),
    row(
        "named_animal",
        3,
        5,
        &[Animals, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::NameAnimal,
        false,
    ),
    // Farm and animals
    row(
        "open_gate",
        2,
        5,
        &[Fixture("fence_gate"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::ToggleFixture("fence_gate"),
        true,
    ),
    row(
        "sheep_dyed",
        2,
        5,
        &[Creature("sheep"), TargetAwayOrAsleep],
        3.0,
        3,
        Full,
        Action::DyeSheep,
        false,
    ),
    row(
        "stare",
        3,
        5,
        &[Animals, TargetOnline],
        2.0,
        4,
        Full,
        Action::Stare,
        false,
    ),
    row(
        "pet_growl",
        2,
        5,
        &[Pet, TargetIndoors, TargetOnline],
        2.0,
        4,
        Half,
        Action::PetGrowl,
        true,
    ),
    row(
        "pet_moved",
        2,
        5,
        &[Pet, Home, TargetAwayOrAsleep],
        3.0,
        3,
        Full,
        Action::MoveCreature("", Where::Corner),
        false,
    ),
    row(
        "animal_indoors",
        3,
        5,
        &[Animals, Home, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::MoveCreature("", Where::Home),
        false,
    ),
    row(
        "villager_at_door",
        3,
        5,
        &[Creature("villager"), Door, Night, TargetIndoors],
        3.0,
        2,
        Full,
        Action::MoveCreature("villager", Where::OutsideDoor),
        false,
    ),
    row(
        "golem_turned",
        3,
        5,
        &[Creature("iron_golem"), Home],
        3.0,
        2,
        Full,
        Action::GolemTurned,
        false,
    ),
    // Chat and the player list
    row(
        "tab_entry",
        3,
        5,
        &[TargetOnline],
        2.0,
        3,
        Full,
        Action::TabEntry,
        false,
    ),
    row(
        "joined",
        3,
        5,
        &[TargetOnline],
        2.0,
        3,
        Full,
        Action::Message("joined"),
        false,
    ),
    row(
        "left",
        4,
        5,
        &[TargetOnline],
        3.0,
        2,
        Full,
        Action::Message("left"),
        false,
    ),
    row(
        "sleepers_wrong",
        3,
        5,
        &[TargetInBed],
        2.0,
        3,
        Half,
        Action::Message("sleepers"),
        false,
    ),
    row(
        "says_back",
        3,
        5,
        &[TargetSaid, TargetOnline],
        1.0,
        4,
        Full,
        Action::Says,
        false,
    ),
    row(
        "says_first",
        4,
        5,
        &[TargetOnline, Night],
        2.0,
        3,
        Full,
        Action::Says,
        false,
    ),
    row(
        "whisper_from_you",
        4,
        5,
        &[TargetOnline],
        3.0,
        2,
        Full,
        Action::Whisper,
        false,
    ),
    row(
        "repeats_you",
        4,
        5,
        &[TargetSaid, TargetOnline],
        2.0,
        3,
        Full,
        Action::RepeatsYou,
        false,
    ),
    // Villages and redstone
    row(
        "bell",
        2,
        5,
        &[Fixture("bell"), Night, TargetOnline],
        2.0,
        4,
        Half,
        Action::SoundAtFixture("bell", "minecraft:block.bell.use"),
        true,
    ),
    row(
        "note",
        2,
        5,
        &[Fixture("note_block"), Night, TargetOnline],
        2.0,
        4,
        Half,
        Action::SoundAtFixture("note_block", "minecraft:block.note_block.harp"),
        true,
    ),
    row(
        "lever",
        2,
        5,
        &[Fixture("lever"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::ToggleFixture("lever"),
        true,
    ),
    // Sounds
    row(
        "door_sound",
        1,
        5,
        &[Home, TargetIndoors, TargetOnline],
        1.0,
        6,
        Half,
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
        Half,
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
        Half,
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
        Half,
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
        Half,
        Action::Taps("", "minecraft:entity.zombie.attack_wooden_door", 3),
        true,
    ),
    row(
        "place_sound",
        2,
        5,
        &[TargetIndoors, TargetOnline],
        1.0,
        6,
        Half,
        Action::SoundAtTarget("minecraft:block.stone.place"),
        true,
    ),
    row(
        "lever_click",
        2,
        5,
        &[TargetIndoors, TargetOnline],
        1.0,
        4,
        Half,
        Action::Sound("minecraft:block.lever.click"),
        true,
    ),
    row(
        "drip",
        2,
        5,
        &[TargetIndoors, TargetOnline],
        1.0,
        4,
        Half,
        Action::Sound("minecraft:block.pointed_dripstone.drip_water"),
        true,
    ),
    row(
        "wolf_far",
        2,
        5,
        &[TargetOutdoors, Night],
        1.0,
        4,
        Half,
        Action::SoundFar("minecraft:entity.wolf.howl"),
        true,
    ),
    row(
        "thunder_clear",
        2,
        5,
        &[TargetOutdoors, NotRaining],
        2.0,
        3,
        Half,
        Action::SoundFar("minecraft:entity.lightning_bolt.thunder"),
        true,
    ),
    row(
        "arrow_hit",
        3,
        5,
        &[TargetIndoors, TargetOnline],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.arrow.hit"),
        true,
    ),
    row(
        "portal_sound",
        3,
        5,
        &[TargetIndoors, TargetOnline],
        2.0,
        3,
        Half,
        Action::Sound("minecraft:block.portal.ambient"),
        true,
    ),
    row(
        "your_voice",
        4,
        5,
        &[TargetIndoors, TargetOnline],
        3.0,
        2,
        Half,
        Action::Sound("minecraft:entity.player.hurt"),
        false,
    ),
    row(
        "silence",
        3,
        5,
        &[TargetOnline, Night],
        2.0,
        3,
        Half,
        Action::Hint("silence", 600.0),
        true,
    ),
    // The sky and the light, and the screen: hints the client renders
    row(
        "flicker",
        2,
        5,
        &[TargetIndoors, TargetOnline],
        1.0,
        6,
        Half,
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
        Full,
        Action::Hint("lights_fail", 12.0),
        false,
    ),
    row(
        "sun_blink",
        3,
        5,
        &[TargetOutdoors, Day],
        2.0,
        4,
        Full,
        Action::Hint("sun_blink", 2.0),
        false,
    ),
    row(
        "lightning_silent",
        3,
        5,
        &[TargetOutdoors, Night, NotRaining],
        2.0,
        3,
        Half,
        Action::Hint("lightning_silent", 0.4),
        true,
    ),
    row(
        "red_rain",
        4,
        5,
        &[TargetOutdoors, Raining],
        3.0,
        2,
        Full,
        Action::Hint("red_rain", 150.0),
        false,
    ),
    row(
        "crosshair_gone",
        3,
        5,
        &[TargetOnline],
        2.0,
        3,
        Half,
        Action::Hint("crosshair_gone", 5.0),
        true,
    ),
    row(
        "hotbar_slip",
        2,
        5,
        &[TargetOnline],
        2.0,
        3,
        Half,
        Action::Hint("hotbar_slip", 0.2),
        true,
    ),
    row(
        "camera_nudge",
        3,
        5,
        &[Home, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::Hint("camera_nudge", 0.5),
        true,
    ),
    row(
        "fov_pull",
        4,
        5,
        &[TargetIndoors, TargetOnline],
        3.0,
        2,
        Full,
        Action::Hint("fov_pull", 10.0),
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

    /// `2 to 5`, or `3` for one stage.
    pub fn stages_words(&self) -> String {
        if self.from_stage == self.to_stage {
            self.from_stage.to_string()
        } else {
            format!("{} to {}", self.from_stage, self.to_stage)
        }
    }

    /// The words the model sees for this event in the `What should it do?` menu.
    pub fn menu_words(&self) -> &'static str {
        match self.id {
            "wait" => "nothing",
            "open_chest" => "open the chest for a moment",
            "shuffle_chest" => "move things around in the chest",
            "stack_split" => "split a stack in the chest",
            "tidy_chest" => "sort the chest",
            "swap_stacks" => "swap two stacks between their chests",
            "fill_gap" => "put some of one thing in the other chest",
            "take" => "take one thing they use",
            "return_taken" => "put the thing back",
            "place_taken" => "leave the thing where they will find it",
            "cook_one" => "cook one thing in the furnace",
            "take_output" => "move what the furnace made to the chest",
            "item_on_table" => "lay a tool on the crafting table",
            "anvil_sound" => "the anvil, from the workshop",
            "page_turn" => "pages turning by the lectern",
            "armor_stand_turn" => "turn the armour stand's head",
            "stand_watch" => "have the armour stands watch them",
            "open_door" => "open the door",
            "close_door" => "close the door",
            "window_tap" => "tap on the window",
            "carpet_shift" => "move the carpets one over",
            "hole" => "dig a small hole with a torch in it",
            "cairn" => "stack three stones",
            "sapling" => "plant their sapling",
            "sign" => "put up a sign with one word",
            "path_worn" => "wear a path from the door",
            "kill_sign" => "put up a sign counting what they killed",
            "grave" => "raise a grave for what they kill",
            "kill_line" => "mention what they killed, in their name",
            "move_light" => "move a torch",
            "add_torch" => "add a torch",
            "light_path" => "light a path of torches away from the house",
            "put_out_campfire" => "put the campfire out",
            "light_campfire" => "light the campfire",
            "candles_lit" => "light or put out the candles",
            "cauldron" => "fill or empty the cauldron",
            "refuse_rest" => "keep them from sleeping",
            "wake" => "wake them and open the door",
            "egg_on_bed" => "leave an egg on the bed",
            "frame_rotate" => "turn the thing in a frame",
            "frames_turn" => "turn every frame the same way",
            "painting_change" => "change a painting",
            "named_animal" => "name one of their animals",
            "open_gate" => "open the gate",
            "sheep_dyed" => "dye a sheep",
            "stare" => "have the animals look at them",
            "pet_growl" => "have the pet growl at nothing",
            "pet_moved" => "move the pet to a corner",
            "animal_indoors" => "bring an animal into the house",
            "villager_at_door" => "stand a villager at the door",
            "golem_turned" => "turn the golem toward the house",
            "tab_entry" => "show their name twice in the player list",
            "joined" => "say they joined the game",
            "left" => "say they left the game",
            "sleepers_wrong" => "count one sleeper too many",
            "says_back" => "answer them in their own name",
            "says_first" => "say something in their name",
            "whisper_from_you" => "whisper to them as them",
            "repeats_you" => "repeat what they said",
            "bell" => "ring the bell once",
            "note" => "one note from the note block",
            "lever" => "flip the lever",
            "door_sound" => "a door sound from another room",
            "chest_sound" => "a chest sound from another room",
            "cut_music" => "stop the music",
            "late_steps" => "their own footsteps, a moment late",
            "knock" => "knock on the door",
            "place_sound" => "a block placed behind them",
            "lever_click" => "a lever's click, in a house with none",
            "drip" => "dripping water indoors",
            "wolf_far" => "a wolf howling far off",
            "thunder_clear" => "thunder under a clear sky",
            "arrow_hit" => "an arrow hitting the wall behind them",
            "portal_sound" => "a portal's hum in the house",
            "your_voice" => "their own hurt sound from the next room",
            "silence" => "silence the world's sounds for an evening",
            "flicker" => "flicker a light",
            "lights_fail" => "put the lights out for a while",
            "sun_blink" => "make the sun blink out",
            "lightning_silent" => "a flash of lightning with no thunder",
            "red_rain" => "turn the rain red",
            "crosshair_gone" => "take the crosshair away for a moment",
            "hotbar_slip" => "slip their hotbar to the favourite tool",
            "camera_nudge" => "nudge their view toward the place",
            "fov_pull" => "narrow their view and let it snap back",
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
        assert!(CATALOGUE.len() >= 80, "{} rows", CATALOGUE.len());
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
    fn every_event_has_words_for_the_menu_and_its_stages() {
        for event in CATALOGUE {
            assert_ne!(
                event.menu_words(),
                event.id,
                "{} has no menu words",
                event.id
            );
        }
        assert_eq!(find("open_door").unwrap().stages_words(), "1 to 5");
        assert_eq!(Need::Fixture("fence_gate").words(), "no fence gate placed");
    }

    /// Every id here is a row in docs/entity.md's catalogue, so the two never drift apart.
    #[test]
    fn every_event_is_in_the_docs_catalogue() {
        let docs =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/entity.md"))
                .expect("docs/entity.md beside the crate");
        let missing: Vec<&str> = CATALOGUE
            .iter()
            .filter(|event| !docs.contains(&format!("| `{}` |", event.id)))
            .map(|event| event.id)
            .collect();
        assert!(
            missing.is_empty(),
            "not rows in docs/entity.md: {missing:?}"
        );
    }
}
