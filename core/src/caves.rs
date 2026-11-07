//! Caves near the home the target has not entered (batch 4, owner): read by the
//! brain's cave read, a sparse scan for dark air (no sky light, no block light) around the
//! target while they are near home, since 1.18's caves are mostly plain air and not the old
//! `cave_air`. `lantern_ahead` puts one of their lanterns far down one; `cave_lit` lights one
//! at the spacing the target lights their own tunnels with.

use craftkit::world::BlockPos;

use crate::memory::{distance, Memory, CELL, HOME_RADIUS};

/// The cave read's half width and depth below the target, and its step.
pub const READ_WIDE: i32 = 48;
pub const READ_BELOW: i32 = 36;
pub const READ_ABOVE: i32 = 4;
pub const READ_STEP: u8 = 3;
/// The most dark places a read keeps.
pub const MOST_CAVE: usize = 512;
/// Dark places within this many blocks of one make a cave of it, not a pocket.
const CAVE_NEIGHBOURS: usize = 6;
/// Within this many blocks of where they walked, stood or built, they have been there.
const ENTERED: f64 = 10.0;
/// The most torches `cave_lit` places.
const MOST_TORCHES: usize = 6;

impl Memory {
    /// Whether the target has been at a place: walked by it, stood near it, built by it, or it
    /// is at home or one of their sites.
    fn entered(&self, pos: BlockPos) -> bool {
        let near_home = self
            .home
            .as_ref()
            .is_some_and(|h| distance(h.center, pos) <= HOME_RADIUS as f64);
        let walked = self
            .trail
            .iter()
            .chain(self.routes.iter().flat_map(|r| r.points.iter()))
            .any(|p| distance(*p, pos) <= ENTERED);
        let stood = self.still_cells.iter().any(|(cell, _)| {
            let middle = [
                cell[0] * CELL + CELL / 2,
                cell[1] * CELL,
                cell[2] * CELL + CELL / 2,
            ];
            distance(middle, pos) <= ENTERED
        });
        let built = self.placed.iter().any(|p| distance(*p, pos) <= 6.0);
        let site = self.sites.iter().any(|s| distance(s.pos, pos) <= 16.0);

        near_home || walked || stood || built || site
    }

    /// The cave they have not entered, as the dark places of it: the darkest crowd of the
    /// last read away from everywhere they have been, and every dark place within 16 blocks of
    /// its middle. None while no read found one.
    pub fn unentered_cave(&self) -> Option<Vec<BlockPos>> {
        let open: Vec<BlockPos> = self
            .caves
            .iter()
            .copied()
            .filter(|p| !self.entered(*p))
            .collect();
        let crowd = |p: &BlockPos| open.iter().filter(|o| distance(**o, *p) <= 8.0).count();
        let middle = open
            .iter()
            .filter(|p| crowd(p) >= CAVE_NEIGHBOURS)
            .max_by_key(|p| crowd(p))
            .copied()?;

        Some(
            open.into_iter()
                .filter(|p| distance(*p, middle) <= 16.0)
                .collect(),
        )
    }

    /// The spacing they light their own tunnels at: the middle of each underground light's
    /// distance to its nearest, from four blocks to fourteen; eight before two lights.
    pub fn torch_spacing(&self) -> f64 {
        let lights = &self.habits.cave_lights;
        if lights.len() < 2 {
            return 8.0;
        }
        let mut gaps: Vec<f64> = lights
            .iter()
            .enumerate()
            .filter_map(|(i, a)| {
                lights
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, b)| distance(*a, *b))
                    .min_by(f64::total_cmp)
            })
            .collect();
        gaps.sort_by(f64::total_cmp);

        gaps[gaps.len() / 2].clamp(4.0, 14.0)
    }

    /// Where `lantern_ahead` puts the lantern: the cave's dark place farthest from the target.
    pub fn lantern_spot(&self) -> Option<BlockPos> {
        let at = crate::memory::block_of(self.last.pos);
        self.unentered_cave()?
            .into_iter()
            .max_by(|a, b| distance(*a, at).total_cmp(&distance(*b, at)))
    }

    /// Where `cave_lit` puts its torches: from the cave's end nearest the target inward, one
    /// every `torch_spacing` blocks, at most six.
    pub fn cave_torches(&self) -> Vec<BlockPos> {
        let Some(mut cave) = self.unentered_cave() else {
            return Vec::new();
        };
        let at = crate::memory::block_of(self.last.pos);
        cave.sort_by(|a, b| distance(*a, at).total_cmp(&distance(*b, at)));
        let spacing = self.torch_spacing();
        let mut chosen: Vec<BlockPos> = Vec::new();
        for p in cave {
            if chosen.iter().all(|c| distance(*c, p) >= spacing) {
                chosen.push(p);
                if chosen.len() == MOST_TORCHES {
                    break;
                }
            }
        }

        chosen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Home, Route};

    /// A tunnel of dark places along x from `from` to `to`, two high and two wide, as a read
    /// every third block finds one.
    fn tunnel(from: i32, to: i32, z: i32) -> Vec<BlockPos> {
        (from..=to)
            .step_by(3)
            .flat_map(|x| [[x, 30, z], [x, 33, z], [x, 30, z + 3], [x, 33, z + 3]])
            .collect()
    }

    fn memory() -> Memory {
        Memory {
            home: Some(Home {
                center: [0, 64, 0],
                floor_y: 64,
                ..Home::default()
            }),
            ..Memory::default()
        }
    }

    #[test]
    fn a_cave_they_never_went_near_is_found_and_one_they_walked_is_not() {
        let mut memory = memory();
        memory.caves = tunnel(20, 50, 20);
        memory.caves.extend(tunnel(-50, -20, -20));
        memory.routes.push(Route {
            points: vec![[-35, 30, -20], [-30, 30, -20]],
            walked: 2,
            first_day: 0.0,
        });

        let cave = memory.unentered_cave().unwrap();
        assert!(
            cave.iter().all(|p| p[0] > 0),
            "the walked one is entered: {cave:?}"
        );
        assert!(cave.len() >= CAVE_NEIGHBOURS);
    }

    #[test]
    fn a_pocket_is_no_cave_and_home_is_entered() {
        let mut memory = memory();
        memory.caves = vec![[30, 30, 30], [31, 30, 30], [3, 60, 3], [4, 60, 3]];
        // A cellar-deep hollow just under the house's floor is home.
        memory.caves.extend(
            tunnel(-9, 9, 0)
                .into_iter()
                .map(|p| [p[0], p[1] + 28, p[2]]),
        );

        assert_eq!(memory.unentered_cave(), None);
    }

    #[test]
    fn the_spacing_is_theirs_and_eight_before_they_light_a_tunnel() {
        let mut memory = memory();
        assert_eq!(memory.torch_spacing(), 8.0);
        memory.habits.cave_lights = vec![[0, 20, 0], [6, 20, 0], [12, 20, 0], [18, 20, 0]];
        assert_eq!(memory.torch_spacing(), 6.0);
        memory.habits.cave_lights = vec![[0, 20, 0], [1, 20, 0]];
        assert_eq!(memory.torch_spacing(), 4.0, "never closer than four");
    }

    #[test]
    fn the_lantern_goes_far_down_and_the_torches_go_in_from_the_near_end() {
        let mut memory = memory();
        memory.caves = tunnel(20, 50, 20);
        memory.last.pos = [10.0, 64.0, 20.0];
        memory.habits.cave_lights = vec![[0, 20, 0], [6, 20, 0], [12, 20, 0]];

        let lantern = memory.lantern_spot().unwrap();
        let torches = memory.cave_torches();
        assert!(lantern[0] >= 44, "far down: {lantern:?}");
        let at = crate::memory::block_of(memory.last.pos);
        let nearest = memory
            .unentered_cave()
            .unwrap()
            .into_iter()
            .min_by(|a, b| distance(*a, at).total_cmp(&distance(*b, at)))
            .unwrap();
        assert_eq!(torches[0], nearest, "the near end first: {torches:?}");
        assert!(
            torches.windows(2).all(|w| distance(w[0], w[1]) >= 6.0),
            "{torches:?}"
        );
        assert!(torches.len() <= MOST_TORCHES);
    }

    #[test]
    fn a_cave_read_answer_fills_the_caves_and_not_the_village_s_list() {
        let mut memory = memory();
        memory.scans_out.push((7, "cave_read".to_string()));
        let positions: Vec<(String, BlockPos)> = tunnel(20, 400, 20)
            .into_iter()
            .map(|p| ("minecraft:air".to_string(), p))
            .collect();
        memory.observe(
            &craftkit::world::Observations {
                scans: vec![craftkit::world::Scan {
                    id: 7,
                    counts: vec![],
                    positions,
                }],
                ..craftkit::world::Observations::default()
            },
            0.1,
        );

        assert!(memory.caves.len() > 64, "past the village list's cap");
        assert!(memory.caves.len() <= MOST_CAVE);
        assert!(memory.scanned.is_empty());
        assert!(memory.scans_out.is_empty());
    }
}
