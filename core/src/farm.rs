//! The farm (batch 6, owner): the field as the brain's field read finds it (every
//! crop block around the crops the surroundings pass saw), its rows, the word a field of 7 by 5
//! or more can be harvested into, and the target's animals: the pens by the home, the coop,
//! the kind they keep most.

use craftkit::world::{BlockPos, Pos};

use crate::memory::{block_of, distance, Creature, Memory, HOME_RADIUS};

/// The crop blocks the field read looks for.
pub const CROPS: &[&str] = &[
    "minecraft:wheat",
    "minecraft:carrots",
    "minecraft:potatoes",
    "minecraft:beetroots",
];
/// The field read's half width around the crops it was told of, and its height.
pub const READ_WIDE: i32 = 12;
pub const READ_HIGH: i32 = 3;
/// The most crop blocks kept.
pub const MOST_FIELD: usize = 256;
/// A row has at least this many crops.
const LEAST_ROW: usize = 4;
/// The farm animals: what a pen holds.
pub const FARM: &[&str] = &[
    "cow",
    "pig",
    "sheep",
    "chicken",
    "goat",
    "horse",
    "donkey",
    "mule",
    "llama",
    "rabbit",
    "mooshroom",
    "camel",
];
/// Their own animals are within this many blocks of the home's middle.
const PEN_BLOCKS: f64 = (HOME_RADIUS + 16) as f64;

/// The letters a field can be harvested into, three wide and five high, `#` a harvested crop.
fn glyph(letter: char) -> Option<[&'static str; 5]> {
    Some(match letter {
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'G' => [".##", "#..", "#.#", "#.#", ".##"],
        'H' => ["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'K' => ["#.#", "##.", "#..", "##.", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#.#", "###", "###", "#.#", "#.#"],
        'N' => ["##.", "#.#", "#.#", "#.#", "#.#"],
        'O' => [".#.", "#.#", "#.#", "#.#", ".#."],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => [".##", "#..", ".#.", "..#", "##."],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'V' => ["#.#", "#.#", "#.#", "#.#", ".#."],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        _ => return None,
    })
}

/// The words a field may read, longest first, by what it wants: every one spelt from the
/// letters above, and "ME" to fall back on in the smallest field.
pub fn field_words(want: &str) -> &'static [&'static str] {
    match want {
        "a place of its own" => &["MINE", "HERE", "STAY", "OUR", "ME"],
        "to be looked at" => &["LOOK", "SEE", "EYE", "ME"],
        "one thing of theirs" => &["GIVE", "MINE", "YOUR", "ME"],
        _ => &["HERE", "STAY", "ME"],
    }
}

/// The field's box: west, east, north, south and the crops' height.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldBox {
    pub west: i32,
    pub east: i32,
    pub north: i32,
    pub south: i32,
    pub y: i32,
}

impl FieldBox {
    pub fn width(&self) -> i32 {
        self.east - self.west + 1
    }

    pub fn depth(&self) -> i32 {
        self.south - self.north + 1
    }
}

impl Memory {
    /// The crops at the field's own height: the height most of them are at.
    fn field_level(&self) -> Vec<BlockPos> {
        let mut heights: Vec<(i32, usize)> = Vec::new();
        for p in &self.field {
            match heights.iter_mut().find(|(y, _)| *y == p[1]) {
                Some((_, n)) => *n += 1,
                None => heights.push((p[1], 1)),
            }
        }
        let Some((y, _)) = heights.into_iter().max_by_key(|(_, n)| *n) else {
            return Vec::new();
        };

        self.field.iter().copied().filter(|p| p[1] == y).collect()
    }

    /// The field's rows, along west to east, north first, each of four crops or more.
    pub fn field_rows(&self) -> Vec<Vec<BlockPos>> {
        let level = self.field_level();
        let mut rows: Vec<Vec<BlockPos>> = Vec::new();
        for p in level {
            match rows.iter_mut().find(|r| r[0][2] == p[2]) {
                Some(row) => row.push(p),
                None => rows.push(vec![p]),
            }
        }
        rows.retain(|r| r.len() >= LEAST_ROW);
        for row in &mut rows {
            row.sort_by_key(|p| p[0]);
        }
        rows.sort_by_key(|r| r[0][2]);

        rows
    }

    /// The field's box when it is a field of 7 by 5 or more, mostly planted (four in five).
    pub fn field_box(&self) -> Option<FieldBox> {
        let level = self.field_level();
        let first = level.first()?;
        let mut b = FieldBox {
            west: first[0],
            east: first[0],
            north: first[2],
            south: first[2],
            y: first[1],
        };
        for p in &level {
            b.west = b.west.min(p[0]);
            b.east = b.east.max(p[0]);
            b.north = b.north.min(p[2]);
            b.south = b.south.max(p[2]);
        }
        let area = (b.width() * b.depth()) as usize;
        (b.width() >= 7 && b.depth() >= 5 && level.len() * 5 >= area * 4).then_some(b)
    }

    /// The crops to harvest so the field reads a word from above, north at the top: the
    /// longest of its words that fits, in the middle of the field. Only where there are crops.
    pub fn field_word(&self) -> Option<(&'static str, Vec<BlockPos>)> {
        let b = self.field_box()?;
        let fits = |w: &str| (w.len() as i32) * 4 - 1 <= b.width();
        let word = field_words(&self.want).iter().copied().find(|w| fits(w))?;
        let wide = word.len() as i32 * 4 - 1;
        let west = b.west + (b.width() - wide) / 2;
        let north = b.north + (b.depth() - 5) / 2;
        let mut cells = Vec::new();
        for (i, letter) in word.chars().enumerate() {
            let rows = glyph(letter)?;
            for (dz, row) in rows.iter().enumerate() {
                for (dx, c) in row.chars().enumerate() {
                    let p = [west + i as i32 * 4 + dx as i32, b.y, north + dz as i32];
                    if c == '#' && self.field.contains(&p) {
                        cells.push(p);
                    }
                }
            }
        }

        Some((word, cells))
    }

    /// Their farm animals, by the home: in the pens.
    pub fn penned(&self) -> Vec<&Creature> {
        let Some(center) = self.home.as_ref().map(|h| h.center) else {
            return Vec::new();
        };
        self.last
            .creatures_near
            .iter()
            .filter(|c| {
                FARM.contains(&c.kind.as_str()) && distance(block_of(c.pos), center) <= PEN_BLOCKS
            })
            .collect()
    }

    /// The kind they keep most, and where those stand on average.
    pub fn kept_most(&self) -> Option<(String, Pos)> {
        let penned = self.penned();
        let mut kinds: Vec<(&str, usize)> = Vec::new();
        for c in &penned {
            match kinds.iter_mut().find(|(k, _)| *k == c.kind) {
                Some((_, n)) => *n += 1,
                None => kinds.push((&c.kind, 1)),
            }
        }
        let (kind, _) = kinds.into_iter().max_by_key(|(_, n)| *n)?;
        let of: Vec<&&Creature> = penned.iter().filter(|c| c.kind == kind).collect();
        let n = of.len() as f64;
        let middle = [
            of.iter().map(|c| c.pos[0]).sum::<f64>() / n,
            of.iter().map(|c| c.pos[1]).sum::<f64>() / n,
            of.iter().map(|c| c.pos[2]).sum::<f64>() / n,
        ];

        Some((kind.to_string(), middle))
    }

    /// Eggs from the coop to the door they use: a point every two blocks, at most six, when
    /// two chickens or more keep by the home.
    pub fn egg_line(&self) -> Vec<Pos> {
        let chickens: Vec<&Creature> = self
            .penned()
            .into_iter()
            .filter(|c| c.kind == "chicken")
            .collect();
        let Some(door) = self.usual_door() else {
            return Vec::new();
        };
        if chickens.len() < 2 {
            return Vec::new();
        }
        let n = chickens.len() as f64;
        let coop = [
            chickens.iter().map(|c| c.pos[0]).sum::<f64>() / n,
            chickens.iter().map(|c| c.pos[2]).sum::<f64>() / n,
        ];
        let to = [door[0] as f64 + 0.5, door[2] as f64 + 0.5];
        let (dx, dz) = (to[0] - coop[0], to[1] - coop[1]);
        let length = (dx * dx + dz * dz).sqrt();
        if length < 4.0 {
            return Vec::new();
        }
        let steps = ((length / 2.0) as usize).min(6);

        (1..=steps)
            .map(|i| {
                let t = i as f64 * 2.0 / length;
                [coop[0] + dx * t, door[1] as f64 + 0.2, coop[1] + dz * t]
            })
            .collect()
    }

    /// The farm animals within 16 blocks of the target, nearest first: what may turn on them.
    pub fn animals_near_target(&self) -> Vec<&Creature> {
        let at = block_of(self.last.pos);
        let mut near: Vec<&Creature> = self
            .last
            .creatures_near
            .iter()
            .filter(|c| {
                FARM.contains(&c.kind.as_str()) && !c.pet && distance(block_of(c.pos), at) <= 16.0
            })
            .collect();
        near.sort_by(|a, b| {
            distance(block_of(a.pos), at).total_cmp(&distance(block_of(b.pos), at))
        });

        near
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Home;

    fn field(west: i32, north: i32, wide: i32, deep: i32) -> Vec<BlockPos> {
        (0..wide)
            .flat_map(|x| (0..deep).map(move |z| [west + x, 64, north + z]))
            .collect()
    }

    fn animal(kind: &str, pos: Pos) -> Creature {
        Creature {
            id: 1,
            kind: kind.to_string(),
            pos,
            named: false,
            pet: false,

            baby: false,
        }
    }

    fn with_home() -> Memory {
        Memory {
            home: Some(Home {
                center: [0, 64, 0],
                floor_y: 64,
                doors: vec![[4, 64, 0]],
                ..Home::default()
            }),
            ..Memory::default()
        }
    }

    #[test]
    fn every_word_is_spelt_from_letters_the_font_has() {
        for want in [
            "a place of its own",
            "to be looked at",
            "one thing of theirs",
            "",
        ] {
            for word in field_words(want) {
                assert!(word.chars().all(|c| glyph(c).is_some()), "{word}");
            }
            assert_eq!(*field_words(want).last().unwrap(), "ME");
        }
    }

    #[test]
    fn a_small_field_reads_me_and_a_wide_one_a_longer_word_in_its_middle() {
        let mut memory = Memory {
            field: field(10, 20, 7, 5),
            want: "to be looked at".to_string(),
            ..Memory::default()
        };
        let (word, cells) = memory.field_word().unwrap();
        assert_eq!(word, "ME");
        assert!(cells.iter().all(|c| memory.field.contains(c)));
        assert!(cells.contains(&[10, 64, 20]), "M's top left corner");

        memory.field = field(0, 0, 16, 7);
        let (word, cells) = memory.field_word().unwrap();
        assert_eq!(word, "LOOK");
        let wide = cells.iter().map(|c| c[0]).max().unwrap()
            - cells.iter().map(|c| c[0]).min().unwrap()
            + 1;
        assert_eq!(wide, 15, "four letters and three gaps");
        assert!(
            cells.iter().all(|c| (1..=5).contains(&c[2])),
            "five rows in the middle of seven"
        );
    }

    #[test]
    fn a_patchy_or_small_field_spells_nothing_and_its_rows_are_west_to_east() {
        let mut memory = Memory {
            field: field(0, 0, 6, 5),
            ..Memory::default()
        };
        assert_eq!(memory.field_box(), None, "too narrow");
        memory.field = field(0, 0, 8, 6);
        memory.field.retain(|p| (p[0] + p[2]) % 2 == 0);
        assert_eq!(memory.field_box(), None, "half planted");

        memory.field = field(0, 0, 8, 3);
        memory.field.push([30, 70, 30]); // a stray plant on a hill
        let rows = memory.field_rows();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0], [0, 64, 0]);
        assert!(rows.iter().all(|r| r.windows(2).all(|w| w[0][0] < w[1][0])));
    }

    #[test]
    fn the_kind_they_keep_most_and_where_it_stands() {
        let mut memory = with_home();
        memory.last.creatures_near = vec![
            animal("sheep", [10.0, 64.0, 10.0]),
            animal("sheep", [12.0, 64.0, 10.0]),
            animal("cow", [8.0, 64.0, -8.0]),
            animal("sheep", [200.0, 64.0, 0.0]), // not theirs
        ];
        let (kind, middle) = memory.kept_most().unwrap();
        assert_eq!(kind, "sheep");
        assert_eq!(middle, [11.0, 64.0, 10.0]);
        assert_eq!(memory.penned().len(), 3);
    }

    #[test]
    fn eggs_go_from_the_coop_toward_the_door() {
        let mut memory = with_home();
        memory.last.creatures_near = vec![
            animal("chicken", [-10.0, 64.0, 0.5]),
            animal("chicken", [-10.0, 64.0, 0.5]),
        ];
        let eggs = memory.egg_line();
        assert_eq!(eggs.len(), 6);
        assert!(
            eggs.windows(2).all(|w| w[1][0] > w[0][0]),
            "toward the door: {eggs:?}"
        );
        assert!(eggs.iter().all(|e| e[0] < 4.5));

        memory.last.creatures_near.pop();
        assert!(memory.egg_line().is_empty(), "one chicken is no coop");
    }

    #[test]
    fn a_field_read_fills_the_field_and_a_lead_is_seen() {
        use craftkit::world::{Entity, Observations, Player, Scan};
        let mut memory = with_home();
        memory.target = Some(0);
        memory.scans_out.push((9, "field_read".to_string()));
        memory.observe(
            &Observations {
                players: vec![Player::default()],
                scans: vec![Scan {
                    id: 9,
                    counts: vec![],
                    positions: field(0, 0, 8, 6)
                        .into_iter()
                        .map(|p| ("minecraft:wheat".to_string(), p))
                        .collect(),
                }],
                entities: vec![
                    Entity {
                        id: 5,
                        kind: "minecraft:horse".to_string(),
                        leashed: true,
                        ..Entity::default()
                    },
                    Entity {
                        id: 6,
                        kind: "minecraft:cow".to_string(),
                        ..Entity::default()
                    },
                ],
                ..Observations::default()
            },
            0.1,
        );

        assert_eq!(memory.field.len(), 48);
        assert!(memory.field_box().is_some());
        assert_eq!(memory.last.leashed, vec![5]);
    }
}
