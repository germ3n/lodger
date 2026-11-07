//! Water, boats and snow (batch 10, owner): their boat, the shore of the water near
//! them on their side, the dark under it, and the way from the door to the bed for wet steps.

use craftkit::world::{BlockPos, Pos};

use crate::memory::{block_of, distance, Creature, Memory};

impl Memory {
    /// Their boat, near the home.
    pub fn their_boat(&self) -> Option<&Creature> {
        self.vehicles
            .iter()
            .find(|v| v.kind.ends_with("boat") || v.kind.ends_with("raft"))
    }

    /// The water near the target: the surroundings' water within `blocks` of them, its middle
    /// and how far across.
    pub fn water_near(&self, blocks: f64) -> Option<(BlockPos, u32)> {
        let at = block_of(self.last.pos);
        let water = self.feature("water")?;
        (distance([water.pos[0], at[1], water.pos[2]], at) <= blocks)
            .then_some((water.pos, water.size))
    }

    /// The shore on the target's side: from the water's middle toward them, `inset` blocks short
    /// of the edge (the edge is half the water across from the middle).
    pub fn shore_toward_target(&self, inset: f64) -> Option<Pos> {
        let (middle, across) = self.water_near(64.0)?;
        let (dx, dz) = (
            self.last.pos[0] - (middle[0] as f64 + 0.5),
            self.last.pos[2] - (middle[2] as f64 + 0.5),
        );
        let length = (dx * dx + dz * dz).sqrt();
        if length < 1.0 {
            return None;
        }
        let reach = (across as f64 / 2.0 - inset)
            .max(0.0)
            .min(length - 2.0)
            .max(0.0);

        Some([
            middle[0] as f64 + 0.5 + dx / length * reach,
            middle[1] as f64,
            middle[2] as f64 + 0.5 + dz / length * reach,
        ])
    }

    /// Steps from the door they use to their bed, a block apart, at most twelve.
    pub fn door_to_bed(&self) -> Vec<Pos> {
        let (Some(door), Some(bed)) = (self.usual_door(), self.home.as_ref().and_then(|h| h.bed))
        else {
            return Vec::new();
        };
        let (dx, dz) = ((bed[0] - door[0]) as f64, (bed[2] - door[2]) as f64);
        let steps = ((dx * dx + dz * dz).sqrt() as usize).clamp(2, 12);

        (0..=steps)
            .map(|i| {
                let t = i as f64 / steps as f64;
                [
                    door[0] as f64 + 0.5 + dx * t,
                    door[1] as f64 + 0.1,
                    door[2] as f64 + 0.5 + dz * t,
                ]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Home, Surround};
    use craftkit::world::Feature;

    fn by_the_lake() -> Memory {
        let mut memory = Memory {
            home: Some(Home {
                center: [0, 64, 0],
                floor_y: 64,
                bed: Some([0, 64, 6]),
                doors: vec![[0, 64, -4]],
                ..Home::default()
            }),
            surroundings: Some(Surround {
                features: vec![Feature {
                    kind: "water".to_string(),
                    pos: [40, 62, 0],
                    size: 20,
                    detail: String::new(),
                }],
                ..Surround::default()
            }),
            ..Memory::default()
        };
        memory.last.pos = [10.0, 64.0, 0.0];
        memory
    }

    #[test]
    fn the_shore_is_on_their_side_short_of_the_edge() {
        let memory = by_the_lake();
        let shore = memory.shore_toward_target(1.0).unwrap();
        assert!(
            (shore[0] - 31.5).abs() < 0.01,
            "nine in from the middle on their side: {shore:?}"
        );
        assert_eq!(shore[1], 62.0);
        assert!(
            memory.water_near(16.0).is_none(),
            "the lake's middle is thirty off"
        );
        assert!(memory.water_near(32.0).is_some());
    }

    #[test]
    fn the_wet_steps_go_from_the_door_to_the_bed() {
        let memory = by_the_lake();
        let steps = memory.door_to_bed();
        assert_eq!(steps.first().unwrap()[2], -3.5);
        assert_eq!(steps.last().unwrap()[2], 6.5);
        assert!(steps.windows(2).all(|w| w[1][2] > w[0][2]));
    }

    #[test]
    fn a_boat_is_theirs_and_a_minecart_is_not() {
        let mut memory = by_the_lake();
        memory.vehicles = vec![Creature {
            id: 4,
            kind: "minecart".to_string(),
            ..Creature::default()
        }];
        assert!(memory.their_boat().is_none());
        memory.vehicles.push(Creature {
            id: 5,
            kind: "oak_boat".to_string(),
            ..Creature::default()
        });
        assert_eq!(memory.their_boat().unwrap().id, 5);
    }
}
