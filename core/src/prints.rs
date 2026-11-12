//! The footprints round their house (`prints_around_the_house`, `docs/events_todo.md` 108,
//! owner's): drawn on their client as the handprints are, kept by the memory and sent again
//! whenever they are near home, so they stay; a few more each night it walks there. Any ground.

use craftkit::world::{BlockPos, Pos};
use serde::{Deserialize, Serialize};

use crate::memory::{distance, Memory};

/// The most prints kept: the oldest go first.
pub const MOST: usize = 160;
/// A stride, blocks, and a foot's way off the line it walks.
pub const STRIDE: f64 = 0.9;
pub const FOOT_OFF: f64 = 0.16;
/// Sent again when they are this near home, this often (ticks), and for this long (seconds).
pub const NEAR_HOME: f64 = 48.0;
pub const RESEND_TICKS: u64 = 9 * 60 * 20;
pub const SHOWN_SECONDS: f32 = 600.0;

/// One print: where on the ground (its y the ground's near there; the client finds the top),
/// and the way the foot points, degrees as the game's yaw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Print {
    pub pos: Pos,
    pub yaw: f32,
}

/// The house's footprint, from the blocks they built round its middle: `[min_x, min_z, max_x,
/// max_z]`, and its floor; four blocks round the middle when they built none.
pub fn house_box(memory: &Memory) -> Option<([i32; 4], i32)> {
    let home = memory.home.as_ref()?;
    let c = home.center;
    let near: Vec<BlockPos> = memory
        .walls
        .iter()
        .map(|(p, _)| *p)
        .filter(|p| distance(*p, c) <= 12.0)
        .collect();
    if near.len() < 8 {
        return Some(([c[0] - 4, c[2] - 4, c[0] + 4, c[2] + 4], home.floor_y));
    }
    let min_x = near.iter().map(|p| p[0]).min()?;
    let max_x = near.iter().map(|p| p[0]).max()?;
    let min_z = near.iter().map(|p| p[2]).min()?;
    let max_z = near.iter().map(|p| p[2]).max()?;

    Some(([min_x, min_z, max_x, max_z], home.floor_y))
}

/// Steps from `from` to `to`, a stride apart, left and right of the line, each pointing the way
/// it goes.
pub fn walk(from: [f64; 2], to: [f64; 2], y: f64) -> Vec<Print> {
    let (dx, dz) = (to[0] - from[0], to[1] - from[1]);
    let length = (dx * dx + dz * dz).sqrt();
    if length < STRIDE {
        return Vec::new();
    }
    let (ux, uz) = (dx / length, dz / length);
    let yaw = (-ux).atan2(uz).to_degrees() as f32;
    let steps = (length / STRIDE) as usize;
    (0..=steps)
        .map(|i| {
            let along = i as f64 * STRIDE;
            let side = if i % 2 == 0 { 1.0 } else { -1.0 } * FOOT_OFF;
            Print {
                pos: [
                    from[0] + ux * along - uz * side,
                    y,
                    from[1] + uz * along + ux * side,
                ],
                yaw,
            }
        })
        .collect()
}

/// A pair standing at `at`, facing `toward`.
pub fn standing(at: [f64; 2], toward: [f64; 2], y: f64) -> Vec<Print> {
    let (dx, dz) = (toward[0] - at[0], toward[1] - at[1]);
    let length = (dx * dx + dz * dz).sqrt().max(1e-6);
    let (ux, uz) = (dx / length, dz / length);
    let yaw = (-ux).atan2(uz).to_degrees() as f32;
    [-1.0, 1.0]
        .iter()
        .map(|side| Print {
            pos: [
                at[0] - uz * side * FOOT_OFF * 1.3,
                y,
                at[1] + ux * side * FOOT_OFF * 1.3,
            ],
            yaw,
        })
        .collect()
}

/// The night's new prints, the `n`th night it walks there, by turns: in from the open to a window,
/// stopping under it; round the walls, a loop; standing before the door in pairs, as if waiting.
pub fn night(memory: &Memory, n: usize) -> Vec<Print> {
    let Some(([x0, z0, x1, z1], floor)) = house_box(memory) else {
        return Vec::new();
    };
    let y = floor as f64;
    let mid = [(x0 + x1) as f64 / 2.0 + 0.5, (z0 + z1) as f64 / 2.0 + 0.5];
    match n % 3 {
        0 => {
            // To a window, else the wall's middle on a side by the night.
            let window = crate::brain::act::windows_of(memory)
                .get(n / 3 % 4)
                .copied()
                .map(|w| [w[0] as f64 + 0.5, w[2] as f64 + 0.5])
                .unwrap_or([x1 as f64 + 1.0, mid[1]]);
            let (dx, dz) = (window[0] - mid[0], window[1] - mid[1]);
            let length = (dx * dx + dz * dz).sqrt().max(1e-6);
            let (ux, uz) = (dx / length, dz / length);
            let under = [window[0] + ux * 1.2, window[1] + uz * 1.2];
            let from = [under[0] + ux * 18.0, under[1] + uz * 18.0];
            let mut prints = walk(from, under, y);
            prints.extend(standing(under, window, y));
            prints
        }
        1 => {
            // Round the walls, a block and a half out.
            let (a, b, c, d) = (
                x0 as f64 - 1.5,
                z0 as f64 - 1.5,
                x1 as f64 + 2.5,
                z1 as f64 + 2.5,
            );
            let mut prints = walk([a, b], [c, b], y);
            prints.extend(walk([c, b], [c, d], y));
            prints.extend(walk([c, d], [a, d], y));
            prints.extend(walk([a, d], [a, b], y));
            prints
        }
        _ => {
            // Before the door they use: pairs, a few, facing it.
            let Some((door, out)) = memory
                .usual_door()
                .zip(memory.home.as_ref())
                .map(|(d, h)| (d, crate::brain::act::out_of(memory, h.center, d)))
            else {
                return Vec::new();
            };
            let face = [door[0] as f64 + 0.5, door[2] as f64 + 0.5];
            let mut prints = Vec::new();
            for (k, shift) in [(2.0, 0.0), (2.6, 0.5), (3.1, -0.4)] {
                let at = [
                    face[0] + out[0] as f64 * k + out[1] as f64 * shift,
                    face[1] + out[1] as f64 * k + out[0] as f64 * shift,
                ];
                prints.extend(standing(at, face, y));
            }
            prints
        }
    }
}

/// The prints as a hint's detail: `x,y,z,yaw` each, `;` between, to a tenth.
pub fn encode(prints: &[Print]) -> String {
    prints
        .iter()
        .map(|p| {
            format!(
                "{:.1},{:.1},{:.1},{:.0}",
                p.pos[0], p.pos[1], p.pos[2], p.yaw
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

impl Memory {
    /// The night's prints kept, the oldest let go past [`MOST`].
    pub fn keep_prints(&mut self, new: Vec<Print>) {
        self.prints.extend(new);
        if self.prints.len() > MOST {
            let over = self.prints.len() - MOST;
            self.prints.drain(..over);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_walk_is_strides_left_and_right_pointing_its_way() {
        let prints = walk([0.0, 0.0], [0.0, 9.0], 64.0);
        assert_eq!(prints.len(), 11);
        assert!(prints.windows(2).all(|w| w[1].pos[2] > w[0].pos[2]));
        assert!(
            prints[0].pos[0] * prints[1].pos[0] < 0.0,
            "left, then right"
        );
        assert!(prints.iter().all(|p| p.yaw.abs() < 1e-3), "south is yaw 0");
        assert!(walk([0.0, 0.0], [0.3, 0.0], 64.0).is_empty());
    }

    #[test]
    fn each_night_by_turns_to_a_window_round_the_walls_and_waiting_at_the_door() {
        let mut memory = Memory::default();
        memory.home = Some(crate::memory::Home {
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            ..Default::default()
        });
        memory.home.as_mut().unwrap().outs = vec![([3, 64, 0], [1, 0])];
        let to_window = night(&memory, 0);
        assert!(to_window.len() > 10, "a way in from the open");
        let round = night(&memory, 1);
        assert!(round.len() > 20, "a loop round the walls");
        let waiting = night(&memory, 2);
        assert_eq!(waiting.len(), 6, "three pairs");
        assert!(
            waiting.iter().all(|p| p.pos[0] > 4.0),
            "out before the door"
        );
        let detail = encode(&waiting);
        assert_eq!(detail.split(';').count(), 6);
        memory.keep_prints(vec![Print::default(); MOST + 5]);
        assert_eq!(memory.prints.len(), MOST, "the oldest let go");
    }
}
