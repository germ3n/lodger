//! Warped animals (owner, 2026-11-10: "warped versions of the entity ... a cow or pig"): one
//! of the target's own animals, or one outside when they keep too few, warped for a minute and
//! put back as it was (`warp_mob`): frozen with its head always on them, and one or two of too
//! tall, upside down, and a step closer each time they look away; now and then it stands in
//! their doorway first. Vanilla tricks only: its size, the Dinnerbone flip, its facing, where
//! it stands.

use craftkit::world::Pos;

use crate::memory::{distance, Creature, Memory, HOME_RADIUS};

/// The animals that warp: plain farm and field animals.
pub const WARPABLE: &[&str] = &[
    "cow",
    "pig",
    "sheep",
    "chicken",
    "goat",
    "horse",
    "rabbit",
    "mooshroom",
];
/// Their own: tamed, or within this many blocks of the home's middle.
const OWN_BLOCKS: f64 = (HOME_RADIUS + 16) as f64;
/// Near enough to see, far enough not to touch: from the target.
const FROM: f64 = 12.0;
const TO: f64 = 32.0;

/// How one warp looks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Warp {
    pub scale: f32,
    pub flip: bool,
    pub creep: bool,
    pub doorway: bool,
}

/// The warp for the `times`th one and a seed: the head on them always; then too tall, upside
/// down, or creeping, one of them, and a second one of them every other time; the doorway one
/// time in three. A named animal is never flipped (its name would be lost).
pub fn warp(times: usize, seed: usize, named: bool) -> Warp {
    // Two of the three always, together (owner, 2026-11-10: "too stiff ... should be
    // spookier"; it was one, or two every other time).
    let first = (times + seed) % 3;
    let second = Some((first + 1 + seed % 2) % 3);
    let has = |n: usize| first == n || second == Some(n);
    let flip = has(1) && !named;

    Warp {
        // A named one that would have flipped grows instead; the height its own each time.
        scale: if has(0) || (has(1) && named) {
            1.5 + (seed % 8) as f32 * 0.1
        } else {
            1.0
        },
        flip,
        creep: has(2),
        doorway: (times + seed).is_multiple_of(3),
    }
}

impl Memory {
    /// The animal to warp: theirs first (tamed, or by the home), else any outside, of the
    /// warpable kinds, 12 to 32 blocks from the target; the nearest of those.
    pub fn warp_animal(&self) -> Option<&Creature> {
        let at = crate::memory::block_of(self.last.pos);
        let center = self.home.as_ref().map(|h| h.center);
        let fits = |c: &&Creature| {
            let d = distance(crate::memory::block_of(c.pos), at);
            WARPABLE.contains(&c.kind.as_str()) && (FROM..=TO).contains(&d)
        };
        let own = |c: &&Creature| {
            c.pet
                || center.is_some_and(|h| distance(crate::memory::block_of(c.pos), h) <= OWN_BLOCKS)
        };
        let nearest = |a: &&Creature, b: &&Creature| {
            distance(crate::memory::block_of(a.pos), at)
                .total_cmp(&distance(crate::memory::block_of(b.pos), at))
        };

        self.last
            .creatures_near
            .iter()
            .filter(fits)
            .filter(own)
            .min_by(nearest)
            .or_else(|| self.last.creatures_near.iter().filter(fits).min_by(nearest))
    }

    /// Where a warped animal may stand first: just outside the door they use.
    pub fn doorway_spot(&self) -> Option<Pos> {
        let home = self.home.as_ref()?;
        let door = self.usual_door()?;
        let dx = (door[0] - home.center[0]).signum();
        let dz = (door[2] - home.center[2]).signum();
        let (sx, sz) = if (door[0] - home.center[0]).abs() >= (door[2] - home.center[2]).abs() {
            (dx, 0)
        } else {
            (0, dz)
        };

        Some([
            (door[0] + sx * 2) as f64 + 0.5,
            door[1] as f64,
            (door[2] + sz * 2) as f64 + 0.5,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Home;

    fn animal(id: u32, kind: &str, pos: Pos) -> Creature {
        Creature {
            id,
            kind: kind.to_string(),
            pos,
            named: false,
            pet: false,
        }
    }

    #[test]
    fn their_own_animal_is_warped_before_one_outside() {
        let mut memory = Memory {
            home: Some(Home {
                center: [0, 64, 0],
                floor_y: 64,
                ..Home::default()
            }),
            ..Memory::default()
        };
        memory.last.pos = [40.0, 64.0, 0.0];
        memory.last.creatures_near = vec![
            animal(1, "cow", [55.0, 64.0, 0.0]), // outside, nearer
            animal(2, "pig", [20.0, 64.0, 0.0]), // by the home: theirs
            animal(3, "zombie", [45.0, 64.0, 0.0]),
            animal(4, "cow", [41.0, 64.0, 0.0]), // too near to see
        ];
        assert_eq!(memory.warp_animal().map(|c| c.id), Some(2));

        memory.last.creatures_near.retain(|c| c.id != 2);
        assert_eq!(
            memory.warp_animal().map(|c| c.id),
            Some(1),
            "one outside when they keep none near"
        );
        memory.last.creatures_near.retain(|c| c.id != 1);
        assert_eq!(memory.warp_animal(), None);
    }

    #[test]
    fn every_warp_has_two_of_the_three_and_a_named_one_is_never_flipped() {
        for times in 0..12 {
            for seed in 0..12 {
                let w = warp(times, seed, false);
                let count = usize::from(w.scale > 1.0) + usize::from(w.flip) + usize::from(w.creep);
                assert_eq!(count, 2, "{times} {seed}: {w:?}");
                assert!(w.scale <= 2.2);
                let named = warp(times, seed, true);
                assert!(!named.flip);
                assert!(named.scale > 1.0 || named.creep);
            }
        }
        // Every one of the three comes up.
        let all: Vec<Warp> = (0..6).map(|t| warp(t, 0, false)).collect();
        assert!(all.iter().any(|w| w.flip));
        assert!(all.iter().any(|w| w.creep));
        assert!(all.iter().any(|w| w.scale > 1.0));
        assert!(all.iter().any(|w| w.doorway));
    }

    #[test]
    fn the_doorway_is_just_outside_the_door_they_use() {
        let memory = Memory {
            home: Some(Home {
                center: [0, 64, 0],
                floor_y: 64,
                doors: vec![[4, 64, 0]],
                ..Home::default()
            }),
            ..Memory::default()
        };

        assert_eq!(memory.doorway_spot(), Some([6.5, 64.0, 0.5]));
    }
}
