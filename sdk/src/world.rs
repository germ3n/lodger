//! What a mod sees and does, per world: the observations the host sends once a tick, the
//! commands the mod answers with, the reports on how they went, and the clock. Every shape is a
//! plain `serde` struct so the host's Gson and the mod's `serde_json` read the same bytes. New
//! fields take `#[serde(default)]`, so an older host and a newer mod, or the reverse, read each
//! other for as long as the ABI version allows. See `docs/sdk.md`.

use serde::{Deserialize, Serialize};

/// A block position.
pub type BlockPos = [i32; 3];
/// A position in the world.
pub type Pos = [f64; 3];

/// A world as the host opens it: which one, its seed, its name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldInfo {
    /// The dimension's id, `minecraft:overworld`.
    pub id: String,
    /// The world's seed, which a mod folds into its own randomness.
    pub seed: i64,
    /// The save's name, for logs.
    pub name: String,
}

/// The clock, so a mod never reads one itself.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Now {
    /// The server's tick count since the game started: it starts again at 0 when the game
    /// does, so a mod that saves a tick to wait for keeps the last one it saw and moves its own
    /// back when this comes back lower (the lodger's `Stay::last_tick`).
    pub tick: u64,
    /// The world's time of day, 0 to 24000.
    pub day_time: u32,
    /// The in-game day, from the world's total time.
    pub day: u64,
    /// The real clock, milliseconds since the epoch.
    pub real_millis: u64,
}

/// What the host saw since the last tick. A quiet tick carries players' positions and nothing
/// else; the rest comes when it happens, on a schedule, or on request.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Observations {
    pub players: Vec<Player>,
    /// Entities within 64 blocks of any player, once a second.
    pub entities: Vec<Entity>,
    /// The world's weather and sky, once a second.
    pub world: Option<WorldState>,
    /// Things that happened: a chest opened, a block placed, sleep refused, a death.
    pub events: Vec<Event>,
    /// Chat lines players typed. Reading them is the mod's business; see `docs/good-to-know.md`.
    pub chat: Vec<ChatLine>,
    /// Answers to `Command::Scan`.
    pub scans: Vec<Scan>,
    /// The daily pass over the surroundings of a point the mod named.
    pub surroundings: Option<Surroundings>,
    /// The other mods loaded, once, on the first tick.
    pub mods: Vec<LoadedModInfo>,
}

/// A player, as the server knows them. `index` is stable for the session and is how events,
/// chat and commands name a player.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Player {
    pub index: u32,
    pub name: String,
    pub uuid: String,
    pub pos: Pos,
    pub yaw: f32,
    pub pitch: f32,
    pub velocity: Pos,
    pub sneaking: bool,
    pub sprinting: bool,
    pub swimming: bool,
    pub sleeping: bool,
    /// What they ride, by entity kind, or empty.
    pub riding: String,
    pub health: f32,
    pub hunger: u32,
    pub air: i32,
    pub level: u32,
    pub main_hand: ItemStack,
    pub off_hand: ItemStack,
    pub armor: Vec<ItemStack>,
    /// Effect ids, `minecraft:night_vision`.
    pub effects: Vec<String>,
    pub dimension: String,
    pub biome: String,
    /// The block light where they stand, 0 to 15, and the sky light.
    pub light: u8,
    pub sky_light: u8,
    /// What they look at within 8 blocks, if anything.
    pub looking_at: Option<Target>,
    /// How they have been moving, from the last seconds.
    pub movement: Movement,
}

/// How a player moves, read off their positions and facing over the last few seconds, so a mod
/// can read fear without being told (`docs/entity.md`, Senses).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Movement {
    /// Blocks a second over the last five seconds.
    pub pace: f32,
    /// Stops of a second or more in the last minute.
    pub stops: u32,
    /// Turns of more than 120 degrees in the last minute.
    pub turns: u32,
    /// A turn and back within two seconds, in the last minute.
    pub look_backs: u32,
    /// Seconds moving opposite to the facing in the last minute.
    pub backing_away: f32,
    /// Sprint starts in the last minute.
    pub sprint_bursts: u32,
    /// Seconds standing still right now.
    pub still_seconds: f32,
}

/// A block or an entity a player looks at.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    Block { pos: BlockPos, block: String },
    Entity { id: u32, entity: String },
}

/// An item and how many, and in a container's contents which slot it sits in.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemStack {
    /// `minecraft:iron_pickaxe`; empty for nothing.
    pub id: String,
    pub count: u32,
    /// The slot, where it matters (a container's contents); 0 otherwise.
    pub slot: u32,
}

/// An entity near a player.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Entity {
    /// The server's entity id, stable while it is loaded.
    pub id: u32,
    /// `minecraft:cow`.
    pub kind: String,
    pub pos: Pos,
    /// Its custom name, or empty.
    pub name: String,
    /// The uuid of its owner if tamed, or empty.
    pub owner: String,
    pub hostile: bool,
    /// The index of the player it faces within 10 degrees, if any.
    pub looking_at_player: Option<u32>,
    /// Tied by a lead to a fence post. ABI 27.
    pub leashed: bool,
    /// A young one (a baby villager cannot trade). ABI 53.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub baby: bool,
}

/// The weather and the sky.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldState {
    pub raining: bool,
    pub thundering: bool,
    /// The world's difficulty: `peaceful`, `easy`, `normal`, `hard`. ABI 41.
    #[serde(default)]
    pub difficulty: String,
    /// 0 to 7, 0 the full moon.
    pub moon_phase: u8,
}

/// Something a player did or had happen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Event {
    pub player: u32,
    pub kind: EventKind,
    pub pos: Option<BlockPos>,
    /// What it concerned: a block id, a dimension, an entity kind.
    pub detail: String,
    /// A container's contents, for `OpenedContainer`.
    pub contents: Vec<ItemStack>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    #[default]
    Joined,
    Left,
    OpenedContainer,
    ClosedContainer,
    PlacedBlock,
    BrokeBlock,
    SleptBegan,
    SleptEnded,
    SleepRefused,
    Ate,
    Fished,
    Mounted,
    Dismounted,
    Died,
    Respawned,
    /// A chest's fresh loot got the mod's armed offering (`arm_loot`): `detail` is the item
    /// placed, empty for an offering spent with nothing to give.
    LootTouched,
    /// The player opened or closed a door: `pos` is the door.
    UsedDoor,
    /// The player rang a bell: `pos` is the bell. ABI 31.
    RangBell,
    /// The player knocked on a door, struck it twice within a second and a half: `pos` is
    /// the door's lower half. ABI 38.
    Knocked,
    /// A confrontation (`confront`) ended: they won it. ABI 43.
    FightWon,
    /// They lost it (they died once, keeping everything). ABI 43.
    FightLost,
    /// They ran from it, or left: it is over without an end. ABI 43.
    FightFled,
    /// A figure shown to them went: `pos` where it stood, `detail` how it was looked at,
    /// `looked=<ticks> looks=<n> longest=<ticks> toward=<blocks> end=<how>` (a look is facing it
    /// within ten degrees for half a second; `toward` the distance they closed; `end` one of
    /// `approached`, `looked_away`, `timed`, `walked`, `gone`). ABI 47.
    FigureSeen,
    /// One of its sales bought: `pos` the villager, `detail` the item, the buyer the nearest
    /// player. ABI 51.
    Bought,
    ChangedDimension,
    /// Hit an entity; `detail` is its kind, `pos` where it stood.
    Hit,
    /// Killed an entity; `detail` is its kind, `pos` where it died.
    Killed,
}

/// A chat line a player typed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatLine {
    pub player: u32,
    pub text: String,
}

/// The answer to a `Command::Scan`: the blocks of the kinds asked for in a box.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scan {
    /// The command's id.
    pub id: u64,
    /// Counts by block id.
    pub counts: Vec<(String, u32)>,
    /// Positions of the kinds asked for by position.
    pub positions: Vec<(String, BlockPos)>,
}

/// The answer to `Command::Surroundings`: points of interest around a point, as features.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Surroundings {
    /// The command's id.
    pub id: u64,
    pub around: BlockPos,
    pub features: Vec<Feature>,
}

/// One thing found around: `water` with its extent, `village` with its center, `field`,
/// `pen` with its animals in `detail`, `tunnel` with its length, `cave`, `snow`, `sand`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Feature {
    pub kind: String,
    pub pos: BlockPos,
    /// Blocks across, or long.
    pub size: u32,
    pub detail: String,
}

/// Another mod loaded beside this one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadedModInfo {
    pub id: String,
    pub version: String,
}

/// What a mod wants done. Each carries an `id` the report names. The host checks each command's
/// preconditions as it runs it and reports `done` or why not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Command {
    /// A line in the host's log, prefixed with the mod's id.
    Log {
        id: u64,
        text: String,
    },
    /// Count and locate blocks of `kinds` in the box, answered as a `Scan` next tick. A kind
    /// starting with `*` is a suffix: `*_stairs` is every stair.
    Scan {
        id: u64,
        min: BlockPos,
        max: BlockPos,
        kinds: Vec<String>,
        /// Read every `step`th block on each axis from `min` (0 or 1 is every block), so a
        /// wide box stays under the host's limit, which counts the blocks read. ABI 23.
        #[serde(default, skip_serializing_if = "is_zero")]
        step: u8,
        /// Only blocks with no sky light and no block light: the dark inside of caves, which
        /// since 1.18 are mostly plain air. ABI 23.
        #[serde(default, skip_serializing_if = "is_false")]
        dark: bool,
        /// At most this many positions answered (0 for all). ABI 23.
        #[serde(default, skip_serializing_if = "is_zero_u32")]
        most: u32,
    },
    /// A sound to one player at a position, by id, once.
    PlaySound {
        id: u64,
        player: u32,
        sound: String,
        pos: Pos,
        volume: f32,
        pitch: f32,
    },
    /// The sound a block makes, to one player at a position: `of` is `step`, `place`,
    /// `break` or `hit`, and `block` a block id, or `under` (the block under the player's
    /// feet) or `held` (the block in their hand, stone if none), so footsteps sound like what
    /// they walk on. ABI 12.
    BlockSound {
        id: u64,
        player: u32,
        block: String,
        of: String,
        pos: Pos,
        volume: f32,
        pitch: f32,
    },
    /// A client-side hint to one player: a kind the host's client knows, a place, a duration.
    Hint {
        id: u64,
        player: u32,
        hint: String,
        pos: Option<BlockPos>,
        seconds: f32,
        detail: String,
    },
    /// A system message to one player, as the server would send it.
    SystemMessage {
        id: u64,
        player: u32,
        text: String,
    },
    /// Open or close a door, trapdoor or gate, flip a lever, press a button.
    ToggleBlock {
        id: u64,
        pos: BlockPos,
        /// Open or lit (`true`), shut or out (`false`), left as it is when it is so already;
        /// absent, the other way from now. ABI 50.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<bool>,
    },
    /// Move one block to another place within reach, as the lodger moves a torch.
    MoveBlock {
        id: u64,
        from: BlockPos,
        to: BlockPos,
        /// Look this far around `to` (its level, one up, one down) for a spot with room and
        /// something solid under it, when `to` itself will not do. A single-block container
        /// (a shulker box, a barrel; never a chest) is moved with what it holds. ABI 15.
        #[serde(default, skip_serializing_if = "is_zero")]
        around: u8,
    },
    /// Set a block, with an optional state the host applies (`facing=north,half=top`). Only where there is nothing,
    /// unless the block there is one of `replace` (block ids): the way a path is worn into grass
    /// or a hole dug in natural ground, and never through anything a player placed, which the
    /// mod keeps out of `replace` by never naming built blocks.
    ///
    /// Never where the block cannot stand: a torch over air is refused, as a player could not
    /// place it. `settle` lets the host look up to that many blocks down and up for a spot where
    /// it can (the nearest first, the ground before a wall); `wall` names the block to put
    /// against a wall when the block itself stands nowhere near (`minecraft:wall_torch` for a
    /// torch), facing out from whichever side holds it. The mod knows positions, not terrain;
    /// the host knows the terrain.
    SetBlock {
        id: u64,
        pos: BlockPos,
        block: String,
        state: String,
        #[serde(default)]
        replace: Vec<String>,
        #[serde(default, skip_serializing_if = "is_zero")]
        settle: u8,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        wall: String,
        /// Only on something solid: a cobblestone of a cairn sits on the ground or on the
        /// block below it, never in the air (a torch or a sign needs no telling). ABI 10.
        #[serde(default, skip_serializing_if = "is_false")]
        rest: bool,
        /// Only over one of `replace`, never into air: ice goes on water, not beside it. ABI 10.
        #[serde(default, skip_serializing_if = "is_false")]
        only: bool,
    },
    /// Put another block in a block's place, keeping every property the two share (a wall
    /// torch's facing, a lantern's hanging), and only over one of `only`: a torch's flame turned
    /// to soul fire where it hangs. ABI 24.
    SwapBlock {
        id: u64,
        pos: BlockPos,
        block: String,
        only: Vec<String>,
    },
    /// Take a block away for `seconds` and put it back as it was, only one of `only` (a light):
    /// the host puts it back at the time, or at once when its level saves or closes, so a quit
    /// never leaves it gone. Nothing is put back over what took the empty place meanwhile.
    /// ABI 24.
    HideBlock {
        id: u64,
        pos: BlockPos,
        seconds: f32,
        only: Vec<String>,
    },
    /// Move a bed one block to its side, both halves, colour and facing kept, onto whichever side
    /// has room and solid floor under both; never one someone lies in. ABI 26.
    ShiftBed {
        id: u64,
        pos: BlockPos,
    },
    /// Put a bed of `block` beside the bed at `pos`, the same facing, on whichever side has room
    /// and solid floor under both halves. ABI 26.
    BedBeside {
        id: u64,
        pos: BlockPos,
        block: String,
    },
    /// Place an entity of kind `of` at `pos` facing `yaw`: an armour stand, or a farm animal. With
    /// `seconds` above 0 it is there for that long only, then gone, and gone at once when its
    /// level saves or closes, or when next seen after a quit. ABI 27.
    PlaceEntity {
        id: u64,
        of: String,
        pos: Pos,
        yaw: f32,
        #[serde(default, skip_serializing_if = "is_zero_f32")]
        seconds: f32,
        /// Put on it the stack kept under this name (`take_item`'s `keep`): the scarecrow's
        /// piece of their armour. ABI 27.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        kept: String,
        /// Hold it still facing this player for its time: the shape under the water. ABI 30.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        face: Option<u32>,
    },
    /// Footprints a player's size in the snow from `from` to `to`: a snow layer taken off at
    /// each step, left and right of the line, where there is snow; refused under three. ABI 30.
    Prints {
        id: u64,
        from: BlockPos,
        to: BlockPos,
    },
    /// The bees of the hive at `hive` out, hanging still around `to`, for `seconds`; then home.
    /// ABI 27.
    ReleaseBees {
        id: u64,
        hive: BlockPos,
        to: Pos,
        seconds: f32,
    },
    /// The lead of an animal tied to a fence post tied to another post within eight blocks of
    /// the first instead. ABI 27.
    MoveLeash {
        id: u64,
        entity: u32,
    },
    /// An animal turns on `player` for `seconds`: it runs at them and hits for `damage` (half
    /// hearts) a second while it touches them, `hits` times at most; with `spare` never below
    /// one half heart. Then it is as it was. ABI 27.
    Charge {
        id: u64,
        entity: u32,
        player: u32,
        damage: f32,
        hits: u32,
        seconds: f32,
        #[serde(default, skip_serializing_if = "is_false")]
        spare: bool,
    },
    /// A jukebox: an empty one plays the disc from `from`'s `slot`; one playing stops, its
    /// disc into the first empty slot of `from`. ABI 29.
    Jukebox {
        id: u64,
        pos: BlockPos,
        from: BlockPos,
        slot: u32,
    },
    /// The book on a lectern open to another page. ABI 29.
    TurnPage {
        id: u64,
        pos: BlockPos,
    },
    /// One ingredient from `from`'s `slot` into a brewing stand that holds potions and an empty
    /// ingredient slot, and a blaze powder from `from` when the stand has no fuel: the game
    /// brews it. ABI 29.
    Brew {
        id: u64,
        stand: BlockPos,
        from: BlockPos,
        slot: u32,
    },
    /// One enchantment gone from an enchanted item in a container, as a grindstone would take
    /// it; the host picks the item and the enchantment. ABI 29.
    Strip {
        id: u64,
        from: BlockPos,
    },
    /// Every villager within 48 blocks of `center` walked into the house whose door is `door`,
    /// packed inside, held still facing its back wall, turning to `player` when they come
    /// within five blocks; after `seconds` (ten minutes at most) each is back where it stood.
    /// ABI 31.
    GatherVillagers {
        id: u64,
        center: BlockPos,
        door: BlockPos,
        seconds: f32,
        player: u32,
    },
    /// A villager offers the stack kept under `kept` (`take_item`'s `keep`) for one emerald,
    /// once. ABI 31.
    OfferTrade {
        id: u64,
        entity: u32,
        kept: String,
    },
    /// Light a Nether portal in an empty obsidian frame within six blocks of `near`, as flint
    /// and steel would. ABI 32.
    LightPortal {
        id: u64,
        near: BlockPos,
    },
    /// The figure `figure` shown to `player` swings its arm once; with `damage` above 0 the
    /// player is hit for it (half hearts), a hurt's jolt and sound, and with `spare` never
    /// below half a heart. ABI 33.
    FigureSwing {
        id: u64,
        player: u32,
        figure: u32,
        damage: f32,
        #[serde(default, skip_serializing_if = "is_false")]
        spare: bool,
    },
    /// `player`, asleep, is kept lying in bed, the night not passing meanwhile, while the figure
    /// `figure` walks slowly (a glide, its legs moving) from where it stands (seven blocks off at
    /// most) to their bed's side by the pillow, the side nearer it with room to stand; there it
    /// takes `item` into its hand, a lead's rope drawn from it to their neck, and for `seconds`
    /// (six at most) squeezes once a second: its arm moves, and `damage` (half hearts) is taken
    /// without waking them, with a drowning gasp, the hurt's jolt and the dark closing in; with
    /// `spare` never below half a heart. Leaving the bed ends it at once, the figure gone; at its
    /// end they wake. `door`, when it is still a door, opens as it begins, with its sound, and is
    /// left open (anything else there is left alone). With `how` `shoot` it takes out a crossbow
    /// (`item`) instead, loads it looking down at it, aims it at them, and shoots once: `damage`,
    /// which wakes them, and they burn a few seconds, `spare` keeping them above half a heart
    /// through the fire too; it watches them `seconds` and is gone. Refused when they are not
    /// asleep. ABI 54.
    Choke {
        id: u64,
        player: u32,
        figure: u32,
        item: String,
        seconds: f32,
        damage: f32,
        #[serde(default, skip_serializing_if = "is_false")]
        spare: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        door: Option<BlockPos>,
        /// `choke` (empty), `shoot`, or `lie`: it lies down on the floor beside the bed facing
        /// them, and does nothing more until they wake as it ends.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        how: String,
    },
    /// For `seconds` (a minute at most), `player`'s air runs out twice as fast under water; the
    /// drowning itself is the game's. ABI 54.
    DrainAir {
        id: u64,
        player: u32,
        seconds: f32,
    },
    /// The mob `entity` wears `player`'s own head for `seconds` (ten minutes at most), never
    /// dropping it; then what it wore before. ABI 54.
    WearFace {
        id: u64,
        entity: u32,
        player: u32,
        seconds: f32,
    },
    /// For `seconds` (half an hour at most), the next eye of ender `player` throws flies toward
    /// `to` instead of the stronghold, once. ABI 54.
    EyeHome {
        id: u64,
        player: u32,
        to: Pos,
        seconds: f32,
    },
    /// The stack kept under `kept` (`write_book`, `take_item`'s `keep`) into `player`'s own
    /// ender chest, the inventory every ender chest of theirs opens on; kept again when it is
    /// full. ABI 54.
    PutEnderChest {
        id: u64,
        player: u32,
        kept: String,
    },
    /// The bed at `pos` (either half) made `player`'s respawn point, quietly: no message. Their
    /// next death wakes them there. Refused when it is no bed. ABI 54.
    SetRespawn {
        id: u64,
        player: u32,
        pos: BlockPos,
    },
    /// Push `player` by `velocity`, blocks a tick, as a hit or a current would (each axis at
    /// most one), once. ABI 54.
    PushPlayer {
        id: u64,
        player: u32,
        velocity: Pos,
    },
    /// The door at `pos` (either half) will not open for `seconds` (ten at most), as if held
    /// from the other side: a player's use of it is refused, the door rattling shut on their
    /// screen, and each try heard as a blow on it. ABI 54.
    HoldDoor {
        id: u64,
        pos: BlockPos,
        seconds: f32,
    },
    /// Lay blocks relative to `at`: `parts`' each `at` an offset, `y` 0 the ground's own
    /// block. With `at`'s y below -1000 the host finds the ground there (the highest block
    /// that stops movement, leaves left out, water counting) and says it in its report
    /// (`base 64 placed 12`). Each block goes only into air or over one of `replace`, never
    /// over anything with contents or a player's building; a door's halves go in together.
    /// ABI 34.
    PlaceBlueprint {
        id: u64,
        at: BlockPos,
        parts: Vec<BlueprintPart>,
        #[serde(default)]
        replace: Vec<String>,
    },
    /// For `seconds`, a second set of steps follows `player`: each step they take on the
    /// ground is heard again `delay` ticks later, `behind` blocks back along their way, in the
    /// sound of the block there. They stop when the player stops; crouching makes none. ABI 36.
    Footsteps {
        id: u64,
        player: u32,
        seconds: f32,
        behind: f32,
        delay: u32,
    },
    /// The confrontation (the fourth ending): a fight with `player` at `at` (its door, or its
    /// place), in three phases under one boss bar in the player's name. Waves of warped animals
    /// while a mannequin in their skin watches from `roof` (none on peaceful); then the duel,
    /// the mannequin in a copy of their best gear, splitting into copies at less than three
    /// fifths of its health; then a construct built of `blocks`, dropping them as it breaks. Its
    /// health and hits are scaled to their armour and the difficulty. Won, they get its
    /// lantern, its head and its weapon, and what it held (`kept` "held") back; lost, they die
    /// once keeping everything. Answered by a `fight_won`, `fight_lost` or `fight_fled` event.
    /// ABI 43.
    Confront {
        id: u64,
        player: u32,
        at: BlockPos,
        roof: Pos,
        blocks: Vec<String>,
        /// How much fiercer, 0 to 1: its temper over its resting value. Its health up to a
        /// quarter more, a fifth wave from half on. ABI 48.
        #[serde(default, skip_serializing_if = "is_zero_f32")]
        fierce: f32,
    },
    /// `player` wakes in a copy of their house (around `center`, `radius` blocks out, its lights
    /// left out) in the content jar's void dimension `lodger:other`, in the copy's `bed`, the
    /// figure at the copy's `door` (walking to a block from them with `comes`); after `seconds`
    /// (three minutes at most) back in their own bed, `door` open. A barrier floor runs on
    /// outward, so no one falls. ABI 49.
    OtherHouse {
        id: u64,
        player: u32,
        center: BlockPos,
        radius: u32,
        bed: BlockPos,
        door: BlockPos,
        seconds: f32,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        comes: bool,
    },
    /// A real arrow from `from` (on the ground there when its y is below -1000) at `at`, aimed
    /// as a skeleton aims; it harms `damage` if it hits `player` (0 for none, and never below half
    /// a heart), cannot be picked up, and stays where it lands. ABI 42.
    Shoot {
        id: u64,
        player: u32,
        from: Pos,
        at: Pos,
        damage: f32,
    },
    /// A splash potion of one effect thrown from `from` (the ground there below -1000) at
    /// `player`'s feet: `darkness`, `slowness`, `blindness`, `nausea` or `levitation` alone,
    /// for `seconds` (levitation at most two); the host refuses any other. ABI 42.
    SplashPotion {
        id: u64,
        player: u32,
        from: Pos,
        effect: String,
        seconds: f32,
    },
    /// A potion of its own brewing put into a container, named, with these effects (`effect`,
    /// seconds): none that harms (the host refuses instant damage, poison, wither). ABI 42.
    PutPotion {
        id: u64,
        into: BlockPos,
        name: String,
        effects: Vec<(String, f32)>,
    },
    /// The potion in a container's slot becomes another kind, its name as it was: never one
    /// that harms (the host keeps to night vision, invisibility, water breathing, fire
    /// resistance, leaping, swiftness, slow falling). ABI 42.
    ChangePotion {
        id: u64,
        pos: BlockPos,
        slot: u32,
        into: String,
    },
    /// A lingering cloud of one effect at `pos`, `radius` wide, gone as soon as it touches
    /// anyone, or after `seconds`: slowness or weakness alone. ABI 42.
    EffectCloud {
        id: u64,
        pos: Pos,
        effect: String,
        seconds: f32,
        radius: f32,
    },
    /// Every stack in the container at `from` dropped at `to`, as items that do not despawn;
    /// with `remove` the container then taken away, empty, as nothing. ABI 41.
    SpillContainer {
        id: u64,
        from: BlockPos,
        to: Pos,
        #[serde(default, skip_serializing_if = "is_false")]
        remove: bool,
    },
    /// The player dies, once, as if fallen out of the world, keeping everything they carry and
    /// their experience: what they carry is held by the host and given back as they respawn,
    /// kept in the world's data meanwhile. Never in creative or spectator. ABI 41.
    KillPlayer {
        id: u64,
        player: u32,
    },
    /// The enderman `entity` holds `block`, as if it had picked it up: it may set it down
    /// again as endermen do. ABI 39.
    CarryBlock {
        id: u64,
        entity: u32,
        block: String,
    },
    /// For `seconds`, the item frames `frames` show `item` to `player` alone, whatever they
    /// hold; then what they hold again. Nothing in them changes. ABI 38.
    FramesShow {
        id: u64,
        player: u32,
        frames: Vec<u32>,
        item: String,
        seconds: f32,
    },
    /// For `seconds`, the first time `player`'s bobber sits in the water it is pulled under on
    /// their screen and kept under for `under` seconds, with no fish on it: reeled in, it
    /// brings nothing. ABI 38.
    Bobber {
        id: u64,
        player: u32,
        seconds: f32,
        under: f32,
    },
    /// The figure `figure`, seen by `player` alone, walks from `from` (on the ground) along
    /// `way` to the first rock face and digs in, a step at a time, the stone closing behind it,
    /// until it is gone; with `through`, a minute later it steps out of the hill's far face. All
    /// of it is block updates to that player's client: nothing in the world changes. ABI 35.
    DigIn {
        id: u64,
        player: u32,
        figure: u32,
        from: BlockPos,
        way: [i32; 2],
        #[serde(default, skip_serializing_if = "is_false")]
        through: bool,
    },
    /// Open a chest's lid, with its sound, and close it after `seconds`. Nothing inside changes.
    OpenLid {
        id: u64,
        pos: BlockPos,
        seconds: f32,
    },
    /// Turn an item frame's item: to `rotation` (0 to 7), or one notch on when absent.
    RotateFrame {
        id: u64,
        entity: u32,
        rotation: Option<u32>,
    },
    /// Hang another painting of the same size in a painting's place.
    SwapPainting {
        id: u64,
        entity: u32,
    },
    /// Turn an armour stand's head: toward `at` once, or to follow player `follow` for
    /// `seconds`, holding still while that player looks straight at it.
    PoseStand {
        id: u64,
        entity: u32,
        at: Option<Pos>,
        follow: Option<u32>,
        seconds: f32,
    },
    /// Name an entity, as a name tag would.
    NameEntity {
        id: u64,
        entity: u32,
        name: String,
    },
    /// Dye a sheep, by colour name (`red`, `light_blue`).
    DyeSheep {
        id: u64,
        entity: u32,
        colour: String,
    },
    /// Move an entity to a position, as if it had walked there.
    MoveEntity {
        id: u64,
        entity: u32,
        to: Pos,
    },
    /// Give an entity a velocity, blocks a tick: a minecart rolls, a boat drifts.
    PushEntity {
        id: u64,
        entity: u32,
        velocity: Pos,
    },
    /// Turn a block a quarter turn clockwise where its state allows: stairs, a banner, an anvil.
    RotateBlock {
        id: u64,
        pos: BlockPos,
    },
    /// Break a block as a player would, dropping its item, so nothing is lost: only beds,
    /// torches, carpets, panes, flowers and signs; the host refuses anything else.
    BreakBlock {
        id: u64,
        pos: BlockPos,
    },
    /// Put an item in an item frame, or empty it with an empty id.
    SetFrameItem {
        id: u64,
        entity: u32,
        item: ItemStack,
        /// Move the stack in this container's `slot` into the frame instead, the very stack,
        /// only into an empty frame: nothing is lost if the frame is full. ABI 28.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<BlockPos>,
        #[serde(default, skip_serializing_if = "is_zero_u32")]
        slot: u32,
        /// With `from`: the frame's own item goes into that slot as the slot's goes into the
        /// frame, the very stacks traded. ABI 40.
        #[serde(default, skip_serializing_if = "is_false")]
        swap: bool,
        /// Where what the frame held lands when it is emptied or given another item: there,
        /// or at the frame when absent; never lost. ABI 40.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        drop_at: Option<Pos>,
    },
    /// Put a piece of armour or a held item on an entity that wears it: an armour stand.
    EquipEntity {
        id: u64,
        entity: u32,
        item: ItemStack,
        /// Put on the very stack kept under this name (`take_item`'s `keep`), enchantments
        /// and all, in place of a plain one of `item`. ABI 27.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        kept: String,
    },
    /// Wear down a damageable item in a container's slot by `amount` uses.
    DamageItem {
        id: u64,
        pos: BlockPos,
        slot: u32,
        amount: u32,
    },
    /// A second entry with the player's own name in that player's player list, for `seconds`,
    /// seen by that player alone.
    PlayerList {
        id: u64,
        player: u32,
        seconds: f32,
    },
    /// A line in one player's chat or action bar in the game's own words: `joined` and `left`
    /// (`<name> joined the game`), `says` (`<name> text`, as if they typed it), `whisper`
    /// (`<name> whispers to you: text`), `sleepers` (`1/2 players sleeping` on the action bar).
    GameMessage {
        id: u64,
        player: u32,
        message: String,
        text: String,
    },
    /// Move items between two containers, or within one.
    MoveItem {
        id: u64,
        from: BlockPos,
        from_slot: u32,
        to: BlockPos,
        to_slot: u32,
        count: u32,
    },
    /// Take `count` of a container's slot away. The mod keeps what it took in its own memory;
    /// nothing in the world holds it until a `DropItem` or `MoveItem` puts it somewhere.
    TakeItem {
        id: u64,
        from: BlockPos,
        slot: u32,
        count: u32,
        /// Keep the very stack taken under this name, saved with the world, for `put_item` or
        /// `drop_item` to give back: enchantments, wear and name and all. ABI 13.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        keep: String,
        /// The item it means to take, by id: a slot holding something else now (the chest
        /// changed since it was seen) is not taken from; that item in another slot is, and with
        /// none the take is refused. Empty: whatever is there. ABI 52.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        expect: String,
    },
    /// Put an item into a container: the slot given, or the first empty one when `slot` is
    /// absent. The mod gives back what it took this way.
    PutItem {
        id: u64,
        into: BlockPos,
        slot: Option<u32>,
        item: ItemStack,
        /// Give back the stack kept under this name by `take_item`'s `keep`, components and
        /// all, instead of making `item`. ABI 13.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        kept: String,
        /// As it goes back: this much more wear on a tool, and this name. ABI 13.
        #[serde(default, skip_serializing_if = "is_zero_u32")]
        wear: u32,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
    },
    /// Drop an item at a position, as an item entity that does not despawn.
    DropItem {
        id: u64,
        pos: Pos,
        item: ItemStack,
        /// As `put_item`'s. ABI 13.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        kept: String,
    },
    /// Write a sign's lines.
    SetSign {
        id: u64,
        pos: BlockPos,
        lines: Vec<String>,
        /// Look this many blocks down and up the column for the sign: a sign placed with
        /// `set_block`'s `settle` may have landed off its spot. ABI 10.
        #[serde(default, skip_serializing_if = "is_zero")]
        settle: u8,
        /// Put the first of `lines` in place of the last line that has text, keeping the
        /// rest; refused on a sign with no text (`sign_word`). ABI 28.
        #[serde(default, skip_serializing_if = "is_false")]
        one_line: bool,
    },
    /// Cancel one player's sleep with the vanilla message, or refuse their next attempt within
    /// `seconds` with it.
    CancelSleep {
        id: u64,
        player: u32,
        seconds: f32,
    },
    /// Arm an offering for the player's next chest of fresh loot: by `chance`, the item (or
    /// the stack kept under `kept`; `@head` is the player's own head) joins the loot as it is
    /// made, swapping one loot item out by `replace`'s chance (kept as `loot_taken`). Answered
    /// by a `loot_touched` event. ABI 16.
    ArmLoot {
        id: u64,
        player: u32,
        chance: f32,
        replace: f32,
        item: ItemStack,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        kept: String,
    },
    /// Bone meal the block at `pos`, as a player would, up to `tries` times, until it is
    /// something else (a sapling a tree). Not done if it did not grow. ABI 18.
    Grow {
        id: u64,
        pos: BlockPos,
        tries: u32,
    },
    /// Change one property of the block at `pos` (`hinge` to `right`, or `@next` for the next
    /// value); a door's other half changes with it. ABI 17. `@turn` on a bed turns it round
    /// where it stands, both halves at once. ABI 19. `@up` one value higher, refused at the
    /// top: a composter one level fuller. ABI 29.
    SetState {
        id: u64,
        pos: BlockPos,
        property: String,
        value: String,
    },
    /// Move a player, gently: to `to`, or the first spot up to two blocks above it with room
    /// to stand, never more than 32 blocks from where they are. ABI 9.
    MovePlayer {
        id: u64,
        player: u32,
        to: Pos,
    },
    /// Sign the book and quill in a container's slot as a written book with these pages, the
    /// title given and no author, as a player signing it would. ABI 14.
    SignBook {
        id: u64,
        pos: BlockPos,
        slot: u32,
        title: String,
        pages: Vec<String>,
    },
    /// Make a written book, `title` by `author` with these pages, and keep it under `kept` for a
    /// sale (`offer_trade`), a chest (`put_item`), a lectern (`put_book`) or a villager's hands
    /// (`approach` with `give`). ABI 51.
    WriteBook {
        id: u64,
        kept: String,
        title: String,
        author: String,
        pages: Vec<String>,
    },
    /// One page more in the book and quill in a container's slot, after its own. ABI 51.
    AddPage {
        id: u64,
        pos: BlockPos,
        slot: u32,
        page: String,
    },
    /// The book kept under `kept` onto the lectern at `pos`, or, holding a book, the first of
    /// `also` that holds none (the mind's other lecterns out of their view, ABI 53). ABI 51.
    PutBook {
        id: u64,
        pos: BlockPos,
        kept: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        also: Vec<BlockPos>,
    },
    /// Mark the filled map in a container's slot with a target at `at`. ABI 14.
    MarkMap {
        id: u64,
        pos: BlockPos,
        slot: u32,
        at: BlockPos,
    },
    /// Name the item in a container's slot, as an anvil would. ABI 13.
    RenameItem {
        id: u64,
        pos: BlockPos,
        slot: u32,
        name: String,
    },
    /// Hold a mob where it stands, facing `player`, moving and attacking no one, for `seconds`
    /// (at most ten minutes); then it is as it was. A mob saved while held is let go when next
    /// seen, so none is ever left frozen in a world. ABI 8.
    Hold {
        id: u64,
        entity: u32,
        player: u32,
        seconds: f32,
        /// Face this point instead of the player: the house. ABI 27.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<Pos>,
        /// A step closer along the ground each moment the player looks away, to six blocks,
        /// and back where it was held the moment they look at it; back there when let go. ABI 37.
        #[serde(default, skip_serializing_if = "is_false")]
        creep: bool,
    },
    /// The mobs `entities` walk toward `player` together, as they walk, and stop `stop` blocks
    /// off, facing them, until `seconds` have passed; then they are as they were. Their AI is
    /// never turned off. ABI 37.
    Approach {
        id: u64,
        entities: Vec<u32>,
        player: u32,
        stop: f32,
        seconds: f32,
        /// How fast, against its own walk (1 when absent; 0.35 a slow, slow walk). ABI 44.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pace: Option<f32>,
        /// Once there: `stare` a few seconds and go its way, `leave` (turn and walk off), `hit`
        /// (strike once for `damage`, never below half a heart, and walk off), or stand facing
        /// them till `seconds` when empty. ABI 44.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        then: String,
        #[serde(default, skip_serializing_if = "is_zero_f32")]
        damage: f32,
        /// With `then` `give`: what is kept under this name, brought and dropped at their feet as
        /// it gets there, the first of `entities` alone; kept again if it never gets there.
        /// ABI 51.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        kept: String,
        /// Walk to this place instead (the ground there when its y is below -1000), and once
        /// there (or after twenty seconds) be as it was: a pet goes back to its owner of itself.
        /// ABI 54.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<Pos>,
    },
    /// For `seconds`, the first moment `player` stands at an edge (a drop of two to `most_drop`
    /// blocks to solid ground just past it) or on a ladder with ground that near below, not
    /// crouching and with more than two hearts, they are pushed off it, once, with a step
    /// behind them. Never a fall of more than `most_drop`, never fatal. ABI 37.
    Shove {
        id: u64,
        player: u32,
        seconds: f32,
        most_drop: u32,
    },
    /// Warp an animal for `seconds` (at most two minutes), then put it back as it was, where
    /// it was: frozen, its head always on `player`; `scale` times its size (1 for none);
    /// `flip` upside down as the game turns a mob named Dinnerbone (never one with a name of
    /// its own); `creep` a step closer each time the player looks away, along the ground. Moved
    /// first to `to` when given. It ends early when the player comes within six blocks. A mob
    /// saved while warped is put back when next seen. ABI 25.
    WarpMob {
        id: u64,
        entity: u32,
        player: u32,
        seconds: f32,
        #[serde(default = "one")]
        scale: f32,
        #[serde(default, skip_serializing_if = "is_false")]
        flip: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        creep: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<Pos>,
    },
    /// Turn an entity's head toward a position and keep it there for `seconds`.
    LookAt {
        id: u64,
        entity: u32,
        at: Pos,
        seconds: f32,
    },
    /// Look around `around` within `radius` blocks for the features of `Surroundings`, answered
    /// as `Observations.surroundings` over the next seconds, in slices, so it never shows in a
    /// tick profile.
    Surroundings {
        id: u64,
        around: BlockPos,
        radius: u32,
    },
    /// Spawn or move a figure wearing a player's skin, standing or walking a path, seen by
    /// that player alone (the host sends it as packets; no entity exists on the server). A `pos`
    /// with a y below -1000 stands on the ground there. It faces the player. Gone after
    /// `seconds` (0: until removed) or when the player comes within `vanish_within` blocks (0:
    /// never).
    Figure {
        id: u64,
        figure: u32,
        skin_of: u32,
        pos: Pos,
        path: Vec<Pos>,
        held: ItemStack,
        name_shown: bool,
        #[serde(default)]
        seconds: f32,
        #[serde(default)]
        vanish_within: f32,
        /// Mirror the player: it moves as they move, by the same steps, from where it was
        /// shown. ABI 33.
        #[serde(default, skip_serializing_if = "is_false")]
        mirror: bool,
        /// Gone the moment it leaves the player's view, once they have seen it. ABI 33.
        #[serde(default, skip_serializing_if = "is_false")]
        once_seen: bool,
        /// Crouching when the player crouches, standing when they stand. ABI 35.
        #[serde(default, skip_serializing_if = "is_false")]
        copy_crouch: bool,
        /// Swinging what it holds now and then, as at the rock. ABI 35.
        #[serde(default, skip_serializing_if = "is_false")]
        swinging: bool,
        /// Hanging feet up from the ceiling over `pos` (the first solid block above it), as the
        /// game draws a player named Dinnerbone. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        upside_down: bool,
        /// Crawling on its belly along `path`, gliding (eased by their client), slower than a
        /// walk, and gone at its end. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        crawls: bool,
        /// Running at the player flat out, after them as they move, and gone two blocks short
        /// with a hit's jolt and sound, no harm. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        rushes: bool,
        /// Sitting in the seat behind the player in the boat they ride, for their client alone;
        /// gone when they leave the boat. Refused when they ride none with a seat to spare. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        rides_behind: bool,
        /// Figures shown with the same number here (above 0) go together: when one is gone,
        /// seen and looked away from or come near or its time up, so are the rest. ABI 54.
        #[serde(default, skip_serializing_if = "is_zero_u32")]
        together: u32,
        /// Lying on the ground as a body lies, on its side. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        lies: bool,
        /// Falling from forty blocks over its place to it, fast, landing with a heavy fall's
        /// sound, and gone there. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        falls: bool,
        /// Walking `path` at a walk, gliding (eased by their client, its legs moving), facing the
        /// way it goes and never them, each point on the ground when its y is below -1000; gone
        /// at its end. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        walks: bool,
        /// Facing this place instead of the player. ABI 54.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        faces: Option<Pos>,
        /// Standing a block and a half into the ground, its head alone above it; once they have
        /// looked at it, sinking slowly out of sight and gone. ABI 54.
        #[serde(default, skip_serializing_if = "is_false")]
        sunk: bool,
    },
    RemoveFigure {
        id: u64,
        figure: u32,
    },
}

impl Command {
    pub fn id(&self) -> u64 {
        match self {
            Command::Log { id, .. }
            | Command::Scan { id, .. }
            | Command::PlaySound { id, .. }
            | Command::Hint { id, .. }
            | Command::SystemMessage { id, .. }
            | Command::ToggleBlock { id, .. }
            | Command::MoveBlock { id, .. }
            | Command::SetBlock { id, .. }
            | Command::SwapBlock { id, .. }
            | Command::HideBlock { id, .. }
            | Command::ShiftBed { id, .. }
            | Command::Jukebox { id, .. }
            | Command::TurnPage { id, .. }
            | Command::Brew { id, .. }
            | Command::Strip { id, .. }
            | Command::Prints { id, .. }
            | Command::GatherVillagers { id, .. }
            | Command::OfferTrade { id, .. }
            | Command::LightPortal { id, .. }
            | Command::FigureSwing { id, .. }
            | Command::Choke { id, .. }
            | Command::HoldDoor { id, .. }
            | Command::PushPlayer { id, .. }
            | Command::SetRespawn { id, .. }
            | Command::PutEnderChest { id, .. }
            | Command::EyeHome { id, .. }
            | Command::WearFace { id, .. }
            | Command::DrainAir { id, .. }
            | Command::PlaceBlueprint { id, .. }
            | Command::DigIn { id, .. }
            | Command::Footsteps { id, .. }
            | Command::Approach { id, .. }
            | Command::Shove { id, .. }
            | Command::FramesShow { id, .. }
            | Command::CarryBlock { id, .. }
            | Command::SpillContainer { id, .. }
            | Command::Shoot { id, .. }
            | Command::Confront { id, .. }
            | Command::OtherHouse { id, .. }
            | Command::WriteBook { id, .. }
            | Command::AddPage { id, .. }
            | Command::PutBook { id, .. }
            | Command::SplashPotion { id, .. }
            | Command::PutPotion { id, .. }
            | Command::ChangePotion { id, .. }
            | Command::EffectCloud { id, .. }
            | Command::KillPlayer { id, .. }
            | Command::Bobber { id, .. }
            | Command::PlaceEntity { id, .. }
            | Command::ReleaseBees { id, .. }
            | Command::MoveLeash { id, .. }
            | Command::Charge { id, .. }
            | Command::BedBeside { id, .. }
            | Command::OpenLid { id, .. }
            | Command::RotateFrame { id, .. }
            | Command::SwapPainting { id, .. }
            | Command::PoseStand { id, .. }
            | Command::NameEntity { id, .. }
            | Command::DyeSheep { id, .. }
            | Command::MoveEntity { id, .. }
            | Command::WarpMob { id, .. }
            | Command::PushEntity { id, .. }
            | Command::RotateBlock { id, .. }
            | Command::BreakBlock { id, .. }
            | Command::SetFrameItem { id, .. }
            | Command::EquipEntity { id, .. }
            | Command::DamageItem { id, .. }
            | Command::PlayerList { id, .. }
            | Command::GameMessage { id, .. }
            | Command::MoveItem { id, .. }
            | Command::TakeItem { id, .. }
            | Command::PutItem { id, .. }
            | Command::DropItem { id, .. }
            | Command::SetSign { id, .. }
            | Command::CancelSleep { id, .. }
            | Command::LookAt { id, .. }
            | Command::Hold { id, .. }
            | Command::MovePlayer { id, .. }
            | Command::ArmLoot { id, .. }
            | Command::SetState { id, .. }
            | Command::Grow { id, .. }
            | Command::BlockSound { id, .. }
            | Command::RenameItem { id, .. }
            | Command::SignBook { id, .. }
            | Command::MarkMap { id, .. }
            | Command::Surroundings { id, .. }
            | Command::Figure { id, .. }
            | Command::RemoveFigure { id, .. } => *id,
        }
    }
}

/// One block of a `place_blueprint`. ABI 34.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BlueprintPart {
    pub at: BlockPos,
    pub block: String,
    pub state: String,
}

/// How a command went.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    pub id: u64,
    pub done: bool,
    /// Why not, when not.
    pub reason: String,
}

fn one() -> f32 {
    1.0
}

fn is_zero_f32(n: &f32) -> bool {
    *n == 0.0
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quiet_tick_is_a_small_message() {
        let quiet = Observations {
            players: vec![Player {
                index: 0,
                name: "razor".to_string(),
                pos: [1.0, 64.0, 2.0],
                ..Player::default()
            }],
            ..Observations::default()
        };
        let text = serde_json::to_string(&quiet).unwrap();

        assert!(text.len() < 1200, "{} bytes: {text}", text.len());
        let back: Observations = serde_json::from_str(&text).unwrap();
        assert_eq!(back, quiet);
    }

    #[test]
    fn an_older_message_without_new_fields_still_reads() {
        let text = r#"{ "players": [ { "index": 3, "name": "a" } ] }"#;
        let obs: Observations = serde_json::from_str(text).unwrap();

        assert_eq!(obs.players[0].index, 3);
        assert_eq!(obs.players[0].movement, Movement::default());
    }

    #[test]
    fn commands_are_tagged_by_kind_and_carry_their_id() {
        let command = Command::ToggleBlock {
            id: 9,
            pos: [1, 2, 3],
            to: Some(true),
        };
        let text = serde_json::to_string(&command).unwrap();

        assert!(text.contains(r#""kind":"toggle_block""#), "{text}");
        assert_eq!(command.id(), 9);
        let back: Command = serde_json::from_str(&text).unwrap();
        assert_eq!(back, command);
    }

    #[test]
    fn moving_a_player_is_its_own_command() {
        let text = serde_json::to_string(&Command::MovePlayer {
            id: 5,
            player: 0,
            to: [1.5, 64.6, 0.5],
        })
        .unwrap();
        assert_eq!(
            text,
            r#"{"kind":"move_player","id":5,"player":0,"to":[1.5,64.6,0.5]}"#
        );
    }

    #[test]
    fn a_hold_is_a_mob_a_player_and_a_time() {
        let hold = Command::Hold {
            id: 4,
            entity: 301,
            player: 0,
            seconds: 250.0,
            at: None,
            creep: false,
        };
        let text = serde_json::to_string(&hold).unwrap();
        assert_eq!(
            text, r#"{"kind":"hold","id":4,"entity":301,"player":0,"seconds":250.0}"#,
            "no creep unless asked"
        );
        assert_eq!(hold.id(), 4);
        let creeping: Command = serde_json::from_str(
            r#"{"kind":"hold","id":5,"entity":302,"player":0,"seconds":180.0,"creep":true}"#,
        )
        .unwrap();
        assert!(matches!(creeping, Command::Hold { creep: true, .. }));
    }

    #[test]
    fn a_set_block_without_replace_reads_as_replacing_nothing() {
        let text =
            r#"{"kind":"set_block","id":1,"pos":[0,0,0],"block":"minecraft:torch","state":""}"#;
        let command: Command = serde_json::from_str(text).unwrap();
        assert!(matches!(command, Command::SetBlock { ref replace, .. } if replace.is_empty()));
        assert!(
            matches!(command, Command::SetBlock { settle: 0, ref wall, .. } if wall.is_empty()),
            "no settling and no wall unless asked"
        );
        let plain = serde_json::to_string(&command).unwrap();
        assert!(
            !plain.contains("settle") && !plain.contains("wall"),
            "{plain}"
        );

        let torch = Command::SetBlock {
            id: 2,
            pos: [0, 65, 0],
            block: "minecraft:torch".to_string(),
            state: String::new(),
            replace: vec![],
            settle: 4,
            wall: "minecraft:wall_torch".to_string(),
            rest: false,
            only: false,
        };
        let text = serde_json::to_string(&torch).unwrap();
        assert!(
            text.contains(r#""settle":4"#) && text.contains(r#""wall":"minecraft:wall_torch""#)
        );
        assert_eq!(serde_json::from_str::<Command>(&text).unwrap(), torch);

        let message = Command::GameMessage {
            id: 2,
            player: 0,
            message: "joined".to_string(),
            text: String::new(),
        };
        let text = serde_json::to_string(&message).unwrap();
        assert!(text.contains(r#""kind":"game_message""#), "{text}");
    }

    #[test]
    fn a_target_is_a_block_or_an_entity() {
        let block: Target =
            serde_json::from_str(r#"{"kind":"block","pos":[0,1,2],"block":"minecraft:chest"}"#)
                .unwrap();
        assert_eq!(
            block,
            Target::Block {
                pos: [0, 1, 2],
                block: "minecraft:chest".to_string()
            }
        );
    }
}
