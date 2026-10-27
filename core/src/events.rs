//! The catalogue as data: every event the lodger can do, what stage offers it, what it needs,
//! how often it may happen, and what it becomes for the host.
//!
//! The rows here are the catalogue in `docs/entity.md` as code; the docs test at the bottom
//! keeps the ids agreeing. Adding an event is a row here, a row there, and if it needs a new
//! command, an executor in the host. The brain never sees the catalogue whole: `brain::offer`
//! narrows it by stage, needs, time, cooldowns and caps before the model is asked.

/// What an event needs from the memory and the moment before it is offered. The debug screen
/// shows the first unmet need beside an event, in [`Need::words`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// The target just typed: met only by the chat trigger, never on the planning menu.
    Heard,
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
    /// Three or more hostile mobs within 48 blocks of the target.
    Hostiles,
    /// The target has killed three or more of one kind.
    Kills,
    /// Water near the home.
    Water,
    /// A feature of the surroundings by kind: `water`, `crops`, `snow`, `sand`, `trees`,
    /// `village`, `mineshaft`.
    Around(&'static str),
    /// A boat or minecart near the home.
    Vehicle,
    /// The target is under the ground.
    TargetUnderground,
    /// The target ate within the last seconds.
    TargetAte,
    /// The target is walking toward a door of the home, within six blocks.
    TargetApproachingDoor,
    /// The village scan found blocks of this kind (`door`, `bell`).
    Scanned(&'static str),
    /// The first hour of the day.
    Dawn,
    /// A placed building block with a cousin (planks, cobblestone, bricks, glass, a torch).
    Wall,
    /// A placed torch or lantern.
    PlacedLight,
    /// A placed glass pane.
    Pane,
    /// The used container holds an item whose id ends with any of these.
    InChestAny(&'static [&'static str]),
    /// The target is walking toward the used chest, within six blocks.
    TargetApproachingChest,
    /// Six or more dyed stacks in the used chest.
    ColouredStacks,
    /// A damageable favourite tool sits in a chest.
    ToolInContainer,
}

impl Need {
    /// Whether the need is a fact of the world the target must change to bring back (the home,
    /// a chest, a door, what a chest holds, a route), rather than a moment that comes and goes
    /// (night, the target away, rain, an animal near). A plan whose lasting need is gone is
    /// cancelled; one whose passing need is unmet waits.
    pub fn lasting(&self) -> bool {
        !matches!(
            self,
            Need::TargetAwayOrAsleep
                | Need::TargetOnline
                | Need::TargetIndoors
                | Need::TargetOutdoors
                | Need::TargetWalking
                | Need::TargetInBed
                | Need::TargetSaid
                | Need::Heard
                | Need::Night
                | Need::Day
                | Need::Dawn
                | Need::Raining
                | Need::NotRaining
                | Need::Creature(_)
                | Need::Pet
                | Need::Animals
                | Need::Hostiles
                | Need::TargetUnderground
                | Need::TargetAte
                | Need::TargetApproachingDoor
                | Need::TargetApproachingChest
        )
    }

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
            Need::Heard => "answered only as they type".to_string(),
            Need::Night => "not night".to_string(),
            Need::Day => "not day".to_string(),
            Need::Raining => "not raining".to_string(),
            Need::NotRaining => "raining".to_string(),
            Need::Route => "no route".to_string(),
            Need::Field => "no field".to_string(),
            Need::Animals => "fewer than three animals near".to_string(),
            Need::Hostiles => "fewer than three hostile mobs near".to_string(),
            Need::Kills => "fewer than three kills of one kind".to_string(),
            Need::Around(kind) => format!("no {} around the house", kind.replace('_', " ")),
            Need::Vehicle => "no boat or minecart near".to_string(),
            Need::TargetUnderground => "the target is not underground".to_string(),
            Need::TargetAte => "the target has not just eaten".to_string(),
            Need::TargetApproachingDoor => "the target is not walking up to a door".to_string(),
            Need::Scanned(kind) => format!("the scan found no {kind}"),
            Need::Dawn => "not dawn".to_string(),
            Need::Wall => "no building block it knows the kind of".to_string(),
            Need::PlacedLight => "no torch or lantern placed".to_string(),
            Need::Pane => "no glass pane placed".to_string(),
            Need::InChestAny(_) => "none of those in the chest".to_string(),
            Need::TargetApproachingChest => "the target is not walking up to the chest".to_string(),
            Need::ColouredStacks => "fewer than six dyed stacks in the chest".to_string(),
            Need::ToolInContainer => "no tool with wear in a chest".to_string(),
            Need::Water => "no water".to_string(),
        }
    }
}

/// How an event counts against the day's budget: a sound or a screen glitch is half an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    Full,
    Half,
}

/// Where a creature is moved to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Where {
    /// Inside the home, by its center.
    Home,
    /// A block outside the home's door.
    OutsideDoor,
    /// A corner of the home, facing the wall.
    Corner,
    /// The far end of the route the target walks most.
    RouteEnd,
}

/// How an event is carried out: which command family, resolved against the memory at run time
/// (`brain::act`).
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// Every hostile mob near holds still and stares at the target until dawn.
    HostilesStare,
    /// An answer to what the target just typed, in their name, to them alone.
    Answers,
    /// A boat or minecart moved to the far side of the water, or its middle.
    MoveVehicle,
    /// A boat or minecart given a push, so it moves on its own in view.
    PushVehicle(&'static str),
    /// A block of water in a corner of the floor, from a bucket in the chest.
    BucketFloor,
    /// Snow cleared from the path between the door and the spot.
    SnowCleared,
    /// Farmland trampled in a line from the field toward the door.
    Trample,
    /// One crop harvested and its yield in the chest.
    HarvestOne,
    /// Pickaxe sounds from inside the rock near the target.
    MiningSounds,
    /// A pressure plate from the chest, in front of the chest.
    PlateLaid,
    /// The door opens as the target walks up to it.
    DoorAhead,
    /// Every door the village scan found, toggled.
    VillageDoors,
    /// The village scanned for its doors and bell.
    ScanVillage,
    /// The rock around the target scanned for ore, while they mine.
    ScanOre,
    /// A torch on an ore vein the scan found.
    OreMarked,
    /// A fixture of this kind turned a quarter turn.
    RotateFixture(&'static str),
    /// A frame emptied, its item laid on the bed.
    FrameEmptied,
    /// A frame's item swapped with one from the chest.
    FrameSwap,
    /// A piece of armour from the chest on the stand.
    StandDress,
    /// The favourite tool in the chest worn down a few dozen uses.
    ToolUsed,
    /// The furnace's fuel back in the chest, mid-smelt.
    ColdFurnace,
    /// The chest's lid opening as the target walks up to it.
    OpenOnEnter,
    /// A flower from the chest planted by the door.
    PlantFlower,
    /// One crop replaced by another from seeds in the chest.
    ReplantOther,
    /// A line of ice across the water toward the house.
    IcePath,
    /// The chest sorted by colour.
    TidyByColour,
    /// Two sounds at a fixture, the second a moment after.
    SoundPair(&'static str, &'static str, &'static str),
    /// Vanilla's "this bed is occupied" and out of the bed.
    BedOccupied,
    /// The target wakes one block from where they lay down.
    WokeElsewhere,
    /// The figure in the target's skin, somewhere, for some seconds, gone when approached.
    Figure(FigureWhere, f32, f32),
    /// One building block of the house becomes its cousin: planks another wood, cobblestone
    /// mossy, a pane iron bars, a torch blue. Matched by id suffix: all, or lights, or panes.
    WallSwap(&'static str),
    /// The bed breaks under them as they lie down; its item drops where it stood.
    BedBreaks,
}

/// Where a figure stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FigureWhere {
    /// Fifty to seventy blocks from the target, away from the home, on the ground.
    Far,
    /// Sixty blocks behind the target along the way they came, on the ground.
    Behind,
    /// On the home's roof.
    Roof,
    /// On the far shore of the water.
    AcrossWater,
    /// In the home's doorway.
    Doorway,
    /// At the chest they use.
    AtChest,
    /// On the bed.
    InBed,
    /// Walking the route, start to end.
    Route,
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
        "animal_gone",
        3,
        5,
        &[Animals, Route, Home, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::MoveCreature("", Where::RouteEnd),
        false,
    ),
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
        "answers_you",
        2,
        5,
        &[Heard, TargetOnline],
        0.25,
        60,
        Half,
        Action::Answers,
        true,
    ),
    row(
        "hostiles_stare",
        3,
        5,
        &[Night, TargetOnline, Hostiles],
        2.0,
        2,
        Full,
        Action::HostilesStare,
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
    // Decoration, the stands and the tools, second round
    row(
        "stairs_turned",
        2,
        5,
        &[Fixture("stairs"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::RotateFixture("stairs"),
        true,
    ),
    row(
        "banner_turned",
        2,
        5,
        &[Fixture("banner"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::RotateFixture("banner"),
        true,
    ),
    row(
        "frame_emptied",
        3,
        5,
        &[Decor("item_frame"), Bed, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::FrameEmptied,
        false,
    ),
    row(
        "frame_swap",
        3,
        5,
        &[Decor("item_frame"), UsedContainer, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::FrameSwap,
        false,
    ),
    row(
        "armor_stand_dress",
        3,
        5,
        &[
            Decor("armor_stand"),
            InChestAny(&["_helmet", "_chestplate", "_leggings", "_boots"]),
            TargetAwayOrAsleep,
        ],
        3.0,
        2,
        Full,
        Action::StandDress,
        false,
    ),
    row(
        "tool_used",
        3,
        5,
        &[ToolInContainer, TargetAwayOrAsleep],
        3.0,
        3,
        Full,
        Action::ToolUsed,
        false,
    ),
    row(
        "cold_furnace",
        3,
        5,
        &[Furnace, UsedContainer, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::ColdFurnace,
        true,
    ),
    row(
        "open_on_enter",
        4,
        5,
        &[TargetApproachingChest],
        2.0,
        3,
        Full,
        Action::OpenOnEnter,
        false,
    ),
    row(
        "tidy_by_colour",
        3,
        5,
        &[ColouredStacks, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::TidyByColour,
        false,
    ),
    row(
        "plant_flower",
        2,
        5,
        &[
            Door,
            InChestAny(&[
                "poppy",
                "dandelion",
                "tulip",
                "orchid",
                "allium",
                "daisy",
                "cornflower",
                "lily_of_the_valley",
            ]),
            TargetAwayOrAsleep,
        ],
        3.0,
        2,
        Full,
        Action::PlantFlower,
        true,
    ),
    row(
        "replant_other",
        3,
        5,
        &[
            Around("crops"),
            InChestAny(&["carrot", "potato", "beetroot_seeds"]),
            TargetAwayOrAsleep,
        ],
        3.0,
        2,
        Full,
        Action::ReplantOther,
        false,
    ),
    row(
        "ice_path",
        3,
        5,
        &[Around("water"), Home, TargetAwayOrAsleep],
        4.0,
        1,
        Full,
        Action::IcePath,
        false,
    ),
    row(
        "bed_occupied",
        3,
        5,
        &[TargetInBed],
        2.0,
        3,
        Full,
        Action::BedOccupied,
        false,
    ),
    row(
        "woke_elsewhere",
        4,
        5,
        &[TargetInBed, Bed],
        3.0,
        2,
        Full,
        Action::WokeElsewhere,
        false,
    ),
    row(
        "pet_outside",
        3,
        5,
        &[Pet, Door, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::MoveCreature("", Where::OutsideDoor),
        false,
    ),
    row(
        "tab_entry_stays",
        4,
        5,
        &[TargetOnline],
        4.0,
        1,
        Full,
        Action::TabEntry,
        false,
    ),
    row(
        "note_tune",
        3,
        5,
        &[Fixture("note_block"), Night, TargetOnline],
        3.0,
        2,
        Half,
        Action::Taps("note_block", "minecraft:block.note_block.harp", 3),
        true,
    ),
    row(
        "piston_once",
        3,
        5,
        &[Fixture("piston"), Night, TargetOnline],
        2.0,
        3,
        Half,
        Action::SoundPair(
            "piston",
            "minecraft:block.piston.extend",
            "minecraft:block.piston.contract",
        ),
        true,
    ),
    row(
        "disc_phantom",
        3,
        5,
        &[Fixture("jukebox"), TargetIndoors, TargetOnline],
        3.0,
        2,
        Half,
        Action::SoundAtFixture("jukebox", "minecraft:music_disc.11"),
        true,
    ),
    row(
        "fog_close",
        3,
        5,
        &[TargetOutdoors],
        2.0,
        3,
        Half,
        Action::Hint("fog_close", 60.0),
        true,
    ),
    row(
        "snow_indoors",
        3,
        5,
        &[TargetIndoors, Around("snow")],
        3.0,
        2,
        Half,
        Action::Hint("snow_indoors", 8.0),
        true,
    ),
    row(
        "rain_indoors",
        3,
        5,
        &[TargetIndoors, Raining],
        3.0,
        2,
        Half,
        Action::Hint("rain_indoors", 8.0),
        true,
    ),
    row(
        "scan_ore",
        2,
        5,
        &[TargetUnderground],
        0.5,
        u32::MAX,
        Half,
        Action::ScanOre,
        true,
    ),
    row(
        "ore_marked",
        2,
        5,
        &[Scanned("_ore"), TargetUnderground],
        2.0,
        4,
        Full,
        Action::OreMarked,
        true,
    ),
    // The house, changed (owner, 2026-11-09: "replace certain blocks of house", "trying to
    // sleep breaks the bed")
    row(
        "wall_swap",
        3,
        5,
        &[Wall, TargetAwayOrAsleep],
        2.0,
        6,
        Full,
        Action::WallSwap(""),
        false,
    ),
    row(
        "torch_blue",
        3,
        5,
        &[PlacedLight, TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::WallSwap("torch"),
        false,
    ),
    row(
        "window_bars",
        4,
        5,
        &[Pane, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::WallSwap("glass_pane"),
        false,
    ),
    row(
        "bed_breaks",
        4,
        5,
        &[TargetInBed, Night],
        4.0,
        1,
        Full,
        Action::BedBreaks,
        false,
    ),
    // The figures
    row(
        "far_figure",
        3,
        5,
        &[Home, TargetOutdoors],
        1.0,
        8,
        Full,
        Action::Figure(FigureWhere::Far, 30.0, 32.0),
        false,
    ),
    row(
        "behind_you",
        3,
        5,
        &[Route, TargetWalking, TargetOutdoors],
        2.0,
        4,
        Full,
        Action::Figure(FigureWhere::Behind, 20.0, 32.0),
        false,
    ),
    row(
        "on_the_roof",
        3,
        5,
        &[Home, TargetOutdoors, Dawn],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::Roof, 15.0, 12.0),
        false,
    ),
    row(
        "across_water",
        3,
        5,
        &[Around("water"), TargetOutdoors],
        2.0,
        4,
        Full,
        Action::Figure(FigureWhere::AcrossWater, 25.0, 24.0),
        false,
    ),
    row(
        "in_doorway",
        4,
        5,
        &[Door, TargetIndoors],
        2.0,
        4,
        Full,
        Action::Figure(FigureWhere::Doorway, 0.4, 0.0),
        false,
    ),
    row(
        "at_the_chest",
        4,
        5,
        &[TargetApproachingChest],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::AtChest, 1.5, 3.0),
        false,
    ),
    row(
        "in_your_bed",
        4,
        5,
        &[Bed, Night, TargetApproachingDoor],
        4.0,
        1,
        Full,
        Action::Figure(FigureWhere::InBed, 1.5, 3.0),
        false,
    ),
    row(
        "replay",
        4,
        5,
        &[Route, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::Route, 40.0, 24.0),
        false,
    ),
    // Water, snow and fields, from the surroundings
    row(
        "boat_moved",
        2,
        5,
        &[Vehicle, Around("water"), TargetAwayOrAsleep],
        3.0,
        3,
        Full,
        Action::MoveVehicle,
        false,
    ),
    row(
        "boat_adrift",
        3,
        5,
        &[Vehicle, Around("water"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::MoveVehicle,
        false,
    ),
    row(
        "bucket_floor",
        3,
        5,
        &[Home, InChest("water_bucket"), TargetAwayOrAsleep],
        4.0,
        1,
        Full,
        Action::BucketFloor,
        false,
    ),
    row(
        "snow_cleared",
        2,
        5,
        &[Door, Around("snow"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::SnowCleared,
        true,
    ),
    row(
        "trample",
        3,
        5,
        &[Door, Around("crops"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::Trample,
        false,
    ),
    row(
        "harvest_one",
        2,
        5,
        &[UsedContainer, Around("crops"), TargetAwayOrAsleep],
        2.0,
        4,
        Full,
        Action::HarvestOne,
        true,
    ),
    row(
        "boat_drifts",
        2,
        5,
        &[Vehicle, Around("water"), TargetOutdoors],
        2.0,
        4,
        Half,
        Action::PushVehicle("boat"),
        true,
    ),
    row(
        "minecart_rolls",
        2,
        5,
        &[Vehicle, TargetOnline],
        2.0,
        4,
        Half,
        Action::PushVehicle("minecart"),
        true,
    ),
    // Mines
    row(
        "mining_sounds",
        2,
        5,
        &[TargetUnderground, TargetOnline],
        1.0,
        6,
        Half,
        Action::MiningSounds,
        true,
    ),
    row(
        "echo_steps",
        2,
        5,
        &[TargetUnderground, TargetWalking],
        1.0,
        6,
        Half,
        Action::SoundAtTarget("minecraft:block.stone.step"),
        true,
    ),
    row(
        "late_eating",
        2,
        5,
        &[TargetAte, TargetOnline],
        1.0,
        6,
        Half,
        Action::SoundAtTarget("minecraft:entity.generic.eat"),
        true,
    ),
    // Villages and redstone
    row(
        "scan_village",
        2,
        5,
        &[Around("village")],
        1.0,
        u32::MAX,
        Half,
        Action::ScanVillage,
        true,
    ),
    row(
        "village_doors",
        3,
        5,
        &[Scanned("door"), Night, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::VillageDoors,
        false,
    ),
    row(
        "lamps_noon",
        3,
        5,
        &[Fixture("redstone_lamp"), Day, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::ToggleFixture("redstone_lamp"),
        false,
    ),
    row(
        "plate_laid",
        3,
        5,
        &[UsedContainer, InChest("pressure_plate"), TargetAwayOrAsleep],
        4.0,
        1,
        Full,
        Action::PlateLaid,
        false,
    ),
    row(
        "door_opens_itself",
        3,
        5,
        &[TargetApproachingDoor],
        2.0,
        3,
        Full,
        Action::DoorAhead,
        false,
    ),
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
        Action::SoundFar("minecraft:entity.wolf_sad.whine"),
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
            "animal_gone" => "lead one of their animals to the end of their path",
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
            "hostiles_stare" => "have the monsters stand and stare at them all night",
            "answers_you" => "answer what they typed",
            "stairs_turned" => "turn a stair block another way",
            "banner_turned" => "turn the banner",
            "frame_emptied" => "take the thing out of the frame and leave it on the bed",
            "frame_swap" => "swap the frame's thing for one from the chest",
            "armor_stand_dress" => "dress the stand in a piece of their armour",
            "tool_used" => "wear their tool down as if used",
            "cold_furnace" => "take the fuel out of the furnace",
            "open_on_enter" => "open the chest as they walk up to it",
            "tidy_by_colour" => "sort the chest by colour",
            "plant_flower" => "plant one of their flowers by the door",
            "replant_other" => "replace a crop with another",
            "ice_path" => "freeze a path across the water toward the house",
            "bed_occupied" => "tell them the bed is occupied",
            "woke_elsewhere" => "wake them a block from the bed",
            "pet_outside" => "put the pet outside the door",
            "tab_entry_stays" => "show their name twice all session",
            "note_tune" => "three notes from the note block",
            "piston_once" => "work the piston once",
            "disc_phantom" => "play a disc they do not own",
            "fog_close" => "close the fog in for a minute",
            "snow_indoors" => "let it snow indoors for a moment",
            "rain_indoors" => "let it rain indoors for a moment",
            "scan_ore" => "look at the rock around them",
            "ore_marked" => "put a torch on an ore they have not found",
            "wall_swap" => "change one block of the house for its cousin",
            "torch_blue" => "make one of their torches burn blue",
            "window_bars" => "turn a window pane to iron bars",
            "bed_breaks" => "break the bed under them as they lie down",
            "far_figure" => "stand far off where they can see it",
            "behind_you" => "stand far behind them on their path",
            "on_the_roof" => "stand on their roof at dawn",
            "across_water" => "stand on the far shore",
            "in_doorway" => "be in the doorway for less than a second",
            "at_the_chest" => "be at their chest as they come in",
            "in_your_bed" => "lie in their bed as they come home",
            "replay" => "walk their path as they walked it",
            "boat_moved" => "move their boat to the other shore",
            "boat_adrift" => "set their boat adrift",
            "boat_drifts" => "let their boat drift off on its own",
            "minecart_rolls" => "let their minecart roll on its own",
            "bucket_floor" => "leave a block of water in the floor",
            "snow_cleared" => "clear the snow from the path",
            "trample" => "trample the field toward the door",
            "harvest_one" => "harvest one crop into the chest",
            "mining_sounds" => "a pickaxe in the rock near them",
            "echo_steps" => "footsteps answering theirs in the tunnel",
            "late_eating" => "one more bite after theirs",
            "scan_village" => "look over the village",
            "village_doors" => "open every door in the village",
            "lamps_noon" => "turn the lamps on at noon",
            "plate_laid" => "lay a pressure plate before the chest",
            "door_opens_itself" => "open the door as they reach it",
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
            "wolf_far" => "a wolf whining far off",
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

    /// Every sound the mind names exists in the game: a renamed sound fails here, not in a
    /// player's world (`entity.wolf.howl` was gone in 26.3, 2026-11-09). The list is the game's
    /// own `sounds.json`, regenerated on a version bump (`docs/upgrading.md`).
    #[test]
    fn every_sound_the_mind_names_is_one_the_game_has() {
        let known: std::collections::HashSet<&str> = include_str!("../tests/data/sounds-26.3.txt")
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .collect();
        let sources = [include_str!("events.rs"), include_str!("brain/act.rs")];
        let categories = [
            "block.",
            "entity.",
            "ambient.",
            "item.",
            "music.",
            "weather.",
            "event.",
            "ui.",
            "enchant.",
            "particle.",
            "random.",
        ];
        let mut unknown = Vec::new();
        for source in sources {
            for (i, _) in source.match_indices("\"minecraft:") {
                let rest = &source[i + 1..];
                let id = &rest[..rest.find('"').unwrap_or(rest.len())];
                let name = &id["minecraft:".len()..];
                if categories.iter().any(|c| name.starts_with(c)) && !known.contains(id) {
                    unknown.push(id.to_string());
                }
            }
        }
        unknown.sort();
        unknown.dedup();

        assert!(unknown.is_empty(), "sounds 26.3 does not have: {unknown:?}");
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
