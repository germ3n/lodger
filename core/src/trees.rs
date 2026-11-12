//! The trees around the target (the forest events, `docs/events_todo.md` 81 to 90): read by
//! the brain's tree read, a scan for logs around them while they are outdoors in a wooded biome,
//! each column of three logs or more a trunk. Not saved: the trees near them are read again as
//! they walk.

use craftkit::world::{BlockPos, Pos};

use crate::memory::{block_of, distance, Memory};

/// The tree read's half width, how far below and above their feet, and how often it reads.
pub const READ_WIDE: i32 = 20;
pub const READ_BELOW: i32 = 4;
pub const READ_ABOVE: i32 = 8;
pub const READ_EVERY_SECONDS: u64 = 20;
/// The most logs a read keeps.
pub const MOST_LOGS: usize = 2048;
/// Logs stacked in a column that make a trunk, not a fallen log or a beam.
pub const TRUNK_LOGS: usize = 3;
/// Trunks within this many blocks of them, and how many, make the woods.
pub const WOODS_WITHIN: f64 = 24.0;
pub const WOODS_TRUNKS: usize = 6;
/// How far out from a trunk's middle a peeking figure stands to its side, and behind it: half of
/// it shows past the bark.
pub const PEEK_SIDE: f64 = 0.6;
pub const PEEK_BEHIND: f64 = 0.85;

/// Whether a biome grows trees enough to read for: the forests, taigas, jungles, groves and
/// swamps, by the words in its id.
pub fn wooded(biome: &str) -> bool {
    [
        "forest", "taiga", "jungle", "grove", "woodland", "swamp", "cherry",
    ]
    .iter()
    .any(|w| biome.contains(w))
}

/// The trunks among the logs read, each with its log's id: each column of [`TRUNK_LOGS`] or
/// more, at its lowest log. A column with a log the target placed is a house's wall or post, not
/// a tree.
pub fn trunks_of(logs: &[(String, BlockPos)], placed: &[BlockPos]) -> Vec<(BlockPos, String)> {
    let mut columns: Vec<([i32; 2], Vec<i32>, String)> = Vec::new();
    for (kind, log) in logs {
        let key = [log[0], log[2]];
        match columns.iter_mut().find(|(k, _, _)| *k == key) {
            Some((_, ys, low)) => {
                if ys.iter().all(|y| log[1] < *y) {
                    *low = kind.clone();
                }
                ys.push(log[1]);
            }
            None => columns.push((key, vec![log[1]], kind.clone())),
        }
    }
    columns
        .into_iter()
        .filter(|([x, z], ys, _)| {
            ys.len() >= TRUNK_LOGS && !placed.iter().any(|p| p[0] == *x && p[2] == *z)
        })
        .map(|([x, z], ys, kind)| ([x, *ys.iter().min().unwrap_or(&0), z], kind))
        .collect()
}

/// A log's stripped kind, `minecraft:oak_log` to `minecraft:stripped_oak_log`; none for one
/// stripped already.
pub fn stripped(log: &str) -> Option<String> {
    let (space, name) = log.split_once(':').unwrap_or(("minecraft", log));
    (!name.starts_with("stripped_")).then(|| format!("{space}:stripped_{name}"))
}

/// Where a figure peeks out from behind `trunk` as seen from `at`, and where it hides: behind the
/// trunk from them, to one side by [`PEEK_SIDE`] so half of it shows, and straight behind it.
/// `side` picks the side, 1 or -1. Both on the trunk's lowest log, the ground it grows from.
pub fn peek_spots(trunk: BlockPos, at: Pos, side: f64) -> (Pos, Pos) {
    let middle = [trunk[0] as f64 + 0.5, trunk[2] as f64 + 0.5];
    let (dx, dz) = (middle[0] - at[0], middle[1] - at[2]);
    let length = (dx * dx + dz * dz).sqrt().max(0.01);
    let (ax, az) = (dx / length, dz / length);
    let (sx, sz) = (-az * side, ax * side);
    let behind = [
        middle[0] + ax * PEEK_BEHIND,
        trunk[1] as f64,
        middle[1] + az * PEEK_BEHIND,
    ];
    (
        [
            behind[0] + sx * PEEK_SIDE,
            behind[1],
            behind[2] + sz * PEEK_SIDE,
        ],
        behind,
    )
}

impl Memory {
    /// A trunk's log, by id, as the last read had it.
    pub fn trunk_log(&self, trunk: BlockPos) -> Option<&str> {
        self.trunk_logs
            .iter()
            .find(|(t, _)| *t == trunk)
            .map(|(_, log)| log.as_str())
    }

    /// Whether they are in the woods: outdoors, with [`WOODS_TRUNKS`] trunks of the last read
    /// within [`WOODS_WITHIN`] blocks of them.
    pub fn in_the_woods(&self) -> bool {
        let at = block_of(self.last.pos);
        self.last.online
            && !self.last.indoors
            && !self.last.underground
            && self
                .trunks
                .iter()
                .filter(|t| distance(**t, at) <= WOODS_WITHIN)
                .count()
                >= WOODS_TRUNKS
    }

    /// The trunks behind them, more than 110 degrees from where they look, from `near` to `far`
    /// blocks off along the ground, farthest first.
    pub fn trunks_behind(&self, near: f64, far: f64) -> Vec<BlockPos> {
        let at = self.last.pos;
        let yaw = (self.last.yaw as f64).to_radians();
        let (fx, fz) = (-yaw.sin(), yaw.cos());
        let mut behind: Vec<(f64, BlockPos)> = self
            .trunks
            .iter()
            .filter_map(|t| {
                let (dx, dz) = (t[0] as f64 + 0.5 - at[0], t[2] as f64 + 0.5 - at[2]);
                let d = (dx * dx + dz * dz).sqrt();
                let cos = (dx * fx + dz * fz) / d.max(0.01);
                (d >= near && d <= far && cos <= 110f64.to_radians().cos()).then_some((d, *t))
            })
            .collect();
        behind.sort_by(|a, b| b.0.total_cmp(&a.0));
        behind.into_iter().map(|(_, t)| t).collect()
    }

    /// Whether they are walking out of the woods: five trunks or more behind them, one at most
    /// ahead.
    pub fn leaving_the_woods(&self) -> bool {
        self.last.online
            && self.last.walking
            && !self.last.indoors
            && self.trunks_behind(6.0, 24.0).len() >= 5
            && self.trunks_ahead(3.0, 24.0).len() <= 1
    }

    /// The trunk behind them they passed last: the nearest of those behind, three to ten blocks.
    pub fn trunk_just_passed(&self) -> Option<BlockPos> {
        self.trunks_behind(3.0, 10.0).last().copied()
    }

    /// The trunks ahead of where they look, within 55 degrees of it, from `near` to `far`
    /// blocks off along the ground, nearest first.
    pub fn trunks_ahead(&self, near: f64, far: f64) -> Vec<BlockPos> {
        let at = self.last.pos;
        let yaw = (self.last.yaw as f64).to_radians();
        let (fx, fz) = (-yaw.sin(), yaw.cos());
        let mut ahead: Vec<(f64, BlockPos)> = self
            .trunks
            .iter()
            .filter_map(|t| {
                let (dx, dz) = (t[0] as f64 + 0.5 - at[0], t[2] as f64 + 0.5 - at[2]);
                let d = (dx * dx + dz * dz).sqrt();
                let cos = (dx * fx + dz * fz) / d.max(0.01);
                (d >= near && d <= far && cos >= 55f64.to_radians().cos()).then_some((d, *t))
            })
            .collect();
        ahead.sort_by(|a, b| a.0.total_cmp(&b.0));
        ahead.into_iter().map(|(_, t)| t).collect()
    }
}

/// From the trunks behind them, farthest first, a way nearer and nearer: each four blocks or more
/// nearer than the last, at most `most`.
pub fn nearer_and_nearer(behind: &[BlockPos], at: Pos, most: usize) -> Vec<BlockPos> {
    let away = |t: &BlockPos| {
        let (dx, dz) = (t[0] as f64 + 0.5 - at[0], t[2] as f64 + 0.5 - at[2]);
        (dx * dx + dz * dz).sqrt()
    };
    let mut way: Vec<BlockPos> = Vec::new();
    for trunk in behind {
        if way.len() >= most {
            break;
        }
        if way.last().is_none_or(|l| away(trunk) <= away(l) - 4.0) {
            way.push(*trunk);
        }
    }
    way
}

/// Along the woods' edge behind them, a place between each two trunks side by side (four to
/// eight blocks apart), on the ground, at most `most`.
pub fn between_trunks(behind: &[BlockPos], most: usize) -> Vec<BlockPos> {
    let mut spots: Vec<BlockPos> = Vec::new();
    for (i, a) in behind.iter().enumerate() {
        for b in &behind[i + 1..] {
            let d = distance(*a, *b);
            let spot = [(a[0] + b[0]) / 2, -10_000, (a[2] + b[2]) / 2];
            let clear = spots.iter().all(|s| {
                let (dx, dz) = (s[0] - spot[0], s[2] - spot[2]);
                dx * dx + dz * dz >= 9
            });
            if (4.0..=8.0).contains(&d) && clear && spots.len() < most {
                spots.push(spot);
            }
        }
    }
    spots
}

/// The sapling that grows a log's tree alone: `minecraft:birch_log` a birch sapling; the woods
/// that need four (dark oak, pale oak) an oak's; a mangrove's propagule.
pub fn sapling_of(log: &str) -> String {
    let (space, name) = log.split_once(':').unwrap_or(("minecraft", log));
    let wood = name
        .trim_start_matches("stripped_")
        .trim_end_matches("_log")
        .trim_end_matches("_wood");
    match wood {
        "dark_oak" | "pale_oak" => "minecraft:oak_sapling".to_string(),
        "mangrove" => "minecraft:mangrove_propagule".to_string(),
        _ => format!("{space}:{wood}_sapling"),
    }
}

impl Memory {
    /// Where they came in by: the point of their trail eight to fourteen blocks back along it,
    /// clear of every trunk by two.
    pub fn way_they_came(&self) -> Option<BlockPos> {
        let at = block_of(self.last.pos);
        self.trail
            .iter()
            .rev()
            .find(|p| {
                let d = distance(**p, at);
                (8.0..=14.0).contains(&d) && self.trunks.iter().all(|t| distance(*t, **p) >= 2.0)
            })
            .copied()
    }
}

/// The animals of the woods and fields that run from what comes: never a pet, a villager or a
/// golem.
pub const FLEEING: &[&str] = &[
    "cow",
    "pig",
    "sheep",
    "chicken",
    "rabbit",
    "fox",
    "wolf",
    "goat",
    "horse",
    "donkey",
    "llama",
    "panda",
    "frog",
    "armadillo",
    "camel",
    "mooshroom",
];

impl Memory {
    /// Up to four animals within twenty-four blocks of them that would run: not theirs, not pets.
    pub fn animals_to_flee(&self) -> Vec<&crate::memory::Creature> {
        let at = block_of(self.last.pos);
        let mut near: Vec<&crate::memory::Creature> = self
            .last
            .creatures_near
            .iter()
            .filter(|c| {
                FLEEING.contains(&c.kind.as_str())
                    && !c.pet
                    && !c.named
                    && distance(block_of(c.pos), at) <= 24.0
            })
            .collect();
        near.sort_by(|a, b| {
            distance(block_of(a.pos), at).total_cmp(&distance(block_of(b.pos), at))
        });
        near.truncate(4);
        near
    }
}

/// A trail of blazes ahead of them: trunks close to the line straight ahead (within 25 degrees),
/// each five blocks or more past the last, nearest first, at most `most`.
pub fn blaze_line(ahead: &[BlockPos], at: Pos, yaw: f32, most: usize) -> Vec<BlockPos> {
    let yaw = (yaw as f64).to_radians();
    let (fx, fz) = (-yaw.sin(), yaw.cos());
    let mut line: Vec<BlockPos> = Vec::new();
    for t in ahead {
        let (dx, dz) = (t[0] as f64 + 0.5 - at[0], t[2] as f64 + 0.5 - at[2]);
        let d = (dx * dx + dz * dz).sqrt().max(0.01);
        let on_line = (dx * fx + dz * fz) / d >= 25f64.to_radians().cos();
        let past = line.last().is_none_or(|l| distance(*t, *l) >= 5.0);
        if on_line && past && line.len() < most {
            line.push(*t);
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walking_out_of_the_woods_figures_stand_between_the_trees_behind() {
        let mut memory = Memory::default();
        memory.last.online = true;
        memory.last.walking = true;
        memory.last.pos = [0.5, 64.0, 0.5];
        // Looking south: the woods behind them, north, and open ground ahead.
        memory.trunks = (0..6).map(|i| [i * 5 - 12, 64, -10]).collect();
        assert!(memory.leaving_the_woods());
        let spots = between_trunks(&memory.trunks_behind(6.0, 24.0), 6);
        assert!(spots.len() >= 4, "{spots:?}");
        assert!(spots.iter().all(|s| s[2] == -10 && s[1] < -1000));
        memory.trunks.push([0, 64, 8]);
        memory.trunks.push([3, 64, 12]);
        assert!(!memory.leaving_the_woods(), "still among trees ahead");
    }

    #[test]
    fn a_tree_grows_where_they_came_in_of_the_woods_own_kind() {
        assert_eq!(sapling_of("minecraft:birch_log"), "minecraft:birch_sapling");
        assert_eq!(
            sapling_of("minecraft:dark_oak_log"),
            "minecraft:oak_sapling"
        );
        assert_eq!(
            sapling_of("minecraft:mangrove_log"),
            "minecraft:mangrove_propagule"
        );
        let mut memory = Memory::default();
        memory.last.pos = [0.5, 64.0, 0.5];
        memory.trail = (0..10).map(|i| [0, 64, -20 + i * 2]).collect();
        memory.trunks = vec![[0, 64, -9]];
        let came = memory.way_they_came().unwrap();
        assert!(came[2] <= -8 && came[2] >= -14, "{came:?}");
        assert!(distance(came, [0, 64, -9]) >= 2.0, "not on a trunk");
    }

    #[test]
    fn the_blazes_go_straight_on_ahead_of_them_spaced_apart() {
        let ahead = vec![
            [0, 64, 6],
            [1, 64, 8],
            [0, 64, 12],
            [9, 64, 13],
            [0, 64, 19],
        ];
        let line = blaze_line(&ahead, [0.5, 64.0, 0.5], 0.0, 5);
        assert_eq!(line, vec![[0, 64, 6], [0, 64, 12], [0, 64, 19]], "{line:?}");
    }

    #[test]
    fn behind_them_a_way_of_trunks_nearer_and_nearer() {
        let mut memory = Memory::default();
        memory.last.online = true;
        memory.last.pos = [0.5, 64.0, 0.5];
        // Looking south (+z): behind is north.
        memory.trunks = vec![
            [0, 64, -24],
            [1, 64, -20],
            [0, 64, -15],
            [0, 64, -9],
            [0, 64, 10],
        ];
        let behind = memory.trunks_behind(6.0, 30.0);
        assert_eq!(behind.first(), Some(&[0, 64, -24]), "farthest first");
        assert!(!behind.contains(&[0, 64, 10]), "none ahead");
        let way = nearer_and_nearer(&behind, memory.last.pos, 4);
        assert_eq!(
            way,
            vec![[0, 64, -24], [0, 64, -15], [0, 64, -9]],
            "four nearer each"
        );
    }

    #[test]
    fn a_column_of_three_logs_is_a_trunk_and_a_wall_of_theirs_is_not() {
        let logs = vec![
            [0, 64, 0],
            [0, 65, 0],
            [0, 66, 0],
            [5, 70, 5],
            [5, 71, 5],
            [9, 64, 9],
            [9, 65, 9],
            [9, 66, 9],
        ];
        let logs: Vec<(String, BlockPos)> = logs
            .into_iter()
            .map(|p| ("minecraft:birch_log".to_string(), p))
            .collect();
        let trunks = trunks_of(&logs, &[[9, 65, 9]]);
        assert_eq!(
            trunks,
            vec![([0, 64, 0], "minecraft:birch_log".to_string())],
            "two logs are a branch; their wall is theirs"
        );
        assert_eq!(
            stripped("minecraft:birch_log").as_deref(),
            Some("minecraft:stripped_birch_log")
        );
        assert_eq!(stripped("minecraft:stripped_birch_log"), None);
        assert!(wooded("minecraft:old_growth_spruce_taiga"));
        assert!(wooded("minecraft:dark_forest"));
        assert!(!wooded("minecraft:plains"));
    }

    #[test]
    fn a_peeking_figure_stands_behind_the_trunk_half_out_to_one_side() {
        // They stand ten blocks south of a trunk at the origin, looking north.
        let (peek, hide) = peek_spots([0, 64, 0], [0.5, 64.0, 10.5], 1.0);
        assert!(hide[2] < 0.5, "behind the trunk from them: {hide:?}");
        assert!((hide[0] - 0.5).abs() < 1e-9, "straight behind: {hide:?}");
        assert!(
            (peek[0] - 0.5).abs() > 0.5,
            "half past the bark, to the side: {peek:?}"
        );
        assert_eq!(peek[1], 64.0);
    }

    #[test]
    fn the_woods_are_trunks_around_them_and_ahead_is_where_they_look() {
        let mut memory = Memory::default();
        memory.last.online = true;
        memory.last.pos = [0.5, 64.0, 0.5];
        // Looking south (+z): yaw 0.
        memory.trunks = (0..8).map(|i| [i - 4, 64, 10]).collect();
        assert!(memory.in_the_woods());
        memory.trunks.push([0, 64, -12]);
        let ahead = memory.trunks_ahead(5.0, 20.0);
        assert!(!ahead.is_empty());
        assert!(
            ahead.iter().all(|t| t[2] == 10),
            "none behind them: {ahead:?}"
        );
        memory.last.indoors = true;
        assert!(!memory.in_the_woods());
    }
}
