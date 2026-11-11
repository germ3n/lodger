//! What it builds (batch 15, owner): its house near theirs, a pillar, a hut on
//! their route, a room in a cave, a dock, a bridge, an outline and a copy of their house. Each
//! is a blueprint, blocks relative to a site (`dy` 0 the ground's own block, 1 the first above),
//! laid a night's share at a time; the memory keeps how far each got and the ground's height
//! once the host has found it. Materials: what it gathered first, the rest made up when short
//! (owner: "gathered, topped up when short").

use craftkit::world::BlockPos;
use serde::{Deserialize, Serialize};

use crate::memory::{distance, Memory};

/// One block of a blueprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    pub at: [i32; 3],
    pub block: &'static str,
    pub state: String,
}

fn part(x: i32, y: i32, z: i32, block: &'static str) -> Part {
    Part {
        at: [x, y, z],
        block,
        state: String::new(),
    }
}

fn with(x: i32, y: i32, z: i32, block: &'static str, state: &str) -> Part {
    Part {
        at: [x, y, z],
        block,
        state: state.to_string(),
    }
}

/// A side as the game names a facing: `north` is -z.
pub fn facing_name(way: [i32; 2]) -> &'static str {
    match way {
        [1, _] => "east",
        [-1, _] => "west",
        [_, 1] => "south",
        _ => "north",
    }
}

/// A build in progress or done: what, where, which way its door faces, how much is laid.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Build {
    /// `house_near`, `pillar`, `hut_on_route`, `cave_room`, `dock`, `bridge`, `copy_build`,
    /// `copy_house`.
    pub kind: String,
    /// The site: x and z, and y once known (the ground's own block), or below -1000 for the
    /// host to find it.
    pub site: BlockPos,
    /// The way its door or its length faces, `[dx, dz]`.
    pub way: [i32; 2],
    /// How many parts of its blueprint are laid.
    pub laid: usize,
    /// The house's second room, added at stage 5.
    pub grown: bool,
    pub last_day: f32,
}

impl Build {
    pub fn ground_known(&self) -> bool {
        self.site[1] > -1_000
    }
}

/// Its own house: five by five, cobblestone walls three high on a plank floor, a log roof, the
/// door facing `way` (toward theirs), a pane in the wall across, a torch and a chest inside.
/// Laid floor and the lower walls first, then the rest of the walls, the door and the pane,
/// then the roof, the torch and the chest. With `grown`, a second room of five by four behind
/// it, through a gap in the back wall.
pub fn house(way: [i32; 2], grown: bool) -> Vec<Part> {
    // Built facing south (+z) and turned to `way`.
    let mut parts = Vec::new();
    for x in -2..=2 {
        for z in -2..=2 {
            parts.push(part(x, 0, z, "minecraft:oak_planks"));
        }
    }
    let wall = |x: i32, z: i32| x.abs() == 2 || z.abs() == 2;
    for y in 1..=3 {
        for x in -2..=2 {
            for z in -2..=2 {
                let door = x == 0 && z == 2 && y <= 2;
                let pane = x == 0 && z == -2 && y == 2;
                if wall(x, z) && !door && !pane {
                    parts.push(part(x, y, z, "minecraft:cobblestone"));
                }
            }
        }
    }
    parts.push(with(
        0,
        1,
        2,
        "minecraft:oak_door",
        "facing=south,half=lower",
    ));
    parts.push(with(
        0,
        2,
        2,
        "minecraft:oak_door",
        "facing=south,half=upper",
    ));
    parts.push(part(0, 2, -2, "minecraft:glass_pane"));
    for x in -2..=2 {
        for z in -2..=2 {
            parts.push(part(x, 4, z, "minecraft:oak_log"));
        }
    }
    parts.push(with(1, 2, -1, "minecraft:wall_torch", "facing=south"));
    parts.push(with(-1, 1, -1, "minecraft:chest", "facing=south"));
    if grown {
        // The second room behind, sharing the back wall, a gap in it where the pane was.
        for x in -2i32..=2 {
            for z in -6i32..=-3 {
                parts.push(part(x, 0, z, "minecraft:oak_planks"));
                for y in 1..=3 {
                    if x.abs() == 2 || z == -6 {
                        parts.push(part(x, y, z, "minecraft:cobblestone"));
                    }
                }
                parts.push(part(x, 4, z, "minecraft:oak_log"));
            }
        }
        // The doorway between the rooms: the comment said so, and nothing made it, so the
        // second room was sealed.
        parts.push(part(0, 1, -2, "minecraft:air"));
        parts.push(part(0, 2, -2, "minecraft:air"));
    }

    turned(parts, way)
}

/// A spot of its house, by its place in the house built facing south, turned to `way`.
pub fn house_spot(way: [i32; 2], at: [i32; 3]) -> [i32; 3] {
    turned(vec![part(at[0], at[1], at[2], "minecraft:air")], way)[0].at
}

/// Where its house's wall torch hangs, and the three spots on its floor its lights stand on
/// (`its_lights_on`), from its site, the house built facing `way`.
pub fn house_lights(site: BlockPos, way: [i32; 2]) -> (BlockPos, Vec<BlockPos>) {
    let at = |p: [i32; 3]| {
        let t = house_spot(way, p);
        [site[0] + t[0], site[1] + t[1], site[2] + t[2]]
    };
    // Clear of the bed `respawn_its_bed` lays along the left wall.
    (
        at([1, 2, -1]),
        vec![at([1, 1, 1]), at([0, 1, 0]), at([1, 1, -1])],
    )
}

/// The bed `respawn_its_bed` lays in its house, along its left wall, the head toward the back,
/// as blueprint parts from its site, and where the head stands.
pub fn house_bed(site: BlockPos, way: [i32; 2]) -> (Vec<Part>, BlockPos) {
    let parts = turned(
        vec![
            with(-1, 1, 1, "minecraft:red_bed", "facing=north,part=foot"),
            with(-1, 1, 0, "minecraft:red_bed", "facing=north,part=head"),
        ],
        way,
    );
    let head = parts[1].at;
    (
        parts,
        [site[0] + head[0], site[1] + head[1], site[2] + head[2]],
    )
}

impl crate::memory::Memory {
    /// Its house near theirs, built whole, its ground known.
    pub fn its_house(&self) -> Option<&Build> {
        self.builds.iter().find(|b| {
            b.kind == "house_near" && b.ground_known() && b.laid >= house(b.way, false).len()
        })
    }
}

/// A pillar of cobblestone, `high` blocks.
pub fn pillar(high: i32) -> Vec<Part> {
    (1..=high.max(1))
        .map(|y| part(0, y, 0, "minecraft:cobblestone"))
        .collect()
}

/// A three by three shelter of logs, two high, a log roof, an opening facing `way`, a torch.
pub fn hut(way: [i32; 2]) -> Vec<Part> {
    let mut parts = Vec::new();
    for y in 1..=2 {
        for x in -1i32..=1 {
            for z in -1i32..=1 {
                let edge = x.abs() == 1 || z.abs() == 1;
                let opening = x == 0 && z == 1;
                if edge && !opening {
                    parts.push(part(x, y, z, "minecraft:oak_log"));
                }
            }
        }
    }
    for x in -1..=1 {
        for z in -1..=1 {
            parts.push(part(x, 3, z, "minecraft:oak_log"));
        }
    }
    parts.push(part(0, 1, 0, "minecraft:torch"));

    turned(parts, way)
}

/// A dock of planks out over the water from the shore, `long` blocks along `way`.
pub fn dock(way: [i32; 2], long: i32) -> Vec<Part> {
    (1..=long.max(1))
        .map(|k| part(way[0] * k, 0, way[1] * k, "minecraft:oak_planks"))
        .collect()
}

/// The outline of their house, one block high, from its walls' footprint, at the site.
pub fn outline(walls: &[BlockPos]) -> Vec<Part> {
    let (Some(min_x), Some(max_x), Some(min_z), Some(max_z)) = (
        walls.iter().map(|w| w[0]).min(),
        walls.iter().map(|w| w[0]).max(),
        walls.iter().map(|w| w[2]).min(),
        walls.iter().map(|w| w[2]).max(),
    ) else {
        return Vec::new();
    };
    let (cx, cz) = ((min_x + max_x) / 2, (min_z + max_z) / 2);
    let mut parts = Vec::new();
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            if x == min_x || x == max_x || z == min_z || z == max_z {
                parts.push(part(x - cx, 1, z - cz, "minecraft:cobblestone"));
            }
        }
    }

    parts
}

/// A copy of their house from what they built with: stone as cobblestone, wood as logs, glass
/// as glass, relative to its middle and floor.
pub fn copy_of(walls: &[(BlockPos, String)], center: BlockPos, floor: i32) -> Vec<Part> {
    walls
        .iter()
        .filter(|(p, _)| distance(*p, center) <= 12.0)
        .map(|(p, id)| {
            let block = if id.contains("glass") {
                "minecraft:glass"
            } else if id.contains("log") || id.contains("planks") || id.contains("wood") {
                "minecraft:oak_log"
            } else {
                "minecraft:cobblestone"
            };
            part(p[0] - center[0], p[1] - floor + 1, p[2] - center[2], block)
        })
        .collect()
}

/// A build's share as it is sent: counted laid, the store spent for each block it has (the rest
/// is made up), and the build begun with its site and way if it is the first share.
pub fn account(memory: &mut Memory, event: &str, command: &craftkit::world::Command, day: f32) {
    let craftkit::world::Command::PlaceBlueprint { at, parts, .. } = command else {
        return;
    };
    let Some(found) =
        crate::events::find(event).filter(|e| matches!(e.action, crate::events::Action::Build(_)))
    else {
        return;
    };
    let kind = build_of(found.id).to_string();
    let way = memory
        .next_build(found.id)
        .map(|(_, way, _)| way)
        .unwrap_or([0, 1]);
    for p in parts {
        if memory.stored(&p.block) > 0 {
            memory.store(&p.block, -1);
        }
    }
    match memory.builds.iter_mut().find(|b| b.kind == kind) {
        Some(build) => {
            build.laid += parts.len();
            build.last_day = day;
            if found.id == "house_grows" {
                build.grown = true;
            }
        }
        None => memory.builds.push(Build {
            kind,
            site: *at,
            way,
            laid: parts.len(),
            grown: false,
            last_day: day,
        }),
    }
}

/// One block's way from one place toward another, along the longer axis.
fn step_way(from: BlockPos, to: BlockPos) -> [i32; 2] {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    if dx.abs() >= dz.abs() {
        [if dx >= 0 { 1 } else { -1 }, 0]
    } else {
        [0, if dz >= 0 { 1 } else { -1 }]
    }
}

/// A blueprint drawn facing south, turned to face `way`; door and torch facings turned too.
fn turned(parts: Vec<Part>, way: [i32; 2]) -> Vec<Part> {
    let turn = |x: i32, z: i32| -> (i32, i32) {
        match way {
            [0, 1] | [0, 0] => (x, z),
            [0, -1] => (-x, -z),
            [1, 0] => (z, -x),
            _ => (-z, x),
        }
    };
    let facing = |f: &str| -> &'static str {
        let (dx, dz) = match f {
            "south" => turn(0, 1),
            "north" => turn(0, -1),
            "east" => turn(1, 0),
            _ => turn(-1, 0),
        };
        facing_name([dx, dz])
    };
    parts
        .into_iter()
        .map(|p| {
            let (x, z) = turn(p.at[0], p.at[2]);
            let state = p
                .state
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|pair| match pair.split_once('=') {
                    Some(("facing", f)) => format!("facing={}", facing(f)),
                    _ => pair.to_string(),
                })
                .collect::<Vec<_>>()
                .join(",");
            Part {
                at: [x, p.at[1], z],
                block: p.block,
                state,
            }
        })
        .collect()
}

/// The build an event lays: the house's growing is the house's, the bridge's end the bridge's.
pub fn build_of(event: &str) -> &str {
    match event {
        "house_grows" => "house_near",
        "bridge_done" => "bridge",
        other => other,
    }
}

/// What a blueprint may lay over besides air: what lies on the ground and the ground's own top
/// (a floor goes in place of the grass), and for a dock or a bridge the water.
pub fn replaceable(event: &str) -> Vec<String> {
    let mut over: Vec<&str> = vec![
        "minecraft:short_grass",
        "minecraft:tall_grass",
        "minecraft:fern",
        "minecraft:snow",
        "minecraft:dandelion",
        "minecraft:poppy",
        "minecraft:grass_block",
        "minecraft:dirt",
        "minecraft:sand",
        "minecraft:gravel",
    ];
    if matches!(event, "dock" | "bridge" | "bridge_done") {
        over.push("minecraft:water");
    }
    // The grown room opens a doorway in its own back wall, where the pane was.
    if event == "house_grows" {
        over.push("minecraft:cobblestone");
        over.push("minecraft:glass_pane");
    }

    over.into_iter().map(String::from).collect()
}

/// The next share of a build, `(site, way, parts)`: the parts still to lay tonight. None when
/// it has nothing left, or nothing to build on.
pub type Share = (BlockPos, [i32; 2], Vec<Part>);

fn share(all: Vec<Part>, laid: usize, per_night: usize, stop: usize) -> Vec<Part> {
    // "All at once" is usize::MAX a night: added to what is laid it must not wrap (a stay's
    // test found it overflowing).
    let end = laid
        .saturating_add(per_night.max(1))
        .min(stop.min(all.len()));
    all.into_iter()
        .skip(laid)
        .take(end.saturating_sub(laid))
        .collect()
}

impl Memory {
    /// The next share an event lays of its build.
    pub fn next_build(&self, event: &str) -> Option<Share> {
        let kind = build_of(event);
        let had = self.build(kind);
        let laid = had.map(|b| b.laid).unwrap_or(0);
        let home = self.home.as_ref()?;
        let place = self
            .place
            .unwrap_or([home.center[0] + 10, -10_000, home.center[2] + 10]);
        let place = had.map(|b| b.site).unwrap_or([place[0], -10_000, place[2]]);
        let (site, way, all, per_night, stop) = match event {
            "house_near" => {
                let (site, way) = self.house_site()?;
                let all = house(way, false);
                let third = all.len().div_ceil(3);
                let stop = all.len();
                (site, way, all, third, stop)
            }
            "house_grows" => {
                let built = had?;
                if built.laid < house(built.way, false).len() {
                    return None;
                }
                let all = house(built.way, true);
                let stop = all.len();
                (built.site, built.way, all, stop, stop)
            }
            "pillar" => {
                let high = if place[1] > -1_000 {
                    (home.floor_y + 5 - place[1]).clamp(2, 8)
                } else {
                    8
                };
                (place, [0, 1], pillar(high), 1, usize::MAX)
            }
            "hut_on_route" => {
                let points = &self.route()?.points;
                if points.len() < 2 {
                    return None;
                }
                let middle = points[points.len() / 2];
                let end = points[points.len() - 1];
                let way = step_way(middle, end);
                let site = had
                    .map(|b| b.site)
                    .unwrap_or([middle[0], -10_000, middle[2]]);
                (site, way, hut(way), usize::MAX, usize::MAX)
            }
            "dock" => {
                let shore = self.last_fished?;
                let water = self.feature("water")?.pos;
                let way = step_way(shore, water);
                let site = had.map(|b| b.site).unwrap_or([shore[0], -10_000, shore[2]]);
                (site, way, dock(way, 8), 1, usize::MAX)
            }
            "bridge" | "bridge_done" => {
                let water = self.feature("water")?;
                let toward = step_way(water.pos, home.center);
                let half = water.size as i32 / 2 + 1;
                let far = [
                    water.pos[0] - toward[0] * half,
                    -10_000,
                    water.pos[2] - toward[1] * half,
                ];
                let site = had.map(|b| b.site).unwrap_or(far);
                let long = water.size as i32 + 2;
                let all = dock(toward, long);
                // Stage 4 leaves the last two blocks; stage 5 finishes it.
                let stop = if event == "bridge" {
                    all.len().saturating_sub(2)
                } else {
                    all.len()
                };
                (site, toward, all, 3, stop)
            }
            "copy_build" => {
                let walls: Vec<BlockPos> = self.walls.iter().map(|(p, _)| *p).collect();
                (place, [0, 1], outline(&walls), usize::MAX, usize::MAX)
            }
            "copy_house" => {
                let all = copy_of(&self.walls, home.center, home.floor_y);
                (place, [0, 1], all, 40, usize::MAX)
            }
            _ => return None,
        };
        let parts = share(all, laid, per_night, stop);

        (!parts.is_empty()).then_some((site, way, parts))
    }

    pub fn build(&self, kind: &str) -> Option<&Build> {
        self.builds.iter().find(|b| b.kind == kind)
    }

    /// Where its own house goes: forty to eighty blocks from theirs, on the side away from
    /// their route's far end, its door facing their house.
    pub fn house_site(&self) -> Option<(BlockPos, [i32; 2])> {
        if let Some(b) = self.build("house_near") {
            return Some((b.site, b.way));
        }
        let home = self.home.as_ref()?.center;
        let away = self
            .route_end()
            .map(|e| [home[0] - e[0], home[2] - e[2]])
            .unwrap_or([1, 0]);
        let length = ((away[0] * away[0] + away[1] * away[1]) as f64)
            .sqrt()
            .max(1.0);
        let (ux, uz) = (away[0] as f64 / length, away[1] as f64 / length);
        let site = [
            home[0] + (ux * 56.0).round() as i32,
            -10_000,
            home[2] + (uz * 56.0).round() as i32,
        ];
        // The door faces back toward their house.
        let way = if ux.abs() >= uz.abs() {
            [-(ux.signum() as i32), 0]
        } else {
            [0, -(uz.signum() as i32)]
        };

        Some((site, way))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_house_is_five_by_five_with_a_door_a_pane_a_torch_and_a_chest() {
        let parts = house([0, 1], false);
        let at = |x: i32, y: i32, z: i32| parts.iter().find(|p| p.at == [x, y, z]);
        assert_eq!(at(0, 1, 2).unwrap().block, "minecraft:oak_door");
        assert_eq!(at(0, 1, 2).unwrap().state, "facing=south,half=lower");
        assert_eq!(at(0, 2, -2).unwrap().block, "minecraft:glass_pane");
        assert_eq!(at(2, 3, 2).unwrap().block, "minecraft:cobblestone");
        assert!(at(0, 1, 0).is_none(), "the inside is empty");
        assert!(parts.iter().any(|p| p.block == "minecraft:chest"));
        let grown = house([0, 1], true);
        assert!(grown.len() > parts.len());
        assert!(
            grown.iter().any(|p| p.at[2] == -6),
            "the second room behind"
        );
        let doorway: Vec<&str> = grown
            .iter()
            .filter(|p| p.at == [0, 1, -2] || p.at == [0, 2, -2])
            .map(|p| p.block)
            .collect();
        assert_eq!(
            doorway.last().copied(),
            Some("minecraft:air"),
            "a doorway into it, laid after the wall"
        );
    }

    #[test]
    fn a_house_turned_east_has_its_door_east_and_facing_east() {
        let parts = house([1, 0], false);
        let door = parts
            .iter()
            .find(|p| p.block == "minecraft:oak_door" && p.state.contains("lower"))
            .unwrap();
        assert_eq!(door.at, [2, 1, 0]);
        assert!(door.state.starts_with("facing=east"), "{}", door.state);
    }

    #[test]
    fn the_hut_has_an_opening_and_the_dock_runs_out() {
        let hut = hut([0, 1]);
        assert!(!hut.iter().any(|p| p.at == [0, 1, 1]), "the way in");
        assert!(hut.iter().any(|p| p.block == "minecraft:torch"));
        let dock = dock([-1, 0], 4);
        assert_eq!(dock.last().unwrap().at, [-4, 0, 0]);
        assert_eq!(pillar(3).len(), 3);
    }

    #[test]
    fn the_outline_and_the_copy_follow_their_walls() {
        let walls: Vec<BlockPos> = (0..6)
            .flat_map(|i| [[10 + i, 64, 20], [10, 64, 20 + i]])
            .collect();
        let outline = outline(&walls);
        assert!(outline.iter().all(|p| p.at[1] == 1));
        assert!(outline.iter().any(|p| p.at == [-2, 1, -2]));
        assert!(outline.iter().any(|p| p.at == [3, 1, 3]));
        assert!(
            !outline.iter().any(|p| p.at == [0, 1, 0]),
            "an outline, not a floor"
        );
        let copy = copy_of(
            &[
                ([12, 65, 20], "minecraft:oak_planks".to_string()),
                ([13, 65, 20], "minecraft:glass".to_string()),
            ],
            [12, 64, 22],
            64,
        );
        assert_eq!(copy[0].block, "minecraft:oak_log");
        assert_eq!(copy[0].at, [0, 2, -2]);
        assert_eq!(copy[1].block, "minecraft:glass");
    }

    fn with_home() -> Memory {
        Memory {
            home: Some(crate::memory::Home {
                center: [0, 64, 0],
                floor_y: 64,
                ..crate::memory::Home::default()
            }),
            ..Memory::default()
        }
    }

    fn send(memory: &mut Memory, event: &str) -> Vec<Part> {
        let (site, _, parts) = memory.next_build(event).expect("a share");
        let command = craftkit::world::Command::PlaceBlueprint {
            id: 1,
            at: site,
            parts: parts
                .iter()
                .map(|p| craftkit::world::BlueprintPart {
                    at: p.at,
                    block: p.block.to_string(),
                    state: p.state.clone(),
                })
                .collect(),
            replace: vec![],
        };
        account(memory, event, &command, 1.0);
        parts
    }

    #[test]
    fn its_house_goes_up_a_third_a_night_from_its_store_then_grows() {
        let mut memory = with_home();
        memory.store("minecraft:cobblestone", 10);
        let total = house([0, 1], false).len();

        let first = send(&mut memory, "house_near");
        assert_eq!(first.len(), total.div_ceil(3));
        let stone = first
            .iter()
            .filter(|p| p.block == "minecraft:cobblestone")
            .count() as u32;
        assert_eq!(
            memory.stored("minecraft:cobblestone"),
            10 - stone,
            "its own stone first, a stone a block"
        );
        assert!(
            memory.next_build("house_grows").is_none(),
            "not before it stands"
        );
        // The host found the ground: kept for the rest.
        memory.builds[0].site[1] = 63;
        send(&mut memory, "house_near");
        send(&mut memory, "house_near");
        assert_eq!(memory.builds[0].laid, total);
        assert!(memory.next_build("house_near").is_none(), "it stands");
        assert_eq!(memory.build("house_near").unwrap().site[1], 63);

        let room = send(&mut memory, "house_grows");
        assert_eq!(room.len(), house([0, 1], true).len() - total);
        assert!(memory.build("house_near").unwrap().grown);
    }

    #[test]
    fn a_pillar_rises_a_block_a_night_to_their_roof() {
        let mut memory = with_home();
        memory.place = Some([10, 64, 10]);
        assert_eq!(send(&mut memory, "pillar").len(), 1);
        memory.builds[0].site[1] = 63;
        let mut nights = 1;
        while memory.next_build("pillar").is_some() {
            send(&mut memory, "pillar");
            nights += 1;
        }
        assert_eq!(
            nights, 6,
            "up to five above their floor from the ground at 63"
        );
    }

    #[test]
    fn a_build_laid_at_once_does_not_overflow_what_is_laid() {
        let parts = hut([0, 1]);
        let n = parts.len();
        assert_eq!(share(parts.clone(), 0, usize::MAX, usize::MAX).len(), n);
        assert!(share(parts, n, usize::MAX, usize::MAX).is_empty());
    }
}
