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
                | Need::JustOpened
                | Need::LootOpened
                | Need::DoorUsed
                | Need::Night
                | Need::Day
                | Need::Dawn
                | Need::Raining
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
            Need::NearTheirDeath => "they are not near where they last died".to_string(),
            Need::DownTheShaft => "they are not down a shaft of their ladders".to_string(),
            Need::TargetSwimming => "the target is not swimming".to_string(),
            Need::ItsHouseNear => "its house is not built, or they are not near it".to_string(),
            Need::LookingUp => "the target is not looking up at the sky".to_string(),
            Need::ItsHouse => "its house is not built".to_string(),
            Need::TargetUnderwater => "the target is not under water".to_string(),
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
    Shove,
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
    /// The same walk to the bed, a crossbow loaded instead and a shot: this much harm, and they
    /// wake burning, never below half a heart.
    ShotInBed(f32),
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
        5,
        &[TargetAwake],
        3.0,
        2,
        Full,
        Action::Shove,
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
        2,
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
        Action::Figure(FigureWhere::UnderWater, 1.0, 0.0),
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
    row(
        "pickup_pop",
        2,
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
        Action::Hint("handprints", 90.0),
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
            | Action::Shove
            | Action::BobberUnder
            | Action::Shoot(_)
            | Action::SplashDark
            | Action::DoorCloud
            | Action::Charge(..) => "seen",
            Action::Hint(_, _) | Action::GhastName => "sky",
            Action::EnderHolds => "seen",
            Action::WhileAway => "house",
            Action::HandOnShoulder => "seen",
            // Their sleep: refused, broken, someone in their bed.
            Action::RefuseRest
            | Action::Wake
            | Action::WokeElsewhere
            | Action::OtherHouse
            | Action::BedBreaks
            | Action::Choke(_)
            | Action::ShotInBed(_)
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
            "shot_in_bed" => "walk to their bed as they sleep and shoot them with a burning bolt",
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
            "red_rain" => "turn the rain red",
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
