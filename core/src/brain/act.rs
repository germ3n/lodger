//! From a decided event to the host's commands, resolved against the memory at the moment it
//! runs. Each action here is one of the general commands the host carries out; the event's
//! `fits` is the host's check when the command arrives, and `ready` is the mind's check that the
//! moment has come.

use crate::brain::text::{self, Kind};
use crate::events::{Action, Event, FigureWhere, Where};
use crate::memory::distance;
use crate::memory::{Held, Memory};
use crate::model::Rng;
use craftkit::world::{BlockPos, Command, ItemStack, Now, Pos};

/// Whether the event's moment is here: its needs still hold.
/// Whether every need of the event holds now, the hour's with them (until now the hour
/// was read from a default clock: noon never came, and the night and the day were not asked).
pub fn ready(event: &Event, memory: &Memory, now: &Now) -> bool {
    event
        .needs
        .iter()
        .all(|need| super::offer::meets(*need, memory, now))
}

/// Whether an item id is something a furnace turns into something else.
pub fn smeltable(id: &str) -> bool {
    id.starts_with("minecraft:raw_")
        || id.ends_with("_ore")
        || matches!(
            id,
            "minecraft:beef"
                | "minecraft:porkchop"
                | "minecraft:chicken"
                | "minecraft:mutton"
                | "minecraft:rabbit"
                | "minecraft:cod"
                | "minecraft:salmon"
                | "minecraft:potato"
                | "minecraft:kelp"
                | "minecraft:cobblestone"
                | "minecraft:sand"
                | "minecraft:red_sand"
                | "minecraft:clay_ball"
                | "minecraft:cactus"
                | "minecraft:netherrack"
                | "minecraft:wet_sponge"
        )
}

/// Whether an item id burns in a furnace.
pub fn fuel(id: &str) -> bool {
    id.ends_with("_log")
        || id.ends_with("_planks")
        || id.ends_with("_wood")
        || id.ends_with("_sapling")
        || matches!(
            id,
            "minecraft:coal"
                | "minecraft:charcoal"
                | "minecraft:coal_block"
                | "minecraft:stick"
                | "minecraft:blaze_rod"
                | "minecraft:lava_bucket"
                | "minecraft:dried_kelp_block"
                | "minecraft:bamboo"
        )
}

fn block_center(p: BlockPos) -> Pos {
    [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5]
}

fn block_of(p: Pos) -> BlockPos {
    [
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    ]
}

/// The lodger's spot: its chosen place, or a point eight to twelve blocks from the home's
/// center, away from the door, picked once from the seed so it stays the same spot.
pub(crate) fn spot(memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    if let Some(place) = memory.place {
        return Some(place);
    }
    let home = memory.home.as_ref()?;
    let angle = (rng.next_u64() % 8) as f64 * std::f64::consts::FRAC_PI_4;
    let distance = 8.0 + (rng.next_u64() % 5) as f64;

    Some([
        home.center[0] + (distance * angle.cos()).round() as i32,
        home.floor_y,
        home.center[2] + (distance * angle.sin()).round() as i32,
    ])
}

/// How far from anything the target placed the lodger digs: never nearer
/// (`docs/good-to-know.md`, What it never does).
pub const DIG_CLEAR: f64 = 8.0;

/// A spot to dig, clear of everything the target placed by [`DIG_CLEAR`]: its place if that is
/// clear, else the first clear one of eight around the home, 12 to 20 blocks out. None when the
/// target has built all around: the event is not offered. The dev client's `hole` went in
/// under a crafting table at the lodger's place, which the target had built up to.
fn dig_spot(memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    if let Some(place) = memory.place.filter(|p| clear_of_built(memory, *p)) {
        return Some(place);
    }
    let home = memory.home.as_ref()?;
    let first = rng.next_u64() % 8;
    let distance = 12.0 + (rng.next_u64() % 9) as f64;
    (0..8)
        .map(|i| {
            let angle = ((first + i) % 8) as f64 * std::f64::consts::FRAC_PI_4;
            [
                home.center[0] + (distance * angle.cos()).round() as i32,
                home.floor_y,
                home.center[2] + (distance * angle.sin()).round() as i32,
            ]
        })
        .find(|p| clear_of_built(memory, *p))
}

/// Whether nothing the target placed is within [`DIG_CLEAR`] of `pos`.
pub fn clear_of_built(memory: &Memory, pos: BlockPos) -> bool {
    // Their torches along a tunnel are no building of theirs: a lit tunnel never got its dig.
    let light = |p: &BlockPos| {
        memory.walls.iter().any(|(q, id)| {
            q == p && (id.ends_with("torch") || id.ends_with("lantern") || id.contains("candle"))
        })
    };
    memory
        .placed
        .iter()
        .chain(memory.placed_by_others.iter())
        .filter(|p| !light(p))
        .all(|p| {
            let (dx, dy, dz) = (
                (p[0] - pos[0]) as f64,
                (p[1] - pos[1]) as f64,
                (p[2] - pos[2]) as f64,
            );
            (dx * dx + dy * dy + dz * dz).sqrt() > DIG_CLEAR
        })
}

/// The block a command digs out of natural ground, if it digs: air set where only ground may
/// be replaced. A path worn into grass or water poured on a floor is not digging.
pub fn dug(command: &Command) -> Option<BlockPos> {
    match command {
        Command::SetBlock {
            pos,
            block,
            replace,
            ..
        } if block == "minecraft:air"
            && replace.iter().any(|r| SOLID_GROUND.contains(&r.as_str())) =>
        {
            Some(*pos)
        }
        _ => None,
    }
}

/// What every frame of the house shows (`item_frames_face`): the thing it holds, or the one
/// they hold most.
pub fn frame_shows(memory: &Memory) -> Option<String> {
    memory
        .held
        .as_ref()
        .map(|h| h.item.id.clone())
        .or_else(|| memory.favourites.first().map(|(id, _)| id.clone()))
        .filter(|id| !id.is_empty())
}

/// The block `blocks` behind the target, the way they face turned round (yaw 0 faces +z, as
/// `sounds::commands` reads it).
fn behind(memory: &Memory, blocks: f64) -> BlockPos {
    let yaw = (memory.last.yaw as f64).to_radians();
    let at = memory.last.pos;
    [
        (at[0] + yaw.sin() * blocks).floor() as i32,
        (at[1] + 1.0).floor() as i32,
        (at[2] - yaw.cos() * blocks).floor() as i32,
    ]
}

/// The place the event happens, from the memory, or none for a thing with no place.
pub fn place(event: &Event, memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    let home = memory.home.as_ref();
    // The house's sounds come from a real door or chest of the house they are in, theirs
    // first (owner: "any house, yours first").
    if super::sounds::HOUSE_SOUNDS.contains(&event.id) {
        return super::sounds::house_spot(event.id, memory, rng.next_u64() as usize);
    }
    // The painting nearest them; a compass's place.
    if event.id == "painting_flip" {
        let at = block_of(memory.last.pos);
        return memory
            .decor("painting")
            .min_by(|a, b| distance(a.pos, at).total_cmp(&distance(b.pos, at)))
            .map(|d| d.pos);
    }
    if event.id == "compass_points" {
        return memory.place.or_else(|| spot(memory, rng));
    }
    // Ripples on the water near them, from the shore on their side outward.
    if event.id == "ripples" {
        return memory.shore_toward_target(2.0).map(block_of);
    }
    // Under the bed they lie in.
    if event.id == "under_the_bed" {
        return home.and_then(|h| h.bed).map(|b| [b[0], b[1] - 1, b[2]]);
    }
    // The knock answered from the door they knocked on.
    if event.id == "answer_knock" {
        return memory.knocked_now;
    }
    // Breathing at the ear of the one who crouches.
    if event.id == "breath" {
        return Some(block_of(memory.last.pos));
    }
    // A cold furnace's crackle comes from their furnace.
    if event.id == "cold_crackle" {
        return memory.furnace().map(|f| f.pos);
    }

    match event.action {
        Action::Wait
        | Action::LootGift
        | Action::Hint(_, _)
        | Action::RefuseRest
        | Action::Stare
        | Action::SoundAtTarget(_)
        | Action::SoundFar(_)
        | Action::TabEntry
        | Action::TabTwice
        | Action::Message(_)
        | Action::DeathMessage
        | Action::Says
        | Action::SaysKills
        | Action::Whisper
        | Action::RepeatsYou
        | Action::Answers
        | Action::HostilesStare
        | Action::NameAnimal
        | Action::DyeSheep
        | Action::GolemTurned
        | Action::StandWatch
        | Action::FramesTurn
        | Action::ToggleCandles
        | Action::CarpetShift
        | Action::LightPath
        | Action::PushVehicle(_)
        | Action::MiningSounds
        | Action::VillageDoors
        | Action::ScanVillage
        | Action::ScanOre
        | Action::FrameEmptied
        | Action::FrameSwap
        | Action::StandDress
        | Action::ToolUsed
        | Action::ColdFurnace
        | Action::TidyByColour
        | Action::IcePath
        | Action::HandOnShoulder
        | Action::Footsteps
        | Action::Pack
        | Action::MobComes(_)
        | Action::HostilesClose
        | Action::Shove
        | Action::FramesShow
        | Action::BobberUnder
        | Action::GhastName
        | Action::EnderHolds
        | Action::WhileAway
        | Action::Shoot(_)
        | Action::SplashDark
        | Action::BrewLeft
        | Action::PotionChanged
        | Action::DoorCloud
        | Action::BedOccupied => None,
        Action::OreMarked => memory.scanned("_ore").next(),
        Action::LanternAhead => memory.lantern_spot(),
        Action::DeathLight => memory.unmarked_death(),
        Action::LightsOut => memory.lights_behind().first().map(|(p, _)| *p),
        Action::WarpAnimal => memory.warp_animal().map(|c| block_of(c.pos)),
        Action::SoulSwap => memory.soulable_light().map(|(p, _)| p),
        Action::LightTaken => memory.habits.last_light.as_ref().map(|(p, _)| *p),
        Action::CaveLit => memory.cave_torches().first().copied(),
        Action::WallSwap(suffix) => {
            let candidates: Vec<BlockPos> = memory
                .walls
                .iter()
                .filter(|(_, id)| match suffix {
                    // A light that soul fire can take: a torch, a wall torch, a lantern.
                    "torch" => soul_light(id).is_some(),
                    // Any block of theirs with a family; a light is among them for its id only.
                    "" => !crate::memory::cousins(id).is_empty(),
                    _ => id.ends_with(suffix),
                })
                .map(|(p, _)| *p)
                .collect();
            pick_ref(&candidates, rng)
        }
        Action::BedBreaks => home.and_then(|h| h.bed),
        // Their door: where the figure comes in from.
        Action::Choke(_) | Action::ShotInBed(_) | Action::LiesBeside => memory.usual_door(),
        // The middle of the house: the ring stands round it.
        Action::RingOfFigures(_) => home.map(|h| h.center),
        Action::UpsideDown => home.and_then(|h| h.bed),
        Action::Crawl => memory.usual_door(),
        // The house's middle, with two windows or more: a figure goes outside each.
        Action::EveryWindow => home
            .filter(|h| windows(memory, h.center).len() >= 2)
            .map(|h| h.center),
        // The door of the house they are walking to; their usual door when the moment has
        // passed since it was offered.
        Action::DoorWontOpen => home
            .and_then(|h| {
                h.doors
                    .iter()
                    .filter(|d| memory.approaching(**d, 6.0))
                    .min_by(|a, b| {
                        distance(block_of(memory.last.pos), **a)
                            .total_cmp(&distance(block_of(memory.last.pos), **b))
                    })
                    .copied()
            })
            .or_else(|| memory.usual_door()),
        // Two blocks out of their usual door, away from the house's middle.
        Action::Sleepwalk => match (memory.usual_door(), home) {
            (Some(door), Some(h)) => {
                let out = step_toward(h.center, door);
                Some([door[0] + out[0] * 2, door[1], door[2] + out[1] * 2])
            }
            _ => None,
        },
        // Two blocks past their newest light from them, on its floor: a wall light hangs a block
        // over the floor, a standing one on it.
        Action::TorchFlash => memory.habits.last_light.as_ref().map(|(light, kind)| {
            let away = step_toward(block_of(memory.last.pos), *light);
            let floor = if kind.contains("wall") {
                light[1] - 1
            } else {
                light[1]
            };
            [light[0] + away[0] * 2, floor - 1, light[2] + away[1] * 2]
        }),
        Action::WhereYouDied => memory.last_death(),
        Action::TheOffer => memory.usual_door(),
        Action::Breadcrumbs => memory
            .used_container()
            .and_then(crumb_stack)
            .and(memory.usual_door()),
        Action::TorchRing => home.map(|h| h.center),
        // Ten blocks south of the house's middle, on the ground there.
        Action::NameInTheSnow => home.map(|h| [h.center[0] - 8, -10_000, h.center[2] + 10]),
        Action::ChestWontOpen => memory.used_container().map(|c| c.pos),
        Action::LostHour => Some(block_of(memory.last.pos)),
        Action::BatsInTheHouse => home.map(|h| h.center),
        // The wall block of theirs nearest their pet.
        Action::PetStaresAtWall => memory.pets().next().and_then(|pet| {
            let at = block_of(pet.pos);
            memory
                .walls
                .iter()
                .map(|(p, _)| *p)
                .min_by(|a, b| distance(*a, at).total_cmp(&distance(*b, at)))
        }),
        // Where they are, holding something, with no monster near.
        Action::SlipsFromHand => (!memory.last.main_hand.is_empty()
            && memory.last.hostiles_near.is_empty())
        .then_some(block_of(memory.last.pos)),
        // Fifty blocks from the house, any way, on the ground there.
        Action::StoneCircle => home.map(|h| {
            let angle = rng.next_f64() * std::f64::consts::TAU;
            [
                h.center[0] + (angle.cos() * 50.0).round() as i32,
                -10_000,
                h.center[2] + (angle.sin() * 50.0).round() as i32,
            ]
        }),
        // A point of their route twenty-four to sixty-four blocks from them: its ground known.
        Action::CampNearby => {
            let at = block_of(memory.last.pos);
            memory.route().and_then(|r| {
                r.points
                    .iter()
                    .copied()
                    .find(|p| (24.0..=64.0).contains(&distance(*p, at)))
            })
        }
        Action::FlowersWilted => memory
            .plants
            .iter()
            .find(|(id, _)| crate::memory::is_flower(id))
            .map(|(_, p)| *p),
        // The house, with two doors or more.
        Action::EveryDoorOpens => home.filter(|h| h.doors.len() >= 2).map(|h| h.center),
        // A window of the house within twenty blocks of them; its first, when none is.
        Action::MirrorInTheWindow => home.and_then(|h| {
            let at = block_of(memory.last.pos);
            let all = windows(memory, h.center);
            all.iter()
                .find(|w| distance(**w, at) <= 20.0)
                .or(all.first())
                .copied()
        }),
        Action::EatsAtTheirChest => memory
            .used_container()
            .filter(|c| chest_food(c).is_some())
            .map(|c| c.pos),
        Action::RidesTheirHorse => their_animal(memory, "horse", false).map(|h| block_of(h.pos)),
        // The cave nearest their door, its x and z, forty-eight blocks off at most along the way.
        Action::PrintsToTheCave => memory.usual_door().and_then(|door| {
            let cave = memory
                .caves
                .iter()
                .min_by(|a, b| distance(**a, door).total_cmp(&distance(**b, door)))?;
            let (dx, dz) = ((cave[0] - door[0]) as f64, (cave[2] - door[2]) as f64);
            let length = (dx * dx + dz * dz).sqrt().max(1.0);
            let far = length.min(48.0);
            Some([
                door[0] + (dx / length * far).round() as i32,
                door[1],
                door[2] + (dz / length * far).round() as i32,
            ])
        }),
        Action::WavesFromInside => home.and_then(|h| windows(memory, h.center).first().copied()),
        // Thirty blocks ahead of where they look, a little aside, on the ground.
        Action::JumpingFar => {
            let at = block_of(memory.last.pos);
            let yaw = (memory.last.yaw as f64).to_radians() - 0.25;
            Some([
                at[0] + (-yaw.sin() * 30.0).round() as i32,
                -10_000,
                at[2] + (yaw.cos() * 30.0).round() as i32,
            ])
        }
        // Their crafting table in the house.
        Action::AtTheirTable => memory.fixtures("crafting_table").map(|f| f.pos).find(|p| {
            memory
                .home
                .as_ref()
                .is_some_and(|h| distance(h.center, *p) <= 12.0)
        }),
        Action::PulledUnder | Action::HeldBreath | Action::AllFreeze => {
            Some(block_of(memory.last.pos))
        }
        Action::ItsLightsOn | Action::RespawnItsBed | Action::FireworkOverItsHouse => {
            memory.its_house().map(|h| h.site)
        }
        Action::EnderChestNote => memory.fixtures("ender_chest").next().map(|f| f.pos),
        Action::EyeTurnsHome => home.map(|h| h.center),
        // Their house, when they are away from it: thirty-two blocks or more.
        Action::FigureOnMap => home
            .map(|h| h.center)
            .filter(|c| distance(*c, block_of(memory.last.pos)) >= 32.0),
        Action::InALine => in_a_line(memory).first().copied(),
        Action::ChickenLays => memory
            .last
            .creatures_near
            .iter()
            .find(|c| c.kind == "chicken")
            .map(|c| block_of(c.pos)),
        // Where they came back: the portal, as the host said it, in this world.
        Action::WaitsAtThePortal => memory
            .milestones
            .iter()
            .rev()
            .find(|m| m.kind == "crossed")
            .and_then(|m| m.pos),
        Action::WearsYourFace => face_wearer(memory).map(|z| block_of(z.pos)),
        Action::AnimalFollows => memory
            .animals_near_target()
            .first()
            .map(|a| block_of(a.pos)),
        // Twenty-four blocks ahead of where they look, a little to the side, on the ground.
        Action::PeeksFromGround => {
            let at = block_of(memory.last.pos);
            let yaw = (memory.last.yaw as f64).to_radians() + 0.3;
            Some([
                at[0] + (-yaw.sin() * PEEK_FROM).round() as i32,
                -10_000,
                at[2] + (yaw.cos() * PEEK_FROM).round() as i32,
            ])
        }
        // Thirty blocks ahead of where they look: the middle of its crossing.
        Action::CrossingAhead => {
            let at = block_of(memory.last.pos);
            let yaw = (memory.last.yaw as f64).to_radians();
            Some([
                at[0] + (-yaw.sin() * CROSSING_AHEAD).round() as i32,
                -10_000,
                at[2] + (yaw.cos() * CROSSING_AHEAD).round() as i32,
            ])
        }
        // Nine blocks ahead of them, on the ground: near enough that their pet never strays far
        // enough from them for the game to bring it back at once (twelve).
        Action::PetFollows => {
            let at = block_of(memory.last.pos);
            let yaw = (memory.last.yaw as f64).to_radians();
            Some([
                at[0] + (-yaw.sin() * PET_TO).round() as i32,
                -10_000,
                at[2] + (yaw.cos() * PET_TO).round() as i32,
            ])
        }
        // Forty blocks ahead of where they look, on the ground: it falls into their sky there.
        Action::FallsFromTheSky => {
            let at = block_of(memory.last.pos);
            let yaw = (memory.last.yaw as f64).to_radians();
            Some([
                at[0] + (-yaw.sin() * FALLS_AHEAD).round() as i32,
                -10_000,
                at[2] + (yaw.cos() * FALLS_AHEAD).round() as i32,
            ])
        }
        // The chest they use, when a face can be laid in it.
        Action::FaceInTheChest => memory
            .used_container()
            .filter(|c| !c.is_furnace() && face_moves(&c.contents).is_some())
            .map(|c| c.pos),
        // The top of the shaft they are down; its highest ladder when they have climbed out
        // since it was offered.
        Action::TopOfTheLadder => shaft_top(memory).or_else(|| {
            memory
                .fixtures("ladder")
                .map(|f| f.pos)
                .max_by_key(|p| p[1])
        }),
        // The point of their trail three blocks or more back from where they stand; three
        // blocks behind them when the trail is short.
        Action::Rubberband => {
            let at = block_of(memory.last.pos);
            memory
                .trail
                .iter()
                .rev()
                .find(|p| distance(**p, at) >= RUBBERBAND_BACK)
                .copied()
                .or_else(|| {
                    let way = facing_way(memory.last.yaw);
                    let back = RUBBERBAND_BACK as i32;
                    Some([at[0] - way[0] * back, at[1], at[2] - way[1] * back])
                })
        }
        // Where they are: the host seats it in their boat.
        Action::BoatPassenger => Some(block_of(memory.last.pos)),
        // Ahead of them, where they look, far off on the ground: they see it start.
        Action::Hunted => {
            let at = block_of(memory.last.pos);
            let yaw = (memory.last.yaw as f64).to_radians();
            Some([
                at[0] + (-yaw.sin() * HUNTED_FROM).round() as i32,
                -10_000,
                at[2] + (yaw.cos() * HUNTED_FROM).round() as i32,
            ])
        }
        Action::Figure(FigureWhere::Roof, _, _) => {
            home.map(|h| [h.center[0], h.floor_y + 5, h.center[2]])
        }
        // The block under its feet, as the figure stands one up: in the doorway, beside the
        // chest, in the bed (they stood a block too high, in the lintel and on the chest).
        Action::Figure(FigureWhere::Doorway, _, _) => home
            .and_then(|h| pick(&h.doors, rng))
            .map(|d| [d[0], d[1] - 1, d[2]]),
        Action::Figure(FigureWhere::AtChest, _, _) => memory
            .used_container()
            .map(|c| [c.pos[0] + 1, c.pos[1] - 1, c.pos[2]]),
        Action::Figure(FigureWhere::InBed, _, _) => {
            home.and_then(|h| h.bed).map(|b| [b[0], b[1] - 1, b[2]])
        }
        // Its feet three below the water's top, so its head is just under the surface.
        Action::Figure(FigureWhere::UnderWater, _, _) => memory
            .shore_toward_target(3.0)
            .map(|p| [p[0].floor() as i32, p[1] as i32 - 3, p[2].floor() as i32]),
        // The floor two blocks past the bed from where they stand as they wake: the figure
        // stands at its foot, across it from them.
        Action::Figure(FigureWhere::BedFoot, _, _) => home.and_then(|h| h.bed).map(|b| {
            let at = block_of(memory.last.pos);
            let (dx, dz) = (b[0] - at[0], b[2] - at[2]);
            let (sx, sz) = if dx.abs() >= dz.abs() {
                (if dx >= 0 { 1 } else { -1 }, 0)
            } else {
                (0, dz.signum())
            };
            [b[0] + sx * 2, b[1] - 1, b[2] + sz * 2]
        }),
        Action::ShiftBed | Action::SheetsChanged | Action::SecondBed => home.and_then(|h| h.bed),
        Action::Scarecrow => field_middle(memory),
        Action::SignWord => pick_ref(
            memory
                .fixtures("sign")
                .map(|f| f.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::CountUp => kill_sign_spot(memory),
        Action::WeaponOnWall => memory.decor("item_frame").next().map(|d| d.pos),
        // Two blocks out from the door they use and one to the side, clear of the kill sign.
        Action::HeadOnPost => match (memory.usual_door(), home) {
            (Some(door), Some(h)) => {
                let step = step_toward(h.center, door);
                Some([
                    door[0] + step[0] * 2 + step[1],
                    door[1],
                    door[2] + step[1] * 2 + step[0],
                ])
            }
            _ => None,
        },
        Action::GraveEach => spot(memory, rng),
        Action::JukeboxDisc => memory.fixtures("jukebox").next().map(|f| f.pos),
        Action::VillageLantern => memory.route_by_village(),
        Action::MineBeside | Action::TunnelGoesOn | Action::SignDeep | Action::TunnelToward => {
            memory.newest_tunnel().map(|t| t.end())
        }
        Action::TunnelSealed => memory.oldest_tunnel().map(|t| t.mouth()),
        Action::PortalLit => memory.placed_obsidian.first().copied(),
        Action::MinedOut => memory.ore_ahead(),
        Action::LadderDown => spot(memory, rng),
        Action::TorchesFarEnd => tunnel_lights(memory).first().copied(),
        Action::WayBackCloses => memory.behind_in_tunnel(),
        Action::VillagerWord => memory.villager_near().map(|v| block_of(v.pos)),
        Action::VillageAbandoned => memory.gathering_door(),
        Action::BellTwice => memory.rang_bell_now,
        Action::Trail | Action::TrailIn => memory.usual_door(),
        Action::BoatWaiting => memory.shore_toward_target(1.0).map(block_of),
        Action::FishingLine => memory.shore_toward_target(4.0).map(block_of),
        // Beside the door, two out, on the side away from the words on the sign.
        Action::Snowman => match (memory.usual_door(), home) {
            (Some(door), Some(h)) => {
                let step = step_toward(h.center, door);
                Some([
                    door[0] + step[0] * 3 + step[1] * 2,
                    door[1],
                    door[2] + step[1] * 3 + step[0] * 2,
                ])
            }
            _ => None,
        },
        Action::BoatInHouse => home.map(|h| [h.center[0], h.floor_y, h.center[2]]),
        Action::UnderWater => memory.shore_toward_target(4.0).map(block_of),
        Action::WetSteps => memory.usual_door(),
        Action::LecternPage => memory.fixtures("lectern").next().map(|f| f.pos),
        Action::SwapFuel => memory.furnace().map(|f| f.pos),
        Action::Brewing => memory.fixtures("brewing_stand").next().map(|f| f.pos),
        Action::GrindstoneStrip => memory.used_container().map(|c| c.pos),
        Action::LampsBlink => memory.fixtures("redstone_lamp").next().map(|f| f.pos),
        Action::ComposterFills => memory.fixtures("composter").next().map(|f| f.pos),
        Action::CraftedForYou => memory.fixtures("crafting_table").next().map(|f| f.pos),
        Action::BellRings => memory.fixtures("bell").next().map(|f| f.pos),
        // Beside the door they use, two out, one to the other side from the head on the post.
        Action::WordsOnSign => match (memory.usual_door(), home) {
            (Some(door), Some(h)) => {
                let step = step_toward(h.center, door);
                Some([
                    door[0] + step[0] * 2 - step[1],
                    door[1],
                    door[2] + step[1] * 2 - step[0],
                ])
            }
            _ => None,
        },
        Action::BeesOut => memory
            .fixtures("beehive")
            .chain(memory.fixtures("bee_nest"))
            .next()
            .map(|f| f.pos),
        Action::HorseMoved => leashed_horse(memory).map(|c| block_of(c.pos)),
        Action::FieldWord => memory
            .field_word()
            .and_then(|(_, cells)| cells.first().copied()),
        Action::FaceTheHouse => home.map(|h| h.center),
        Action::OneMore => memory.kept_most().map(|(_, at)| block_of(at)),
        Action::RowWithered => {
            let rows = memory.field_rows();
            (!rows.is_empty()).then(|| rows[rng.next_u64() as usize % rows.len()][0])
        }
        Action::EggsToDoor => memory.egg_line().first().map(|p| block_of(*p)),
        Action::Charge(..) => memory
            .animals_near_target()
            .first()
            .map(|c| block_of(c.pos)),
        Action::Figure(FigureWhere::AcrossWater, _, _) => memory.feature("water").map(|w| {
            // The far side: past the water's middle from the target, by its width.
            let at = block_of(memory.last.pos);
            let step = step_toward(at, w.pos);
            let reach = w.size as i32 / 2 + 2;
            [
                w.pos[0] + step[0] * reach,
                -10_000,
                w.pos[2] + step[1] * reach,
            ]
        }),
        Action::Figure(FigureWhere::Behind, _, _) => memory.route().map(|r| {
            // The route's end farther from where they stand now.
            let at = block_of(memory.last.pos);
            let end = if distance(r.start(), at) > distance(r.end(), at) {
                r.start()
            } else {
                r.end()
            };
            [end[0], -10_000, end[2]]
        }),
        Action::Figure(FigureWhere::Route, _, _) => memory
            .route()
            .map(|r| [r.start()[0], -10_000, r.start()[2]]),
        // Forty blocks to their side, on the ground, to mirror them.
        Action::Figure(FigureWhere::Mirror, _, _) => {
            let at = block_of(memory.last.pos);
            let yaw = (memory.last.yaw as f64).to_radians();
            let (sx, sz) = (yaw.cos(), yaw.sin());
            Some([
                at[0] + (sx * 40.0).round() as i32,
                -10_000,
                at[2] + (sz * 40.0).round() as i32,
            ])
        }
        // Outside a window of the house: a pane, a block out from it away from the middle,
        // its head at the pane.
        Action::Figure(FigureWhere::Window, _, _) => {
            match (memory.fixtures("glass_pane").next(), home) {
                (Some(pane), Some(h)) => {
                    let out = step_toward(h.center, pane.pos);
                    Some([pane.pos[0] + out[0], pane.pos[1] - 2, pane.pos[2] + out[1]])
                }
                _ => None,
            }
        }
        // Beside the villager nearest them.
        Action::Figure(FigureWhere::Crowd, _, _) => memory.villager_near().map(|v| {
            let p = block_of(v.pos);
            [p[0] + 1, p[1] - 1, p[2]]
        }),
        Action::Figure(FigureWhere::Close, _, _) | Action::FigureSwing(_) => {
            Some(behind_close(memory))
        }
        Action::FigureWave
        | Action::Figure(FigureWhere::FarCopying, _, _)
        | Action::Figure(FigureWhere::FarWorking, _, _) => far_spot(memory, rng),
        // Thirty blocks ahead of where they face, on the ground: where the figure starts for the
        // rock.
        Action::DigInto(_) => {
            let at = block_of(memory.last.pos);
            let way = facing_way(memory.last.yaw);
            Some([at[0] + way[0] * 30, -10_000, at[2] + way[1] * 30])
        }
        // Seven blocks past their light from them, on the ground: the edge of its reach.
        Action::IntoTheDark
        | Action::OneMoreEachNight
        | Action::Figure(FigureWhere::EdgeOfLight, _, _) => light_near(memory).map(|light| {
            let at = block_of(memory.last.pos);
            let away = step_toward(at, light);
            [light[0] + away[0] * 7, -10_000, light[2] + away[1] * 7]
        }),
        Action::Build(event) => memory.next_build(event).map(|(site, _, _)| site),
        Action::CaveRoom => memory.unentered_cave().and_then(|c| c.first().copied()),
        Action::Figure(FigureWhere::Far, _, _) => {
            // Fifty to seventy blocks from the target, away from the home.
            let at = block_of(memory.last.pos);
            let from = home.map(|h| h.center).unwrap_or(at);
            let dx = (at[0] - from[0]) as f64;
            let dz = (at[2] - from[2]) as f64;
            let length = (dx * dx + dz * dz).sqrt();
            let (ux, uz) = if length < 1.0 {
                let angle = rng.next_f64() * std::f64::consts::TAU;
                (angle.cos(), angle.sin())
            } else {
                (dx / length, dz / length)
            };
            let reach = 50.0 + (rng.next_u64() % 21) as f64;
            Some([
                at[0] + (ux * reach).round() as i32,
                -10_000,
                at[2] + (uz * reach).round() as i32,
            ])
        }
        Action::RotateFixture(kind) => pick_ref(
            memory
                .fixtures(kind)
                .map(|f| f.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::SoundPair(kind, _, _) => pick_ref(
            memory
                .fixtures(kind)
                .map(|f| f.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::OpenOnEnter => memory.used_container().map(|c| c.pos),
        Action::PlantFlower => home.and_then(|h| pick(&h.doors, rng)),
        Action::ReplantOther => memory.feature("crops").map(|f| f.pos),
        Action::WokeElsewhere | Action::OtherHouse => home.and_then(|h| h.bed),
        // A block of the floor itself, under where they stand: at the standing level it was a
        // source block in the room's air, which flooded it.
        Action::BucketFloor => home.map(|h| [h.center[0] + 2, h.floor_y - 1, h.center[2] + 2]),
        // The door they walk up to, not another of the house's.
        Action::DoorAhead => home.and_then(|h| {
            h.doors
                .iter()
                .find(|d| memory.approaching(**d, 6.0))
                .copied()
                .or_else(|| pick(&h.doors, rng))
        }),
        Action::SnowCleared | Action::Trample => home.and_then(|h| pick(&h.doors, rng)),
        Action::HarvestOne => memory.feature("crops").map(|f| f.pos),
        Action::DiarySold | Action::DiaryBrought => memory.villager_near().map(|v| block_of(v.pos)),
        Action::VillagerNamed => memory.unnamed_villager_near().map(|v| block_of(v.pos)),
        Action::LetterInChest => memory.used_container().map(|c| c.pos),
        Action::LecternNotes => memory.fixtures("lectern").next().map(|f| f.pos),
        Action::PageInBook => memory
            .in_used_container(|id| id == "minecraft:writable_book")
            .map(|(c, _)| c.pos),
        // Where the boat is: never moved while they look at it.
        Action::MoveVehicle => memory.their_boat().map(|c| block_of(c.pos)),
        Action::PlateLaid => memory.used_container().map(|c| c.pos),
        Action::ToggleDoor
        | Action::DoorHinge
        | Action::DoorHeld
        | Action::Wake
        | Action::KillSign => home.and_then(|h| pick(&h.doors, rng)),
        // The path wears where they walk: the door they come and go by (N1).
        Action::PathWorn => memory.usual_door(),
        Action::ToggleFixture(kind) | Action::SoundAtFixture(kind, _) => pick_ref(
            memory
                .fixtures(kind)
                .map(|f| f.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::Taps(kind, _, _) => {
            if kind.is_empty() {
                // A knock comes at the door they use (N1).
                memory.usual_door()
            } else {
                pick_ref(
                    memory
                        .fixtures(kind)
                        .map(|f| f.pos)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    rng,
                )
            }
        }
        Action::Sound(_) => home.map(|h| {
            // Another room: six blocks off the center, a random way.
            let angle = rng.next_f64() * std::f64::consts::TAU;
            [
                h.center[0] + (6.0 * angle.cos()).round() as i32,
                h.floor_y + 1,
                h.center[2] + (6.0 * angle.sin()).round() as i32,
            ]
        }),
        Action::OpenLid
        | Action::ShuffleContainer
        | Action::SplitStack
        | Action::TidyContainer
        | Action::SwapStacks
        | Action::FillGap => memory.used_container().map(|c| c.pos),
        Action::CookOne | Action::TakeOutput => memory.furnace().map(|f| f.pos),
        Action::ItemOnTable => memory.fixtures("crafting_table").next().map(|f| f.pos),
        Action::EggOnBed => home.and_then(|h| h.bed),
        Action::Take => memory.favourite_in_container().map(|(c, _)| c.pos),
        Action::ReturnTaken | Action::ReturnUsed => memory.held.as_ref().map(|h| h.from),
        Action::HopperEmptied => memory.fixtures("hopper").next().map(|f| f.pos),
        Action::LadderGap => ladder_column(memory).map(|c| c[c.len() / 2]),
        Action::TreeGrown => memory
            .plants
            .iter()
            .find(|(id, _)| id.ends_with("_sapling"))
            .map(|(_, p)| *p),
        Action::Backfill => memory.holes.last().map(|(p, _)| *p),
        Action::Alcove => pick_ref(
            memory
                .walls
                .iter()
                .map(|(p, _)| *p)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        // A crafting table, a furnace or a barrel of theirs: never a chest, which may be one
        // half of two.
        // Another mod's furniture first (owner: "use their decorations"), then the
        // game's work blocks.
        Action::ShiftFurniture => pick_ref(
            &memory
                .fixtures("furniture")
                .chain(memory.fixtures("crafting_table"))
                .map(|f| f.pos)
                .collect::<Vec<_>>(),
            rng,
        )
        .or_else(|| memory.furnace().map(|f| f.pos))
        .or_else(|| {
            memory
                .containers
                .iter()
                .find(|c| c.theirs() && c.kind.ends_with("barrel"))
                .map(|c| c.pos)
        }),
        Action::RoomDark => home.and_then(|h| pick(&h.lights, rng)),
        Action::BedTurned => home.and_then(|h| h.bed),
        Action::FlowersPotted => memory.fixtures("flower_pot").next().map(|f| f.pos),
        Action::PumpkinFace => memory
            .plants
            .iter()
            .find(|(id, _)| id == "minecraft:pumpkin")
            .map(|(_, p)| *p),
        Action::FullOfOne => memory.used_container().map(|c| c.pos),
        Action::ShulkerMoved => memory.fixtures("shulker_box").next().map(|f| f.pos),
        Action::RenameOne | Action::BookWord | Action::MapMarked => {
            memory.used_container().map(|c| c.pos)
        }
        // Another chest of theirs than the one just opened, one of the house's.
        Action::ChestAnswers => {
            let opened = memory.opened_now;
            pick_ref(
                memory
                    .containers
                    .iter()
                    .filter(|c| c.theirs() && !c.is_furnace() && Some(c.pos) != opened)
                    .map(|c| c.pos)
                    .collect::<Vec<_>>()
                    .as_slice(),
                rng,
            )
        }
        Action::PlaceTaken => memory
            .place
            .or_else(|| home.and_then(|h| h.bed))
            .or_else(|| home.map(|h| h.center)),
        Action::MoveLight => home.and_then(|h| pick(&h.lights, rng)),
        Action::AddTorch => home.map(|h| {
            let dx = (rng.next_u64() % 7) as i32 - 3;
            let dz = (rng.next_u64() % 7) as i32 - 3;
            [h.center[0] + dx, h.floor_y, h.center[2] + dz]
        }),
        Action::Hole => dig_spot(memory, rng),
        Action::Cairn | Action::Sapling | Action::Sign | Action::Grave => spot(memory, rng),
        Action::Cauldron => memory.fixtures("cauldron").next().map(|f| f.pos),
        Action::RotateFrame => pick_ref(
            memory
                .decor("item_frame")
                .map(|d| d.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::SwapPainting => pick_ref(
            memory
                .decor("painting")
                .map(|d| d.pos)
                .collect::<Vec<_>>()
                .as_slice(),
            rng,
        ),
        Action::StandTurn => memory.decor("armor_stand").next().map(|d| d.pos),
        Action::PetGrowl => memory.pets().next().map(|p| block_of(p.pos)),
        Action::MoveCreature(_, Where::Home) => home.map(|h| h.center),
        // Two blocks out from the door they use, on the ground (it was the door's own block).
        Action::MoveCreature(_, Where::OutsideDoor) => memory.doorway_spot().map(|s| {
            let b = block_of(s);
            [b[0], b[1] - 1, b[2]]
        }),
        Action::MoveCreature(_, Where::RouteEnd) => memory.route_end(),
        Action::MoveCreature(_, Where::Corner) => {
            home.map(|h| [h.center[0] + 3, h.floor_y, h.center[2] + 3])
        }
    }
}

fn pick<T: Copy>(list: &[T], rng: &mut Rng) -> Option<T> {
    if list.is_empty() {
        None
    } else {
        Some(list[(rng.next_u64() % list.len() as u64) as usize])
    }
}

fn pick_ref(list: &[BlockPos], rng: &mut Rng) -> Option<BlockPos> {
    pick(list, rng)
}

fn sound_at(
    ids: &mut impl FnMut() -> u64,
    target: u32,
    sound: &str,
    pos: Pos,
    volume: f32,
) -> Command {
    Command::PlaySound {
        id: ids(),
        player: target,
        sound: sound.to_string(),
        pos,
        volume,
        pitch: 1.0,
    }
}

fn set(ids: &mut impl FnMut() -> u64, pos: BlockPos, block: &str, replace: &[&str]) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: block.to_string(),
        state: String::new(),
        replace: replace.iter().map(|r| r.to_string()).collect(),
        settle: 0,
        wall: String::new(),
        rest: false,
        only: false,
    }
}

/// A full block on the ground near `pos`, as a player would build: within three blocks down
/// or up, on something solid (the ground, or the block placed before it), never in the air.
fn on_ground(ids: &mut impl FnMut() -> u64, pos: BlockPos, block: &str) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: block.to_string(),
        state: String::new(),
        replace: GROUND_COVER.iter().map(|r| r.to_string()).collect(),
        settle: 3,
        wall: String::new(),
        rest: true,
        only: false,
    }
}

/// What lies on the ground and a block placed there may take the place of: grass, ferns,
/// snow, never anything built.
const GROUND_COVER: &[&str] = &[
    "minecraft:short_grass",
    "minecraft:tall_grass",
    "minecraft:fern",
    "minecraft:snow",
];

/// The blocks a command changes where the target could see it: what the view rule keeps out of
/// sight. What happens inside a chest, to an entity by its id, or to the screen is not here; an
/// event's own spot covers the entities.
pub fn spots(command: &Command) -> Vec<BlockPos> {
    let block = |p: [f64; 3]| {
        [
            p[0].floor() as i32,
            p[1].floor() as i32,
            p[2].floor() as i32,
        ]
    };
    match command {
        Command::ToggleBlock { pos, .. }
        | Command::SetBlock { pos, .. }
        | Command::SwapBlock { pos, .. }
        | Command::HideBlock { pos, .. }
        | Command::RotateBlock { pos, .. }
        | Command::BreakBlock { pos, .. }
        | Command::SetSign { pos, .. } => vec![*pos],
        Command::MoveBlock { from, to, .. } => vec![*from, *to],
        Command::DropItem { pos, .. } => vec![block(*pos)],
        Command::MoveEntity { to, .. } => vec![block(*to)],
        _ => vec![],
    }
}

/// The weapon they hold most, in a chest of theirs: the chest and the stack.
pub fn their_weapon(memory: &Memory) -> Option<(&crate::memory::Container, &ItemStack)> {
    const WEAPONS: &[&str] = &["_sword", "_axe", "bow", "crossbow", "trident", "mace"];
    memory
        .favourites
        .iter()
        .filter(|(id, _)| WEAPONS.iter().any(|w| id.ends_with(w)))
        .find_map(|(id, _)| {
            memory
                .containers
                .iter()
                .filter(|c| c.theirs() && !c.is_furnace())
                .find_map(|c| c.contents.iter().find(|s| s.id == *id).map(|s| (c, s)))
        })
}

/// The kind they kill most of those with a head, and its head block.
pub fn head_of_their_kills(memory: &Memory) -> Option<(&str, &'static str)> {
    memory.tally.iter().filter(|t| t.kills > 0).find_map(|t| {
        let head = match t.kind.as_str() {
            "zombie" | "husk" | "drowned" | "zombie_villager" => "minecraft:zombie_head",
            "skeleton" | "stray" | "bogged" => "minecraft:skeleton_skull",
            "creeper" => "minecraft:creeper_head",
            "piglin" | "zombified_piglin" | "piglin_brute" => "minecraft:piglin_head",
            // Never a wither skeleton's skull: one is a wither's makings, worth far more than paper.
            _ => return None,
        };
        Some((t.kind.as_str(), head))
    })
}

/// Where its kill sign stands: two blocks out from the door it was put by, away from the
/// house, as `kill_sign` put it.
pub fn kill_sign_spot(memory: &Memory) -> Option<BlockPos> {
    let home = memory.home.as_ref()?;
    let door = memory
        .landmarks
        .iter()
        .rev()
        .find(|l| l.event == "kill_sign")?
        .pos?;
    let step = step_toward(home.center, door);

    Some([door[0] + step[0] * 2, door[1], door[2] + step[1] * 2])
}

/// Words cut into a sign's lines: whole words, at most fifteen letters a line, four lines,
/// lower case; a word too long for a line is cut.
pub fn sign_lines(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.to_lowercase().split_whitespace() {
        let word: String = word.chars().take(15).collect();
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= 15 => {
                line.push(' ');
                line.push_str(&word);
            }
            _ => {
                if lines.len() == 4 {
                    break;
                }
                lines.push(word);
            }
        }
    }

    lines
}

/// Their lights in the tunnel they stand in, farthest from them first.
pub fn tunnel_lights(memory: &Memory) -> Vec<BlockPos> {
    let Some(tunnel) = memory.tunnel_at_target() else {
        return Vec::new();
    };
    let at = block_of(memory.last.pos);
    let mut lights: Vec<BlockPos> = memory
        .habits
        .cave_lights
        .iter()
        .copied()
        .filter(|l| tunnel.holds(*l))
        .collect();
    lights.sort_by(|a, b| distance(*b, at).total_cmp(&distance(*a, at)));

    lights
}

/// A dig of its own: the block made air if it is natural rock (or air already), never what a
/// player built.
fn dig(ids: &mut impl FnMut() -> u64, pos: BlockPos) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: "minecraft:air".to_string(),
        state: String::new(),
        replace: crate::tunnels::ROCK_IDS
            .iter()
            .map(|r| r.to_string())
            .collect(),
        settle: 0,
        wall: String::new(),
        rest: false,
        only: false,
    }
}

/// A block from nothing, only into air.
fn fill(ids: &mut impl FnMut() -> u64, pos: BlockPos, block: &str) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: block.to_string(),
        state: String::new(),
        replace: vec![],
        settle: 0,
        wall: String::new(),
        rest: false,
        only: false,
    }
}

/// The way they face, as one block along the nearer axis.
fn facing_way(yaw: f32) -> [i32; 2] {
    let yaw = (yaw as f64).to_radians();
    let (x, z) = (-yaw.sin(), yaw.cos());
    if x.abs() >= z.abs() {
        [if x >= 0.0 { 1 } else { -1 }, 0]
    } else {
        [0, if z >= 0.0 { 1 } else { -1 }]
    }
}

/// A light of theirs within ten blocks of the target, the nearest.
pub fn light_near(memory: &Memory) -> Option<BlockPos> {
    let at = block_of(memory.last.pos);
    memory
        .their_lights()
        .into_iter()
        .map(|(p, _)| p)
        .chain(memory.home.iter().flat_map(|h| h.lights.iter().copied()))
        .filter(|p| distance(*p, at) <= 10.0)
        .min_by(|a, b| distance(*a, at).total_cmp(&distance(*b, at)))
}

/// Two blocks behind the target, its feet at theirs (the figure stands one above the spot).
fn behind_close(memory: &Memory) -> BlockPos {
    let at = block_of(memory.last.pos);
    let yaw = (memory.last.yaw as f64).to_radians();
    let back = [yaw.sin(), -yaw.cos()];
    [
        at[0] + (back[0] * 2.0).round() as i32,
        at[1] - 1,
        at[2] + (back[1] * 2.0).round() as i32,
    ]
}

/// The far figure's spot: fifty blocks off, away from the home, on the ground.
fn far_spot(memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    let far = find_far(memory);
    far.or_else(|| {
        let at = block_of(memory.last.pos);
        let angle = rng.next_f64() * std::f64::consts::TAU;
        Some([
            at[0] + (angle.cos() * 50.0) as i32,
            -10_000,
            at[2] + (angle.sin() * 50.0) as i32,
        ])
    })
}

/// Fifty blocks from the target along the way from the home to them, on the ground.
fn find_far(memory: &Memory) -> Option<BlockPos> {
    let at = block_of(memory.last.pos);
    let from = memory.home.as_ref()?.center;
    let (dx, dz) = ((at[0] - from[0]) as f64, (at[2] - from[2]) as f64);
    let length = (dx * dx + dz * dz).sqrt();
    (length >= 1.0).then(|| {
        [
            at[0] + (dx / length * 50.0).round() as i32,
            -10_000,
            at[2] + (dz / length * 50.0).round() as i32,
        ]
    })
}

/// Asleep: the figure in their doorway walks slowly to the bed's side (the host finds it, by the
/// pillow) and, with a lead, chokes them five seconds, or with a crossbow (`shoot`) loads it,
/// aims and shoots them once, and they wake burning; never a kill. Their eyes kept open to see
/// it come (the host's own part of the command, whatever `events.glitches` says), and the night
/// held meanwhile. Their door opens, heard, and is left open: by the
/// command itself, as it begins, so a door gone since is passed over (a toggle of its own that
/// failed stopped the rest of the event).
fn bedside(
    ids: &mut impl FnMut() -> u64,
    memory: &Memory,
    target: u32,
    pos: Option<BlockPos>,
    damage: f32,
    how: &str,
) -> Vec<(Command, u64)> {
    pos.map(|door| {
        let mut shown = figure_shown(
            ids,
            memory,
            target,
            [door[0], door[1] - 1, door[2]],
            20.0,
            false,
        );
        if let Command::Figure { held, .. } = &mut shown {
            *held = ItemStack::default();
        }
        vec![
            (shown, 0),
            (
                Command::Choke {
                    id: ids(),
                    player: target,
                    figure: 1,
                    item: match how {
                        "shoot" => "minecraft:crossbow",
                        "lie" => "",
                        _ => "minecraft:lead",
                    }
                    .to_string(),
                    seconds: if how == "shoot" { 2.0 } else { 5.0 },
                    damage,
                    spare: true,
                    door: Some(door),
                    how: how.to_string(),
                },
                1,
            ),
        ]
    })
    .unwrap_or_default()
}

/// The first of `one_more_each_night`'s figures' numbers.
const ONE_MORE_FIRST: u32 = 31;

/// The most windows `every_window` fills, and the first of their figures' numbers.
const WINDOWS_MOST: usize = 8;
const WINDOWS_FIRST: u32 = 21;

/// The house's windows: its panes within twelve blocks of its middle, one for each run of
/// panes (a pane two blocks or less from one already taken is the same window).
fn windows(memory: &Memory, center: BlockPos) -> Vec<BlockPos> {
    let mut taken: Vec<BlockPos> = Vec::new();
    for pane in memory.fixtures("glass_pane").map(|f| f.pos) {
        if distance(pane, center) <= 12.0 && !taken.iter().any(|t| distance(*t, pane) <= 2.0) {
            taken.push(pane);
        }
    }
    taken
}

/// The top ladder of the shaft they are down: the highest of the ladders in the column within
/// two blocks of them across, when they are six blocks or more under it.
pub fn shaft_top(memory: &Memory) -> Option<BlockPos> {
    let at = block_of(memory.last.pos);
    memory
        .fixtures("ladder")
        .map(|f| f.pos)
        .filter(|p| (p[0] - at[0]).abs() <= 2 && (p[2] - at[2]).abs() <= 2)
        .max_by_key(|p| p[1])
        .filter(|top| top[1] - at[1] >= 6)
}

/// `animal_follows`: a walk after them every eighty seconds, for a day of the game's.
const FOLLOW_EVERY: u64 = 80 * 20;
const FOLLOW_WALKS: u64 = 24_000 / FOLLOW_EVERY;

/// The zombie (a husk or a drowned too) nearest them, for `wears_your_face`.
fn face_wearer(memory: &Memory) -> Option<&crate::memory::Creature> {
    memory
        .last
        .hostiles_near
        .iter()
        .filter(|h| matches!(h.kind.as_str(), "zombie" | "husk" | "drowned"))
        .min_by(|a, b| {
            let d = |c: &crate::memory::Creature| {
                (c.pos[0] - memory.last.pos[0]).powi(2) + (c.pos[2] - memory.last.pos[2]).powi(2)
            };
            d(a).total_cmp(&d(b))
        })
}

/// Food of theirs in a chest, by id: the first eaten thing it holds.
fn chest_food(chest: &crate::memory::Container) -> Option<String> {
    const FOOD: &[&str] = &[
        "bread",
        "apple",
        "carrot",
        "potato",
        "beef",
        "porkchop",
        "chicken",
        "mutton",
        "cod",
        "salmon",
        "cookie",
        "melon_slice",
        "pie",
        "stew",
    ];
    chest
        .contents
        .iter()
        .map(|s| s.id.as_str())
        .find(|id| FOOD.iter().any(|f| id.ends_with(f)))
        .map(str::to_string)
}

/// How long `flowers_wilted` keeps their flowers dead: a day of the game's.
const WILTED_FOR: u64 = 24_000;

/// A block set with a state, only into air: `camp_nearby`'s fire and seats.
fn lantern_state(
    ids: &mut impl FnMut() -> u64,
    pos: BlockPos,
    block: &str,
    state: &str,
) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: block.to_string(),
        state: state.to_string(),
        replace: Vec::new(),
        settle: 0,
        wall: String::new(),
        rest: true,
        only: false,
    }
}

/// A letter of `name_in_the_snow` in a font three wide and five long: the cells trodden,
/// `[x, z]`, z down the letter. Anything it has no shape for is a blank.
pub fn snow_letter(letter: char) -> Vec<[i32; 2]> {
    const FONT: &[(char, [&str; 5])] = &[
        ('A', ["###", "#.#", "###", "#.#", "#.#"]),
        ('B', ["##.", "#.#", "##.", "#.#", "##."]),
        ('C', ["###", "#..", "#..", "#..", "###"]),
        ('D', ["##.", "#.#", "#.#", "#.#", "##."]),
        ('E', ["###", "#..", "##.", "#..", "###"]),
        ('F', ["###", "#..", "##.", "#..", "#.."]),
        ('G', ["###", "#..", "#.#", "#.#", "###"]),
        ('H', ["#.#", "#.#", "###", "#.#", "#.#"]),
        ('I', ["###", ".#.", ".#.", ".#.", "###"]),
        ('J', ["..#", "..#", "..#", "#.#", "###"]),
        ('K', ["#.#", "#.#", "##.", "#.#", "#.#"]),
        ('L', ["#..", "#..", "#..", "#..", "###"]),
        ('M', ["#.#", "###", "#.#", "#.#", "#.#"]),
        ('N', ["##.", "#.#", "#.#", "#.#", "#.#"]),
        ('O', ["###", "#.#", "#.#", "#.#", "###"]),
        ('P', ["###", "#.#", "###", "#..", "#.."]),
        ('Q', ["###", "#.#", "#.#", "###", "..#"]),
        ('R', ["###", "#.#", "##.", "#.#", "#.#"]),
        ('S', ["###", "#..", "###", "..#", "###"]),
        ('T', ["###", ".#.", ".#.", ".#.", ".#."]),
        ('U', ["#.#", "#.#", "#.#", "#.#", "###"]),
        ('V', ["#.#", "#.#", "#.#", "#.#", ".#."]),
        ('W', ["#.#", "#.#", "#.#", "###", "#.#"]),
        ('X', ["#.#", "#.#", ".#.", "#.#", "#.#"]),
        ('Y', ["#.#", "#.#", ".#.", ".#.", ".#."]),
        ('Z', ["###", "..#", ".#.", "#..", "###"]),
        ('0', ["###", "#.#", "#.#", "#.#", "###"]),
        ('1', [".#.", "##.", ".#.", ".#.", "###"]),
        ('2', ["###", "..#", "###", "#..", "###"]),
        ('3', ["###", "..#", "###", "..#", "###"]),
        ('4', ["#.#", "#.#", "###", "..#", "..#"]),
        ('5', ["###", "#..", "###", "..#", "###"]),
        ('6', ["###", "#..", "###", "#.#", "###"]),
        ('7', ["###", "..#", "..#", "..#", "..#"]),
        ('8', ["###", "#.#", "###", "#.#", "###"]),
        ('9', ["###", "#.#", "###", "..#", "###"]),
        ('_', ["...", "...", "...", "...", "###"]),
    ];
    let upper = letter.to_ascii_uppercase();
    FONT.iter()
        .find(|(c, _)| *c == upper)
        .map(|(_, rows)| {
            rows.iter()
                .enumerate()
                .flat_map(|(z, row)| {
                    row.chars()
                        .enumerate()
                        .filter(|(_, c)| *c == '#')
                        .map(move |(x, _)| [x as i32, z as i32])
                })
                .collect()
        })
        .unwrap_or_default()
}

/// How many `breadcrumbs` lays.
const CRUMBS: usize = 6;

/// The most plentiful stack in a chest of something that is a block, with enough for the
/// crumbs: `breadcrumbs`' line.
fn crumb_stack(chest: &crate::memory::Container) -> Option<&ItemStack> {
    chest
        .contents
        .iter()
        .filter(|s| s.count as usize >= CRUMBS && !crate::memory::family(&s.id).is_empty())
        .max_by_key(|s| s.count)
}

/// How far ahead of them `peeks_from_ground`'s head shows.
const PEEK_FROM: f64 = 24.0;

/// `in_a_line`'s figures: the first's number, how far apart along the route, how many at most.
const IN_A_LINE_FIRST: u32 = 41;
const IN_A_LINE_APART: f64 = 8.0;
const IN_A_LINE_MOST: usize = 6;

/// Where `in_a_line`'s figures stand: along their route from the point nearest them, the way
/// that leads from them (toward its far end), one every eight blocks, beginning eight on.
fn in_a_line(memory: &Memory) -> Vec<BlockPos> {
    let Some(route) = memory.route() else {
        return Vec::new();
    };
    let at = block_of(memory.last.pos);
    let points = &route.points;
    let Some(near) = (0..points.len())
        .min_by(|a, b| distance(points[*a], at).total_cmp(&distance(points[*b], at)))
    else {
        return Vec::new();
    };
    // On from them: toward the end farther from where they stand.
    let ahead: Vec<BlockPos> = if distance(points[points.len() - 1], at) >= distance(points[0], at)
    {
        points[near..].to_vec()
    } else {
        points[..=near].iter().rev().copied().collect()
    };
    let mut spots = Vec::new();
    let mut last = at;
    for p in ahead {
        if distance(p, last) >= IN_A_LINE_APART {
            spots.push(p);
            last = p;
            if spots.len() == IN_A_LINE_MOST {
                break;
            }
        }
    }
    spots
}

/// How far ahead of them `crossing_ahead`'s figure crosses, and half its way across.
const CROSSING_AHEAD: f64 = 30.0;
const CROSSING_HALF: f64 = 10.0;

/// How far ahead of them `pet_follows`' figure stands.
const PET_TO: f64 = 9.0;

/// How far ahead of them `falls_from_the_sky`'s figure lands.
const FALLS_AHEAD: f64 = 40.0;

/// How far back `rubberband` sets them, at least.
const RUBBERBAND_BACK: f64 = 3.0;

/// How far ahead of them `hunted`'s figure starts its run.
const HUNTED_FROM: f64 = 36.0;

/// How far out the ring of figures stands from the house's middle.
const RING_BLOCKS: f64 = 20.0;
/// The ring's figures are numbered from here, apart from the one figure (1).
const RING_FIRST: u32 = 11;

/// A figure shown to the target at a spot (the y one below where it stands, or below -1000 for
/// the ground there), for a while.
fn figure_shown(
    ids: &mut impl FnMut() -> u64,
    memory: &Memory,
    target: u32,
    p: BlockPos,
    seconds: f32,
    name_shown: bool,
) -> Command {
    let standing = if p[1] <= -1_000 {
        [p[0] as f64 + 0.5, -10_000.0, p[2] as f64 + 0.5]
    } else {
        [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5]
    };
    Command::Figure {
        id: ids(),
        figure: 1,
        skin_of: target,
        pos: standing,
        path: Vec::new(),
        held: memory
            .favourites
            .first()
            .map(|(id, _)| ItemStack {
                id: id.clone(),
                count: 1,
                slot: 0,
            })
            .unwrap_or_default(),
        name_shown,
        seconds,
        vanish_within: 0.0,
        mirror: false,
        once_seen: false,
        copy_crouch: false,
        swinging: false,
        upside_down: false,
        crawls: false,
        rushes: false,
        rides_behind: false,
        together: 0,
        lies: false,
        falls: false,
        walks: false,
        faces: None,
        sunk: false,
        jumps: false,
        rides: 0,
        eats: false,
    }
}

/// Whether an item is a piece of armour a stand wears.
pub fn is_armour(id: &str) -> bool {
    ["helmet", "chestplate", "leggings", "boots"]
        .iter()
        .any(|k| id.ends_with(k))
}

/// A horse of theirs (or a donkey, a mule, a llama, a camel) tied to a post near.
pub fn leashed_horse(memory: &Memory) -> Option<&crate::memory::Creature> {
    memory.last.creatures_near.iter().find(|c| {
        memory.last.leashed.contains(&c.id)
            && matches!(
                c.kind.as_str(),
                "horse" | "donkey" | "mule" | "llama" | "camel"
            )
    })
}

/// The game's yaw from one point to another: 0 faces +z, 90 faces -x.
fn yaw_toward(from: Pos, to: Pos) -> f32 {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);

    (-dx).atan2(dz).to_degrees() as f32
}

/// The middle of the field, where a scarecrow stands.
fn field_middle(memory: &Memory) -> Option<BlockPos> {
    let rows = memory.field_rows();
    let row = rows.get(rows.len() / 2)?;

    Some(row[row.len() / 2])
}

/// What an alcove may be dug into: natural stone, never dirt, never anything placed.
pub const STONE: &[&str] = &[
    "minecraft:stone",
    "minecraft:deepslate",
    "minecraft:andesite",
    "minecraft:diorite",
    "minecraft:granite",
    "minecraft:tuff",
];

/// What it fills a hole with, from what it dug: the blocks a player's hole held.
pub const FILL: &[&str] = &["minecraft:dirt", "minecraft:cobblestone", "minecraft:stone"];

/// The home's tallest ladder of four rungs or more, bottom to top, if there is one.
pub fn ladder_column(memory: &Memory) -> Option<Vec<BlockPos>> {
    let rungs: Vec<BlockPos> = memory.fixtures("ladder").map(|f| f.pos).collect();
    rungs
        .iter()
        .filter(|r| !rungs.contains(&[r[0], r[1] - 1, r[2]]))
        .map(|bottom| {
            let mut column = vec![*bottom];
            loop {
                let top = column[column.len() - 1];
                let next = [top[0], top[1] + 1, top[2]];
                if !rungs.contains(&next) {
                    break;
                }
                column.push(next);
            }
            column
        })
        .filter(|c| c.len() >= 4)
        .max_by_key(|c| c.len())
}

/// The words a command puts in the world: a sign's lines, a chat line, a name.
pub fn texts(command: &Command) -> Vec<String> {
    match command {
        Command::SetSign { lines, .. } => lines.iter().filter(|l| !l.is_empty()).cloned().collect(),
        Command::GameMessage { text, .. } if !text.is_empty() => vec![text.clone()],
        Command::NameEntity { name, .. } => vec![name.clone()],
        // A book's own line, the first of its first page (the facts after it are the memory's,
        // not words of its own), and a page written in theirs.
        Command::WriteBook { pages, .. } => pages
            .first()
            .and_then(|p| p.split("\n\n").next())
            .filter(|l| !l.is_empty())
            .map(|l| vec![l.to_string()])
            .unwrap_or_default(),
        Command::AddPage { page, .. } if !page.is_empty() => vec![page.clone()],
        _ => vec![],
    }
}

/// A torch near `pos`, as a player would place one: on the ground within `settle` blocks down
/// or up, else on a wall there, never floating. The mind knows positions, not terrain; the
/// host finds the spot (`docs/sdk.md`, `set_block`).
/// The face in a chest's grid of nine across: two eyes in the top row, a mouth of five along
/// the bottom of the first three rows.
pub const FACE: [u32; 7] = [2, 6, 19, 20, 21, 22, 23];

/// The moves that lay a face in a chest (`a_face_in_the_chest`), `(from, to, count)` in order:
/// each stack where the face goes moved whole to an empty slot elsewhere in the first three rows,
/// then one item of their biggest stack into each slot of the face. None when there is no stack
/// of seven or more to take from, or not room enough to clear the face.
pub fn face_moves(contents: &[ItemStack]) -> Option<Vec<(u32, u32, u32)>> {
    let in_grid = |slot: u32| slot < 27;
    let mut occupied: std::collections::BTreeMap<u32, u32> = contents
        .iter()
        .filter(|s| in_grid(s.slot) && s.count > 0)
        .map(|s| (s.slot, s.count))
        .collect();
    let mut source = contents
        .iter()
        .filter(|s| in_grid(s.slot) && s.count >= FACE.len() as u32)
        .max_by_key(|s| (s.count, std::cmp::Reverse(s.slot)))?
        .slot;
    let mut moves = Vec::new();
    for face in FACE {
        let Some(count) = occupied.get(&face).copied() else {
            continue;
        };
        let empty = (0..27).find(|s| !FACE.contains(s) && !occupied.contains_key(s))?;
        moves.push((face, empty, count));
        occupied.remove(&face);
        occupied.insert(empty, count);
        if source == face {
            source = empty;
        }
    }
    for face in FACE {
        moves.push((source, face, 1));
    }

    Some(moves)
}

/// A block set on a floor spot, only into air or over one of `replace`: its lanterns and their
/// going (`its_lights_on`).
fn lantern(ids: &mut impl FnMut() -> u64, pos: BlockPos, block: &str, replace: &[&str]) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: block.to_string(),
        state: String::new(),
        replace: replace.iter().map(|r| r.to_string()).collect(),
        settle: 0,
        wall: String::new(),
        rest: false,
        only: !replace.is_empty(),
    }
}

fn torch(ids: &mut impl FnMut() -> u64, pos: BlockPos, settle: u8) -> Command {
    Command::SetBlock {
        id: ids(),
        pos,
        block: "minecraft:torch".to_string(),
        state: String::new(),
        replace: vec![],
        settle,
        wall: "minecraft:wall_torch".to_string(),
        rest: false,
        only: false,
    }
}

/// Natural ground the lodger may wear, dig or build on; never anything a player builds with.
const GROUND: &[&str] = &[
    "minecraft:grass_block",
    "minecraft:dirt",
    "minecraft:coarse_dirt",
    "minecraft:podzol",
    "minecraft:stone",
    "minecraft:gravel",
    "minecraft:sand",
    "minecraft:snow",
    "minecraft:short_grass",
    "minecraft:tall_grass",
    "minecraft:fern",
];

/// The ground itself, not what lies on it (snow, grass, ferns): taking one of these out is
/// digging.
const SOLID_GROUND: &[&str] = &[
    "minecraft:grass_block",
    "minecraft:dirt",
    "minecraft:coarse_dirt",
    "minecraft:podzol",
    "minecraft:stone",
    "minecraft:gravel",
    "minecraft:sand",
];

/// A step of one block from `from` toward `to`, along the longer axis.
fn step_toward(from: BlockPos, to: BlockPos) -> [i32; 2] {
    let dx = to[0] - from[0];
    let dz = to[2] - from[2];
    if dx.abs() >= dz.abs() {
        [dx.signum(), 0]
    } else {
        [0, dz.signum()]
    }
}

/// Which way an event's toggle goes: open or lit, shut or out, or (none) the other way from now.
/// `close_door` on a door left shut used to open it.
fn toggle_to(event: &str) -> Option<bool> {
    match event {
        "open_door" | "open_gate" | "light_campfire" | "wake" | "door_opens_itself"
        | "village_doors" => Some(true),
        "close_door" | "put_out_campfire" => Some(false),
        _ => None,
    }
}

/// What a ripe crop cleared gives: the crop's own item.
pub fn harvest_of(block: &str) -> Option<&'static str> {
    match block {
        "minecraft:wheat" => Some("minecraft:wheat"),
        "minecraft:carrots" => Some("minecraft:carrot"),
        "minecraft:potatoes" => Some("minecraft:potato"),
        "minecraft:beetroots" => Some("minecraft:beetroot"),
        _ => None,
    }
}

/// A torch's or a lantern's soul-fire kind, as it hangs; none for one already soul or another.
fn soul_light(id: &str) -> Option<String> {
    let name = id.rsplit(':').next().unwrap_or(id);
    let soul = match name {
        "torch" => "soul_torch",
        "wall_torch" => "soul_wall_torch",
        "lantern" => "soul_lantern",
        _ => return None,
    };
    Some(format!("minecraft:{soul}"))
}

/// The other house, a copy of theirs in the void dimension, a minute or so (`other_house`).
pub fn other_house(
    id: u64,
    player: u32,
    center: BlockPos,
    bed: BlockPos,
    door: BlockPos,
    seconds: f32,
    comes: bool,
) -> Command {
    Command::OtherHouse {
        id,
        player,
        center,
        radius: 12,
        bed,
        door,
        seconds,
        comes,
    }
}

/// The moves that sort a container by item id into the first slots, each as (from, to, count),
/// using one empty slot as the hand. None when there is no empty slot or it is sorted already.
pub fn tidy_moves(contents: &[ItemStack], size: u32) -> Vec<(u32, u32, u32)> {
    let mut slots: Vec<Option<ItemStack>> = vec![None; size as usize];
    for stack in contents {
        if (stack.slot as usize) < slots.len() {
            slots[stack.slot as usize] = Some(stack.clone());
        }
    }
    // What the slots hold, not the list as given: two stacks said to share a slot, or a slot
    // past the container, counted twice ran the target past the last slot and panicked
    //, which would take the whole mind down.
    let mut wanted: Vec<ItemStack> = slots.iter().flatten().cloned().collect();
    wanted.sort_by(|a, b| a.id.cmp(&b.id).then(b.count.cmp(&a.count)));

    let mut moves = Vec::new();
    for (target, stack) in wanted.iter().enumerate() {
        if target >= slots.len() {
            break;
        }
        // Only from the slots not yet sorted: the slots before `target` are done, and two equal
        // stacks found there sent a sorted chest's stack out of place.
        let from = slots[target..]
            .iter()
            .position(|s| {
                s.as_ref()
                    .map(|s| s.id == stack.id && s.count == stack.count)
                    .unwrap_or(false)
            })
            .map(|i| (i + target) as u32);
        let target = target as u32;
        let Some(from) = from else { continue };
        if from == target {
            continue;
        }
        if slots[target as usize].is_some() {
            let Some(empty) = slots.iter().position(|s| s.is_none()) else {
                return Vec::new();
            };
            let moved = slots[target as usize].take().unwrap();
            moves.push((target, empty as u32, moved.count));
            slots[empty] = Some(moved);
        }
        let moved = slots[from as usize].take().unwrap();
        moves.push((from, target, moved.count));
        slots[target as usize] = Some(moved);
    }

    moves
}

/// The commands for the event at `pos`, each with the ticks to wait before sending it (0 for
/// now). `ids` hands out command ids.
pub fn commands(
    event: &Event,
    pos: Option<BlockPos>,
    memory: &Memory,
    ids: &mut impl FnMut() -> u64,
    now: &Now,
    written: Option<&str>,
) -> Vec<(Command, u64)> {
    let target = memory.target.unwrap_or(0);
    let at = block_center;
    let place_seed = pos
        .map(|p| (p[0] + p[2]).unsigned_abs() as usize)
        .unwrap_or(0);
    let seed = place_seed + memory.done.len();
    // A sound event plays one of its ways to sound, the next after the last: from the house's
    // door or chest, from far off, or around the target as it moves.
    if let Some(ways) = super::sounds::variants(event.id) {
        // A house's sound with no house to come from is not made at all.
        if pos.is_none() && super::sounds::HOUSE_SOUNDS.contains(&event.id) {
            return vec![];
        }
        let spot = match event.action {
            Action::SoundFar(_) => {
                let angle = (seed % 8) as f64 * std::f64::consts::FRAC_PI_4;
                Some([
                    memory.last.pos[0] + 36.0 * angle.cos(),
                    memory.last.pos[1],
                    memory.last.pos[2] + 36.0 * angle.sin(),
                ])
            }
            _ => pos.map(block_center),
        };
        // The next after the last from one place: the seed of the place alone, since the
        // count of all things done moved between two plays and could land on the same way.
        let way = super::sounds::pick(ways, place_seed, memory.times(event.id));
        return super::sounds::commands(way, spot, memory, target, seed, ids);
    }
    // The words: what the model wrote for this event, checked, or the bank's.
    let words = |kind: Kind, mob: &str, count: u32| -> String {
        written
            .map(|w| w.to_string())
            .unwrap_or_else(|| text::pick_fresh(kind, seed, mob, count, &memory.lines_written))
    };

    match event.action {
        Action::Wait => vec![],
        // Its sounds are its ways to sound, above.
        Action::ChestAnswers => vec![],
        // Armed by the brain (`Brain::arm_loot`), given by the host.
        Action::LootGift => vec![],
        Action::ToggleDoor | Action::ToggleFixture(_) => pos
            .map(|p| {
                vec![(
                    Command::ToggleBlock {
                        id: ids(),
                        pos: p,
                        to: toggle_to(event.id),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Their candles, as their lights know them: the fixtures never held a candle.
        Action::ToggleCandles => memory
            .walls
            .iter()
            .filter(|(_, id)| id.contains("candle") && !id.contains("cake"))
            .map(|(p, _)| {
                (
                    Command::ToggleBlock {
                        id: ids(),
                        pos: *p,
                        to: toggle_to(event.id),
                    },
                    0,
                )
            })
            .collect(),
        Action::Sound(sound) | Action::SoundAtFixture(_, sound) => pos
            .map(|p| vec![(sound_at(ids, target, sound, at(p), 1.0), 0)])
            .unwrap_or_default(),
        Action::SoundAtTarget(sound) => {
            vec![(sound_at(ids, target, sound, memory.last.pos, 0.8), 10)]
        }
        Action::SoundFar(sound) => {
            let angle = (seed % 8) as f64 * std::f64::consts::FRAC_PI_4;
            let far = [
                memory.last.pos[0] + 36.0 * angle.cos(),
                memory.last.pos[1],
                memory.last.pos[2] + 36.0 * angle.sin(),
            ];
            vec![(sound_at(ids, target, sound, far, 2.0), 0)]
        }
        Action::Taps(_, sound, count) => pos
            .map(|p| {
                (0..count)
                    .map(|i| (sound_at(ids, target, sound, at(p), 0.9), i as u64 * 10))
                    .collect()
            })
            .unwrap_or_default(),
        Action::OpenLid => pos
            .map(|p| {
                vec![(
                    Command::OpenLid {
                        id: ids(),
                        pos: p,
                        seconds: 1.5,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::MoveLight => pos
            .map(|p| {
                // A step of one to three blocks along one axis, same height.
                let steps = [
                    [2, 0, 0],
                    [-2, 0, 0],
                    [0, 0, 2],
                    [0, 0, -2],
                    [3, 0, 0],
                    [0, 0, -3],
                ];
                let step = steps[(p[0] + p[2]).rem_euclid(steps.len() as i32) as usize];
                vec![(
                    Command::MoveBlock {
                        id: ids(),
                        from: p,
                        to: [p[0] + step[0], p[1], p[2] + step[2]],
                        around: 0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::AddTorch => pos
            .map(|p| vec![(torch(ids, [p[0], p[1] + 1, p[2]], 2), 0)])
            .unwrap_or_default(),
        Action::LightPath => {
            let Some(home) = memory.home.as_ref() else {
                return vec![];
            };
            let Some(spot) = memory
                .place
                .or_else(|| memory.route_end())
                .or_else(|| Some([home.center[0] + 10, home.floor_y, home.center[2] + 10]))
            else {
                return vec![];
            };
            let count = 3 + (seed % 5) as i32;
            let mut made = Vec::new();
            let mut p = home.center;
            for i in 0..count {
                let step = step_toward(p, spot);
                p = [p[0] + step[0] * 3, p[1], p[2] + step[1] * 3];
                // Footing looked for eight blocks up and down: the ground falls away toward water.
                made.push((torch(ids, [p[0], p[1] + 1, p[2]], 8), i as u64 * 20));
            }
            made
        }
        Action::ShuffleContainer => match (pos, memory.used_container()) {
            (Some(p), Some(container)) if !container.contents.is_empty() => {
                match container.empty_slots().first() {
                    Some(&to_slot) => {
                        let stack = &container.contents[seed % container.contents.len()];
                        vec![(
                            Command::MoveItem {
                                id: ids(),
                                from: p,
                                from_slot: stack.slot,
                                to: p,
                                to_slot,
                                count: stack.count,
                            },
                            0,
                        )]
                    }
                    None => vec![],
                }
            }
            _ => vec![],
        },
        Action::SplitStack => match (pos, memory.used_container()) {
            (Some(p), Some(container)) => {
                match (
                    container.contents.iter().find(|s| s.count >= 2),
                    container.empty_slots().first(),
                ) {
                    (Some(stack), Some(&to_slot)) => vec![(
                        Command::MoveItem {
                            id: ids(),
                            from: p,
                            from_slot: stack.slot,
                            to: p,
                            to_slot,
                            count: stack.count / 2,
                        },
                        0,
                    )],
                    _ => vec![],
                }
            }
            _ => vec![],
        },
        Action::TidyContainer => match (pos, memory.used_container()) {
            (Some(p), Some(container)) => tidy_moves(&container.contents, 27)
                .into_iter()
                .enumerate()
                .map(|(i, (from_slot, to_slot, count))| {
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: p,
                            from_slot,
                            to: p,
                            to_slot,
                            count,
                        },
                        i as u64,
                    )
                })
                .collect(),
            _ => vec![],
        },
        Action::SwapStacks => match (memory.used_container(), memory.other_container()) {
            (Some(a), Some(b)) => {
                let (Some(sa), Some(sb)) = (a.contents.first(), b.contents.first()) else {
                    return vec![];
                };
                let (Some(&ea), Some(&eb)) = (a.empty_slots().first(), b.empty_slots().first())
                else {
                    return vec![];
                };
                vec![
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: a.pos,
                            from_slot: sa.slot,
                            to: b.pos,
                            to_slot: eb,
                            count: sa.count,
                        },
                        0,
                    ),
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: b.pos,
                            from_slot: sb.slot,
                            to: a.pos,
                            to_slot: ea,
                            count: sb.count,
                        },
                        1,
                    ),
                ]
            }
            _ => vec![],
        },
        Action::FillGap => match (memory.used_container(), memory.other_container()) {
            (Some(a), Some(b)) => {
                let Some(biggest) = a
                    .contents
                    .iter()
                    .filter(|s| s.count >= 2)
                    .max_by_key(|s| s.count)
                else {
                    return vec![];
                };
                let Some(&empty) = b.empty_slots().first() else {
                    return vec![];
                };
                vec![(
                    Command::MoveItem {
                        id: ids(),
                        from: a.pos,
                        from_slot: biggest.slot,
                        to: b.pos,
                        to_slot: empty,
                        count: biggest.count / 2,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::CookOne => {
            let (Some(furnace), Some((chest, raw)), Some((_, burn))) = (
                memory.furnace(),
                memory.in_used_container(smeltable),
                memory.in_used_container(fuel),
            ) else {
                return vec![];
            };
            let mut made = vec![(
                Command::MoveItem {
                    id: ids(),
                    from: chest.pos,
                    from_slot: raw.slot,
                    to: furnace.pos,
                    to_slot: 0,
                    count: 1,
                },
                0,
            )];
            if !furnace.contents.iter().any(|s| s.slot == 1) {
                made.push((
                    Command::MoveItem {
                        id: ids(),
                        from: chest.pos,
                        from_slot: burn.slot,
                        to: furnace.pos,
                        to_slot: 1,
                        count: 1,
                    },
                    1,
                ));
            }
            made
        }
        Action::TakeOutput => match (memory.furnace(), memory.used_container()) {
            (Some(furnace), Some(chest)) => {
                let (Some(out), Some(&empty)) = (
                    furnace.contents.iter().find(|s| s.slot == 2),
                    chest.empty_slots().first(),
                ) else {
                    return vec![];
                };
                vec![(
                    Command::MoveItem {
                        id: ids(),
                        from: furnace.pos,
                        from_slot: 2,
                        to: chest.pos,
                        to_slot: empty,
                        count: out.count,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::ItemOnTable => match (pos, memory.favourite_in_container()) {
            (Some(table), Some((container, stack))) => vec![
                (
                    Command::TakeItem {
                        id: ids(),
                        from: container.pos,
                        slot: stack.slot,
                        expect: stack.id.clone(),
                        count: 1,
                        keep: "item_on_table".to_string(),
                    },
                    0,
                ),
                (
                    Command::DropItem {
                        id: ids(),
                        pos: [
                            table[0] as f64 + 0.5,
                            table[1] as f64 + 1.1,
                            table[2] as f64 + 0.5,
                        ],
                        item: ItemStack {
                            id: stack.id.clone(),
                            count: 1,
                            slot: 0,
                        },
                        kept: "item_on_table".to_string(),
                    },
                    1,
                ),
            ],
            _ => vec![],
        },
        Action::EggOnBed => pos
            .map(|bed| {
                vec![(
                    Command::DropItem {
                        id: ids(),
                        pos: [
                            bed[0] as f64 + 0.5,
                            bed[1] as f64 + 0.7,
                            bed[2] as f64 + 0.5,
                        ],
                        item: ItemStack {
                            id: "minecraft:egg".to_string(),
                            count: 1,
                            slot: 0,
                        },
                        kept: String::new(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::Take => match memory.favourite_in_container() {
            Some((container, stack)) => vec![(
                Command::TakeItem {
                    id: ids(),
                    from: container.pos,
                    slot: stack.slot,
                    expect: stack.id.clone(),
                    count: 1,
                    keep: "held".to_string(),
                },
                0,
            )],
            None => vec![],
        },
        Action::ReturnTaken => match &memory.held {
            Some(held) => vec![(
                Command::PutItem {
                    id: ids(),
                    into: held.from,
                    slot: Some(held.slot),
                    item: held.item.clone(),
                    kept: "held".to_string(),
                    wear: 0,
                    name: String::new(),
                },
                0,
            )],
            None => vec![],
        },
        // What it took, given back a little used: a tool worn some more, anything else named.
        Action::ReturnUsed => match &memory.held {
            Some(held) => {
                let tool = [
                    "_pickaxe", "_axe", "_sword", "_shovel", "_hoe", "bow", "shears",
                ]
                .iter()
                .any(|t| held.item.id.ends_with(t));
                vec![(
                    Command::PutItem {
                        id: ids(),
                        into: held.from,
                        slot: Some(held.slot),
                        item: held.item.clone(),
                        kept: "held".to_string(),
                        wear: if tool { 4 + (seed % 9) as u32 } else { 0 },
                        name: if tool {
                            String::new()
                        } else {
                            words(Kind::ItemName, "", 0)
                        },
                    },
                    0,
                )]
            }
            None => vec![],
        },
        // Grown first, then the bone meal taken: a sapling that does not grow costs nothing.
        Action::TreeGrown => match (
            pos,
            memory.in_used_container(|id| id == "minecraft:bone_meal"),
        ) {
            (Some(p), Some((chest, meal))) => vec![
                (
                    Command::Grow {
                        id: ids(),
                        pos: p,
                        tries: 6,
                    },
                    0,
                ),
                (
                    Command::TakeItem {
                        id: ids(),
                        from: chest.pos,
                        slot: meal.slot,
                        expect: meal.id.clone(),
                        count: meal.count.min(3),
                        keep: String::new(),
                    },
                    2,
                ),
            ],
            _ => vec![],
        },
        // Their holes by the house filled, newest first, with what it dug and holds: five at
        // most, never more than it holds.
        Action::Backfill => {
            let mut stock: Vec<(&str, u32)> = FILL.iter().map(|b| (*b, memory.stored(b))).collect();
            memory
                .holes
                .iter()
                .rev()
                .take(5)
                .enumerate()
                .filter_map(|(i, (at, _))| {
                    let (block, left) = stock.iter_mut().find(|(_, n)| *n > 0)?;
                    *left -= 1;
                    Some((
                        Command::SetBlock {
                            id: ids(),
                            pos: *at,
                            block: block.to_string(),
                            state: String::new(),
                            replace: GROUND_COVER.iter().map(|r| r.to_string()).collect(),
                            settle: 0,
                            wall: String::new(),
                            rest: false,
                            only: false,
                        },
                        i as u64 * 8,
                    ))
                })
                .collect()
        }
        // One block of natural stone behind the wall, away from the house: a torch in it, and
        // what it holds of theirs if anything. The one dig beside what they built (owner), and only into stone.
        Action::Alcove => match (pos, memory.home.as_ref()) {
            (Some(wall), Some(home)) => {
                let (dx, dz) = (wall[0] - home.center[0], wall[2] - home.center[2]);
                let out = if dx.abs() >= dz.abs() {
                    [dx.signum(), 0]
                } else {
                    [0, dz.signum()]
                };
                let hollow = [wall[0] + out[0], wall[1], wall[2] + out[1]];
                let mut made = vec![
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: hollow,
                            block: "minecraft:air".to_string(),
                            state: String::new(),
                            replace: STONE.iter().map(|b| b.to_string()).collect(),
                            settle: 0,
                            wall: String::new(),
                            rest: false,
                            only: true,
                        },
                        0,
                    ),
                    (torch(ids, hollow, 0), 5),
                ];
                if let Some(held) = &memory.held {
                    made.push((
                        Command::DropItem {
                            id: ids(),
                            pos: block_center(hollow),
                            item: held.item.clone(),
                            kept: "held".to_string(),
                        },
                        8,
                    ));
                }
                made
            }
            _ => vec![],
        },
        Action::ShiftFurniture => pos
            .map(|p| {
                let way = [[1, 0], [-1, 0], [0, 1], [0, -1]][seed % 4];
                vec![(
                    Command::MoveBlock {
                        id: ids(),
                        from: p,
                        to: [p[0] + way[0], p[1], p[2] + way[1]],
                        around: 1,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Each torch within five blocks of the one picked taken down, and at once put in their
        // chest: a failure stops the rest, and no torch is ever lost between the two.
        // As many torches as their chest has room for: one put out with no room for it was lost.
        Action::RoomDark => match (pos, memory.used_container(), memory.home.as_ref()) {
            (Some(spot), Some(chest), Some(home)) => home
                .lights
                .iter()
                .filter(|l| distance(**l, spot) <= 5.0)
                .take(6.min(chest.empty_slots().len()))
                .enumerate()
                .flat_map(|(i, light)| {
                    [
                        (
                            Command::SetBlock {
                                id: ids(),
                                pos: *light,
                                block: "minecraft:air".to_string(),
                                state: String::new(),
                                replace: vec![
                                    "minecraft:torch".to_string(),
                                    "minecraft:wall_torch".to_string(),
                                ],
                                settle: 0,
                                wall: String::new(),
                                rest: false,
                                only: true,
                            },
                            i as u64 * 2,
                        ),
                        (
                            Command::PutItem {
                                id: ids(),
                                into: chest.pos,
                                slot: None,
                                item: ItemStack {
                                    id: "minecraft:torch".to_string(),
                                    count: 1,
                                    slot: 0,
                                },
                                kept: String::new(),
                                wear: 0,
                                name: String::new(),
                            },
                            i as u64 * 2 + 1,
                        ),
                    ]
                })
                .collect(),
            _ => vec![],
        },
        // The door they just opened shuts again, a moment later.
        Action::DoorHeld => pos
            .map(|door| {
                vec![(
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                        to: toggle_to(event.id),
                    },
                    3,
                )]
            })
            .unwrap_or_default(),
        Action::BedTurned => pos
            .map(|bed| {
                vec![(
                    Command::SetState {
                        id: ids(),
                        pos: bed,
                        property: "facing".to_string(),
                        value: "@turn".to_string(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::DoorHinge => pos
            .map(|door| {
                vec![(
                    Command::SetState {
                        id: ids(),
                        pos: door,
                        property: "hinge".to_string(),
                        value: "@next".to_string(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // A rung from the middle, set on top of the ladder: the climb has a gap, and the rung
        // is where they will find it.
        Action::LadderGap => match ladder_column(memory) {
            Some(column) => {
                let rung = column[column.len() / 2];
                let top = column[column.len() - 1];
                vec![(
                    Command::MoveBlock {
                        id: ids(),
                        from: rung,
                        to: [top[0], top[1] + 1, top[2]],
                        around: 0,
                    },
                    0,
                )]
            }
            None => vec![],
        },
        // Each flower picked (it holds it then) and set in an empty pot of the home: three at
        // most.
        Action::FlowersPotted => {
            let pots: Vec<BlockPos> = memory.fixtures("flower_pot").map(|f| f.pos).collect();
            memory
                .plants
                .iter()
                .filter(|(id, _)| crate::memory::is_flower(id))
                .zip(pots)
                .take(3)
                .enumerate()
                .flat_map(|(i, ((flower, at), pot))| {
                    let name = flower.rsplit(':').next().unwrap_or(flower);
                    // The pot filled first, the flower taken after: a pot filled since it was
                    // seen fails first, and the flower stays (taken first, it was lost).
                    let mut pot = set(
                        ids,
                        pot,
                        &format!("minecraft:potted_{name}"),
                        &["minecraft:flower_pot"],
                    );
                    if let Command::SetBlock { only, .. } = &mut pot {
                        *only = true;
                    }
                    [
                        (pot, i as u64 * 10),
                        (
                            set(ids, *at, "minecraft:air", &[flower.as_str()]),
                            i as u64 * 10 + 4,
                        ),
                    ]
                })
                .collect()
        }
        // Carved where it grows, its face to the nearest door, their shears a little worn.
        Action::PumpkinFace => match (pos, memory.in_used_container(|id| id == "minecraft:shears"))
        {
            (Some(p), Some((chest, shears))) => {
                let door = memory
                    .home
                    .as_ref()
                    .and_then(|h| {
                        h.doors
                            .iter()
                            .min_by_key(|d| (d[0] - p[0]).abs() + (d[2] - p[2]).abs())
                            .copied()
                    })
                    .unwrap_or([p[0], p[1], p[2] + 1]);
                let (dx, dz) = (door[0] - p[0], door[2] - p[2]);
                let facing = if dx.abs() >= dz.abs() {
                    if dx >= 0 {
                        "east"
                    } else {
                        "west"
                    }
                } else if dz >= 0 {
                    "south"
                } else {
                    "north"
                };
                vec![
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: p,
                            block: "minecraft:carved_pumpkin".to_string(),
                            state: format!("facing={facing}"),
                            replace: vec!["minecraft:pumpkin".to_string()],
                            settle: 0,
                            wall: String::new(),
                            rest: false,
                            only: true,
                        },
                        0,
                    ),
                    (
                        Command::DamageItem {
                            id: ids(),
                            pos: chest.pos,
                            slot: shears.slot,
                            amount: 1,
                        },
                        2,
                    ),
                ]
            }
            _ => vec![],
        },
        // One dirt in every empty slot, as many as it dug: never one it does not hold.
        Action::FullOfOne => match (pos, memory.used_container()) {
            (Some(p), Some(chest)) => chest
                .empty_slots()
                .into_iter()
                .take(memory.stored("minecraft:dirt") as usize)
                .enumerate()
                .map(|(i, slot)| {
                    (
                        Command::PutItem {
                            id: ids(),
                            into: p,
                            slot: Some(slot),
                            item: ItemStack {
                                id: "minecraft:dirt".to_string(),
                                count: 1,
                                slot,
                            },
                            kept: String::new(),
                            wear: 0,
                            name: String::new(),
                        },
                        i as u64,
                    )
                })
                .collect(),
            _ => vec![],
        },
        // What waits in the hopper goes into the chest beside or under it that is theirs; the
        // mind never sees inside a hopper, so the host moves whatever is there.
        Action::HopperEmptied => pos
            .and_then(|hopper| {
                memory
                    .containers
                    .iter()
                    .filter(|c| c.theirs() && !c.is_furnace())
                    .find(|c| {
                        let d = [
                            c.pos[0] - hopper[0],
                            c.pos[1] - hopper[1],
                            c.pos[2] - hopper[2],
                        ];
                        d[0].abs() + d[1].abs() + d[2].abs() == 1
                    })
                    .map(|chest| {
                        // The slots that hold something, as last seen: an empty one asked for
                        // failed the event and dropped the rest. Never looked in, one move (the
                        // host takes the first slot that is not empty).
                        let mut slots: Vec<u32> = memory
                            .containers
                            .iter()
                            .find(|c| c.pos == hopper)
                            .map(|c| {
                                c.contents
                                    .iter()
                                    .filter(|s| !s.id.is_empty() && s.count > 0)
                                    .map(|s| s.slot)
                                    .filter(|s| *s < 5)
                                    .collect()
                            })
                            .unwrap_or_default();
                        if slots.is_empty() {
                            slots.push(0);
                        }
                        slots
                            .into_iter()
                            .map(|slot| {
                                (
                                    Command::MoveItem {
                                        id: ids(),
                                        from: hopper,
                                        from_slot: slot,
                                        to: chest.pos,
                                        to_slot: 0,
                                        count: 64,
                                    },
                                    slot as u64,
                                )
                            })
                            .collect()
                    })
            })
            .unwrap_or_default(),
        // Across the room: three to five blocks the seed's way, wherever there is room and
        // ground, its contents with it.
        Action::ShulkerMoved => pos
            .map(|p| {
                let way = [[1, 0], [-1, 0], [0, 1], [0, -1]][seed % 4];
                let reach = 3 + (seed % 3) as i32;
                vec![(
                    Command::MoveBlock {
                        id: ids(),
                        from: p,
                        to: [p[0] + way[0] * reach, p[1], p[2] + way[1] * reach],
                        around: 2,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::BookWord => match memory.in_used_container(|id| id == "minecraft:writable_book") {
            Some((container, book)) => {
                let word = words(Kind::SignWord, "", 0);
                vec![(
                    Command::SignBook {
                        id: ids(),
                        pos: container.pos,
                        slot: book.slot,
                        title: word.clone(),
                        pages: vec![word],
                    },
                    0,
                )]
            }
            None => vec![],
        },
        // Its place, or the home's middle if it has none yet.
        Action::MapMarked => match memory.in_used_container(|id| id == "minecraft:filled_map") {
            Some((container, map)) => {
                let at = memory
                    .place
                    .or_else(|| memory.home.as_ref().map(|h| h.center));
                at.map(|at| {
                    vec![(
                        Command::MarkMap {
                            id: ids(),
                            pos: container.pos,
                            slot: map.slot,
                            at,
                        },
                        0,
                    )]
                })
                .unwrap_or_default()
            }
            None => vec![],
        },
        Action::RenameOne => match (pos, memory.used_container()) {
            (Some(p), Some(container)) if !container.contents.is_empty() => {
                // Their favourite if it is in there, else whatever the seed lands on.
                let favourite = memory.favourites.first().map(|(id, _)| id.as_str());
                let stack = container
                    .contents
                    .iter()
                    .find(|s| Some(s.id.as_str()) == favourite)
                    .unwrap_or(&container.contents[seed % container.contents.len()]);
                vec![(
                    Command::RenameItem {
                        id: ids(),
                        pos: p,
                        slot: stack.slot,
                        name: words(Kind::ItemName, "", 0),
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::PlaceTaken => match (&memory.held, pos) {
            (Some(held), Some(p)) => vec![(
                Command::DropItem {
                    id: ids(),
                    pos: [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5],
                    item: held.item.clone(),
                    kept: "held".to_string(),
                },
                0,
            )],
            _ => vec![],
        },
        Action::Hint(kind, seconds) => {
            let detail = match kind {
                "hotbar_slip" => memory
                    .favourites
                    .first()
                    .map(|(id, _)| id.clone())
                    .unwrap_or_default(),
                "name_called" => memory.target_name.clone(),
                _ => String::new(),
            };
            let hint_pos = match kind {
                "camera_nudge" => memory
                    .place
                    .or_else(|| memory.home.as_ref().map(|h| h.center)),
                "ripples" | "painting_flip" | "compass_points" => pos,
                // A window of the house: its first pane.
                "handprints" => memory
                    .home
                    .as_ref()
                    .and_then(|h| windows(memory, h.center).first().copied()),
                // Four blocks behind them, where the subtitle's arrow points.
                "name_called" => Some(behind(memory, 4.0)),
                _ => None,
            };
            vec![(
                Command::Hint {
                    id: ids(),
                    player: target,
                    hint: kind.to_string(),
                    pos: hint_pos,
                    seconds,
                    detail,
                },
                0,
            )]
        }
        Action::RefuseRest => vec![(
            Command::CancelSleep {
                id: ids(),
                player: target,
                seconds: 120.0,
            },
            0,
        )],
        Action::Wake => {
            let mut made = vec![(
                Command::CancelSleep {
                    id: ids(),
                    player: target,
                    seconds: 0.0,
                },
                0,
            )];
            if let Some(door) = pos {
                made.push((
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                        to: toggle_to(event.id),
                    },
                    5,
                ));
            }
            made
        }
        // Held still while they look: a look alone fought the animals' own walking, and they
        // jittered (the owner).
        Action::Stare => memory
            .creatures("")
            .take(8)
            .map(|creature| {
                (
                    Command::Hold {
                        id: ids(),
                        at: None,
                        entity: creature.id,
                        player: target,
                        seconds: 10.0 + (seed % 21) as f32,
                        creep: false,
                    },
                    0,
                )
            })
            .collect(),
        Action::Hole => pos
            .map(|p| {
                let mut made: Vec<(Command, u64)> = (0..3)
                    .map(|depth| {
                        (
                            set(ids, [p[0], p[1] - depth, p[2]], "minecraft:air", GROUND),
                            depth as u64 * 15,
                        )
                    })
                    .collect();
                made.push((torch(ids, [p[0], p[1] - 2, p[2]], 1), 50));
                made
            })
            .unwrap_or_default(),
        Action::Cairn => pos
            .map(|p| {
                (0..3)
                    .map(|height| {
                        (
                            on_ground(
                                ids,
                                [p[0], p[1] + 1 + height, p[2]],
                                "minecraft:cobblestone",
                            ),
                            height as u64 * 15,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Action::Sapling => match (pos, memory.in_used_container(|id| id.ends_with("sapling"))) {
            // Planted first, on the ground within three blocks, and only then taken from the
            // chest: a planting that fails drops the take, so no sapling of theirs is lost.
            (Some(p), Some((container, stack))) => vec![
                (
                    Command::SetBlock {
                        id: ids(),
                        pos: [p[0], p[1] + 1, p[2]],
                        block: stack.id.clone(),
                        state: String::new(),
                        replace: GROUND_COVER.iter().map(|r| r.to_string()).collect(),
                        settle: 3,
                        wall: String::new(),
                        rest: false,
                        only: false,
                    },
                    0,
                ),
                (
                    Command::TakeItem {
                        id: ids(),
                        from: container.pos,
                        slot: stack.slot,
                        expect: stack.id.clone(),
                        count: 1,
                        keep: String::new(),
                    },
                    2,
                ),
            ],
            _ => vec![],
        },
        Action::Sign => pos
            .map(|p| {
                let word = words(Kind::SignWord, "", 0);
                sign_at(ids, [p[0], p[1] + 1, p[2]], vec![String::new(), word])
            })
            .unwrap_or_default(),
        Action::KillSign => match (pos, memory.most_killed()) {
            (Some(door), Some(tally)) => {
                let Some(home) = memory.home.as_ref() else {
                    return vec![];
                };
                // Two blocks out from the door, away from the house.
                let step = step_toward(home.center, door);
                let p = [door[0] + step[0] * 2, door[1], door[2] + step[1] * 2];
                sign_at(
                    ids,
                    p,
                    vec![words(Kind::KillSign, &tally.kind, tally.kills)],
                )
            }
            _ => vec![],
        },
        Action::Grave => match (pos, memory.most_killed()) {
            (Some(p), Some(tally)) => {
                let mut made = vec![
                    (
                        on_ground(ids, [p[0], p[1] + 1, p[2]], "minecraft:cobblestone"),
                        0,
                    ),
                    (
                        on_ground(ids, [p[0], p[1] + 2, p[2]], "minecraft:cobblestone"),
                        15,
                    ),
                ];
                made.extend(
                    sign_at(
                        ids,
                        [p[0] + 1, p[1] + 1, p[2]],
                        vec![words(Kind::GraveSign, &tally.kind, tally.kills)],
                    )
                    .into_iter()
                    .map(|(c, t)| (c, t + 30)),
                );
                made
            }
            _ => vec![],
        },
        Action::PathWorn => match (pos, memory.home.as_ref()) {
            (Some(door), Some(home)) => {
                let spot = memory.place.unwrap_or([
                    home.center[0] + 10,
                    home.floor_y,
                    home.center[2] + 10,
                ]);
                let mut p = [door[0], door[1] - 1, door[2]];
                let step = step_toward(home.center, door);
                p = [p[0] + step[0], p[1], p[2] + step[1]];
                let count = 6 + (seed % 5) as i32;
                let mut made = Vec::new();
                for i in 0..count {
                    let step = step_toward(p, [spot[0], p[1], spot[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                    made.push((
                        set(ids, p, "minecraft:dirt_path", &["minecraft:grass_block"]),
                        i as u64 * 10,
                    ));
                }
                made
            }
            _ => vec![],
        },
        Action::CarpetShift => {
            let carpets: Vec<BlockPos> = memory.fixtures("carpet").map(|f| f.pos).collect();
            let mut sorted = carpets.clone();
            // The farthest along x moves first, so none lands on another.
            sorted.sort_by_key(|p| std::cmp::Reverse(p[0]));
            sorted
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    (
                        Command::MoveBlock {
                            id: ids(),
                            from: p,
                            to: [p[0] + 1, p[1], p[2]],
                            around: 0,
                        },
                        i as u64,
                    )
                })
                .collect()
        }
        // Full that was empty, or empty that was full: the host's toggle of a cauldron (it was
        // filled and emptied a tick apart, or failed on a full one).
        Action::Cauldron => pos
            .map(|p| {
                vec![(
                    Command::ToggleBlock {
                        id: ids(),
                        pos: p,
                        to: toggle_to(event.id),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::RotateFrame => pos
            .and_then(|p| memory.decor("item_frame").find(|d| d.pos == p))
            .map(|frame| {
                vec![(
                    Command::RotateFrame {
                        id: ids(),
                        entity: frame.id,
                        rotation: None,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::FramesTurn => {
            let rotation = (seed % 8) as u32;
            unseen_decor(memory, "item_frame")
                .map(|frame| {
                    (
                        Command::RotateFrame {
                            id: ids(),
                            entity: frame.id,
                            rotation: Some(rotation),
                        },
                        0,
                    )
                })
                .collect()
        }
        Action::SwapPainting => pos
            .and_then(|p| memory.decor("painting").find(|d| d.pos == p))
            .map(|painting| {
                vec![(
                    Command::SwapPainting {
                        id: ids(),
                        entity: painting.id,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::StandTurn => {
            let Some(stand) = unseen_decor(memory, "armor_stand").next() else {
                return vec![];
            };
            let look = memory
                .home
                .as_ref()
                .and_then(|h| h.doors.first().copied().or(h.bed))
                .map(at)
                .unwrap_or(memory.last.pos);
            vec![(
                Command::PoseStand {
                    id: ids(),
                    entity: stand.id,
                    at: Some(look),
                    follow: None,
                    seconds: 0.0,
                },
                0,
            )]
        }
        Action::StandWatch => memory
            .decor("armor_stand")
            .map(|stand| {
                (
                    Command::PoseStand {
                        id: ids(),
                        entity: stand.id,
                        at: None,
                        follow: Some(target),
                        seconds: 90.0,
                    },
                    0,
                )
            })
            .collect(),
        Action::NameAnimal => their_animal(memory, "", true)
            .map(|c| {
                vec![(
                    Command::NameEntity {
                        id: ids(),
                        entity: c.id,
                        name: words(Kind::AnimalName, "", 0),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::DyeSheep => {
            const COLOURS: &[&str] = &["red", "black", "light_blue", "purple", "lime", "pink"];
            their_animal(memory, "sheep", false)
                .map(|sheep| {
                    vec![(
                        Command::DyeSheep {
                            id: ids(),
                            entity: sheep.id,
                            colour: COLOURS[seed % COLOURS.len()].to_string(),
                        },
                        0,
                    )]
                })
                .unwrap_or_default()
        }
        Action::PetGrowl => memory
            .pets()
            .next()
            .map(|pet| {
                let sound = if pet.kind == "cat" {
                    "minecraft:entity.cat.hiss"
                } else {
                    "minecraft:entity.wolf.growl"
                };
                vec![
                    (sound_at(ids, target, sound, pet.pos, 1.0), 0),
                    (sound_at(ids, target, sound, pet.pos, 1.0), 25),
                ]
            })
            .unwrap_or_default(),
        Action::MoveCreature(kind, _) => {
            let creature = if kind.is_empty() {
                if event.id.starts_with("pet_") {
                    // A pet at home, out of their view: one at their side vanished from it.
                    let home = memory.home.as_ref().map(|h| h.center);
                    memory.pets().find(|c| {
                        home.is_some_and(|h| {
                            distance(block_of(c.pos), h) <= (crate::memory::HOME_RADIUS * 2) as f64
                        }) && !memory.in_view(block_of(c.pos), ANIMAL_VIEW)
                    })
                } else {
                    their_animal(memory, "", false)
                }
            } else {
                memory.creatures(kind).next()
            };
            match (creature, pos) {
                (Some(c), Some(p)) => {
                    let mut made = vec![(
                        Command::MoveEntity {
                            id: ids(),
                            entity: c.id,
                            to: [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5],
                        },
                        0,
                    )];
                    if let Some(home) = memory.home.as_ref() {
                        made.push((
                            Command::LookAt {
                                id: ids(),
                                entity: c.id,
                                at: at(home.center),
                                seconds: 600.0,
                            },
                            1,
                        ));
                    }
                    // The villager back where it stood by morning, as its row says.
                    if event.id == "villager_at_door" {
                        made.push((
                            Command::MoveEntity {
                                id: ids(),
                                entity: c.id,
                                to: c.pos,
                            },
                            12_000,
                        ));
                    }
                    made
                }
                _ => vec![],
            }
        }
        Action::GolemTurned => {
            match (memory.creatures("iron_golem").next(), memory.home.as_ref()) {
                (Some(golem), Some(home)) => vec![(
                    Command::LookAt {
                        id: ids(),
                        entity: golem.id,
                        at: at(home.center),
                        seconds: 1200.0,
                    },
                    0,
                )],
                _ => vec![],
            }
        }
        Action::TabTwice => vec![(
            Command::PlayerList {
                id: ids(),
                player: target,
                seconds: 6.0,
            },
            0,
        )],
        Action::TabEntry => vec![(
            Command::PlayerList {
                id: ids(),
                player: target,
                // `tab_entry_stays`: the whole session, as its row says (it shared the minute).
                seconds: if event.id == "tab_entry_stays" {
                    6.0 * 3600.0
                } else {
                    60.0
                },
            },
            0,
        )],
        Action::Message(kind) => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: kind.to_string(),
                text: String::new(),
            },
            0,
        )],
        Action::Says => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: "says".to_string(),
                text: words(Kind::ChatLine, "", 0),
            },
            0,
        )],
        Action::SaysKills => memory
            .most_killed()
            .map(|tally| {
                vec![(
                    Command::GameMessage {
                        id: ids(),
                        player: target,
                        message: "says".to_string(),
                        text: words(Kind::KillLine, &tally.kind, tally.kills),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::Whisper => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: "whisper".to_string(),
                text: words(Kind::Whisper, "", 0),
            },
            0,
        )],
        Action::MoveVehicle => {
            // Their boat, never a minecart of their mine moved into the lake.
            let (Some(vehicle), Some(water)) = (memory.their_boat(), memory.feature("water"))
            else {
                return vec![];
            };
            // Adrift: the water's middle. Moved: across it, the same distance past the middle.
            let to = if event.id == "boat_adrift" {
                [
                    water.pos[0] as f64 + 0.5,
                    water.pos[1] as f64 + 0.6,
                    water.pos[2] as f64 + 0.5,
                ]
            } else {
                [
                    2.0 * water.pos[0] as f64 - vehicle.pos[0],
                    water.pos[1] as f64 + 0.6,
                    2.0 * water.pos[2] as f64 - vehicle.pos[2],
                ]
            };
            vec![(
                Command::MoveEntity {
                    id: ids(),
                    entity: vehicle.id,
                    to,
                },
                0,
            )]
        }
        Action::PushVehicle(kind) => memory
            .vehicles
            .iter()
            .find(|v| v.kind.contains(kind) || (kind == "boat" && v.kind.contains("raft")))
            .map(|vehicle| {
                // Away from the target, slowly: a drift, not a shove.
                let dx = vehicle.pos[0] - memory.last.pos[0];
                let dz = vehicle.pos[2] - memory.last.pos[2];
                let length = (dx * dx + dz * dz).sqrt().max(1.0);
                let speed = if kind == "boat" { 0.12 } else { 0.3 };
                vec![(
                    Command::PushEntity {
                        id: ids(),
                        entity: vehicle.id,
                        velocity: [dx / length * speed, 0.0, dz / length * speed],
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::BucketFloor => match (
            pos,
            memory.in_used_container(|id| id == "minecraft:water_bucket"),
        ) {
            // The water set first, then the full bucket taken and an empty one put back: water
            // that cannot go down drops the rest, so their bucket stays full.
            (Some(p), Some((container, stack))) => vec![
                (set(ids, p, "minecraft:water", GROUND), 0),
                (
                    Command::TakeItem {
                        id: ids(),
                        from: container.pos,
                        slot: stack.slot,
                        expect: stack.id.clone(),
                        count: 1,
                        keep: String::new(),
                    },
                    2,
                ),
                (
                    Command::PutItem {
                        id: ids(),
                        into: container.pos,
                        slot: Some(stack.slot),
                        item: ItemStack {
                            id: "minecraft:bucket".to_string(),
                            count: 1,
                            slot: 0,
                        },
                        kept: String::new(),
                        wear: 0,
                        name: String::new(),
                    },
                    3,
                ),
            ],
            _ => vec![],
        },
        Action::SnowCleared => match (pos, memory.home.as_ref()) {
            (Some(door), Some(home)) => {
                let spot = memory.place.unwrap_or([
                    home.center[0] + 10,
                    home.floor_y,
                    home.center[2] + 10,
                ]);
                let mut p = door;
                let mut made = Vec::new();
                for i in 0..8 {
                    let step = step_toward(p, [spot[0], p[1], spot[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                    made.push((
                        set(ids, p, "minecraft:air", &["minecraft:snow"]),
                        i as u64 * 8,
                    ));
                }
                made
            }
            _ => vec![],
        },
        Action::Trample => match (pos, memory.feature("crops")) {
            (Some(door), Some(field)) => {
                let mut p = [field.pos[0], field.pos[1] - 1, field.pos[2]];
                let mut made = Vec::new();
                for i in 0..5 {
                    made.push((
                        set(ids, p, "minecraft:dirt", &["minecraft:farmland"]),
                        i as u64 * 10,
                    ));
                    let step = step_toward(p, [door[0], p[1], door[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                }
                made
            }
            _ => vec![],
        },
        // One ripe crop of any kind cleared; what it was goes in their chest, as the host says it
        // replaced (the brain's report handling, `harvest_of`): wheat, a carrot, a potato, a
        // beetroot. A seedling is never cut.
        Action::HarvestOne => match (pos, memory.used_container()) {
            (Some(field), Some(chest)) if !chest.empty_slots().is_empty() => {
                let dx = (seed % 5) as i32 - 2;
                let dz = ((seed / 5) % 5) as i32 - 2;
                vec![(
                    set(
                        ids,
                        [field[0] + dx, field[1], field[2] + dz],
                        "minecraft:air",
                        &[
                            "minecraft:wheat[age=7]",
                            "minecraft:carrots[age=7]",
                            "minecraft:potatoes[age=7]",
                            "minecraft:beetroots[age=3]",
                        ],
                    ),
                    0,
                )]
            }
            _ => vec![],
        },
        // Its diary of them, signed with their name, on sale from the villager near them for an
        // emerald, once.
        Action::DiarySold => match memory.villager_near() {
            Some(v) => {
                let book = ids();
                let kept = format!("book_{book}");
                vec![
                    (
                        Command::WriteBook {
                            id: book,
                            kept: kept.clone(),
                            title: crate::books::signed(memory),
                            author: crate::books::signed(memory),
                            pages: crate::books::diary(memory, &words(Kind::DiaryLine, "", 0)),
                        },
                        0,
                    ),
                    (
                        Command::OfferTrade {
                            id: ids(),
                            entity: v.id,
                            kept,
                        },
                        1,
                    ),
                ]
            }
            None => vec![],
        },
        // Not bought: the villager walks up, slowly, and drops a newer one at their feet.
        Action::DiaryBrought => match memory.villager_near() {
            Some(v) => {
                let book = ids();
                let kept = format!("book_{book}");
                vec![
                    (
                        Command::WriteBook {
                            id: book,
                            kept: kept.clone(),
                            title: crate::books::signed(memory),
                            author: crate::books::signed(memory),
                            pages: crate::books::diary(memory, &words(Kind::DiaryLine, "", 0)),
                        },
                        0,
                    ),
                    (
                        Command::Approach {
                            id: ids(),
                            entities: vec![v.id],
                            player: target,
                            stop: 1.8,
                            seconds: 60.0,
                            pace: Some(0.6),
                            then: "give".to_string(),
                            damage: 0.0,
                            kept,
                            to: None,
                        },
                        1,
                    ),
                ]
            }
            None => vec![],
        },
        // A note in its words, signed for them, into their ender chest: the one place nothing
        // else can reach.
        Action::EnderChestNote => {
            let book = ids();
            let kept = format!("book_{book}");
            vec![
                (
                    Command::WriteBook {
                        id: book,
                        kept: kept.clone(),
                        title: format!("for {}", crate::books::signed(memory)),
                        author: String::new(),
                        pages: vec![words(Kind::DiaryLine, "", 0)],
                    },
                    0,
                ),
                (
                    Command::PutEnderChest {
                        id: ids(),
                        player: target,
                        kept,
                    },
                    1,
                ),
            ]
        }
        // The next eye they throw, within ten minutes, flies toward their house.
        Action::EyeTurnsHome => pos
            .map(|home| {
                vec![(
                    Command::EyeHome {
                        id: ids(),
                        player: target,
                        to: [
                            home[0] as f64 + 0.5,
                            home[1] as f64 + 1.0,
                            home[2] as f64 + 0.5,
                        ],
                        seconds: 600.0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // A letter for them in their chest, in its words.
        Action::LetterInChest => match memory.used_container() {
            Some(chest) if !chest.empty_slots().is_empty() => {
                let book = ids();
                let kept = format!("book_{book}");
                vec![
                    (
                        Command::WriteBook {
                            id: book,
                            kept: kept.clone(),
                            title: format!("for {}", crate::books::signed(memory)),
                            author: String::new(),
                            pages: vec![words(Kind::DiaryLine, "", 0)],
                        },
                        0,
                    ),
                    (
                        Command::PutItem {
                            id: ids(),
                            into: chest.pos,
                            slot: chest.empty_slots().first().copied(),
                            item: ItemStack {
                                id: "minecraft:written_book".to_string(),
                                count: 1,
                                slot: 0,
                            },
                            kept,
                            wear: 0,
                            name: String::new(),
                        },
                        1,
                    ),
                ]
            }
            _ => vec![],
        },
        // A villager with no name yet: one the player named keeps theirs.
        Action::VillagerNamed => memory
            .unnamed_villager_near()
            .map(|v| {
                vec![(
                    Command::NameEntity {
                        id: ids(),
                        entity: v.id,
                        name: words(Kind::AnimalName, "", 0),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Its notes on a lectern that holds no book, a page a day.
        Action::LecternNotes => pos
            .map(|lectern| {
                let book = ids();
                let kept = format!("book_{book}");
                vec![
                    (
                        Command::WriteBook {
                            id: book,
                            kept: kept.clone(),
                            title: "notes".to_string(),
                            author: String::new(),
                            pages: crate::books::notes(memory, &words(Kind::DiaryLine, "", 0)),
                        },
                        0,
                    ),
                    (
                        Command::PutBook {
                            id: ids(),
                            pos: lectern,
                            kept,
                            // Its other lecterns out of their view, should this one hold a book:
                            // the host looked within eight blocks, at any, one before them too.
                            also: memory
                                .fixtures("lectern")
                                .map(|f| f.pos)
                                .filter(|p| *p != lectern && !memory.in_view(*p, ANIMAL_VIEW))
                                .collect(),
                        },
                        1,
                    ),
                ]
            })
            .unwrap_or_default(),
        // One page more in a book and quill of theirs, after their own.
        Action::PageInBook => memory
            .in_used_container(|id| id == "minecraft:writable_book")
            .map(|(chest, book)| {
                vec![(
                    Command::AddPage {
                        id: ids(),
                        pos: chest.pos,
                        slot: book.slot,
                        page: words(Kind::DiaryLine, "", 0),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::MiningSounds => {
            let angle = (seed % 8) as f64 * std::f64::consts::FRAC_PI_4;
            let at = [
                memory.last.pos[0] + 9.0 * angle.cos(),
                memory.last.pos[1] - 1.0,
                memory.last.pos[2] + 9.0 * angle.sin(),
            ];
            (0..6)
                .map(|i| {
                    (
                        sound_at(ids, target, "minecraft:block.stone.break", at, 0.6),
                        i as u64 * 14 + 10,
                    )
                })
                .collect()
        }
        Action::PlateLaid => match (
            pos,
            memory.in_used_container(|id| id.ends_with("pressure_plate")),
        ) {
            (Some(chest), Some((container, stack))) => {
                let step = memory
                    .home
                    .as_ref()
                    .map(|h| step_toward(chest, h.center))
                    .filter(|s| *s != [0, 0])
                    .unwrap_or([1, 0]);
                // Laid first, taken after: a plate that cannot go down drops the take.
                vec![
                    (
                        set(
                            ids,
                            [chest[0] + step[0], chest[1], chest[2] + step[1]],
                            &stack.id,
                            &[],
                        ),
                        0,
                    ),
                    (
                        Command::TakeItem {
                            id: ids(),
                            from: container.pos,
                            slot: stack.slot,
                            expect: stack.id.clone(),
                            count: 1,
                            keep: String::new(),
                        },
                        2,
                    ),
                ]
            }
            _ => vec![],
        },
        Action::DoorAhead => pos
            .map(|door| {
                vec![(
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                        to: toggle_to(event.id),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Each door once: the read holds both halves of every door, and toggling the upper after
        // the lower shut it again.
        Action::VillageDoors => memory
            .village_doors()
            .into_iter()
            .take(6)
            .enumerate()
            .map(|(i, door)| {
                (
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                        to: toggle_to(event.id),
                    },
                    i as u64 * 20,
                )
            })
            .collect(),
        Action::ScanVillage => memory
            .feature("village")
            .map(|village| {
                vec![(
                    Command::Scan {
                        step: 0,
                        dark: false,
                        most: 0,
                        id: ids(),
                        min: [village.pos[0] - 19, village.pos[1] - 8, village.pos[2] - 19],
                        max: [
                            village.pos[0] + 19,
                            village.pos[1] + 15,
                            village.pos[2] + 19,
                        ],
                        kinds: vec![
                            "minecraft:oak_door".to_string(),
                            "minecraft:spruce_door".to_string(),
                            "minecraft:birch_door".to_string(),
                            "minecraft:acacia_door".to_string(),
                            "minecraft:jungle_door".to_string(),
                            "minecraft:dark_oak_door".to_string(),
                            "minecraft:bell".to_string(),
                        ],
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::RotateFixture(_) => pos
            .map(|p| vec![(Command::RotateBlock { id: ids(), pos: p }, 0)])
            .unwrap_or_default(),
        Action::SoundPair(_, first, second) => pos
            .map(|p| {
                vec![
                    (sound_at(ids, target, first, at(p), 0.8), 0),
                    (sound_at(ids, target, second, at(p), 0.8), 12),
                ]
            })
            .unwrap_or_default(),
        // The frame's own item, out of it and onto the bed, the very stack (the host drops it
        // there; until now the frame's was deleted and a stand-in laid on the bed).
        Action::FrameEmptied => match (
            unseen_decor(memory, "item_frame").next(),
            memory.home.as_ref().and_then(|h| h.bed),
        ) {
            (Some(frame), Some(bed)) => vec![(
                Command::SetFrameItem {
                    id: ids(),
                    from: None,
                    slot: 0,
                    entity: frame.id,
                    item: ItemStack::default(),
                    swap: false,
                    drop_at: Some([
                        bed[0] as f64 + 0.5,
                        bed[1] as f64 + 0.7,
                        bed[2] as f64 + 0.5,
                    ]),
                },
                0,
            )],
            _ => vec![],
        },
        // A thing of one from their chest into the frame, the frame's into that slot: the very
        // stacks traded (until now the frame's was lost and the chest's made anew).
        Action::FrameSwap => match (
            unseen_decor(memory, "item_frame").next(),
            memory.used_container(),
        ) {
            (Some(frame), Some(chest)) => {
                let Some(stack) = chest.contents.iter().find(|s| s.count == 1) else {
                    return vec![];
                };
                vec![(
                    Command::SetFrameItem {
                        id: ids(),
                        from: Some(chest.pos),
                        slot: stack.slot,
                        entity: frame.id,
                        item: ItemStack::default(),
                        swap: true,
                        drop_at: None,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::StandDress => {
            let armour = |id: &str| {
                ["_helmet", "_chestplate", "_leggings", "_boots"]
                    .iter()
                    .any(|a| id.ends_with(a))
            };
            match (
                unseen_decor(memory, "armor_stand").next(),
                memory.in_used_container(armour),
            ) {
                // The very piece, kept between the chest and the stand, its enchantments and
                // name with it (until now a new one of its id was made, and those lost).
                (Some(stand), Some((chest, stack))) => vec![
                    (
                        Command::TakeItem {
                            id: ids(),
                            from: chest.pos,
                            slot: stack.slot,
                            expect: stack.id.clone(),
                            count: 1,
                            keep: "stand_dress".to_string(),
                        },
                        0,
                    ),
                    (
                        Command::EquipEntity {
                            id: ids(),
                            kept: "stand_dress".to_string(),
                            entity: stand.id,
                            item: ItemStack {
                                id: stack.id.clone(),
                                count: 1,
                                slot: 0,
                            },
                        },
                        1,
                    ),
                ],
                _ => vec![],
            }
        }
        Action::ToolUsed => memory
            .tool_in_container()
            .map(|(chest, stack)| {
                vec![(
                    Command::DamageItem {
                        id: ids(),
                        pos: chest.pos,
                        slot: stack.slot,
                        amount: 20 + (seed % 30) as u32,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::ColdFurnace => match (memory.furnace(), memory.used_container()) {
            (Some(furnace), Some(chest)) => {
                let (Some(fuel), Some(&empty)) = (
                    furnace.contents.iter().find(|s| s.slot == 1),
                    chest.empty_slots().first(),
                ) else {
                    return vec![];
                };
                vec![(
                    Command::MoveItem {
                        id: ids(),
                        from: furnace.pos,
                        from_slot: 1,
                        to: chest.pos,
                        to_slot: empty,
                        count: fuel.count,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::OpenOnEnter => pos
            .map(|p| {
                vec![(
                    Command::OpenLid {
                        id: ids(),
                        pos: p,
                        seconds: 8.0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::TidyByColour => match (memory.used_container(), pos) {
            (Some(container), _) => {
                // Sorted by the colour word first, so wool and concrete of one colour sit together.
                let mut by_colour = container.contents.clone();
                for stack in &mut by_colour {
                    if crate::memory::is_coloured(&stack.id) {
                        let name = stack.id.rsplit(':').next().unwrap_or(&stack.id).to_string();
                        let (colour, kind) = name.rsplit_once('_').unwrap_or((&name, ""));
                        stack.id = format!("0{colour}_{kind}");
                    }
                }
                tidy_moves(&by_colour, 27)
                    .into_iter()
                    .enumerate()
                    .map(|(i, (from_slot, to_slot, count))| {
                        (
                            Command::MoveItem {
                                id: ids(),
                                from: container.pos,
                                from_slot,
                                to: container.pos,
                                to_slot,
                                count,
                            },
                            i as u64,
                        )
                    })
                    .collect()
            }
            _ => vec![],
        },
        Action::PlantFlower => {
            let flower = |id: &str| {
                [
                    "poppy",
                    "dandelion",
                    "tulip",
                    "orchid",
                    "allium",
                    "daisy",
                    "cornflower",
                    "lily_of_the_valley",
                ]
                .iter()
                .any(|f| id.ends_with(f))
            };
            match (pos, memory.in_used_container(flower), memory.home.as_ref()) {
                (Some(door), Some((chest, stack)), Some(home)) => {
                    let step = step_toward(home.center, door);
                    let p = [
                        door[0] + step[0] * 2 + step[1],
                        door[1],
                        door[2] + step[1] * 2 + step[0],
                    ];
                    // Planted first, taken after: a flower that cannot grow there (gravel, a
                    // path) drops the take, so none of theirs is lost.
                    vec![
                        (set(ids, p, &stack.id, &["minecraft:short_grass"]), 0),
                        (
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: stack.slot,
                                expect: stack.id.clone(),
                                count: 1,
                                keep: String::new(),
                            },
                            2,
                        ),
                    ]
                }
                _ => vec![],
            }
        }
        Action::ReplantOther => {
            let seeds = |id: &str| {
                [":carrot", ":potato", ":beetroot_seeds"]
                    .iter()
                    .any(|s| id.ends_with(s))
            };
            match (pos, memory.in_used_container(seeds)) {
                (Some(field), Some((chest, stack))) => {
                    let crop = if stack.id.ends_with(":carrot") {
                        "minecraft:carrots"
                    } else if stack.id.ends_with(":potato") {
                        "minecraft:potatoes"
                    } else {
                        "minecraft:beetroots"
                    };
                    // Planted first, taken after: a planting that fails drops the take, so no
                    // seed of theirs is lost (until now it was taken first).
                    // Young wheat only: a ripe one replaced was their harvest gone.
                    vec![
                        (
                            set(
                                ids,
                                field,
                                crop,
                                &[
                                    "minecraft:wheat[age=0]",
                                    "minecraft:wheat[age=1]",
                                    "minecraft:wheat[age=2]",
                                    "minecraft:wheat[age=3]",
                                ],
                            ),
                            0,
                        ),
                        (
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: stack.slot,
                                expect: stack.id.clone(),
                                count: 1,
                                keep: String::new(),
                            },
                            2,
                        ),
                    ]
                }
                _ => vec![],
            }
        }
        Action::IcePath => match (memory.feature("water"), memory.home.as_ref()) {
            (Some(water), Some(home)) => {
                let mut p = water.pos;
                let mut made = Vec::new();
                for i in 0..6 {
                    // On the water's surface wherever it is within two blocks, never on land.
                    made.push((
                        Command::SetBlock {
                            id: ids(),
                            pos: p,
                            block: "minecraft:ice".to_string(),
                            state: String::new(),
                            replace: vec!["minecraft:water".to_string()],
                            settle: 2,
                            wall: String::new(),
                            rest: false,
                            only: true,
                        },
                        i as u64 * 10,
                    ));
                    let step = step_toward(p, [home.center[0], p[1], home.center[2]]);
                    if step == [0, 0] {
                        break;
                    }
                    p = [p[0] + step[0], p[1], p[2] + step[1]];
                }
                made
            }
            _ => vec![],
        },
        Action::BedOccupied => vec![
            (
                Command::GameMessage {
                    id: ids(),
                    player: target,
                    message: "bed_occupied".to_string(),
                    text: String::new(),
                },
                0,
            ),
            (
                Command::CancelSleep {
                    id: ids(),
                    player: target,
                    seconds: 0.0,
                },
                1,
            ),
        ],
        Action::OtherHouse => match (pos, memory.home.as_ref(), memory.usual_door()) {
            (Some(bed), Some(home), Some(door)) => vec![(
                other_house(ids(), target, home.center, bed, door, 60.0, false),
                0,
            )],
            _ => vec![],
        },
        Action::WokeElsewhere => pos
            .map(|bed| {
                vec![
                    (
                        Command::CancelSleep {
                            id: ids(),
                            player: target,
                            seconds: 0.0,
                        },
                        0,
                    ),
                    (
                        Command::MovePlayer {
                            id: ids(),
                            player: target,
                            to: [
                                bed[0] as f64 + 1.5,
                                bed[1] as f64 + 0.6,
                                bed[2] as f64 + 0.5,
                            ],
                        },
                        2,
                    ),
                ]
            })
            .unwrap_or_default(),
        Action::ScanOre => {
            let at = block_of(memory.last.pos);
            vec![(
                Command::Scan {
                    step: 0,
                    dark: false,
                    most: 0,
                    id: ids(),
                    min: [at[0] - 12, at[1] - 6, at[2] - 12],
                    max: [at[0] + 12, at[1] + 6, at[2] + 12],
                    kinds: [
                        "coal", "iron", "copper", "gold", "redstone", "lapis", "diamond", "emerald",
                    ]
                    .iter()
                    .flat_map(|ore| {
                        [
                            format!("minecraft:{ore}_ore"),
                            format!("minecraft:deepslate_{ore}_ore"),
                        ]
                    })
                    .collect(),
                },
                0,
            )]
        }
        Action::OreMarked => pos
            .map(|ore| vec![(torch(ids, [ore[0], ore[1] + 1, ore[2]], 1), 0)])
            .unwrap_or_default(),
        // Their lantern, set down far in the dark, then taken from the chest: a lantern that
        // finds no floor stays theirs (a failed step drops the rest).
        Action::LanternAhead => match (pos, memory.in_used_container(|id| id.ends_with("lantern")))
        {
            (Some(spot), Some((chest, stack))) => vec![
                (
                    Command::SetBlock {
                        id: ids(),
                        pos: spot,
                        block: stack.id.clone(),
                        state: String::new(),
                        replace: vec![],
                        settle: 4,
                        wall: String::new(),
                        rest: true,
                        only: false,
                    },
                    0,
                ),
                (
                    Command::TakeItem {
                        id: ids(),
                        from: chest.pos,
                        slot: stack.slot,
                        expect: stack.id.clone(),
                        count: 1,
                        keep: String::new(),
                    },
                    10,
                ),
            ],
            _ => vec![],
        },
        // Tonight's share of a build, from what it gathered first (the brain spends the store);
        // its house's chest takes what it holds once the ground is known.
        Action::Build(event) => match memory.next_build(event) {
            Some((site, _, parts)) => {
                let chest = parts
                    .iter()
                    .find(|p| p.block == "minecraft:chest")
                    .filter(|_| site[1] > -1_000 && memory.held.is_some())
                    .map(|p| [site[0] + p.at[0], site[1] + p.at[1], site[2] + p.at[2]]);
                let mut made = vec![(
                    Command::PlaceBlueprint {
                        id: ids(),
                        at: site,
                        parts: parts
                            .iter()
                            .map(|p| craftkit::BlueprintPart {
                                at: p.at,
                                block: p.block.to_string(),
                                state: p.state.clone(),
                            })
                            .collect(),
                        replace: crate::building::replaceable(event),
                    },
                    0,
                )];
                if let Some(chest) = chest {
                    made.push((
                        Command::PutItem {
                            id: ids(),
                            into: chest,
                            slot: None,
                            item: memory
                                .held
                                .as_ref()
                                .map(|h| h.item.clone())
                                .unwrap_or_default(),
                            kept: "held".to_string(),
                            wear: 0,
                            name: String::new(),
                        },
                        20,
                    ));
                }
                made
            }
            None => vec![],
        },
        // A room of three by three by three dug where the cave is, lit, a door at its way in,
        // and a chest holding what it took.
        Action::CaveRoom => pos
            .map(|c| {
                let mut made = Vec::new();
                let mut t = 0;
                for dy in 0..3 {
                    for dx in -1..=1 {
                        for dz in -1..=1 {
                            made.push((dig(ids, [c[0] + dx, c[1] + dy, c[2] + dz]), t));
                            t += 6;
                        }
                    }
                }
                made.push((torch(ids, [c[0] + 1, c[1], c[2] + 1], 0), t + 5));
                made.push((
                    Command::SetBlock {
                        id: ids(),
                        pos: [c[0] - 1, c[1], c[2] - 1],
                        block: "minecraft:chest".to_string(),
                        state: String::new(),
                        replace: vec![],
                        settle: 0,
                        wall: String::new(),
                        rest: true,
                        only: false,
                    },
                    t + 10,
                ));
                if let Some(held) = &memory.held {
                    made.push((
                        Command::PutItem {
                            id: ids(),
                            into: [c[0] - 1, c[1], c[2] - 1],
                            slot: None,
                            item: held.item.clone(),
                            kept: "held".to_string(),
                            wear: 0,
                            name: String::new(),
                        },
                        t + 20,
                    ));
                }
                made.push((
                    Command::PlaceBlueprint {
                        id: ids(),
                        at: [c[0], c[1] - 1, c[2] + 2],
                        parts: vec![
                            craftkit::BlueprintPart {
                                at: [0, 1, 0],
                                block: "minecraft:oak_door".to_string(),
                                state: "facing=south,half=lower".to_string(),
                            },
                            craftkit::BlueprintPart {
                                at: [0, 2, 0],
                                block: "minecraft:oak_door".to_string(),
                                state: "facing=south,half=upper".to_string(),
                            },
                        ],
                        replace: crate::tunnels::ROCK_IDS
                            .iter()
                            .map(|r| r.to_string())
                            .collect(),
                    },
                    t + 30,
                ));
                made
            })
            .unwrap_or_default(),
        // The figure walks to the rock ahead and digs in, the stone closing behind; through the
        // hill and out its far side a minute later with `through`. All on their client.
        Action::DigInto(through) => pos
            .map(|from| {
                vec![(
                    Command::DigIn {
                        id: ids(),
                        player: target,
                        figure: 1,
                        from,
                        way: facing_way(memory.last.yaw),
                        through,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Their view turned two degrees, as by a hand, and a step on the ground right behind.
        Action::HandOnShoulder => {
            let at = memory.last.pos;
            let yaw = (memory.last.yaw as f64).to_radians();
            let (back, right) = ([yaw.sin(), -yaw.cos()], [yaw.cos(), yaw.sin()]);
            let side = if seed.is_multiple_of(2) { 1.0 } else { -1.0 };
            // Toward their shoulder: a point ahead and to one side, so the turn is small.
            let toward = [
                (at[0] - back[0] * 8.0 + right[0] * side * 2.0).floor() as i32,
                at[1].floor() as i32,
                (at[2] - back[1] * 8.0 + right[1] * side * 2.0).floor() as i32,
            ];
            vec![
                (
                    Command::BlockSound {
                        id: ids(),
                        player: target,
                        block: "under".to_string(),
                        of: "step".to_string(),
                        pos: [at[0] + back[0] * 0.8, at[1], at[2] + back[1] * 0.8],
                        volume: 0.9,
                        pitch: 0.9,
                    },
                    0,
                ),
                (
                    Command::Hint {
                        id: ids(),
                        player: target,
                        hint: "camera_nudge".to_string(),
                        pos: Some(toward),
                        seconds: 1.0,
                        detail: String::new(),
                    },
                    2,
                ),
            ]
        }
        // A second set of steps a moment behind theirs, for most of a minute.
        Action::Footsteps => vec![(
            Command::Footsteps {
                id: ids(),
                player: target,
                seconds: 45.0,
                behind: 3.0,
                delay: 6,
            },
            0,
        )],
        // At the edge of their light, a step back out of it, the light flickering as it goes.
        Action::IntoTheDark => pos
            .map(|p| {
                let at = block_of(memory.last.pos);
                let away = step_toward(at, [p[0], at[1], p[2]]);
                let mut shown = figure_shown(ids, memory, target, p, 2.5, false);
                if let Command::Figure { path, pos, .. } = &mut shown {
                    *path = vec![
                        *pos,
                        [pos[0] + away[0] as f64, pos[1], pos[2] + away[1] as f64],
                    ];
                }
                vec![
                    (shown, 0),
                    (
                        Command::Hint {
                            id: ids(),
                            player: target,
                            hint: "flicker".to_string(),
                            pos: None,
                            seconds: 1.5,
                            detail: String::new(),
                        },
                        10,
                    ),
                ]
            })
            .unwrap_or_default(),
        // Right behind them for a second, it strikes once: by the harm stages, never a kill.
        Action::FigureSwing(damage) => pos
            .map(|p| {
                vec![
                    (figure_shown(ids, memory, target, p, 1.2, false), 0),
                    (
                        Command::FigureSwing {
                            id: ids(),
                            player: target,
                            figure: 1,
                            damage,
                            spare: true,
                        },
                        8,
                    ),
                ]
            })
            .unwrap_or_default(),
        // The far figure, its name shown, raises its arm once and is gone.
        // Round the house, twenty blocks out, on the ground, empty-handed and facing in: each
        // gone once it has been seen and they look away; numbered apart from the one figure.
        Action::RingOfFigures(count) => pos
            .map(|center| {
                (0..count)
                    .map(|i| {
                        let angle = std::f64::consts::TAU * i as f64 / count as f64;
                        let spot = [
                            center[0] + (RING_BLOCKS * angle.cos()).round() as i32,
                            -10_000,
                            center[2] + (RING_BLOCKS * angle.sin()).round() as i32,
                        ];
                        let mut shown = figure_shown(ids, memory, target, spot, 60.0, false);
                        if let Command::Figure {
                            figure,
                            held,
                            once_seen,
                            ..
                        } = &mut shown
                        {
                            *figure = RING_FIRST + i;
                            *held = ItemStack::default();
                            *once_seen = true;
                        }
                        (shown, 0)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        // Over the bed, feet up from the ceiling (the host finds it), facing them as they get
        // up; gone once seen and looked away from.
        Action::UpsideDown => pos
            .map(|bed| {
                let mut shown = figure_shown(
                    ids,
                    memory,
                    target,
                    [bed[0], bed[1] - 1, bed[2]],
                    8.0,
                    false,
                );
                if let Command::Figure {
                    held,
                    once_seen,
                    upside_down,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *once_seen = true;
                    *upside_down = true;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // From their doorway across the floor to the bed, on its belly, and gone there.
        Action::Crawl => match (pos, memory.home.as_ref().and_then(|h| h.bed)) {
            (Some(door), Some(bed)) => {
                let mut shown = figure_shown(
                    ids,
                    memory,
                    target,
                    [door[0], door[1] - 1, door[2]],
                    40.0,
                    false,
                );
                if let Command::Figure {
                    held,
                    path,
                    crawls,
                    pos,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *crawls = true;
                    *path = vec![*pos, [bed[0] as f64 + 0.5, pos[1], bed[2] as f64 + 0.5]];
                }
                vec![(shown, 0)]
            }
            _ => Vec::new(),
        },
        // It runs at them flat out from there, after them as they move, gone just short with a
        // jolt (the host's `rushes`); empty-handed, its name hidden.
        Action::Hunted => pos
            .map(|from| {
                let mut shown = figure_shown(ids, memory, target, from, 20.0, false);
                if let Command::Figure { held, rushes, .. } = &mut shown {
                    *held = ItemStack::default();
                    *rushes = true;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // In the seat behind them, empty-handed, for as long as they do not turn round to it:
        // gone once seen and looked away from, or with the boat left.
        Action::BoatPassenger => pos
            .map(|at| {
                let mut shown = figure_shown(ids, memory, target, at, 60.0, false);
                if let Command::Figure {
                    held,
                    rides_behind,
                    once_seen,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *rides_behind = true;
                    *once_seen = true;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // Outside each window, a block out from it, its head at the glass, looking in; all gone
        // together when one is seen and looked away from.
        Action::EveryWindow => pos
            .map(|center| {
                windows(memory, center)
                    .into_iter()
                    .take(WINDOWS_MOST)
                    .enumerate()
                    .map(|(i, pane)| {
                        let out = step_toward(center, pane);
                        let spot = [pane[0] + out[0], pane[1] - 2, pane[2] + out[1]];
                        let mut shown = figure_shown(ids, memory, target, spot, 40.0, false);
                        if let Command::Figure {
                            figure,
                            held,
                            once_seen,
                            together,
                            ..
                        } = &mut shown
                        {
                            *figure = WINDOWS_FIRST + i as u32;
                            *held = ItemStack::default();
                            *once_seen = true;
                            *together = WINDOWS_FIRST;
                        }
                        (shown, 0)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        // At the edge of their light, one more than the nights it came before, side by side
        // across the way out from them, still, empty-handed; gone together when one is seen and
        // looked away from, or they come within six blocks.
        Action::OneMoreEachNight => pos
            .map(|edge| {
                let count = (memory.times("one_more_each_night") as u32 + 1)
                    .min(crate::events::ONE_MORE_MOST);
                let at = block_of(memory.last.pos);
                let out = step_toward(at, edge);
                let across = [-out[1], out[0]];
                (0..count)
                    .map(|i| {
                        // Side by side from the middle out: 0, +2, -2, +4, -4 ...
                        let side = ((i as i32 + 1) / 2) * 2 * if i % 2 == 1 { 1 } else { -1 };
                        let spot = [
                            edge[0] + across[0] * side,
                            -10_000,
                            edge[2] + across[1] * side,
                        ];
                        let mut shown = figure_shown(ids, memory, target, spot, 40.0, false);
                        if let Command::Figure {
                            figure,
                            held,
                            once_seen,
                            together,
                            vanish_within,
                            ..
                        } = &mut shown
                        {
                            *figure = ONE_MORE_FIRST + i;
                            *held = ItemStack::default();
                            *once_seen = true;
                            *together = ONE_MORE_FIRST;
                            *vanish_within = 6.0;
                        }
                        (shown, 0)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        // Held shut against them six seconds, each try a blow on it; and a knock from the other
        // side as they stand there.
        Action::DoorWontOpen => pos
            .map(|door| {
                let mut made = vec![(
                    Command::HoldDoor {
                        id: ids(),
                        pos: door,
                        seconds: 6.0,
                    },
                    0,
                )];
                for (i, at) in [40, 48, 56].into_iter().enumerate() {
                    made.push((
                        Command::PlaySound {
                            id: ids(),
                            player: target,
                            sound: "minecraft:block.wooden_door.close".to_string(),
                            pos: [
                                door[0] as f64 + 0.5,
                                door[1] as f64 + 1.0,
                                door[2] as f64 + 0.5,
                            ],
                            volume: 0.7,
                            pitch: 0.75 + 0.05 * i as f32,
                        },
                        at,
                    ));
                }
                made
            })
            .unwrap_or_default(),
        // Woken and stood outside, the door open behind them, their prints from the bed where
        // there is snow. The door last, so a door gone since loses only itself.
        Action::Sleepwalk => match (
            pos,
            memory.usual_door(),
            memory.home.as_ref().and_then(|h| h.bed),
        ) {
            (Some(outside), Some(door), Some(bed)) => vec![
                (
                    Command::CancelSleep {
                        id: ids(),
                        player: target,
                        seconds: 0.0,
                    },
                    0,
                ),
                (
                    Command::MovePlayer {
                        id: ids(),
                        player: target,
                        to: [
                            outside[0] as f64 + 0.5,
                            outside[1] as f64,
                            outside[2] as f64 + 0.5,
                        ],
                    },
                    2,
                ),
                (
                    Command::Prints {
                        id: ids(),
                        from: bed,
                        to: outside,
                    },
                    3,
                ),
                (
                    Command::ToggleBlock {
                        id: ids(),
                        pos: door,
                        to: Some(true),
                    },
                    4,
                ),
            ],
            _ => Vec::new(),
        },
        // In the light, facing them, for a blink.
        Action::TorchFlash => pos
            .map(|spot| {
                let mut shown = figure_shown(ids, memory, target, spot, 0.2, false);
                if let Command::Figure { held, .. } = &mut shown {
                    *held = ItemStack::default();
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // Lying where they died, on its side as a body lies, empty-handed; gone when they come
        // within ten blocks.
        Action::WhereYouDied => pos
            .map(|died| {
                let mut shown = figure_shown(
                    ids,
                    memory,
                    target,
                    [died[0], died[1] - 1, died[2]],
                    90.0,
                    false,
                );
                if let Command::Figure {
                    held,
                    lies,
                    vanish_within,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *lies = true;
                    *vanish_within = 10.0;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // Set back there as on a bad connection, and where they had been a figure stands, facing
        // them, empty-handed: gone once seen and looked away from.
        Action::Rubberband => pos
            .map(|back| {
                let was = block_of(memory.last.pos);
                let mut shown =
                    figure_shown(ids, memory, target, [was[0], -10_000, was[2]], 6.0, false);
                if let Command::Figure {
                    held, once_seen, ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *once_seen = true;
                }
                vec![
                    (
                        Command::MovePlayer {
                            id: ids(),
                            player: target,
                            to: [back[0] as f64 + 0.5, back[1] as f64, back[2] as f64 + 0.5],
                        },
                        0,
                    ),
                    (shown, 1),
                ]
            })
            .unwrap_or_default(),
        // In their doorway, holding out what it took, facing them; come within three blocks and
        // it is gone, what it held at their feet (the brain, on the host's word).
        Action::TheOffer => match (pos, &memory.held) {
            (Some(door), Some(held)) => {
                let mut shown = figure_shown(
                    ids,
                    memory,
                    target,
                    [door[0], door[1] - 1, door[2]],
                    120.0,
                    false,
                );
                if let Command::Figure {
                    held: holding,
                    vanish_within,
                    ..
                } = &mut shown
                {
                    *holding = ItemStack {
                        id: held.item.id.clone(),
                        count: 1,
                        slot: 0,
                    };
                    *vanish_within = 3.0;
                }
                vec![(shown, 0)]
            }
            _ => Vec::new(),
        },
        // At the shaft's mouth, over its top ladder, facing down at them, empty-handed; gone
        // when they climb within six blocks of it.
        Action::TopOfTheLadder => pos
            .map(|top| {
                let mut shown = figure_shown(ids, memory, target, top, 30.0, false);
                if let Command::Figure {
                    held,
                    vanish_within,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *vanish_within = 6.0;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // Pulled down twice, a moment apart, as if taken by the ankle: a whirl of water and a
        // drowning gasp, no harm; then let go.
        Action::PulledUnder => {
            let at = memory.last.pos;
            let sound = |ids: &mut dyn FnMut() -> u64, sound: &str| Command::PlaySound {
                id: ids(),
                player: target,
                sound: sound.to_string(),
                pos: at,
                volume: 0.9,
                pitch: 0.8,
            };
            vec![
                (
                    Command::PushPlayer {
                        id: ids(),
                        player: target,
                        velocity: [0.0, -0.9, 0.0],
                    },
                    0,
                ),
                (
                    sound(ids, "minecraft:block.bubble_column.whirlpool_inside"),
                    0,
                ),
                (
                    Command::PushPlayer {
                        id: ids(),
                        player: target,
                        velocity: [0.0, -0.7, 0.0],
                    },
                    6,
                ),
                (sound(ids, "minecraft:entity.player.hurt_drown"), 8),
            ]
        }
        // Its house dark (its torch taken away a while), then a lantern on its floor, and
        // another, and another, a few seconds apart; half a minute on, all out again, and its
        // torch back.
        Action::ItsLightsOn => match memory.its_house() {
            Some(house) => {
                let (torch, floor) = crate::building::house_lights(house.site, house.way);
                let mut made = vec![(
                    Command::HideBlock {
                        id: ids(),
                        pos: torch,
                        seconds: 45.0,
                        only: vec!["minecraft:wall_torch".to_string()],
                    },
                    0,
                )];
                for (i, spot) in floor.iter().enumerate() {
                    made.push((
                        lantern(ids, *spot, "minecraft:lantern", &[]),
                        80 + i as u64 * 60,
                    ));
                }
                for spot in &floor {
                    made.push((
                        lantern(ids, *spot, "minecraft:air", &["minecraft:lantern"]),
                        720,
                    ));
                }
                made
            }
            None => Vec::new(),
        },
        // What sits where the face goes moved aside into empty slots, then one of their biggest
        // stack into each of the face's slots: two eyes and a mouth, nothing lost or added.
        Action::FaceInTheChest => match (pos, memory.used_container()) {
            (Some(at), Some(chest)) => face_moves(&chest.contents)
                .unwrap_or_default()
                .into_iter()
                .enumerate()
                .map(|(i, (from_slot, to_slot, count))| {
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: at,
                            from_slot,
                            to: at,
                            to_slot,
                            count,
                        },
                        i as u64 * 2,
                    )
                })
                .collect(),
            _ => Vec::new(),
        },
        // From forty blocks over the spot down to it, fast, with a heavy fall's thud as it lands,
        // and gone there (the host's `falls`).
        Action::FallsFromTheSky => pos
            .map(|spot| {
                let mut shown = figure_shown(ids, memory, target, spot, 15.0, false);
                if let Command::Figure { held, falls, .. } = &mut shown {
                    *held = ItemStack::default();
                    *falls = true;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // A figure there, empty-handed; their pet gets up and walks
        // to it; the figure is gone as the pet gets there, and the pet comes back to them.
        Action::PetFollows => match (pos, memory.pets().next()) {
            (Some(spot), Some(pet)) => {
                let mut shown = figure_shown(ids, memory, target, spot, 7.0, false);
                let mut to = [
                    spot[0] as f64 + 0.5,
                    memory.last.pos[1],
                    spot[2] as f64 + 0.5,
                ];
                if let Command::Figure { held, pos, .. } = &mut shown {
                    *held = ItemStack::default();
                    to = *pos;
                }
                vec![
                    (shown, 0),
                    (
                        Command::Approach {
                            id: ids(),
                            entities: vec![pet.id],
                            player: target,
                            stop: 1.5,
                            seconds: 20.0,
                            pace: Some(1.0),
                            then: String::new(),
                            damage: 0.0,
                            kept: String::new(),
                            to: Some(to),
                        },
                        10,
                    ),
                ]
            }
            _ => Vec::new(),
        },
        // A bed laid along its house's left wall, then quietly made their respawn point: their
        // next death wakes them in its house.
        Action::RespawnItsBed => match memory.its_house() {
            Some(house) => {
                let (parts, head) = crate::building::house_bed(house.site, house.way);
                vec![
                    (
                        Command::PlaceBlueprint {
                            id: ids(),
                            at: house.site,
                            parts: parts
                                .into_iter()
                                .map(|p| craftkit::world::BlueprintPart {
                                    at: p.at,
                                    block: p.block.to_string(),
                                    state: p.state,
                                })
                                .collect(),
                            replace: Vec::new(),
                        },
                        0,
                    ),
                    (
                        Command::SetRespawn {
                            id: ids(),
                            player: target,
                            pos: head,
                        },
                        2,
                    ),
                ]
            }
            None => Vec::new(),
        },
        // Across their way there, from their left to their right, twenty blocks, at a walk,
        // facing its way and never them, empty-handed; gone at the far side.
        Action::CrossingAhead => pos
            .map(|middle| {
                let yaw = (memory.last.yaw as f64).to_radians();
                // Their right, facing yaw: the game's yaw turns clockwise, +x is east.
                let right = [-yaw.cos(), -yaw.sin()];
                let point = |side: f64| {
                    [
                        middle[0] as f64 + 0.5 + right[0] * side,
                        -10_000.0,
                        middle[2] as f64 + 0.5 + right[1] * side,
                    ]
                };
                let mut shown = figure_shown(ids, memory, target, middle, 40.0, false);
                if let Command::Figure {
                    held,
                    path,
                    walks,
                    pos,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *walks = true;
                    *pos = point(-CROSSING_HALF);
                    *path = vec![point(-CROSSING_HALF), point(CROSSING_HALF)];
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // Along their route ahead of them, one every eight blocks, all facing the house, empty-
        // handed; each gone as they come within four blocks of it.
        Action::InALine => match memory.home.as_ref() {
            Some(h) => {
                let house = [
                    h.center[0] as f64 + 0.5,
                    h.center[1] as f64 + 1.5,
                    h.center[2] as f64 + 0.5,
                ];
                in_a_line(memory)
                    .into_iter()
                    .enumerate()
                    .map(|(i, spot)| {
                        let mut shown = figure_shown(
                            ids,
                            memory,
                            target,
                            [spot[0], -10_000, spot[2]],
                            120.0,
                            false,
                        );
                        if let Command::Figure {
                            figure,
                            held,
                            vanish_within,
                            faces,
                            ..
                        } = &mut shown
                        {
                            *figure = IN_A_LINE_FIRST + i as u32;
                            *held = ItemStack::default();
                            *vanish_within = 4.0;
                            *faces = Some(house);
                        }
                        (shown, 0)
                    })
                    .collect()
            }
            None => Vec::new(),
        },
        // Sunk to its head there, facing them; once seen, it sinks out of sight (the host's
        // `sunk`).
        Action::PeeksFromGround => pos
            .map(|spot| {
                let mut shown = figure_shown(ids, memory, target, spot, 60.0, false);
                if let Command::Figure { held, sunk, .. } = &mut shown {
                    *held = ItemStack::default();
                    *sunk = true;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // The zombie nearest them wears their own head five minutes, and walks at them.
        Action::WearsYourFace => face_wearer(memory)
            .map(|zombie| {
                vec![
                    (
                        Command::WearFace {
                            id: ids(),
                            entity: zombie.id,
                            player: target,
                            seconds: 300.0,
                        },
                        0,
                    ),
                    (
                        Command::Approach {
                            id: ids(),
                            entities: vec![zombie.id],
                            player: target,
                            stop: 2.0,
                            seconds: 40.0,
                            pace: Some(0.8),
                            then: String::new(),
                            damage: 0.0,
                            kept: String::new(),
                            to: None,
                        },
                        2,
                    ),
                ]
            })
            .unwrap_or_default(),
        // The animal nearest them walks after them, stopping six blocks off when they stop, a
        // walk renewed every eighty seconds (a walk lasts ninety at most) for a whole day.
        Action::AnimalFollows => memory
            .animals_near_target()
            .first()
            .map(|animal| {
                (0..FOLLOW_WALKS)
                    .map(|i| {
                        (
                            Command::Approach {
                                id: ids(),
                                entities: vec![animal.id],
                                player: target,
                                stop: 6.0,
                                seconds: 90.0,
                                pace: Some(1.0),
                                then: String::new(),
                                damage: 0.0,
                                kept: String::new(),
                                to: None,
                            },
                            i * FOLLOW_EVERY,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        // This dive, their air runs out twice as fast; nothing there.
        Action::HeldBreath => vec![(
            Command::DrainAir {
                id: ids(),
                player: target,
                seconds: 20.0,
            },
            0,
        )],
        // Every animal, villager and mob within twenty-four blocks of them stops at once, three
        // seconds, facing them; then as it was.
        Action::AllFreeze => {
            let at = memory.last.pos;
            memory
                .last
                .creatures_near
                .iter()
                .chain(memory.last.hostiles_near.iter())
                .filter(|c| {
                    ((c.pos[0] - at[0]).powi(2) + (c.pos[2] - at[2]).powi(2)).sqrt() <= 24.0
                })
                .map(|c| {
                    (
                        Command::Hold {
                            id: ids(),
                            entity: c.id,
                            player: target,
                            seconds: 3.0,
                            at: None,
                            creep: false,
                        },
                        0,
                    )
                })
                .collect()
        }
        // A player's marker at their house on the map they hold, five minutes.
        Action::FigureOnMap => pos
            .map(|house| {
                vec![(
                    Command::MapMark {
                        id: ids(),
                        player: target,
                        at: [
                            house[0] as f64 + 0.5,
                            house[1] as f64,
                            house[2] as f64 + 0.5,
                        ],
                        seconds: 300.0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Three blocks to the side of the portal, facing it, empty-handed: gone once seen and
        // looked away from.
        Action::WaitsAtThePortal => pos
            .map(|portal| {
                let mut shown = figure_shown(
                    ids,
                    memory,
                    target,
                    [portal[0] + 3, -10_000, portal[2]],
                    12.0,
                    false,
                );
                if let Command::Figure {
                    held,
                    once_seen,
                    faces,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *once_seen = true;
                    *faces = Some([
                        portal[0] as f64 + 0.5,
                        portal[1] as f64 + 1.0,
                        portal[2] as f64 + 0.5,
                    ]);
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // From its roof, high, a pale face in the dark.
        Action::FireworkOverItsHouse => pos
            .map(|site| {
                vec![(
                    Command::Firework {
                        id: ids(),
                        pos: [
                            site[0] as f64 + 0.5,
                            site[1] as f64 + 6.0,
                            site[2] as f64 + 0.5,
                        ],
                        shape: "creeper".to_string(),
                        colors: vec![0xE8E8E0],
                        flight: 2,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Where the chicken stands, with the sound of an egg laid: what it took, the very thing.
        Action::ChickenLays => match (
            memory
                .last
                .creatures_near
                .iter()
                .find(|c| c.kind == "chicken"),
            &memory.held,
        ) {
            (Some(chicken), Some(held)) => vec![
                (
                    Command::PlaySound {
                        id: ids(),
                        player: target,
                        sound: "minecraft:entity.chicken.egg".to_string(),
                        pos: chicken.pos,
                        volume: 1.0,
                        pitch: 0.8,
                    },
                    0,
                ),
                (
                    Command::DropItem {
                        id: ids(),
                        pos: [chicken.pos[0], chicken.pos[1] + 0.3, chicken.pos[2]],
                        item: held.item.clone(),
                        kept: "held".to_string(),
                    },
                    1,
                ),
            ],
            _ => Vec::new(),
        },
        // Beside it, toward the house's middle, facing it, working at it with their own tool;
        // gone as they come within five blocks.
        Action::AtTheirTable => match (pos, memory.home.as_ref()) {
            (Some(table), Some(home)) => {
                let back = step_toward(table, home.center);
                let spot = [table[0] + back[0], table[1] - 1, table[2] + back[1]];
                let mut shown = figure_shown(ids, memory, target, spot, 60.0, false);
                if let Command::Figure {
                    swinging,
                    vanish_within,
                    faces,
                    ..
                } = &mut shown
                {
                    *swinging = true;
                    *vanish_within = 5.0;
                    *faces = Some([
                        table[0] as f64 + 0.5,
                        table[1] as f64 + 0.5,
                        table[2] as f64 + 0.5,
                    ]);
                }
                vec![(shown, 0)]
            }
            _ => Vec::new(),
        },
        // Jumping in place there, again and again, empty-handed; gone once seen and looked
        // away from.
        Action::JumpingFar => pos
            .map(|spot| {
                let mut shown = figure_shown(ids, memory, target, spot, 30.0, false);
                if let Command::Figure {
                    held,
                    jumps,
                    once_seen,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *jumps = true;
                    *once_seen = true;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // Just inside the window, a block in from the glass, its head at it, empty-handed: it
        // raises its arm three times, as a wave; gone as they come within five blocks.
        Action::WavesFromInside => match (pos, memory.home.as_ref()) {
            (Some(pane), Some(home)) => {
                let inward = step_toward(pane, home.center);
                let spot = [pane[0] + inward[0], pane[1] - 2, pane[2] + inward[1]];
                let mut shown = figure_shown(ids, memory, target, spot, 30.0, false);
                if let Command::Figure {
                    held,
                    vanish_within,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *vanish_within = 5.0;
                }
                let mut made = vec![(shown, 0)];
                for at in [20, 34, 48] {
                    made.push((
                        Command::FigureSwing {
                            id: ids(),
                            player: target,
                            figure: 1,
                            damage: 0.0,
                            spare: true,
                        },
                        at,
                    ));
                }
                made
            }
            _ => Vec::new(),
        },
        // From their door to there, where there is snow.
        Action::PrintsToTheCave => match (memory.usual_door(), pos) {
            (Some(door), Some(to)) => vec![(
                Command::Prints {
                    id: ids(),
                    from: door,
                    to,
                },
                0,
            )],
            _ => Vec::new(),
        },
        // In their horse's saddle, for them alone; gone once seen and looked away from, or as
        // anyone gets on it.
        Action::RidesTheirHorse => their_animal(memory, "horse", false)
            .map(|horse| {
                let mut shown = figure_shown(ids, memory, target, block_of(horse.pos), 60.0, false);
                if let Command::Figure {
                    held,
                    rides,
                    once_seen,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *rides = horse.id;
                    *once_seen = true;
                }
                vec![(shown, 0)]
            })
            .unwrap_or_default(),
        // Crouched in front of their chest, facing it, eating their food, the sound of it; gone
        // once seen and looked away from.
        Action::EatsAtTheirChest => match (pos, memory.used_container().and_then(chest_food)) {
            (Some(chest), Some(food)) => {
                let home = memory.home.as_ref().map(|h| h.center).unwrap_or(chest);
                let front = step_toward(chest, home);
                let spot = [chest[0] + front[0], chest[1] - 1, chest[2] + front[1]];
                let mut shown = figure_shown(ids, memory, target, spot, 30.0, false);
                if let Command::Figure {
                    held,
                    eats,
                    once_seen,
                    faces,
                    ..
                } = &mut shown
                {
                    *held = ItemStack {
                        id: food.clone(),
                        count: 1,
                        slot: 0,
                    };
                    *eats = true;
                    *once_seen = true;
                    *faces = Some([
                        chest[0] as f64 + 0.5,
                        chest[1] as f64,
                        chest[2] as f64 + 0.5,
                    ]);
                }
                let at = [
                    spot[0] as f64 + 0.5,
                    spot[1] as f64 + 1.5,
                    spot[2] as f64 + 0.5,
                ];
                let mut made = vec![(shown, 0)];
                for (i, tick) in [8, 16, 24, 32, 40].into_iter().enumerate() {
                    made.push((
                        Command::PlaySound {
                            id: ids(),
                            player: target,
                            sound: "minecraft:entity.generic.eat".to_string(),
                            pos: at,
                            volume: 0.7,
                            pitch: 0.9 + 0.05 * i as f32,
                        },
                        tick,
                    ));
                }
                made
            }
            _ => Vec::new(),
        },
        // Just inside that window, its head at the glass, moving as they move from where both
        // stood; twenty seconds, or gone once seen and looked away from.
        Action::MirrorInTheWindow => match (pos, memory.home.as_ref()) {
            (Some(pane), Some(home)) => {
                let inward = step_toward(pane, home.center);
                let spot = [pane[0] + inward[0], pane[1] - 2, pane[2] + inward[1]];
                let mut shown = figure_shown(ids, memory, target, spot, 20.0, false);
                if let Command::Figure {
                    held,
                    mirror,
                    once_seen,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *mirror = true;
                    *once_seen = true;
                }
                vec![(shown, 0)]
            }
            _ => Vec::new(),
        },
        // Every door of the house open at once, with their sounds.
        Action::EveryDoorOpens => match (pos, memory.home.as_ref()) {
            (Some(_), Some(home)) => home
                .doors
                .iter()
                .map(|door| {
                    (
                        Command::ToggleBlock {
                            id: ids(),
                            pos: *door,
                            to: Some(true),
                        },
                        0,
                    )
                })
                .collect(),
            _ => Vec::new(),
        },
        // Each of their flowers (six at most) a dead bush, its own kind only; a day on, each
        // the flower it was, over the dead bush only.
        Action::FlowersWilted => memory
            .plants
            .iter()
            .filter(|(id, _)| crate::memory::is_flower(id))
            .take(6)
            .flat_map(|(id, at)| {
                [
                    (
                        Command::SwapBlock {
                            id: ids(),
                            pos: *at,
                            block: "minecraft:dead_bush".to_string(),
                            only: vec![id.clone()],
                        },
                        0,
                    ),
                    (
                        Command::SwapBlock {
                            id: ids(),
                            pos: *at,
                            block: id.clone(),
                            only: vec!["minecraft:dead_bush".to_string()],
                        },
                        WILTED_FOR,
                    ),
                ]
            })
            .collect(),
        // A lit campfire there with a log seat either side, a figure crouched by it facing the
        // fire; at dawn every block of it gone again (only its own), and the figure long gone.
        Action::CampNearby => pos
            .map(|spot| {
                let fire = [spot[0] + 2, spot[1], spot[2]];
                let seats = [
                    [spot[0] + 4, spot[1], spot[2]],
                    [spot[0] + 2, spot[1], spot[2] + 2],
                ];
                let mut made = vec![(
                    lantern_state(ids, fire, "minecraft:campfire", "lit=true"),
                    0,
                )];
                for seat in seats {
                    made.push((lantern_state(ids, seat, "minecraft:oak_log", "axis=z"), 1));
                }
                let mut shown = figure_shown(
                    ids,
                    memory,
                    target,
                    [spot[0], spot[1] - 1, spot[2]],
                    120.0,
                    false,
                );
                if let Command::Figure {
                    held,
                    eats,
                    once_seen,
                    faces,
                    ..
                } = &mut shown
                {
                    *held = ItemStack::default();
                    *eats = true;
                    *once_seen = true;
                    *faces = Some([fire[0] as f64 + 0.5, fire[1] as f64, fire[2] as f64 + 0.5]);
                }
                made.push((shown, 2));
                let dawn = u64::from((24_000 - now.day_time.min(23_999) + 500).max(600));
                made.push((
                    lantern(ids, fire, "minecraft:air", &["minecraft:campfire"]),
                    dawn,
                ));
                for seat in seats {
                    made.push((
                        lantern(ids, seat, "minecraft:air", &["minecraft:oak_log"]),
                        dawn,
                    ));
                }
                made
            })
            .unwrap_or_default(),
        // Eight pillars of stone bricks, three high, in a ring five blocks out, each on the
        // ground's own block; only into air.
        Action::StoneCircle => pos
            .map(|middle| {
                let mut parts = Vec::new();
                for i in 0..8 {
                    let angle = std::f64::consts::TAU * i as f64 / 8.0;
                    let (x, z) = (
                        (angle.cos() * 5.0).round() as i32,
                        (angle.sin() * 5.0).round() as i32,
                    );
                    for y in 1..=3 {
                        parts.push(craftkit::world::BlueprintPart {
                            at: [x, y, z],
                            block: if y == 3 {
                                "minecraft:chiseled_stone_bricks"
                            } else {
                                "minecraft:stone_bricks"
                            }
                            .to_string(),
                            state: String::new(),
                        });
                    }
                }
                vec![(
                    Command::PlaceBlueprint {
                        id: ids(),
                        at: middle,
                        parts,
                        replace: Vec::new(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::SlipsFromHand => pos
            .map(|_| {
                vec![(
                    Command::DropHeld {
                        id: ids(),
                        player: target,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Their pet held a minute where it stands, facing that wall.
        Action::PetStaresAtWall => match (pos, memory.pets().next()) {
            (Some(wall), Some(pet)) => vec![(
                Command::Hold {
                    id: ids(),
                    entity: pet.id,
                    player: target,
                    seconds: 60.0,
                    at: Some([
                        wall[0] as f64 + 0.5,
                        wall[1] as f64 + 0.5,
                        wall[2] as f64 + 0.5,
                    ]),
                    creep: false,
                },
                0,
            )],
            _ => Vec::new(),
        },
        // Four bats two blocks over the house's floor round its middle, a minute, then gone.
        Action::BatsInTheHouse => match (pos, memory.home.as_ref()) {
            (Some(center), Some(home)) => [[1, 1], [-1, 1], [1, -1], [-1, -1]]
                .into_iter()
                .map(|[dx, dz]| {
                    (
                        Command::PlaceEntity {
                            id: ids(),
                            of: "minecraft:bat".to_string(),
                            pos: [
                                center[0] as f64 + 0.5 + dx as f64,
                                home.floor_y as f64 + 2.0,
                                center[2] as f64 + 0.5 + dz as f64,
                            ],
                            yaw: 0.0,
                            seconds: 60.0,
                            kept: String::new(),
                            face: None,
                        },
                        0,
                    )
                })
                .collect(),
            _ => Vec::new(),
        },
        // Their screen black a moment (the lights failing), and as it comes back an hour of the
        // day is gone, the sun moved on.
        Action::LostHour => vec![
            (
                Command::Hint {
                    id: ids(),
                    player: target,
                    hint: "lights_fail".to_string(),
                    pos: None,
                    seconds: 1.2,
                    detail: String::new(),
                },
                0,
            ),
            (
                Command::SkipTime {
                    id: ids(),
                    ticks: 1_000,
                },
                8,
            ),
        ],
        // Held shut five seconds, each try a locked chest's rattle; two knocks from inside it.
        Action::ChestWontOpen => pos
            .map(|chest| {
                let at = [
                    chest[0] as f64 + 0.5,
                    chest[1] as f64 + 0.5,
                    chest[2] as f64 + 0.5,
                ];
                let mut made = vec![(
                    Command::HoldContainer {
                        id: ids(),
                        pos: chest,
                        seconds: 5.0,
                    },
                    0,
                )];
                for tick in [30, 40] {
                    made.push((
                        Command::PlaySound {
                            id: ids(),
                            player: target,
                            sound: "minecraft:block.wooden_trapdoor.close".to_string(),
                            pos: at,
                            volume: 0.6,
                            pitch: 0.6,
                        },
                        tick,
                    ));
                }
                made
            })
            .unwrap_or_default(),
        // Their name in letters three blocks wide and five long, a block apart, along x: the
        // snow taken off where the letters are, snow only (its layer on the ground or over it).
        Action::NameInTheSnow => pos
            .map(|at| {
                let mut parts = Vec::new();
                // Their name as the books sign it: "you" before it is known.
                for (i, letter) in crate::books::signed(memory).chars().take(10).enumerate() {
                    for [x, z] in snow_letter(letter) {
                        for y in 0..=1 {
                            parts.push(craftkit::world::BlueprintPart {
                                at: [i as i32 * 4 + x, y, z],
                                block: "minecraft:air".to_string(),
                                state: String::new(),
                            });
                        }
                    }
                }
                if parts.is_empty() {
                    return Vec::new();
                }
                vec![(
                    Command::PlaceBlueprint {
                        id: ids(),
                        at,
                        parts,
                        replace: vec!["minecraft:snow".to_string()],
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Sixteen torches twelve blocks out round the house's middle, each finding its footing
        // within eight blocks up or down of the floor.
        Action::TorchRing => match (pos, memory.home.as_ref()) {
            (Some(center), Some(home)) => (0..16)
                .map(|i| {
                    let angle = std::f64::consts::TAU * i as f64 / 16.0;
                    let spot = [
                        center[0] + (angle.cos() * 12.0).round() as i32,
                        home.floor_y,
                        center[2] + (angle.sin() * 12.0).round() as i32,
                    ];
                    (torch(ids, spot, 8), i as u64 * 4)
                })
                .collect(),
            _ => Vec::new(),
        },
        // Six of their most plentiful block taken from their chest one at a time, each kept,
        // and dropped one by one, three blocks apart, from their door away from the house: a
        // line to follow. Theirs still, there to pick up.
        Action::Breadcrumbs => match (
            memory.used_container(),
            memory.usual_door(),
            memory.home.as_ref(),
        ) {
            (Some(chest), Some(door), Some(home)) => match crumb_stack(chest) {
                Some(stack) => {
                    let away = step_toward(home.center, door);
                    let mut made = Vec::new();
                    for i in 0..CRUMBS {
                        let keep = format!("crumb_{i}");
                        made.push((
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: stack.slot,
                                count: 1,
                                keep: keep.clone(),
                                expect: stack.id.clone(),
                            },
                            i as u64 * 2,
                        ));
                        let out = (i as i32 + 1) * 3;
                        made.push((
                            Command::DropItem {
                                id: ids(),
                                pos: [
                                    (door[0] + away[0] * out) as f64 + 0.5,
                                    door[1] as f64 + 0.5,
                                    (door[2] + away[1] * out) as f64 + 0.5,
                                ],
                                item: ItemStack {
                                    id: stack.id.clone(),
                                    count: 1,
                                    slot: 0,
                                },
                                kept: keep,
                            },
                            i as u64 * 2 + 1,
                        ));
                    }
                    made
                }
                None => Vec::new(),
            },
            _ => Vec::new(),
        },
        Action::FigureWave => pos
            .map(|p| {
                vec![
                    (figure_shown(ids, memory, target, p, 6.0, true), 0),
                    (
                        Command::FigureSwing {
                            id: ids(),
                            player: target,
                            figure: 1,
                            damage: 0.0,
                            spare: true,
                        },
                        40,
                    ),
                ]
            })
            .unwrap_or_default(),
        // Their newest tunnel again, three blocks over, two high, lit at their spacing; its ore
        // goes to their chest as it is dug (the brain, on the report).
        Action::MineBeside => match memory.newest_tunnel() {
            Some(t) => {
                let way = t.way();
                let side = [-way[1], way[0]];
                let spacing = memory.torch_spacing().round().max(1.0) as usize;
                let mut made = Vec::new();
                for (i, c) in t.columns().into_iter().enumerate() {
                    let p = [c[0] + side[0] * 3, c[1], c[2] + side[1] * 3];
                    let t0 = i as u64 * 20;
                    made.push((dig(ids, p), t0));
                    made.push((dig(ids, [p[0], p[1] + 1, p[2]]), t0 + 5));
                    if i % spacing == spacing / 2 {
                        made.push((torch(ids, p, 0), t0 + 10));
                    }
                }
                made
            }
            None => vec![],
        },
        // The mouth of their oldest tunnel walled with the lodger's own cobblestone.
        Action::TunnelSealed => pos
            .map(|m| {
                vec![
                    (fill(ids, m, "minecraft:cobblestone"), 0),
                    (
                        fill(ids, [m[0], m[1] + 1, m[2]], "minecraft:cobblestone"),
                        15,
                    ),
                ]
            })
            .unwrap_or_default(),
        // Digging toward them from twenty-five blocks past their tunnel's end, a column every
        // four seconds, heard as it comes; it breaks through, or stops one block short.
        Action::TunnelToward => match memory.newest_tunnel() {
            Some(t) => {
                let (end, way) = (t.end(), t.way());
                let stop = if seed.is_multiple_of(2) { 1 } else { 2 };
                let mut made = Vec::new();
                for (i, k) in (stop..=25).rev().enumerate() {
                    let p = [end[0] + way[0] * k, end[1], end[2] + way[1] * k];
                    let t0 = i as u64 * 80;
                    made.push((dig(ids, p), t0));
                    made.push((dig(ids, [p[0], p[1] + 1, p[2]]), t0 + 10));
                    made.push((
                        Command::BlockSound {
                            id: ids(),
                            player: target,
                            block: "minecraft:stone".to_string(),
                            of: "break".to_string(),
                            pos: block_center(p),
                            volume: 0.6,
                            pitch: 0.9,
                        },
                        t0 + 2,
                    ));
                }
                made
            }
            None => vec![],
        },
        // Their frame lit, and their flint and steel a use the worse.
        Action::PortalLit => match (
            pos,
            memory.in_used_container(|id| id == "minecraft:flint_and_steel"),
        ) {
            (Some(frame), Some((chest, flint))) => vec![
                (
                    Command::LightPortal {
                        id: ids(),
                        near: frame,
                    },
                    0,
                ),
                (
                    Command::DamageItem {
                        id: ids(),
                        pos: chest.pos,
                        slot: flint.slot,
                        amount: 1,
                    },
                    2,
                ),
            ],
            _ => vec![],
        },
        // The vein ahead dug out, a torch where it was.
        Action::MinedOut => pos
            .map(|ore| {
                let vein: Vec<BlockPos> = memory
                    .ores
                    .iter()
                    .map(|(_, p)| *p)
                    .filter(|p| distance(*p, ore) <= 2.0)
                    .collect();
                let mut made: Vec<(Command, u64)> = vein
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (dig(ids, *p), i as u64 * 15))
                    .collect();
                made.push((torch(ids, ore, 1), vein.len() as u64 * 15 + 5));
                made
            })
            .unwrap_or_default(),
        // A shaft a block wide down from its place, as deep as their ladders go (four to
        // twelve), the ladders up its side, then taken from their chest.
        Action::LadderDown => {
            match (pos, memory.in_used_container(|id| id == "minecraft:ladder")) {
                // As deep as the ladders they have, four at least: one ladder in the chest used
                // to make four.
                (Some(top), Some((chest, ladders))) if ladders.count >= 4 => {
                    let deep = ladders.count.min(12) as i32;
                    let mut made = Vec::new();
                    for y in 1..=deep {
                        let p = [top[0], top[1] - y, top[2]];
                        let t0 = y as u64 * 20;
                        made.push((dig(ids, p), t0));
                        // A ladder of theirs taken for each one laid, before it is: a shaft that
                        // stopped part way left ladders laid that were never taken (one that
                        // will not go down is put back with the event's failure).
                        made.push((
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: ladders.slot,
                                expect: ladders.id.clone(),
                                count: 1,
                                keep: String::new(),
                            },
                            t0 + 3,
                        ));
                        made.push((
                            Command::SetBlock {
                                id: ids(),
                                pos: p,
                                block: "minecraft:ladder".to_string(),
                                state: "facing=north".to_string(),
                                replace: vec![],
                                settle: 0,
                                wall: String::new(),
                                rest: false,
                                only: false,
                            },
                            t0 + 5,
                        ));
                    }
                    made
                }
                _ => vec![],
            }
        }
        // Their tunnel five columns further, their size, unlit.
        Action::TunnelGoesOn => match memory.newest_tunnel() {
            Some(t) => {
                let (end, way) = (t.end(), t.way());
                (1..=5)
                    .flat_map(|k| {
                        let p = [end[0] + way[0] * k, end[1], end[2] + way[1] * k];
                        let t0 = k as u64 * 20;
                        [
                            (dig(ids, p), t0),
                            (dig(ids, [p[0], p[1] + 1, p[2]]), t0 + 5),
                        ]
                    })
                    .collect()
            }
            None => vec![],
        },
        // Their tunnel's lights out from the far end toward them, a second apart, back in five
        // minutes.
        Action::TorchesFarEnd => tunnel_lights(memory)
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    Command::HideBlock {
                        id: ids(),
                        pos: p,
                        seconds: 300.0,
                        only: [
                            "minecraft:torch",
                            "minecraft:wall_torch",
                            "minecraft:soul_torch",
                            "minecraft:soul_wall_torch",
                            "minecraft:lantern",
                        ]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    },
                    i as u64 * 20,
                )
            })
            .collect(),
        Action::SignDeep => pos
            .map(|end| sign_at(ids, end, vec![words(Kind::SignWord, "", 0)]))
            .unwrap_or_default(),
        // Gravel behind them, floor and head, the way they came closed (they can dig it).
        Action::WayBackCloses => pos
            .map(|b| {
                vec![
                    (fill(ids, b, "minecraft:gravel"), 0),
                    (fill(ids, [b[0], b[1] + 1, b[2]], "minecraft:gravel"), 4),
                ]
            })
            .unwrap_or_default(),
        // A lantern of the village set down on their way, then gone from the village: one that
        // finds no ground stays where it hung.
        Action::VillageLantern => match (pos, memory.scanned("lantern").next()) {
            (Some(on_route), Some(lantern)) => {
                let kind = memory
                    .scanned
                    .iter()
                    .find(|(_, p)| *p == lantern)
                    .map(|(k, _)| format!("minecraft:{k}"))
                    .unwrap_or_else(|| "minecraft:lantern".to_string());
                vec![
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: [on_route[0], on_route[1] + 1, on_route[2]],
                            block: kind.clone(),
                            state: String::new(),
                            replace: vec![],
                            settle: 3,
                            wall: String::new(),
                            rest: true,
                            only: false,
                        },
                        0,
                    ),
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: lantern,
                            block: "minecraft:air".to_string(),
                            state: String::new(),
                            replace: vec![kind],
                            settle: 0,
                            wall: String::new(),
                            rest: false,
                            only: true,
                        },
                        2,
                    ),
                ]
            }
            _ => vec![],
        },
        // What it took, on a villager's trades for an emerald, once.
        Action::VillagerWord => memory
            .villager_near()
            .map(|v| {
                vec![(
                    Command::OfferTrade {
                        id: ids(),
                        entity: v.id,
                        kept: "held".to_string(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Every villager into one house till dawn, every other door open, the lanterns out;
        // at dawn the doors shut again (the villagers and the lights come back on their own).
        Action::VillageAbandoned => match (pos, memory.village_middle()) {
            (Some(door), Some(middle)) => {
                let to_dawn =
                    (24_000u32.saturating_sub(now.day_time) as f32 / 20.0).clamp(60.0, 600.0);
                let dawn = (to_dawn * 20.0) as u64;
                let mut made = vec![(
                    Command::GatherVillagers {
                        id: ids(),
                        center: middle,
                        door,
                        seconds: to_dawn,
                        player: target,
                    },
                    0,
                )];
                for (i, other) in memory
                    .village_doors()
                    .into_iter()
                    .filter(|d| *d != door)
                    .take(16)
                    .enumerate()
                {
                    let open = |ids: &mut dyn FnMut() -> u64, value: &str| Command::SetState {
                        id: ids(),
                        pos: other,
                        property: "open".to_string(),
                        value: value.to_string(),
                    };
                    made.push((open(ids, "true"), 5 + i as u64));
                    made.push((open(ids, "false"), dawn + i as u64));
                }
                let lanterns: Vec<(String, BlockPos)> = memory
                    .scanned
                    .iter()
                    .filter(|(k, _)| k.ends_with("lantern"))
                    .take(16)
                    .cloned()
                    .collect();
                for (i, (kind, p)) in lanterns.into_iter().enumerate() {
                    made.push((
                        Command::HideBlock {
                            id: ids(),
                            pos: p,
                            seconds: to_dawn,
                            only: vec![format!("minecraft:{kind}")],
                        },
                        30 + i as u64,
                    ));
                }
                made
            }
            _ => vec![],
        },
        // The bell they rang, once more, a second and a half later.
        Action::BellTwice => pos
            .map(|bell| {
                vec![(
                    Command::ToggleBlock {
                        id: ids(),
                        pos: bell,
                        to: toggle_to(event.id),
                    },
                    30,
                )]
            })
            .unwrap_or_default(),
        // Prints from the door to its place, or from the dark up to the door and none away.
        Action::Trail => match (pos, memory.home.as_ref()) {
            (Some(door), Some(_)) => {
                let to = spot(memory, &mut Rng::new(seed as u64)).unwrap_or(door);
                vec![(
                    Command::Prints {
                        id: ids(),
                        from: door,
                        to,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::TrailIn => match (pos, memory.home.as_ref()) {
            (Some(door), Some(home)) => {
                let step = step_toward(home.center, door);
                vec![(
                    Command::Prints {
                        id: ids(),
                        from: [door[0] + step[0] * 20, door[1], door[2] + step[1] * 20],
                        to: door,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        Action::BoatWaiting | Action::BoatInHouse => match (pos, memory.their_boat()) {
            (Some(p), Some(boat)) => vec![(
                Command::MoveEntity {
                    id: ids(),
                    entity: boat.id,
                    to: [p[0] as f64 + 0.5, p[1] as f64 + 0.6, p[2] as f64 + 0.5],
                },
                0,
            )],
            _ => vec![],
        },
        // A bobber cast, splashes and a catch, over a minute, with no one there; ripples where
        // it lies.
        Action::FishingLine => pos
            .map(|p| {
                let at = block_center(p);
                let mut made = vec![(
                    Command::Hint {
                        id: ids(),
                        player: target,
                        hint: "bobber".to_string(),
                        pos: Some(p),
                        seconds: 60.0,
                        detail: String::new(),
                    },
                    0,
                )];
                made.push((
                    sound_at(
                        ids,
                        target,
                        "minecraft:entity.fishing_bobber.throw",
                        at,
                        0.7,
                    ),
                    0,
                ));
                for (i, t) in [200u64, 520, 900].into_iter().enumerate() {
                    made.push((
                        sound_at(
                            ids,
                            target,
                            "minecraft:entity.fishing_bobber.splash",
                            at,
                            0.6,
                        ),
                        t,
                    ));
                    if i == 2 {
                        made.push((
                            sound_at(
                                ids,
                                target,
                                "minecraft:entity.fishing_bobber.retrieve",
                                at,
                                0.7,
                            ),
                            t + 10,
                        ));
                    }
                }
                made
            })
            .unwrap_or_default(),
        // Two snow blocks and a pumpkin on them, facing the door.
        Action::Snowman => match (pos, memory.usual_door()) {
            (Some(p), Some(door)) => {
                let yaw = yaw_toward(block_center(p), block_center(door));
                let facing = match ((yaw.rem_euclid(360.0) + 45.0) / 90.0) as i32 % 4 {
                    0 => "south",
                    1 => "west",
                    2 => "north",
                    _ => "east",
                };
                vec![
                    (on_ground(ids, p, "minecraft:snow_block"), 0),
                    (
                        on_ground(ids, [p[0], p[1] + 1, p[2]], "minecraft:snow_block"),
                        10,
                    ),
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: [p[0], p[1] + 2, p[2]],
                            block: "minecraft:carved_pumpkin".to_string(),
                            state: format!("facing={facing}"),
                            replace: vec![],
                            settle: 3,
                            wall: String::new(),
                            rest: true,
                            only: false,
                        },
                        20,
                    ),
                ]
            }
            _ => vec![],
        },
        // A squid under the water near them, still, facing them, twenty seconds.
        Action::UnderWater => pos
            .map(|p| {
                vec![(
                    Command::PlaceEntity {
                        id: ids(),
                        of: "minecraft:squid".to_string(),
                        pos: [p[0] as f64 + 0.5, p[1] as f64 - 1.5, p[2] as f64 + 0.5],
                        yaw: 0.0,
                        seconds: 20.0,
                        kept: String::new(),
                        face: Some(target),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Wet steps from the door to the bed, a step every half second, a drip with each.
        Action::WetSteps => memory
            .door_to_bed()
            .into_iter()
            .enumerate()
            .flat_map(|(i, p)| {
                let t = i as u64 * 10;
                [
                    (sound_at(ids, target, "minecraft:block.mud.step", p, 0.5), t),
                    (
                        sound_at(
                            ids,
                            target,
                            "minecraft:block.pointed_dripstone.drip_water",
                            p,
                            0.4,
                        ),
                        t + 4,
                    ),
                ]
            })
            .collect(),
        Action::JukeboxDisc => match (pos, memory.disc_in_chest()) {
            (Some(jukebox), Some((chest, disc))) => vec![(
                Command::Jukebox {
                    id: ids(),
                    pos: jukebox,
                    from: chest.pos,
                    slot: disc.slot,
                },
                0,
            )],
            _ => vec![],
        },
        Action::LecternPage => pos
            .map(|p| vec![(Command::TurnPage { id: ids(), pos: p }, 0)])
            .unwrap_or_default(),
        // Their fuel out to the chest, the other in, as long a burn.
        Action::SwapFuel => match memory.fuel_swap() {
            Some((furnace, fuel, chest, other, count)) => {
                let Some(&free) = chest.empty_slots().first() else {
                    return vec![];
                };
                vec![
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: furnace.pos,
                            from_slot: 1,
                            to: chest.pos,
                            to_slot: free,
                            count: fuel.count,
                        },
                        0,
                    ),
                    (
                        Command::MoveItem {
                            id: ids(),
                            from: chest.pos,
                            from_slot: other.slot,
                            to: furnace.pos,
                            to_slot: 1,
                            count,
                        },
                        2,
                    ),
                ]
            }
            None => vec![],
        },
        Action::Brewing => match (pos, memory.brewing_ingredient()) {
            (Some(stand), Some((chest, ingredient))) => vec![(
                Command::Brew {
                    id: ids(),
                    stand,
                    from: chest.pos,
                    slot: ingredient.slot,
                },
                0,
            )],
            _ => vec![],
        },
        Action::GrindstoneStrip => pos
            .map(|chest| {
                vec![(
                    Command::Strip {
                        id: ids(),
                        from: chest,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Every lamp of theirs, up to six, three short, three long, three short, together.
        Action::LampsBlink => memory
            .fixtures("redstone_lamp")
            .take(6)
            .flat_map(|lamp| {
                crate::workshop::blink_ticks()
                    .into_iter()
                    .map(|t| (lamp.pos, t))
                    .collect::<Vec<_>>()
            })
            .map(|(p, t)| {
                (
                    Command::ToggleBlock {
                        id: ids(),
                        pos: p,
                        to: toggle_to(event.id),
                    },
                    t,
                )
            })
            .collect(),
        Action::ComposterFills => pos
            .map(|p| {
                vec![(
                    Command::SetState {
                        id: ids(),
                        pos: p,
                        property: "level".to_string(),
                        value: "@up".to_string(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // The makings taken from their chest, what they make lying on the table.
        Action::CraftedForYou => match (pos, memory.recipe_at_hand()) {
            (Some(table), Some((recipe, chest, stacks))) => {
                let mut made: Vec<(Command, u64)> = stacks
                    .into_iter()
                    .enumerate()
                    .map(|(i, (stack, n))| {
                        (
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: stack.slot,
                                expect: stack.id.clone(),
                                count: n,
                                keep: String::new(),
                            },
                            i as u64 * 2,
                        )
                    })
                    .collect();
                made.push((
                    Command::DropItem {
                        id: ids(),
                        pos: [
                            table[0] as f64 + 0.5,
                            table[1] as f64 + 1.1,
                            table[2] as f64 + 0.5,
                        ],
                        item: ItemStack {
                            id: recipe.result.0.to_string(),
                            count: recipe.result.1,
                            slot: 0,
                        },
                        kept: String::new(),
                    },
                    10,
                ));
                made
            }
            _ => vec![],
        },
        Action::BellRings => pos
            .map(|p| {
                vec![(
                    Command::ToggleBlock {
                        id: ids(),
                        pos: p,
                        to: toggle_to(event.id),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // The game's own death line, one of a few, about them while they live.
        Action::DeathMessage => {
            const DEATHS: &[&str] = &[
                "death.fell.accident.generic",
                "death.attack.drown",
                "death.attack.inWall",
                "death.attack.outOfWorld",
                "death.attack.generic",
                "death.attack.starve",
                "death.attack.freeze",
            ];
            vec![(
                Command::GameMessage {
                    id: ids(),
                    player: target,
                    message: "death".to_string(),
                    text: DEATHS[seed % DEATHS.len()].to_string(),
                },
                0,
            )]
        }
        // What they last typed, in their own words, on a sign: up to four lines of fifteen.
        Action::WordsOnSign => pos
            .map(|p| {
                let lines = sign_lines(&memory.last_said);
                if lines.is_empty() {
                    return vec![];
                }
                sign_at(ids, p, lines)
            })
            .unwrap_or_default(),
        // One line of their sign, the last with text, reads a word of its own.
        Action::SignWord => pos
            .map(|sign| {
                vec![(
                    Command::SetSign {
                        id: ids(),
                        one_line: true,
                        pos: sign,
                        lines: vec![words(Kind::SignWord, "", 0)],
                        settle: 0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // The kill sign counts the kill: the number as it is now.
        Action::CountUp => match (pos, memory.sign_tally()) {
            (Some(sign), Some(tally)) => vec![(
                Command::SetSign {
                    id: ids(),
                    one_line: false,
                    pos: sign,
                    lines: vec![format!(
                        "{} {}",
                        tally.kills,
                        text::plural(&tally.kind, tally.kills)
                    )],
                    settle: 3,
                },
                0,
            )],
            _ => vec![],
        },
        // The very weapon, from its slot into their empty frame: a full frame refuses it and
        // it stays in the chest.
        Action::WeaponOnWall => match (their_weapon(memory), memory.decor("item_frame").next()) {
            (Some((chest, weapon)), Some(frame)) => vec![(
                Command::SetFrameItem {
                    id: ids(),
                    from: Some(chest.pos),
                    slot: weapon.slot,
                    entity: frame.id,
                    item: weapon.clone(),
                    swap: false,
                    drop_at: None,
                },
                0,
            )],
            _ => vec![],
        },
        // A post, and on it the head of what they kill most, facing the door.
        Action::HeadOnPost => match (pos, head_of_their_kills(memory), memory.usual_door()) {
            (Some(p), Some((_, head)), Some(door)) => {
                let yaw = yaw_toward(block_center(p), block_center(door));
                let rotation = ((yaw / 22.5).round() as i32).rem_euclid(16);
                vec![
                    (on_ground(ids, p, "minecraft:oak_fence"), 0),
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: [p[0], p[1] + 1, p[2]],
                            block: head.to_string(),
                            state: format!("rotation={rotation}"),
                            replace: vec![],
                            settle: 3,
                            wall: String::new(),
                            rest: true,
                            only: false,
                        },
                        10,
                    ),
                ]
            }
            _ => vec![],
        },
        // A row of graves at its place, two blocks apart: a stone each and a sign with the kind
        // and the count, up to five kinds, most killed first.
        Action::GraveEach => pos
            .map(|p| {
                memory
                    .tally
                    .iter()
                    .filter(|t| t.kills > 0)
                    .take(5)
                    .enumerate()
                    .flat_map(|(i, t)| {
                        let x = p[0] + i as i32 * 2;
                        let mut made = vec![(
                            on_ground(ids, [x, p[1] + 1, p[2]], "minecraft:cobblestone"),
                            i as u64 * 40,
                        )];
                        made.extend(
                            sign_at(
                                ids,
                                [x, p[1] + 1, p[2] + 1],
                                vec![t.kills.to_string(), text::plural(&t.kind, t.kills)],
                            )
                            .into_iter()
                            .map(move |(c, d)| (c, d + i as u64 * 40 + 10)),
                        );
                        made
                    })
                    .collect()
            })
            .unwrap_or_default(),
        // Their stand moved into the field, or one from the chest set there, turned to the
        // house, wearing the piece of their armour taken from the chest.
        Action::Scarecrow => match (
            pos,
            memory.in_used_container(is_armour),
            memory.home.as_ref(),
        ) {
            (Some(spot), Some((chest, piece)), Some(home)) => {
                let stand_at = [spot[0] as f64 + 0.5, spot[1] as f64, spot[2] as f64 + 0.5];
                let house = block_center(home.center);
                let take = Command::TakeItem {
                    id: ids(),
                    from: chest.pos,
                    slot: piece.slot,
                    expect: piece.id.clone(),
                    count: 1,
                    keep: "scarecrow".to_string(),
                };
                if let Some(stand) = memory.decor("armor_stand").next() {
                    vec![
                        (take, 0),
                        (
                            Command::MoveEntity {
                                id: ids(),
                                entity: stand.id,
                                to: stand_at,
                            },
                            2,
                        ),
                        (
                            Command::PoseStand {
                                id: ids(),
                                entity: stand.id,
                                at: Some(house),
                                follow: None,
                                seconds: 0.0,
                            },
                            3,
                        ),
                        (
                            Command::EquipEntity {
                                id: ids(),
                                entity: stand.id,
                                item: piece.clone(),
                                kept: "scarecrow".to_string(),
                            },
                            4,
                        ),
                    ]
                } else if let Some((_, item)) =
                    memory.in_used_container(|id| id == "minecraft:armor_stand")
                {
                    vec![
                        (take, 0),
                        (
                            Command::TakeItem {
                                id: ids(),
                                from: chest.pos,
                                slot: item.slot,
                                expect: item.id.clone(),
                                count: 1,
                                keep: String::new(),
                            },
                            2,
                        ),
                        (
                            Command::PlaceEntity {
                                id: ids(),
                                face: None,
                                of: "minecraft:armor_stand".to_string(),
                                pos: stand_at,
                                yaw: yaw_toward(stand_at, house),
                                seconds: 0.0,
                                kept: "scarecrow".to_string(),
                            },
                            4,
                        ),
                    ]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        },
        // Out around the door they use, two minutes, then home.
        Action::BeesOut => match (pos, memory.doorway_spot()) {
            (Some(hive), Some(door)) => vec![(
                Command::ReleaseBees {
                    id: ids(),
                    hive,
                    to: [door[0], door[1] + 1.5, door[2]],
                    seconds: 120.0,
                },
                0,
            )],
            _ => vec![],
        },
        Action::HorseMoved => leashed_horse(memory)
            .map(|horse| {
                vec![(
                    Command::MoveLeash {
                        id: ids(),
                        entity: horse.id,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Each crop of the word cut back to a seedling, a tenth of a second apart.
        Action::FieldWord => memory
            .field_word()
            .map(|(_, cells)| {
                cells
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| {
                        (
                            Command::SetState {
                                id: ids(),
                                pos: p,
                                property: "age".to_string(),
                                value: "0".to_string(),
                            },
                            i as u64 * 2,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        // Every animal in the pens, still, facing the house, two minutes.
        Action::FaceTheHouse => match memory.home.as_ref() {
            Some(home) => memory
                .penned()
                .into_iter()
                .take(12)
                .map(|c| {
                    (
                        Command::Hold {
                            id: ids(),
                            entity: c.id,
                            player: target,
                            seconds: 120.0,
                            at: Some(block_center(home.center)),
                            creep: false,
                        },
                        0,
                    )
                })
                .collect(),
            None => vec![],
        },
        // One more of what they keep, beside the others, gone by noon.
        // Never a mooshroom (the host refuses one for a while: shorn, it is a cow that stays).
        Action::OneMore => memory
            .kept_most()
            .filter(|(kind, _)| *kind != "mooshroom")
            .map(|(kind, at)| {
                let to_noon = 6_000u32.saturating_sub(now.day_time) as f32 / 20.0;
                vec![(
                    Command::PlaceEntity {
                        id: ids(),
                        face: None,
                        of: format!("minecraft:{kind}"),
                        pos: [at[0] + 1.5, at[1], at[2] + 1.5],
                        yaw: (seed % 360) as f32,
                        seconds: to_noon.clamp(60.0, 300.0),
                        kept: String::new(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::RowWithered => pos
            .and_then(|first| memory.field_rows().into_iter().find(|r| r[0] == first))
            .map(|row| {
                row.into_iter()
                    .enumerate()
                    .map(|(i, p)| {
                        (
                            Command::SetState {
                                id: ids(),
                                pos: p,
                                property: "age".to_string(),
                                value: "0".to_string(),
                            },
                            i as u64 * 2,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Action::EggsToDoor => memory
            .egg_line()
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    Command::DropItem {
                        id: ids(),
                        pos: p,
                        item: ItemStack {
                            id: "minecraft:egg".to_string(),
                            count: 1,
                            slot: 0,
                        },
                        kept: String::new(),
                    },
                    i as u64 * 10,
                )
            })
            .collect(),
        // The nearest animals run at them and hit, never past one half heart left: a nip at
        // stage 3, real harm from 4, no kill (owner).
        Action::Charge(count, damage, hits) => memory
            .animals_near_target()
            .into_iter()
            .take(count as usize)
            .enumerate()
            .map(|(i, c)| {
                (
                    Command::Charge {
                        id: ids(),
                        entity: c.id,
                        player: target,
                        damage,
                        hits,
                        seconds: 30.0,
                        spare: true,
                    },
                    i as u64 * 10,
                )
            })
            .collect(),
        Action::ShiftBed => pos
            .map(|bed| {
                vec![(
                    Command::ShiftBed {
                        id: ids(),
                        pos: bed,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Another colour, both halves, same place and facing: every bed colour but the new one
        // may be there.
        Action::SheetsChanged => pos
            .map(|bed| {
                let colour = crate::memory::COLOURS[seed % crate::memory::COLOURS.len()];
                let block = format!("minecraft:{colour}_bed");
                let only = crate::memory::COLOURS
                    .iter()
                    .map(|c| format!("minecraft:{c}_bed"))
                    .filter(|b| *b != block)
                    .collect();
                vec![(
                    Command::SwapBlock {
                        id: ids(),
                        pos: bed,
                        block,
                        only,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::SecondBed => pos
            .map(|bed| {
                let colour = crate::memory::COLOURS[seed % crate::memory::COLOURS.len()];
                vec![(
                    Command::BedBeside {
                        id: ids(),
                        pos: bed,
                        block: format!("minecraft:{colour}_bed"),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::WarpAnimal => match memory.warp_animal() {
            Some(animal) => {
                let w = crate::warped::warp(memory.times(event.id), seed, animal.named);
                vec![
                    (
                        Command::WarpMob {
                            id: ids(),
                            entity: animal.id,
                            player: target,
                            seconds: 60.0,
                            scale: w.scale,
                            flip: w.flip,
                            creep: w.creep,
                            to: if w.doorway {
                                memory.doorway_spot()
                            } else {
                                None
                            },
                        },
                        0,
                    ),
                    // Its own model bent on their screen while it lasts, as the fight's are.
                    (
                        Command::Hint {
                            id: ids(),
                            player: target,
                            hint: "warp_models".to_string(),
                            pos: None,
                            seconds: 60.0,
                            detail: animal.id.to_string(),
                        },
                        0,
                    ),
                ]
            }
            None => vec![],
        },
        // The lights behind them out, nearest first, three quarters of a second apart, each
        // back a minute after it went.
        Action::LightsOut => memory
            .lights_behind()
            .into_iter()
            .enumerate()
            .map(|(i, (p, id))| {
                (
                    Command::HideBlock {
                        id: ids(),
                        pos: p,
                        seconds: 60.0,
                        only: vec![id],
                    },
                    i as u64 * 15,
                )
            })
            .collect(),
        Action::SoulSwap => match memory.soulable_light() {
            Some((p, id)) => crate::lights::soul_of(&id)
                .map(|soul| {
                    vec![(
                        Command::SwapBlock {
                            id: ids(),
                            pos: p,
                            block: soul.to_string(),
                            only: vec![id.clone()],
                        },
                        0,
                    )]
                })
                .unwrap_or_default(),
            None => vec![],
        },
        // A candle, lit, on the ground where they died.
        Action::DeathLight => pos
            .map(|p| {
                vec![(
                    Command::SetBlock {
                        id: ids(),
                        pos: p,
                        block: "minecraft:candle".to_string(),
                        state: "lit=true".to_string(),
                        replace: vec![],
                        settle: 3,
                        wall: String::new(),
                        rest: true,
                        only: false,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // The same light a few blocks on, against a wall if nothing else holds it, and then
        // the old one gone: a light that finds no place stays where they put it.
        Action::LightTaken => match (&memory.habits.last_light, memory.light_taken_to()) {
            (Some((from, id)), Some(to)) => {
                let (block, wall) = crate::memory::light_kinds(id);
                vec![
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: to,
                            block,
                            state: String::new(),
                            replace: vec![],
                            settle: 3,
                            wall,
                            rest: false,
                            only: false,
                        },
                        0,
                    ),
                    (
                        Command::SetBlock {
                            id: ids(),
                            pos: *from,
                            block: "minecraft:air".to_string(),
                            state: String::new(),
                            replace: vec![id.clone()],
                            settle: 0,
                            wall: String::new(),
                            rest: false,
                            only: true,
                        },
                        2,
                    ),
                ]
            }
            _ => vec![],
        },
        Action::CaveLit => memory
            .cave_torches()
            .into_iter()
            .enumerate()
            .map(|(i, p)| (torch(ids, p, 4), i as u64 * 40))
            .collect(),
        Action::WallSwap(suffix) => pos
            .and_then(|p| memory.walls.iter().find(|(q, _)| *q == p))
            .and_then(|(p, id)| {
                // A light burns with soul fire (`torch_blue`), a pane becomes bars
                // (`window_bars`); anything else another member of its family, by the seed:
                // any wood, any colour.
                let swapped = match suffix {
                    "torch" => soul_light(id),
                    "glass_pane" => Some("minecraft:iron_bars".to_string()),
                    _ => {
                        let others = crate::memory::cousins(id);
                        (!others.is_empty()).then(|| others[seed % others.len()].clone())
                    }
                };
                swapped.map(|to| (*p, id.clone(), to))
            })
            // Swapped with its state kept (facing, axis, half, hanging): rebuilt in its default
            // state a wall torch fell, stairs turned, a double slab was half of one.
            .map(|(p, id, to)| {
                vec![(
                    Command::SwapBlock {
                        id: ids(),
                        pos: p,
                        block: to,
                        only: vec![id],
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Asleep, a figure comes to the bed (`bedside`): a lead, or a crossbow and a shot.
        Action::ShotInBed(damage) => bedside(ids, memory, target, pos, damage, "shoot"),
        Action::Choke(damage) => bedside(ids, memory, target, pos, damage, ""),
        // The same walk; it lies down on the floor beside the bed, facing them, until they wake.
        Action::LiesBeside => bedside(ids, memory, target, pos, 0.0, "lie"),
        Action::BedBreaks => pos
            .map(|bed| {
                vec![
                    (
                        Command::CancelSleep {
                            id: ids(),
                            player: target,
                            seconds: 0.0,
                        },
                        0,
                    ),
                    (
                        Command::BreakBlock {
                            id: ids(),
                            pos: bed,
                        },
                        3,
                    ),
                ]
            })
            .unwrap_or_default(),
        Action::Figure(where_, seconds, vanish) => pos
            .map(|p| {
                let on_ground = p[1] <= -1_000;
                let standing = if on_ground {
                    [p[0] as f64 + 0.5, -10_000.0, p[2] as f64 + 0.5]
                } else {
                    [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5]
                };
                let path: Vec<Pos> = match where_ {
                    FigureWhere::Route => memory
                        .route()
                        .map(|r| {
                            r.points
                                .iter()
                                .map(|q| [q[0] as f64 + 0.5, -10_000.0, q[2] as f64 + 0.5])
                                .collect()
                        })
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };
                vec![(
                    Command::Figure {
                        id: ids(),
                        figure: 1,
                        skin_of: target,
                        pos: standing,
                        path,
                        held: memory
                            .favourites
                            .first()
                            .map(|(id, _)| ItemStack {
                                id: id.clone(),
                                count: 1,
                                slot: 0,
                            })
                            .unwrap_or_default(),
                        name_shown: matches!(where_, FigureWhere::Far | FigureWhere::Behind),
                        seconds,
                        vanish_within: vanish,
                        mirror: matches!(where_, FigureWhere::Mirror),
                        once_seen: matches!(where_, FigureWhere::Crowd),
                        copy_crouch: matches!(where_, FigureWhere::FarCopying),
                        swinging: matches!(where_, FigureWhere::FarWorking),
                        upside_down: false,
                        crawls: false,
                        rushes: false,
                        rides_behind: false,
                        together: 0,
                        lies: false,
                        falls: false,
                        walks: false,
                        faces: None,
                        sunk: false,
                        jumps: false,
                        rides: 0,
                        eats: false,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        // Until dawn: the night's remaining ticks, a minute at least, ten minutes at most.
        // In the Nether: the ghasts' cries are their name for five minutes, and one cries now,
        // far off, so it is heard.
        Action::GhastName => {
            let at = memory.last.pos;
            let angle = (seed % 8) as f64 * std::f64::consts::FRAC_PI_4;
            vec![
                (
                    Command::Hint {
                        id: ids(),
                        player: target,
                        hint: "ghast_name".to_string(),
                        pos: None,
                        seconds: 300.0,
                        detail: memory.target_name.clone(),
                    },
                    0,
                ),
                (
                    Command::PlaySound {
                        id: ids(),
                        player: target,
                        sound: "minecraft:entity.ghast.ambient".to_string(),
                        pos: [
                            at[0] + 28.0 * angle.cos(),
                            at[1] + 8.0,
                            at[2] + 28.0 * angle.sin(),
                        ],
                        volume: 2.5,
                        pitch: 0.9,
                    },
                    20,
                ),
            ]
        }
        // An enderman near them holds what they build with most. The block comes from
        // `crate::elsewhere`, which knows what they build at home; here, the nearest enderman.
        Action::EnderHolds => {
            let at = block_of(memory.last.pos);
            let enderman = memory
                .last
                .hostiles_near
                .iter()
                .filter(|m| m.kind == "enderman")
                .min_by(|a, b| {
                    distance(block_of(a.pos), at).total_cmp(&distance(block_of(b.pos), at))
                });
            match (enderman, written) {
                (Some(e), Some(block)) => vec![(
                    Command::CarryBlock {
                        id: ids(),
                        entity: e.id,
                        block: block.to_string(),
                    },
                    0,
                )],
                _ => vec![],
            }
        }
        // Runs another event: the brain picks it (`Brain::consider_back`).
        Action::WhileAway => vec![],
        // From the dark behind them and to one side, the figure with a bow for a moment, then
        // the arrow: past their head with no harm, or at them.
        Action::Shoot(damage) => {
            let at = memory.last.pos;
            let yaw = (memory.last.yaw as f64).to_radians();
            let back = [yaw.sin(), -yaw.cos()];
            let right = [back[1], -back[0]];
            let side = if seed.is_multiple_of(2) { 1.0 } else { -1.0 };
            let from = [
                at[0] + back[0] * 12.0 + right[0] * side * 8.0,
                -10_000.0,
                at[2] + back[1] * 12.0 + right[1] * side * 8.0,
            ];
            let aim = if damage > 0.0 {
                [at[0], at[1] + 1.1, at[2]]
            } else {
                // A block and a little beside their head, on the side it shoots from.
                [
                    at[0] + right[0] * side * 1.3,
                    at[1] + 1.7,
                    at[2] + right[1] * side * 1.3,
                ]
            };
            let mut shown = figure_shown(
                ids,
                memory,
                target,
                [from[0].floor() as i32, -10_000, from[2].floor() as i32],
                3.0,
                false,
            );
            if let Command::Figure { held, .. } = &mut shown {
                *held = ItemStack {
                    id: "minecraft:bow".to_string(),
                    count: 1,
                    slot: 0,
                };
            }
            vec![
                (shown, 0),
                (
                    Command::Shoot {
                        id: ids(),
                        player: target,
                        from,
                        at: aim,
                        damage,
                    },
                    20,
                ),
            ]
        }
        // A potion from the dark at their feet: what darkens or slows, never what harms.
        Action::SplashDark => {
            let (effect, seconds) = [
                ("darkness", 10.0),
                ("slowness", 8.0),
                ("blindness", 5.0),
                ("levitation", 1.5),
                ("nausea", 8.0),
            ][seed % 5];
            let at = memory.last.pos;
            let yaw = (memory.last.yaw as f64).to_radians();
            vec![(
                Command::SplashPotion {
                    id: ids(),
                    player: target,
                    from: [at[0] + yaw.sin() * 7.0, -10_000.0, at[2] - yaw.cos() * 7.0],
                    effect: effect.to_string(),
                    seconds,
                },
                0,
            )]
        }
        // Its brew, in the chest they use: night vision and darkness at once, for the night.
        Action::BrewLeft => match memory.used_container() {
            Some(chest) => vec![(
                Command::PutPotion {
                    id: ids(),
                    into: chest.pos,
                    name: "for the night".to_string(),
                    effects: vec![
                        ("night_vision".to_string(), 120.0),
                        ("darkness".to_string(), 60.0),
                    ],
                },
                0,
            )],
            None => vec![],
        },
        // One of their potions, another kind, its name as it was.
        Action::PotionChanged => match memory.in_used_container(|id| id.ends_with("potion")) {
            Some((chest, stack)) => vec![(
                Command::ChangePotion {
                    id: ids(),
                    pos: chest.pos,
                    slot: stack.slot,
                    into: [
                        "night_vision",
                        "invisibility",
                        "water_breathing",
                        "fire_resistance",
                        "leaping",
                        "swiftness",
                        "slow_falling",
                    ][seed % 7]
                        .to_string(),
                },
                0,
            )],
            None => vec![],
        },
        // A cloud on the step outside the door they use, for the morning.
        Action::DoorCloud => match (memory.usual_door(), memory.home.as_ref()) {
            (Some(door), Some(home)) => {
                let step = step_toward(home.center, door);
                vec![(
                    Command::EffectCloud {
                        id: ids(),
                        pos: [
                            (door[0] + step[0]) as f64 + 0.5,
                            door[1] as f64,
                            (door[2] + step[1]) as f64 + 0.5,
                        ],
                        effect: if seed.is_multiple_of(2) {
                            "slowness"
                        } else {
                            "weakness"
                        }
                        .to_string(),
                        seconds: 600.0,
                        radius: 1.5,
                    },
                    0,
                )]
            }
            _ => vec![],
        },
        // Every frame of the house shows the one thing, to them alone, for three minutes.
        Action::FramesShow => match frame_shows(memory) {
            Some(item) => {
                let frames: Vec<u32> = memory.decor("item_frame").map(|d| d.id).collect();
                vec![(
                    Command::FramesShow {
                        id: ids(),
                        player: target,
                        frames,
                        item,
                        seconds: 180.0,
                    },
                    0,
                )]
            }
            None => vec![],
        },
        // The next time their bobber sits in the water, within two minutes, it goes under and
        // stays under ten seconds, nothing on it.
        Action::BobberUnder => vec![(
            Command::Bobber {
                id: ids(),
                player: target,
                seconds: 120.0,
                under: 10.0,
            },
            0,
        )],
        // One animal near them, now and then bent, comes at them very slowly or at a run; once
        // there it stares and goes, turns and walks off, or strikes once (never below half a heart).
        Action::MobComes(pace) => {
            let at = block_of(memory.last.pos);
            let near: Vec<&crate::memory::Creature> = memory
                .last
                .creatures_near
                .iter()
                .filter(|c| !c.pet && c.kind != "villager" && c.kind != "iron_golem")
                .filter(|c| (8.0..=28.0).contains(&distance(block_of(c.pos), at)))
                .collect();
            let Some(animal) = near.get(seed % near.len().max(1)).copied() else {
                return vec![];
            };
            let then = ["stare", "leave", "hit"][(seed / 3) % 3];
            let mut made = vec![(
                Command::Approach {
                    id: ids(),
                    entities: vec![animal.id],
                    player: target,
                    stop: 1.6,
                    seconds: 90.0,
                    pace: Some(pace),
                    then: then.to_string(),
                    // A nip, half a heart: these run from stage three, the owner's nip stage.
                    damage: if then == "hit" { 1.0 } else { 0.0 },
                    kept: String::new(),
                    to: None,
                },
                0,
            )];
            if seed.is_multiple_of(2) {
                made.push((
                    Command::Hint {
                        id: ids(),
                        player: target,
                        hint: "warp_models".to_string(),
                        pos: None,
                        seconds: 90.0,
                        detail: animal.id.to_string(),
                    },
                    0,
                ));
            }
            made
        }
        // Their animals, together, walk up to them and stop four blocks off, facing them.
        Action::Pack => {
            let herd: Vec<u32> = memory
                .animals_near_target()
                .iter()
                .take(12)
                .map(|c| c.id)
                .collect();
            if herd.len() < 3 {
                return vec![];
            }
            vec![(
                Command::Approach {
                    id: ids(),
                    entities: herd,
                    player: target,
                    stop: 4.0,
                    seconds: 40.0,
                    pace: None,
                    then: String::new(),
                    damage: 0.0,
                    kept: String::new(),
                    to: None,
                },
                0,
            )]
        }
        // The monsters near hold and stare, and come closer whenever they look away; three
        // minutes, then back where they stood and as they were.
        Action::HostilesClose => memory
            .last
            .hostiles_near
            .iter()
            .take(16)
            .map(|mob| {
                (
                    Command::Hold {
                        id: ids(),
                        at: None,
                        entity: mob.id,
                        player: target,
                        seconds: 180.0,
                        creep: true,
                    },
                    0,
                )
            })
            .collect(),
        // A push at the next edge they stand at, within two minutes, never more than three
        // blocks down.
        Action::Shove => vec![(
            Command::Shove {
                id: ids(),
                player: target,
                seconds: 120.0,
                most_drop: 3,
            },
            0,
        )],
        Action::HostilesStare => {
            let to_dawn = 23_000u32.saturating_sub(now.day_time) as f32 / 20.0;
            let seconds = to_dawn.clamp(60.0, 600.0);
            memory
                .last
                .hostiles_near
                .iter()
                .take(16)
                .map(|mob| {
                    (
                        Command::Hold {
                            id: ids(),
                            at: None,
                            entity: mob.id,
                            player: target,
                            seconds,
                            creep: false,
                        },
                        0,
                    )
                })
                .collect()
        }
        Action::Answers => vec![(
            Command::GameMessage {
                id: ids(),
                player: target,
                message: "says".to_string(),
                text: words(Kind::Reply, "", 0),
            },
            0,
        )],
        Action::RepeatsYou => {
            if memory.last_said.is_empty() {
                return vec![];
            }
            vec![(
                Command::GameMessage {
                    id: ids(),
                    player: target,
                    message: "says".to_string(),
                    text: memory.last_said.clone(),
                },
                0,
            )]
        }
    }
}

/// Whether the event's place may be the model's `Where?` answer rather than the memory's
/// default: the things the lodger leaves or makes at a spot.
pub fn flexible(event: &Event) -> bool {
    matches!(
        event.action,
        Action::PlaceTaken
            | Action::Sign
            | Action::Cairn
            | Action::Hole
            | Action::Grave
            | Action::Sapling
    )
}

/// The places the memory can name for `Where?`, each with its words, in a fixed order so the
/// model's answer maps back.
pub fn named_places(memory: &Memory) -> Vec<(&'static str, BlockPos)> {
    let mut places = Vec::new();
    if let Some(home) = &memory.home {
        if let Some(door) = home.doors.first() {
            let step = step_toward(home.center, *door);
            places.push((
                "by the door",
                [door[0] + step[0], door[1], door[2] + step[1]],
            ));
        }
        if let Some(bed) = home.bed {
            places.push(("on the bed", bed));
        }
    }
    if let Some(chest) = memory.used_container() {
        places.push((
            "by the chest they use",
            [chest.pos[0] + 1, chest.pos[1], chest.pos[2]],
        ));
    }
    if let Some(end) = memory.route_end() {
        places.push(("at the end of their path", end));
    }
    if let Some(place) = memory.place {
        places.push(("at its place", place));
    }

    places
}

/// The kind of line an event writes, if it writes one: what the model is asked to write.
pub fn text_kind(event: &Event) -> Option<Kind> {
    match event.action {
        Action::Sign => Some(Kind::SignWord),
        Action::KillSign => Some(Kind::KillSign),
        Action::Grave => Some(Kind::GraveSign),
        Action::NameAnimal => Some(Kind::AnimalName),
        Action::Says => Some(Kind::ChatLine),
        Action::SaysKills => Some(Kind::KillLine),
        Action::Whisper => Some(Kind::Whisper),
        Action::Answers => Some(Kind::Reply),
        Action::RenameOne | Action::ReturnUsed => Some(Kind::ItemName),
        Action::BookWord | Action::SignWord | Action::SignDeep => Some(Kind::SignWord),
        Action::DiarySold
        | Action::DiaryBrought
        | Action::LetterInChest
        | Action::LecternNotes
        | Action::PageInBook => Some(Kind::DiaryLine),
        Action::VillagerNamed => Some(Kind::AnimalName),
        _ => None,
    }
}

/// A sign placed and written: the block now, the text a tick later.
fn sign_at(
    ids: &mut impl FnMut() -> u64,
    pos: BlockPos,
    lines: Vec<String>,
) -> Vec<(Command, u64)> {
    // On the ground within three blocks, else on a wall there; the text finds it where it
    // landed. The dev client's signs went into the air on a slope.
    vec![
        (
            Command::SetBlock {
                id: ids(),
                pos,
                block: "minecraft:oak_sign".to_string(),
                state: String::new(),
                replace: GROUND_COVER.iter().map(|r| r.to_string()).collect(),
                settle: 3,
                wall: "minecraft:oak_wall_sign".to_string(),
                rest: false,
                only: false,
            },
            0,
        ),
        (
            Command::SetSign {
                id: ids(),
                one_line: false,
                pos,
                lines,
                settle: 3,
            },
            2,
        ),
    ]
}

/// The host took the favourite: the memory holds it now and the container's contents lose it.
/// One of their farm animals by the house (`Memory::penned`), of `kind` (any, empty), not a pet,
/// not in front of them within the distance the view rule keeps (the event's own spot is
/// elsewhere, or none, so the animal's was never checked), unnamed when `unnamed`. Until now
/// any creature near the target would do: a wild one, another player's, one they watched.
pub fn their_animal<'a>(
    memory: &'a Memory,
    kind: &str,
    unnamed: bool,
) -> Option<&'a crate::memory::Creature> {
    memory.penned().into_iter().find(|c| {
        (kind.is_empty() || c.kind == kind)
            && !c.pet
            && !(unnamed && c.named)
            && !memory.in_view(block_of(c.pos), ANIMAL_VIEW)
    })
}

/// How far an animal it moves, names or dyes, or a frame or stand it turns or dresses, is kept
/// out of their view: the view rule's distance (`events.min_distance`, 64 by default).
const ANIMAL_VIEW: f64 = 64.0;

/// Their frames or stands of a kind not in front of them: these events have no spot of their
/// own, so the view rule never looked, and a frame turned or a stand dressed as they watched.
fn unseen_decor<'a>(
    memory: &'a Memory,
    kind: &'a str,
) -> impl Iterator<Item = &'a crate::memory::Decor> + 'a {
    memory
        .decor(kind)
        .filter(move |d| !memory.in_view(d.pos, ANIMAL_VIEW))
}

/// What a `take` took, as its command said: the chest, the slot and the item. Held from that,
/// not from the plan's chest (the chest changed between the plan and the take, and the memory
/// held another thing than the host, or nothing, and a second take was kept for good).
pub fn took_this(memory: &mut Memory, taken: &crate::brain::Took) {
    if let Some(container) = memory.containers.iter_mut().find(|c| c.pos == taken.from) {
        if let Some(index) = container
            .contents
            .iter()
            .position(|s| s.slot == taken.slot && s.id == taken.item)
            .or_else(|| container.contents.iter().position(|s| s.id == taken.item))
        {
            if container.contents[index].count <= 1 {
                container.contents.remove(index);
            } else {
                container.contents[index].count -= 1;
            }
        }
    }
    memory.held = Some(Held {
        item: ItemStack {
            id: taken.item.clone(),
            count: 1,
            slot: 0,
        },
        from: taken.from,
        slot: taken.slot,
        taken_day: memory.done.last().map(|d| d.day).unwrap_or(0.0),
    });
}

/// What a `take` from a save without its `Took` took: the plan's chest's first favourite.
pub fn took(memory: &mut Memory, pos: Option<BlockPos>) {
    let Some(pos) = pos else { return };
    // The favourite `take` chose (`Memory::favourite_in_container`): the first of their
    // favourites in that chest. Until now the first favourite or else the first stack,
    // which was not always what the host took.
    let favourites: Vec<String> = memory.favourites.iter().map(|(id, _)| id.clone()).collect();
    let Some(container) = memory.containers.iter_mut().find(|c| c.pos == pos) else {
        return;
    };
    let Some(index) = favourites
        .iter()
        .find_map(|f| container.contents.iter().position(|s| s.id == *f))
    else {
        return;
    };
    let stack = &mut container.contents[index];
    let held = Held {
        item: ItemStack {
            id: stack.id.clone(),
            count: 1,
            slot: 0,
        },
        from: pos,
        slot: stack.slot,
        taken_day: memory.done.last().map(|d| d.day).unwrap_or(0.0),
    };
    if stack.count <= 1 {
        container.contents.remove(index);
    } else {
        stack.count -= 1;
    }
    memory.held = Some(held);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{find, CATALOGUE};
    use crate::memory::{Container, Creature, Decor, Fixture, Home, Tally};

    fn stack(id: &str, count: u32, slot: u32) -> ItemStack {
        ItemStack {
            id: id.to_string(),
            count,
            slot,
        }
    }

    fn memory() -> Memory {
        let mut memory = Memory {
            target: Some(0),
            ..Memory::default()
        };
        memory.last.online = false;
        memory.home = Some(Home {
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![[1, 65, 1]],
        });
        memory
            .favourites
            .push(("minecraft:iron_pickaxe".to_string(), 9));
        memory.containers.push(Container {
            pos: [2, 64, 0],
            kind: "minecraft:chest".to_string(),
            opens: 5,
            placed: true,
            days: 1,
            in_home: false,
            last_opened_day: 0.0,
            contents: vec![
                stack("minecraft:iron_pickaxe", 1, 4),
                stack("minecraft:cobblestone", 40, 5),
                stack("minecraft:raw_iron", 6, 6),
                stack("minecraft:coal", 9, 7),
                stack("minecraft:oak_sapling", 2, 8),
            ],
        });
        memory
    }

    #[test]
    fn it_takes_what_take_chose_holds_one_thing_and_waits_for_the_hour() {
        let mut memory = memory();
        // Their first favourite in no chest; the second in it, after a torch.
        memory.favourites = vec![
            ("minecraft:diamond_sword".to_string(), 9),
            ("minecraft:bread".to_string(), 5),
        ];
        memory.containers[0].contents = vec![
            stack("minecraft:torch", 8, 0),
            stack("minecraft:bread", 3, 5),
        ];
        let (_, chosen) = memory.favourite_in_container().unwrap();
        assert_eq!(chosen.slot, 5);
        took(&mut memory, Some([2, 64, 0]));
        let held = memory.held.clone().expect("held");
        assert_eq!((held.item.id.as_str(), held.slot), ("minecraft:bread", 5));
        assert!(memory.containers[0]
            .contents
            .iter()
            .any(|s| s.id == "minecraft:torch" && s.count == 8));
        // Holding it, no second take.
        assert!(!crate::brain::offer::meets(
            crate::events::Need::HoldsNothing,
            &memory,
            &Now::default()
        ));
        // A furnace is never where its favourite is found.
        let mut furnace = self::memory();
        furnace.containers[0].kind = "minecraft:furnace".to_string();
        assert!(furnace.favourite_in_container().is_none());

        // The hour, asked with the real clock: noon at noon, not before.
        let mut out = self::memory();
        out.last.online = true;
        let sky = find("wrong_sky").unwrap();
        let morning = Now {
            day_time: 1_000,
            ..Now::default()
        };
        let noon = Now {
            day_time: 6_000,
            ..Now::default()
        };
        assert!(!ready(sky, &out, &morning));
        assert!(ready(sky, &out, &noon));
        let footsteps = find("footsteps_follow").unwrap();
        out.last.walking = true;
        assert!(
            !ready(footsteps, &out, &noon),
            "a night's event waits for the night"
        );
    }

    #[test]
    fn its_kill_sign_counts_the_kind_it_wrote() {
        let mut memory = memory();
        memory.tally = vec![
            crate::memory::Tally {
                kind: "zombie".to_string(),
                kills: 9,
                ..Default::default()
            },
            crate::memory::Tally {
                kind: "cow".to_string(),
                kills: 5,
                ..Default::default()
            },
        ];
        memory.kill_sign_kind = "cow".to_string();
        assert_eq!(memory.sign_tally().map(|t| t.kind.as_str()), Some("cow"));
        memory.kill_sign_kind.clear();
        assert_eq!(
            memory.sign_tally().map(|t| t.kind.as_str()),
            Some("zombie"),
            "before it kept one"
        );
    }

    #[test]
    fn the_worn_path_and_the_knock_come_to_the_door_they_use() {
        let mut memory = memory();
        memory.home.as_mut().unwrap().doors.push([-3, 64, 0]);
        memory.last.pos = [0.5, 64.0, 0.5];
        for _ in 0..3 {
            memory.habit_door([-3, 64, 0]);
        }
        let mut rng = Rng::new(1);
        for _ in 0..8 {
            assert_eq!(
                place(find("path_worn").unwrap(), &memory, &mut rng),
                Some([-3, 64, 0])
            );
            assert_eq!(
                place(find("knock").unwrap(), &memory, &mut rng),
                Some([-3, 64, 0])
            );
        }
    }

    /// The memory with everything the catalogue could ask for.
    fn rich_memory() -> Memory {
        let mut memory = memory();
        memory.store("minecraft:dirt", 5);
        memory
            .plants
            .push(("minecraft:poppy".to_string(), [6, 64, 6]));
        memory
            .plants
            .push(("minecraft:pumpkin".to_string(), [-6, 64, 6]));
        // A book and quill and a map in the chest they use, for `book_word` and `map_marked`;
        // a potion, for `potion_changed`.
        memory.containers[0].contents.extend([
            stack("minecraft:writable_book", 1, 12),
            stack("minecraft:filled_map", 1, 13),
            stack("minecraft:potion", 1, 20),
        ]);
        memory.containers[0]
            .contents
            .push(stack("minecraft:shears", 1, 14));
        memory.containers[0]
            .contents
            .push(stack("minecraft:bone_meal", 9, 15));
        memory
            .plants
            .push(("minecraft:oak_sapling".to_string(), [5, 64, -5]));
        memory
            .holes
            .push(([7, 63, 0], "minecraft:dirt".to_string()));
        memory.containers.push(Container {
            pos: [-2, 64, 0],
            kind: "minecraft:chest".to_string(),
            opens: 2,
            placed: true,
            days: 1,
            in_home: false,
            last_opened_day: 0.0,
            contents: vec![stack("minecraft:dirt", 12, 0)],
        });
        memory.containers.push(Container {
            pos: [0, 64, 2],
            kind: "minecraft:furnace".to_string(),
            opens: 3,
            placed: true,
            days: 1,
            in_home: false,
            last_opened_day: 0.0,
            contents: vec![
                stack("minecraft:iron_ingot", 3, 2),
                stack("minecraft:coal", 3, 1),
            ],
        });
        for (kind, pos) in [
            ("jukebox", [3, 64, -2]),
            ("brewing_stand", [-3, 64, 2]),
            ("grindstone", [-3, 64, -1]),
            ("redstone_lamp", [0, 66, 0]),
            ("composter", [5, 64, 5]),
            ("crafting_table", [1, 64, 2]),
            ("hopper", [2, 65, 0]),
            ("ladder", [-6, 64, 0]),
            ("ladder", [-6, 65, 0]),
            ("ladder", [-6, 66, 0]),
            ("ladder", [-6, 67, 0]),
            ("flower_pot", [2, 65, 2]),
            ("shulker_box", [-4, 64, -4]),
            ("anvil", [-1, 64, 2]),
            ("lectern", [-1, 64, -2]),
            ("glass_pane", [0, 65, -3]),
            // A second window, for `every_window`.
            ("glass_pane", [4, 65, 3]),
            ("carpet", [1, 64, 1]),
            ("carpet", [2, 64, 1]),
            ("carpet", [3, 64, 1]),
            ("campfire", [6, 64, 6]),
            ("cauldron", [-2, 64, -2]),
            ("fence_gate", [8, 64, 0]),
            ("bell", [10, 65, 0]),
            ("note_block", [0, 64, -4]),
            ("lever", [2, 65, -2]),
            ("candle", [0, 65, 1]),
        ] {
            memory.fixtures.push(Fixture {
                kind: kind.to_string(),
                pos,
            });
        }
        for (kind, id, pos) in [
            ("item_frame", 101, [0, 66, -3]),
            ("item_frame", 102, [1, 66, -3]),
            ("item_frame", 103, [2, 66, -3]),
            ("painting", 104, [-2, 66, -3]),
            ("armor_stand", 105, [-2, 64, 2]),
        ] {
            memory.decor.push(Decor {
                kind: kind.to_string(),
                id,
                pos,
            });
        }
        for (kind, id, pet, named) in [
            ("cow", 201, false, false),
            ("sheep", 202, false, false),
            ("chicken", 203, false, false),
            ("villager", 204, false, false),
            ("iron_golem", 205, false, false),
            ("wolf", 206, true, true),
        ] {
            memory.last.creatures_near.push(Creature {
                id,
                kind: kind.to_string(),
                pos: [10.0, 64.0, 10.0],
                named,
                pet,
                baby: false,
            });
        }
        for (kind, id) in [("zombie", 301), ("skeleton", 302), ("spider", 303)] {
            memory.last.hostiles_near.push(Creature {
                id,
                kind: kind.to_string(),
                pos: [20.0, 64.0, 20.0],
                ..Creature::default()
            });
        }
        memory.tally.push(Tally {
            kind: "cow".to_string(),
            hits: 20,
            kills: 14,
        });
        // A disc, wood to burn, wart to brew, sticks to make torches of.
        for (id, count, slot) in [
            ("minecraft:music_disc_cat", 1, 12),
            ("minecraft:oak_planks", 32, 13),
            ("minecraft:nether_wart", 4, 14),
            ("minecraft:stick", 8, 15),
        ] {
            memory.containers[0].contents.push(stack(id, count, slot));
        }
        // A second kind, with a head; a sign of theirs; their sword; its kill sign.
        memory.tally.push(Tally {
            kind: "zombie".to_string(),
            hits: 5,
            kills: 3,
        });
        memory.fixtures.push(crate::memory::Fixture {
            kind: "sign".to_string(),
            pos: [-2, 64, -2],
        });
        memory
            .favourites
            .push(("minecraft:iron_sword".to_string(), 4));
        memory.containers[0]
            .contents
            .push(stack("minecraft:iron_sword", 1, 11));
        memory.landmarks.push(crate::memory::Done {
            event: "kill_sign".to_string(),
            pos: Some([3, 64, 0]),
            day: 0.5,
            ok: true,
            reason: String::new(),
        });
        memory.last_said = "who is there".to_string();
        memory.last.raining = true;
        memory.last.underground = true;
        memory.last.ate_day = 0.0;
        memory.last.walking = true;
        memory.containers[0]
            .contents
            .push(stack("minecraft:water_bucket", 1, 9));
        memory.containers[0]
            .contents
            .push(stack("minecraft:oak_pressure_plate", 2, 10));
        memory.fixtures.push(Fixture {
            kind: "redstone_lamp".to_string(),
            pos: [0, 66, 2],
        });
        memory.vehicles.push(Creature {
            id: 301,
            kind: "oak_boat".to_string(),
            pos: [20.0, 63.0, 0.0],
            named: false,
            pet: false,

            baby: false,
        });
        memory.vehicles.push(Creature {
            id: 302,
            kind: "minecart".to_string(),
            pos: [5.0, 64.0, 5.0],
            named: false,
            pet: false,

            baby: false,
        });
        memory.surroundings = Some(crate::memory::Surround {
            day: 0.0,
            around: [0, 64, 0],
            features: vec![
                craftkit::world::Feature {
                    kind: "water".to_string(),
                    pos: [30, 62, 0],
                    size: 20,
                    detail: String::new(),
                },
                craftkit::world::Feature {
                    kind: "crops".to_string(),
                    pos: [-12, 64, 0],
                    size: 6,
                    detail: String::new(),
                },
                craftkit::world::Feature {
                    kind: "snow".to_string(),
                    pos: [0, 64, 12],
                    size: 30,
                    detail: String::new(),
                },
                craftkit::world::Feature {
                    kind: "village".to_string(),
                    pos: [60, 64, 60],
                    size: 40,
                    detail: String::new(),
                },
            ],
        });
        memory.scanned.push(("oak_door".to_string(), [58, 64, 61]));
        memory.scanned.push(("bell".to_string(), [60, 65, 60]));
        memory.scanned.push(("lantern".to_string(), [62, 67, 60]));
        memory.rang_bell_now = Some([60, 65, 60]);
        memory.scanned.push(("oak_door".to_string(), [58, 64, 60]));
        memory
            .scanned
            .push(("spruce_door".to_string(), [66, 64, 58]));
        for (kind, pos) in [
            ("stairs", [4, 65, 3]),
            ("banner", [0, 66, 3]),
            ("piston", [-3, 64, 3]),
            ("jukebox", [-3, 64, -3]),
        ] {
            memory.fixtures.push(Fixture {
                kind: kind.to_string(),
                pos,
            });
        }
        for (id, count, slot) in [
            ("minecraft:iron_helmet", 1, 11),
            ("minecraft:poppy", 3, 12),
            ("minecraft:carrot", 5, 13),
            ("minecraft:red_wool", 4, 14),
            ("minecraft:blue_wool", 2, 15),
            ("minecraft:red_concrete", 7, 16),
            ("minecraft:lime_wool", 1, 17),
            ("minecraft:black_wool", 9, 18),
            ("minecraft:white_bed", 1, 19),
        ] {
            memory.containers[0].contents.push(stack(id, count, slot));
        }
        memory.containers[2]
            .contents
            .push(stack("minecraft:coal", 3, 1));
        memory.scanned.push(("iron_ore".to_string(), [20, 40, 20]));
        memory
            .walls
            .push(([5, 65, 5], "minecraft:oak_planks".to_string()));
        memory
            .walls
            .push(([5, 66, 5], "minecraft:glass_pane".to_string()));
        memory
            .walls
            .push(([1, 65, 1], "minecraft:torch".to_string()));
        // A candle of theirs, as the game records one: among the blocks they placed.
        memory
            .walls
            .push(([0, 65, 1], "minecraft:candle".to_string()));
        memory.routes.push(crate::memory::Route {
            points: (0..20).map(|i| [3 + i * 2, 64, 0]).collect(),
            walked: 3,
            first_day: 0.0,
        });
        // A field, a piece of armour and a stand, a hive, a horse on a lead, pens and a coop.
        memory.field = (0..9)
            .flat_map(|x| (0..6).map(move |z| [10 + x, 1, 30 + z]))
            .collect();
        memory.containers[0]
            .contents
            .push(stack("minecraft:iron_helmet", 1, 10));
        memory.decor.push(crate::memory::Decor {
            kind: "armor_stand".to_string(),
            id: 90,
            pos: [1, 64, 1],
        });
        memory.fixtures.push(crate::memory::Fixture {
            kind: "beehive".to_string(),
            pos: [-3, 64, 3],
        });
        for (id, kind, x) in [
            (80, "horse", 6.5),
            (81, "sheep", 8.5),
            (82, "sheep", 9.5),
            (83, "chicken", -9.5),
            (84, "chicken", -8.5),
        ] {
            memory.last.creatures_near.push(crate::memory::Creature {
                id,
                kind: kind.to_string(),
                pos: [x, 64.0, 6.5],
                named: false,
                pet: false,

                baby: false,
            });
        }
        memory.last.leashed.push(80);
        // A pig right by where they stand.
        memory.last.creatures_near.push(crate::memory::Creature {
            id: 85,
            kind: "pig".to_string(),
            pos: [3.5, 1.0, 3.5],
            named: false,
            pet: false,

            baby: false,
        });
        // Two sheep grazing near them, with the pig a herd (`pack`).
        for (id, x) in [(86, -5.5), (87, -6.5)] {
            memory.last.creatures_near.push(crate::memory::Creature {
                id,
                kind: "sheep".to_string(),
                pos: [x, 1.0, -4.5],
                named: false,
                pet: false,

                baby: false,
            });
        }
        // A cow of theirs in sight, twenty blocks off.
        memory.last.creatures_near.push(crate::memory::Creature {
            id: 77,
            kind: "cow".to_string(),
            pos: [0.5, 1.0, 20.5],
            named: false,
            pet: false,

            baby: false,
        });
        // A torch of theirs behind where they stand (the memory's player is at the origin,
        // facing +z).
        memory
            .walls
            .push(([0, 1, -6], "minecraft:torch".to_string()));
        // A tunnel of theirs, east from far under the house, with an ore ahead, two lights in
        // it, and obsidian for a frame; the memory's player stands in it.
        memory.tunnels.push(crate::tunnels::Tunnel {
            blocks: (0..14)
                .flat_map(|x| [[x - 6, 0, 0], [x - 6, 1, 0]])
                .collect(),
            first_day: 0.2,
            last_day: 0.4,
        });
        memory.ores.push(("iron_ore".to_string(), [14, 0, 1]));
        memory.habits.cave_lights.extend([[-4, 1, 0], [4, 1, 0]]);
        memory.placed_obsidian = (0..10).map(|i| [20, 64 + i, 20]).collect();
        memory.last_fished = Some([4, 64, 30]);
        memory.store("minecraft:cobblestone", 4);
        memory.containers[0]
            .contents
            .push(stack("minecraft:flint_and_steel", 1, 16));
        memory.containers[0]
            .contents
            .push(stack("minecraft:ladder", 8, 17));
        // A death, and their last torch on the route.
        memory.milestones.push(crate::memory::Milestone {
            kind: "died".to_string(),
            day: 0.5,
            pos: Some([20, 64, 20]),
            detail: String::new(),
        });
        memory.habits.last_light = Some(([5, 64, 0], "minecraft:wall_torch".to_string()));
        // A cave nobody went near, and a lantern in the chest.
        for x in (30..=60).step_by(3) {
            for y in [30, 33] {
                for z in [40, 43] {
                    memory.caves.push([x, y, z]);
                }
            }
        }
        memory.containers[0]
            .contents
            .push(stack("minecraft:lantern", 1, 9));
        memory
    }

    fn ids() -> impl FnMut() -> u64 {
        let mut next = 1;
        move || {
            next += 1;
            next
        }
    }

    #[test]
    fn a_door_toggles_a_chest_opens_its_lid_and_a_take_removes_the_favourite() {
        let memory = memory();
        let mut rng = Rng::new(1);

        let door = find("open_door").unwrap();
        let p = place(door, &memory, &mut rng);
        assert_eq!(p, Some([3, 64, 0]));
        let made = commands(door, p, &memory, &mut ids(), &Now::default(), None);
        assert!(matches!(
            made[0].0,
            Command::ToggleBlock {
                pos: [3, 64, 0],
                ..
            }
        ));

        let chest = find("open_chest").unwrap();
        let made = commands(
            chest,
            place(chest, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::OpenLid {
                    pos: [2, 64, 0],
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let take = find("take").unwrap();
        let made = commands(
            take,
            place(take, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::TakeItem {
                    from: [2, 64, 0],
                    slot: 4,
                    count: 1,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn taking_moves_the_item_from_the_container_to_the_hand_and_returning_puts_it_back() {
        let mut memory = memory();
        took(&mut memory, Some([2, 64, 0]));

        let held = memory.held.as_ref().unwrap();
        assert_eq!(held.item.id, "minecraft:iron_pickaxe");
        assert_eq!(held.slot, 4);
        assert!(memory.containers[0]
            .contents
            .iter()
            .all(|s| s.id != "minecraft:iron_pickaxe"));

        let back = find("return_taken").unwrap();
        let mut rng = Rng::new(1);
        let made = commands(
            back,
            place(back, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::PutItem {
                    into: [2, 64, 0],
                    slot: Some(4),
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn shuffling_moves_a_stack_to_an_empty_slot_and_splitting_halves_one() {
        let memory = memory();
        let mut rng = Rng::new(1);

        let shuffle = find("shuffle_chest").unwrap();
        let made = commands(
            shuffle,
            place(shuffle, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        match &made[0].0 {
            Command::MoveItem {
                from, to, to_slot, ..
            } => {
                assert_eq!(from, to);
                assert!(![4, 5, 6, 7, 8].contains(to_slot));
            }
            other => panic!("{other:?}"),
        }

        let split = find("stack_split").unwrap();
        let made = commands(
            split,
            place(split, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::MoveItem {
                    from_slot: 5,
                    count: 20,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn tidying_sorts_a_chest_into_its_first_slots_with_one_empty_slot_as_the_hand() {
        let contents = vec![
            stack("minecraft:torch", 5, 0),
            stack("minecraft:apple", 3, 1),
            stack("minecraft:cobblestone", 40, 9),
        ];
        let moves = tidy_moves(&contents, 27);

        // Replay the moves and check the order that comes out.
        let mut slots: Vec<Option<ItemStack>> = vec![None; 27];
        for s in &contents {
            slots[s.slot as usize] = Some(s.clone());
        }
        for (from, to, _) in &moves {
            assert!(slots[*to as usize].is_none(), "move onto a full slot");
            slots[*to as usize] = slots[*from as usize].take();
        }
        let ids: Vec<&str> = slots
            .iter()
            .take(3)
            .map(|s| s.as_ref().unwrap().id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "minecraft:apple",
                "minecraft:cobblestone",
                "minecraft:torch"
            ]
        );

        let full: Vec<ItemStack> = (0..27).map(|i| stack("minecraft:dirt", 1, i)).collect();
        assert!(tidy_moves(&full, 27).is_empty(), "no hand, no tidying");

        // Two equal stacks, already sorted: nothing to do, with a hand to spare.
        let sorted = vec![
            stack("minecraft:cobblestone", 64, 0),
            stack("minecraft:cobblestone", 64, 1),
            stack("minecraft:torch", 5, 2),
        ];
        assert!(
            tidy_moves(&sorted, 27).is_empty(),
            "a sorted chest is left alone"
        );
    }

    #[test]
    fn cooking_moves_a_raw_thing_and_a_fuel_and_taking_output_empties_the_furnace() {
        let memory = rich_memory();
        let mut rng = Rng::new(1);

        let cook = find("cook_one").unwrap();
        let made = commands(
            cook,
            place(cook, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        // The furnace holds fuel already, so one move: the raw thing. Cobblestone, in slot 5,
        // is the first smeltable thing in the chest.
        assert_eq!(made.len(), 1);
        assert!(
            matches!(
                made[0].0,
                Command::MoveItem {
                    from_slot: 5,
                    to_slot: 0,
                    count: 1,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let out = find("take_output").unwrap();
        let made = commands(
            out,
            place(out, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::MoveItem {
                    from: [0, 64, 2],
                    from_slot: 2,
                    to: [2, 64, 0],
                    count: 3,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn a_kill_sign_counts_the_kills_and_a_grave_names_the_kind() {
        let memory = rich_memory();
        let mut rng = Rng::new(1);

        let sign = find("kill_sign").unwrap();
        let made = commands(
            sign,
            place(sign, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::SetBlock {
                    pos: [5, 64, 0],
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
        match &made[1].0 {
            Command::SetSign { lines, .. } => assert!(
                lines[0].contains("14") || lines[0].contains("cow"),
                "{lines:?}"
            ),
            other => panic!("{other:?}"),
        }

        let grave = find("grave").unwrap();
        let made = commands(
            grave,
            place(grave, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 4);
        assert!(
            matches!(&made[3].0, Command::SetSign { lines, .. } if lines[0].contains("cow") || lines[0] == "rest")
        );

        let line = find("kill_line").unwrap();
        let made = commands(line, None, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[0].0, Command::GameMessage { message, text, .. } if message == "says" && text.contains("cow"))
        );
    }

    #[test]
    fn a_worn_path_replaces_grass_only_and_a_hole_digs_natural_ground_only() {
        let memory = rich_memory();
        let mut rng = Rng::new(1);

        let path = find("path_worn").unwrap();
        let made = commands(
            path,
            place(path, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(made.len() >= 6, "{} blocks of path", made.len());
        for (command, _) in &made {
            match command {
                Command::SetBlock { block, replace, .. } => {
                    assert_eq!(block, "minecraft:dirt_path");
                    assert_eq!(replace, &["minecraft:grass_block".to_string()]);
                }
                other => panic!("{other:?}"),
            }
        }

        let hole = find("hole").unwrap();
        let made = commands(
            hole,
            place(hole, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 4);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, replace, .. } if block == "minecraft:air" && replace.contains(&"minecraft:grass_block".to_string()) && !replace.iter().any(|r| r.contains("planks")))
        );
        assert!(
            matches!(&made[3].0, Command::SetBlock { block, .. } if block == "minecraft:torch")
        );
    }

    #[test]
    fn every_event_makes_at_least_one_command_when_the_memory_has_everything() {
        let memory = rich_memory();
        let mut rng = Rng::new(7);
        let now = Now::default();

        for event in CATALOGUE {
            if event.id == "wait"
                || event.id == "return_taken"
                // It holds out what it took, which this memory has not: the brain's test.
                || event.id == "the_offer"
                || event.id == "place_taken"
                || event.id == "returned_used"
                || event.id == "loot_gift"
                || event.id == "door_opens_itself"
                // Its house must stand first: its own test below.
                || event.id == "house_grows"
                || event.id == "its_lights_on"
                || event.id == "respawn_its_bed"
                // A zombie near them, which this memory has not: its own test.
                || event.id == "wears_your_face"
                // Away from home, which this memory is not: its own test.
                || event.id == "figure_on_map"
                // Back through a portal, which this memory never went: its own test.
                || event.id == "waits_at_the_portal"
                || event.id == "firework_over_its_house"
                // What it took, which this memory holds not: its own test.
                || event.id == "chicken_lays"
                // Something held, monsters away: its own test.
                || event.id == "slips_from_hand"
                // Their route, which this memory has not: its own test.
                || event.id == "camp_nearby"
                // Two doors or more, which this memory has not: its own test.
                || event.id == "every_door_opens"
                // A cave known, which this memory has not: its own test.
                || event.id == "prints_to_the_cave"
                // The block comes from home and the enderman from the End: `elsewhere`'s tests.
                || event.id == "ender_holds"
                // Runs another event in its stead: the brain's test.
                || event.id == "busy_away"
            {
                continue;
            }
            let pos = place(event, &memory, &mut rng);
            let made = commands(event, pos, &memory, &mut ids(), &now, None);
            assert!(!made.is_empty(), "{} makes no command", event.id);
            for (command, _) in &made {
                assert!(command.id() > 1, "{} gave a command no id", event.id);
                // No made-up ids: `woke_elsewhere` sent the player as entity u32::MAX - 0,
                // which no host knows.
                let text = serde_json::to_string(command).unwrap();
                assert!(
                    !text.contains("4294967") && !text.contains(&format!("{}", i64::MAX)),
                    "{} sent a placeholder id: {text}",
                    event.id
                );
            }
        }
    }

    #[test]
    fn boats_move_across_the_water_or_drift_and_the_village_is_scanned_then_opened() {
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let now = Now::default();

        let moved = find("boat_moved").unwrap();
        let made = commands(moved, None, &memory, &mut ids(), &now, None);
        match &made[0].0 {
            Command::MoveEntity { entity, to, .. } => {
                assert_eq!(*entity, 301);
                assert!((to[0] - 40.0).abs() < 0.01, "across the water: {to:?}");
            }
            other => panic!("{other:?}"),
        }

        let drift = find("boat_drifts").unwrap();
        let made = commands(drift, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(made[0].0, Command::PushEntity { entity: 301, .. }),
            "{:?}",
            made[0].0
        );
        let rolls = find("minecart_rolls").unwrap();
        let made = commands(rolls, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(made[0].0, Command::PushEntity { entity: 302, .. }),
            "{:?}",
            made[0].0
        );

        let scan = find("scan_village").unwrap();
        let made = commands(scan, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::Scan { kinds, .. } if kinds.iter().any(|k| k == "minecraft:bell"))
        );
        let doors = find("village_doors").unwrap();
        let made = commands(
            doors,
            place(doors, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(
                made[0].0,
                Command::ToggleBlock {
                    pos: [58, 64, 61],
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let trample = find("trample").unwrap();
        let made = commands(
            trample,
            place(trample, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(made.len() >= 2);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, replace, .. } if block == "minecraft:dirt" && replace == &["minecraft:farmland".to_string()])
        );
    }

    #[test]
    fn an_animal_is_led_to_the_far_end_of_the_route_and_the_path_of_torches_follows_it() {
        let memory = rich_memory();
        let mut rng = Rng::new(4);
        let now = Now::default();

        assert_eq!(memory.route_end(), Some([41, 64, 0]));
        let gone = find("animal_gone").unwrap();
        let p = place(gone, &memory, &mut rng);
        assert_eq!(p, Some([41, 64, 0]));
        let made = commands(gone, p, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::MoveEntity { entity: 201, to, .. } if (to[0] - 41.5).abs() < 0.01),
            "{:?}",
            made[0].0
        );

        let path = find("light_path").unwrap();
        let made = commands(path, None, &memory, &mut ids(), &now, None);
        assert!(made.len() >= 3);
        for (command, _) in &made {
            match command {
                Command::SetBlock {
                    pos, settle, wall, ..
                } => {
                    assert!(pos[0] > 0 && pos[2] == 0, "toward the route's end: {pos:?}");
                    // The dev client: the path floated over a slope.
                    assert!(*settle >= 3, "the host may find the ground: {settle}");
                    assert_eq!(wall, "minecraft:wall_torch");
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn digging_is_air_into_ground_and_a_worn_path_or_a_puddle_is_not() {
        let mut n = 0;
        let mut ids = || {
            n += 1;
            n
        };
        assert_eq!(
            dug(&set(&mut ids, [1, 63, 1], "minecraft:air", GROUND)),
            Some([1, 63, 1])
        );
        assert_eq!(
            dug(&set(
                &mut ids,
                [1, 63, 1],
                "minecraft:dirt_path",
                &["minecraft:grass_block"]
            )),
            None
        );
        assert_eq!(
            dug(&set(&mut ids, [1, 64, 1], "minecraft:water", GROUND)),
            None
        );
        assert_eq!(
            dug(&set(
                &mut ids,
                [1, 64, 1],
                "minecraft:air",
                &["minecraft:snow"]
            )),
            None,
            "clearing snow is not digging"
        );
    }

    #[test]
    fn a_hole_is_dug_clear_of_what_they_built_or_not_at_all() {
        let mut memory = Memory::default();
        memory.home = Some(crate::memory::Home {
            center: [0, 64, 0],
            floor_y: 64,
            ..Default::default()
        });
        memory.place = Some([10, 64, 0]);
        memory.placed.push([12, 64, 0]); // they built up to its place
        let hole = find("hole").unwrap();
        for seed in 0..20 {
            let spot = place(hole, &memory, &mut Rng::new(seed)).expect("a clear spot");
            assert!(clear_of_built(&memory, spot), "{spot:?}");
            assert_ne!(spot, [10, 64, 0]);
        }

        // Built all around, out to twenty blocks: nowhere to dig.
        for x in (-24..=24).step_by(4) {
            for z in (-24..=24).step_by(4) {
                memory.placed.push([x, 64, z]);
            }
        }
        assert_eq!(place(hole, &memory, &mut Rng::new(3)), None);
    }

    #[test]
    fn what_it_builds_lands_on_the_ground_and_ice_only_on_water() {
        // The dev client: signs in the air, cairns in a village's stairs, ice in
        // the lake bed.
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let now = Now::default();
        let made_by = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        for (command, _) in made_by("cairn", &mut rng) {
            assert!(
                matches!(&command, Command::SetBlock { rest: true, settle, .. } if *settle >= 3),
                "{command:?}"
            );
        }
        let sign = made_by("sign", &mut rng);
        assert!(
            matches!(&sign[0].0, Command::SetBlock { wall, settle, .. } if wall == "minecraft:oak_wall_sign" && *settle >= 3),
            "{:?}",
            sign[0].0
        );
        assert!(
            matches!(&sign[1].0, Command::SetSign { settle, .. } if *settle >= 3),
            "the text looks where the sign landed"
        );
        for (command, _) in made_by("ice_path", &mut rng) {
            assert!(
                matches!(&command, Command::SetBlock { only: true, replace, .. } if replace == &["minecraft:water".to_string()]),
                "{command:?}"
            );
        }
        let sapling = made_by("sapling", &mut rng);
        assert!(
            matches!(sapling[0].0, Command::SetBlock { .. })
                && matches!(sapling[1].0, Command::TakeItem { .. })
                && sapling[1].1 > sapling[0].1,
            "planted before it is taken: {sapling:?}"
        );
    }

    #[test]
    fn what_it_took_comes_back_itself_worn_or_named_and_a_thing_in_the_chest_gets_a_name() {
        let mut memory = rich_memory();
        let now = Now::default();
        let held = |id: &str| crate::memory::Held {
            item: stack(id, 1, 0),
            from: [2, 64, 0],
            slot: 4,
            ..Default::default()
        };

        memory.held = Some(held("minecraft:iron_pickaxe"));
        let used = find("returned_used").unwrap();
        let made = commands(used, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::PutItem { kept, wear, name, .. } if kept == "held" && *wear > 0 && name.is_empty()),
            "the pickaxe itself, worn: {:?}",
            made[0].0
        );
        memory.held = Some(held("minecraft:bread"));
        let made = commands(used, None, &memory, &mut ids(), &now, Some("borrowed"));
        assert!(
            matches!(&made[0].0, Command::PutItem { wear: 0, name, .. } if name == "borrowed"),
            "bread is named, not worn: {:?}",
            made[0].0
        );

        let take = commands(find("take").unwrap(), None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&take[0].0, Command::TakeItem { keep, .. } if keep == "held"),
            "what it takes the host keeps: {:?}",
            take[0].0
        );
        let back = commands(
            find("return_taken").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(matches!(&back[0].0, Command::PutItem { kept, .. } if kept == "held"));

        let rename = find("renamed_item").unwrap();
        let made = commands(
            rename,
            Some([2, 64, 0]),
            &memory,
            &mut ids(),
            &now,
            Some("mine"),
        );
        assert!(
            matches!(&made[0].0, Command::RenameItem { name, slot: 4, .. } if name == "mine"),
            "the favourite, the pickaxe in slot 4: {:?}",
            made[0].0
        );
    }

    #[test]
    fn the_book_is_signed_with_one_word_and_the_map_marked_at_its_place() {
        let mut memory = rich_memory();
        memory.place = Some([40, 64, -12]);
        let now = Now::default();
        let book = commands(
            find("book_word").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            Some("under"),
        );
        assert!(
            matches!(&book[0].0, Command::SignBook { slot: 12, title, pages, .. } if title == "under" && pages == &["under".to_string()]),
            "{:?}",
            book[0].0
        );
        let map = commands(
            find("map_marked").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&map[0].0, Command::MarkMap { slot: 13, at, .. } if *at == [40, 64, -12]),
            "{:?}",
            map[0].0
        );
    }

    #[test]
    fn the_notes_name_only_lecterns_out_of_their_view() {
        let mut memory = rich_memory();
        memory.fixtures = [[0, 64, 10], [0, 64, -10], [0, 64, -20]]
            .iter()
            .map(|p| crate::memory::Fixture {
                kind: "lectern".to_string(),
                pos: *p,
            })
            .collect();
        memory.last.online = true;
        memory.last.sleeping = false;
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 180.0;
        let made = commands(
            find("lectern_notes").unwrap(),
            Some([0, 64, 10]),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let also = made.iter().find_map(|(c, _)| match c {
            Command::PutBook { also, .. } => Some(also.clone()),
            _ => None,
        });
        assert_eq!(
            also,
            Some(Vec::<BlockPos>::new()),
            "both others are in front of them (yaw 180 faces north)"
        );
        memory.last.yaw = 0.0;
        let made = commands(
            find("lectern_notes").unwrap(),
            Some([0, 64, 10]),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(made.iter().any(|(c, _)| matches!(c, Command::PutBook { also, .. } if also == &vec![[0, 64, -10], [0, 64, -20]])));
    }

    #[test]
    fn a_frame_in_front_of_them_is_never_turned() {
        let mut memory = rich_memory();
        memory.decor = vec![crate::memory::Decor {
            kind: "item_frame".to_string(),
            id: 5,
            pos: [0, 64, 10],
        }];
        memory.last.online = true;
        memory.last.sleeping = false;
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 0.0;
        let now = Now::default();
        let turn = |memory: &Memory| {
            commands(
                find("frames_turn").unwrap(),
                None,
                memory,
                &mut ids(),
                &now,
                None,
            )
        };
        assert!(turn(&memory).is_empty(), "facing it, ten blocks off");
        memory.last.yaw = 180.0;
        assert_eq!(turn(&memory).len(), 1, "their back to it");
    }

    #[test]
    fn the_hopper_empties_into_its_chest_and_the_box_moves_with_what_it_holds() {
        let mut memory = rich_memory();
        let now = Now::default();
        let mut rng = Rng::new(2);
        let hopper = find("hopper_emptied").unwrap();
        let made = commands(
            hopper,
            place(hopper, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert_eq!(
            made.len(),
            1,
            "never looked in: one move, the host's first full slot"
        );
        let stack = |slot| craftkit::world::ItemStack {
            id: "minecraft:iron_ingot".to_string(),
            count: 3,
            slot,
        };
        memory.containers.push(crate::memory::Container {
            pos: [2, 65, 0],
            kind: "minecraft:hopper".to_string(),
            contents: vec![stack(1), stack(3)],
            ..Default::default()
        });
        let made = commands(
            hopper,
            place(hopper, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            made.len() == 2
                && made
                    .iter()
                    .all(|(c, _)| matches!(c, Command::MoveItem { from_slot, .. } if *from_slot == 1 || *from_slot == 3)),
            "the slots that hold something, an empty one never asked for: {made:?}"
        );
        assert!(
            made.iter().all(|(c, _)| matches!(c, Command::MoveItem { from, to, .. } if *from == [2, 65, 0] && *to == [2, 64, 0])),
            "from the hopper into the chest under it: {made:?}"
        );
        let shulker = find("shulker_moved").unwrap();
        let made = commands(
            shulker,
            place(shulker, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&made[0].0, Command::MoveBlock { from, around: 2, to, .. } if *from == [-4, 64, -4] && to != from),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn a_chest_full_of_one_holds_only_as_much_dirt_as_it_dug() {
        let mut memory = rich_memory();
        memory.stores.clear();
        memory.store("minecraft:dirt", 3);
        let fill = find("chest_full_of_one").unwrap();
        let made = commands(
            fill,
            Some([2, 64, 0]),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 3, "three dug, three slots");
        assert!(made.iter().all(|(c, _)| matches!(c, Command::PutItem { item, kept, .. } if item.id == "minecraft:dirt" && item.count == 1 && kept.is_empty())));
    }

    #[test]
    fn the_door_is_rehung_a_rung_goes_on_top_the_flower_is_potted_and_the_pumpkin_faces_the_door() {
        let memory = rich_memory();
        let now = Now::default();
        let mut rng = Rng::new(4);
        let made_by = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            commands(
                event,
                place(event, &memory, rng),
                &memory,
                &mut ids(),
                &now,
                None,
            )
        };
        assert!(
            matches!(&made_by("door_hinge", &mut rng)[0].0, Command::SetState { property, value, .. } if property == "hinge" && value == "@next")
        );
        assert!(
            matches!(&made_by("ladder_gap", &mut rng)[0].0, Command::MoveBlock { from, to, .. } if *from == [-6, 66, 0] && *to == [-6, 68, 0]),
            "a middle rung onto the top"
        );
        let potted = made_by("flowers_potted", &mut rng);
        assert!(
            matches!(&potted[0].0, Command::SetBlock { block, pos, only: true, .. } if block == "minecraft:potted_poppy" && *pos == [2, 65, 2]),
            "the pot first"
        );
        assert!(
            matches!(&potted[1].0, Command::SetBlock { block, pos, .. } if block == "minecraft:air" && *pos == [6, 64, 6])
        );
        let carved = made_by("pumpkin_face", &mut rng);
        assert!(
            matches!(&carved[0].0, Command::SetBlock { block, state, only: true, .. } if block == "minecraft:carved_pumpkin" && state.starts_with("facing=")),
            "{:?}",
            carved[0].0
        );
        assert!(matches!(
            &carved[1].0,
            Command::DamageItem { amount: 1, .. }
        ));
    }

    #[test]
    fn a_sapling_grows_before_the_bone_meal_is_taken_and_a_hole_is_filled_with_what_it_dug() {
        let memory = rich_memory();
        let now = Now::default();
        let mut rng = Rng::new(1);
        let tree = find("tree_grown").unwrap();
        let made = commands(
            tree,
            place(tree, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(matches!(&made[0].0, Command::Grow { pos, .. } if *pos == [5, 64, -5]));
        assert!(
            matches!(
                &made[1].0,
                Command::TakeItem {
                    count: 3,
                    slot: 15,
                    ..
                }
            ) && made[1].1 > made[0].1,
            "the meal after, three of nine"
        );
        let fill = find("backfill").unwrap();
        let made = commands(
            fill,
            place(fill, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&made[0].0, Command::SetBlock { pos, block, .. } if *pos == [7, 63, 0] && block == "minecraft:dirt"),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn an_alcove_is_stone_only_and_the_room_goes_dark_a_torch_at_a_time_into_the_chest() {
        let mut memory = rich_memory();
        memory.held = Some(crate::memory::Held {
            item: stack("minecraft:iron_pickaxe", 1, 0),
            from: [2, 64, 0],
            slot: 4,
            ..Default::default()
        });
        let now = Now::default();
        let alcove = find("alcove").unwrap();
        let wall = memory.walls[0].0;
        let made = commands(alcove, Some(wall), &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, only: true, replace, pos, .. }
                if block == "minecraft:air" && replace.iter().all(|r| STONE.contains(&r.as_str())) && *pos != wall),
            "behind the wall, stone only: {:?}",
            made[0].0
        );
        assert!(
            matches!(&made[1].0, Command::SetBlock { block, .. } if block == "minecraft:torch")
        );
        assert!(matches!(&made[2].0, Command::DropItem { kept, .. } if kept == "held"));

        let mut memory = rich_memory();
        memory.home.as_mut().unwrap().lights = vec![[1, 65, 1], [2, 65, 2], [30, 65, 30]];
        let dark = find("one_room_dark").unwrap();
        let made = commands(dark, Some([1, 65, 1]), &memory, &mut ids(), &now, None);
        assert_eq!(
            made.len(),
            4,
            "the two near, each taken then put: not the far one"
        );
        for pair in made.chunks(2) {
            assert!(
                matches!(&pair[0].0, Command::SetBlock { block, .. } if block == "minecraft:air")
            );
            assert!(
                matches!(&pair[1].0, Command::PutItem { item, .. } if item.id == "minecraft:torch" && item.count == 1)
            );
            assert_eq!(
                pair[1].1,
                pair[0].1 + 1,
                "put at once after its torch came down"
            );
        }

        let turn = commands(
            find("bed_turned").unwrap(),
            Some([0, 64, 0]),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(matches!(&turn[0].0, Command::SetState { value, .. } if value == "@turn"));
        let held = commands(
            find("door_held").unwrap(),
            Some([3, 64, 0]),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(matches!(&held[0], (Command::ToggleBlock { pos, .. }, 3) if *pos == [3, 64, 0]));
    }

    #[test]
    fn its_diary_is_written_signed_with_their_name_and_sold_or_brought_to_them() {
        let mut memory = rich_memory();
        memory.target_name = "razor".to_string();
        memory.last.creatures_near.push(crate::memory::Creature {
            id: 77,
            kind: "villager".to_string(),
            pos: [4.0, 64.0, 4.0],
            ..Default::default()
        });
        let now = Now::default();

        let sold = commands(
            find("diary_sold").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            Some("i watched you"),
        );
        assert_eq!(
            texts(&sold[0].0),
            vec!["i watched you".to_string()],
            "its own line is kept as written, not the facts after it"
        );
        let kept = match &sold[0].0 {
            Command::WriteBook {
                kept,
                title,
                author,
                pages,
                ..
            } => {
                assert_eq!((title.as_str(), author.as_str()), ("razor", "razor"));
                assert!(pages[0].starts_with("i watched you"), "{pages:?}");
                kept.clone()
            }
            other => panic!("{other:?}"),
        };
        assert!(
            matches!(&sold[1].0, Command::OfferTrade { entity: 77, kept: k, .. } if *k == kept)
        );

        let brought = commands(
            find("diary_brought").unwrap(),
            None,
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&brought[1].0, Command::Approach { then, entities, kept: k, .. }
            if then == "give" && entities == &vec![77] && !k.is_empty())
        );

        memory.diary_offered = 4.0;
        memory.chain_landed("diary_sold", 4.0);
        assert!(memory.chains.iter().any(|o| o.id == "the_diary"));
        let target = memory.target.unwrap_or(0);
        memory.observe(
            &craftkit::world::Observations {
                players: vec![craftkit::world::Player {
                    index: target,
                    uuid: memory.target_uuid.clone(),
                    ..Default::default()
                }],
                events: vec![craftkit::world::Event {
                    player: target + 1,
                    kind: craftkit::world::EventKind::Bought,
                    detail: "minecraft:written_book".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            5.0,
        );
        assert!(
            memory.diary_bought,
            "bought, by someone else too: never brought"
        );
        assert!(
            memory.chains.iter().all(|o| o.id != "the_diary"),
            "and its chain is over"
        );
    }

    #[test]
    fn the_other_house_copies_the_house_around_its_bed_and_door_for_a_minute() {
        let memory = rich_memory();
        let now = Now::default();
        let bed = memory.home.as_ref().unwrap().bed;
        let made = commands(
            find("other_house").unwrap(),
            bed,
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert_eq!(made.len(), 1, "{made:?}");
        match &made[0].0 {
            Command::OtherHouse {
                center,
                bed: at,
                door,
                seconds,
                comes,
                ..
            } => {
                assert_eq!(*center, memory.home.as_ref().unwrap().center);
                assert_eq!(Some(*at), bed);
                assert_eq!(Some(*door), memory.usual_door());
                assert_eq!(*seconds, 60.0);
                assert!(
                    !comes,
                    "the figure only stands in the doorway the first time"
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(
            crate::config::Preset::Calm
                .config()
                .events
                .disabled
                .contains(&"other_house".to_string()),
            "calm keeps it out, a sleep event"
        );
    }

    #[test]
    fn waking_elsewhere_moves_the_player_and_the_animals_stand_still_to_stare() {
        let memory = rich_memory();
        let now = Now::default();
        let woke = find("woke_elsewhere").unwrap();
        let bed = memory.home.as_ref().unwrap().bed;
        let made = commands(woke, bed, &memory, &mut ids(), &now, None);
        assert!(
            made.iter().any(|(c, _)| matches!(c, Command::MovePlayer { player, .. } if *player == memory.target.unwrap_or(0))),
            "{made:?}"
        );

        let stare = find("stare").unwrap();
        let made = commands(stare, None, &memory, &mut ids(), &now, None);
        assert!(!made.is_empty());
        for (command, _) in &made {
            assert!(
                matches!(command, Command::Hold { seconds, .. } if (10.0..=30.0).contains(seconds)),
                "{command:?}"
            );
        }
    }

    #[test]
    fn the_monsters_near_are_held_staring_until_dawn_and_no_longer_than_ten_minutes() {
        let memory = rich_memory();
        let stare = find("hostiles_stare").unwrap();
        let at = |day_time| Now {
            day_time,
            ..Now::default()
        };
        let made = commands(stare, None, &memory, &mut ids(), &at(18_000), None);
        assert_eq!(made.len(), 3, "one hold for each of the three");
        for (command, later) in &made {
            match command {
                Command::Hold {
                    seconds, player, ..
                } => {
                    assert_eq!(*seconds, 250.0, "5000 ticks to dawn");
                    assert_eq!(*player, memory.target.unwrap_or(0));
                    assert_eq!(*later, 0);
                }
                other => panic!("{other:?}"),
            }
        }
        let early = commands(stare, None, &memory, &mut ids(), &at(13_000), None);
        assert!(matches!(early[0].0, Command::Hold { seconds, .. } if seconds == 500.0));
        let late = commands(stare, None, &memory, &mut ids(), &at(22_900), None);
        assert!(
            matches!(late[0].0, Command::Hold { seconds, .. } if seconds == 60.0),
            "a minute at least"
        );
        assert!(
            !crate::rules::hidden_from_view(stare.action),
            "it is about being seen"
        );
    }

    #[test]
    fn asleep_a_figure_lies_down_beside_their_bed() {
        let memory = rich_memory();
        let lies = find("lies_beside").unwrap();
        assert_eq!(lies.effect(), "night");
        let mut no_figures = crate::config::Config::default();
        no_figures.events.figure = false;
        assert!(!crate::brain::offer::switched_on(lies, &no_figures));
        let door = memory.usual_door().unwrap();
        let made = commands(
            lies,
            place(lies, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[1].0, Command::Choke { how, item, damage, door: Some(d), .. } if how == "lie" && item.is_empty() && *damage == 0.0 && *d == door),
            "{:?}",
            made[1]
        );
    }

    #[test]
    fn their_pet_walks_off_to_a_figure_ahead_of_them() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 0.0;
        memory.last.creatures_near.push(crate::memory::Creature {
            id: 77,
            kind: "wolf".to_string(),
            pos: [2.0, 64.0, 0.0],
            pet: true,
            ..Default::default()
        });
        let follows = find("pet_follows").unwrap();
        let p = place(follows, &memory, &mut Rng::new(1));
        assert_eq!(p, Some([0, -10_000, PET_TO as i32]));
        let made = commands(follows, p, &memory, &mut ids(), &Now::default(), None);
        assert!(matches!(&made[0].0, Command::Figure { .. }));
        assert!(
            matches!(&made[1].0, Command::Approach { entities, to: Some(to), .. }
                if entities == &vec![memory.pets().next().unwrap().id] && to[2] == PET_TO + 0.5 && to[1] < -1000.0),
            "{:?}",
            made[1]
        );
        // Never far enough for the game to bring the pet back at once.
        const { assert!(PET_TO < 12.0) };
    }

    #[test]
    fn looking_up_at_night_they_see_a_figure_fall_far_off() {
        let mut memory = rich_memory();
        memory.last.online = true;
        memory.last.indoors = false;
        memory.last.underground = false;
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 0.0;
        memory.last.pitch = -50.0;
        assert!(crate::brain::offer::meets(
            crate::events::Need::LookingUp,
            &memory,
            &Now::default()
        ));
        let fall = find("falls_from_the_sky").unwrap();
        let p = place(fall, &memory, &mut Rng::new(1));
        assert_eq!(
            p,
            Some([0, -10_000, FALLS_AHEAD as i32]),
            "ahead of them, on the ground"
        );
        let made = commands(fall, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { falls: true, .. }, 0)]),
            "{made:?}"
        );
        memory.last.pitch = 10.0;
        assert!(
            !crate::brain::offer::meets(crate::events::Need::LookingUp, &memory, &Now::default()),
            "looking down"
        );
    }

    #[test]
    fn a_face_is_laid_out_in_their_chest_with_nothing_lost() {
        let stack = |slot, count| ItemStack {
            id: "minecraft:cobblestone".to_string(),
            count,
            slot,
        };
        // A stack in an eye's slot, the biggest stack in the mouth.
        let contents = vec![stack(2, 5), stack(21, 40), stack(0, 3)];
        let moves = face_moves(&contents).unwrap();
        // Replay the moves: every face slot holds one, nothing else is where the face goes.
        let mut grid = [0u32; 27];
        for s in &contents {
            grid[s.slot as usize] = s.count;
        }
        let before: u32 = grid.iter().sum();
        for (from, to, count) in &moves {
            assert!(
                grid[*from as usize] >= *count,
                "a move from {from} of {count}: {grid:?}"
            );
            grid[*from as usize] -= count;
            grid[*to as usize] += count;
        }
        assert_eq!(grid.iter().sum::<u32>(), before, "nothing lost or added");
        for face in FACE {
            assert_eq!(grid[face as usize], 1, "face slot {face}: {grid:?}");
        }
        // No stack of seven: no face.
        assert!(face_moves(&[stack(0, 6), stack(1, 3)]).is_none());

        let mut memory = rich_memory();
        memory.containers[0].contents = contents.clone();
        let event = find("a_face_in_the_chest").unwrap();
        let p = place(event, &memory, &mut Rng::new(1));
        assert_eq!(p, Some(memory.containers[0].pos));
        let made = commands(event, p, &memory, &mut ids(), &Now::default(), None);
        assert_eq!(made.len(), moves.len());
    }

    #[test]
    fn coming_home_there_are_handprints_on_a_window() {
        let memory = rich_memory();
        let event = find("handprints").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let pane = windows(&memory, memory.home.as_ref().unwrap().center)[0];
        assert!(
            matches!(&made[..], [(Command::Hint { hint, pos: Some(p), .. }, _)] if hint == "handprints" && *p == pane),
            "{made:?}"
        );
    }

    #[test]
    fn a_chicken_lays_what_it_took_instead_of_an_egg() {
        let mut memory = rich_memory();
        memory.held = Some(crate::memory::Held {
            item: ItemStack {
                id: "minecraft:compass".to_string(),
                count: 1,
                slot: 0,
            },
            ..Default::default()
        });
        let chicken = memory
            .last
            .creatures_near
            .iter()
            .find(|c| c.kind == "chicken")
            .expect("a chicken near them")
            .pos;
        let event = find("chicken_lays").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[0].0, Command::PlaySound { sound, .. } if sound.ends_with("chicken.egg"))
        );
        assert!(
            matches!(&made[1].0, Command::DropItem { kept, item, pos, .. } if kept == "held" && item.id == "minecraft:compass" && pos[0] == chicken[0] && pos[2] == chicken[2]),
            "{:?}",
            made[1]
        );
    }

    #[test]
    fn at_night_a_firework_bursts_as_a_face_over_its_house() {
        let mut memory = rich_memory();
        memory.builds.push(crate::building::Build {
            kind: "house_near".to_string(),
            site: [40, 63, 0],
            way: [0, 1],
            laid: crate::building::house([0, 1], false).len(),
            ..Default::default()
        });
        let event = find("firework_over_its_house").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::Firework { shape, pos, flight, .. }, 0)] if shape == "creeper" && pos[1] > 64.0 && *flight >= 1),
            "{made:?}"
        );
    }

    #[test]
    fn back_through_their_portal_a_figure_waits_beside_it() {
        let mut memory = rich_memory();
        memory.milestone("crossed", 2.0, Some([40, 64, 10]), "minecraft:overworld");
        let event = find("waits_at_the_portal").unwrap();
        let p = place(event, &memory, &mut Rng::new(1));
        assert_eq!(p, Some([40, 64, 10]));
        let made = commands(event, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { faces: Some(f), pos, .. }, 0)] if f[0] == 40.5 && pos[0] == 43.5),
            "beside it, facing it: {made:?}"
        );
    }

    #[test]
    fn away_with_a_map_someone_is_marked_at_their_house() {
        let mut memory = rich_memory();
        let center = memory.home.as_ref().unwrap().center;
        let event = find("figure_on_map").unwrap();
        memory.last.pos = [center[0] as f64 + 0.5, 64.0, center[2] as f64 + 0.5];
        assert_eq!(
            place(event, &memory, &mut Rng::new(1)),
            None,
            "at home: nobody marked there"
        );
        memory.last.pos = [200.5, 64.0, 0.5];
        let p = place(event, &memory, &mut Rng::new(1));
        assert_eq!(p, Some(center));
        let made = commands(event, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::MapMark { at, .. }, 0)] if at[0] == center[0] as f64 + 0.5),
            "{made:?}"
        );
    }

    #[test]
    fn every_animal_and_mob_around_them_stops_at_once() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.creatures_near = vec![
            crate::memory::Creature {
                id: 1,
                kind: "cow".to_string(),
                pos: [5.0, 64.0, 0.0],
                ..Default::default()
            },
            crate::memory::Creature {
                id: 2,
                kind: "sheep".to_string(),
                pos: [60.0, 64.0, 0.0],
                ..Default::default()
            },
        ];
        memory.last.hostiles_near = vec![crate::memory::Creature {
            id: 3,
            kind: "zombie".to_string(),
            pos: [0.0, 64.0, 10.0],
            ..Default::default()
        }];
        let event = find("all_freeze").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let held: Vec<u32> = made
            .iter()
            .filter_map(|(c, at)| match c {
                Command::Hold {
                    entity, seconds, ..
                } if *at == 0 && *seconds <= 5.0 => Some(*entity),
                _ => None,
            })
            .collect();
        assert_eq!(held, vec![1, 3], "all at once, the far sheep left alone");
    }

    #[test]
    fn under_water_their_air_runs_out_twice_as_fast() {
        let mut memory = rich_memory();
        memory.last.online = true;
        memory.last.air = 300;
        assert!(
            !crate::brain::offer::meets(
                crate::events::Need::TargetUnderwater,
                &memory,
                &Now::default()
            ),
            "full air"
        );
        memory.last.air = 200;
        assert!(crate::brain::offer::meets(
            crate::events::Need::TargetUnderwater,
            &memory,
            &Now::default()
        ));
        let event = find("held_breath").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::DrainAir { seconds, .. }, 0)] if *seconds <= 60.0),
            "{made:?}"
        );
    }

    #[test]
    fn one_of_their_animals_follows_them_all_day() {
        let mut memory = rich_memory();
        memory.last.online = true;
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.creatures_near.push(crate::memory::Creature {
            id: 31,
            kind: "cow".to_string(),
            pos: [4.0, 64.0, 0.0],
            ..Default::default()
        });
        let event = find("animal_follows").unwrap();
        let animal = memory.animals_near_target()[0].id;
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(made.len() >= 14, "a day of walks: {}", made.len());
        assert!(made.iter().all(|(c, _)| matches!(c, Command::Approach { entities, stop, seconds, .. } if entities == &vec![animal] && *stop >= 4.0 && *seconds <= 90.0)));
        let last = made.last().unwrap().1;
        assert!(last + 90 * 20 >= 24_000, "it lasts the day: {last}");
    }

    #[test]
    fn at_night_a_zombie_wears_their_face_and_walks_at_them() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.hostiles_near = vec![
            crate::memory::Creature {
                id: 5,
                kind: "skeleton".to_string(),
                pos: [3.0, 64.0, 0.0],
                ..Default::default()
            },
            crate::memory::Creature {
                id: 9,
                kind: "zombie".to_string(),
                pos: [12.0, 64.0, 0.0],
                ..Default::default()
            },
        ];
        let event = find("wears_your_face").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[0].0, Command::WearFace { entity: 9, .. }),
            "the zombie, not the skeleton: {made:?}"
        );
        assert!(matches!(&made[1].0, Command::Approach { entities, .. } if entities == &vec![9]));
    }

    #[test]
    fn far_off_a_head_shows_above_the_ground() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 0.0;
        let event = find("peeks_from_ground").unwrap();
        let p = place(event, &memory, &mut Rng::new(1)).unwrap();
        assert_eq!(p[1], -10_000, "on the ground");
        assert!((distance([p[0], 0, p[2]], [0, 0, 0]) - PEEK_FROM).abs() <= 1.5);
        let made = commands(event, Some(p), &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { sunk: true, .. }, 0)]),
            "{made:?}"
        );
    }

    #[test]
    fn along_their_route_figures_stand_in_single_file_facing_the_house() {
        let mut memory = rich_memory();
        memory.routes = vec![crate::memory::Route {
            points: (0..40).map(|i| [i * 2, 64, 0]).collect(),
            walked: 3,
            first_day: 0.0,
        }];
        memory.last.pos = [0.5, 64.0, 0.5];
        let event = find("in_a_line").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), IN_A_LINE_MOST, "{made:?}");
        let center = memory.home.as_ref().unwrap().center;
        let mut xs = Vec::new();
        for (command, _) in &made {
            let Command::Figure {
                faces: Some(faces),
                vanish_within,
                pos,
                ..
            } = command
            else {
                panic!("{command:?}");
            };
            assert_eq!(faces[0], center[0] as f64 + 0.5, "facing the house");
            assert!(*vanish_within > 0.0);
            xs.push(pos[0]);
        }
        assert!(
            xs.windows(2).all(|w| w[1] - w[0] >= IN_A_LINE_APART),
            "single file on from them: {xs:?}"
        );
    }

    #[test]
    fn far_ahead_a_figure_walks_across_their_way_left_to_right() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        // Facing south (+z): their left is east (+x), their right west (-x).
        memory.last.yaw = 0.0;
        let event = find("crossing_ahead").unwrap();
        let p = place(event, &memory, &mut Rng::new(1));
        assert_eq!(p, Some([0, -10_000, CROSSING_AHEAD as i32]));
        let made = commands(event, p, &memory, &mut ids(), &Now::default(), None);
        let [(
            Command::Figure {
                walks: true,
                path,
                pos,
                ..
            },
            0,
        )] = &made[..]
        else {
            panic!("{made:?}");
        };
        assert_eq!(path.first(), Some(pos));
        let (from, to) = (path[0], path[1]);
        assert!(
            from[0] > to[0],
            "from their left (east) to their right (west): {from:?} {to:?}"
        );
        assert!((from[2] - to[2]).abs() < 1e-9, "straight across their way");
        assert!(from[1] < -1000.0 && to[1] < -1000.0, "on the ground");
    }

    #[test]
    fn holding_an_eye_of_ender_the_next_throw_flies_home() {
        let memory = rich_memory();
        let event = find("eye_turns_home").unwrap();
        assert!(event
            .needs
            .contains(&crate::events::Need::Holding("minecraft:ender_eye")));
        let center = memory.home.as_ref().unwrap().center;
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::EyeHome { to, seconds, .. }, 0)] if to[0] == center[0] as f64 + 0.5 && *seconds > 0.0),
            "{made:?}"
        );
    }

    #[test]
    fn a_note_is_left_in_their_ender_chest() {
        let mut memory = rich_memory();
        memory.fixtures.push(crate::memory::Fixture {
            kind: "ender_chest".to_string(),
            pos: [3, 64, -3],
        });
        let event = find("ender_chest_note").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let (Command::WriteBook { kept, pages, .. }, Command::PutEnderChest { kept: put, .. }) =
            (&made[0].0, &made[1].0)
        else {
            panic!("{made:?}");
        };
        assert_eq!(kept, put, "the very book written");
        assert!(!pages[0].is_empty());
        assert_eq!(
            crate::memory::fixture_kind_of("minecraft:ender_chest"),
            Some("ender_chest")
        );
    }

    #[test]
    fn a_bed_is_laid_in_its_house_and_made_their_respawn_point() {
        let mut memory = rich_memory();
        let way = [1, 0];
        memory.builds.push(crate::building::Build {
            kind: "house_near".to_string(),
            site: [40, 63, 0],
            way,
            laid: crate::building::house(way, false).len(),
            ..Default::default()
        });
        let event = find("respawn_its_bed").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let (_, head) = crate::building::house_bed([40, 63, 0], way);
        let Command::PlaceBlueprint { parts, .. } = &made[0].0 else {
            panic!("{made:?}");
        };
        assert_eq!(parts.len(), 2, "both halves");
        assert!(parts.iter().all(|p| p.block.ends_with("_bed")));
        assert!(parts.iter().any(
            |p| p.state.contains("part=head") && [40 + p.at[0], 63 + p.at[1], p.at[2]] == head
        ));
        assert!(matches!(&made[1], (Command::SetRespawn { pos, .. }, 2) if *pos == head));
        // Its lights never stand where the bed lies.
        let (_, lights) = crate::building::house_lights([40, 63, 0], way);
        let bed: Vec<BlockPos> = parts
            .iter()
            .map(|p| [40 + p.at[0], 63 + p.at[1], p.at[2]])
            .collect();
        assert!(
            lights.iter().all(|l| !bed.contains(l)),
            "{lights:?} {bed:?}"
        );
    }

    #[test]
    fn at_night_its_house_goes_dark_then_lights_one_by_one_then_out() {
        let mut memory = rich_memory();
        let way = [0, 1];
        memory.builds.push(crate::building::Build {
            kind: "house_near".to_string(),
            site: [40, 63, 0],
            way,
            laid: crate::building::house(way, false).len(),
            ..Default::default()
        });
        memory.last.online = true;
        memory.last.pos = [20.5, 64.0, 0.5];
        assert!(crate::brain::offer::meets(
            crate::events::Need::ItsHouseNear,
            &memory,
            &Now::default()
        ));
        let lights = find("its_lights_on").unwrap();
        let made = commands(
            lights,
            place(lights, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let (torch, floor) = crate::building::house_lights([40, 63, 0], way);
        assert_eq!(
            torch,
            [41, 65, -1],
            "its wall torch, as the blueprint hangs it"
        );
        assert!(
            matches!(&made[0], (Command::HideBlock { pos, .. }, 0) if *pos == torch),
            "dark first"
        );
        let on: Vec<u64> = made
            .iter()
            .filter(|(c, _)| matches!(c, Command::SetBlock { block, .. } if block == "minecraft:lantern"))
            .map(|(_, at)| *at)
            .collect();
        assert_eq!(on.len(), floor.len());
        assert!(on.windows(2).all(|w| w[0] < w[1]), "one by one: {on:?}");
        let off = made
            .iter()
            .filter(|(c, at)| matches!(c, Command::SetBlock { block, only: true, replace, .. } if block == "minecraft:air" && replace == &["minecraft:lantern".to_string()]) && *at > on[2])
            .count();
        assert_eq!(off, floor.len(), "all out again, its lanterns only");

        // Half built, or they are inside it: not now.
        memory.builds[0].laid = 3;
        assert!(!crate::brain::offer::meets(
            crate::events::Need::ItsHouseNear,
            &memory,
            &Now::default()
        ));
    }

    #[test]
    fn swimming_they_are_pulled_under_a_moment() {
        let mut memory = rich_memory();
        memory.last.online = true;
        memory.last.swimming = true;
        assert!(crate::brain::offer::meets(
            crate::events::Need::TargetSwimming,
            &memory,
            &Now::default()
        ));
        let under = find("pulled_under").unwrap();
        let made = commands(
            under,
            place(under, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let pushes: Vec<f64> = made
            .iter()
            .filter_map(|(c, _)| match c {
                Command::PushPlayer { velocity, .. } => Some(velocity[1]),
                _ => None,
            })
            .collect();
        assert_eq!(pushes.len(), 2, "pulled twice");
        assert!(
            pushes.iter().all(|y| *y < 0.0 && *y >= -1.0),
            "down, never more than a block a tick"
        );
        assert!(made.iter().any(
            |(c, _)| matches!(c, Command::PlaySound { sound, .. } if sound.ends_with("hurt_drown"))
        ));
    }

    #[test]
    fn down_their_shaft_a_figure_looks_down_from_its_top() {
        let mut memory = rich_memory();
        memory.fixtures.retain(|f| f.kind != "ladder");
        for y in 50..=64 {
            memory.fixtures.push(crate::memory::Fixture {
                kind: "ladder".to_string(),
                pos: [-6, y, 0],
            });
        }
        memory.last.online = true;
        memory.last.pos = [-5.5, 52.0, 0.5];
        let top = find("top_of_the_ladder").unwrap();
        assert!(crate::brain::offer::meets(
            crate::events::Need::DownTheShaft,
            &memory,
            &Now::default()
        ));
        let p = place(top, &memory, &mut Rng::new(1));
        assert_eq!(p, Some([-6, 64, 0]), "its top ladder");
        let made = commands(top, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { pos, vanish_within, .. }, 0)] if pos[1] == 65.0 && *vanish_within > 0.0),
            "{made:?}"
        );
        // Near the top: not down the shaft.
        memory.last.pos = [-5.5, 61.0, 0.5];
        assert!(!crate::brain::offer::meets(
            crate::events::Need::DownTheShaft,
            &memory,
            &Now::default()
        ));
    }

    #[test]
    fn walking_they_are_set_back_and_a_figure_stands_where_they_were() {
        let mut memory = rich_memory();
        memory.trail = vec![[0, 64, 0], [2, 64, 0], [4, 64, 0], [6, 64, 0]];
        memory.last.pos = [6.5, 64.0, 0.5];
        let band = find("rubberband").unwrap();
        let back = place(band, &memory, &mut Rng::new(1));
        assert_eq!(
            back,
            Some([2, 64, 0]),
            "the trail three blocks back or more"
        );
        let made = commands(band, back, &memory, &mut ids(), &Now::default(), None);
        assert!(matches!(&made[0], (Command::MovePlayer { to, .. }, 0) if to[0] == 2.5));

        // A trail too short: behind them, the way they face (south) the other way.
        memory.trail.clear();
        memory.last.yaw = 0.0;
        assert_eq!(place(band, &memory, &mut Rng::new(1)), Some([6, 64, -3]));
        assert!(
            matches!(&made[1], (Command::Figure { pos, once_seen: true, .. }, 1) if pos[0] == 6.5 && pos[2] == 0.5),
            "where they had been: {:?}",
            made[1]
        );
    }

    #[test]
    fn where_they_last_died_a_figure_lies_as_a_body_would() {
        let mut memory = rich_memory();
        memory.milestone("died", 2.0, Some([30, 64, 0]), "");
        memory.last.online = true;
        memory.last.pos = [0.5, 64.0, 0.5];
        let died = find("where_you_died").unwrap();
        assert!(crate::brain::offer::meets(
            crate::events::Need::NearTheirDeath,
            &memory,
            &Now::default()
        ));
        let p = place(died, &memory, &mut Rng::new(1));
        assert_eq!(p, Some([30, 64, 0]));
        let made = commands(died, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { lies: true, vanish_within, pos, .. }, 0)] if *vanish_within >= 8.0 && pos[1] == 64.0),
            "{made:?}"
        );
        // Standing on the spot, or far off: not now.
        memory.last.pos = [30.5, 64.0, 0.5];
        assert!(!crate::brain::offer::meets(
            crate::events::Need::NearTheirDeath,
            &memory,
            &Now::default()
        ));
    }

    #[test]
    fn the_light_they_place_underground_shows_a_figure_for_a_blink() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 20.0, 0.5];
        memory.habits.last_light = Some(([4, 21, 0], "minecraft:wall_torch".to_string()));
        let flash = find("torch_flash").unwrap();
        let spot = place(flash, &memory, &mut Rng::new(1)).unwrap();
        assert_eq!(
            spot,
            [6, 19, 0],
            "two blocks past it, on the floor under the wall torch"
        );
        let made = commands(
            flash,
            Some(spot),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::Figure { seconds, pos, .. }, 0)] if *seconds <= 0.25 && pos[1] == 20.0),
            "{made:?}"
        );
        // The trigger is the light placed underground, never the menu.
        assert!(!crate::brain::offer::meets(
            crate::events::Need::LitUnderground,
            &memory,
            &Now::default()
        ));
    }

    #[test]
    fn asleep_they_wake_outside_their_door_with_it_open_behind_them() {
        let memory = rich_memory();
        let walk = find("sleepwalk").unwrap();
        assert!(walk.needs.contains(&crate::events::Need::TargetInBed));
        let door = memory.usual_door().unwrap();
        let center = memory.home.as_ref().unwrap().center;
        let outside = place(walk, &memory, &mut Rng::new(1)).unwrap();
        assert!(
            distance(outside, center) > distance(door, center),
            "out of the door"
        );
        let made = commands(
            walk,
            Some(outside),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(made[0].0, Command::CancelSleep { .. }),
            "woken first"
        );
        assert!(
            matches!(&made[1].0, Command::MovePlayer { to, .. } if to[0] == outside[0] as f64 + 0.5)
        );
        assert!(matches!(&made[2].0, Command::Prints { to, .. } if *to == outside));
        assert!(
            matches!(&made.last().unwrap().0, Command::ToggleBlock { pos, to: Some(true), .. } if *pos == door),
            "the door open behind them, last"
        );
    }

    #[test]
    fn the_door_they_walk_to_will_not_open_and_is_knocked_on() {
        let mut memory = rich_memory();
        let door = memory.home.as_ref().unwrap().doors[0];
        // Walking toward it, four blocks off, facing it.
        memory.last.online = true;
        memory.last.walking = true;
        memory.last.pos = [door[0] as f64 - 3.5, door[1] as f64, door[2] as f64 + 0.5];
        memory.last.yaw = -90.0;
        let held = find("door_wont_open").unwrap();
        let p = place(held, &memory, &mut Rng::new(1));
        assert_eq!(p, Some(door), "the door they walk to");
        let made = commands(held, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[0], (Command::HoldDoor { pos, seconds, .. }, 0) if *pos == door && *seconds >= 4.0),
            "{made:?}"
        );
        assert_eq!(
            made.iter()
                .filter(|(c, at)| matches!(c, Command::PlaySound { .. }) && *at > 0)
                .count(),
            3,
            "a knock"
        );
    }

    #[test]
    fn each_night_one_more_figure_stands_at_the_edge_of_their_light() {
        let mut memory = rich_memory();
        let event = find("one_more_each_night").unwrap();
        let p = place(event, &memory, &mut Rng::new(1));
        assert!(p.is_some(), "a light near them");
        let figures = |memory: &Memory| {
            commands(event, p, memory, &mut ids(), &Now::default(), None)
                .iter()
                .filter(|(c, _)| matches!(c, Command::Figure { together, .. } if *together > 0))
                .count()
        };
        assert_eq!(figures(&memory), 1, "the first night, one");
        memory.record("one_more_each_night", p, 1.0, true, "");
        assert_eq!(figures(&memory), 2, "the next, two");
        memory.record("one_more_each_night", p, 2.0, true, "");
        assert_eq!(figures(&memory), 3);
        for day in 3..12 {
            memory.record("one_more_each_night", p, day as f32, true, "");
        }
        assert_eq!(
            figures(&memory),
            crate::events::ONE_MORE_MOST as usize,
            "never more than the most"
        );
        assert_eq!(event.most_per_stay, crate::events::ONE_MORE_MOST);
    }

    #[test]
    fn at_night_a_figure_stands_outside_every_window_and_they_go_together() {
        let mut memory = rich_memory();
        let center = memory.home.as_ref().unwrap().center;
        memory.fixtures.retain(|f| f.kind != "glass_pane");
        // Two panes of one window, and two more windows.
        for pos in [
            [center[0] + 5, 65, center[2]],
            [center[0] + 5, 65, center[2] + 1],
            [center[0] - 5, 65, center[2]],
            [center[0], 65, center[2] + 5],
        ] {
            memory.fixtures.push(crate::memory::Fixture {
                kind: "glass_pane".to_string(),
                pos,
            });
        }
        let every = find("every_window").unwrap();
        let p = place(every, &memory, &mut Rng::new(1));
        assert_eq!(p, Some(center));
        let made = commands(every, p, &memory, &mut ids(), &Now::default(), None);
        assert_eq!(made.len(), 3, "one a window: {made:?}");
        for (command, _) in &made {
            let Command::Figure {
                together,
                once_seen,
                pos,
                ..
            } = command
            else {
                panic!("{command:?}");
            };
            assert!(*together > 0 && *once_seen);
            assert_eq!(pos[1], 64.0, "its head at the glass");
            let out = ((pos[0] - 0.5 - center[0] as f64).powi(2)
                + (pos[2] - 0.5 - center[2] as f64).powi(2))
            .sqrt();
            assert!(out >= 5.9, "outside, {out} out");
        }

        // One window only: not this event.
        memory
            .fixtures
            .retain(|f| f.kind != "glass_pane" || f.pos[0] == center[0] + 5);
        assert_eq!(place(every, &memory, &mut Rng::new(1)), None);
    }

    #[test]
    fn rowing_alone_the_figure_sits_behind_them() {
        let mut memory = rich_memory();
        let boat = find("boat_passenger").unwrap();
        memory.last.online = true;
        memory.last.riding = "minecraft:oak_boat".to_string();
        assert!(crate::brain::offer::meets(
            crate::events::Need::TargetInBoat,
            &memory,
            &Now::default()
        ));
        memory.last.riding = "minecraft:oak_chest_boat".to_string();
        assert!(
            !crate::brain::offer::meets(
                crate::events::Need::TargetInBoat,
                &memory,
                &Now::default()
            ),
            "a chest boat has no seat"
        );
        memory.last.riding = "minecraft:bamboo_raft".to_string();
        assert!(crate::brain::offer::meets(
            crate::events::Need::TargetInBoat,
            &memory,
            &Now::default()
        ));
        let made = commands(
            boat,
            place(boat, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::Figure { rides_behind: true, once_seen: true, held, .. }, 0)] if held.id.is_empty()),
            "{made:?}"
        );
    }

    #[test]
    fn in_the_dark_the_figure_runs_at_them_from_ahead() {
        let mut memory = rich_memory();
        memory.last.pos = [0.0, 64.0, 0.0];
        // Facing south (+z).
        memory.last.yaw = 0.0;
        let mut rng = Rng::new(3);
        let hunted = find("hunted").unwrap();
        let p = place(hunted, &memory, &mut rng).unwrap();
        assert_eq!(
            p,
            [0, -10_000, HUNTED_FROM as i32],
            "ahead of them, on the ground"
        );
        let made = commands(hunted, Some(p), &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { rushes: true, held, name_shown: false, .. }, 0)] if held.id.is_empty()),
            "{made:?}"
        );
    }

    #[test]
    fn at_night_the_figure_crawls_from_their_door_to_their_bed() {
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let crawl = find("the_crawl").unwrap();
        let door = memory.usual_door().unwrap();
        let bed = memory.home.as_ref().unwrap().bed.unwrap();
        let p = place(crawl, &memory, &mut rng);
        assert_eq!(p, Some(door));
        let made = commands(crawl, p, &memory, &mut ids(), &Now::default(), None);
        let [(
            Command::Figure {
                crawls: true,
                path,
                pos,
                held,
                ..
            },
            0,
        )] = &made[..]
        else {
            panic!("{made:?}");
        };
        assert!(held.id.is_empty());
        assert_eq!(path.first(), Some(pos), "from the doorway");
        let end = path.last().unwrap();
        assert_eq!(
            [end[0], end[2]],
            [bed[0] as f64 + 0.5, bed[2] as f64 + 0.5],
            "to the bed"
        );
        assert_eq!(end[1], door[1] as f64, "along the floor");
    }

    #[test]
    fn as_they_wake_the_figure_hangs_from_the_ceiling_over_the_bed() {
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let hung = find("upside_down").unwrap();
        assert!(hung.needs.contains(&crate::events::Need::JustWoke));
        let bed = memory.home.as_ref().unwrap().bed.unwrap();
        let p = place(hung, &memory, &mut rng);
        assert_eq!(p, Some(bed));
        let made = commands(hung, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { upside_down: true, once_seen: true, pos, held, .. }, 0)]
                if pos[0] == bed[0] as f64 + 0.5 && pos[1] == bed[1] as f64 && held.id.is_empty()),
            "{made:?}"
        );
        assert!(
            crate::brain::WOKE_EVENTS.contains(&"upside_down"),
            "a waking may start it"
        );
    }

    #[test]
    fn at_night_figures_stand_in_a_ring_round_the_house() {
        let memory = rich_memory();
        let mut rng = Rng::new(3);
        let ring = find("ring_of_figures").unwrap();
        assert!(ring.needs.contains(&crate::events::Need::TargetIndoors));
        let center = memory.home.as_ref().unwrap().center;
        let p = place(ring, &memory, &mut rng);
        assert_eq!(p, Some(center));
        let made = commands(ring, p, &memory, &mut ids(), &Now::default(), None);
        assert_eq!(made.len(), 8);
        let mut numbers = Vec::new();
        for (command, at) in &made {
            let Command::Figure {
                figure,
                pos,
                held,
                once_seen,
                seconds,
                ..
            } = command
            else {
                panic!("{command:?}");
            };
            assert_eq!(*at, 0, "all at once");
            assert!(*once_seen && held.id.is_empty() && *seconds > 0.0);
            assert!(pos[1] < -1000.0, "on the ground");
            let out = ((pos[0] - 0.5 - center[0] as f64).powi(2)
                + (pos[2] - 0.5 - center[2] as f64).powi(2))
            .sqrt();
            assert!((out - RING_BLOCKS).abs() <= 1.0, "{out} blocks out");
            numbers.push(*figure);
        }
        numbers.sort_unstable();
        numbers.dedup();
        assert_eq!(numbers.len(), 8, "each its own figure");
        assert!(!numbers.contains(&1), "apart from the one figure");
    }

    #[test]
    fn coming_home_a_figure_works_at_their_table() {
        let memory = rich_memory();
        let event = find("at_their_table").unwrap();
        let table = memory
            .fixtures("crafting_table")
            .next()
            .expect("a table")
            .pos;
        let p = place(event, &memory, &mut Rng::new(1));
        assert_eq!(p, Some(table));
        let made = commands(event, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Figure { swinging: true, faces: Some(f), vanish_within, .. }, 0)] if f[0] == table[0] as f64 + 0.5 && *vanish_within > 0.0),
            "{made:?}"
        );
    }

    #[test]
    fn far_off_a_figure_jumps_in_place() {
        let memory = rich_memory();
        let event = find("jumping_far").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::Figure { jumps: true, pos, .. }, 0)] if pos[1] < -1000.0),
            "{made:?}"
        );
    }

    #[test]
    fn coming_home_a_figure_waves_from_inside_a_window() {
        let memory = rich_memory();
        let event = find("waves_from_inside").unwrap();
        let center = memory.home.as_ref().unwrap().center;
        let pane = windows(&memory, center)[0];
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let Command::Figure { pos, .. } = &made[0].0 else {
            panic!("{made:?}")
        };
        let inside = |p: f64, c: i32, q: i32| (p - 0.5 - c as f64).abs() < (q - c).abs() as f64;
        assert!(
            inside(pos[0], center[0], pane[0]) || inside(pos[2], center[2], pane[2]),
            "inside the glass: {pos:?} {pane:?}"
        );
        assert_eq!(
            made.iter()
                .filter(|(c, _)| matches!(c, Command::FigureSwing { damage, .. } if *damage == 0.0))
                .count(),
            3,
            "a wave, no harm"
        );
    }

    #[test]
    fn prints_lead_from_their_door_toward_a_cave() {
        let mut memory = rich_memory();
        let door = memory.usual_door().unwrap();
        memory.caves = vec![[door[0] + 100, 20, door[2]]];
        let event = find("prints_to_the_cave").unwrap();
        let p = place(event, &memory, &mut Rng::new(1));
        assert_eq!(
            p,
            Some([door[0] + 48, door[1], door[2]]),
            "toward it, forty-eight blocks at most"
        );
        let made = commands(event, p, &memory, &mut ids(), &Now::default(), None);
        assert!(
            matches!(&made[..], [(Command::Prints { from, .. }, 0)] if *from == door),
            "{made:?}"
        );
    }

    #[test]
    fn their_horse_stands_with_a_figure_in_its_saddle() {
        let mut memory = rich_memory();
        memory.last.creatures_near.push(crate::memory::Creature {
            id: 88,
            kind: "horse".to_string(),
            pos: [20.0, 64.0, 0.0],
            pet: true,
            ..Default::default()
        });
        let event = find("rides_their_horse").unwrap();
        let horse = their_animal(&memory, "horse", false)
            .expect("their horse")
            .id;
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::Figure { rides, .. }, 0)] if *rides == horse),
            "{made:?}"
        );
    }

    #[test]
    fn coming_home_a_figure_eats_their_food_at_their_chest() {
        let mut memory = rich_memory();
        memory.containers[0]
            .contents
            .push(stack("minecraft:bread", 5, 22));
        let event = find("eats_at_their_chest").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let Command::Figure {
            eats: true, held, ..
        } = &made[0].0
        else {
            panic!("{made:?}")
        };
        assert!(
            held.id.ends_with("bread")
                || chest_food(&memory.containers[0]).as_deref() == Some(held.id.as_str())
        );
        assert!(
            made.iter()
                .filter(
                    |(c, _)| matches!(c, Command::PlaySound { sound, .. } if sound.ends_with("eat"))
                )
                .count()
                >= 3
        );
    }

    #[test]
    fn in_their_window_a_figure_copies_their_every_move() {
        let mut memory = rich_memory();
        let center = memory.home.as_ref().unwrap().center;
        memory.last.pos = [center[0] as f64 + 8.5, 64.0, center[2] as f64 + 0.5];
        let event = find("mirror_in_the_window").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(
                &made[..],
                [(
                    Command::Figure {
                        mirror: true,
                        once_seen: true,
                        ..
                    },
                    0
                )]
            ),
            "{made:?}"
        );
    }

    #[test]
    fn at_night_every_door_of_the_house_opens_at_once() {
        let mut memory = rich_memory();
        memory.home.as_mut().unwrap().doors = vec![[4, 64, 0], [-4, 64, 0], [0, 64, 4]];
        let event = find("every_door_opens").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 3);
        assert!(made.iter().all(|(c, at)| *at == 0 && matches!(c, Command::ToggleBlock { to: Some(true), .. })), "{made:?}");
        memory.home.as_mut().unwrap().doors.truncate(1);
        assert_eq!(
            place(event, &memory, &mut Rng::new(1)),
            None,
            "one door is no 'every door'"
        );
    }

    #[test]
    fn their_flowers_wilt_for_a_day_then_bloom_again() {
        let memory = rich_memory();
        let event = find("flowers_wilted").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let dead: Vec<_> = made.iter().filter(|(c, at)| *at == 0 && matches!(c, Command::SwapBlock { block, .. } if block == "minecraft:dead_bush")).collect();
        let back: Vec<_> = made.iter().filter(|(c, at)| *at == WILTED_FOR && matches!(c, Command::SwapBlock { only, .. } if only == &vec!["minecraft:dead_bush".to_string()])).collect();
        assert!(!dead.is_empty(), "{made:?}");
        assert_eq!(dead.len(), back.len(), "each one back");
    }

    #[test]
    fn at_night_a_camp_stands_out_where_they_walk_and_is_gone_by_morning() {
        let mut memory = rich_memory();
        memory.routes = vec![crate::memory::Route {
            points: (0..40).map(|i| [i * 2, 64, 0]).collect(),
            walked: 3,
            first_day: 0.0,
        }];
        memory.last.pos = [0.5, 64.0, 0.5];
        let event = find("camp_nearby").unwrap();
        let p = place(event, &memory, &mut Rng::new(1)).unwrap();
        assert!(distance(p, [0, 64, 0]) >= 24.0);
        let now = Now {
            day_time: 14_000,
            ..Now::default()
        };
        let made = commands(event, Some(p), &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::SetBlock { block, .. } if block == "minecraft:campfire")
        );
        let gone: Vec<u64> = made.iter().filter(|(c, _)| matches!(c, Command::SetBlock { block, only: true, .. } if block == "minecraft:air")).map(|(_, at)| *at).collect();
        assert_eq!(gone.len(), 3, "the fire and both seats taken away");
        assert!(gone.iter().all(|at| *at >= 10_000), "at dawn: {gone:?}");
        assert!(
            made.iter()
                .any(|(c, _)| matches!(c, Command::Figure { eats: true, .. })),
            "a figure by it"
        );
    }

    #[test]
    fn overnight_a_ring_of_stone_pillars_stands_far_off() {
        let memory = rich_memory();
        let event = find("stone_circle").unwrap();
        let p = place(event, &memory, &mut Rng::new(1)).unwrap();
        assert_eq!(p[1], -10_000, "on the ground");
        let made = commands(event, Some(p), &memory, &mut ids(), &Now::default(), None);
        let [(Command::PlaceBlueprint { parts, .. }, 0)] = &made[..] else {
            panic!("{made:?}")
        };
        assert_eq!(parts.len(), 24, "eight pillars three high");
        // Five out, as near as whole blocks go (the diagonals round to five and a half).
        assert!(parts.iter().all(|p| {
            let out = ((p.at[0] * p.at[0] + p.at[2] * p.at[2]) as f64).sqrt();
            p.at[1] >= 1 && (4.0..=6.0).contains(&out)
        }));
    }

    #[test]
    fn what_they_hold_slips_from_their_hand_never_with_monsters_near() {
        let mut memory = rich_memory();
        memory.last.main_hand = "minecraft:iron_pickaxe".to_string();
        memory.last.hostiles_near.clear();
        let event = find("slips_from_hand").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::DropHeld { .. }, 0)]),
            "{made:?}"
        );
        memory.last.hostiles_near.push(crate::memory::Creature {
            id: 3,
            kind: "zombie".to_string(),
            ..Default::default()
        });
        assert_eq!(
            place(event, &memory, &mut Rng::new(1)),
            None,
            "not with a monster near"
        );
        memory.last.hostiles_near.clear();
        memory.last.main_hand.clear();
        assert_eq!(
            place(event, &memory, &mut Rng::new(1)),
            None,
            "nothing held"
        );
    }

    #[test]
    fn inside_a_window_is_heard_breaking_and_every_pane_is_whole() {
        let memory = rich_memory();
        let event = find("glass_breaks").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(made.iter().all(|(c, _)| matches!(c, Command::PlaySound { sound, .. } if sound == "minecraft:block.glass.break")), "a sound alone: {made:?}");
        assert!(!made.is_empty());
    }

    #[test]
    fn their_pet_stares_at_a_wall_a_minute() {
        let mut memory = rich_memory();
        memory.walls = vec![
            ([9, 65, 9], "minecraft:oak_planks".to_string()),
            ([1, 65, 3], "minecraft:oak_planks".to_string()),
        ];
        let pet = memory.pets().next().expect("a pet").clone();
        let event = find("pet_stares_at_wall").unwrap();
        let wall = place(event, &memory, &mut Rng::new(1)).unwrap();
        let made = commands(
            event,
            Some(wall),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[..], [(Command::Hold { entity, at: Some(at), seconds, .. }, 0)] if *entity == pet.id && at[0] == wall[0] as f64 + 0.5 && *seconds >= 30.0),
            "{made:?}"
        );
    }

    #[test]
    fn at_night_bats_flutter_in_the_house_a_minute() {
        let memory = rich_memory();
        let event = find("bats_in_the_house").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 4);
        assert!(made.iter().all(|(c, _)| matches!(c, Command::PlaceEntity { of, seconds, .. } if of == "minecraft:bat" && *seconds > 0.0)), "for a while only: {made:?}");
    }

    #[test]
    fn in_a_blink_an_hour_of_their_day_is_gone() {
        let memory = rich_memory();
        let event = find("lost_hour").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(matches!(&made[0].0, Command::Hint { hint, .. } if hint == "lights_fail"));
        assert!(
            matches!(&made[1], (Command::SkipTime { ticks: 1_000, .. }, at) if *at > 0),
            "{made:?}"
        );
    }

    #[test]
    fn the_chest_they_walk_to_will_not_open_and_knocks_inside() {
        let memory = rich_memory();
        let chest = memory.used_container().unwrap().pos;
        let event = find("chest_wont_open").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert!(
            matches!(&made[0], (Command::HoldContainer { pos, .. }, 0) if *pos == chest),
            "{made:?}"
        );
        assert_eq!(
            made.iter()
                .filter(|(c, _)| matches!(c, Command::PlaySound { .. }))
                .count(),
            2,
            "two knocks"
        );
    }

    #[test]
    fn their_name_is_trodden_into_the_snow() {
        let mut memory = rich_memory();
        memory.target_name = "Ab".to_string();
        let event = find("name_in_the_snow").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let [(Command::PlaceBlueprint { parts, replace, .. }, 0)] = &made[..] else {
            panic!("{made:?}")
        };
        assert_eq!(replace, &vec!["minecraft:snow".to_string()], "snow only");
        let cells = (snow_letter('A').len() + snow_letter('b').len()) * 2;
        assert_eq!(
            parts.len(),
            cells,
            "each letter's cells, at the ground and a block over"
        );
        assert!(
            parts.iter().any(|p| p.at[0] >= 4),
            "the second letter four blocks on"
        );
        assert_eq!(snow_letter('?'), Vec::<[i32; 2]>::new());
    }

    #[test]
    fn overnight_a_ring_of_torches_stands_round_the_house() {
        let memory = rich_memory();
        let center = memory.home.as_ref().unwrap().center;
        let event = find("torch_ring").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 16);
        for (c, _) in &made {
            let Command::SetBlock {
                pos, block, settle, ..
            } = c
            else {
                panic!("{c:?}")
            };
            assert_eq!(block, "minecraft:torch");
            assert!(*settle > 0);
            let out = (((pos[0] - center[0]).pow(2) + (pos[2] - center[2]).pow(2)) as f64).sqrt();
            assert!((11.0..=13.0).contains(&out), "{out}");
        }
    }

    #[test]
    fn a_line_of_their_blocks_leads_from_their_door() {
        let mut memory = rich_memory();
        memory.containers[0]
            .contents
            .push(stack("minecraft:cobblestone", 40, 23));
        let event = find("breadcrumbs").unwrap();
        let made = commands(
            event,
            place(event, &memory, &mut Rng::new(1)),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        let takes = made.iter().filter(|(c, _)| matches!(c, Command::TakeItem { count: 1, keep, .. } if keep.starts_with("crumb_"))).count();
        let drops: Vec<&String> = made
            .iter()
            .filter_map(|(c, _)| match c {
                Command::DropItem { kept, .. } => Some(kept),
                _ => None,
            })
            .collect();
        assert_eq!(takes, CRUMBS);
        assert_eq!(drops.len(), CRUMBS, "each one taken is the one dropped");
        assert!(drops.iter().all(|k| k.starts_with("crumb_")));
    }

    #[test]
    fn asleep_the_figure_comes_in_at_their_door_and_chokes_them_with_a_lead() {
        let memory = rich_memory();
        let mut rng = Rng::new(8);
        let choke = find("choke").unwrap();
        assert_eq!(choke.effect(), "night");
        assert!(choke.needs.contains(&crate::events::Need::TargetInBed));
        let door = memory.usual_door().expect("a door");
        let p = place(choke, &memory, &mut rng);
        assert_eq!(p, Some(door), "it comes in at their door");

        let made = commands(choke, p, &memory, &mut ids(), &Now::default(), None);
        assert_eq!(made.len(), 2, "{made:?}");
        // Their eyes are kept open by the host's part of the command: no hint of the mind's,
        // which events.glitches would have kept back.
        assert!(!made.iter().any(|(c, _)| matches!(c, Command::Hint { .. })));
        // In the doorway, standing on the door's own level, empty-handed until the bed.
        assert!(
            matches!(&made[0], (Command::Figure { pos, held, path, figure: 1, .. }, 0)
                if pos[1] == door[1] as f64 && held.id.is_empty() && path.is_empty()),
            "{:?}",
            made[0]
        );
        // After the figure is shown: the lead, a heart a squeeze, never a kill; the door it came
        // in by opened by the choke itself, so a door gone since never stops the rest (a toggle
        // of its own that failed would have).
        assert!(
            matches!(&made[1], (Command::Choke { item, damage, spare: true, figure: 1, seconds, door: Some(d), .. }, 1)
                if item == "minecraft:lead" && *damage == 2.0 && *seconds == 5.0 && *d == door),
            "{:?}",
            made[1]
        );

        // The variation: the same walk, a crossbow and one shot, two hearts, then a watch.
        let shot = find("shot_in_bed").unwrap();
        assert_eq!(shot.effect(), "night");
        let made = commands(
            shot,
            place(shot, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
            None,
        );
        assert_eq!(made.len(), 2, "{made:?}");
        assert!(
            matches!(&made[1], (Command::Choke { item, how, damage, spare: true, seconds, door: Some(d), .. }, 1)
                if item == "minecraft:crossbow" && how == "shoot" && *damage == 4.0 && *seconds == 2.0 && *d == door),
            "{:?}",
            made[1]
        );
    }

    #[test]
    fn a_wall_block_becomes_its_cousin_and_the_bed_breaks_into_its_item() {
        let memory = rich_memory();
        let mut rng = Rng::new(8);
        let now = Now::default();

        let swap = find("window_bars").unwrap();
        let p = place(swap, &memory, &mut rng);
        assert_eq!(p, Some([5, 66, 5]));
        let made = commands(swap, p, &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::SwapBlock { block, only, .. } if block == "minecraft:iron_bars" && only == &["minecraft:glass_pane".to_string()]),
            "{:?}",
            made[0].0
        );

        let torch = find("torch_blue").unwrap();
        let made = commands(
            torch,
            place(torch, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(&made[0].0, Command::SwapBlock { block, only, .. } if block == "minecraft:soul_torch" && only == &["minecraft:torch".to_string()]),
            "{:?}",
            made[0].0
        );

        let wall = find("wall_swap").unwrap();
        let made = commands(wall, Some([5, 65, 5]), &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::SwapBlock { block, .. } if block.ends_with("_planks") && block != "minecraft:oak_planks"),
            "{:?}",
            made[0].0
        );
        assert_eq!(
            crate::memory::family("minecraft:cherry_planks").len(),
            crate::memory::WOODS.len()
        );
        assert_eq!(
            crate::memory::cousins("minecraft:lime_wool").len(),
            crate::memory::COLOURS.len() - 1
        );
        assert!(
            crate::memory::cousins("minecraft:oak_door").is_empty(),
            "doors are two blocks and stay"
        );

        let bed = find("bed_breaks").unwrap();
        let made = commands(
            bed,
            place(bed, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(matches!(made[0].0, Command::CancelSleep { .. }));
        assert!(
            matches!(
                made[1].0,
                Command::BreakBlock {
                    pos: [0, 64, 0],
                    ..
                }
            ),
            "{:?}",
            made[1].0
        );
        assert_eq!(
            crate::memory::cousins("minecraft:chest"),
            Vec::<String>::new(),
            "a chest has no family"
        );
    }

    #[test]
    fn frames_turn_the_same_way_stands_follow_the_target_and_the_pet_moves_to_a_corner() {
        let memory = rich_memory();
        let mut rng = Rng::new(2);
        let now = Now::default();

        let frames = find("frames_turn").unwrap();
        let made = commands(frames, None, &memory, &mut ids(), &now, None);
        assert_eq!(made.len(), 3);
        let rotations: Vec<Option<u32>> = made
            .iter()
            .map(|(c, _)| match c {
                Command::RotateFrame { rotation, .. } => *rotation,
                other => panic!("{other:?}"),
            })
            .collect();
        assert!(rotations.iter().all(|r| r.is_some() && *r == rotations[0]));

        let watch = find("stand_watch").unwrap();
        let made = commands(watch, None, &memory, &mut ids(), &now, None);
        assert!(
            matches!(
                made[0].0,
                Command::PoseStand {
                    entity: 105,
                    follow: Some(0),
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );

        let pet = find("pet_moved").unwrap();
        let made = commands(
            pet,
            place(pet, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(made[0].0, Command::MoveEntity { entity: 206, .. }),
            "{:?}",
            made[0].0
        );

        let villager = find("villager_at_door").unwrap();
        let made = commands(
            villager,
            place(villager, &memory, &mut rng),
            &memory,
            &mut ids(),
            &now,
            None,
        );
        assert!(
            matches!(made[0].0, Command::MoveEntity { entity: 204, .. }),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn the_beds_of_batch_five_go_where_they_should() {
        let mut memory = memory();
        memory.last.pos = [-2.5, 64.0, 0.5]; // woken on the bed's west side
        let now = Now::default();
        let mut rng = Rng::new(4);

        // At the foot of the bed: the floor two blocks past it, across it from them.
        let foot = find("foot_of_bed").unwrap();
        assert_eq!(place(foot, &memory, &mut rng), Some([2, 63, 0]));
        let made = commands(foot, Some([2, 63, 0]), &memory, &mut ids(), &now, None);
        assert!(
            matches!(&made[0].0, Command::Figure { pos, seconds, .. } if pos[1] == 64.0 && *seconds == 0.5),
            "standing on the floor by the bed for half a second: {made:?}"
        );

        // Under the bed: the block below it, a still sound there.
        let under = find("under_the_bed").unwrap();
        assert_eq!(place(under, &memory, &mut rng), Some([0, 63, 0]));
        let made = commands(under, Some([0, 63, 0]), &memory, &mut ids(), &now, None);
        assert!(made.len() >= 3);
        assert!(
            made.iter()
                .all(|(c, _)| matches!(c, Command::PlaySound { pos, .. } if pos[1] < 64.0)),
            "{made:?}"
        );

        // The sheets: another colour, and the new one never among what may be there.
        let sheets = find("sheets_changed").unwrap();
        let made = commands(sheets, Some([0, 64, 0]), &memory, &mut ids(), &now, None);
        match &made[0].0 {
            Command::SwapBlock { block, only, .. } => {
                assert!(block.ends_with("_bed"));
                assert!(!only.contains(block));
                assert_eq!(only.len(), crate::memory::COLOURS.len() - 1);
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            &commands(find("shift_bed").unwrap(), Some([0, 64, 0]), &memory, &mut ids(), &now, None)[0].0,
            Command::ShiftBed { pos, .. } if *pos == [0, 64, 0]
        ));
        assert!(matches!(
            &commands(find("second_bed").unwrap(), Some([0, 64, 0]), &memory, &mut ids(), &now, None)[0].0,
            Command::BedBeside { block, .. } if block.ends_with("_bed")
        ));
    }

    #[test]
    fn waking_is_noticed_once() {
        use craftkit::world::{Observations, Player};
        let mut memory = memory();
        memory.target = Some(0);
        let lying = |sleeping| Observations {
            players: vec![Player {
                sleeping,
                ..Player::default()
            }],
            ..Observations::default()
        };
        memory.observe(&lying(true), 0.6);
        assert!(!memory.woke_now);
        memory.observe(&lying(false), 1.0);
        assert!(memory.woke_now);
        memory.woke_now = false;
        memory.observe(&lying(false), 1.01);
        assert!(!memory.woke_now, "once");
    }

    #[test]
    fn the_farm_of_batch_six_sends_what_it_should() {
        let memory = rich_memory();
        let now = Now {
            day_time: 1_000,
            ..Now::default()
        };
        let mut rng = Rng::new(6);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        // The scarecrow wears the very helmet taken: kept, then put on.
        let scarecrow = made("scarecrow", &mut rng);
        assert!(matches!(&scarecrow[0].0, Command::TakeItem { keep, .. } if keep == "scarecrow"));
        let stand = memory.decor("armor_stand").next().unwrap().id;
        assert!(scarecrow.iter().any(|(c, _)| matches!(c, Command::EquipEntity { kept, entity, .. } if kept == "scarecrow" && *entity == stand)));

        // The animals face the house, not the player.
        let facing = made("face_the_house", &mut rng);
        assert!(!facing.is_empty());
        assert!(facing.iter().all(
            |(c, _)| matches!(c, Command::Hold { at: Some(at), .. } if *at == [0.5, 64.5, 0.5])
        ));

        // One more till noon: from an hour after dawn, the four minutes and more left.
        let more = made("one_more", &mut rng);
        assert!(
            matches!(&more[0].0, Command::PlaceEntity { of, seconds, .. } if *of == format!("minecraft:{}", memory.kept_most().unwrap().0) && *seconds == 250.0),
            "{more:?}"
        );

        // A nip spares them; the horse's lead moves; eggs go down one by one.
        assert!(made("animals_nip", &mut rng).iter().all(|(c, _)| matches!(
            c,
            Command::Charge {
                spare: true,
                hits: 2,
                ..
            }
        )));
        assert_eq!(
            made("animals_turn", &mut rng).len(),
            memory.animals_near_target().len().min(3),
            "the animals near, three at most"
        );
        assert!(matches!(
            &made("horse_moved", &mut rng)[0].0,
            Command::MoveLeash { entity: 80, .. }
        ));
        assert!(made("eggs_to_door", &mut rng).iter().all(
            |(c, _)| matches!(c, Command::DropItem { item, .. } if item.id == "minecraft:egg")
        ));

        // The field: a word cut into it, a row withered, every cut a seedling.
        for id in ["field_word", "row_withered"] {
            let cut = made(id, &mut rng);
            assert!(!cut.is_empty(), "{id}");
            assert!(cut.iter().all(|(c, _)| matches!(c, Command::SetState { property, value, .. } if property == "age" && value == "0")));
        }
    }

    #[test]
    fn the_signs_and_kills_of_batch_seven_send_what_they_should() {
        let memory = rich_memory();
        let now = Now::default();
        let mut rng = Rng::new(7);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            (pos, commands(event, pos, &memory, &mut ids(), &now, None))
        };

        let (_, word) = made("sign_word", &mut rng);
        assert!(
            matches!(&word[0].0, Command::SetSign { one_line: true, lines, .. } if lines.len() == 1)
        );

        // The count: at the kill sign, two out from its door, the number as it is.
        let (at, count) = made("count_up", &mut rng);
        assert_eq!(at, Some([5, 64, 0]));
        assert!(
            matches!(&count[0].0, Command::SetSign { lines, one_line: false, .. } if lines[0] == "14 cows"),
            "{count:?}"
        );

        let (_, weapon) = made("weapon_on_wall", &mut rng);
        assert!(
            matches!(
                &weapon[0].0,
                Command::SetFrameItem {
                    from: Some(_),
                    slot: 11,
                    ..
                }
            ),
            "{weapon:?}"
        );

        // The zombie's head, on a post, facing the door.
        let (_, head) = made("head_on_post", &mut rng);
        assert!(
            matches!(&head[0].0, Command::SetBlock { block, .. } if block == "minecraft:oak_fence")
        );
        assert!(
            matches!(&head[1].0, Command::SetBlock { block, state, .. } if block == "minecraft:zombie_head" && state.starts_with("rotation="))
        );

        // A grave for each kind: a stone and a sign each, the counts on them.
        let (_, graves) = made("grave_each", &mut rng);
        let signs: Vec<&Vec<String>> = graves
            .iter()
            .filter_map(|(c, _)| match c {
                Command::SetSign { lines, .. } => Some(lines),
                _ => None,
            })
            .collect();
        assert_eq!(signs.len(), 2);
        assert_eq!(signs[0], &vec!["14".to_string(), "cows".to_string()]);
        assert_eq!(signs[1], &vec!["3".to_string(), "zombies".to_string()]);
    }

    #[test]
    fn a_kill_is_noticed_for_the_count() {
        use craftkit::world::{Event, EventKind, Observations, Player};
        let mut memory = memory();
        memory.target = Some(0);
        memory.observe(
            &Observations {
                players: vec![Player::default()],
                events: vec![Event {
                    player: 0,
                    kind: EventKind::Killed,
                    detail: "minecraft:cow".to_string(),
                    ..Event::default()
                }],
                ..Observations::default()
            },
            0.3,
        );

        assert_eq!(memory.killed_now.as_deref(), Some("cow"));
    }

    #[test]
    fn what_they_typed_fits_a_sign_in_whole_words() {
        assert_eq!(
            sign_lines("Who keeps opening my DOOR at night and why does it"),
            vec![
                "who keeps",
                "opening my door",
                "at night and",
                "why does it"
            ]
        );
        assert_eq!(
            sign_lines("a supercalifragilistic word"),
            vec!["a", "supercalifragil", "word"]
        );
        assert!(sign_lines("   ").is_empty());
        assert_eq!(
            sign_lines("one two three four five six seven eight nine ten eleven twelve thirteen")
                .len(),
            4,
            "four lines at most"
        );
    }

    #[test]
    fn the_chat_of_batch_eight_sends_what_it_should() {
        let mut memory = rich_memory();
        memory.last_said = "hello is anyone there".to_string();
        let now = Now::default();
        let mut rng = Rng::new(8);
        let made = |id: &str, memory: &Memory, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, memory, rng);
            commands(event, pos, memory, &mut ids(), &now, None)
        };
        let message = |c: &Command| match c {
            Command::GameMessage { message, text, .. } => Some((message.clone(), text.clone())),
            _ => None,
        };

        assert_eq!(
            message(&made("sleep_message", &memory, &mut rng)[0].0),
            Some(("sleepers".to_string(), String::new()))
        );
        assert_eq!(
            message(&made("advancement", &memory, &mut rng)[0].0)
                .unwrap()
                .0,
            "advancement"
        );
        assert_eq!(
            message(&made("ping_fades", &memory, &mut rng)[0].0)
                .unwrap()
                .0,
            "no_ping"
        );
        assert_eq!(
            message(&made("empty_message", &memory, &mut rng)[0].0),
            Some(("says".to_string(), String::new()))
        );
        let (kind, key) = message(&made("death_message", &memory, &mut rng)[0].0).unwrap();
        assert_eq!(kind, "death");
        assert!(key.starts_with("death."), "{key}");

        let sign = made("words_on_sign", &memory, &mut rng);
        assert!(
            matches!(&sign[1].0, Command::SetSign { lines, .. } if lines == &vec!["hello is anyone", "there"]),
            "{sign:?}"
        );
    }

    #[test]
    fn the_workshop_of_batch_nine_sends_what_it_should() {
        let memory = rich_memory();
        let now = Now::default();
        let mut rng = Rng::new(9);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        assert!(matches!(
            &made("jukebox", &mut rng)[0].0,
            Command::Jukebox { slot: 12, .. }
        ));
        assert!(
            matches!(&made("lectern_page", &mut rng)[0].0, Command::TurnPage { pos, .. } if *pos == [-1, 64, -2])
        );
        assert!(matches!(
            &made("brewing_done", &mut rng)[0].0,
            Command::Brew { slot: 14, .. }
        ));
        assert!(matches!(
            &made("grindstone_strip", &mut rng)[0].0,
            Command::Strip { .. }
        ));
        assert!(
            matches!(&made("composter_fills", &mut rng)[0].0, Command::SetState { property, value, .. } if property == "level" && value == "@up")
        );
        assert!(
            matches!(&made("bell_rings", &mut rng)[0].0, Command::ToggleBlock { pos, .. } if *pos == [10, 65, 0])
        );

        // The fuel: theirs out to a free slot, the other in, as long a burn.
        let swap = made("swap_fuel", &mut rng);
        assert!(matches!(
            &swap[0].0,
            Command::MoveItem {
                from_slot: 1,
                count: 3,
                ..
            }
        ));
        assert!(
            matches!(
                &swap[1].0,
                Command::MoveItem {
                    from_slot: 13,
                    to_slot: 1,
                    count: 16,
                    ..
                }
            ),
            "{swap:?}"
        );

        // The lamps: an even number of toggles, so the lamp ends as it was.
        assert_eq!(made("lamps_blink", &mut rng).len() % 2, 0);

        // Torches from their stick and coal, on the table.
        let crafted = made("crafted_for_you", &mut rng);
        assert!(
            crafted
                .iter()
                .filter(|(c, _)| matches!(c, Command::TakeItem { .. }))
                .count()
                >= 1
        );
        assert!(
            matches!(&crafted.last().unwrap().0, Command::DropItem { item, .. } if item.id == "minecraft:torch" && item.count == 4),
            "{crafted:?}"
        );
    }

    #[test]
    fn the_water_and_snow_of_batch_ten_send_what_they_should() {
        let memory = rich_memory();
        let now = Now::default();
        let mut rng = Rng::new(10);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            (pos, commands(event, pos, &memory, &mut ids(), &now, None))
        };

        // Prints out from the door, and in to it from twenty blocks off.
        let door = memory.usual_door().unwrap();
        let (_, out) = made("trail", &mut rng);
        assert!(matches!(&out[0].0, Command::Prints { from, .. } if *from == door));
        let (_, back) = made("trail_in", &mut rng);
        assert!(
            matches!(&back[0].0, Command::Prints { from, to, .. } if *to == door && distance(*from, door) >= 19.0)
        );

        // The squid faces them; the snowman ends in a pumpkin; the steps drip.
        let (_, shape) = made("under_the_water", &mut rng);
        assert!(
            matches!(&shape[0].0, Command::PlaceEntity { of, face: Some(_), seconds, .. } if of == "minecraft:squid" && *seconds == 20.0)
        );
        let (_, snowman) = made("snowman", &mut rng);
        assert_eq!(snowman.len(), 3);
        assert!(
            matches!(&snowman[2].0, Command::SetBlock { block, state, .. } if block == "minecraft:carved_pumpkin" && state.starts_with("facing="))
        );
        let (_, steps) = made("wet_steps", &mut rng);
        assert!(
            steps.len() >= 6
                && steps
                    .iter()
                    .all(|(c, _)| matches!(c, Command::PlaySound { .. }))
        );

        // The bobber: its ripples for a minute, a cast, splashes, a catch.
        let (_, fishing) = made("fishing_line", &mut rng);
        assert!(
            matches!(&fishing[0].0, Command::Hint { hint, seconds, .. } if hint == "bobber" && *seconds == 60.0)
        );
        assert!(fishing.iter().any(
            |(c, _)| matches!(c, Command::PlaySound { sound, .. } if sound.ends_with("retrieve"))
        ));
    }

    #[test]
    fn the_sky_and_screen_of_batch_eleven_send_what_they_should() {
        let mut memory = rich_memory();
        memory.place = Some([30, 64, 30]);
        let now = Now::default();
        let mut rng = Rng::new(11);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            (pos, commands(event, pos, &memory, &mut ids(), &now, None))
        };
        let hint = |c: &Command| match c {
            Command::Hint {
                hint, pos, seconds, ..
            } => Some((hint.clone(), *pos, *seconds)),
            _ => None,
        };

        for (id, seconds) in [
            ("mist", 240.0),
            ("dusk_noon", 300.0),
            ("red_mist", 270.0),
            ("darkness", 200.0),
        ] {
            let (_, sent) = made(id, &mut rng);
            assert_eq!(
                hint(&sent[0].0),
                Some((id.to_string(), None, seconds)),
                "{id}"
            );
        }
        // The compass points at its place; the painting is the one nearest them.
        let (_, compass) = made("compass_points", &mut rng);
        assert_eq!(hint(&compass[0].0).unwrap().1, Some([30, 64, 30]));
        let (painting, flip) = made("painting_flip", &mut rng);
        assert!(painting.is_some());
        assert_eq!(hint(&flip[0].0).unwrap().1, painting);
        // Their name twice, six seconds.
        let (_, twice) = made("tab_twice", &mut rng);
        assert!(matches!(&twice[0].0, Command::PlayerList { seconds, .. } if *seconds == 6.0));
    }

    #[test]
    fn the_face_under_the_water_has_its_head_below_the_top() {
        let memory = rich_memory();
        let water = memory
            .feature("water")
            .expect("the rich memory has water")
            .pos;
        let event = find("reflection").unwrap();
        let p = place(event, &memory, &mut Rng::new(1)).expect("a spot under the water");
        // The figure stands one above its spot: feet two below the water's top, head under.
        assert_eq!(p[1], water[1] - 3);
    }

    #[test]
    fn the_village_of_batch_twelve_sends_what_it_should() {
        let mut memory = rich_memory();
        memory.held = Some(crate::memory::Held {
            item: stack("minecraft:iron_pickaxe", 1, 0),
            from: [2, 64, 0],
            slot: 4,
            taken_day: 0.5,
        });
        let now = Now {
            day_time: 18_000,
            ..Now::default()
        };
        let mut rng = Rng::new(12);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        // The village empties till dawn: the villagers in, every other door open and shut at
        // dawn, the lanterns out till then.
        let empty = made("village_abandoned", &mut rng);
        assert!(
            matches!(&empty[0].0, Command::GatherVillagers { seconds, .. } if *seconds == 300.0)
        );
        let opened = empty
            .iter()
            .filter(|(c, _)| matches!(c, Command::SetState { value, .. } if value == "true"))
            .count();
        let shut: Vec<u64> = empty
            .iter()
            .filter(|(c, _)| matches!(c, Command::SetState { value, .. } if value == "false"))
            .map(|(_, t)| *t)
            .collect();
        assert_eq!(opened, shut.len());
        assert!(shut.iter().all(|t| *t >= 6_000), "shut at dawn: {shut:?}");
        assert!(empty
            .iter()
            .any(|(c, _)| matches!(c, Command::HideBlock { seconds, .. } if *seconds == 300.0)));

        // What it took, for sale; the lantern on their way; the bell again.
        assert!(
            matches!(&made("villager_word", &mut rng)[0].0, Command::OfferTrade { kept, .. } if kept == "held")
        );
        let lantern = made("village_lantern", &mut rng);
        assert!(
            matches!(&lantern[1].0, Command::SetBlock { pos, only: true, .. } if *pos == [62, 67, 60])
        );
        assert!(
            matches!(&made("bell_twice", &mut rng)[0], (Command::ToggleBlock { pos, .. }, 30) if *pos == [60, 65, 60])
        );
    }

    #[test]
    fn a_rung_bell_is_noticed() {
        use craftkit::world::{Event, EventKind, Observations, Player};
        let mut memory = memory();
        memory.target = Some(0);
        memory.observe(
            &Observations {
                players: vec![Player::default()],
                events: vec![Event {
                    player: 0,
                    kind: EventKind::RangBell,
                    pos: Some([60, 65, 60]),
                    ..Event::default()
                }],
                ..Observations::default()
            },
            0.3,
        );

        assert_eq!(memory.rang_bell_now, Some([60, 65, 60]));
    }

    #[test]
    fn gone_into_another_dimension_and_back_is_noticed_and_leaving_the_game_is_not_that() {
        use craftkit::world::{Event, EventKind, Observations, Player};
        let here = Observations {
            players: vec![Player::default()],
            ..Observations::default()
        };
        let mut memory = memory();
        memory.target = Some(0);
        memory.observe(&here, 1.0);
        memory.observe(&Observations::default(), 1.2);
        assert_eq!(
            memory.elsewhere_since,
            Some(1.2),
            "gone, no word of leaving"
        );
        memory.observe(
            &Observations {
                players: vec![Player::default()],
                events: vec![Event {
                    player: 0,
                    kind: EventKind::ChangedDimension,
                    detail: "minecraft:overworld".to_string(),
                    ..Event::default()
                }],
                ..Observations::default()
            },
            1.5,
        );
        assert!((memory.back_after.unwrap() - 0.3).abs() < 1e-4);
        assert_eq!(memory.elsewhere_since, None);

        // Leaving the game is not another world.
        let mut left = self::memory();
        left.target = Some(0);
        left.observe(&here, 1.0);
        left.observe(
            &Observations {
                events: vec![Event {
                    player: 0,
                    kind: EventKind::Left,
                    ..Event::default()
                }],
                ..Observations::default()
            },
            1.2,
        );
        assert_eq!(left.elsewhere_since, None);
    }

    #[test]
    fn a_knock_on_a_door_is_noticed() {
        use craftkit::world::{Event, EventKind, Observations, Player};
        let mut memory = memory();
        memory.target = Some(0);
        let knock = r#"{"player":0,"kind":"knocked","pos":[3,64,0],"detail":"minecraft:oak_door"}"#;
        let event: Event = serde_json::from_str(knock).unwrap();
        assert_eq!(event.kind, EventKind::Knocked);
        memory.observe(
            &Observations {
                players: vec![Player::default()],
                events: vec![event],
                ..Observations::default()
            },
            0.3,
        );

        assert_eq!(memory.knocked_now, Some([3, 64, 0]));
    }

    #[test]
    fn tidying_a_chest_said_to_hold_two_stacks_in_one_slot_never_panics() {
        let contents = vec![
            stack("minecraft:water_bucket", 1, 9),
            stack("minecraft:lantern", 1, 9),
            stack("minecraft:stone", 3, 30),
            stack("minecraft:apple", 2, 1),
        ];
        let moves = tidy_moves(&contents, 27);
        assert!(
            moves.iter().all(|(from, to, _)| *from < 27 && *to < 27),
            "{moves:?}"
        );
        assert!(tidy_moves(&contents, 0).is_empty());
    }

    #[test]
    fn the_mines_of_batch_thirteen_send_what_they_should() {
        let mut memory = rich_memory();
        memory.last.pos = [5.5, 0.0, 0.5];
        memory.last.online = true;
        memory.last.underground = true;
        let now = Now::default();
        let mut rng = Rng::new(13);
        let made = |id: &str, memory: &Memory, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, memory, rng);
            commands(event, pos, memory, &mut ids(), &now, None)
        };
        let dug = |made: &Vec<(Command, u64)>| -> Vec<BlockPos> {
            made.iter()
                .filter_map(|(c, _)| match c {
                    Command::SetBlock { block, pos, .. } if block == "minecraft:air" => Some(*pos),
                    _ => None,
                })
                .collect()
        };

        // The copy runs three blocks beside theirs (it goes east, so three south), two high.
        let beside = made("mine_beside", &memory, &mut rng);
        let holes = dug(&beside);
        assert_eq!(holes.len(), 28);
        assert!(holes.iter().all(|p| p[2] == 3), "{holes:?}");
        assert!(beside.iter().any(
            |(c, _)| matches!(c, Command::SetBlock { block, .. } if block == "minecraft:torch")
        ));

        // Toward them: from twenty-five past the end, heard, to one or two short of it.
        let toward = made("tunnel_toward", &memory, &mut rng);
        let first = dug(&toward)[0];
        assert_eq!(first, [7 + 25, 0, 0]);
        assert!(toward
            .iter()
            .any(|(c, _)| matches!(c, Command::BlockSound { of, .. } if of == "break")));

        // The way back: gravel five columns behind them, floor and head.
        let closed = made("way_back_closes", &memory, &mut rng);
        assert!(closed.iter().all(
            |(c, _)| matches!(c, Command::SetBlock { block, .. } if block == "minecraft:gravel")
        ));
        assert!(matches!(&closed[0].0, Command::SetBlock { pos, .. } if *pos == [0, 0, 0]));

        // The far end's light out first; the ladders as deep as there are (eight).
        let lights = made("torches_far_end", &memory, &mut rng);
        assert!(matches!(&lights[0].0, Command::HideBlock { pos, .. } if *pos == [-4, 1, 0]));
        let shaft = made("ladder_down", &memory, &mut rng);
        assert_eq!(shaft.iter().filter(|(c, _)| matches!(c, Command::SetBlock { block, .. } if block == "minecraft:ladder")).count(), 8);
        assert_eq!(
            shaft
                .iter()
                .filter(|(c, _)| matches!(c, Command::TakeItem { count: 1, .. }))
                .count(),
            8,
            "a ladder taken for each one laid"
        );
        assert!(matches!(
            &made("portal_lit", &memory, &mut rng)[0].0,
            Command::LightPortal { .. }
        ));
    }

    #[test]
    fn the_figures_of_batch_fourteen_send_what_they_should() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 0.0; // facing +z
        let now = Now::default();
        let mut rng = Rng::new(14);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        // Right behind them (two blocks back, -z), then a strike that spares them.
        let nip = made("figure_nip", &mut rng);
        assert!(
            matches!(&nip[0].0, Command::Figure { pos, .. } if pos[2] == -1.5 && pos[1] == 64.0),
            "{nip:?}"
        );
        assert!(
            matches!(&nip[1], (Command::FigureSwing { damage, spare: true, .. }, 8) if *damage == 1.0)
        );
        assert!(
            matches!(&made("figure_swing", &mut rng)[1].0, Command::FigureSwing { damage, .. } if *damage == 4.0)
        );
        // The wave harms no one.
        assert!(
            matches!(&made("figure_waves", &mut rng)[1].0, Command::FigureSwing { damage, .. } if *damage == 0.0)
        );
        // The mirror mirrors; the crowd's goes once seen.
        assert!(matches!(
            &made("figure_mirror", &mut rng)[0].0,
            Command::Figure {
                mirror: true,
                once_seen: false,
                ..
            }
        ));
        assert!(matches!(
            &made("figure_crowd", &mut rng)[0].0,
            Command::Figure {
                once_seen: true,
                ..
            }
        ));
        // At the window: outside the pane, its head at the glass.
        let window = made("figure_window", &mut rng);
        assert!(
            matches!(&window[0].0, Command::Figure { pos, .. } if pos[1] == 64.0),
            "{window:?}"
        );
    }

    #[test]
    fn the_close_ones_of_batch_sixteen_send_what_they_should() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 1.0, 0.5];
        memory.last.yaw = 0.0; // facing +z
        memory.last.online = true;
        memory.target_name = "Player612".to_string();
        let now = Now::default();
        let mut rng = Rng::new(16);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        // Their farm animals near, together, stopping four blocks off; never the villager.
        let pack = made("pack", &mut rng);
        assert!(
            matches!(&pack[0].0, Command::Approach { entities, stop, .. }
                if entities.len() >= 3 && *stop == 4.0 && !entities.contains(&204)),
            "{pack:?}"
        );
        // Every monster near held, creeping.
        let close = made("hostiles_close", &mut rng);
        assert_eq!(close.len(), memory.last.hostiles_near.len());
        assert!(close
            .iter()
            .all(|(c, _)| matches!(c, Command::Hold { creep: true, .. })));
        // A push armed, three blocks at most.
        assert!(matches!(
            &made("shove", &mut rng)[0].0,
            Command::Shove { most_drop: 3, .. }
        ));
        // A step right behind them, then the view turned a little toward a point ahead.
        let hand = made("hand_on_shoulder", &mut rng);
        assert!(
            matches!(&hand[0].0, Command::BlockSound { pos, .. } if pos[2] < 0.0 && pos[2] > -0.5),
            "{hand:?}"
        );
        assert!(
            matches!(&hand[1].0, Command::Hint { hint, pos: Some(p), .. } if hint == "camera_nudge" && p[2] > 5),
            "{hand:?}"
        );
        // Their name, from behind them.
        let called = made("name_called", &mut rng);
        assert!(
            matches!(&called[0].0, Command::Hint { detail, pos: Some(p), .. } if detail == "Player612" && p[2] < 0),
            "{called:?}"
        );
        assert!(matches!(
            &made("footsteps_follow", &mut rng)[0].0,
            Command::Footsteps { behind, .. } if *behind == 3.0
        ));
    }

    #[test]
    fn a_frame_s_own_item_is_never_lost_emptied_onto_the_bed_or_traded_for_the_chest_s() {
        let mut memory = rich_memory();
        memory.last.online = true;
        let now = Now::default();
        let mut rng = Rng::new(18);
        let mut made = |id: &str| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, &mut rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };
        let bed = memory.home.as_ref().unwrap().bed.unwrap();
        let emptied = made("frame_emptied");
        assert_eq!(
            emptied.len(),
            1,
            "no stand-in made from nothing: {emptied:?}"
        );
        assert!(matches!(&emptied[0].0,
            Command::SetFrameItem { item, drop_at: Some(at), from: None, .. }
                if item.id.is_empty() && at[0] == bed[0] as f64 + 0.5 && at[2] == bed[2] as f64 + 0.5));
        let swapped = made("frame_swap");
        assert_eq!(
            swapped.len(),
            1,
            "nothing taken out of the chest first: {swapped:?}"
        );
        assert!(matches!(
            &swapped[0].0,
            Command::SetFrameItem {
                from: Some(_),
                swap: true,
                ..
            }
        ));
    }

    #[test]
    fn the_frames_the_sky_the_line_and_the_knock_send_what_they_should() {
        let mut memory = rich_memory();
        memory.last.online = true;
        memory.knocked_now = Some([3, 64, 0]);
        let now = Now::default();
        let mut rng = Rng::new(17);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        // Every frame near the home, the one thing they hold most.
        let frames = made("item_frames_face", &mut rng);
        let want: Vec<u32> = memory.decor("item_frame").map(|d| d.id).collect();
        assert!(
            matches!(&frames[0].0, Command::FramesShow { frames, item, .. }
                if *frames == want && Some(item) == memory.favourites.first().map(|(id, _)| id)),
            "{frames:?}"
        );
        // What it took, when it holds something, is what every frame shows.
        let mut holding = memory.clone();
        holding.held = Some(crate::memory::Held {
            item: stack("minecraft:clock", 1, 0),
            ..Default::default()
        });
        assert_eq!(frame_shows(&holding).as_deref(), Some("minecraft:clock"));
        assert!(matches!(
            &made("fish_on_line", &mut rng)[0].0,
            Command::Bobber { under, .. } if *under == 10.0
        ));
        assert!(matches!(
            &made("wrong_sky", &mut rng)[0].0,
            Command::Hint { hint, seconds, .. } if hint == "wrong_sky" && *seconds == 60.0
        ));
        // Knocks back from the door they knocked on, a moment after theirs.
        let knock = made("answer_knock", &mut rng);
        assert!(!knock.is_empty());
        assert!(
            knock.iter().all(|(c, t)| *t >= 20
                && matches!(c, Command::PlaySound { pos, .. } if *pos == [3.5, 64.5, 0.5])),
            "{knock:?}"
        );
    }

    #[test]
    fn arrows_and_potions_send_what_they_should() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 0.0; // facing +z: behind is -z
        memory.last.online = true;
        let now = Now::default();
        let mut rng = Rng::new(19);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        // The figure with a bow behind them, then the arrow: past their head, harmless.
        let near = made("arrow_near", &mut rng);
        assert!(matches!(&near[0].0, Command::Figure { held, .. } if held.id == "minecraft:bow"));
        let (from, at, damage) = match &near[1].0 {
            Command::Shoot {
                from, at, damage, ..
            } => (*from, *at, *damage),
            other => panic!("{other:?}"),
        };
        assert_eq!(damage, 0.0);
        assert!(
            from[2] < -10.0 && from[1] <= -1_000.0,
            "behind, on the ground: {from:?}"
        );
        assert!(
            (at[0] - 0.5).abs() > 1.0 && at[1] > 65.5,
            "beside the head: {at:?}"
        );
        assert!(near[1].1 > near[0].1, "the figure first");
        // Grazes at four, hits at five; at them.
        assert!(matches!(&made("arrow_grazes", &mut rng)[1].0,
            Command::Shoot { damage, at, .. } if *damage == 2.0 && (at[0] - 0.5).abs() < 0.01));
        assert!(
            matches!(&made("arrow_hits", &mut rng)[1].0, Command::Shoot { damage, .. } if *damage == 4.0)
        );

        assert!(matches!(&made("splash_dark", &mut rng)[0].0,
            Command::SplashPotion { effect, seconds, .. }
                if ["darkness", "slowness", "blindness", "levitation", "nausea"].contains(&effect.as_str())
                    && (effect != "levitation" || *seconds <= 2.0)));
        assert!(matches!(&made("brew_left", &mut rng)[0].0,
            Command::PutPotion { name, effects, .. } if name == "for the night" && !effects.is_empty()));
        assert!(matches!(
            &made("potion_changed", &mut rng)[0].0,
            Command::ChangePotion { slot: 20, .. }
        ));
        // The cloud on the step outside the door they use, a block out from the house.
        assert!(matches!(&made("door_cloud", &mut rng)[0].0,
            Command::EffectCloud { pos, .. } if pos[0] == 4.5));
    }

    #[test]
    fn one_animal_comes_at_them_slowly_or_at_a_run_and_then_stares_leaves_or_strikes() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.online = true;
        let now = Now::default();
        let mut thens = std::collections::BTreeSet::new();
        for seed in 0..12u64 {
            let mut rng = Rng::new(seed);
            for (id, want) in [("slow_approach", 0.35), ("quick_approach", 1.5)] {
                let event = find(id).unwrap();
                let pos = place(event, &memory, &mut rng);
                memory.done.push(crate::memory::Done::default());
                let made = commands(event, pos, &memory, &mut ids(), &now, None);
                match &made[0].0 {
                    Command::Approach {
                        entities,
                        pace,
                        then,
                        damage,
                        stop,
                        ..
                    } => {
                        assert_eq!(entities.len(), 1);
                        assert_eq!(*pace, Some(want));
                        assert!(*stop < 2.0, "right in front of them");
                        assert_eq!(*damage > 0.0, then == "hit");
                        thens.insert(then.clone());
                    }
                    other => panic!("{other:?}"),
                }
            }
        }
        assert_eq!(thens.len(), 3, "it stares, leaves and strikes: {thens:?}");
    }

    #[test]
    fn like_a_player_sends_what_it_should() {
        let mut memory = rich_memory();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.last.yaw = 90.0; // facing -x
        memory.last.online = true;
        memory.home.as_mut().unwrap().lights = vec![[3, 65, 0]];
        let now = Now::default();
        let mut rng = Rng::new(15);
        let made = |id: &str, rng: &mut Rng| {
            let event = find(id).unwrap();
            let pos = place(event, &memory, rng);
            commands(event, pos, &memory, &mut ids(), &now, None)
        };

        // Thirty blocks the way they face, on the ground, toward the rock.
        assert!(matches!(&made("into_the_mountain", &mut rng)[0].0,
            Command::DigIn { from, way, through: false, .. } if *way == [-1, 0] && from[0] == -30 && from[1] <= -1_000));
        assert!(matches!(
            &made("through_the_hill", &mut rng)[0].0,
            Command::DigIn { through: true, .. }
        ));
        // The far figure copying their crouch; the one at work.
        assert!(matches!(
            &made("copy_crouch", &mut rng)[0].0,
            Command::Figure {
                copy_crouch: true,
                swinging: false,
                ..
            }
        ));
        assert!(matches!(
            &made("tools_in_hand", &mut rng)[0].0,
            Command::Figure { swinging: true, .. }
        ));
        // At the edge of their light, a step back, and the flicker.
        let dark = made("into_the_dark", &mut rng);
        assert!(matches!(&dark[0].0, Command::Figure { path, .. } if path.len() == 2));
        assert!(matches!(&dark[1].0, Command::Hint { hint, .. } if hint == "flicker"));
    }
}
