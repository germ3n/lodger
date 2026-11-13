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
    /// It holds at least this many of a block from its own digging.
    Stored(&'static str, u32),
    /// A plant grows around the home: `flower` for any flower, else a block id's name.
    Plant(&'static str),
    /// A ladder of four rungs or more in the home.
    Ladder,
    /// A hole the target dug near the home, and something it dug to fill it with.
    Hole,
    /// The target just used a door: met only by that, never on the menu.
    DoorUsed,
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
    /// The target just opened a chest of theirs: met only by that, never on the menu.
    JustOpened,
    /// Fresh loot: armed by the brain and given by the host, never on the menu.
    LootOpened,
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
    /// A villager within 32 blocks of the target.
    Villagers,
    /// The used container holds an item whose id ends with any of these.
    InChestAny(&'static [&'static str]),
    /// The target is walking toward the used chest, within six blocks.
    TargetApproachingChest,
    /// Six or more dyed stacks in the used chest.
    ColouredStacks,
    /// A damageable favourite tool sits in a chest.
    ToolInContainer,
    /// A cave near the home the target has not entered, from the cave read (`crate::caves`).
    Cave,
    /// A place they died that no light of its own marks yet.
    UnmarkedDeath,
    /// The light they placed last, and a way on from it along where they went.
    LastLight,
    /// Lights of theirs behind them as they go, out of view (`crate::lights`).
    LightsBehind,
    /// A light of theirs with a soul fire form, out of view.
    SoulableLight,
    /// An animal to warp, theirs first, 12 to 32 blocks from the target (`crate::warped`).
    WarpableAnimal,
    /// The target just woke; never offered, the brain starts it (`foot_of_bed`).
    JustWoke,
    /// A field of 7 by 5 or more that a word fits in (`crate::farm`).
    FieldWord,
    /// A piece of armour in their chest, and an armour stand or one in the chest.
    ScarecrowParts,
    /// A beehive of theirs, and a door.
    Hive,
    /// A horse of theirs tied to a post.
    LeashedHorse,
    /// This many of their farm animals in the pens by the home.
    Penned(usize),
    /// Two chickens or more by the home, and a door to lay eggs toward.
    Coop,
    /// Farm animals within 16 blocks of the target.
    AnimalsNearTarget,
    /// A sign of theirs (one they placed).
    TheirSign,
    /// They just killed one of the kind its kill sign counts; never offered, the kill starts it.
    KilledNow,
    /// The weapon they use most in their chest, and an item frame of theirs.
    WeaponAndFrame,
    /// They kill most a kind that has a head (a zombie, a skeleton, a creeper, a piglin).
    HeadKind,
    /// Two kinds or more killed.
    KillKinds,
    /// The target in the game and awake.
    TargetAwake,
    /// A music disc in the chest they use.
    Disc,
    /// Their furnace's fuel and another fuel in their chest (`crate::workshop`).
    SwapFuel,
    /// A brewing ingredient in the chest they use.
    BrewIngredient,
    /// The makings of something in the chest they use.
    Makings,
    /// A boat of theirs near the home.
    TheirBoat,
    /// Its diary went on sale and they never bought it (`diary_brought`).
    DiaryUnbought,
    /// A villager near them no one has named (`villager_named`).
    UnnamedVillager,
    /// One of their farm animals by the house, out of their view, of this kind (any, empty).
    TheirAnimal(&'static str),
    /// One of them with no name (`named_animal`).
    TheirUnnamedAnimal,
    /// Water within this many blocks of the target.
    WaterNear(u32),
    /// The target holds this item in their main hand.
    Holding(&'static str),
    /// The target this many blocks or more from the village, or away from the game.
    FarFromVillage(u32),
    /// The target just rang a bell; never offered, the ringing starts it.
    RangBell,
    /// A tunnel of theirs of ten columns or more (`crate::tunnels`).
    Tunnel,
    /// The target in one of their tunnels.
    InTunnel,
    /// Unfound ore ahead of their newest tunnel.
    OreAhead,
    /// Ten obsidian or more they placed: a frame.
    Frame,
    /// Their lights in the tunnel they stand in, two or more.
    TunnelLights,
    /// Room behind them in their tunnel to close.
    BehindThem,
    /// Something left for this event to build tonight (`crate::building`).
    BuildLeft(&'static str),
    /// Twenty building blocks or more of theirs: a house to copy.
    Walls,
    /// A light of theirs within ten blocks of the target.
    LightNear,
    /// Noon: the sun high, from a quarter to an hour past midday.
    Noon,
    /// The target has just crouched still ten seconds; never offered, that starts it (`breath`).
    CrouchedStill,
    /// Three or more of their farm animals within sixteen blocks of the target.
    Herd,
    /// The target just knocked on a door; never offered, the knock starts it.
    Knocked,
    /// An item frame in the house, and something to show in every one: what it holds, or
    /// their favourite.
    FramesToShow,
    /// The target in the Nether: met only there (`crate::elsewhere`), never in the overworld.
    InNether,
    /// The target in the End, an enderman near them: met only there.
    InEnd,
    /// The target has just come back from another dimension, a while gone; never offered.
    CameBack,
    /// It holds nothing it took: one thing at a time (until now a second take lost the
    /// first).
    HoldsNothing,
    /// The target in a boat or raft with a second seat (a chest boat has none).
    TargetInBoat,
    /// A light they have just placed underground: met only then.
    LitUnderground,
    /// Copper of theirs that weathers and is not yet green through (`copper_ages`).
    Copper,
    /// The target riding a mob whose kind ends with this (`horse_balks`).
    Riding(&'static str),
    /// A pet of theirs of this kind near (`cat_gift`).
    PetOf(&'static str),
    /// A young animal near them (`the_child_stares`).
    BabyNear,
    /// A diamond ore they have just broken: met only then (`ore_counted`).
    MinedDiamond,
    /// A mob of this kind near them, a creature or a hostile (the Nether's and the End's).
    MobNear(&'static str),
    /// A chorus fruit they have just eaten: met only then (`chorus_lands_home`).
    AteChorus,
    /// Where they last died is in sight: twelve to sixty-four blocks from them.
    NearTheirDeath,
    /// Down a shaft of their ladders, six blocks or more below its top.
    DownTheShaft,
    /// The target swimming.
    TargetSwimming,
    /// Its house built whole, the target within forty-eight blocks of it and out of it.
    ItsHouseNear,
    /// The target outdoors, looking up at the sky.
    LookingUp,
    /// Its house built whole.
    ItsHouse,
    /// The target under water, their air going.
    TargetUnderwater,
    /// The target outdoors among trees: trunks of the tree read around them
    /// (`Memory::in_the_woods`).
    InTheWoods,
    /// They have run a while at night with a light in hand: met only then.
    RanLit,
    /// A shield in either hand (`blow_on_the_shield`).
    HoldsShield,
    /// Its things taken up have been up there half a day or more (`Memory::taken_up_due`).
    TakenUpDue,
    /// Walking out of the woods (`Memory::leaving_the_woods`).
    LeavingTheWoods,
    /// The biome they stand in lies under snow (`trees::snowy`).
    SnowyHere,
    /// Walking into the village: within forty blocks of its middle and coming nearer, villagers
    /// near them.
    EnteringVillage,
    /// Dusk: the sun going down.
    Dusk,
    /// The target running, past a walk's pace.
    TargetRunning,
    /// No village within 160 blocks of the home.
    NoVillageNear,
    /// A pressure plate, button or lever of theirs at the home.
    RedstoneAtHome,
    /// Underground, three lights of theirs or more within twenty-four blocks.
    TheirCaveLights,
    /// A thunderstorm over the world now.
    Thundering,
    /// The target stands in a biome whose id holds one of these (`desert`, `ocean`).
    InBiome(&'static [&'static str]),
    /// The rain stopped in the last two minutes, and they are out under the sky.
    RainJustEnded,
    /// Gliding on elytra.
    Gliding,
    /// Standing still this many seconds or more, awake and out of bed.
    StoodStill(u32),
    /// The furnace they use, left empty (`furnace_glows`).
    ColdFurnace,
    /// The target's feet below this height (`knock_from_below` at the bedrock).
    Below(i32),
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
                | Need::Riding(_)
                | Need::PetOf(_)
                | Need::BabyNear
                | Need::MinedDiamond
                | Need::MobNear(_)
                | Need::AteChorus
                | Need::TargetInBed
                | Need::TargetSaid
                | Need::Heard
                | Need::JustOpened
                | Need::LootOpened
                | Need::DoorUsed
                | Need::Night
                | Need::Day
                | Need::Dawn
                | Need::Raining
                | Need::Thundering
                | Need::InBiome(_)
                | Need::RainJustEnded
                | Need::Gliding
                | Need::StoodStill(_)
                | Need::Below(_)
                | Need::NotRaining
                | Need::Creature(_)
                | Need::TheirAnimal(_)
                | Need::TheirUnnamedAnimal
                | Need::UnnamedVillager
                | Need::Pet
                | Need::Animals
                | Need::Hostiles
                | Need::TargetUnderground
                | Need::LightsBehind
                | Need::WarpableAnimal
                | Need::JustWoke
                | Need::LeashedHorse
                | Need::Penned(_)
                | Need::Coop
                | Need::AnimalsNearTarget
                | Need::KilledNow
                | Need::TargetAwake
                | Need::WaterNear(_)
                | Need::Holding(_)
                | Need::HoldsShield
                | Need::FarFromVillage(_)
                | Need::RangBell
                | Need::InTunnel
                | Need::Villagers
                | Need::LightNear
                | Need::Noon
                | Need::CrouchedStill
                | Need::Herd
                | Need::Knocked
                | Need::SoulableLight
                | Need::LastLight
                | Need::InNether
                | Need::InEnd
                | Need::CameBack
                | Need::TunnelLights
                | Need::BehindThem
                | Need::TargetAte
                | Need::TargetApproachingDoor
                | Need::TargetApproachingChest
                | Need::TargetInBoat
                | Need::LitUnderground
                | Need::NearTheirDeath
                | Need::DownTheShaft
                | Need::TargetSwimming
                | Need::ItsHouseNear
                | Need::LookingUp
                | Need::TargetUnderwater
                | Need::InTheWoods
                | Need::RanLit
                | Need::TakenUpDue
                | Need::LeavingTheWoods
                | Need::SnowyHere
                | Need::EnteringVillage
                | Need::Dusk
                | Need::TargetRunning
                | Need::NoVillageNear
                | Need::RedstoneAtHome
                | Need::TheirCaveLights
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
            Need::Stored(id, n) => {
                format!("fewer than {n} {} dug", id.rsplit(':').next().unwrap_or(id))
            }
            Need::Plant(kind) => format!("no {kind} around the home"),
            Need::Ladder => "no ladder of four rungs".to_string(),
            Need::Hole => "no hole of theirs near the home, or nothing dug to fill it".to_string(),
            Need::DoorUsed => "happens only as they open a door".to_string(),
            Need::Furnace => "no furnace used".to_string(),
            Need::ColdFurnace => "no furnace of theirs left empty".to_string(),
            Need::Below(y) => format!("not below {y}"),
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
            Need::JustOpened => "happens only as they open a chest".to_string(),
            Need::LootOpened => "armed for the next fresh loot they open".to_string(),
            Need::Night => "not night".to_string(),
            Need::Day => "not day".to_string(),
            Need::Raining => "not raining".to_string(),
            Need::Thundering => "no thunderstorm".to_string(),
            Need::InBiome(kinds) => format!("not in {}", kinds.join(" or ").replace('_', " ")),
            Need::RainJustEnded => "no rain just ended".to_string(),
            Need::Gliding => "not gliding".to_string(),
            Need::StoodStill(seconds) => format!("not standing still {seconds} seconds"),
            Need::NotRaining => "raining".to_string(),
            Need::Route => "no route".to_string(),
            Need::Field => "no field".to_string(),
            Need::Animals => "fewer than three animals near".to_string(),
            Need::Hostiles => "fewer than three hostile mobs near".to_string(),
            Need::Kills => "fewer than three kills of one kind".to_string(),
            Need::Around(kind) => format!("no {} around the house", kind.replace('_', " ")),
            Need::Vehicle => "no boat or minecart near".to_string(),
            Need::TargetUnderground => "the target is not underground".to_string(),
            Need::Cave => "no cave near that they have not been in".to_string(),
            Need::UnmarkedDeath => "no death of theirs left unmarked".to_string(),
            Need::LastLight => "no light of theirs with a way on from it".to_string(),
            Need::LightsBehind => "no light of theirs behind them".to_string(),
            Need::SoulableLight => "no torch or lantern of theirs out of view".to_string(),
            Need::WarpableAnimal => "no animal 12 to 32 blocks from them".to_string(),
            Need::JustWoke => "happens only as they wake".to_string(),
            Need::FieldWord => "no field of 7 by 5 to spell in".to_string(),
            Need::ScarecrowParts => "no armour, or no armour stand, of theirs".to_string(),
            Need::Hive => "no beehive of theirs".to_string(),
            Need::LeashedHorse => "no horse of theirs on a lead".to_string(),
            Need::Penned(n) => format!("fewer than {n} animals in their pens"),
            Need::Coop => "no coop by the home".to_string(),
            Need::AnimalsNearTarget => "no farm animal near them".to_string(),
            Need::TheirSign => "no sign of theirs".to_string(),
            Need::KilledNow => "happens only as they kill".to_string(),
            Need::WeaponAndFrame => "no weapon of theirs in a chest, or no frame".to_string(),
            Need::HeadKind => "they kill nothing that has a head".to_string(),
            Need::KillKinds => "fewer than two kinds killed".to_string(),
            Need::TargetAwake => "the target is away or asleep".to_string(),
            Need::Disc => "no music disc in their chest".to_string(),
            Need::SwapFuel => "no fuel in their furnace, or no other fuel in the chest".to_string(),
            Need::BrewIngredient => "no brewing ingredient in their chest".to_string(),
            Need::Makings => "nothing in their chest it could make something of".to_string(),
            Need::TheirBoat => "no boat of theirs".to_string(),
            Need::DiaryUnbought => "its diary not on sale, or bought".to_string(),
            Need::UnnamedVillager => "no villager near them without a name".to_string(),
            Need::TheirAnimal("") => "none of their animals by the house".to_string(),
            Need::TheirUnnamedAnimal => {
                "none of their animals by the house without a name".to_string()
            }
            Need::TheirAnimal(kind) => format!("no {kind} of theirs by the house"),
            Need::WaterNear(n) => format!("no water within {n} blocks of them"),
            Need::Holding(id) => format!("they do not hold a {}", crate::memory::bare(id)),
            Need::FarFromVillage(n) => format!("the target is within {n} blocks of the village"),
            Need::RangBell => "happens only as they ring a bell".to_string(),
            Need::Villagers => "no villager near them".to_string(),
            Need::Tunnel => "no tunnel of theirs of ten blocks".to_string(),
            Need::InTunnel => "the target is not in a tunnel of theirs".to_string(),
            Need::OreAhead => "no ore ahead of their tunnel".to_string(),
            Need::Frame => "no portal frame of theirs".to_string(),
            Need::TunnelLights => "fewer than two lights in their tunnel".to_string(),
            Need::BehindThem => "no room behind them in their tunnel".to_string(),
            Need::BuildLeft(e) => format!("nothing left of {e} to build"),
            Need::Walls => "fewer than twenty blocks of their building".to_string(),
            Need::LightNear => "no light of theirs near them".to_string(),
            Need::Noon => "not noon".to_string(),
            Need::CrouchedStill => "happens only as they crouch still".to_string(),
            Need::Herd => "fewer than three farm animals near them".to_string(),
            Need::Knocked => "happens only as they knock on a door".to_string(),
            Need::FramesToShow => "no item frame, or nothing to show in one".to_string(),
            Need::InNether => "happens only in the Nether".to_string(),
            Need::InEnd => "happens only in the End, by an enderman".to_string(),
            Need::CameBack => "happens only as they come back from another dimension".to_string(),
            Need::HoldsNothing => "it holds something already".to_string(),
            Need::TargetInBoat => "the target is not in a boat with a seat to spare".to_string(),
            Need::LitUnderground => "happens only as they place a light underground".to_string(),
            Need::Copper => "no copper of theirs left to weather".to_string(),
            Need::Riding(kind) => format!("they ride no {}", kind.replace('_', " ")),
            Need::PetOf(kind) => format!("no {} of theirs near", kind.replace('_', " ")),
            Need::BabyNear => "no young animal near them".to_string(),
            Need::MinedDiamond => "happens only as they mine a diamond".to_string(),
            Need::MobNear(kind) => format!("no {} near them", kind.replace('_', " ")),
            Need::AteChorus => "happens only as they eat a chorus fruit".to_string(),
            Need::NearTheirDeath => "they are not near where they last died".to_string(),
            Need::DownTheShaft => "they are not down a shaft of their ladders".to_string(),
            Need::TargetSwimming => "the target is not swimming".to_string(),
            Need::ItsHouseNear => "its house is not built, or they are not near it".to_string(),
            Need::LookingUp => "the target is not looking up at the sky".to_string(),
            Need::ItsHouse => "its house is not built".to_string(),
            Need::TargetUnderwater => "the target is not under water".to_string(),
            Need::InTheWoods => "the target is not among trees".to_string(),
            Need::RanLit => "happens only as they run about at night with a light".to_string(),
            Need::HoldsShield => "no shield in hand".to_string(),
            Need::TakenUpDue => "nothing of theirs up there long enough".to_string(),
            Need::LeavingTheWoods => "the target is not walking out of the woods".to_string(),
            Need::SnowyHere => "no snow where they are".to_string(),
            Need::EnteringVillage => "the target is not walking into a village".to_string(),
            Need::Dusk => "not dusk".to_string(),
            Need::TargetRunning => "the target is not running".to_string(),
            Need::NoVillageNear => "a village is near the home".to_string(),
            Need::TheirCaveLights => {
                "fewer than three lights of theirs near them underground".to_string()
            }
            Need::RedstoneAtHome => "no plate, button or lever of theirs at the home".to_string(),
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
    /// What it took comes back worn a little, or with a name on it.
    ReturnUsed,
    /// The book and quill in their chest is signed with one word, by no one.
    BookWord,
    /// What waits in their hopper is in the chest it feeds.
    HopperEmptied,
    /// Their shulker box is on another wall of the room, with all it holds.
    ShulkerMoved,
    /// Every empty slot of their chest holds one dirt it dug.
    FullOfOne,
    /// Their map shows a mark at its place.
    MapMarked,
    /// One of their things in the chest gets a name.
    RenameOne,
    /// Another chest of the house closes as they open theirs.
    ChestAnswers,
    /// Something of its own in the loot of a chest they open for the first time.
    LootGift,
    /// A door hung on the other side of its frame.
    DoorHinge,
    /// One rung of a ladder missing, on top of the ladder.
    LadderGap,
    /// The flowers around the house in its pots by the door.
    FlowersPotted,
    /// A pumpkin of theirs carved, facing the door.
    PumpkinFace,
    /// The sapling by the house a tree overnight, their bone meal spent.
    TreeGrown,
    /// A hole they dug near the house filled in again.
    Backfill,
    /// A hollow dug into the stone behind a wall of the house, a torch and its things in it.
    Alcove,
    /// A work block of theirs one block over, contents and all.
    ShiftFurniture,
    /// Every torch of one room taken down and put in their chest.
    RoomDark,
    /// The door they open shuts again a moment later.
    DoorHeld,
    /// Their bed turned round where it stands.
    BedTurned,
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
    /// One of their lanterns far down a cave they have not entered.
    LanternAhead,
    /// Torches down a cave they have not entered, at their own spacing.
    CaveLit,
    /// A lit candle where they last died.
    DeathLight,
    /// The light they placed last, a few blocks on along the way they went.
    LightTaken,
    /// Their lights behind them go out one by one, back a minute later.
    LightsOut,
    /// A light of theirs turned to soul fire.
    SoulSwap,
    /// An animal warped for a minute: too tall, upside down, creeping, its head on them.
    WarpAnimal,
    /// Their bed one block to its side.
    ShiftBed,
    /// Their bed another colour.
    SheetsChanged,
    /// A second bed beside theirs.
    SecondBed,
    /// An armour stand in the field wearing a piece of their armour, facing the house.
    Scarecrow,
    /// The bees out around the door at night.
    BeesOut,
    /// Their horse tied to another post.
    HorseMoved,
    /// The field harvested into a word.
    FieldWord,
    /// Their animals stand facing the house.
    FaceTheHouse,
    /// One more of the kind they keep in the pen, till noon.
    OneMore,
    /// One row of the field back to seedlings.
    RowWithered,
    /// Eggs from the coop to the door.
    EggsToDoor,
    /// Animals turn on them: this many, this much harm a hit, this many hits.
    Charge(u32, f32, u32),
    /// One written line of a sign of theirs reads a word of its own.
    SignWord,
    /// The kill sign's count goes up as they kill.
    CountUp,
    /// Their weapon in a frame of theirs.
    WeaponOnWall,
    /// A head on a post by the door.
    HeadOnPost,
    /// A grave for each kind they killed.
    GraveEach,
    /// A death message about them while they live.
    DeathMessage,
    /// What they typed, on a sign by the door.
    WordsOnSign,
    /// Their disc playing in their jukebox, or the one playing stopped.
    JukeboxDisc,
    /// The lectern's book open to another page.
    LecternPage,
    /// Their furnace's fuel another kind.
    SwapFuel,
    /// Their brewing stand given an ingredient.
    Brewing,
    /// An enchantment gone from an item in their chest.
    GrindstoneStrip,
    /// Their lamps blink a pattern.
    LampsBlink,
    /// The composter a level fuller.
    ComposterFills,
    /// Something they make, made, on the crafting table.
    CraftedForYou,
    /// A bell of theirs rings once.
    BellRings,
    /// Footprints in the snow from the door to its place.
    Trail,
    /// Footprints in the snow to the door, none away.
    TrailIn,
    /// Their boat at the shore they walk toward.
    BoatWaiting,
    /// A bobber's splashes and catches on the water, no one there.
    FishingLine,
    /// Two snow blocks and a pumpkin by the door.
    Snowman,
    /// Their boat in the middle of the house.
    BoatInHouse,
    /// A shape under the water, still, facing them.
    UnderWater,
    /// Wet steps from the door to the bed.
    WetSteps,
    /// Their own name twice in the player list for a few seconds.
    TabTwice,
    /// A village lantern on their route.
    VillageLantern,
    /// What it took, for sale at a villager for an emerald.
    VillagerWord,
    /// The village empty, every villager in one house, until dawn.
    VillageAbandoned,
    /// The bell they rang rings again.
    BellTwice,
    /// A copy of their newest tunnel beside it.
    MineBeside,
    /// Their oldest tunnel's mouth walled.
    TunnelSealed,
    /// Digging toward them as they dig.
    TunnelToward,
    /// Their portal frame lit.
    PortalLit,
    /// The ore ahead of their tunnel mined out.
    MinedOut,
    /// A shaft down, their ladders in it.
    LadderDown,
    /// Their tunnel five blocks longer.
    TunnelGoesOn,
    /// Their tunnel's torches out from the far end toward them.
    TorchesFarEnd,
    /// A sign at the end of their tunnel.
    SignDeep,
    /// Gravel behind them in their tunnel.
    WayBackCloses,
    /// The figure right behind them swings once, this much harm (never below half a heart).
    FigureSwing(f32),
    /// The far figure raises its arm once.
    FigureWave,
    /// Tonight's share of a build, by the event.
    Build(&'static str),
    /// A room dug in a cave, a door, a chest holding what it took.
    CaveRoom,
    /// The figure digs into the rock ahead of them, the stone closing behind; with true, out
    /// of the hill's far side a minute later.
    DigInto(bool),
    /// The figure at the edge of their light steps back out of it as the light flickers.
    IntoTheDark,
    /// Their view turned two degrees and a step right behind them.
    HandOnShoulder,
    /// A second set of steps a moment behind theirs, stopping when they stop.
    Footsteps,
    /// The farm animals near them walk toward them together and stop a few blocks off.
    Pack,
    /// One animal near them, sometimes bent, comes at them at this pace against its own walk
    /// (0.35 very slowly, 1.5 at a run), then stares, leaves or strikes once.
    MobComes(f32),
    /// The hostile mobs near hold and stare, a step closer each time they look away, back where
    /// they stood when looked at.
    HostilesClose,
    /// A push at the next edge or ladder they stand on, never a fall of more than three.
    Shove(u32),
    /// Every frame of the house shows the one thing, to them alone, for a while.
    FramesShow,
    /// Their bobber pulled under and kept under, nothing on it.
    BobberUnder,
    /// In the Nether, the ghasts' cries in the subtitles are their name.
    GhastName,
    /// In the End, an enderman near them holds the block they build with most.
    EnderHolds,
    /// Coming back from another dimension, one of its bold events happened while they were
    /// gone: the brain picks which and runs it at once (`Brain::consider_back`).
    WhileAway,
    /// From the dark, where the figure stands with a bow, a real arrow: beside their head with
    /// 0, at them with this much harm (never below half a heart).
    Shoot(f32),
    /// A splash potion from the dark at their feet: darkness, slowness, blindness or a moment's
    /// levitation.
    SplashDark,
    /// A potion of its brewing in their chest, named `for the night`.
    BrewLeft,
    /// One of their potions another kind, never a harmful one, its name as it was.
    PotionChanged,
    /// A lingering cloud at their door at dawn, gone as they step out.
    DoorCloud,
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
    /// Its diary of them on sale from a villager near them, signed with their name (owner).
    DiarySold,
    /// The diary not bought: a villager walks up and drops a newer one at their feet.
    DiaryBrought,
    /// A letter from it in the chest they use, for them.
    LetterInChest,
    /// The villager near them named, in a word or two it chose.
    VillagerNamed,
    /// Its notes of them on a lectern, a page a day.
    LecternNotes,
    /// One page more in a book and quill of theirs, in its words.
    PageInBook,
    /// They wake in a copy of their house in the void dimension, the figure in its doorway,
    /// for a minute; then in their own bed, the door open (owner).
    OtherHouse,
    /// The figure in the target's skin, somewhere, for some seconds, gone when approached.
    Figure(FigureWhere, f32, f32),
    /// One building block of the house becomes its cousin: planks another wood, cobblestone
    /// mossy, a pane iron bars, a torch blue. Matched by id suffix: all, or lights, or panes.
    WallSwap(&'static str),
    /// The bed breaks under them as they lie down; its item drops where it stood.
    BedBreaks,
    /// Asleep, their door opens and is left open, and a figure walks in from it to the bed's
    /// side, takes out a lead and chokes them for a few seconds, this much harm a squeeze, never
    /// below half a heart.
    Choke(f32),
    /// The same walk to the bed's foot, a crossbow loaded there instead and a shot from there:
    /// this much harm, which wakes them, never below half a heart.
    ShotInBed(f32),
    /// The same walk, looking at a flint and steel at the bed's foot, then round to the pillow to
    /// strike it at them: they wake burning, never below half a heart.
    BurnedInBed,
    /// At night, figures in a ring round the house, this many, each gone once seen and looked
    /// away from.
    RingOfFigures(u32),
    /// As they wake, the figure hangs feet up from the ceiling over the bed.
    UpsideDown,
    /// At night, a figure crawls on its belly from their door to their bed while they are in.
    Crawl,
    /// At night, outdoors: far ahead in the dark a figure runs at them, gone just short.
    Hunted,
    /// In their boat, a figure in the seat behind them.
    BoatPassenger,
    /// At night, a figure outside every window of the house at once, all gone together.
    EveryWindow,
    /// At the edge of their light, one figure more each night it comes.
    OneMoreEachNight,
    /// The door they walk to will not open a few seconds, held from the other side; a knock.
    DoorWontOpen,
    /// Asleep: they wake standing outside their front door, the door open behind them, their
    /// prints in the snow from the bed.
    Sleepwalk,
    /// The light they just placed underground shows a figure in front of it, for a blink.
    TorchFlash,
    /// Where they last died, a figure lies as a body would.
    WhereYouDied,
    /// Walking, they are set back a few blocks, and a figure stands where they were.
    Rubberband,
    /// In their doorway, a figure holds out what it took; walk to it and it is at their feet.
    TheOffer,
    /// Down their shaft, a figure at its top looking down at them.
    TopOfTheLadder,
    /// Swimming, they are pulled under a moment, a drowning gasp, and let go.
    PulledUnder,
    /// At night, its house goes dark, then its lights come on one by one, then out.
    ItsLightsOn,
    /// Their chest's grid laid out as a face: two eyes and a mouth of single items.
    FaceInTheChest,
    /// Looking up at night, a figure falls from high above and lands far off, gone there.
    FallsFromTheSky,
    /// Their pet walks off to a figure ahead of them; the figure is gone, the pet comes back.
    PetFollows,
    /// Asleep, a figure comes to the bed and lies down on the floor beside it, facing them.
    LiesBeside,
    /// A bed laid in its house, quietly made their respawn point.
    RespawnItsBed,
    /// A note of its own in their ender chest.
    EnderChestNote,
    /// The next eye of ender they throw flies toward their house.
    EyeTurnsHome,
    /// Far ahead, a figure walks across their way, left to right, never looking at them.
    CrossingAhead,
    /// Along their route at dusk, figures in single file facing the house, each gone as passed.
    InALine,
    /// Far off, a figure's head alone above the ground; it sinks out of sight once seen.
    PeeksFromGround,
    /// At night a zombie near them wears their own head and walks at them.
    WearsYourFace,
    /// One of their animals follows them at a distance all day, stopping when they stop.
    AnimalFollows,
    /// Under water, their air runs out twice as fast, once.
    HeldBreath,
    /// Every animal and mob around them stops at once, a few seconds, facing them.
    AllFreeze,
    /// On the map they hold, a player's marker at their house while they are away.
    FigureOnMap,
    /// Back through their portal, a figure beside it facing it, as if it had waited.
    WaitsAtThePortal,
    /// At night a firework over its house, bursting as a face.
    FireworkOverItsHouse,
    /// A chicken near them lays what it took instead of an egg.
    ChickenLays,
    /// Coming home, a figure at their crafting table, working at it.
    AtTheirTable,
    /// Far off in the open, a figure jumping in place, facing them.
    JumpingFar,
    /// Coming home, a figure inside the house waves at them from a window.
    WavesFromInside,
    /// Prints in the snow from their door toward the nearest cave.
    PrintsToTheCave,
    /// Their horse far off with a figure in its saddle.
    RidesTheirHorse,
    /// Coming home, a figure crouched at their chest, eating their food.
    EatsAtTheirChest,
    /// In their own window, a figure copying their every move.
    MirrorInTheWindow,
    /// At night, inside, every door of the house opens at once.
    EveryDoorOpens,
    /// Their flowers dead bushes for a day, then flowers again.
    FlowersWilted,
    /// At night a campfire with log seats out where they walk, a figure by it; gone by morning.
    CampNearby,
    /// Overnight, a ring of stone pillars far off.
    StoneCircle,
    /// What they hold slips from their hand to their feet.
    SlipsFromHand,
    /// Their pet stands facing a wall of the house, still, a minute.
    PetStaresAtWall,
    /// At night, bats fluttering about inside the house a minute.
    BatsInTheHouse,
    /// A blink of black, and an hour of the day gone.
    LostHour,
    /// The chest they walk to will not open a few seconds; something knocks inside.
    ChestWontOpen,
    /// Their name trodden into the snow near the house.
    NameInTheSnow,
    /// Overnight, a ring of torches round the house.
    TorchRing,
    /// A line of single blocks of theirs from their door off into the dark.
    Breadcrumbs,
    /// Where they fish, a figure fishing.
    FishingFigure,
    /// Coming in at night, a figure beside their bed, facing it.
    ByTheBed,
    /// Swimming, a figure swims past beneath them.
    SwimsBeneath,
    /// In the rain, a figure in the open, its face to the sky.
    StandsInTheRain,
    /// A black banner of its own on its house's front.
    ItsBanner,
    /// Their chest's lid opens and shuts five times fast, then is still.
    ChestRattles,
    /// A trapdoor of theirs taps open and shut, again and again.
    TrapdoorTaps,
    /// In their shaft, a figure climbs ahead of them, above, and is gone at the top.
    ClimbsAhead,
    /// Far off, a figure standing on the water's surface.
    OnTheWater,
    /// Far off, a figure standing on top of the trees.
    InTheTreetops,
    /// Overnight, a straight line cut through the leaves of a forest, pointing at their house.
    PathThroughTheTrees,
    /// A minecart of theirs rolls past with a figure sitting in it.
    RidesTheCart,
    /// At night far off, a lantern moving, held by a figure walking.
    LanternInTheWoods,
    /// A figure crouched near them, gone as they come round to it.
    CrouchedBehindATree,
    /// In the woods, a figure peeking out from behind a trunk ahead, slipping back once seen.
    PeeksFromTheTrees,
    /// A figure behind them that comes on only while unseen, and strikes for this at the end.
    WhileUnseen(f32),
    /// Far off, a figure drawn to the light they run with, lost when they hold none.
    DrawnToTheLight,
    /// In the woods, a figure behind a trunk one tree nearer each time they look back.
    TreeToTree,
    /// Knocks on a trunk ahead, three at a time, then farther in; the last tree's bark stripped.
    KnocksOnWood,
    /// A trail of blazes on the trunks ahead of them, a strip of bark gone from each.
    BlazedTrees,
    /// A branch cracks in the tree they just passed, its leaves torn.
    BranchSnaps,
    /// Walking out of the woods, figures between the trees behind them, gone together.
    TheTreeLine,
    /// Deep in the woods, a full-grown tree where they came in.
    TreesCloseBehind,
    /// A tree beside them comes down across their way.
    TreeComesDown,
    /// The animals run out of the woods past them; far back, a figure walking.
    AnimalsFleeTheWoods,
    /// In snowy woods, footprints round and round one trunk.
    CirclesATree,
    /// Coming into the village, every villager runs from them; the bell rings.
    VillagersFleeYou,
    /// A day of every villager refusing them, a shake of the head and a no.
    TradesRefused,
    /// A villager turning round and round where it stands, till they come near.
    VillagerSpins,
    /// At dusk the villagers walk out of the village single file, toward its place.
    VillagersWalkOut,
    /// A villager drops dead on their screen, and is up again a minute on.
    VillagerCollapses,
    /// A block of their wall breaks and silverfish pour out at them.
    WallBursts,
    /// At noon a phantom out of the sun swoops at them once, burning, and is gone.
    DaylightPhantom,
    /// Underground, a cave spider drops from the ceiling right before their face.
    DropsFromTheCeiling,
    /// At night, wild wolves circle them, nearer each round, then run.
    WolvesCircle,
    /// At night a zombie walks at them in armour like theirs, a tool like theirs in its hand.
    ZombieInYourGear(f32),
    /// Vexes out of nowhere at them: this many, each hit this much harm, never a kill.
    Vexes(u32, f32),
    /// A creeper right behind them that always goes off, and harms no one.
    CreeperBehind,
    /// Every wall torch of the house turned on its block to point at their bed.
    TorchesTurn,
    /// A boss bar named for it at the top of their screen, filling a minute.
    BossBarIt,
    /// A copper golem standing outside their door at night, facing in.
    CopperGolemAtDoor,
    /// A creaking in their field at night: still while watched, nearer while not.
    CreakingInTheField,
    /// Pale moss creeping across their floor from a corner, more each night.
    PaleMossCreeps,
    /// One window of theirs tinted black a day: from inside, the night is black.
    WindowTinted,
    /// The big dripleaves near them tipped one after another, as if walked across.
    DripleafSteps,
    /// Every axolotl near them floating still at once, playing dead.
    AxolotlsPlayDead,
    /// Sculk veins over the walls of their house low down, more each night.
    SculkCreeps,
    /// Cracks spreading over a block of their wall, as if broken from the other side, then gone.
    CracksInTheWall,
    /// A charged respawn anchor at its place, glowing, on their screen alone (no anchor made).
    AnchorCharged,
    /// Pointed dripstone falling from the ceiling right beside them, shattering; it never hits.
    DripstoneFalls,
    /// A wither rose in one of their flower pots as they wake, on their screen alone a while.
    WitherRoseInPot,
    /// Mushrooms grown overnight on the floor of their house along its walls, the corners first, gone in a day.
    MushroomsInCorners,
    /// Their shulker box opening a crack by itself, a moment, and shutting.
    ShulkerLidPeeks,
    /// A sign by their door with three numbers on it, a line each: where their bed is.
    SignWithCoords,
    /// A trident upright in the ground at its place, as if thrown from the sky, never theirs, gone in a day.
    TridentPlanted,
    /// A mace's smash booming near the house at night, and a small bowl dug in the ground there.
    SlamCrater,
    /// One more step at the foot of their staircase as they wake, down into the floor, on their screen alone a while.
    OneStepMore,
    /// One wall of their house a block further in as they wake, the room smaller, on their screen alone a while.
    RoomClosesIn,
    /// A new trapdoor in their floor over a pit two deep, a torch at its bottom; natural ground alone.
    TrapdoorToNothing,
    /// A ring of fences round their bed by morning, a gap only where they lie, gone in a day.
    BedFenced,
    /// Their tools from their chests laid on the floor in a neat row, the very stacks, kept from despawning.
    ToolsLaidOut,
    /// A column of pointed dripstone grown overnight across their tunnel, floor to ceiling, gone in a day.
    DripstoneInTunnel,
    /// A skeleton skull set into the wall at the far end of the tunnel they are in, facing back along it at them, on their screen alone a while.
    SkullInTheTunnel,
    /// Their copper blocks a stage more weathered overnight, as if years had passed.
    CopperAges,
    /// A skeleton horse among the animals in their pen at night, still, facing them, a while.
    PaleHorse,
    /// A line of evoker fangs snapping up out of the ground toward them, stopping short, on their screen alone.
    FangsInALine,
    /// A guardian's beam from the water near them to them, charging, never firing, on their screen alone.
    BeamFromTheWater,
    /// The fish in a bucket of theirs another kind overnight, a bucket for a bucket.
    BucketFishChanged,
    /// A head with their own face on their pillow as they wake, on their screen alone a while.
    HeadOnThePillow,
    /// A lead knotted on a fence post at its place, nothing on the other end.
    EmptyLead,
    /// A dried ghast on their doorstep as they come home, on their screen alone a while.
    DriedGhastOnDoorstep,
    /// By day, every eyeblossom near them opening its eyes at once, looking, a minute.
    EyeblossomsOpen,
    /// A copper golem statue by their door that turns to face them, again and again, on their screen alone.
    GolemStatueTurns,
    /// The things on a shelf of theirs in another order by morning.
    ShelfShuffled,
    /// At night, the firefly bushes near them going dark one by one toward them, as if something passed.
    FirefliesGoOut,
    /// A trial key lying on their doorstep in the morning, on their screen alone: gone as they reach for it.
    TrialKeyLeft,
    /// Their own name over their screen in big letters for a second, at night.
    TitleYourName,
    /// Far off at its place the figure digging, the shovel's sound carrying; there, a fresh hole.
    FigureDigging,
    /// Far off, the figure bowing to them, slowly, and straightening, and gone.
    FigureBows,
    /// At night the figure on the outside wall of the house, climbing it past a window; gone once looked at.
    ClimbingTheHouse,
    /// Through the window, the figure standing at their lectern, facing it, reading; gone as they come in.
    FigureAtTheLectern,
    /// Far off, the figure wearing exactly the armour they wear, its shimmer and all.
    FigureInYourArmour,
    /// At sunset, the figure on top of the nearest mountain, against the sky.
    OnThePeak,
    /// Every cow near them turning its back on them at once, a while.
    CattleTurnAway,
    /// The horse they ride standing still a few seconds, staring at nothing ahead, refusing them.
    HorseBalks,
    /// Their cat by their bed in the morning with a gift, as cats do: one thing of theirs, from their chest.
    CatGift,
    /// Every armadillo near them curling up at once, as if something passed that they feared.
    ArmadillosCurl,
    /// A sniffer at its place a while, snuffling and digging at the ground, then gone.
    SnifferAtItsPlace,
    /// The next animal they feed turning its head away, eating nothing, once.
    RefusesFood,
    /// The young animals near them staring at them wherever they go, a long while.
    TheChildStares,
    /// What they next mine dropping a moment late, as if held.
    LateDrop,
    /// Their bundle holding one thing more than they put in it: a pebble of cobblestone, of its digging.
    BundleGains,
    /// Their leather armour in the chest dyed black overnight.
    LeatherDyed,
    /// Each diamond they mine, a soft chime, one more each time, as if counted.
    OreCounted,
    /// A snowball they throw into the dark coming flying back from where it landed.
    SnowballComesBack,
    /// An arrow they shoot into the dark landing at their feet a moment later, their own.
    ArrowReturned,
    /// Crouched at a ledge's edge, a tug toward the drop, a little; never over.
    EdgeTug,
    /// In the Nether, every piglin near them stopping to watch them a few seconds; none turns on them.
    PiglinsStare,
    /// In the Nether, the strider they ride stopping, deaf to the rod, facing the way their house would be.
    StriderTurnsHome,
    /// In the Nether, every ghast near them going quiet, firing nothing, drifting where it is facing them a minute.
    GhastWatches,
    /// In the Nether, the hoglins near them standing off, facing them, charging none, as round something beside them.
    HoglinsPart,
    /// In the Nether, every magma cube near them stopping its bouncing and sitting still a few seconds.
    MagmaCubesSettle,
    /// In the Nether, the wither skeletons near them standing with their backs to them, faces to the wall, striking none.
    BackToTheWall,
    /// In the End, every enderman near them turning its back on them at once, so there is nothing to look at.
    EndermenTurnAway,
    /// In the Nether, the zombified piglins near them coming a step nearer each time they look away, silent, and back the moment they look.
    ZombifiedFollow,
    /// In the Nether, where it never rains, the sound of rain round them, as at home, a while.
    RainInTheNether,
    /// In the Nether, behind them, the sound of a door like theirs opening and shutting.
    TheirDoorBehind,
    /// In the Nether, steps on the rock overhead, slow, crossing over them and stopping right above.
    NetherRoofSteps,
    /// In the Nether, at the portal they came through, the sounds of home coming through it.
    HomeThroughThePortal,
    /// In the End, the dragon's growl from the central island, far off, long after it died.
    TheDragonGrowls,
    /// In the End, the beat of a second pair of wings just behind them, keeping pace.
    SecondWings,
    /// In the End, at an end city, a knock from the other side of the wall beside them.
    PurpurKnock,
    /// In the End, by the ship, the deck creaking under weight, step by step, as if someone walked it.
    ShipCreaks,
    /// In a Nether fortress, a torch ahead of them that they did not place, standing as theirs stand.
    TheirTorchAhead,
    /// In the Nether, their name cut a block deep into the netherrack behind them, read as they turn back.
    NameInNetherrack,
    /// In the Nether, prints their size ahead of them, leading away into the dark.
    PrintsInSoulSand,
    /// In the End, prints across the end stone from them toward the island's edge and off it.
    PrintsOffTheEdge,
    /// Back from another dimension, prints already leading out of their portal toward the house.
    PrintsFromThePortal,
    /// In the Nether, the obsidian of the portal they came through weeping, purple tears, on their screen alone.
    PortalWeeps,
    /// In the Nether, the figure standing just inside the portal they came through, facing them.
    StandingInThePortal,
    /// In a Nether fortress, the figure at the far end of the bridge ahead of them, still.
    FarEndOfTheBridge,
    /// In the End, their name from the void below them in the subtitles, once.
    NameFromTheVoid,
    /// In the End, the figure standing on nothing out over the void, facing them.
    StandingOnTheVoid,
    /// In the Nether, a splash potion from the dark landing at their feet, as at home.
    SplashInTheNether,
    /// At home, the sound of someone going through their portal behind them.
    SomeoneWentThrough,
    /// The obsidian near them, a ruined portal's, weeping as if it were lit, on their screen alone.
    RuinedPortalGlows,
    /// Indoors at night, a splash potion shattering against the outside of their window; nothing comes in.
    PotionAtTheWindow,
    /// In the Nether or the End, where there is no weather, lightning striking a few blocks from them; nothing burns.
    LightningBelow,
    /// At an end city, every shulker near them opening its shell at once, and shutting it at once.
    ShulkersOpen,
    /// In the End, the gateway near them flashing its beam as if someone went through; no one is near.
    GatewayFlashes,
    /// In a fortress, the blazes near them drifting into a ring round them and hanging there, firing nothing.
    BlazeRing,
    /// In the End, the endermen near them standing in a row out toward the island's edge, spaced, all facing the void.
    EndermenAtTheEdge,
    /// In the End, the music stopping and every enderman's sound with it a minute: only their own steps.
    EndGoesQuiet,
    /// In the End, every chorus plant near them flowering at once, on their screen alone a while.
    ChorusBlooms,
    /// The dragon egg at home on their pillow in the morning.
    EggOnThePillow,
    /// A chorus fruit eaten near home setting them down on their doorstep.
    ChorusLandsHome,
    /// An ender pearl they throw landing them where it should, turned round, facing the way they came.
    PearlTurned,
    /// From the dark, one of their own potions from the chest at home thrown at their feet; the chest is one short.
    TheirOwnPotion,
    /// In the Nether, the vines near them a block longer each time they look back, reaching toward them.
    VinesReach,
    /// In the Nether, their portal going out as they reach it, and lit again ten seconds on; they are never left there.
    PortalGoesOut,
    /// In the Nether, a piglin near them dropping one thing of theirs from their chest at home; the chest is one short.
    BarterFromHome,
    /// The end ship's elytra gone from its frame, hung instead in an item frame of theirs at home.
    ElytraAtHome,
    /// As the dragon dies, the figure standing on the exit portal's bedrock, facing them; gone as the portal opens.
    OnTheExitPortal,
    /// A little leaf litter on the step outside the door they use, as if blown in.
    LeavesAtTheDoor,
    /// A thread of smoke over the top of their house, as from a chimney, on their screen alone.
    SmokeFromTheRoof,
    /// A few seconds of their steps outdoors sounding like their own floorboards (the `sound_swap` hint).
    WrongFooting,
    /// The books on their chiseled bookshelf each one slot along (`rotate_slots`).
    BookMovedOnShelf,
    /// A head or skull they set on the floor turned to face their bed (its `rotation`).
    SkullTurned,
    /// A pet they left sitting got up where it sat, looking at the door (`stand_up`).
    PetGotUp,
    /// A light behind one of their windows a few seconds, on their screen alone, as if someone carried a torch past.
    LightInYourWindow,
    /// Their cold furnace's front lit a moment and dark again, nothing in it.
    FurnaceGlows,
    /// A sheet of paper on the floor inside their door, on their screen alone, gone as they reach for it.
    PaperOnTheFloor,
    /// Far off ahead of them in daylight, a light flashing three times on the ground, on their screen alone.
    AGlintOnTheHill,
    /// Underground, a slow heartbeat from inside the rock ahead of them.
    HeartbeatInTheStone,
    /// In a storm, a bolt far ahead of them and its thunder from right behind them.
    ThunderBehind,
    /// The creak of someone sitting down on their bed, no one on it.
    BedCreaks,
    /// At the bottom of the world, a knock from under the bedrock, then two.
    KnockFromBelow,
    /// In a storm, five claps of thunder a beat apart, counting down, then none.
    ThunderCounts,
    /// In the mountains, their name called far off and its echo from the peaks, in the subtitles.
    NameEchoes,
    /// A small whirl of dust crossing their field toward them, falling apart at their feet.
    DustDevil,
    /// Their lightning rod sparking and crackling under a clear sky.
    LightningRodSparks,
    /// At night on the ocean, a pale light far under their boat keeping pace with them.
    LightBelowTheBoat,
    /// At night in a swamp, small lights over the water drifting away ahead of them.
    LightsOverTheSwamp,
    /// Their campfire's smoke rising in puffs: three short, three long, three short.
    SmokeSignals,
    /// The water in their cauldron splashing once, as if something dipped into it.
    CauldronRipples,
    /// A few leaves drifting down from the ceiling of their room, on their screen alone.
    LeavesFallInside,
    /// Far along the way ahead of them by day, a trail of dust as of someone walking toward them; no one there.
    DustOnTheRoad,
    /// The tall grass parting in a line coming toward them, as if someone walked through it; nothing there.
    TheGrassParts,
    /// Under a clear sky, rain on them alone, a patch round them that follows them a minute.
    ACloudOfItsOwn,
    /// In a warm place, a few snowflakes falling on them out of a clear sky.
    SnowInTheHeat,
    /// Just after the rain, it goes on raining a minute for them alone, the drops and the sound.
    RainStays,
    /// At night on a river, a small flame drifting past on the water, as of a lantern carried down it.
    LanternDownTheRiver,
    /// In a storm, lightning striking the ground round them again and again, in a ring that closes in.
    LightningRoundYou,
    /// In a storm, lightning striking in a line, one bolt after another, leading away from them toward its place.
    BoltsInALine,
    /// In a storm, their door banging open and shut with each thunderclap.
    DoorBangsInTheStorm,
    /// In a storm, each flash of lightning showing a figure in the field, nearer each time.
    FigureInTheFlash,
    /// By day, far across their field, the figure bent over the crops as if tending them.
    FigureInYourField,
    /// By day, a dark figure on the skyline under the sun, cut out against the glare; gone once seen.
    AgainstTheSun,
    /// Gliding on elytra, the figure gliding beside them, wingless, keeping pace.
    FliesBesideYou,
    /// In an ancient city, the candles near them lighting one by one, on their screen alone.
    TheCityCandles,
    /// In a lush cave, the glow berries round them going dark one by one, on their screen alone.
    GlowBerriesGoDark,
    /// What it took from them, held in this block (suspicious sand, a decorated pot) for them to
    /// find: brushed or broken, it gives it back.
    KeepIn(&'static str),
    /// A grave at its place with their own name on it and a day not come yet.
    YourGrave,
    /// Cobwebs in the upper corners of their house, gone in a day.
    Cobwebs,
    /// A door against an inside wall of theirs, the wall behind it, gone in a day.
    FalseDoor,
    /// A hollow in the snow by the house the shape of a person lying down.
    SnowShape,
    /// The house cold a while: their breath shows, frost on the floor.
    ColdHouse,
    /// The killer rabbit at them: this much harm a bite, never a kill.
    KillerRabbit(f32),
    /// Their windows frosted white from inside a while.
    FrostOnGlass,
    /// A flight of stairs up from their floor into the ceiling, going nowhere, gone in a day.
    StairsToNowhere,
    /// A face cut into the leaves of a tree by the house, looking at their door.
    FaceInTheCanopy,
    /// A pair of boots like theirs standing inside their door, as if someone came in.
    BootsAtTheDoor,
    /// The figure crawling upside down across the ceiling over them.
    CeilingCrawl,
    /// The figure standing in a lava lake near them, unburnt, facing them.
    InTheLava,
    /// A head peeking over the cliff edge high above them.
    OverTheEdge,
    /// The figure on a skeleton horse far off, watching.
    RiderOnTheRidge,
    /// The figure riding a happy ghast in the air over the house.
    HappyGhastOverhead,
    /// Far from any village, a villager bangs on their door, runs in and drops dead.
    VillagerFleesIn,
    /// Every sound they hear goes wrong a while, one way of four.
    SoundWarp,
    /// Their redstone by the house dead a while: plates, buttons and levers power nothing.
    DeadRedstone,
    /// Footprints round the house, more each night, that stay.
    PrintsAroundTheHouse,
    /// Underground, their torches flicker one after another down the line toward them.
    CaveTorchesFlicker,
    /// A small scene that does not belong, set out where they will come on it.
    OutOfPlace,
    /// A stand of trees near them gone bare, or skinned, or both.
    BareTrees,
    /// The next villager they go to trade with stabs them: this much harm, never a kill.
    VillagerStabs(f32),
    /// Their most valuable things, carried and in their chests, up into the sky.
    TakenUp,
    /// What it took up falls out of the sky in front of their door.
    SkyGivesBack,
    /// A snowball hits their back; no one is there.
    SnowballThrown,
    /// While they are out, an arrow struck in their front door.
    ArrowInTheDoor,
    /// Far off, a figure holding their own sword, raising it.
    RaisesTheirSword,
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
    /// At the foot of the bed, the far side from the target as they wake.
    BedFoot,
    /// Just under still water near them, its head below the surface.
    UnderWater,
    /// Forty blocks to their side, moving as they move.
    Mirror,
    /// Outside a window of the house, looking in.
    Window,
    /// Among the villagers near them.
    Crowd,
    /// Two blocks behind them.
    Close,
    /// Far, crouching when they crouch.
    FarCopying,
    /// Far, swinging their tool at the rock now and then.
    FarWorking,
    /// At the edge of their light, stepping back out of it.
    EdgeOfLight,
}

/// The buckets of fish `bucket_fish_changed` changes one into another: none of a living
/// creature with a kind of its own (an axolotl, a tadpole).
pub const FISH_BUCKETS: &[&str] = &[
    "cod_bucket",
    "salmon_bucket",
    "pufferfish_bucket",
    "tropical_fish_bucket",
];

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

/// The most nights `one_more_each_night` comes, and so the most figures it stands.
pub const ONE_MORE_MOST: u32 = 6;
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
        &[FavouriteInContainer, HoldsNothing, TargetAwayOrAsleep],
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
    // Batch 5 (owner): the bed moved, recoloured, doubled; under it; at its foot.
    row(
        "shift_bed",
        3,
        5,
        &[Bed, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::ShiftBed,
        false,
    ),
    row(
        "sheets_changed",
        3,
        5,
        &[Bed, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::SheetsChanged,
        false,
    ),
    row(
        "under_the_bed",
        3,
        5,
        &[Bed, TargetInBed],
        2.0,
        3,
        Half,
        Action::Sound("minecraft:entity.silverfish.step"),
        true,
    ),
    row(
        "foot_of_bed",
        4,
        5,
        &[Bed, JustWoke],
        2.0,
        2,
        Full,
        Action::Figure(FigureWhere::BedFoot, 0.5, 0.0),
        false,
    ),
    row(
        "second_bed",
        4,
        5,
        &[Bed, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::SecondBed,
        false,
    ),
    // Batch 6 (owner): the four rows built, four new, and the animals turning on
    // them, by the harm stages: a nip from stage 3, real harm from 4, never a kill.
    row(
        "scarecrow",
        3,
        5,
        &[Field, ScarecrowParts, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::Scarecrow,
        false,
    ),
    row(
        "bees_out",
        3,
        5,
        &[Hive, Night],
        2.0,
        3,
        Full,
        Action::BeesOut,
        false,
    ),
    row(
        "horse_moved",
        2,
        5,
        &[LeashedHorse, TargetAwayOrAsleep],
        2.0,
        3,
        Full,
        Action::HorseMoved,
        false,
    ),
    row(
        "field_word",
        4,
        5,
        &[FieldWord, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::FieldWord,
        false,
    ),
    row(
        "face_the_house",
        4,
        5,
        &[Penned(3), Dawn],
        2.0,
        3,
        Full,
        Action::FaceTheHouse,
        false,
    ),
    row(
        "one_more",
        3,
        5,
        &[Penned(2), Dawn],
        3.0,
        2,
        Full,
        Action::OneMore,
        false,
    ),
    row(
        "row_withered",
        3,
        5,
        &[Field, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::RowWithered,
        false,
    ),
    row(
        "eggs_to_door",
        2,
        5,
        &[Coop, TargetAwayOrAsleep],
        2.0,
        3,
        Full,
        Action::EggsToDoor,
        false,
    ),
    row(
        "animals_nip",
        3,
        3,
        &[AnimalsNearTarget, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::Charge(1, 1.0, 2),
        false,
    ),
    row(
        "animals_turn",
        4,
        5,
        &[AnimalsNearTarget, TargetOutdoors],
        3.0,
        3,
        Full,
        Action::Charge(3, 3.0, 3),
        false,
    ),
    // Batch 15, like a player (owner: all of them).
    row(
        "into_the_mountain",
        4,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::DigInto(false),
        false,
    ),
    row(
        "through_the_hill",
        4,
        5,
        &[TargetOutdoors],
        4.0,
        1,
        Full,
        Action::DigInto(true),
        false,
    ),
    row(
        "into_the_dark",
        3,
        5,
        &[LightNear, Night, TargetOutdoors],
        2.0,
        3,
        Full,
        Action::IntoTheDark,
        false,
    ),
    row(
        "copy_crouch",
        4,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::FarCopying, 40.0, 32.0),
        false,
    ),
    row(
        "tools_in_hand",
        3,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::FarWorking, 30.0, 32.0),
        false,
    ),
    // Batch 16, close and quiet (owner: "the close and quiet ones").
    row(
        "hand_on_shoulder",
        3,
        5,
        &[Noon, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::HandOnShoulder,
        false,
    ),
    row(
        "breath",
        3,
        5,
        &[CrouchedStill],
        1.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.player.breath"),
        true,
    ),
    row(
        "footsteps_follow",
        3,
        5,
        &[Night, TargetOutdoors, TargetWalking],
        2.0,
        3,
        Half,
        Action::Footsteps,
        false,
    ),
    row(
        "name_called",
        3,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Hint("name_called", 4.0),
        true,
    ),
    // Arrows and potions (owner: "near misses, then hits"; every potion idea).
    row(
        "arrow_near",
        3,
        5,
        &[Night, TargetOutdoors],
        1.5,
        3,
        Full,
        Action::Shoot(0.0),
        false,
    ),
    row(
        "arrow_grazes",
        4,
        4,
        &[Night, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::Shoot(2.0),
        false,
    ),
    row(
        "arrow_hits",
        5,
        5,
        &[Night, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::Shoot(4.0),
        false,
    ),
    row(
        "splash_dark",
        3,
        5,
        &[Night, TargetOutdoors],
        1.5,
        4,
        Full,
        Action::SplashDark,
        false,
    ),
    row(
        "brew_left",
        3,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::BrewLeft,
        false,
    ),
    row(
        "potion_changed",
        3,
        5,
        &[InChestAny(&["potion"]), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::PotionChanged,
        true,
    ),
    row(
        "door_cloud",
        3,
        5,
        &[Dawn, Door],
        2.0,
        2,
        Full,
        Action::DoorCloud,
        false,
    ),
    // Batch 16, the Nether and the End (owner: all three).
    row(
        "ghast_name",
        3,
        5,
        &[InNether],
        2.0,
        3,
        Half,
        Action::GhastName,
        false,
    ),
    row(
        "ender_holds",
        3,
        5,
        &[InEnd],
        2.0,
        2,
        Full,
        Action::EnderHolds,
        false,
    ),
    row(
        "busy_away",
        3,
        5,
        &[CameBack],
        1.0,
        3,
        Full,
        Action::WhileAway,
        false,
    ),
    // Batch 16, more (owner: `answer_knock`, the frames, the sky, the line).
    row(
        "answer_knock",
        2,
        5,
        &[Knocked],
        0.5,
        4,
        Half,
        Action::Sound("minecraft:block.wooden_door.close"),
        true,
    ),
    row(
        "item_frames_face",
        3,
        5,
        &[FramesToShow, TargetIndoors],
        3.0,
        2,
        Full,
        Action::FramesShow,
        false,
    ),
    row(
        "wrong_sky",
        3,
        5,
        &[Noon, TargetOutdoors],
        3.0,
        2,
        Half,
        Action::Hint("wrong_sky", 60.0),
        false,
    ),
    row(
        "fish_on_line",
        2,
        5,
        &[Holding("minecraft:fishing_rod"), WaterNear(16)],
        2.0,
        3,
        Half,
        Action::BobberUnder,
        true,
    ),
    // Batch 16, the close ones (owner: `pack`, `hostiles_close`, `shove`).
    row(
        "pack",
        3,
        5,
        &[Herd, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Pack,
        false,
    ),
    row(
        "slow_approach",
        3,
        5,
        &[Animals, TargetOutdoors],
        1.5,
        4,
        Full,
        Action::MobComes(0.35),
        false,
    ),
    row(
        "quick_approach",
        3,
        5,
        &[Animals, TargetOutdoors],
        1.5,
        4,
        Full,
        Action::MobComes(1.5),
        false,
    ),
    row(
        "hostiles_close",
        4,
        5,
        &[Hostiles, Night, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::HostilesClose,
        false,
    ),
    row(
        "shove",
        4,
        4,
        &[TargetAwake],
        3.0,
        2,
        Full,
        Action::Shove(3),
        false,
    ),
    // docs/events_todo.md, 114 (owner: "depending on the stage higher falls are okay"): stage
    // five's push may drop them six blocks, a fall that harms, never one that kills.
    row(
        "shove_far",
        5,
        5,
        &[TargetAwake],
        3.0,
        2,
        Full,
        Action::Shove(6),
        false,
    ),
    // Batch 15, building (owner: every one; materials gathered first, topped up
    // when short): a night's share at a time.
    row(
        "house_near",
        4,
        5,
        &[Home, BuildLeft("house_near"), TargetAwayOrAsleep],
        0.7,
        3,
        Full,
        Action::Build("house_near"),
        false,
    ),
    row(
        "house_grows",
        5,
        5,
        &[BuildLeft("house_grows"), TargetAwayOrAsleep],
        0.7,
        1,
        Full,
        Action::Build("house_grows"),
        false,
    ),
    row(
        "pillar",
        3,
        5,
        &[Home, BuildLeft("pillar"), TargetAwayOrAsleep],
        0.7,
        8,
        Full,
        Action::Build("pillar"),
        false,
    ),
    row(
        "hut_on_route",
        3,
        5,
        &[Route, BuildLeft("hut_on_route"), TargetAwayOrAsleep],
        3.0,
        1,
        Full,
        Action::Build("hut_on_route"),
        false,
    ),
    row(
        "dock",
        3,
        5,
        &[BuildLeft("dock"), TargetAwayOrAsleep],
        0.7,
        8,
        Full,
        Action::Build("dock"),
        false,
    ),
    row(
        "bridge",
        4,
        4,
        &[Around("water"), BuildLeft("bridge"), TargetAwayOrAsleep],
        0.7,
        8,
        Full,
        Action::Build("bridge"),
        false,
    ),
    row(
        "bridge_done",
        5,
        5,
        &[BuildLeft("bridge_done"), TargetAwayOrAsleep],
        0.7,
        8,
        Full,
        Action::Build("bridge_done"),
        false,
    ),
    row(
        "copy_build",
        4,
        5,
        &[Walls, BuildLeft("copy_build"), TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::Build("copy_build"),
        false,
    ),
    row(
        "copy_house",
        5,
        5,
        &[Walls, BuildLeft("copy_house"), TargetAwayOrAsleep],
        0.7,
        12,
        Full,
        Action::Build("copy_house"),
        false,
    ),
    row(
        "cave_room",
        3,
        5,
        &[Cave, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::CaveRoom,
        false,
    ),
    // Batch 14 (owner): the figure's swing by the harm stages (a nip at 3, real
    // harm from 4, never a kill), and four new: it mirrors them, at the window, in the crowd,
    // it waves.
    row(
        "figure_nip",
        3,
        3,
        &[TargetAwake],
        3.0,
        2,
        Full,
        Action::FigureSwing(1.0),
        false,
    ),
    row(
        "figure_swing",
        4,
        5,
        &[TargetAwake],
        3.0,
        3,
        Full,
        Action::FigureSwing(4.0),
        false,
    ),
    row(
        "figure_mirror",
        3,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::Mirror, 40.0, 20.0),
        false,
    ),
    row(
        "figure_window",
        4,
        5,
        &[Pane, Night, TargetIndoors],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::Window, 30.0, 6.0),
        false,
    ),
    row(
        "figure_crowd",
        3,
        5,
        &[Villagers, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Figure(FigureWhere::Crowd, 40.0, 8.0),
        false,
    ),
    row(
        "figure_waves",
        3,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::FigureWave,
        false,
    ),
    // Batch 13 (owner): every mine row, and four new: the tunnel goes on, its
    // torches out from the far end, a sign deep down, the way back closed.
    row(
        "mine_beside",
        3,
        5,
        &[Tunnel, UsedContainer, TargetAwayOrAsleep],
        4.0,
        2,
        Full,
        Action::MineBeside,
        false,
    ),
    row(
        "tunnel_sealed",
        3,
        5,
        &[
            Tunnel,
            Stored("minecraft:cobblestone", 2),
            TargetAwayOrAsleep,
        ],
        4.0,
        2,
        Full,
        Action::TunnelSealed,
        false,
    ),
    row(
        "tunnel_toward",
        4,
        5,
        &[Tunnel, InTunnel],
        4.0,
        2,
        Full,
        Action::TunnelToward,
        false,
    ),
    row(
        "portal_lit",
        3,
        5,
        &[Frame, InChest("flint_and_steel"), TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::PortalLit,
        false,
    ),
    row(
        "mined_out",
        3,
        5,
        &[OreAhead, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::MinedOut,
        false,
    ),
    row(
        "ladder_down",
        3,
        5,
        &[InChest("ladder"), Home, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::LadderDown,
        false,
    ),
    row(
        "tunnel_goes_on",
        3,
        5,
        &[Tunnel, TargetAwayOrAsleep],
        3.0,
        3,
        Full,
        Action::TunnelGoesOn,
        false,
    ),
    row(
        "torches_far_end",
        3,
        5,
        &[InTunnel, TunnelLights],
        2.0,
        3,
        Full,
        Action::TorchesFarEnd,
        false,
    ),
    row(
        "sign_deep",
        3,
        5,
        &[Tunnel, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::SignDeep,
        false,
    ),
    row(
        "way_back_closes",
        4,
        5,
        &[InTunnel, BehindThem],
        3.0,
        2,
        Full,
        Action::WayBackCloses,
        false,
    ),
    // Batch 12 (owner): the village's lantern on their way, what it took for sale,
    // the village emptied till dawn, and the bell that rings again (G1).
    row(
        "village_lantern",
        2,
        5,
        &[Scanned("lantern"), Route, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::VillageLantern,
        false,
    ),
    row(
        "villager_word",
        4,
        5,
        &[Held, Creature("villager")],
        5.0,
        1,
        Full,
        Action::VillagerWord,
        false,
    ),
    row(
        "village_abandoned",
        4,
        5,
        &[Scanned("_door"), Night, FarFromVillage(64)],
        99.0,
        1,
        Full,
        Action::VillageAbandoned,
        false,
    ),
    row(
        "bell_twice",
        2,
        5,
        &[RangBell],
        0.5,
        6,
        Half,
        Action::BellTwice,
        false,
    ),
    // Batch 11 (owner): the atmospheres as real fog (the review's D), the sky's
    // hints by render hooks, the HUD's, a painting, a compass, a face under the water.
    row(
        "mist",
        2,
        5,
        &[TargetAwake],
        2.0,
        4,
        Full,
        Action::Hint("mist", 240.0),
        false,
    ),
    row(
        "dusk_noon",
        3,
        5,
        &[Day, TargetAwake],
        3.0,
        2,
        Full,
        Action::Hint("dusk_noon", 300.0),
        false,
    ),
    row(
        "red_mist",
        4,
        5,
        &[Night, TargetAwake],
        3.0,
        3,
        Full,
        Action::Hint("red_mist", 270.0),
        false,
    ),
    row(
        "darkness",
        4,
        5,
        &[Night, TargetAwake],
        3.0,
        3,
        Full,
        Action::Hint("darkness", 200.0),
        false,
    ),
    row(
        "moon_wrong",
        3,
        5,
        &[Night, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::Hint("moon_wrong", 8.0),
        true,
    ),
    row(
        "stars_out",
        3,
        5,
        &[Night, NotRaining, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::Hint("stars_out", 60.0),
        true,
    ),
    row(
        "night_early",
        4,
        5,
        &[Day, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Hint("night_early", 60.0),
        false,
    ),
    row(
        "painting_flip",
        2,
        5,
        &[Decor("painting"), TargetIndoors],
        2.0,
        3,
        Half,
        Action::Hint("painting_flip", 120.0),
        true,
    ),
    row(
        "compass_points",
        3,
        5,
        &[Holding("minecraft:compass")],
        2.0,
        3,
        Half,
        Action::Hint("compass_points", 120.0),
        true,
    ),
    row(
        "hunger_shake",
        2,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Hint("hunger_shake", 3.0),
        true,
    ),
    row(
        "heart_skip",
        3,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Hint("heart_skip", 2.0),
        true,
    ),
    row(
        "subtitle",
        1,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Hint("subtitle", 2.0),
        true,
    ),
    row(
        "tab_twice",
        3,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::TabTwice,
        true,
    ),
    row(
        "reflection",
        4,
        5,
        &[WaterNear(24), TargetOutdoors],
        3.0,
        2,
        Full,
        // Six seconds at most, gone once seen and looked away from: a second, it was gone before
        // anyone found it under the water (owner: "nothing happens when i go to water").
        Action::Figure(FigureWhere::UnderWater, 6.0, 0.0),
        false,
    ),
    // Batch 10 (owner): prints in the snow, the boat ahead, ripples, a bobber with
    // no one there; a snowman, the boat in the house, a shape under the water, wet steps.
    row(
        "trail",
        3,
        5,
        &[Around("snow"), Door, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::Trail,
        false,
    ),
    row(
        "trail_in",
        4,
        5,
        &[Around("snow"), Door, TargetAwayOrAsleep],
        4.0,
        2,
        Full,
        Action::TrailIn,
        false,
    ),
    row(
        "boat_waiting",
        3,
        5,
        &[TheirBoat, WaterNear(48), TargetWalking, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::BoatWaiting,
        false,
    ),
    row(
        "ripples",
        3,
        5,
        &[WaterNear(32), TargetOutdoors],
        2.0,
        3,
        Half,
        Action::Hint("ripples", 6.0),
        true,
    ),
    row(
        "fishing_line",
        4,
        5,
        &[WaterNear(32), InChest("fishing_rod"), TargetOutdoors],
        3.0,
        2,
        Full,
        Action::FishingLine,
        false,
    ),
    row(
        "snowman",
        3,
        5,
        &[Around("snow"), Door, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::Snowman,
        false,
    ),
    row(
        "boat_in_house",
        4,
        5,
        &[TheirBoat, Home, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::BoatInHouse,
        false,
    ),
    row(
        "under_the_water",
        3,
        5,
        &[WaterNear(24), TargetOutdoors],
        2.0,
        3,
        Full,
        Action::UnderWater,
        false,
    ),
    row(
        "wet_steps",
        3,
        5,
        &[Bed, Door, TargetIndoors],
        2.0,
        3,
        Half,
        Action::WetSteps,
        true,
    ),
    // Batch 9 (owner): every row built, four new.
    row(
        "jukebox",
        3,
        5,
        &[Fixture("jukebox"), Disc, TargetApproachingDoor],
        3.0,
        2,
        Full,
        Action::JukeboxDisc,
        false,
    ),
    row(
        "lectern_page",
        2,
        5,
        &[Fixture("lectern"), TargetAwayOrAsleep],
        2.0,
        3,
        Half,
        Action::LecternPage,
        true,
    ),
    row(
        "swap_fuel",
        2,
        5,
        &[SwapFuel, TargetAwayOrAsleep],
        2.0,
        3,
        Half,
        Action::SwapFuel,
        true,
    ),
    row(
        "brewing_done",
        3,
        5,
        &[Fixture("brewing_stand"), BrewIngredient, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::Brewing,
        false,
    ),
    row(
        "grindstone_strip",
        4,
        5,
        &[Fixture("grindstone"), UsedContainer, TargetAwayOrAsleep],
        4.0,
        1,
        Full,
        Action::GrindstoneStrip,
        false,
    ),
    row(
        "lamps_blink",
        3,
        5,
        &[Fixture("redstone_lamp"), Night],
        2.0,
        3,
        Full,
        Action::LampsBlink,
        false,
    ),
    row(
        "composter_fills",
        2,
        5,
        &[Fixture("composter"), Dawn],
        1.0,
        4,
        Half,
        Action::ComposterFills,
        true,
    ),
    row(
        "crafted_for_you",
        3,
        5,
        &[Fixture("crafting_table"), Makings, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::CraftedForYou,
        false,
    ),
    row(
        "bell_rings",
        3,
        5,
        &[Fixture("bell"), Night],
        2.0,
        3,
        Full,
        Action::BellRings,
        false,
    ),
    // Batch 8 (owner): sleepers while they are awake, an advancement again, a death
    // message, their words on a sign, their ping gone, an empty message.
    row(
        "sleep_message",
        3,
        5,
        &[Night, TargetAwake],
        2.0,
        3,
        Half,
        Action::Message("sleepers"),
        false,
    ),
    row(
        "advancement",
        4,
        5,
        &[TargetAwake],
        3.0,
        2,
        Full,
        Action::Message("advancement"),
        false,
    ),
    row(
        "death_message",
        4,
        5,
        &[TargetAwake],
        3.0,
        2,
        Full,
        Action::DeathMessage,
        false,
    ),
    row(
        "words_on_sign",
        3,
        5,
        &[TargetSaid, Door, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::WordsOnSign,
        false,
    ),
    row(
        "ping_fades",
        3,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Message("no_ping"),
        false,
    ),
    row(
        "empty_message",
        3,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Message("says"),
        false,
    ),
    // Batch 7 (owner): a word in their sign, the count that rises, their weapon on
    // the wall, a head on a post, a grave for each.
    row(
        "sign_word",
        3,
        5,
        &[TheirSign, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::SignWord,
        false,
    ),
    row(
        "count_up",
        3,
        5,
        &[KilledNow],
        0.0,
        u32::MAX,
        Half,
        Action::CountUp,
        true,
    ),
    row(
        "weapon_on_wall",
        3,
        5,
        &[WeaponAndFrame, TargetAwayOrAsleep],
        4.0,
        1,
        Full,
        Action::WeaponOnWall,
        false,
    ),
    row(
        "head_on_post",
        4,
        5,
        &[HeadKind, Door, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::HeadOnPost,
        false,
    ),
    row(
        "grave_each",
        4,
        5,
        &[KillKinds, Home, TargetAwayOrAsleep],
        99.0,
        1,
        Full,
        Action::GraveEach,
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
        &[TheirUnnamedAnimal, TargetAwayOrAsleep],
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
        &[TheirAnimal(""), Route, Home, TargetAwayOrAsleep],
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
        &[TheirAnimal("sheep"), TargetAwayOrAsleep],
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
        &[TheirAnimal(""), Home, TargetAwayOrAsleep],
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
        "chest_full_of_one",
        3,
        5,
        &[
            UsedContainer,
            Stored("minecraft:dirt", 3),
            TargetAwayOrAsleep,
        ],
        3.0,
        2,
        Full,
        Action::FullOfOne,
        false,
    ),
    row(
        "hopper_emptied",
        3,
        5,
        &[Fixture("hopper"), UsedContainer, TargetAwayOrAsleep],
        2.0,
        3,
        Half,
        Action::HopperEmptied,
        true,
    ),
    row(
        "shulker_moved",
        3,
        5,
        &[Fixture("shulker_box"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::ShulkerMoved,
        false,
    ),
    row(
        "book_word",
        3,
        5,
        &[InChest("writable_book"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::BookWord,
        false,
    ),
    row(
        "map_marked",
        3,
        5,
        &[InChest("filled_map"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::MapMarked,
        false,
    ),
    row(
        "returned_used",
        3,
        5,
        &[Held, TargetAwayOrAsleep],
        1.0,
        3,
        Full,
        Action::ReturnUsed,
        false,
    ),
    row(
        "renamed_item",
        3,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        2.0,
        3,
        Full,
        Action::RenameOne,
        false,
    ),
    row(
        "door_hinge",
        3,
        5,
        &[Door, TargetAwayOrAsleep],
        2.0,
        3,
        Full,
        Action::DoorHinge,
        false,
    ),
    row(
        "ladder_gap",
        3,
        5,
        &[Ladder, TargetAwayOrAsleep],
        2.0,
        2,
        Full,
        Action::LadderGap,
        false,
    ),
    row(
        "flowers_potted",
        2,
        5,
        &[Plant("flower"), Fixture("flower_pot"), TargetAwayOrAsleep],
        2.0,
        2,
        Full,
        Action::FlowersPotted,
        false,
    ),
    row(
        "pumpkin_face",
        3,
        5,
        &[Plant("pumpkin"), InChest("shears"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::PumpkinFace,
        false,
    ),
    row(
        "backfill",
        3,
        5,
        &[Hole, TargetAwayOrAsleep],
        2.0,
        3,
        Full,
        Action::Backfill,
        false,
    ),
    row(
        "alcove",
        4,
        5,
        &[Wall, TargetAwayOrAsleep],
        4.0,
        1,
        Full,
        Action::Alcove,
        false,
    ),
    row(
        "furniture_shifted",
        2,
        5,
        &[Home, TargetAwayOrAsleep],
        2.0,
        3,
        Full,
        Action::ShiftFurniture,
        false,
    ),
    row(
        "one_room_dark",
        3,
        5,
        &[Light, UsedContainer, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::RoomDark,
        false,
    ),
    row(
        "door_held",
        2,
        5,
        &[DoorUsed],
        0.5,
        6,
        Half,
        Action::DoorHeld,
        true,
    ),
    row(
        "bed_turned",
        3,
        5,
        &[Bed, TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::BedTurned,
        false,
    ),
    row(
        "tree_grown",
        3,
        5,
        &[Plant("sapling"), InChest("bone_meal"), TargetAwayOrAsleep],
        3.0,
        2,
        Full,
        Action::TreeGrown,
        false,
    ),
    row(
        "loot_gift",
        2,
        5,
        &[LootOpened],
        0.0,
        12,
        Half,
        Action::LootGift,
        true,
    ),
    row(
        "chest_answers",
        2,
        5,
        &[JustOpened, TwoContainers],
        0.5,
        8,
        Half,
        Action::ChestAnswers,
        true,
    ),
    row(
        "steps_above",
        2,
        5,
        &[TargetIndoors, TargetOnline],
        1.0,
        4,
        Half,
        Action::SoundAtTarget("minecraft:block.wood.step"),
        true,
    ),
    row(
        "scratching",
        3,
        5,
        &[TargetIndoors, Night],
        1.0,
        4,
        Half,
        Action::SoundAtTarget("minecraft:entity.silverfish.step"),
        true,
    ),
    row(
        "ladder_climb",
        3,
        5,
        &[TargetIndoors, TargetOnline],
        1.5,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.ladder.step"),
        true,
    ),
    // Stage one's too (owner: at the start the music was cut every time, the one thing that
    // needed nothing learned): an item's pop, a drip, a block placed, a subtitle.
    row(
        "pickup_pop",
        1,
        5,
        &[TargetOnline],
        1.0,
        4,
        Half,
        Action::SoundAtTarget("minecraft:entity.item.pickup"),
        true,
    ),
    row(
        "cold_crackle",
        2,
        5,
        &[Furnace, TargetIndoors, TargetOnline],
        1.5,
        3,
        Half,
        Action::Sound("minecraft:block.furnace.fire_crackle"),
        true,
    ),
    row(
        "breath_close",
        3,
        5,
        &[TargetOnline],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.player.breath"),
        true,
    ),
    row(
        "cave_air",
        2,
        5,
        &[TargetOutdoors, Night],
        1.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:ambient.cave"),
        true,
    ),
    row(
        "door_scratch",
        3,
        5,
        &[Home, TargetIndoors, Night],
        1.5,
        3,
        Half,
        Action::Taps("", "minecraft:block.wood.hit", 5),
        true,
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
            // The very items: `golden_carrot` and `baked_potato` are no seed.
            InChestAny(&[":carrot", ":potato", ":beetroot_seeds"]),
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
        "diary_sold",
        3,
        5,
        &[Creature("villager"), TargetOutdoors],
        8.0,
        1,
        Full,
        Action::DiarySold,
        false,
    ),
    row(
        "diary_brought",
        4,
        5,
        &[DiaryUnbought, Creature("villager"), TargetOutdoors],
        6.0,
        1,
        Full,
        Action::DiaryBrought,
        false,
    ),
    row(
        "letter_in_chest",
        2,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        5.0,
        3,
        Full,
        Action::LetterInChest,
        false,
    ),
    row(
        "villager_named",
        3,
        5,
        &[UnnamedVillager],
        6.0,
        2,
        Full,
        Action::VillagerNamed,
        false,
    ),
    row(
        "lectern_notes",
        4,
        5,
        &[Fixture("lectern"), TargetAwayOrAsleep],
        8.0,
        1,
        Full,
        Action::LecternNotes,
        false,
    ),
    row(
        "page_in_your_book",
        4,
        5,
        &[InChest("writable_book"), TargetAwayOrAsleep],
        6.0,
        2,
        Full,
        Action::PageInBook,
        false,
    ),
    row(
        "other_house",
        4,
        4,
        &[TargetInBed, Bed, Door, Home],
        30.0,
        1,
        Full,
        Action::OtherHouse,
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
    // Caves they have not entered (batch 4, owner: a lantern from their chest, the
    // torches its own).
    row(
        "lantern_ahead",
        3,
        5,
        &[Cave, InChest("lantern")],
        3.0,
        2,
        Full,
        Action::LanternAhead,
        false,
    ),
    row(
        "cave_lit",
        3,
        5,
        &[Cave],
        4.0,
        2,
        Full,
        Action::CaveLit,
        false,
    ),
    // A light where they died, and their last light taken along (batch 4, owner).
    row(
        "death_light",
        2,
        5,
        &[UnmarkedDeath],
        1.0,
        3,
        Full,
        Action::DeathLight,
        false,
    ),
    row(
        "light_taken",
        3,
        5,
        &[LastLight],
        2.0,
        3,
        Full,
        Action::LightTaken,
        false,
    ),
    row(
        "lights_out",
        4,
        5,
        &[Night, LightsBehind],
        1.0,
        4,
        Full,
        Action::LightsOut,
        false,
    ),
    row(
        "soul_swap",
        3,
        5,
        &[SoulableLight],
        2.0,
        3,
        Full,
        Action::SoulSwap,
        false,
    ),
    // The owner's warped animals.
    row(
        "warped_animal",
        3,
        5,
        &[WarpableAnimal, TargetOutdoors],
        1.0,
        6,
        Full,
        Action::WarpAnimal,
        false,
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
    // The house, changed (owner: "replace certain blocks of house", "trying to
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
    // The owner: "a figure walks up to you while sleeping, then pulls out something and chokes
    // you"; a lead, held in bed. By the harm stages: from 4, never a kill.
    row(
        "choke",
        4,
        5,
        &[TargetInBed, Bed, Door, Night],
        6.0,
        2,
        Full,
        Action::Choke(2.0),
        false,
    ),
    // The owner: "a variation of this where we get lit on fire, maybe shot at with a crossbow".
    row(
        "shot_in_bed",
        4,
        5,
        &[TargetInBed, Bed, Door, Night],
        6.0,
        2,
        Full,
        Action::ShotInBed(4.0),
        false,
    ),
    // The owner: "a choke event, a fire event, and a crossbow event ... use flint and steel for
    // the fire one".
    row(
        "burned_in_bed",
        4,
        5,
        &[TargetInBed, Bed, Door, Night],
        6.0,
        2,
        Full,
        Action::BurnedInBed,
        false,
    ),
    // The owner: "while we are inside our house, we get multiple figures standing in a circle
    // around our home" (docs/events_todo.md, 1).
    row(
        "ring_of_figures",
        4,
        5,
        &[Home, Night, TargetIndoors],
        5.0,
        2,
        Full,
        Action::RingOfFigures(8),
        false,
    ),
    // docs/events_todo.md, 2.
    row(
        "upside_down",
        4,
        5,
        &[JustWoke, Bed],
        4.0,
        2,
        Full,
        Action::UpsideDown,
        false,
    ),
    // docs/events_todo.md, 3.
    row(
        "the_crawl",
        4,
        5,
        &[Night, TargetIndoors, Bed, Door],
        4.0,
        2,
        Full,
        Action::Crawl,
        false,
    ),
    // docs/events_todo.md, 4.
    row(
        "hunted",
        5,
        5,
        &[Night, TargetOutdoors],
        4.0,
        2,
        Full,
        Action::Hunted,
        false,
    ),
    // docs/events_todo.md, 5.
    row(
        "boat_passenger",
        4,
        5,
        &[TargetInBoat],
        4.0,
        2,
        Full,
        Action::BoatPassenger,
        false,
    ),
    // docs/events_todo.md, 6.
    row(
        "every_window",
        5,
        5,
        &[Night, TargetIndoors, Pane, Home],
        5.0,
        2,
        Full,
        Action::EveryWindow,
        false,
    ),
    // docs/events_todo.md, 7: a night apart, one more each time.
    row(
        "one_more_each_night",
        4,
        5,
        &[Night, LightNear],
        0.8,
        ONE_MORE_MOST,
        Full,
        Action::OneMoreEachNight,
        false,
    ),
    // docs/events_todo.md, 8: past `door_held` (shut again once open), it will not open at all.
    row(
        "door_wont_open",
        4,
        5,
        &[TargetApproachingDoor, Door, Home],
        3.0,
        2,
        Full,
        Action::DoorWontOpen,
        false,
    ),
    // docs/events_todo.md, 9.
    row(
        "sleepwalk",
        5,
        5,
        &[TargetInBed, Night, Bed, Door, Home],
        5.0,
        1,
        Full,
        Action::Sleepwalk,
        false,
    ),
    // docs/events_todo.md, 10: never from the menu, on the light placed.
    row(
        "torch_flash",
        4,
        5,
        &[LitUnderground],
        1.0,
        4,
        Full,
        Action::TorchFlash,
        false,
    ),
    // docs/events_todo.md, 11.
    row(
        "where_you_died",
        3,
        5,
        &[NearTheirDeath],
        4.0,
        2,
        Full,
        Action::WhereYouDied,
        false,
    ),
    // docs/events_todo.md, 12.
    row(
        "rubberband",
        4,
        5,
        &[TargetWalking, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::Rubberband,
        false,
    ),
    // docs/events_todo.md, 13: what it took, not what they lost dying (that it never holds).
    row(
        "the_offer",
        4,
        5,
        &[Held, Door, Home, TargetOutdoors],
        6.0,
        1,
        Full,
        Action::TheOffer,
        false,
    ),
    // docs/events_todo.md, 14.
    row(
        "top_of_the_ladder",
        4,
        5,
        &[DownTheShaft],
        3.0,
        2,
        Full,
        Action::TopOfTheLadder,
        false,
    ),
    // docs/events_todo.md, 15.
    row(
        "pulled_under",
        4,
        5,
        &[TargetSwimming],
        4.0,
        2,
        Full,
        Action::PulledUnder,
        false,
    ),
    // docs/events_todo.md, 16.
    row(
        "its_lights_on",
        4,
        5,
        &[Night, ItsHouseNear],
        3.0,
        3,
        Full,
        Action::ItsLightsOn,
        false,
    ),
    // docs/events_todo.md, 17.
    row(
        "a_face_in_the_chest",
        3,
        5,
        &[UsedContainer, TargetAwayOrAsleep],
        4.0,
        2,
        Full,
        Action::FaceInTheChest,
        false,
    ),
    // docs/events_todo.md, 18.
    row(
        "falls_from_the_sky",
        5,
        5,
        &[Night, LookingUp],
        4.0,
        2,
        Full,
        Action::FallsFromTheSky,
        false,
    ),
    // docs/events_todo.md, 19.
    row(
        "pet_follows",
        4,
        5,
        &[Pet, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::PetFollows,
        false,
    ),
    // docs/events_todo.md, 20.
    row(
        "lies_beside",
        5,
        5,
        &[TargetInBed, Bed, Door, Night],
        5.0,
        2,
        Full,
        Action::LiesBeside,
        false,
    ),
    // docs/events_todo.md, 23.
    row(
        "respawn_its_bed",
        5,
        5,
        &[ItsHouse, TargetOnline],
        30.0,
        1,
        Full,
        Action::RespawnItsBed,
        false,
    ),
    // docs/events_todo.md, 24.
    row(
        "ender_chest_note",
        4,
        5,
        &[Fixture("ender_chest"), TargetOnline],
        8.0,
        2,
        Full,
        Action::EnderChestNote,
        false,
    ),
    // docs/events_todo.md, 25: armed as they hold one.
    row(
        "eye_turns_home",
        5,
        5,
        &[Holding("minecraft:ender_eye"), Home],
        10.0,
        1,
        Full,
        Action::EyeTurnsHome,
        false,
    ),
    // docs/events_todo.md, 26.
    row(
        "crossing_ahead",
        3,
        5,
        &[TargetWalking, TargetOutdoors],
        2.0,
        4,
        Full,
        Action::CrossingAhead,
        false,
    ),
    // docs/events_todo.md, 27.
    row(
        "in_a_line",
        4,
        5,
        &[Route, Night, TargetOutdoors, Home],
        4.0,
        2,
        Full,
        Action::InALine,
        false,
    ),
    // docs/events_todo.md, 28.
    row(
        "peeks_from_ground",
        4,
        5,
        &[TargetOutdoors],
        3.0,
        3,
        Full,
        Action::PeeksFromGround,
        false,
    ),
    // docs/events_todo.md, 29.
    row(
        "wears_your_face",
        4,
        5,
        &[Night, Hostiles, TargetOutdoors],
        4.0,
        2,
        Full,
        Action::WearsYourFace,
        false,
    ),
    // docs/events_todo.md, 30.
    row(
        "animal_follows",
        3,
        5,
        &[AnimalsNearTarget],
        4.0,
        2,
        Full,
        Action::AnimalFollows,
        false,
    ),
    // docs/events_todo.md, 31.
    row(
        "held_breath",
        4,
        5,
        &[TargetUnderwater],
        4.0,
        2,
        Full,
        Action::HeldBreath,
        false,
    ),
    // docs/events_todo.md, 32.
    row(
        "all_freeze",
        4,
        5,
        &[AnimalsNearTarget, TargetOutdoors],
        3.0,
        3,
        Full,
        Action::AllFreeze,
        false,
    ),
    // docs/events_todo.md, 33.
    row(
        "figure_on_map",
        4,
        5,
        &[Holding("minecraft:filled_map"), Home],
        3.0,
        3,
        Full,
        Action::FigureOnMap,
        false,
    ),
    // docs/events_todo.md, 34: never from the menu, as they come back.
    row(
        "waits_at_the_portal",
        4,
        5,
        &[CameBack],
        2.0,
        3,
        Full,
        Action::WaitsAtThePortal,
        false,
    ),
    // docs/events_todo.md, 35: a face, not their skin's colours, which the game never says.
    row(
        "firework_over_its_house",
        4,
        5,
        &[Night, ItsHouseNear],
        3.0,
        3,
        Full,
        Action::FireworkOverItsHouse,
        false,
    ),
    // docs/events_todo.md, 36.
    row(
        "chicken_lays",
        3,
        5,
        &[Held, Creature("chicken")],
        4.0,
        1,
        Full,
        Action::ChickenLays,
        false,
    ),
    // docs/events_todo.md, 41.
    row(
        "at_their_table",
        4,
        5,
        &[TargetApproachingDoor, Fixture("crafting_table"), Home],
        3.0,
        2,
        Full,
        Action::AtTheirTable,
        false,
    ),
    // docs/events_todo.md, 42.
    row(
        "jumping_far",
        4,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::JumpingFar,
        false,
    ),
    // docs/events_todo.md, 43.
    row(
        "waves_from_inside",
        4,
        5,
        &[TargetApproachingDoor, Pane, Home],
        3.0,
        2,
        Full,
        Action::WavesFromInside,
        false,
    ),
    // docs/events_todo.md, 44.
    row(
        "prints_to_the_cave",
        4,
        5,
        &[Around("snow"), Door, Home, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::PrintsToTheCave,
        false,
    ),
    // docs/events_todo.md, 45.
    row(
        "rides_their_horse",
        4,
        5,
        &[TheirAnimal("horse"), TargetOutdoors],
        4.0,
        2,
        Full,
        Action::RidesTheirHorse,
        false,
    ),
    // docs/events_todo.md, 46.
    row(
        "eats_at_their_chest",
        4,
        5,
        &[TargetApproachingDoor, UsedContainer, Home],
        4.0,
        2,
        Full,
        Action::EatsAtTheirChest,
        false,
    ),
    // docs/events_todo.md, 47.
    row(
        "mirror_in_the_window",
        5,
        5,
        &[TargetOutdoors, Pane, Home],
        4.0,
        2,
        Full,
        Action::MirrorInTheWindow,
        false,
    ),
    // docs/events_todo.md, 48.
    row(
        "every_door_opens",
        5,
        5,
        &[Night, TargetIndoors, Door, Home],
        4.0,
        2,
        Full,
        Action::EveryDoorOpens,
        false,
    ),
    // docs/events_todo.md, 49.
    row(
        "flowers_wilted",
        3,
        5,
        &[TargetAwayOrAsleep],
        5.0,
        2,
        Full,
        Action::FlowersWilted,
        false,
    ),
    // docs/events_todo.md, 50.
    row(
        "camp_nearby",
        4,
        5,
        &[Night, TargetOutdoors, Route],
        5.0,
        2,
        Full,
        Action::CampNearby,
        false,
    ),
    // docs/events_todo.md, 51.
    row(
        "stone_circle",
        4,
        5,
        &[TargetAwayOrAsleep, Home],
        10.0,
        1,
        Full,
        Action::StoneCircle,
        false,
    ),
    // docs/events_todo.md, 52: never with monsters near.
    row(
        "slips_from_hand",
        4,
        5,
        &[TargetWalking],
        3.0,
        3,
        Full,
        Action::SlipsFromHand,
        false,
    ),
    // docs/events_todo.md, 56.
    row(
        "pet_stares_at_wall",
        3,
        5,
        &[Pet, Home],
        3.0,
        3,
        Full,
        Action::PetStaresAtWall,
        false,
    ),
    // docs/events_todo.md, 57.
    row(
        "bats_in_the_house",
        4,
        5,
        &[Night, TargetIndoors, Home],
        4.0,
        2,
        Full,
        Action::BatsInTheHouse,
        false,
    ),
    // docs/events_todo.md, 58: by day; the world's clock, for everyone in it.
    row(
        "lost_hour",
        5,
        5,
        &[Day, TargetOnline],
        6.0,
        2,
        Full,
        Action::LostHour,
        false,
    ),
    // docs/events_todo.md, 59.
    row(
        "chest_wont_open",
        4,
        5,
        &[TargetApproachingChest, UsedContainer],
        3.0,
        2,
        Full,
        Action::ChestWontOpen,
        false,
    ),
    // docs/events_todo.md, 60.
    row(
        "name_in_the_snow",
        4,
        5,
        &[Around("snow"), Home, TargetAwayOrAsleep],
        6.0,
        1,
        Full,
        Action::NameInTheSnow,
        false,
    ),
    // docs/events_todo.md, 61.
    row(
        "torch_ring",
        4,
        5,
        &[TargetAwayOrAsleep, Home],
        8.0,
        1,
        Full,
        Action::TorchRing,
        false,
    ),
    // docs/events_todo.md, 62.
    row(
        "breadcrumbs",
        4,
        5,
        &[UsedContainer, Door, Home, Night, TargetAwayOrAsleep],
        5.0,
        1,
        Full,
        Action::Breadcrumbs,
        false,
    ),
    // docs/events_todo.md, 63.
    row(
        "fishing_figure",
        3,
        5,
        &[TargetOutdoors, Around("water")],
        3.0,
        2,
        Full,
        Action::FishingFigure,
        false,
    ),
    // docs/events_todo.md, 64.
    row(
        "by_the_bed",
        4,
        5,
        &[Night, TargetApproachingDoor, Bed, Home],
        3.0,
        2,
        Full,
        Action::ByTheBed,
        false,
    ),
    // docs/events_todo.md, 65.
    row(
        "swims_beneath",
        4,
        5,
        &[TargetSwimming],
        3.0,
        2,
        Full,
        Action::SwimsBeneath,
        false,
    ),
    // docs/events_todo.md, 66.
    row(
        "stands_in_the_rain",
        3,
        5,
        &[Raining, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::StandsInTheRain,
        false,
    ),
    // docs/events_todo.md, 67.
    row(
        "its_banner",
        4,
        5,
        &[ItsHouse],
        20.0,
        1,
        Full,
        Action::ItsBanner,
        false,
    ),
    // docs/events_todo.md, 68.
    row(
        "chest_rattles",
        3,
        5,
        &[UsedContainer, TargetIndoors],
        2.0,
        3,
        Full,
        Action::ChestRattles,
        false,
    ),
    // docs/events_todo.md, 69.
    row(
        "trapdoor_taps",
        3,
        5,
        &[Fixture("trapdoor"), TargetOnline],
        2.0,
        3,
        Full,
        Action::TrapdoorTaps,
        false,
    ),
    // docs/events_todo.md, 70.
    row(
        "climbs_ahead",
        4,
        5,
        &[DownTheShaft],
        3.0,
        2,
        Full,
        Action::ClimbsAhead,
        false,
    ),
    // docs/events_todo.md, 71.
    row(
        "on_the_water",
        4,
        5,
        &[Around("water"), TargetOutdoors],
        3.0,
        2,
        Full,
        Action::OnTheWater,
        false,
    ),
    // docs/events_todo.md, 72.
    row(
        "in_the_treetops",
        4,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::InTheTreetops,
        false,
    ),
    // docs/events_todo.md, 73.
    row(
        "path_through_the_trees",
        5,
        5,
        &[TargetAwayOrAsleep, Home],
        10.0,
        1,
        Full,
        Action::PathThroughTheTrees,
        false,
    ),
    // docs/events_todo.md, 74.
    row(
        "rides_the_cart",
        4,
        5,
        &[Vehicle, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::RidesTheCart,
        false,
    ),
    // docs/events_todo.md, 76.
    row(
        "lantern_in_the_woods",
        4,
        5,
        &[Night, TargetOutdoors],
        3.0,
        3,
        Full,
        Action::LanternInTheWoods,
        false,
    ),
    // docs/events_todo.md, 101 (owner's): the hit by the harm stages, two hearts as
    // `figure_swing`'s.
    row(
        "while_unseen",
        4,
        5,
        &[Night, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::WhileUnseen(4.0),
        false,
    ),
    // docs/events_todo.md, 102 (owner's): never from the menu, on their running lit a while.
    row(
        "drawn_to_the_light",
        4,
        5,
        &[RanLit, Night, TargetOutdoors],
        1.0,
        3,
        Full,
        Action::DrawnToTheLight,
        false,
    ),
    // docs/events_todo.md, 103 (owner's): "consider chests as well, and a random number of
    // items".
    row(
        "taken_up",
        4,
        5,
        &[TargetOutdoors],
        1.0,
        3,
        Full,
        Action::TakenUp,
        false,
    ),
    row(
        "sky_gives_back",
        4,
        5,
        &[TakenUpDue, Home],
        0.0,
        6,
        Full,
        Action::SkyGivesBack,
        false,
    ),
    // docs/events_todo.md, 115 (owner's): "opening the main menu it kinda freaks out, like
    // buttons vibrate"; armed ten minutes, the next time they open it.
    row(
        "menu_freaks",
        4,
        5,
        &[TargetOnline],
        2.0,
        3,
        Full,
        Action::Hint("menu_freaks", 600.0),
        false,
    ),
    // docs/events_todo.md, 112 (owner's): "some trees dont have leaves? or like have all of the
    // logs stripped".
    row(
        "bare_trees",
        4,
        5,
        &[InTheWoods],
        2.0,
        3,
        Full,
        Action::BareTrees,
        false,
    ),
    // docs/events_todo.md, 110 (owner's).
    row(
        "out_of_place",
        3,
        5,
        &[TargetOnline],
        1.5,
        4,
        Full,
        Action::OutOfPlace,
        false,
    ),
    // docs/events_todo.md, 109 (owner's): "in a cave their torches flicker".
    row(
        "cave_torches_flicker",
        3,
        5,
        &[TargetUnderground, TheirCaveLights],
        1.0,
        4,
        Full,
        Action::CaveTorchesFlicker,
        false,
    ),
    // docs/events_todo.md, 108 (owner's): "persistent footsteps around the house that appear over
    // time".
    row(
        "prints_around_the_house",
        3,
        5,
        &[Home, TargetAwayOrAsleep],
        0.8,
        9,
        Full,
        Action::PrintsAroundTheHouse,
        false,
    ),
    // docs/events_todo.md, 107 (owner's): "their pressure plates dont work, or like redstone
    // doesnt fire".
    row(
        "dead_redstone",
        3,
        5,
        &[Home, RedstoneAtHome],
        2.0,
        3,
        Full,
        Action::DeadRedstone,
        false,
    ),
    // docs/events_todo.md, 106 (owner's): "sound distortion effects, like DSP".
    row(
        "sound_warp",
        3,
        5,
        &[TargetAwake],
        1.5,
        4,
        Full,
        Action::SoundWarp,
        false,
    ),
    // docs/events_todo.md, 105 (owner's): "a villager appear in front of their house (away from a
    // village), then it bangs on the door, runs in and dies".
    row(
        "villager_flees_in",
        4,
        5,
        &[NoVillageNear, Home, Door, TargetIndoors],
        6.0,
        1,
        Full,
        Action::VillagerFleesIn,
        false,
    ),
    // docs/events_todo.md, 104 (owner's): "a fake creeper ... that explodes always but does no
    // damage".
    row(
        "creeper_behind",
        3,
        5,
        &[TargetAwake],
        2.0,
        3,
        Full,
        Action::CreeperBehind,
        false,
    ),
    // docs/events_todo.md, 113 (owner's): "single should do damage based on stage, wave should
    // be a bunch coming at the player but dealing little damage, and never killing as always".
    row(
        "vex_single",
        4,
        4,
        &[TargetAwake],
        2.0,
        2,
        Full,
        Action::Vexes(1, 4.0),
        false,
    ),
    row(
        "vex_strikes",
        5,
        5,
        &[TargetAwake],
        2.0,
        2,
        Full,
        Action::Vexes(1, 6.0),
        false,
    ),
    row(
        "vex_wave",
        4,
        5,
        &[TargetAwake],
        3.0,
        2,
        Full,
        Action::Vexes(12, 1.0),
        false,
    ),
    // docs/events_todo.md, 100: two hearts a hit by the harm stages, as `figure_swing`'s.
    row(
        "zombie_in_your_gear",
        5,
        5,
        &[Night, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::ZombieInYourGear(4.0),
        false,
    ),
    // docs/events_todo.md, 99.
    row(
        "wolves_circle",
        4,
        5,
        &[Night, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::WolvesCircle,
        false,
    ),
    // docs/events_todo.md, 98.
    row(
        "drops_from_the_ceiling",
        4,
        5,
        &[TargetUnderground],
        2.0,
        3,
        Full,
        Action::DropsFromTheCeiling,
        false,
    ),
    // docs/events_todo.md, 97.
    row(
        "daylight_phantom",
        4,
        5,
        &[Noon, TargetOutdoors],
        3.0,
        2,
        Full,
        Action::DaylightPhantom,
        false,
    ),
    // docs/events_todo.md, 96.
    row(
        "wall_bursts",
        4,
        5,
        &[TargetIndoors, Home],
        3.0,
        2,
        Full,
        Action::WallBursts,
        false,
    ),
    // docs/events_todo.md, 95.
    row(
        "villager_collapses",
        4,
        5,
        &[Creature("villager")],
        3.0,
        2,
        Full,
        Action::VillagerCollapses,
        false,
    ),
    // docs/events_todo.md, 94.
    row(
        "villagers_walk_out",
        4,
        5,
        &[Dusk, Creature("villager"), Around("village")],
        3.0,
        2,
        Full,
        Action::VillagersWalkOut,
        false,
    ),
    // docs/events_todo.md, 93.
    row(
        "villager_spins",
        4,
        5,
        &[Creature("villager")],
        2.0,
        3,
        Full,
        Action::VillagerSpins,
        false,
    ),
    // docs/events_todo.md, 92.
    row(
        "trades_refused",
        3,
        5,
        &[Creature("villager")],
        3.0,
        2,
        Full,
        Action::TradesRefused,
        false,
    ),
    // docs/events_todo.md, 111 (owner's): the stab by the harm stages, two hearts as
    // `figure_swing`'s.
    row(
        "villager_stabs",
        4,
        5,
        &[Creature("villager")],
        4.0,
        2,
        Full,
        Action::VillagerStabs(4.0),
        false,
    ),
    // docs/events_todo.md, 91.
    row(
        "villagers_flee_you",
        4,
        5,
        &[EnteringVillage],
        3.0,
        2,
        Full,
        Action::VillagersFleeYou,
        false,
    ),
    // docs/events_todo.md, 90.
    row(
        "circles_a_tree",
        3,
        5,
        &[InTheWoods, SnowyHere],
        2.0,
        2,
        Full,
        Action::CirclesATree,
        false,
    ),
    // docs/events_todo.md, 89.
    row(
        "animals_flee_the_woods",
        4,
        5,
        &[InTheWoods, AnimalsNearTarget],
        2.0,
        2,
        Full,
        Action::AnimalsFleeTheWoods,
        false,
    ),
    // docs/events_todo.md, 88.
    row(
        "tree_comes_down",
        4,
        5,
        &[InTheWoods, TargetWalking],
        2.0,
        2,
        Full,
        Action::TreeComesDown,
        false,
    ),
    // docs/events_todo.md, 87.
    row(
        "trees_close_behind",
        4,
        5,
        &[InTheWoods, TargetWalking],
        2.0,
        2,
        Full,
        Action::TreesCloseBehind,
        false,
    ),
    // docs/events_todo.md, 86.
    row(
        "the_tree_line",
        5,
        5,
        &[LeavingTheWoods],
        2.0,
        2,
        Full,
        Action::TheTreeLine,
        false,
    ),
    // docs/events_todo.md, 85.
    row(
        "branch_snaps",
        3,
        5,
        &[InTheWoods, TargetWalking],
        1.0,
        3,
        Full,
        Action::BranchSnaps,
        false,
    ),
    // docs/events_todo.md, 84.
    row(
        "blazed_trees",
        3,
        5,
        &[InTheWoods],
        1.5,
        3,
        Full,
        Action::BlazedTrees,
        false,
    ),
    // docs/events_todo.md, 83.
    row(
        "knocks_on_wood",
        3,
        5,
        &[InTheWoods],
        1.0,
        3,
        Full,
        Action::KnocksOnWood,
        false,
    ),
    // docs/events_todo.md, 82.
    row(
        "tree_to_tree",
        4,
        5,
        &[InTheWoods],
        1.5,
        3,
        Full,
        Action::TreeToTree,
        false,
    ),
    // docs/events_todo.md, 116.
    row(
        "breathing_in_the_wall",
        3,
        5,
        &[TargetIndoors],
        1.5,
        3,
        Full,
        Action::SoundAtTarget("minecraft:entity.player.breath"),
        false,
    ),
    // docs/events_todo.md, 117.
    row(
        "scream_indoors",
        4,
        5,
        &[TargetIndoors, Night],
        3.0,
        2,
        Full,
        Action::SoundAtTarget("minecraft:entity.goat.screaming.ambient"),
        false,
    ),
    // docs/events_todo.md, 118.
    row(
        "item_break_behind",
        3,
        5,
        &[TargetAwake],
        1.0,
        4,
        Half,
        Action::SoundAtTarget("minecraft:entity.item.break"),
        true,
    ),
    // docs/events_todo.md, 119.
    row(
        "armour_behind",
        4,
        5,
        &[TargetWalking, Night],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:item.armor.equip_iron"),
        false,
    ),
    // docs/events_todo.md, 120.
    row(
        "breath_in_the_chest",
        3,
        5,
        &[UsedContainer, TargetIndoors],
        2.0,
        3,
        Full,
        Action::Sound("minecraft:entity.player.breath"),
        false,
    ),
    // docs/events_todo.md, 121.
    row(
        "rain_on_the_roof",
        3,
        5,
        &[TargetIndoors, NotRaining],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:weather.rain.above"),
        false,
    ),
    // docs/events_todo.md, 122.
    row(
        "sniffing",
        4,
        5,
        &[Night, TargetOutdoors],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:entity.sniffer.sniffing"),
        false,
    ),
    // docs/events_todo.md, 123.
    row(
        "gasp_when_turning",
        4,
        5,
        &[TargetWalking, Night],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:entity.player.breath"),
        false,
    ),
    // docs/events_todo.md, 124.
    row(
        "water_where_none",
        3,
        5,
        &[TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.player.swim"),
        true,
    ),
    // docs/events_todo.md, 125.
    row(
        "doors_slam_far",
        4,
        5,
        &[Night, Around("village")],
        3.0,
        2,
        Full,
        Action::SoundFar("minecraft:block.wooden_door.close"),
        false,
    ),
    // docs/events_todo.md, 126.
    row(
        "the_count",
        4,
        5,
        &[Dusk, Home],
        0.8,
        9,
        Full,
        Action::Sound("minecraft:block.bell.use"),
        false,
    ),
    // docs/events_todo.md, 128.
    row(
        "rattling_round_the_house",
        4,
        5,
        &[TargetIndoors, Night, Home],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:entity.skeleton.step"),
        false,
    ),
    // docs/events_todo.md, 129.
    row(
        "chains_dragging",
        4,
        5,
        &[TargetUnderground],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:block.chain.step"),
        false,
    ),
    // docs/events_todo.md, 130.
    row(
        "shriek_below",
        5,
        5,
        &[Home, TargetIndoors],
        4.0,
        1,
        Full,
        Action::SoundAtTarget("minecraft:block.sculk_shrieker.shriek"),
        false,
    ),
    // docs/events_todo.md, 131.
    row(
        "witch_laughs",
        3,
        5,
        &[Night, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::SoundFar("minecraft:entity.witch.celebrate"),
        true,
    ),
    // docs/events_todo.md, 132.
    row(
        "pet_whines_at_door",
        3,
        5,
        &[Pet, Door, Night],
        3.0,
        2,
        Full,
        Action::SoundAtTarget("minecraft:entity.wolf.whine"),
        false,
    ),
    // docs/events_todo.md, 134.
    row(
        "glass_rubbed",
        3,
        5,
        &[Pane, Night, TargetIndoors],
        2.0,
        3,
        Full,
        Action::Sound("minecraft:block.glass.step"),
        false,
    ),
    // docs/events_todo.md, 135.
    row(
        "bones_underfoot",
        4,
        5,
        &[TargetIndoors, TargetWalking],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:block.bone_block.step"),
        false,
    ),
    // docs/events_todo.md, 136.
    row(
        "bubbling_under",
        3,
        5,
        &[TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.bubble_column.upwards_ambient"),
        true,
    ),
    // docs/events_todo.md, 137.
    row(
        "the_bell_in_the_chest",
        4,
        5,
        &[UsedContainer, TargetIndoors],
        2.0,
        2,
        Full,
        Action::Sound("minecraft:block.bell.use"),
        false,
    ),
    // docs/events_todo.md, 138.
    row(
        "panting_behind",
        4,
        5,
        &[TargetRunning, Night],
        2.0,
        3,
        Full,
        Action::SoundAtTarget("minecraft:entity.player.breath"),
        false,
    ),
    // docs/events_todo.md, 139.
    row(
        "iron_door_below",
        4,
        5,
        &[TargetUnderground],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.iron_door.close"),
        true,
    ),
    // docs/events_todo.md, 140.
    row(
        "ice_cracks",
        3,
        5,
        &[SnowyHere, TargetWalking, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.glass.break"),
        true,
    ),
    // docs/events_todo.md, 141.
    row(
        "bush_rustles",
        3,
        5,
        &[TargetOutdoors, TargetWalking],
        1.0,
        4,
        Half,
        Action::SoundAtTarget("minecraft:block.sweet_berry_bush.place"),
        true,
    ),
    // docs/events_todo.md, 142.
    row(
        "snoring_beside",
        4,
        5,
        &[TargetInBed],
        3.0,
        2,
        Full,
        Action::SoundAtTarget("minecraft:entity.fox.sleep"),
        false,
    ),
    // docs/events_todo.md, 166.
    row(
        "sigh_at_the_door",
        3,
        5,
        &[Home, Door, TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.player.breath"),
        true,
    ),
    // docs/events_todo.md, 167.
    row(
        "humming_ahead",
        3,
        5,
        &[TargetUnderground, TargetWalking],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.villager.ambient"),
        false,
    ),
    // docs/events_todo.md, 168: in the dark, each step a note one higher, eight, then nothing.
    row(
        "steps_counted",
        4,
        5,
        &[TargetWalking, Night],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.note_block.harp"),
        false,
    ),
    // docs/events_todo.md, 169.
    row(
        "cart_overhead",
        3,
        5,
        &[TargetUnderground],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.minecart.riding"),
        false,
    ),
    // docs/events_todo.md, 170.
    row(
        "ram_on_the_wall",
        4,
        5,
        &[Home, TargetIndoors, Night],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.goat.ram_impact"),
        false,
    ),
    // docs/events_todo.md, 171.
    row(
        "bell_underwater",
        4,
        5,
        &[TargetSwimming],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.bell.use"),
        false,
    ),
    // docs/events_todo.md, 172.
    row(
        "bark_from_home",
        3,
        5,
        &[Pet, TargetOutdoors, Home],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.wolf.ambient"),
        false,
    ),
    // docs/events_todo.md, 173: hoofbeats gallop a loop round the house at night, and away.
    row(
        "hoofbeats_circling",
        4,
        5,
        &[TargetIndoors, Night, Home],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.horse.gallop"),
        false,
    ),
    // docs/events_todo.md, 174.
    row(
        "snipping_in_the_dark",
        3,
        5,
        &[Creature("sheep"), Night],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.sheep.shear"),
        false,
    ),
    // docs/events_todo.md, 175.
    row(
        "eating_in_the_wall",
        3,
        5,
        &[TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.generic.eat"),
        false,
    ),
    // docs/events_todo.md, 176.
    row(
        "ominous_hum",
        4,
        5,
        &[Home, TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.trial_spawner.ominous_activate"),
        false,
    ),
    // docs/events_todo.md, 177.
    row(
        "vault_opens_by_the_bed",
        4,
        5,
        &[TargetInBed],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.vault.open_shutter"),
        false,
    ),
    // docs/events_todo.md, 178.
    row(
        "creaking_behind",
        4,
        5,
        &[InTheWoods, TargetWalking],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.creaking.step"),
        false,
    ),
    // docs/events_todo.md, 179.
    row(
        "bulbs_clicking",
        3,
        5,
        &[TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.copper_bulb.turn_on"),
        false,
    ),
    // docs/events_todo.md, 180: a wind charge's burst beside them and a soft push.
    row(
        "gust_from_nowhere",
        4,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.wind_charge.wind_burst"),
        false,
    ),
    // docs/events_todo.md, 181.
    row(
        "digging_under_the_floor",
        3,
        5,
        &[TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.sniffer.digging"),
        false,
    ),
    // docs/events_todo.md, 182: the next time they open their inventory, a bundle rustles in it.
    row(
        "rustle_in_the_pack",
        3,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Hint("rustle_in_the_pack", 120.0),
        true,
    ),
    // docs/events_todo.md, 183.
    row(
        "wings_in_the_room",
        4,
        5,
        &[TargetIndoors, Night, TargetAwake],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:entity.phantom.flap"),
        false,
    ),
    // docs/events_todo.md, 184.
    row(
        "lava_in_the_wall",
        4,
        5,
        &[TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.lava.pop"),
        false,
    ),
    // docs/events_todo.md, 185.
    row(
        "crossbow_click",
        4,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:item.crossbow.loading_end"),
        false,
    ),
    // docs/events_todo.md, 187.
    row(
        "teleport_beside",
        4,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:item.chorus_fruit.teleport"),
        false,
    ),
    // docs/events_todo.md, 188.
    row(
        "sweeping_next_room",
        3,
        5,
        &[Home, TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:item.brush.brushing.generic"),
        true,
    ),
    // docs/events_todo.md, 189.
    row(
        "pot_shatters",
        3,
        5,
        &[TargetIndoors],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:block.decorated_pot.shatter"),
        false,
    ),
    // docs/events_todo.md, 186: a blow lands on their shield from nothing, and shoves them back.
    row(
        "blow_on_the_shield",
        4,
        5,
        &[HoldsShield],
        2.0,
        3,
        Half,
        Action::SoundAtTarget("minecraft:item.shield.block"),
        false,
    ),
    // docs/events_todo.md, 81 (owner's).
    row(
        "peeks_from_the_trees",
        4,
        5,
        &[InTheWoods, TargetWalking],
        1.0,
        4,
        Full,
        Action::PeeksFromTheTrees,
        false,
    ),
    // docs/events_todo.md, 77.
    row(
        "crouched_behind_a_tree",
        4,
        5,
        &[TargetOutdoors],
        3.0,
        3,
        Full,
        Action::CrouchedBehindATree,
        false,
    ),
    // docs/events_todo.md, 78.
    row(
        "snowball_thrown",
        3,
        5,
        &[Around("snow"), TargetOutdoors],
        2.0,
        3,
        Full,
        Action::SnowballThrown,
        false,
    ),
    // docs/events_todo.md, 79.
    row(
        "arrow_in_the_door",
        4,
        5,
        &[TargetAwayOrAsleep, Door, Home],
        5.0,
        1,
        Full,
        Action::ArrowInTheDoor,
        false,
    ),
    // docs/events_todo.md, 80.
    row(
        "raises_their_sword",
        4,
        5,
        &[TargetOutdoors],
        3.0,
        2,
        Full,
        Action::RaisesTheirSword,
        false,
    ),
    // docs/events_todo.md, 53: heard inside, every pane whole.
    row(
        "glass_breaks",
        4,
        5,
        &[TargetIndoors, Fixture("glass_pane")],
        3.0,
        2,
        Half,
        Action::SoundAtFixture("glass_pane", "minecraft:block.glass.break"),
        true,
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
        &[TheirBoat, Around("water"), TargetAwayOrAsleep],
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
        &[TheirBoat, Around("water"), TargetAwayOrAsleep],
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
        3,
        &[Home, TargetIndoors, Night],
        1.0,
        4,
        Half,
        Action::Taps("", "minecraft:block.wooden_door.close", 3),
        true,
    ),
    row(
        "knock_heavy",
        4,
        5,
        &[Home, TargetIndoors, Night],
        1.0,
        4,
        Full,
        Action::Taps("", "minecraft:entity.zombie.attack_wooden_door", 3),
        false,
    ),
    row(
        "place_sound",
        1,
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
        1,
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
    // docs/events_todo.md, 143: overnight every wall torch of the house on another wall of its
    // block, all pointing at their bed.
    row(
        "torches_turn",
        4,
        5,
        &[Home, Bed, Light, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::TorchesTurn,
        false,
    ),
    // docs/events_todo.md, 145: at its place, a grave with their own name and a day not come.
    row(
        "your_grave",
        5,
        5,
        &[Home, TargetAwayOrAsleep],
        3.0,
        1,
        Full,
        Action::YourGrave,
        false,
    ),
    // docs/events_todo.md, 151: overnight their house hung with cobwebs, gone in a day.
    row(
        "cobwebs",
        3,
        5,
        &[Home, Walls, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::Cobwebs,
        false,
    ),
    // docs/events_todo.md, 157: overnight a door on an inside wall, the wall behind it.
    row(
        "false_door",
        4,
        5,
        &[Home, Walls, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::FalseDoor,
        false,
    ),
    // docs/events_todo.md, 144: coming home, red specks on the floor from the door to the bed.
    row(
        "red_trail",
        4,
        5,
        &[Home, Bed, Door, TargetApproachingDoor],
        2.0,
        2,
        Half,
        Action::Hint("red_trail", 60.0),
        false,
    ),
    // docs/events_todo.md, 146: a hollow in the snow by the house the shape of a person lying.
    row(
        "snow_shape",
        4,
        5,
        &[Home, SnowyHere, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::SnowShape,
        false,
    ),
    // docs/events_todo.md, 147: the clock in their hand shows night at noon a while.
    row(
        "clock_wrong",
        3,
        5,
        &[Holding("minecraft:clock"), Day],
        1.0,
        3,
        Half,
        Action::Hint("clock_wrong", 90.0),
        true,
    ),
    // docs/events_todo.md, 149: the house goes cold, their breath shows, frost on the floor.
    row(
        "cold_house",
        4,
        5,
        &[Home, TargetIndoors],
        2.0,
        2,
        Half,
        Action::ColdHouse,
        false,
    ),
    // docs/events_todo.md, 150: a white rabbit watching them comes at them red-eyed; a nip.
    row(
        "the_killer_rabbit",
        4,
        5,
        &[TargetOutdoors, TargetAwake],
        2.0,
        2,
        Full,
        Action::KillerRabbit(2.0),
        false,
    ),
    // docs/events_todo.md, 152: a paper named "behind you" in an empty slot of their inventory.
    row(
        "phantom_item",
        4,
        5,
        &[TargetAwake],
        1.0,
        2,
        Half,
        Action::Hint("phantom_item", 120.0),
        false,
    ),
    // docs/events_todo.md, 153: the world border's red edge close round them a moment.
    row(
        "border_closes",
        5,
        5,
        &[TargetAwake],
        2.0,
        2,
        Full,
        Action::Hint("border_closes", 2.5),
        false,
    ),
    // docs/events_todo.md, 162: their windows frosted from inside a while, though not cold.
    row(
        "frost_on_the_glass",
        3,
        5,
        &[Home, Pane, TargetIndoors],
        1.0,
        3,
        Half,
        Action::FrostOnGlass,
        false,
    ),
    // docs/events_todo.md, 148: indoors at night, a shadow of a person on the wall behind them
    // cast by the light they hold, gone when they turn.
    row(
        "shadow_on_the_wall",
        4,
        5,
        &[TargetIndoors, Night],
        2.0,
        2,
        Half,
        Action::Hint("shadow_on_the_wall", 20.0),
        false,
    ),
    // docs/events_todo.md, 158: overnight, stairs up from their floor into the ceiling.
    row(
        "stairs_up_to_nowhere",
        4,
        5,
        &[Home, Walls, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::StairsToNowhere,
        false,
    ),
    // docs/events_todo.md, 159: a face cut into the leaves of a tree by the house.
    row(
        "face_in_the_canopy",
        4,
        5,
        &[Home, Door, TargetAwayOrAsleep],
        2.0,
        1,
        Half,
        Action::FaceInTheCanopy,
        false,
    ),
    // docs/events_todo.md, 161: coming home, boots like theirs just inside the door.
    row(
        "boots_at_the_door",
        4,
        5,
        &[Home, Door, TargetApproachingDoor],
        2.0,
        2,
        Half,
        Action::BootsAtTheDoor,
        false,
    ),
    // docs/events_todo.md, 163: as they wake, one frame of the figure's face fills the screen.
    row(
        "nightmare",
        5,
        5,
        &[JustWoke],
        2.0,
        2,
        Full,
        Action::Hint("nightmare", 0.15),
        false,
    ),
    // docs/events_todo.md, 154: at night in the house, the figure crawls across the ceiling
    // over them, upside down.
    row(
        "ceiling_crawl",
        5,
        5,
        &[TargetIndoors, Night, Home],
        2.0,
        2,
        Full,
        Action::CeilingCrawl,
        false,
    ),
    // docs/events_todo.md, 155: underground, the figure in a lava lake, unburnt, facing them.
    row(
        "in_the_lava",
        4,
        5,
        &[TargetUnderground],
        2.0,
        2,
        Full,
        Action::InTheLava,
        false,
    ),
    // docs/events_todo.md, 156: at a cliff's foot, a head peeks over the edge high above.
    row(
        "over_the_edge",
        4,
        5,
        &[TargetOutdoors, TargetAwake],
        2.0,
        2,
        Full,
        Action::OverTheEdge,
        false,
    ),
    // docs/events_todo.md, 160: at dusk the figure on a skeleton horse far off, watching.
    row(
        "rider_on_the_ridge",
        4,
        5,
        &[Dusk, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::RiderOnTheRidge,
        false,
    ),
    // docs/events_todo.md, 190: through a spyglass, far off, the figure looks back.
    row(
        "spyglass_figure",
        4,
        5,
        &[TargetOutdoors, TargetAwake],
        2.0,
        3,
        Half,
        Action::Hint("spyglass_figure", 300.0),
        false,
    ),
    // docs/events_todo.md, 191: their recovery compass points at its place, not their death.
    row(
        "recovery_compass_lies",
        3,
        5,
        &[Holding("minecraft:recovery_compass")],
        2.0,
        3,
        Half,
        Action::Hint("recovery_compass_lies", 120.0),
        true,
    ),
    // docs/events_todo.md, 192: a boss bar reading "it", filling slowly over a minute.
    row(
        "boss_bar_it",
        5,
        5,
        &[TargetAwake],
        2.0,
        2,
        Full,
        Action::BossBarIt,
        false,
    ),
    // docs/events_todo.md, 193: at night a copper golem stands still outside their door.
    row(
        "copper_golem_at_door",
        4,
        5,
        &[Home, Door, Night],
        2.0,
        2,
        Half,
        Action::CopperGolemAtDoor,
        false,
    ),
    // docs/events_todo.md, 194: a creaking in their field at night, nearer each time they look away.
    row(
        "creaking_in_the_field",
        5,
        5,
        &[Field, Night],
        2.0,
        2,
        Full,
        Action::CreakingInTheField,
        false,
    ),
    // docs/events_todo.md, 195: pale moss creeps across their floor overnight, more each night.
    row(
        "pale_moss_creeps",
        4,
        5,
        &[Home, TargetAwayOrAsleep],
        1.0,
        5,
        Half,
        Action::PaleMossCreeps,
        false,
    ),
    // docs/events_todo.md, 196: a patch of suspicious sand by the house; brushed, it gives
    // back what it took.
    row(
        "sand_gives_back",
        4,
        5,
        &[Held, Home, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::KeepIn("minecraft:suspicious_sand"),
        false,
    ),
    // docs/events_todo.md, 197: a decorated pot at its place; broken, it gives back what it took.
    row(
        "pot_of_returns",
        4,
        5,
        &[Held, Home, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::KeepIn("minecraft:decorated_pot"),
        false,
    ),
    // docs/events_todo.md, 198: their crafter clicks and crafts once by itself.
    row(
        "crafter_clicks",
        3,
        5,
        &[Fixture("crafter")],
        2.0,
        3,
        Half,
        Action::ToggleFixture("crafter"),
        true,
    ),
    // docs/events_todo.md, 199: at dusk a happy ghast over the house, the figure riding it.
    row(
        "happy_ghast_overhead",
        4,
        5,
        &[Dusk, TargetOutdoors, Home],
        2.0,
        2,
        Full,
        Action::HappyGhastOverhead,
        false,
    ),
    // docs/events_todo.md, 200: overnight one window of theirs is tinted glass, a day.
    row(
        "window_tinted",
        4,
        5,
        &[Home, Pane, TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::WindowTinted,
        false,
    ),
    // docs/events_todo.md, 201: dark footprints across the ceiling, from the door to over the bed.
    row(
        "prints_on_the_ceiling",
        4,
        5,
        &[Home, Bed, Door, TargetIndoors],
        2.0,
        2,
        Half,
        Action::Hint("prints_on_the_ceiling", 60.0),
        false,
    ),
    // docs/events_todo.md, 202: the big dripleaves near them tip one after another toward them.
    row(
        "dripleaf_steps",
        3,
        5,
        &[TargetOutdoors, TargetAwake],
        2.0,
        3,
        Half,
        Action::DripleafSteps,
        true,
    ),
    // docs/events_todo.md, 203: every axolotl near them floats still at once, a while.
    row(
        "axolotls_play_dead",
        3,
        5,
        &[Creature("axolotl")],
        2.0,
        3,
        Half,
        Action::AxolotlsPlayDead,
        true,
    ),
    // docs/events_todo.md, 204: sculk veins spread over their walls low down, more each night.
    row(
        "sculk_creeps",
        5,
        5,
        &[Home, Walls, TargetAwayOrAsleep],
        1.0,
        5,
        Half,
        Action::SculkCreeps,
        false,
    ),
    // docs/events_todo.md, 205: cracks spread over a block of their wall, then vanish.
    row(
        "cracks_in_the_wall",
        4,
        5,
        &[Home, Walls, TargetIndoors],
        2.0,
        3,
        Half,
        Action::CracksInTheWall,
        false,
    ),
    // docs/events_todo.md, 206: a charged respawn anchor glowing at its place, on their screen alone.
    row(
        "anchor_charged",
        5,
        5,
        &[Home, Night, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::AnchorCharged,
        false,
    ),
    // docs/events_todo.md, 207: pointed dripstone falls beside them and shatters.
    row(
        "dripstone_falls",
        4,
        5,
        &[TargetUnderground, TargetAwake],
        1.0,
        3,
        Half,
        Action::DripstoneFalls,
        false,
    ),
    // docs/events_todo.md, 208: a wither rose in one of their pots in the morning.
    row(
        "wither_rose_in_pot",
        4,
        5,
        &[Fixture("flower_pot"), JustWoke],
        3.0,
        2,
        Half,
        Action::WitherRoseInPot,
        false,
    ),
    // docs/events_todo.md, 209: mushrooms in the dark corners of their house overnight.
    row(
        "mushrooms_in_corners",
        3,
        5,
        &[Home, Walls, TargetAwayOrAsleep],
        2.0,
        3,
        Half,
        Action::MushroomsInCorners,
        false,
    ),
    // docs/events_todo.md, 210: their shulker box opens a crack by itself and shuts.
    row(
        "shulker_lid_peeks",
        4,
        5,
        &[Fixture("shulker_box"), TargetIndoors],
        1.0,
        3,
        Half,
        Action::ShulkerLidPeeks,
        false,
    ),
    // docs/events_todo.md, 211: a sign by their door with their bed's coordinates.
    row(
        "sign_with_coords",
        4,
        5,
        &[Home, Bed, Door, TargetAwayOrAsleep],
        3.0,
        1,
        Half,
        Action::SignWithCoords,
        false,
    ),
    // docs/events_todo.md, 212: a trident stands upright in the ground at its place.
    row(
        "trident_planted",
        4,
        5,
        &[Home, TargetAwayOrAsleep],
        3.0,
        2,
        Half,
        Action::TridentPlanted,
        false,
    ),
    // docs/events_todo.md, 213: a mace's smash at night near the house, a bowl in the ground there by morning.
    row(
        "slam_crater",
        5,
        5,
        &[Home, Night, TargetIndoors],
        3.0,
        2,
        Full,
        Action::SlamCrater,
        false,
    ),
    // docs/events_todo.md, 214: one more step at the bottom of their staircase, into the floor.
    row(
        "one_step_more",
        4,
        5,
        &[Fixtures("stairs", 2), JustWoke],
        1.0,
        3,
        Half,
        Action::OneStepMore,
        false,
    ),
    // docs/events_todo.md, 215: one wall of their room a block further in.
    row(
        "room_closes_in",
        5,
        5,
        &[Home, Walls, JustWoke],
        3.0,
        2,
        Full,
        Action::RoomClosesIn,
        false,
    ),
    // docs/events_todo.md, 216: a trapdoor in their floor over a pit with a torch at the bottom.
    row(
        "trapdoor_to_nothing",
        4,
        5,
        &[Home, Walls, TargetAwayOrAsleep],
        4.0,
        1,
        Full,
        Action::TrapdoorToNothing,
        false,
    ),
    // docs/events_todo.md, 217: a ring of fences round their bed by morning.
    row(
        "bed_fenced",
        4,
        5,
        &[Home, Bed, TargetInBed],
        4.0,
        1,
        Full,
        Action::BedFenced,
        false,
    ),
    // docs/events_todo.md, 218: their tools laid out on the floor in a neat row.
    row(
        "tools_laid_out",
        4,
        5,
        &[Home, ToolInContainer, TargetAwayOrAsleep],
        3.0,
        1,
        Full,
        Action::ToolsLaidOut,
        false,
    ),
    // docs/events_todo.md, 219: a column of dripstone across their tunnel overnight.
    row(
        "dripstone_in_tunnel",
        4,
        5,
        &[Tunnel, TargetAwayOrAsleep],
        3.0,
        2,
        Half,
        Action::DripstoneInTunnel,
        false,
    ),
    // docs/events_todo.md, 220: a skeleton skull in the wall of their tunnel, facing them as they come.
    row(
        "skull_in_the_tunnel",
        4,
        5,
        &[InTunnel],
        2.0,
        2,
        Half,
        Action::SkullInTheTunnel,
        false,
    ),
    // docs/events_todo.md, 221: their copper a stage more weathered in the morning.
    row(
        "copper_ages",
        3,
        5,
        &[Copper, TargetAwayOrAsleep],
        2.0,
        3,
        Half,
        Action::CopperAges,
        false,
    ),
    // docs/events_todo.md, 222: a skeleton horse stands among the animals in their pen at night.
    row(
        "pale_horse_in_your_pen",
        4,
        5,
        &[Home, Night, Penned(2)],
        3.0,
        2,
        Half,
        Action::PaleHorse,
        false,
    ),
    // docs/events_todo.md, 223: a line of evoker fangs snaps up toward them and stops short.
    row(
        "fangs_in_a_line",
        5,
        5,
        &[TargetOutdoors, TargetAwake],
        2.0,
        2,
        Full,
        Action::FangsInALine,
        false,
    ),
    // docs/events_todo.md, 224: a guardian's beam charges at them from the water and never fires.
    row(
        "beam_from_the_water",
        5,
        5,
        &[WaterNear(32), TargetOutdoors],
        2.0,
        2,
        Full,
        Action::BeamFromTheWater,
        false,
    ),
    // docs/events_todo.md, 225: the fish in their bucket is another kind in the morning.
    row(
        "bucket_fish_changed",
        3,
        5,
        &[InChestAny(FISH_BUCKETS), TargetAwayOrAsleep],
        2.0,
        2,
        Half,
        Action::BucketFishChanged,
        false,
    ),
    // docs/events_todo.md, 226: a head with their own face on their pillow in the morning.
    row(
        "head_on_the_pillow",
        5,
        5,
        &[Home, Bed, JustWoke],
        4.0,
        1,
        Full,
        Action::HeadOnThePillow,
        false,
    ),
    // docs/events_todo.md, 227: a lead knotted on a fence post at its place, nothing on its end.
    row(
        "empty_lead",
        3,
        5,
        &[Home, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::EmptyLead,
        false,
    ),
    // docs/events_todo.md, 228: a dried ghast sits on their doorstep.
    row(
        "dried_ghast_on_doorstep",
        4,
        5,
        &[Home, Door, TargetApproachingDoor],
        3.0,
        1,
        Half,
        Action::DriedGhastOnDoorstep,
        false,
    ),
    // docs/events_todo.md, 229: by day, the eyeblossoms near them open all at once.
    row(
        "eyeblossoms_open",
        3,
        5,
        &[Day, TargetOutdoors],
        1.0,
        3,
        Half,
        Action::EyeblossomsOpen,
        false,
    ),
    // docs/events_todo.md, 230: a copper golem statue by their door keeps turning to face them.
    row(
        "golem_statue_turns",
        4,
        5,
        &[Home, Door, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::GolemStatueTurns,
        false,
    ),
    // docs/events_todo.md, 231: the things on their shelf in another order in the morning.
    row(
        "shelf_shuffled",
        3,
        5,
        &[Fixture("shelf"), TargetAwayOrAsleep],
        1.0,
        3,
        Half,
        Action::ShelfShuffled,
        false,
    ),
    // docs/events_todo.md, 232: at night the firefly bushes near them go dark one by one toward them.
    row(
        "fireflies_go_out",
        3,
        5,
        &[Night, TargetOutdoors],
        1.0,
        3,
        Half,
        Action::FirefliesGoOut,
        false,
    ),
    // docs/events_todo.md, 233: a trial key lies on their doorstep in the morning.
    row(
        "trial_key_left",
        3,
        5,
        &[Home, Door, JustWoke],
        3.0,
        1,
        Half,
        Action::TrialKeyLeft,
        false,
    ),
    // docs/events_todo.md, 234: their own name fills their screen for a second at night.
    row(
        "title_your_name",
        4,
        5,
        &[TargetAwake, Night],
        2.0,
        2,
        Half,
        Action::TitleYourName,
        false,
    ),
    // docs/events_todo.md, 235: the game's own toast pops up, new recipes unlocked, and the thing unlocked is their own head, named "you".
    row(
        "recipe_toast",
        3,
        5,
        &[TargetAwake],
        1.0,
        3,
        Half,
        Action::Hint("recipe_toast", 1.0),
        false,
    ),
    // docs/events_todo.md, 236: for three seconds every stack in their hotbar shows a count of 0, even a tool's.
    row(
        "count_shows_zero",
        3,
        5,
        &[TargetAwake],
        1.0,
        3,
        Half,
        Action::Hint("count_shows_zero", 3.0),
        false,
    ),
    // docs/events_todo.md, 237: for two minutes, whenever they open their inventory, one empty slot shows a barrier in it, the same slot each time.
    row(
        "slot_barred",
        4,
        5,
        &[TargetAwake],
        1.0,
        3,
        Half,
        Action::Hint("slot_barred", 120.0),
        false,
    ),
    // docs/events_todo.md, 238: for a minute, each time they switch item, the name that pops up over the hotbar reads "its".
    row(
        "held_name_wrong",
        3,
        5,
        &[TargetAwake],
        1.0,
        3,
        Half,
        Action::Hint("held_name_wrong", 60.0),
        false,
    ),
    // docs/events_todo.md, 239: armed now, the next time the game's title screen comes up its yellow splash text reads "we watched you sleep", once.
    row(
        "title_splash",
        4,
        5,
        &[TargetAwake],
        5.0,
        1,
        Half,
        Action::Hint("title_splash", 1.0),
        false,
    ),
    // docs/events_todo.md, 240: for ten minutes, at any enchanting table, the three offers all read a cost of 6.
    row(
        "enchant_six",
        4,
        5,
        &[TargetAwake],
        1.0,
        3,
        Half,
        Action::Hint("enchant_six", 600.0),
        false,
    ),
    // docs/events_todo.md, 241: for a minute their compass spins as it does in the nether, here in the overworld, and then points home again.
    row(
        "compass_spins",
        3,
        5,
        &[Holding("minecraft:compass")],
        1.0,
        3,
        Half,
        Action::Hint("compass_spins", 60.0),
        false,
    ),
    // docs/events_todo.md, 242: the figure digs at its place, a fresh hole there after.
    row(
        "figure_digging",
        4,
        5,
        &[Home, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::FigureDigging,
        false,
    ),
    // docs/events_todo.md, 243: far off, the figure bows to them and is gone.
    row(
        "figure_bows",
        4,
        5,
        &[TargetOutdoors],
        2.0,
        2,
        Half,
        Action::FigureBows,
        false,
    ),
    // docs/events_todo.md, 244: at night the figure climbs the outside of the house past a window.
    row(
        "climbing_the_house",
        5,
        5,
        &[Home, Night, TargetIndoors, Pane],
        3.0,
        2,
        Full,
        Action::ClimbingTheHouse,
        false,
    ),
    // docs/events_todo.md, 245: the figure stands at their lectern, reading.
    row(
        "figure_at_the_lectern",
        4,
        5,
        &[Home, Fixture("lectern"), TargetOutdoors],
        3.0,
        2,
        Half,
        Action::FigureAtTheLectern,
        false,
    ),
    // docs/events_todo.md, 246: the figure far off wears exactly the armour they wear.
    row(
        "figure_in_your_armour",
        4,
        5,
        &[TargetOutdoors],
        2.0,
        2,
        Half,
        Action::FigureInYourArmour,
        false,
    ),
    // docs/events_todo.md, 247: at sunset the figure stands on the top of the nearest mountain.
    row(
        "on_the_peak",
        4,
        5,
        &[Dusk, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::OnThePeak,
        false,
    ),
    // docs/events_todo.md, 248: all the cows near them turn their backs on them at once.
    row(
        "cattle_turn_away",
        3,
        5,
        &[Creature("cow"), TargetOutdoors],
        1.0,
        3,
        Half,
        Action::CattleTurnAway,
        false,
    ),
    // docs/events_todo.md, 249: their horse will not move a few seconds, staring at nothing ahead.
    row(
        "horse_balks",
        4,
        5,
        &[Riding("horse")],
        1.0,
        3,
        Half,
        Action::HorseBalks,
        false,
    ),
    // docs/events_todo.md, 250: their cat lies by their bed in the morning with a gift from their chest.
    row(
        "cat_gift",
        3,
        5,
        &[PetOf("cat"), Bed, UsedContainer, TargetInBed],
        2.0,
        2,
        Half,
        Action::CatGift,
        false,
    ),
    // docs/events_todo.md, 251: every armadillo near them curls up at once.
    row(
        "armadillos_curl",
        3,
        5,
        &[Creature("armadillo"), TargetOutdoors],
        1.0,
        3,
        Half,
        Action::ArmadillosCurl,
        false,
    ),
    // docs/events_todo.md, 252: a sniffer snuffles and digs at its place, then is gone.
    row(
        "sniffer_at_its_place",
        4,
        5,
        &[Home, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::SnifferAtItsPlace,
        false,
    ),
    // docs/events_todo.md, 253: the next animal they feed turns its head away and will not eat.
    row(
        "refuses_food",
        3,
        5,
        &[Animals, TargetOutdoors],
        1.0,
        3,
        Half,
        Action::RefusesFood,
        false,
    ),
    // docs/events_todo.md, 254: the young animals near them stare at them wherever they go.
    row(
        "the_child_stares",
        4,
        5,
        &[BabyNear],
        1.0,
        3,
        Half,
        Action::TheChildStares,
        false,
    ),
    // docs/events_todo.md, 255: what they mine drops a moment late, as if held.
    row(
        "late_drop",
        3,
        5,
        &[TargetUnderground, TargetAwake],
        1.0,
        3,
        Half,
        Action::LateDrop,
        false,
    ),
    // docs/events_todo.md, 256: their bundle holds one thing more: a pebble of cobblestone.
    row(
        "bundle_gains",
        3,
        5,
        &[
            InChest("bundle"),
            Stored("minecraft:cobblestone", 1),
            TargetAwayOrAsleep,
        ],
        2.0,
        2,
        Half,
        Action::BundleGains,
        false,
    ),
    // docs/events_todo.md, 257: their leather armour in the chest is dyed black overnight.
    row(
        "leather_dyed",
        4,
        5,
        &[
            InChestAny(&[
                "leather_helmet",
                "leather_chestplate",
                "leather_leggings",
                "leather_boots",
            ]),
            TargetAwayOrAsleep,
        ],
        3.0,
        1,
        Half,
        Action::LeatherDyed,
        false,
    ),
    // docs/events_todo.md, 258: each diamond they mine, a chime, one more each time.
    row(
        "ore_counted",
        4,
        5,
        &[MinedDiamond],
        0.0,
        64,
        Half,
        Action::OreCounted,
        true,
    ),
    // docs/events_todo.md, 259: a snowball they throw into the dark comes flying back.
    row(
        "snowball_comes_back",
        3,
        5,
        &[Night, TargetOutdoors, Holding("minecraft:snowball")],
        1.0,
        3,
        Half,
        Action::SnowballComesBack,
        false,
    ),
    // docs/events_todo.md, 260: an arrow they shoot into the dark lands at their feet a moment later.
    row(
        "arrow_returned",
        4,
        5,
        &[Night, TargetOutdoors, Holding("minecraft:bow")],
        1.0,
        3,
        Half,
        Action::ArrowReturned,
        false,
    ),
    // docs/events_todo.md, 261: crouched at an edge, a tug toward the drop.
    row(
        "edge_tug",
        4,
        5,
        &[CrouchedStill, TargetOutdoors],
        1.0,
        3,
        Half,
        Action::EdgeTug,
        false,
    ),
    // docs/events_todo.md, 262: ash drifts down round them out of a clear sky, grey and white, a minute, on their screen alone.
    row(
        "ash_on_the_wind",
        4,
        5,
        &[TargetOutdoors],
        1.0,
        3,
        Half,
        Action::Hint("ash_on_the_wind", 60.0),
        false,
    ),
    // docs/events_todo.md, 263: red petals fall round them, slowly, where there are no trees, forty-five seconds, on their screen alone.
    row(
        "red_petals",
        3,
        5,
        &[TargetOutdoors],
        1.0,
        3,
        Half,
        Action::Hint("red_petals", 45.0),
        false,
    ),
    // docs/events_todo.md, 264: indoors, sculk's charge creeps along the floor from their door to their bed, pulsing, thirty seconds, on their screen alone.
    row(
        "sculk_trail",
        5,
        5,
        &[Home, Bed, Door, TargetIndoors],
        1.0,
        3,
        Half,
        Action::Hint("sculk_trail", 30.0),
        false,
    ),
    // docs/events_todo.md, 265: for a minute and a half the thing in their hand shimmers as if enchanted.
    row(
        "false_glint",
        3,
        5,
        &[TargetAwake],
        1.0,
        3,
        Half,
        Action::Hint("false_glint", 90.0),
        false,
    ),
    // docs/events_todo.md, 266: in the Nether, every piglin near them stops and watches them.
    row(
        "piglins_stare",
        3,
        5,
        &[InNether, MobNear("piglin")],
        1.0,
        3,
        Half,
        Action::PiglinsStare,
        false,
    ),
    // docs/events_todo.md, 268: in the Nether, the strider they ride turns the way their house would be.
    row(
        "strider_turns_home",
        4,
        5,
        &[InNether, Riding("strider")],
        1.0,
        3,
        Half,
        Action::StriderTurnsHome,
        false,
    ),
    // docs/events_todo.md, 269: in the nether, every ghast near them stops firing and hangs where it is, facing them, a minute, as if watching.
    row(
        "ghast_watches",
        3,
        5,
        &[InNether, MobNear("ghast")],
        1.0,
        3,
        Half,
        Action::GhastWatches,
        false,
    ),
    // docs/events_todo.md, 277: in the nether, the hoglins near them stand off where they are, twelve seconds, facing them and charging none, as if something walked beside them that they feared.
    row(
        "hoglins_part",
        3,
        5,
        &[InNether, MobNear("hoglin")],
        1.0,
        3,
        Half,
        Action::HoglinsPart,
        false,
    ),
    // docs/events_todo.md, 282: in the nether, every magma cube near them stops bouncing at once and sits still, six seconds.
    row(
        "magma_cubes_settle",
        3,
        5,
        &[InNether, MobNear("magma_cube")],
        1.0,
        3,
        Half,
        Action::MagmaCubesSettle,
        false,
    ),
    // docs/events_todo.md, 278: in a fortress, every wither skeleton near them stands with its back to them, facing away, thirty seconds, and does not turn as they pass or strike them.
    row(
        "back_to_the_wall",
        4,
        5,
        &[InNether, MobNear("wither_skeleton")],
        1.0,
        3,
        Half,
        Action::BackToTheWall,
        false,
    ),
    // docs/events_todo.md, 292: in the end, every enderman near them turns its back on them at once and stands so a minute, facing away, so there is no face to meet.
    row(
        "endermen_turn_away",
        3,
        5,
        &[InEnd, MobNear("enderman")],
        1.0,
        3,
        Half,
        Action::EndermenTurnAway,
        false,
    ),
    // docs/events_todo.md, 287: in the nether, the zombified piglins near them come a step nearer each moment they look away, silent, to six blocks, and stand back where they were the moment they look round.
    row(
        "zombified_follow",
        4,
        5,
        &[InNether, MobNear("zombified_piglin")],
        1.0,
        3,
        Half,
        Action::ZombifiedFollow,
        false,
    ),
    // docs/events_todo.md, 271: in the nether, where it never rains, the sound of rain falls round them, as at home, half a minute, on their ears alone.
    row(
        "rain_in_the_nether",
        3,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::RainInTheNether,
        true,
    ),
    // docs/events_todo.md, 272: in the nether, four blocks behind them, the sound of a wooden door like the one at home opening, and a moment on shutting.
    row(
        "their_door_behind",
        3,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::TheirDoorBehind,
        true,
    ),
    // docs/events_todo.md, 281: in the nether, slow steps on the rock six blocks over their heads, crossing from behind them to right above, and stopping there.
    row(
        "nether_roof_steps",
        4,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::NetherRoofSteps,
        true,
    ),
    // docs/events_todo.md, 285: in the nether, where they came through, the sounds of home come through the portal, faint: a hen, a door shutting, a cow, rain.
    row(
        "home_through_the_portal",
        3,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::HomeThroughThePortal,
        true,
    ),
    // docs/events_todo.md, 294: in the end, far off, the dragon's growl from the central island, as if it were there still.
    row(
        "the_dragon_growls",
        4,
        5,
        &[InEnd],
        1.0,
        3,
        Half,
        Action::TheDragonGrowls,
        true,
    ),
    // docs/events_todo.md, 297: in the end, the beat of a second pair of wings three blocks behind them, keeping pace with them, six beats, and gone when they turn.
    row(
        "second_wings",
        4,
        5,
        &[InEnd],
        1.0,
        3,
        Half,
        Action::SecondWings,
        true,
    ),
    // docs/events_todo.md, 300: at an end city.
    row(
        "purpur_knock",
        3,
        5,
        &[InEnd, MobNear("shulker")],
        1.0,
        3,
        Half,
        Action::PurpurKnock,
        true,
    ),
    // docs/events_todo.md, 302: at an end city or its ship.
    row(
        "ship_creaks",
        3,
        5,
        &[InEnd, MobNear("shulker")],
        1.0,
        3,
        Half,
        Action::ShipCreaks,
        true,
    ),
    // docs/events_todo.md, 270: in the nether, walking the soul sand valley, the souls rise only where they have just stepped, a trail of them behind, a minute, on their screen alone.
    row(
        "souls_follow",
        3,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::Hint("souls_follow", 60.0),
        false,
    ),
    // docs/events_todo.md, 288: in the heat of the nether their breath shows before their face, in puffs, as on a winter night, twenty seconds.
    row(
        "breath_in_the_nether",
        3,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::Hint("cold_breath", 20.0),
        false,
    ),
    // docs/events_todo.md, 274: in a fortress, a torch ahead of them they did not place.
    row(
        "their_torch_ahead",
        4,
        5,
        &[InNether, MobNear("wither_skeleton")],
        2.0,
        2,
        Half,
        Action::TheirTorchAhead,
        false,
    ),
    // docs/events_todo.md, 275: their name cut into the netherrack behind them.
    row(
        "name_in_netherrack",
        4,
        5,
        &[InNether],
        3.0,
        1,
        Full,
        Action::NameInNetherrack,
        false,
    ),
    // docs/events_todo.md, 280: prints their size ahead of them, leading away.
    row(
        "prints_in_soul_sand",
        4,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::PrintsInSoulSand,
        false,
    ),
    // docs/events_todo.md, 299: prints across the end stone to the edge and off it.
    row(
        "prints_off_the_edge",
        4,
        5,
        &[InEnd],
        1.0,
        3,
        Half,
        Action::PrintsOffTheEdge,
        false,
    ),
    // docs/events_todo.md, 312: back through their portal, prints already lead from it toward the house.
    row(
        "prints_from_the_portal",
        4,
        5,
        &[Home, CameBack],
        2.0,
        2,
        Half,
        Action::PrintsFromThePortal,
        false,
    ),
    // docs/events_todo.md, 276: their portal on the Nether side weeps crying obsidian's tears.
    row(
        "portal_weeps",
        4,
        5,
        &[InNether],
        2.0,
        2,
        Half,
        Action::PortalWeeps,
        false,
    ),
    // docs/events_todo.md, 279: their compass steadies on where they came in.
    row(
        "compass_steadies",
        3,
        5,
        &[InNether, Holding("minecraft:compass")],
        1.0,
        3,
        Half,
        Action::Hint("compass_steadies", 60.0),
        false,
    ),
    // docs/events_todo.md, 289: in the Nether, the figure standing just inside the portal they came through, facing them.
    row(
        "standing_in_the_portal",
        4,
        5,
        &[InNether],
        2.0,
        2,
        Full,
        Action::StandingInThePortal,
        false,
    ),
    // docs/events_todo.md, 290: in a Nether fortress, the figure at the far end of the bridge ahead of them, still.
    row(
        "far_end_of_the_bridge",
        4,
        5,
        &[InNether, MobNear("wither_skeleton")],
        2.0,
        2,
        Full,
        Action::FarEndOfTheBridge,
        false,
    ),
    // docs/events_todo.md, 295: their name from the void below them in the subtitles.
    row(
        "name_from_the_void",
        4,
        5,
        &[InEnd],
        2.0,
        2,
        Half,
        Action::NameFromTheVoid,
        false,
    ),
    // docs/events_todo.md, 306: the figure stands on nothing over the void, facing them.
    row(
        "standing_on_the_void",
        5,
        5,
        &[InEnd],
        3.0,
        2,
        Full,
        Action::StandingOnTheVoid,
        false,
    ),
    // docs/events_todo.md, 320: a splash potion from the dark lands at their feet in the Nether.
    row(
        "splash_in_the_nether",
        3,
        5,
        &[InNether],
        1.5,
        3,
        Full,
        Action::SplashInTheNether,
        false,
    ),
    // docs/events_todo.md, 311: near the portal they built at home.
    row(
        "someone_went_through",
        3,
        5,
        &[TargetOutdoors],
        1.0,
        3,
        Half,
        Action::SomeoneWentThrough,
        true,
    ),
    // docs/events_todo.md, 313: their compass points to their bed instead.
    row(
        "lodestone_wrong",
        3,
        5,
        &[Home, Bed, Holding("minecraft:compass")],
        2.0,
        2,
        Half,
        Action::Hint("compass_points", 60.0),
        false,
    ),
    // docs/events_todo.md, 314: a ruined portal's obsidian near them weeps as if lit.
    row(
        "ruined_portal_glows",
        3,
        5,
        &[TargetOutdoors],
        2.0,
        2,
        Half,
        Action::RuinedPortalGlows,
        false,
    ),
    // docs/events_todo.md, 317: indoors at night, a splash potion shatters against the outside of one of their windows, the glass's crack with it.
    row(
        "potion_at_the_window",
        4,
        5,
        &[Night, TargetIndoors, Pane],
        1.0,
        3,
        Half,
        Action::PotionAtTheWindow,
        true,
    ),
    // docs/events_todo.md, 310: back from another dimension, the sky says another hour a moment.
    row(
        "came_back_wrong_hour",
        4,
        5,
        &[CameBack],
        2.0,
        2,
        Half,
        Action::Hint("wrong_sky", 6.0),
        false,
    ),
    // docs/events_todo.md, 316: lightning strikes beside them in the Nether, where there is no weather.
    row(
        "lightning_below",
        4,
        5,
        &[InNether],
        1.5,
        3,
        Full,
        Action::LightningBelow,
        false,
    ),
    // docs/events_todo.md, 291: every shulker near them opens at once and shuts at once.
    row(
        "shulkers_open",
        3,
        5,
        &[InEnd, MobNear("shulker")],
        1.0,
        3,
        Half,
        Action::ShulkersOpen,
        false,
    ),
    // docs/events_todo.md, 296: the end gateway near them flashes as if someone went through.
    row(
        "gateway_flashes",
        3,
        5,
        &[InEnd],
        1.0,
        3,
        Half,
        Action::GatewayFlashes,
        false,
    ),
    // docs/events_todo.md, 286: the blazes near them ring them round and fire nothing.
    row(
        "blaze_ring",
        4,
        5,
        &[InNether, MobNear("blaze")],
        1.5,
        2,
        Full,
        Action::BlazeRing,
        false,
    ),
    // docs/events_todo.md, 293: the endermen near them stand in a row at the edge, facing the void.
    row(
        "endermen_at_the_edge",
        4,
        5,
        &[InEnd, MobNear("enderman")],
        1.5,
        2,
        Full,
        Action::EndermenAtTheEdge,
        false,
    ),
    // docs/events_todo.md, 301: the End goes quiet, the music and the endermen, a minute.
    row(
        "end_goes_quiet",
        4,
        5,
        &[InEnd],
        2.0,
        2,
        Half,
        Action::EndGoesQuiet,
        false,
    ),
    // docs/events_todo.md, 303: every chorus plant near them flowers at once.
    row(
        "chorus_blooms",
        3,
        5,
        &[InEnd],
        1.5,
        2,
        Half,
        Action::ChorusBlooms,
        false,
    ),
    // docs/events_todo.md, 308: the dragon egg on its plinth at home is on their pillow in the morning.
    row(
        "egg_on_the_pillow",
        4,
        5,
        &[Fixture("dragon_egg"), Bed, TargetAwayOrAsleep],
        3.0,
        1,
        Full,
        Action::EggOnThePillow,
        false,
    ),
    // docs/events_todo.md, 309: a chorus fruit eaten near home sets them down on their doorstep.
    row(
        "chorus_lands_home",
        4,
        5,
        &[Home, Door, AteChorus],
        2.0,
        2,
        Full,
        Action::ChorusLandsHome,
        false,
    ),
    // docs/events_todo.md, 298: a pearl they throw lands them turned round.
    row(
        "pearl_turned",
        3,
        5,
        &[Holding("minecraft:ender_pearl")],
        1.0,
        3,
        Half,
        Action::PearlTurned,
        false,
    ),
    // docs/events_todo.md, 318: one of their own potions is thrown at their feet from the dark.
    row(
        "their_own_potion",
        4,
        5,
        &[Night, TargetOutdoors, InChest("minecraft:potion")],
        2.0,
        2,
        Full,
        Action::TheirOwnPotion,
        false,
    ),
    // docs/events_todo.md, 283: the vines near them grow a block longer each time they look back.
    row(
        "vines_reach",
        3,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::VinesReach,
        false,
    ),
    // docs/events_todo.md, 284: their portal goes out as they reach it, lit again ten seconds on.
    row(
        "portal_goes_out",
        5,
        5,
        &[InNether],
        3.0,
        1,
        Full,
        Action::PortalGoesOut,
        false,
    ),
    // docs/events_todo.md, 273: in the nether, the lava sea round them goes still: no pops, no rising sparks, no sound, twenty seconds, then all at once again, on their screen alone.
    row(
        "lava_goes_still",
        4,
        5,
        &[InNether],
        1.0,
        3,
        Half,
        Action::Hint("lava_still", 20.0),
        false,
    ),
    // docs/events_todo.md, 304: in the end, as they look up, for a moment the sky makes a face, two eyes and a mouth of its own dark purple, faint, swelling and gone, on their screen alone.
    row(
        "end_sky_face",
        4,
        5,
        &[InEnd],
        1.0,
        3,
        Half,
        Action::Hint("end_sky_face", 8.0),
        false,
    ),
    // docs/events_todo.md, 267: a piglin near them drops one thing of theirs from their chest at home.
    row(
        "barter_from_home",
        4,
        5,
        &[InNether, MobNear("piglin")],
        3.0,
        2,
        Full,
        Action::BarterFromHome,
        false,
    ),
    // docs/events_todo.md, 305: the end ship's elytra is gone, and hangs in their frame at home.
    row(
        "elytra_at_home",
        5,
        5,
        &[InEnd, MobNear("shulker")],
        10.0,
        1,
        Full,
        Action::ElytraAtHome,
        false,
    ),
    // docs/events_todo.md, 307: as the dragon dies, the figure stands on the exit portal, facing them.
    row(
        "on_the_exit_portal",
        5,
        5,
        &[InEnd, KilledNow],
        30.0,
        1,
        Full,
        Action::OnTheExitPortal,
        false,
    ),
    // docs/events_todo.md, 315: armed in the end, the next time the end poem is read on their screen it ends with three lines of its own in its two voices: another in the dream, one the player did not dream.
    row(
        "the_poem_knows",
        5,
        5,
        &[InEnd],
        30.0,
        1,
        Half,
        Action::Hint("the_poem_knows", 1.0),
        false,
    ),
    // docs/events_todo.md, 321: leaf litter on their doorstep, as if blown in, no tree by the door.
    row(
        "leaves_at_the_door",
        1,
        5,
        &[Door, TargetAwayOrAsleep],
        2.0,
        4,
        Half,
        Action::LeavesAtTheDoor,
        true,
    ),
    // docs/events_todo.md, 327: coming home, smoke rises from the top of their house as from a chimney; nothing burns.
    row(
        "smoke_from_the_roof",
        1,
        5,
        &[Home, TargetApproachingDoor, NotRaining],
        2.0,
        4,
        Half,
        Action::SmokeFromTheRoof,
        true,
    ),
    // docs/events_todo.md, 322: walking outdoors, for a few steps their footsteps sound like the boards of their floor.
    row(
        "wrong_footing",
        1,
        5,
        &[TargetWalking, TargetOutdoors],
        2.0,
        4,
        Half,
        Action::WrongFooting,
        true,
    ),
    // docs/events_todo.md, 323: a pressure plate of theirs clicks down with no one on it; nothing it powers moves.
    row(
        "plate_clicks",
        1,
        5,
        &[Fixture("pressure_plate"), TargetIndoors],
        2.0,
        4,
        Half,
        Action::SoundAtFixture(
            "pressure_plate",
            "minecraft:block.wooden_pressure_plate.click_on",
        ),
        true,
    ),
    // docs/events_todo.md, 324: the books on their chiseled bookshelf stand each one slot along from where they put them.
    row(
        "book_moved_on_shelf",
        1,
        5,
        &[Fixture("chiseled_bookshelf"), TargetAwayOrAsleep],
        2.0,
        3,
        Half,
        Action::BookMovedOnShelf,
        true,
    ),
    // docs/events_todo.md, 325: a head or skull they set out faces their bed when they come back.
    row(
        "skull_turned",
        1,
        5,
        &[Fixture("head"), Bed, TargetAwayOrAsleep],
        2.0,
        3,
        Half,
        Action::SkullTurned,
        true,
    ),
    // docs/events_todo.md, 326: coming home, the pet they left sitting is standing where it sat, looking at the door.
    row(
        "pet_got_up",
        1,
        5,
        &[Pet, Door, TargetApproachingDoor],
        2.0,
        3,
        Half,
        Action::PetGotUp,
        true,
    ),
    // docs/events_todo.md, 328: at dusk, coming home, one window of their house glows a moment as if someone inside carried a torch past.
    row(
        "light_in_your_window",
        2,
        5,
        &[Dusk, Pane, TargetApproachingDoor],
        2.0,
        3,
        Half,
        Action::LightInYourWindow,
        true,
    ),
    // docs/events_todo.md, 329: their cold furnace's front glows lit a moment, and goes dark; nothing is in it.
    row(
        "furnace_glows",
        1,
        5,
        &[ColdFurnace, TargetIndoors],
        2.0,
        3,
        Half,
        Action::FurnaceGlows,
        true,
    ),
    // docs/events_todo.md, 330: a sheet of paper on the floor inside their door, as if slipped under it; reach for it and it is not there.
    row(
        "paper_on_the_floor",
        2,
        5,
        &[Door, TargetApproachingDoor],
        2.0,
        3,
        Half,
        Action::PaperOnTheFloor,
        true,
    ),
    // docs/events_todo.md, 331: in daylight, far off ahead of them, a glint flashes three times, as of a spyglass looking back; then nothing there.
    row(
        "a_glint_on_the_hill",
        2,
        5,
        &[Day, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::AGlintOnTheHill,
        false,
    ),
    // docs/events_todo.md, 343: deep down, a slow heartbeat from inside the rock ahead of them.
    row(
        "heartbeat_in_the_stone",
        3,
        5,
        &[TargetUnderground],
        2.0,
        3,
        Half,
        Action::HeartbeatInTheStone,
        false,
    ),
    // docs/events_todo.md, 345: in a storm, a flash ahead of them, and the thunder comes from right behind them.
    row(
        "thunder_behind",
        3,
        5,
        &[Thundering, TargetOutdoors],
        1.5,
        3,
        Half,
        Action::ThunderBehind,
        false,
    ),
    // docs/events_todo.md, 352: indoors, the creak of someone sitting down on their bed; no one is on it.
    row(
        "bed_creaks",
        1,
        5,
        &[Bed, TargetIndoors],
        2.0,
        4,
        Half,
        Action::BedCreaks,
        true,
    ),
    // docs/events_todo.md, 353: their brewing stand bubbles as if brewing; it is empty and cold.
    row(
        "stand_bubbles",
        1,
        5,
        &[Fixture("brewing_stand"), TargetIndoors],
        2.0,
        4,
        Half,
        Action::SoundAtFixture("brewing_stand", "minecraft:block.brewing_stand.brew"),
        true,
    ),
    // docs/events_todo.md, 368: down at the bedrock, a knock from under it, where nothing can be; then two.
    row(
        "knock_from_below",
        4,
        5,
        &[TargetUnderground, Below(-50)],
        2.0,
        2,
        Full,
        Action::KnockFromBelow,
        false,
    ),
    // docs/events_todo.md, 378: in a storm, the thunder comes in claps that count down: five, four, three, two, one; then nothing.
    row(
        "thunder_counts",
        4,
        5,
        &[Thundering, TargetAwake],
        2.0,
        2,
        Half,
        Action::ThunderCounts,
        false,
    ),
    // docs/events_todo.md, 388: among mountains, someone calls their name far off, and it echoes back from the peaks.
    row(
        "name_echoes",
        2,
        5,
        &[
            InBiome(&["peaks", "slopes", "windswept", "grove"]),
            TargetOutdoors,
        ],
        2.0,
        2,
        Half,
        Action::NameEchoes,
        false,
    ),
    // docs/events_todo.md, 333: in the afternoon, a small whirl of dust crosses their field toward them and falls apart at their feet.
    row(
        "dust_devil",
        2,
        5,
        &[Day, Field, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::DustDevil,
        false,
    ),
    // docs/events_todo.md, 347: their lightning rod sparks and crackles under a clear sky, as if it had been struck.
    row(
        "lightning_rod_sparks",
        2,
        5,
        &[Fixture("lightning_rod"), NotRaining],
        2.0,
        3,
        Half,
        Action::LightningRodSparks,
        false,
    ),
    // docs/events_todo.md, 349: in a boat over deep ocean at night, a pale light far below keeps pace with the boat.
    row(
        "light_below_the_boat",
        3,
        5,
        &[Night, TargetInBoat, InBiome(&["ocean"])],
        2.0,
        2,
        Half,
        Action::LightBelowTheBoat,
        false,
    ),
    // docs/events_todo.md, 350: at night in a swamp, small lights hang over the water and drift away as they come.
    row(
        "lights_over_the_swamp",
        2,
        5,
        &[Night, InBiome(&["swamp"]), TargetOutdoors],
        2.0,
        3,
        Half,
        Action::LightsOverTheSwamp,
        false,
    ),
    // docs/events_todo.md, 355: their campfire's smoke rises in puffs: three short, three long, three short.
    row(
        "smoke_signals",
        2,
        5,
        &[Fixture("campfire"), TargetOutdoors],
        2.0,
        2,
        Half,
        Action::SmokeSignals,
        false,
    ),
    // docs/events_todo.md, 357: the water in their cauldron ripples and splashes once, as if something dipped into it.
    row(
        "cauldron_ripples",
        1,
        5,
        &[Fixture("cauldron"), TargetIndoors],
        2.0,
        4,
        Half,
        Action::CauldronRipples,
        true,
    ),
    // docs/events_todo.md, 358: indoors, a few leaves drift down from the ceiling and are gone on the floor.
    row(
        "leaves_fall_inside",
        1,
        5,
        &[Home, TargetIndoors],
        2.0,
        4,
        Half,
        Action::LeavesFallInside,
        true,
    ),
    // docs/events_todo.md, 366: by day, far ahead along their way, a trail of dust as of someone walking it toward them; no one there.
    row(
        "dust_on_the_road",
        2,
        5,
        &[Day, Route, TargetOutdoors],
        2.0,
        3,
        Half,
        Action::DustOnTheRoad,
        false,
    ),
    // docs/events_todo.md, 367: the grass bends apart in a line coming toward them, as if someone walks through it; it stops a few blocks off.
    row(
        "the_grass_parts",
        2,
        5,
        &[TargetOutdoors, Around("trees")],
        2.0,
        3,
        Half,
        Action::TheGrassParts,
        false,
    ),
    // docs/events_todo.md, 376: under a clear sky, it rains on them alone: a patch of rain round them that follows them a minute.
    row(
        "a_cloud_of_its_own",
        3,
        5,
        &[NotRaining, TargetOutdoors],
        2.0,
        2,
        Half,
        Action::ACloudOfItsOwn,
        false,
    ),
    // docs/events_todo.md, 379: in a warm place, a few snowflakes fall on them out of a clear sky.
    row(
        "snow_in_the_heat",
        2,
        5,
        &[
            InBiome(&["desert", "savanna", "badlands", "jungle"]),
            NotRaining,
            TargetOutdoors,
        ],
        2.0,
        2,
        Half,
        Action::SnowInTheHeat,
        false,
    ),
    // docs/events_todo.md, 346: the storm ends, the sky clears, and for them it goes on raining a minute.
    row(
        "rain_stays",
        3,
        5,
        &[RainJustEnded],
        2.0,
        2,
        Half,
        Action::RainStays,
        false,
    ),
    // docs/events_todo.md, 390: on a river at night, a lit lantern floats past them on the water.
    row(
        "lantern_down_the_river",
        3,
        5,
        &[Night, InBiome(&["river"]), TargetOutdoors],
        2.0,
        2,
        Half,
        Action::LanternDownTheRiver,
        false,
    ),
    // docs/events_todo.md, 375: in a storm, lightning strikes round them again and again in a closing ring, the last a step away (owner's idea).
    row(
        "lightning_round_you",
        4,
        5,
        &[Thundering, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::LightningRoundYou,
        false,
    ),
    // docs/events_todo.md, 381: in a storm, lightning strikes in a line, bolt after bolt, leading away from them toward its place.
    row(
        "bolts_in_a_line",
        4,
        5,
        &[Thundering, TargetOutdoors, Home],
        2.0,
        2,
        Full,
        Action::BoltsInALine,
        false,
    ),
    // docs/events_todo.md, 380: in a storm, their door bangs open and shut with each thunderclap.
    row(
        "door_bangs_in_the_storm",
        3,
        5,
        &[Thundering, Door, TargetIndoors],
        2.0,
        2,
        Full,
        Action::DoorBangsInTheStorm,
        false,
    ),
    // docs/events_todo.md, 344: in a storm, each flash shows a figure in the field, nearer each time; between flashes there is no one.
    row(
        "figure_in_the_flash",
        4,
        5,
        &[Thundering, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::FigureInTheFlash,
        false,
    ),
    // docs/events_todo.md, 335: by day, far across their field, the figure is bent over the crops as if tending them; gone when they come near.
    row(
        "figure_in_your_field",
        3,
        5,
        &[Day, Field, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::FigureInYourField,
        false,
    ),
    // docs/events_todo.md, 359: looking toward the sun, a dark figure on the ridge under it, against the glare; gone when they look again.
    row(
        "against_the_sun",
        3,
        5,
        &[Day, TargetOutdoors],
        2.0,
        2,
        Full,
        Action::AgainstTheSun,
        false,
    ),
    // docs/events_todo.md, 398: gliding on elytra, a figure glides beside them, wingless, keeping pace.
    row(
        "flies_beside_you",
        4,
        5,
        &[Gliding],
        2.0,
        2,
        Full,
        Action::FliesBesideYou,
        false,
    ),
    // docs/events_todo.md, 389: in an ancient city, the candles round them light one by one, coming toward them.
    row(
        "the_city_candles",
        4,
        5,
        &[InBiome(&["deep_dark"]), TargetUnderground],
        2.0,
        2,
        Half,
        Action::TheCityCandles,
        false,
    ),
    // docs/events_todo.md, 393: in a lush cave, the glow berries round them go dark one by one, the way fireflies go out.
    row(
        "glow_berries_go_dark",
        3,
        5,
        &[InBiome(&["lush_caves"]), TargetUnderground],
        2.0,
        2,
        Half,
        Action::GlowBerriesGoDark,
        false,
    ),
    // docs/events_todo.md, 164: a second of static over the screen, a hiss with it.
    row(
        "static",
        4,
        5,
        &[TargetAwake],
        1.0,
        4,
        Half,
        Action::Hint("static", 1.0),
        true,
    ),
    // docs/events_todo.md, 165: the elder guardian's face and cry, far from any ocean, no curse.
    row(
        "elder_face",
        5,
        5,
        &[TargetAwake],
        2.0,
        2,
        Full,
        Action::Hint("elder_face", 2.0),
        false,
    ),
    // docs/events_todo.md, 127: a deep rumble under them, the view shaking, dust from the
    // ceiling indoors; nothing breaks.
    row(
        "rumble",
        4,
        5,
        &[TargetAwake],
        2.0,
        3,
        Half,
        Action::Hint("rumble", 3.0),
        false,
    ),
    // docs/events_todo.md, 133: at night a far explosion from the way of its place, a flash on
    // the sky there; nothing harmed.
    row(
        "boom_far",
        4,
        5,
        &[TargetOutdoors, Night, Home],
        2.0,
        3,
        Half,
        Action::Hint("boom_far", 4.0),
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
    // docs/events_todo.md, 75: ahead in the dark, gone as they come near.
    row(
        "eyes_in_the_dark",
        4,
        5,
        &[TargetUnderground],
        3.0,
        3,
        Full,
        Action::Hint("eyes_in_the_dark", 60.0),
        false,
    ),
    // docs/events_todo.md, 55: a second and a half, no harm.
    row(
        "hearts_drop",
        5,
        5,
        &[TargetOnline],
        3.0,
        3,
        Half,
        Action::Hint("hearts_drop", 1.5),
        true,
    ),
    // docs/events_todo.md, 54: four seconds, nothing lost.
    row(
        "xp_slips",
        4,
        5,
        &[TargetOnline],
        3.0,
        3,
        Half,
        Action::Hint("xp_slips", 4.0),
        true,
    ),
    // docs/events_todo.md, 40: half an hour, for whenever they open it.
    row(
        "f3_line",
        5,
        5,
        &[TargetOnline],
        6.0,
        2,
        Half,
        Action::Hint("f3_line", 1800.0),
        true,
    ),
    // docs/events_todo.md, 39: ten minutes, for whenever they look at themselves.
    row(
        "only_in_third_person",
        4,
        5,
        &[TargetOnline],
        5.0,
        2,
        Half,
        Action::Hint("only_in_third_person", 600.0),
        true,
    ),
    // docs/events_todo.md, 38: half an hour for them to take one.
    row(
        "in_the_screenshot",
        5,
        5,
        &[TargetOnline],
        6.0,
        2,
        Half,
        Action::Hint("in_the_screenshot", 1800.0),
        true,
    ),
    // docs/events_todo.md, 37: as they come home, on a window of the house.
    row(
        "handprints",
        4,
        5,
        &[TargetApproachingDoor, Pane, Home],
        4.0,
        2,
        Full,
        // Half a minute: a minute and a half, they were still on the glass long after they had
        // been seen (owner: "the handprints dont seem to stop").
        Action::Hint("handprints", 30.0),
        false,
    ),
    // docs/events_todo.md, 22: a second of the death screen, nothing died.
    row(
        "false_death",
        5,
        5,
        &[TargetWalking, TargetOutdoors],
        6.0,
        1,
        Full,
        Action::Hint("false_death", 1.2),
        false,
    ),
    // docs/events_todo.md, 21: ten minutes, a few openings of their inventory.
    row(
        "inventory_stare",
        4,
        5,
        &[TargetOnline],
        4.0,
        2,
        Half,
        Action::Hint("inventory_stare", 600.0),
        true,
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

/// The catalogue's page, built into the mind so the debug screen can say what each event is
/// without a second copy of the words: `docs/entity.md`'s rows, `| \`id\` | stages | what
/// happens | needs |`.
const CATALOGUE_PAGE: &str = include_str!("../../docs/entity.md");

/// What the catalogue says of an event: what happens, and what it needs. None for an event
/// with no row (the docs test keeps every coded event in the page).
pub fn described(id: &str) -> Option<(&'static str, &'static str)> {
    static ROWS: std::sync::OnceLock<
        std::collections::HashMap<&'static str, (&'static str, &'static str)>,
    > = std::sync::OnceLock::new();
    ROWS.get_or_init(|| {
        CATALOGUE_PAGE
            .lines()
            .filter_map(|line| {
                let cells: Vec<&str> = line.split(" | ").collect();
                let id = cells.first()?.strip_prefix("| `")?.strip_suffix('`')?;
                let what = cells.get(2)?.trim();
                let needs = cells
                    .get(3)?
                    .trim()
                    .trim_end_matches(" |")
                    .trim_end_matches('|')
                    .trim();
                Some((id, (what, needs)))
            })
            .collect()
    })
    .get(id)
    .copied()
}

impl Event {
    pub fn offered_in(&self, stage: u8) -> bool {
        self.from_stage <= stage && stage <= self.to_stage
    }

    /// What the event counts against the day's budget, by its weight alone (a sound half).
    pub fn cost(&self) -> f32 {
        match self.weight {
            Weight::Full => 1.0,
            Weight::Half => 0.5,
        }
    }

    /// What it counts against the day's budget as the config has it: a small event (one they
    /// may not notice) at most `events.small_cost`, so more of them fit beside the things they
    /// will notice.
    pub fn cost_in(&self, config: &crate::config::Config) -> f32 {
        if self.small {
            self.cost().min(config.events.small_cost)
        } else {
            self.cost()
        }
    }

    /// What the event does to the target, for how they took it (the review's N2): `sounds`,
    /// `words` (signs, chat, names, books), `things` (their things moved, taken, given),
    /// `house` (blocks of the house and around it changed), `seen` (figures and creatures that
    /// show themselves), `sky` (the screen and the sky), `night` (their sleep); empty for
    /// nothing and the scans. Every action is named here, so a new one must be sorted.
    pub fn effect(&self) -> &'static str {
        match self.action {
            Action::Wait | Action::ScanVillage | Action::ScanOre => "",
            Action::Sound(_)
            | Action::SoundAtTarget(_)
            | Action::SoundFar(_)
            | Action::SoundAtFixture(_, _)
            | Action::Taps(_, _, _)
            | Action::MiningSounds
            | Action::FishingLine
            | Action::WetSteps
            | Action::SoundPair(..)
            | Action::ChestAnswers
            | Action::KnocksOnWood
            | Action::SoundWarp
            | Action::BranchSnaps
            | Action::OreCounted
            | Action::RainInTheNether
            | Action::TheirDoorBehind
            | Action::NetherRoofSteps
            | Action::HomeThroughThePortal
            | Action::TheDragonGrowls
            | Action::SecondWings
            | Action::PurpurKnock
            | Action::ShipCreaks
            | Action::SomeoneWentThrough
            | Action::PotionAtTheWindow
            | Action::WrongFooting
            | Action::HeartbeatInTheStone
            | Action::ThunderBehind
            | Action::BedCreaks
            | Action::KnockFromBelow
            | Action::ThunderCounts
            | Action::Footsteps => "sounds",
            Action::FramesShow | Action::BrewLeft | Action::PotionChanged => "things",
            Action::Sign
            | Action::SignDeep
            | Action::DeathMessage
            | Action::WordsOnSign
            | Action::SignWord
            | Action::CountUp
            | Action::KillSign
            | Action::Message(..)
            | Action::Says
            | Action::SaysKills
            | Action::Whisper
            | Action::RepeatsYou
            | Action::BookWord
            | Action::RenameOne
            | Action::NameAnimal
            | Action::TabEntry
            | Action::Answers
            | Action::DiarySold
            | Action::DiaryBrought
            | Action::LetterInChest
            | Action::VillagerNamed
            | Action::LecternNotes
            | Action::PageInBook => "words",
            Action::OpenLid
            | Action::ShuffleContainer
            | Action::SplitStack
            | Action::TidyContainer
            | Action::SwapStacks
            | Action::FillGap
            | Action::CookOne
            | Action::TakeOutput
            | Action::ItemOnTable
            | Action::EggOnBed
            | Action::Take
            | Action::ReturnTaken
            | Action::TakenUp
            | Action::SkyGivesBack
            | Action::PlaceTaken
            | Action::ReturnUsed
            | Action::HopperEmptied
            | Action::ShulkerMoved
            | Action::FullOfOne
            | Action::MapMarked
            | Action::LootGift
            | Action::VillagerWord
            | Action::JukeboxDisc
            | Action::LecternPage
            | Action::SwapFuel
            | Action::Brewing
            | Action::GrindstoneStrip
            | Action::CraftedForYou
            | Action::ToolUsed
            | Action::TidyByColour
            | Action::RotateFrame
            | Action::FramesTurn
            | Action::SwapPainting
            | Action::StandTurn
            | Action::StandDress
            | Action::FrameEmptied
            | Action::FrameSwap
            | Action::MoveVehicle
            | Action::SignWithCoords
            | Action::ToolsLaidOut
            | Action::BucketFishChanged
            | Action::ShelfShuffled
            | Action::CatGift
            | Action::BundleGains
            | Action::LeatherDyed
            | Action::EggOnThePillow
            | Action::BarterFromHome
            | Action::ElytraAtHome
            | Action::BookMovedOnShelf
            | Action::PetGotUp
            | Action::FurnaceGlows
            | Action::PushVehicle(..) => "things",
            Action::ToggleDoor
            | Action::ToggleFixture(_)
            | Action::ToggleCandles
            | Action::MoveLight
            | Action::AddTorch
            | Action::LightPath
            | Action::Hole
            | Action::Cairn
            | Action::Sapling
            | Action::Grave
            | Action::YourGrave
            | Action::Cobwebs
            | Action::FalseDoor
            | Action::TorchesTurn
            | Action::SnowShape
            | Action::ColdHouse
            | Action::FrostOnGlass
            | Action::StairsToNowhere
            | Action::PaleMossCreeps
            | Action::WindowTinted
            | Action::SculkCreeps
            | Action::DripleafSteps
            | Action::AxolotlsPlayDead
            | Action::KeepIn(_)
            | Action::FaceInTheCanopy
            | Action::BootsAtTheDoor
            | Action::PathWorn
            | Action::CarpetShift
            | Action::Cauldron
            | Action::DoorHinge
            | Action::LadderGap
            | Action::FlowersPotted
            | Action::PumpkinFace
            | Action::TreeGrown
            | Action::Backfill
            | Action::Alcove
            | Action::ShiftFurniture
            | Action::RoomDark
            | Action::DoorHeld
            | Action::BedTurned
            | Action::BucketFloor
            | Action::SnowCleared
            | Action::Trample
            | Action::HarvestOne
            | Action::PlateLaid
            | Action::DoorAhead
            | Action::VillageDoors
            | Action::OreMarked
            | Action::LanternAhead
            | Action::CaveLit
            | Action::DeathLight
            | Action::LightTaken
            | Action::LightsOut
            | Action::SoulSwap
            | Action::ShiftBed
            | Action::SheetsChanged
            | Action::SecondBed
            | Action::Scarecrow
            | Action::FieldWord
            | Action::RowWithered
            | Action::EggsToDoor
            | Action::WeaponOnWall
            | Action::HeadOnPost
            | Action::GraveEach
            | Action::LampsBlink
            | Action::ComposterFills
            | Action::BellRings
            | Action::Trail
            | Action::TrailIn
            | Action::VillageLantern
            | Action::BellTwice
            | Action::MineBeside
            | Action::TunnelSealed
            | Action::TunnelToward
            | Action::PortalLit
            | Action::MinedOut
            | Action::LadderDown
            | Action::TunnelGoesOn
            | Action::TorchesFarEnd
            | Action::WayBackCloses
            | Action::Build(_)
            | Action::CaveRoom
            | Action::Snowman
            | Action::BoatInHouse
            | Action::RotateFixture(..)
            | Action::ColdFurnace
            | Action::OpenOnEnter
            | Action::PlantFlower
            | Action::ReplantOther
            | Action::IcePath
            | Action::ItsLightsOn
            | Action::FaceInTheChest
            | Action::RespawnItsBed
            | Action::EnderChestNote
            | Action::FireworkOverItsHouse
            | Action::PrintsToTheCave
            | Action::EveryDoorOpens
            | Action::FlowersWilted
            | Action::CampNearby
            | Action::StoneCircle
            | Action::NameInTheSnow
            | Action::TorchRing
            | Action::Breadcrumbs
            | Action::ItsBanner
            | Action::BlazedTrees
            | Action::TreesCloseBehind
            | Action::DeadRedstone
            | Action::BareTrees
            | Action::OutOfPlace
            | Action::CaveTorchesFlicker
            | Action::PrintsAroundTheHouse
            | Action::TreeComesDown
            | Action::CirclesATree
            | Action::PathThroughTheTrees
            | Action::ArrowInTheDoor
            | Action::WallSwap(..) => "house",
            Action::TabTwice => "words",
            Action::Stare
            | Action::StandWatch
            | Action::PetGrowl
            | Action::MoveCreature(..)
            | Action::GolemTurned
            | Action::DyeSheep
            | Action::DoorWontOpen
            | Action::PulledUnder
            | Action::EyeTurnsHome
            | Action::WearsYourFace
            | Action::AnimalFollows
            | Action::HeldBreath
            | Action::AllFreeze
            | Action::FigureOnMap
            | Action::ChickenLays
            | Action::AtTheirTable
            | Action::JumpingFar
            | Action::WavesFromInside
            | Action::RidesTheirHorse
            | Action::EatsAtTheirChest
            | Action::MirrorInTheWindow
            | Action::SlipsFromHand
            | Action::PetStaresAtWall
            | Action::BatsInTheHouse
            | Action::LostHour
            | Action::ChestWontOpen
            | Action::FishingFigure
            | Action::ByTheBed
            | Action::SwimsBeneath
            | Action::StandsInTheRain
            | Action::ChestRattles
            | Action::TrapdoorTaps
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
            | Action::CeilingCrawl
            | Action::InTheLava
            | Action::OverTheEdge
            | Action::RiderOnTheRidge
            | Action::HappyGhastOverhead
            | Action::TheTreeLine
            | Action::AnimalsFleeTheWoods
            | Action::VillagersFleeYou
            | Action::TradesRefused
            | Action::VillagerStabs(_)
            | Action::VillagerSpins
            | Action::VillagersWalkOut
            | Action::VillagerCollapses
            | Action::WallBursts
            | Action::DaylightPhantom
            | Action::DropsFromTheCeiling
            | Action::WolvesCircle
            | Action::ZombieInYourGear(_)
            | Action::Vexes(..)
            | Action::KillerRabbit(_)
            | Action::CopperGolemAtDoor
            | Action::CreakingInTheField
            | Action::CreeperBehind
            | Action::VillagerFleesIn
            | Action::SnowballThrown
            | Action::RaisesTheirSword
            | Action::HostilesStare
            | Action::Figure(..)
            | Action::FigureSwing(_)
            | Action::FigureWave
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
            | Action::DigInto(_)
            | Action::IntoTheDark
            | Action::WarpAnimal
            | Action::BeesOut
            | Action::HorseMoved
            | Action::FaceTheHouse
            | Action::OneMore
            | Action::BoatWaiting
            | Action::UnderWater
            | Action::VillageAbandoned
            | Action::Pack
            | Action::MobComes(_)
            | Action::HostilesClose
            | Action::Shove(_)
            | Action::BobberUnder
            | Action::Shoot(_)
            | Action::SplashDark
            | Action::DoorCloud
            | Action::AnchorCharged
            | Action::DripstoneFalls
            | Action::WitherRoseInPot
            | Action::ShulkerLidPeeks
            | Action::TridentPlanted
            | Action::SlamCrater
            | Action::OneStepMore
            | Action::RoomClosesIn
            | Action::DripstoneInTunnel
            | Action::SkullInTheTunnel
            | Action::PaleHorse
            | Action::FangsInALine
            | Action::BeamFromTheWater
            | Action::HeadOnThePillow
            | Action::EmptyLead
            | Action::DriedGhastOnDoorstep
            | Action::EyeblossomsOpen
            | Action::GolemStatueTurns
            | Action::FirefliesGoOut
            | Action::TrialKeyLeft
            | Action::FigureDigging
            | Action::FigureBows
            | Action::ClimbingTheHouse
            | Action::FigureAtTheLectern
            | Action::FigureInYourArmour
            | Action::OnThePeak
            | Action::CattleTurnAway
            | Action::HorseBalks
            | Action::ArmadillosCurl
            | Action::SnifferAtItsPlace
            | Action::RefusesFood
            | Action::TheChildStares
            | Action::LateDrop
            | Action::SnowballComesBack
            | Action::ArrowReturned
            | Action::EdgeTug
            | Action::PiglinsStare
            | Action::StriderTurnsHome
            | Action::GhastWatches
            | Action::HoglinsPart
            | Action::MagmaCubesSettle
            | Action::BackToTheWall
            | Action::EndermenTurnAway
            | Action::ZombifiedFollow
            | Action::TheirTorchAhead
            | Action::NameInNetherrack
            | Action::PrintsInSoulSand
            | Action::PrintsOffTheEdge
            | Action::PrintsFromThePortal
            | Action::PortalWeeps
            | Action::StandingInThePortal
            | Action::FarEndOfTheBridge
            | Action::StandingOnTheVoid
            | Action::SplashInTheNether
            | Action::RuinedPortalGlows
            | Action::LightningBelow
            | Action::ShulkersOpen
            | Action::GatewayFlashes
            | Action::BlazeRing
            | Action::EndermenAtTheEdge
            | Action::ChorusBlooms
            | Action::ChorusLandsHome
            | Action::PearlTurned
            | Action::TheirOwnPotion
            | Action::VinesReach
            | Action::PortalGoesOut
            | Action::OnTheExitPortal
            | Action::LightningRoundYou
            | Action::BoltsInALine
            | Action::FigureInTheFlash
            | Action::FigureInYourField
            | Action::AgainstTheSun
            | Action::FliesBesideYou
            | Action::Charge(..) => "seen",
            Action::Hint(_, _)
            | Action::GhastName
            | Action::BossBarIt
            | Action::TitleYourName
            | Action::NameFromTheVoid
            | Action::EndGoesQuiet
            | Action::SmokeFromTheRoof
            | Action::LightInYourWindow
            | Action::PaperOnTheFloor
            | Action::AGlintOnTheHill
            | Action::NameEchoes
            | Action::DustDevil
            | Action::LightningRodSparks
            | Action::LightBelowTheBoat
            | Action::LightsOverTheSwamp
            | Action::SmokeSignals
            | Action::CauldronRipples
            | Action::LeavesFallInside
            | Action::DustOnTheRoad
            | Action::TheGrassParts
            | Action::ACloudOfItsOwn
            | Action::SnowInTheHeat
            | Action::RainStays
            | Action::LanternDownTheRiver
            | Action::TheCityCandles
            | Action::GlowBerriesGoDark
            | Action::CracksInTheWall => "sky",
            Action::EnderHolds => "seen",
            Action::WhileAway
            | Action::DoorBangsInTheStorm
            | Action::SkullTurned
            | Action::LeavesAtTheDoor
            | Action::CopperAges
            | Action::BedFenced
            | Action::TrapdoorToNothing
            | Action::MushroomsInCorners => "house",
            Action::HandOnShoulder => "seen",
            // Their sleep: refused, broken, someone in their bed.
            Action::RefuseRest
            | Action::Wake
            | Action::WokeElsewhere
            | Action::OtherHouse
            | Action::BedBreaks
            | Action::Choke(_)
            | Action::ShotInBed(_)
            | Action::BurnedInBed
            | Action::Sleepwalk
            | Action::LiesBeside
            | Action::BedOccupied => "night",
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
            "raises_their_sword" => "have the far figure raise their own sword",
            "arrow_in_the_door" => "shoot an arrow into their front door",
            "snowball_thrown" => "throw a snowball at their back",
            "crouched_behind_a_tree" => "crouch the figure near them, hiding",
            "peeks_from_the_trees" => {
                "have the figure peek out at them from behind the trees ahead"
            }
            "while_unseen" => "set the figure behind them, coming on only while they do not look",
            "drawn_to_the_light" => "draw a figure out of the dark to the light they carry",
            "menu_freaks" => "make the game's menu go wrong the next time they open it",
            "bare_trees" => "strip a stand of trees near them bare, or skin their bark",
            "out_of_place" => "set out something that does not belong where they will come on it",
            "cave_torches_flicker" => {
                "flicker their torches in the cave, one after another toward them"
            }
            "prints_around_the_house" => "leave more footprints round their house, that stay",
            "dead_redstone" => "make their redstone by the house dead a while",
            "sound_warp" => "make every sound they hear go wrong a while",
            "villager_flees_in" => "send a villager banging at their door, to run in and die",
            "breathing_in_the_wall" => "breathe slowly inside the wall beside them",
            "scream_indoors" => "scream like a goat from another room of their house",
            "item_break_behind" => "snap a tool right behind them",
            "armour_behind" => "clank like armour walking behind them",
            "breath_in_the_chest" => "breathe from inside the chest they use",
            "rain_on_the_roof" => "drum rain on their roof under a clear sky",
            "sniffing" => "sniff after them in the dark",
            "gasp_when_turning" => "gasp behind them, as if startled",
            "water_where_none" => "splash water in a dry corner of their house",
            "doors_slam_far" => "slam every door far off at once",
            "the_count" => "ring the bell at their house once more than last time",
            "rattling_round_the_house" => "rattle bones round the outside of their house",
            "chains_dragging" => "drag chains along the stone behind them",
            "shriek_below" => "shriek from deep under their house",
            "witch_laughs" => "laugh like a witch far off in the dark",
            "pet_whines_at_door" => "set their dog whining at the door all night",
            "glass_rubbed" => "drag a finger slowly across their window",
            "bones_underfoot" => "crunch bone under their steps",
            "bubbling_under" => "bubble water under their floor",
            "the_bell_in_the_chest" => "ring a bell inside the chest they use",
            "panting_behind" => "pant right behind them as they run",
            "iron_door_below" => "slam an iron door far under them",
            "ice_cracks" => "crack the ice under them",
            "bush_rustles" => "rustle a bush hard beside them",
            "snoring_beside" => "snore softly beside them in bed",
            "sigh_at_the_door" => "sigh just outside their door",
            "humming_ahead" => "hum a few notes in the tunnel ahead",
            "steps_counted" => "count their steps in notes, one higher each",
            "cart_overhead" => "roll a minecart through the rock over them",
            "ram_on_the_wall" => "ram the outside of their wall three times",
            "bell_underwater" => "ring a bell under the water below them",
            "bark_from_home" => "bark from inside their empty house",
            "hoofbeats_circling" => "gallop hoofbeats round the outside of their house",
            "snipping_in_the_dark" => "snip shears beside their sheep in the dark",
            "eating_in_the_wall" => "chew slowly inside the wall beside them",
            "ominous_hum" => "hum like a trial spawner under the floor",
            "vault_opens_by_the_bed" => "unlock a vault beside their bed",
            "creaking_behind" => "creak and step behind them in the woods",
            "bulbs_clicking" => "click copper bulbs on and off in the room",
            "gust_from_nowhere" => "burst a gust of wind beside them that pushes them",
            "digging_under_the_floor" => "dig and snuffle under the floor",
            "rustle_in_the_pack" => "rustle something in their pack as they open it",
            "wings_in_the_room" => "beat wings close round them in the room",
            "lava_in_the_wall" => "pop lava inside the wall beside them",
            "crossbow_click" => "load a crossbow right behind them",
            "teleport_beside" => "warp twice beside them, nearer each time",
            "sweeping_next_room" => "sweep with a brush in the next room",
            "pot_shatters" => "shatter a pot somewhere in the house",
            "blow_on_the_shield" => "strike their shield from nothing",
            "creeper_behind" => {
                "set a creeper hissing right behind them, that goes off and harms no one"
            }
            "vex_single" => "send a vex at them out of nowhere",
            "vex_strikes" => "send a vex at them out of nowhere to strike hard",
            "vex_wave" => "send a swarm of vexes at them",
            "zombie_in_your_gear" => "send a zombie at them dressed in what they wear",
            "wolves_circle" => "set wild wolves circling them in the dark",
            "drops_from_the_ceiling" => "drop a cave spider from the ceiling before their face",
            "daylight_phantom" => "send a phantom at them out of the noon sun",
            "wall_bursts" => "break a block of their wall and pour silverfish out of it at them",
            "villager_collapses" => "drop a villager dead before their eyes, for a minute",
            "villagers_walk_out" => "walk the villagers out of the village at dusk, single file",
            "villager_spins" => "set a villager turning round and round where it stands",
            "trades_refused" => "have every villager refuse them for a day",
            "villager_stabs" => "have the next villager they trade with stab them",
            "villagers_flee_you" => {
                "make every villager run from them as they come into the village"
            }
            "circles_a_tree" => "leave footprints round and round a tree in the snow",
            "animals_flee_the_woods" => {
                "send the animals running out of the woods past them, from something"
            }
            "tree_comes_down" => "bring a tree down across their way",
            "trees_close_behind" => "grow a tree in the way they came into the woods",
            "the_tree_line" => {
                "stand figures between the trees behind them as they leave the woods"
            }
            "branch_snaps" => "snap a branch in the tree they just passed",
            "blazed_trees" => "blaze a trail on the trees ahead of them",
            "knocks_on_wood" => "knock on the trees ahead of them, deeper and deeper in",
            "tree_to_tree" => {
                "set the figure behind the trees, a tree nearer each time they look back"
            }
            "taken_up" => "take their most valuable things up into the sky before their eyes",
            "sky_gives_back" => "let what it took up fall out of the sky before their door",
            "lantern_in_the_woods" => "walk a figure with a lantern through the dark far off",
            "rides_the_cart" => "roll their minecart past with a figure in it",
            "path_through_the_trees" => "cut a straight line through the trees toward their house",
            "in_the_treetops" => "stand the figure on top of the trees far off",
            "on_the_water" => "stand the figure on the water far off",
            "climbs_ahead" => "have the figure climb their shaft above them",
            "trapdoor_taps" => "tap a trapdoor of theirs open and shut",
            "chest_rattles" => "rattle their chest's lid open and shut",
            "its_banner" => "hang a black banner on its house",
            "stands_in_the_rain" => "stand the figure in the rain, its face to the sky",
            "swims_beneath" => "have the figure swim past beneath them",
            "by_the_bed" => "stand the figure beside their bed as they come in",
            "fishing_figure" => "have the figure fish where they fish",
            "breadcrumbs" => "lay a line of their blocks from their door off into the dark",
            "torch_ring" => "ring the house with torches",
            "name_in_the_snow" => "tread their name into the snow near the house",
            "chest_wont_open" => "hold their chest shut as they come to it, and knock inside",
            "lost_hour" => "take an hour of their day in a blink",
            "bats_in_the_house" => "let bats loose in the house a minute",
            "pet_stares_at_wall" => "stand their pet staring at a wall of the house",
            "glass_breaks" => "make the sound of a window breaking in the house",
            "slips_from_hand" => "make what they hold slip from their hand",
            "stone_circle" => "raise a ring of stone pillars far off",
            "camp_nearby" => "make a camp out where they walk, a figure sitting by its fire",
            "flowers_wilted" => "wilt their flowers to dead bushes for a day",
            "every_door_opens" => "open every door of the house at once",
            "mirror_in_the_window" => "have a figure in their window copy their every move",
            "eats_at_their_chest" => {
                "have the figure eat their food at their chest as they come home"
            }
            "rides_their_horse" => "sit the figure on their horse",
            "prints_to_the_cave" => "leave prints in the snow from their door to a cave",
            "waves_from_inside" => {
                "have the figure wave at them from inside a window as they come home"
            }
            "jumping_far" => "have the figure jump in place far off",
            "at_their_table" => {
                "stand the figure at their crafting table, working, as they come home"
            }
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
            "anvil_sound" => "ring the anvil in the workshop",
            "page_turn" => "turn the pages by the lectern",
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
            "knock_heavy" => "pound on the door, heavy and slow",
            "steps_above" => "walk across the floor above them",
            "scratching" => "scratch inside the wall beside them",
            "ladder_climb" => "climb a ladder near them",
            "pickup_pop" => "pick something up behind them",
            "cold_crackle" => "crackle in their cold furnace",
            "breath_close" => "breathe at their ear",
            "cave_air" => "bring the sound of a cave into the open",
            "door_scratch" => "scratch at the door",
            "returned_used" => "give back what it took, a little used",
            "book_word" => "write one word in their book and sign it",
            "hopper_emptied" => "empty their hopper into the chest it feeds",
            "chest_full_of_one" => "fill every empty slot of their chest with one dirt",
            "shulker_moved" => "move their shulker box to another wall",
            "map_marked" => "mark its place on their map",
            "renamed_item" => "write a name on one of their things",
            "chest_answers" => "close another chest as they open theirs",
            "loot_gift" => "leave something of its own in the loot they find",
            "door_hinge" => "hang a door on the other side of its frame",
            "ladder_gap" => "take a rung out of a ladder and leave it on top",
            "flowers_potted" => "pick the flowers outside and pot them by the door",
            "pumpkin_face" => "carve a pumpkin to face the door",
            "tree_grown" => "grow the sapling by the house into a tree",
            "backfill" => "fill in a hole they dug by the house",
            "alcove" => "dig a hollow behind a wall of the house",
            "furniture_shifted" => "move a work block of theirs one block over",
            "one_room_dark" => "take down every torch in one room",
            "door_held" => "hold the door shut a moment as they open it",
            "bed_turned" => "turn their bed round",
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
            "other_house" => "wake them in a copy of their house, somewhere else",
            "diary_sold" => "have a villager sell its diary of them",
            "diary_brought" => "have a villager bring them its diary",
            "letter_in_chest" => "leave a letter for them in their chest",
            "villager_named" => "give the villager near them a name",
            "lectern_notes" => "leave its notes of them on a lectern",
            "page_in_your_book" => "write a page in their book",
            "pet_outside" => "put the pet outside the door",
            "tab_entry_stays" => "show their name twice all session",
            "note_tune" => "play three notes on the note block",
            "piston_once" => "work the piston once",
            "disc_phantom" => "play a disc they do not own",
            "fog_close" => "close the fog in for a minute",
            "snow_indoors" => "let it snow indoors for a moment",
            "rain_indoors" => "let it rain indoors for a moment",
            "scan_ore" => "look at the rock around them",
            "ore_marked" => "put a torch on an ore they have not found",
            "hand_on_shoulder" => "turn them a little, a step right behind them",
            "pack" => "walk their animals toward them together",
            "slow_approach" => "walk one animal at them very slowly",
            "quick_approach" => "run one animal at them",
            "answer_knock" => "knock back on the door they knocked on",
            "ghast_name" => "make the ghasts cry their name",
            "arrow_near" => "shoot an arrow from the dark, just past their head",
            "arrow_grazes" => "shoot an arrow from the dark that grazes them",
            "arrow_hits" => "shoot an arrow from the dark that hits them",
            "splash_dark" => "throw a potion at their feet from the dark",
            "brew_left" => "leave a potion of its own in their chest",
            "potion_changed" => "turn one of their potions into another",
            "door_cloud" => "leave a cloud at their door for the morning",
            "ender_holds" => "give an enderman the block they build with",
            "busy_away" => "do something at home while they are in another world",
            "item_frames_face" => "show the one thing in every frame of the house",
            "wrong_sky" => "put the moon in the noon sky",
            "fish_on_line" => "pull their bobber under with nothing on it",
            "hostiles_close" => "bring the monsters closer whenever they look away",
            "shove" => "push them at the next edge, a short fall",
            "shove_far" => "push them at the next edge, a long fall",
            "breath" => "breathe at their ear while they crouch still",
            "footsteps_follow" => "walk a step behind them in the dark",
            "name_called" => "say their name where only the subtitles hear it",
            "into_the_mountain" => "have the figure dig into the rock ahead of them",
            "through_the_hill" => "have the figure walk through the hill",
            "into_the_dark" => "have the figure step back out of their light",
            "copy_crouch" => "have the far figure crouch when they crouch",
            "tools_in_hand" => "have the far figure work the rock with their tool",
            "house_near" => "build its own house near theirs, a third a night",
            "house_grows" => "add a room to its house",
            "pillar" => "raise its pillar a block",
            "hut_on_route" => "build a log hut halfway along their route",
            "dock" => "lengthen its dock where they fish",
            "bridge" | "bridge_done" => "lengthen its bridge toward their shore",
            "copy_build" => "lay the outline of their house at its place",
            "copy_house" => "build more of the copy of their house",
            "cave_room" => "dig itself a room in a cave",
            "figure_nip" => "have the figure behind them strike once",
            "figure_swing" => "have the figure behind them strike hard once",
            "figure_mirror" => "have the figure copy their steps from afar",
            "figure_window" => "stand the figure outside their window",
            "figure_crowd" => "stand the figure among the villagers",
            "figure_waves" => "have the far figure wave",
            "mine_beside" => "dig a copy of their tunnel beside it",
            "tunnel_sealed" => "wall up the mouth of their oldest tunnel",
            "tunnel_toward" => "dig toward them while they dig",
            "portal_lit" => "light their portal",
            "mined_out" => "mine out the ore ahead of their tunnel",
            "ladder_down" => "dig a shaft down with their ladders",
            "tunnel_goes_on" => "dig their tunnel five blocks further",
            "torches_far_end" => "put out their tunnel's torches from the far end",
            "sign_deep" => "leave a sign at the end of their tunnel",
            "way_back_closes" => "close their tunnel behind them",
            "village_lantern" => "carry a village lantern onto their way",
            "villager_word" => "have a villager sell what it took",
            "village_abandoned" => "empty the village into one house until dawn",
            "bell_twice" => "ring the bell again after they ring it",
            "mist" => "let a mist roll in",
            "dusk_noon" => "drain the light at noon",
            "red_mist" => "let a red mist roll in",
            "darkness" => "close the dark in around them",
            "moon_wrong" => "make the moon full",
            "stars_out" => "put out the stars",
            "night_early" => "move the sky two hours on",
            "painting_flip" => "hang a painting upside down until they look at it",
            "compass_points" => "turn their compass toward its place",
            "hunger_shake" => "shake their hunger bar",
            "heart_skip" => "make their hearts flash",
            "subtitle" => "show footsteps in their subtitles",
            "tab_twice" => "show their name twice in the player list",
            "reflection" => "show a face under the water",
            "trail" => "leave footprints in the snow from their door",
            "trail_in" => "leave footprints in the snow up to their door",
            "boat_waiting" => "have their boat waiting at the shore ahead of them",
            "ripples" => "make ripples cross the water toward them",
            "fishing_line" => "fish in their water with no one there",
            "snowman" => "stack a snowman by their door",
            "boat_in_house" => "put their boat inside their house",
            "under_the_water" => "show them a shape under the water",
            "wet_steps" => "walk wet steps from their door to their bed",
            "jukebox" => "play one of their discs in their jukebox",
            "lectern_page" => "turn the page of the book on their lectern",
            "swap_fuel" => "swap the fuel in their furnace for another",
            "brewing_done" => "brew a potion in their brewing stand",
            "grindstone_strip" => "take an enchantment off one of their things",
            "lamps_blink" => "blink their lamps in a pattern",
            "composter_fills" => "fill their composter a little",
            "crafted_for_you" => "make something for them on the crafting table",
            "bell_rings" => "ring their bell once",
            "sleep_message" => "tell them another player is sleeping",
            "advancement" => "show them an advancement of theirs again",
            "death_message" => "say in chat that they died",
            "words_on_sign" => "write what they typed on a sign by their door",
            "ping_fades" => "take away their connection in the player list for a minute",
            "empty_message" => "send an empty message in their name",
            "sign_word" => "change one line of their sign to a word of its own",
            "count_up" => "add one to the count on its kill sign",
            "weapon_on_wall" => "hang their weapon in their frame",
            "head_on_post" => "put a head on a post by their door",
            "grave_each" => "dig a grave for every kind they killed",
            "scarecrow" => "stand a scarecrow in their field, wearing their armour",
            "bees_out" => "let the bees out around their door",
            "horse_moved" => "tie their horse to another post",
            "field_word" => "harvest a word into their field",
            "face_the_house" => "make their animals stand facing the house",
            "one_more" => "put one more animal in their pen until noon",
            "row_withered" => "wither one row of their crops",
            "eggs_to_door" => "lay eggs in a line to their door",
            "animals_nip" => "have an animal turn on them and nip",
            "animals_turn" => "have their animals turn on them",
            "shift_bed" => "move their bed one block over",
            "sheets_changed" => "change the colour of their bed",
            "under_the_bed" => "scratch under their bed as they lie down",
            "foot_of_bed" => "stand at the foot of their bed as they wake",
            "second_bed" => "put a second bed beside theirs",
            "warped_animal" => "make one of the animals wrong for a minute, watching them",
            "lights_out" => "put out their lights behind them one by one, for a minute",
            "soul_swap" => "turn one of their lights to soul fire",
            "death_light" => "leave a lit candle where they died",
            "light_taken" => "carry their last torch a few blocks on, the way they went",
            "lantern_ahead" => {
                "leave one of their lanterns lit far down a cave they have not been in"
            }
            "cave_lit" => "light a cave they have not been in, the way they light their tunnels",
            "wall_swap" => "change one block of the house for its cousin",
            "torch_blue" => "make one of their torches burn blue",
            "window_bars" => "turn a window pane to iron bars",
            "bed_breaks" => "break the bed under them as they lie down",
            "choke" => "walk to their bed as they sleep and choke them with a lead",
            "shot_in_bed" => "walk to the foot of their bed as they sleep and shoot them",
            "burned_in_bed" => "walk to their bed as they sleep and set them alight",
            "ring_of_figures" => "stand figures in a ring round the house while they are inside",
            "upside_down" => "hang the figure from the ceiling over their bed as they wake",
            "the_crawl" => "have the figure crawl from their door to their bed while they are in",
            "hunted" => "have the figure run at them out of the dark",
            "boat_passenger" => "sit the figure in their boat behind them",
            "every_window" => "stand a figure outside every window of the house at once",
            "one_more_each_night" => {
                "stand one more figure at the edge of their light than last night"
            }
            "door_wont_open" => "hold their door shut as they come to it, and knock",
            "sleepwalk" => "wake them outside their front door, the door open behind them",
            "torch_flash" => "show a figure in the light they just placed underground",
            "where_you_died" => "lay a figure where they last died",
            "rubberband" => "set them back a few steps and stand a figure where they were",
            "the_offer" => "hold out what it took from their doorway, and give it if they come",
            "top_of_the_ladder" => "stand the figure at the top of their shaft looking down",
            "pulled_under" => "pull them under the water a moment as they swim",
            "its_lights_on" => "light its house one light at a time, then put them out",
            "a_face_in_the_chest" => "lay out a face in their chest's grid",
            "falls_from_the_sky" => "drop the figure from the sky far off as they look up",
            "pet_follows" => "walk their pet off to a figure ahead of them",
            "lies_beside" => "lay the figure on the floor beside their bed as they sleep",
            "respawn_its_bed" => "make the bed in its house their respawn point",
            "ender_chest_note" => "leave a note in their ender chest",
            "eye_turns_home" => "turn the next eye of ender they throw toward their house",
            "crossing_ahead" => "walk the figure across their way far ahead",
            "in_a_line" => "stand figures in single file along their route, facing the house",
            "peeks_from_ground" => "show the figure's head alone above the ground far off",
            "wears_your_face" => "put their own head on a zombie and walk it at them",
            "animal_follows" => "have one of their animals follow them all day",
            "held_breath" => "make their air run out twice as fast under water",
            "all_freeze" => "stop every animal and mob around them at once",
            "figure_on_map" => "mark someone at their house on the map they hold",
            "waits_at_the_portal" => "stand the figure beside their portal as they come back",
            "firework_over_its_house" => "send a firework up over its house, bursting as a face",
            "chicken_lays" => "have a chicken lay what it took instead of an egg",
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
            "mining_sounds" => "dig with a pickaxe in the rock near them",
            "echo_steps" => "answer their footsteps in the tunnel",
            "late_eating" => "take one more bite after theirs",
            "scan_village" => "look over the village",
            "village_doors" => "open every door in the village",
            "lamps_noon" => "turn the lamps on at noon",
            "plate_laid" => "lay a pressure plate before the chest",
            "door_opens_itself" => "open the door as they reach it",
            "bell" => "ring the bell once",
            "note" => "play one note on the note block",
            "lever" => "flip the lever",
            "door_sound" => "make a door sound in another room",
            "chest_sound" => "make a chest sound in another room",
            "cut_music" => "stop the music",
            "late_steps" => "echo their own footsteps a moment late",
            "knock" => "knock on the door",
            "place_sound" => "place a block behind them, by sound alone",
            "lever_click" => "click a lever in a house with none",
            "drip" => "dripping water indoors",
            "wolf_far" => "make a wolf whine far off",
            "thunder_clear" => "bring thunder under a clear sky",
            "arrow_hit" => "sound an arrow hitting the wall behind them",
            "portal_sound" => "hum like a portal in the house",
            "your_voice" => "make their own hurt sound in the next room",
            "silence" => "silence the world's sounds for an evening",
            "flicker" => "flicker a light",
            "lights_fail" => "put the lights out for a while",
            "sun_blink" => "make the sun blink out",
            "lightning_silent" => "flash lightning with no thunder",
            "rumble" => "shake the ground under them with a deep rumble",
            "torches_turn" => "turn every wall torch of the house toward their bed",
            "your_grave" => "dig a grave at its place with their name on it",
            "cobwebs" => "hang their house with cobwebs overnight",
            "false_door" => "set a door on an inside wall that opens on the wall",
            "static" => "cover their screen in static a moment",
            "anchor_charged" => "show a charged respawn anchor at its place",
            "dripstone_falls" => "drop pointed dripstone from the ceiling beside them",
            "wither_rose_in_pot" => "put a wither rose in one of their flower pots",
            "mushrooms_in_corners" => "grow mushrooms in the corners of their house",
            "shulker_lid_peeks" => "open their shulker box a crack by itself",
            "sign_with_coords" => "put a sign by their door saying where they sleep",
            "trident_planted" => "plant a trident upright in the ground at its place",
            "slam_crater" => "boom a mace's smash near the house and leave a crater",
            "one_step_more" => "add one more step to the foot of their stairs",
            "room_closes_in" => "move one wall of their room a block further in",
            "trapdoor_to_nothing" => "set a trapdoor in their floor over a lit pit",
            "bed_fenced" => "fence their bed round while they sleep",
            "tools_laid_out" => "lay their tools out on the floor in a row",
            "dripstone_in_tunnel" => "grow a column of dripstone across their tunnel",
            "skull_in_the_tunnel" => "set a skull into their tunnel's wall facing them",
            "copper_ages" => "weather their copper a stage overnight",
            "pale_horse_in_your_pen" => "stand a skeleton horse among the animals in their pen",
            "fangs_in_a_line" => "snap a line of evoker fangs up toward them",
            "beam_from_the_water" => "charge a guardian's beam at them from the water",
            "bucket_fish_changed" => "change the fish in their bucket to another kind",
            "head_on_the_pillow" => "lay a head with their own face on their pillow",
            "empty_lead" => "knot a lead on a post at its place with nothing on it",
            "dried_ghast_on_doorstep" => "sit a dried ghast on their doorstep",
            "eyeblossoms_open" => "open every eyeblossom near them by day",
            "golem_statue_turns" => "stand a copper golem statue by their door that turns to them",
            "shelf_shuffled" => "shuffle the things on their shelf",
            "fireflies_go_out" => "put out the fireflies near them one by one",
            "trial_key_left" => "leave a trial key on their doorstep",
            "title_your_name" => "fill their screen with their own name a second",
            "recipe_toast" => "pop up a recipe toast for them",
            "count_shows_zero" => "show every count in their hotbar as 0",
            "slot_barred" => "bar one slot of their inventory",
            "held_name_wrong" => "make their held item's name read its",
            "title_splash" => "put a line about them in the title screen's splash",
            "enchant_six" => "make every enchanting offer cost 6",
            "compass_spins" => "spin their compass as in the Nether",
            "figure_digging" => "dig at its place, the figure with a shovel",
            "figure_bows" => "bow to them far off, the figure",
            "climbing_the_house" => "climb the outside of their house past a window",
            "figure_at_the_lectern" => "stand at their lectern reading, the figure",
            "figure_in_your_armour" => "stand far off in the armour they wear, the figure",
            "on_the_peak" => "stand on the nearest peak at sunset, the figure",
            "cattle_turn_away" => "turn every cow near them away from them",
            "horse_balks" => "make the horse they ride balk at nothing",
            "cat_gift" => "lay a gift from their chest by the bed, their cat",
            "armadillos_curl" => "curl every armadillo near them up at once",
            "sniffer_at_its_place" => "stand a sniffer snuffling at its place",
            "refuses_food" => "make the next animal they feed refuse it",
            "the_child_stares" => "make the young animals near them stare at them",
            "late_drop" => "hold back what they next mine a moment",
            "bundle_gains" => "slip a pebble into their bundle",
            "leather_dyed" => "dye their leather armour black overnight",
            "ore_counted" => "count each diamond they mine with a chime",
            "snowball_comes_back" => "throw back the next snowball they throw",
            "arrow_returned" => "bring back the next arrow they shoot",
            "edge_tug" => "tug them toward the drop as they crouch at an edge",
            "ash_on_the_wind" => "drift ash down round them from a clear sky",
            "red_petals" => "let red petals fall round them",
            "sculk_trail" => "creep sculk's charge along the floor to their bed",
            "false_glint" => "make the thing in their hand shimmer as if enchanted",
            "piglins_stare" => "make every piglin near them stop and watch them",
            "strider_turns_home" => "turn the strider they ride toward their house",
            "ghast_watches" => "make a ghast near them stop and watch them",
            "hoglins_part" => "make the hoglins near them stand off and not charge",
            "magma_cubes_settle" => "make every magma cube near them stop bouncing",
            "back_to_the_wall" => "turn the wither skeletons near them to the wall",
            "endermen_turn_away" => "turn every enderman near them away from them",
            "zombified_follow" => "make the zombified piglins near them follow when unwatched",
            "rain_in_the_nether" => "rain on them in the Nether, the sound alone",
            "their_door_behind" => "open and shut a door behind them in the Nether",
            "nether_roof_steps" => "walk steps across the rock over them",
            "home_through_the_portal" => "let the sounds of home through their portal",
            "the_dragon_growls" => "growl the dragon's growl from the empty island",
            "second_wings" => "beat a second pair of wings behind them",
            "purpur_knock" => "knock on the purpur wall beside them",
            "ship_creaks" => "creak the deck as if someone walked it",
            "souls_follow" => "raise the souls only where they have just stepped",
            "breath_in_the_nether" => "show their breath in the heat of the Nether",
            "their_torch_ahead" => "light a torch ahead of them in the fortress",
            "name_in_netherrack" => "cut their name into the netherrack behind them",
            "prints_in_soul_sand" => "lay prints ahead of them leading away",
            "prints_off_the_edge" => "lay prints from them to the island's edge",
            "prints_from_the_portal" => "lay prints from their portal to their door",
            "portal_weeps" => "make the portal they came through weep",
            "compass_steadies" => "steady their compass on where they came in",
            "standing_in_the_portal" => "stand the figure in the portal they came through",
            "far_end_of_the_bridge" => "stand the figure at the far end of the fortress bridge",
            "name_from_the_void" => "call their name from the void below them",
            "standing_on_the_void" => "stand the figure on nothing over the void",
            "splash_in_the_nether" => "throw a splash potion at their feet from the Nether's dark",
            "someone_went_through" => "sound someone going through their portal",
            "lodestone_wrong" => "turn their compass to their bed",
            "ruined_portal_glows" => "make the ruined portal near them weep",
            "potion_at_the_window" => "shatter a splash potion against their window",
            "came_back_wrong_hour" => "show them the wrong hour's sky as they come back",
            "lightning_below" => "strike lightning beside them where there is no weather",
            "shulkers_open" => "open every shulker near them at once",
            "gateway_flashes" => "flash the end gateway near them as if used",
            "blaze_ring" => "ring them round with the blazes near them",
            "endermen_at_the_edge" => "stand the endermen near them in a row facing the void",
            "end_goes_quiet" => "hush the End round them a minute",
            "chorus_blooms" => "flower every chorus plant near them at once",
            "egg_on_the_pillow" => "move their dragon egg onto their pillow",
            "chorus_lands_home" => "set them down on their doorstep as they eat a chorus fruit",
            "pearl_turned" => "turn them round where their next pearl lands them",
            "their_own_potion" => "throw one of their own potions at their feet from the dark",
            "vines_reach" => "grow the vines near them toward them",
            "portal_goes_out" => "put out their portal as they reach it, and light it again",
            "lava_goes_still" => "still the lava round them a moment",
            "end_sky_face" => "make a face in the End's sky",
            "barter_from_home" => "give a piglin near them one thing from their chest at home",
            "elytra_at_home" => "take the ship's elytra and hang it at their home",
            "on_the_exit_portal" => "stand the figure on the exit portal as the dragon dies",
            "the_poem_knows" => "arm the end poem to know them",
            "leaves_at_the_door" => "leave a little leaf litter on their doorstep",
            "smoke_from_the_roof" => "send smoke up from the top of their house",
            "wrong_footing" => "make their steps outside sound like their floorboards",
            "plate_clicks" => "click one of their pressure plates down",
            "book_moved_on_shelf" => "move the books on their bookshelf one slot along",
            "skull_turned" => "turn the head they set out to face their bed",
            "pet_got_up" => "have the pet they left sitting stand up and look at the door",
            "light_in_your_window" => "light a window of their house a moment from inside",
            "furnace_glows" => "light their cold furnace a moment",
            "paper_on_the_floor" => "leave a sheet of paper inside their door",
            "a_glint_on_the_hill" => "flash a glint at them from far off",
            "heartbeat_in_the_stone" => "beat a heart inside the rock ahead of them",
            "thunder_behind" => "strike lightning ahead and sound its thunder behind them",
            "bed_creaks" => "creak their bed as if someone sat on it",
            "stand_bubbles" => "bubble their brewing stand",
            "knock_from_below" => "knock from under the bedrock",
            "thunder_counts" => "count the thunder down from five",
            "name_echoes" => "call their name across the mountains",
            "dust_devil" => "send a whirl of dust across the field at them",
            "lightning_rod_sparks" => "spark their lightning rod",
            "light_below_the_boat" => "keep a light under their boat",
            "lights_over_the_swamp" => "hang lights over the swamp that drift away",
            "smoke_signals" => "send their campfire's smoke up in signals",
            "cauldron_ripples" => "splash the water in their cauldron",
            "leaves_fall_inside" => "drift a few leaves down from their ceiling",
            "dust_on_the_road" => "raise dust far along the road ahead",
            "the_grass_parts" => "part the grass in a line toward them",
            "a_cloud_of_its_own" => "rain on them alone under a clear sky",
            "snow_in_the_heat" => "snow on them in the heat",
            "rain_stays" => "keep the rain on them after it stopped",
            "lantern_down_the_river" => "float a light down the river past them",
            "lightning_round_you" => "strike lightning round them in a closing ring",
            "bolts_in_a_line" => "strike lightning in a line toward its place",
            "door_bangs_in_the_storm" => "bang their door with the thunder",
            "figure_in_the_flash" => "show the figure in the lightning, nearer each flash",
            "figure_in_your_field" => "have the figure tend their field far across it",
            "against_the_sun" => "stand the figure under the sun",
            "flies_beside_you" => "glide the figure beside them",
            "the_city_candles" => "light the city's candles one by one toward them",
            "glow_berries_go_dark" => "put out the glow berries round them",
            "cracks_in_the_wall" => "crack a block of their wall as if from the other side",
            "sculk_creeps" => "spread sculk over their walls low down",
            "axolotls_play_dead" => "make every axolotl near them play dead",
            "dripleaf_steps" => "tip the dripleaves near them as if walked across",
            "prints_on_the_ceiling" => "walk dark footprints across their ceiling to the bed",
            "window_tinted" => "tint one of their windows black for a day",
            "happy_ghast_overhead" => "ride a happy ghast over their house",
            "crafter_clicks" => "make their crafter craft once by itself",
            "sand_gives_back" => "bury what it took in a patch of sand by the house",
            "pot_of_returns" => "leave what it took in a pot at its place",
            "pale_moss_creeps" => "creep pale moss across their floor",
            "creaking_in_the_field" => "stand a creaking in their field that comes while unwatched",
            "copper_golem_at_door" => "stand a copper golem outside their door",
            "boss_bar_it" => "fill a boss bar named for it over their screen",
            "recovery_compass_lies" => "turn their recovery compass to its place",
            "spyglass_figure" => "stand far off where they look through a spyglass",
            "red_trail" => "leave red specks from their door to their bed",
            "snow_shape" => "press the shape of a body into the snow by the house",
            "clock_wrong" => "turn the clock in their hand to night",
            "cold_house" => "make their house cold enough to see their breath",
            "the_killer_rabbit" => "set a red-eyed rabbit on them",
            "phantom_item" => "slip a strange paper into their inventory a moment",
            "border_closes" => "close the world's edge round them a moment",
            "frost_on_the_glass" => "frost their windows over from inside",
            "shadow_on_the_wall" => "cast a shadow of someone on the wall behind them",
            "stairs_up_to_nowhere" => "build stairs from their floor into the ceiling",
            "face_in_the_canopy" => "cut a face into the leaves of a tree by the house",
            "boots_at_the_door" => "stand a pair of boots like theirs inside the door",
            "nightmare" => "flash the figure's face at them as they wake",
            "ceiling_crawl" => "crawl across the ceiling over them",
            "in_the_lava" => "stand in the lava facing them",
            "over_the_edge" => "peek over the cliff edge above them",
            "rider_on_the_ridge" => "watch them from a skeleton horse far off",
            "elder_face" => "show them the elder guardian's face",
            "boom_far" => "set off a far explosion from the way of its place",
            "red_rain" => "turn the rain red",
            "eyes_in_the_dark" => "show two eyes ahead of them in the dark",
            "hearts_drop" => "show half their hearts empty a moment",
            "xp_slips" => "empty their experience bar on their screen a moment",
            "f3_line" => "write one more line on their debug screen",
            "only_in_third_person" => {
                "stand the figure behind them where only the third person shows it"
            }
            "in_the_screenshot" => "put the figure in the next screenshot they take",
            "handprints" => "press handprints on a window of the house from inside",
            "false_death" => "show them the death screen for a second",
            "inventory_stare" => "make their figure in the inventory stare out at them",
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

    #[test]
    fn a_small_event_costs_a_quarter_a_noticed_one_a_whole_and_calm_counts_them_all() {
        let config = crate::config::Config::default();
        let small = CATALOGUE
            .iter()
            .find(|e| e.small && e.weight == Weight::Full)
            .unwrap();
        let noticed = CATALOGUE
            .iter()
            .find(|e| !e.small && e.weight == Weight::Full)
            .unwrap();
        assert_eq!(small.cost_in(&config), 0.25, "{}", small.id);
        assert_eq!(noticed.cost_in(&config), 1.0, "{}", noticed.id);
        let calm = crate::config::Preset::named("calm").unwrap().config();
        assert_eq!(
            small.cost_in(&calm),
            1.0,
            "calm: a small thing is any event"
        );
    }
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

    /// Every event's words start with what it does, a verb: the situation says `Today it did
    /// <words>` and `Today it could not <words>`, and a noun phrase read as `Today it could not
    /// a wolf howling far off`.
    #[test]
    fn every_event_s_words_start_with_a_verb() {
        let not_verbs = [
            "a",
            "an",
            "the",
            "their",
            "his",
            "one",
            "two",
            "three",
            "pages",
            "footsteps",
            "thunder",
            "lightning",
            "nothing",
        ];
        let nouns: Vec<String> = CATALOGUE
            .iter()
            .filter(|e| e.id != "wait")
            .filter(|e| {
                let first = e.menu_words().split_whitespace().next().unwrap_or("");
                not_verbs.contains(&first) || e.menu_words() == e.id
            })
            .map(|e| format!("{}: {}", e.id, e.menu_words()))
            .collect();

        assert!(nouns.is_empty(), "words that are not a deed: {nouns:?}");
    }

    /// Every sound the mind names exists in the game: a renamed sound fails here, not in a
    /// player's world (`entity.wolf.howl` was gone in 26.3). The list is the game's
    /// own `sounds.json`, regenerated on a version bump (`docs/upgrading.md`).
    #[test]
    fn every_sound_the_mind_names_is_one_the_game_has() {
        let known: std::collections::HashSet<&str> = include_str!("../tests/data/sounds-26.3.txt")
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .collect();
        let sources = [
            include_str!("events.rs"),
            include_str!("brain/act.rs"),
            include_str!("brain/sounds.rs"),
        ];
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
    fn every_event_has_its_description_and_needs_from_the_catalogue_page() {
        let missing: Vec<&str> = CATALOGUE
            .iter()
            .filter(|e| e.id != "wait")
            .filter(|e| {
                described(e.id).is_none_or(|(what, needs)| what.is_empty() || needs.is_empty())
            })
            .map(|e| e.id)
            .collect();
        assert!(missing.is_empty(), "no description: {missing:?}");
        let (what, needs) = described("open_chest").unwrap();
        assert!(what.starts_with("A chest in the home opens"), "{what}");
        assert!(!needs.ends_with('|'), "{needs}");
    }

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
