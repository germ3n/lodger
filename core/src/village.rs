//! The village (batch 12, owner): its middle, its doors one to a door, its lanterns,
//! the villager nearest the target, and the door of the house the villagers are packed into
//! when the village empties.

use craftkit::world::BlockPos;

use crate::memory::{block_of, distance, Creature, Memory};

impl Memory {
    /// The village's middle, from the surroundings.
    pub fn village_middle(&self) -> Option<BlockPos> {
        self.feature("village").map(|v| v.pos)
    }

    /// The village's doors, one block to a door (the lower half), from the village read.
    pub fn village_doors(&self) -> Vec<BlockPos> {
        let mut doors: Vec<BlockPos> = Vec::new();
        for p in self.scanned("_door") {
            match doors.iter_mut().find(|d| d[0] == p[0] && d[2] == p[2]) {
                Some(d) if p[1] < d[1] => *d = p,
                Some(_) => {}
                None => doors.push(p),
            }
        }

        doors
    }

    /// The door of the house they are packed into: the village's door nearest its middle.
    pub fn gathering_door(&self) -> Option<BlockPos> {
        let middle = self.village_middle()?;
        self.village_doors()
            .into_iter()
            .min_by(|a, b| distance(*a, middle).total_cmp(&distance(*b, middle)))
    }

    /// Whether the target is at least `blocks` from the village, or away from the game.
    pub fn far_from_village(&self, blocks: f64) -> bool {
        let Some(middle) = self.village_middle() else {
            return false;
        };
        !self.last.online || distance(block_of(self.last.pos), middle) >= blocks
    }

    /// The villager nearest the target.
    pub fn villager_near(&self) -> Option<&Creature> {
        let at = block_of(self.last.pos);
        self.last
            .creatures_near
            .iter()
            .filter(|c| c.kind == "villager")
            .min_by(|a, b| distance(block_of(a.pos), at).total_cmp(&distance(block_of(b.pos), at)))
    }

    /// The point of their route nearest the village: where its lantern is set down.
    pub fn route_by_village(&self) -> Option<BlockPos> {
        let middle = self.village_middle()?;
        self.route()?
            .points
            .iter()
            .copied()
            .min_by(|a, b| distance(*a, middle).total_cmp(&distance(*b, middle)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Route, Surround};
    use craftkit::world::Feature;

    fn village() -> Memory {
        let mut memory = Memory {
            surroundings: Some(Surround {
                features: vec![Feature {
                    kind: "village".to_string(),
                    pos: [100, 64, 100],
                    size: 40,
                    detail: String::new(),
                }],
                ..Surround::default()
            }),
            ..Memory::default()
        };
        memory.scanned = vec![
            ("oak_door".to_string(), [104, 65, 100]),
            ("oak_door".to_string(), [104, 64, 100]),
            ("spruce_door".to_string(), [120, 64, 90]),
            ("bell".to_string(), [100, 65, 100]),
        ];
        memory
    }

    #[test]
    fn a_door_is_counted_once_by_its_lower_half_and_the_nearest_is_the_gathering() {
        let memory = village();
        let doors = memory.village_doors();
        assert_eq!(doors.len(), 2);
        assert!(doors.contains(&[104, 64, 100]));
        assert_eq!(memory.gathering_door(), Some([104, 64, 100]));
    }

    #[test]
    fn far_from_the_village_is_far_or_away() {
        let mut memory = village();
        memory.last.online = true;
        memory.last.pos = [110.0, 64.0, 100.0];
        assert!(!memory.far_from_village(64.0));
        memory.last.pos = [300.0, 64.0, 100.0];
        assert!(memory.far_from_village(64.0));
        memory.last.pos = [110.0, 64.0, 100.0];
        memory.last.online = false;
        assert!(memory.far_from_village(64.0), "away from the game");
    }

    #[test]
    fn the_lantern_goes_on_the_route_where_it_passes_the_village() {
        let mut memory = village();
        memory.routes.push(Route {
            points: (0..20).map(|i| [i * 10, 64, 80]).collect(),
            walked: 3,
            first_day: 0.0,
        });
        assert_eq!(memory.route_by_village(), Some([100, 64, 80]));
    }
}
