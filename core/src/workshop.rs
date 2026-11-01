//! Furnaces, workstations and redstone (batch 9, owner 2026-11-10): what the jukebox, the
//! furnace, the brewing stand and the crafting table events need to know from the memory: a
//! disc in their chest, a fair swap of fuel by burn time, an ingredient to brew, a recipe they
//! have the makings of, and the lamps' blink.

use craftkit::world::ItemStack;

use crate::memory::{Container, Memory};

/// Burn time in ticks of the fuels a player keeps, for a swap kept fair.
pub fn burn_ticks(id: &str) -> Option<u32> {
    let name = id.rsplit(':').next().unwrap_or(id);
    Some(match name {
        "coal" | "charcoal" => 1_600,
        "coal_block" => 16_000,
        "lava_bucket" => 20_000,
        "blaze_rod" => 2_400,
        "dried_kelp_block" => 4_001,
        "stick" | "bamboo" => 100,
        n if n.ends_with("_planks") || n.ends_with("_log") || n.ends_with("_wood") => 300,
        _ => return None,
    })
}

/// The ingredients a brewing stand takes from a chest: nether wart first, the base of most.
pub const BREWING: &[&str] = &[
    "minecraft:nether_wart",
    "minecraft:sugar",
    "minecraft:redstone",
    "minecraft:glowstone_dust",
    "minecraft:spider_eye",
    "minecraft:fermented_spider_eye",
    "minecraft:magma_cream",
    "minecraft:golden_carrot",
    "minecraft:glistering_melon_slice",
    "minecraft:rabbit_foot",
    "minecraft:ghast_tear",
    "minecraft:phantom_membrane",
    "minecraft:pufferfish",
    "minecraft:blaze_powder",
    "minecraft:gunpowder",
];

/// What it crafts for them, if they have the makings: the makings (id, count) and the result
/// (id, count). Torches first, then bread, planks, sticks.
pub struct Recipe {
    pub makings: &'static [(&'static str, u32)],
    pub result: (&'static str, u32),
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        makings: &[("minecraft:stick", 1), ("minecraft:coal", 1)],
        result: ("minecraft:torch", 4),
    },
    Recipe {
        makings: &[("minecraft:stick", 1), ("minecraft:charcoal", 1)],
        result: ("minecraft:torch", 4),
    },
    Recipe {
        makings: &[("minecraft:wheat", 3)],
        result: ("minecraft:bread", 1),
    },
    Recipe {
        makings: &[("minecraft:oak_log", 1)],
        result: ("minecraft:oak_planks", 4),
    },
    Recipe {
        makings: &[("minecraft:spruce_log", 1)],
        result: ("minecraft:spruce_planks", 4),
    },
    Recipe {
        makings: &[("minecraft:birch_log", 1)],
        result: ("minecraft:birch_planks", 4),
    },
    Recipe {
        makings: &[("minecraft:oak_planks", 2)],
        result: ("minecraft:stick", 4),
    },
];

/// A recipe at hand: the recipe, the chest, and each making's stack with the count it takes.
pub type AtHand<'a> = (&'static Recipe, &'a Container, Vec<(&'a ItemStack, u32)>);

/// The lamps' blink, as toggles a tick apart: three short, three long, three short. Every
/// blink is two toggles, so each lamp ends as it was.
pub fn blink_ticks() -> Vec<u64> {
    let mut at = 0u64;
    let mut ticks = Vec::new();
    for on in [4u64, 4, 4, 12, 12, 12, 4, 4, 4] {
        ticks.push(at);
        ticks.push(at + on);
        at += on + 4;
    }

    ticks
}

impl Memory {
    /// A music disc in the chest they use, and where.
    pub fn disc_in_chest(&self) -> Option<(&Container, &ItemStack)> {
        self.in_used_container(|id| id.starts_with("minecraft:music_disc"))
    }

    /// A fair swap of their furnace's fuel: the furnace, its fuel, and a stack of another fuel
    /// in their chest with the count that burns as long (at least one, at most what is there).
    pub fn fuel_swap(&self) -> Option<(&Container, &ItemStack, &Container, &ItemStack, u32)> {
        let furnace = self.furnace()?;
        let fuel = furnace.contents.iter().find(|s| s.slot == 1)?;
        let burns = burn_ticks(&fuel.id)? * fuel.count;
        let chest = self.used_container()?;
        let other = chest
            .contents
            .iter()
            .find(|s| s.id != fuel.id && burn_ticks(&s.id).is_some())?;
        let each = burn_ticks(&other.id)?;
        let count = (burns as f64 / each as f64).round().max(1.0) as u32;

        Some((furnace, fuel, chest, other, count.min(other.count).min(64)))
    }

    /// A brewing ingredient in the chest they use.
    pub fn brewing_ingredient(&self) -> Option<(&Container, &ItemStack)> {
        BREWING
            .iter()
            .find_map(|i| self.in_used_container(|id| id == *i))
    }

    /// The first recipe they have every making of in the chest they use, with the stacks.
    pub fn recipe_at_hand(&self) -> Option<AtHand<'_>> {
        let chest = self.used_container()?;
        RECIPES.iter().find_map(|r| {
            let stacks: Option<Vec<(&ItemStack, u32)>> = r
                .makings
                .iter()
                .map(|(id, n)| {
                    chest
                        .contents
                        .iter()
                        .find(|s| s.id == *id && s.count >= *n)
                        .map(|s| (s, *n))
                })
                .collect();
            stacks.map(|s| (r, chest, s))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: &str, count: u32, slot: u32) -> ItemStack {
        ItemStack {
            id: id.to_string(),
            count,
            slot,
        }
    }

    fn memory(chest: Vec<ItemStack>, furnace: Vec<ItemStack>) -> Memory {
        Memory {
            containers: vec![
                Container {
                    pos: [0, 64, 0],
                    kind: "minecraft:chest".to_string(),
                    opens: 5,
                    placed: true,
                    contents: chest,
                    ..Container::default()
                },
                Container {
                    pos: [2, 64, 0],
                    kind: "minecraft:furnace".to_string(),
                    opens: 2,
                    placed: true,
                    contents: furnace,
                    ..Container::default()
                },
            ],
            ..Memory::default()
        }
    }

    #[test]
    fn a_fuel_swap_burns_as_long_as_what_it_took() {
        let memory = memory(
            vec![stack("minecraft:oak_planks", 64, 3)],
            vec![
                stack("minecraft:raw_iron", 4, 0),
                stack("minecraft:coal", 3, 1),
            ],
        );
        let (furnace, fuel, chest, other, count) = memory.fuel_swap().unwrap();
        assert_eq!(furnace.pos, [2, 64, 0]);
        assert_eq!(fuel.id, "minecraft:coal");
        assert_eq!(chest.pos, [0, 64, 0]);
        assert_eq!(other.id, "minecraft:oak_planks");
        assert_eq!(count, 16, "three coal burn as long as sixteen planks");

        let same = super::tests::memory(
            vec![stack("minecraft:coal", 9, 3)],
            vec![stack("minecraft:coal", 3, 1)],
        );
        assert!(same.fuel_swap().is_none(), "the same fuel is no swap");
    }

    #[test]
    fn a_disc_an_ingredient_and_a_recipe_are_found_in_their_chest() {
        let memory = memory(
            vec![
                stack("minecraft:music_disc_cat", 1, 0),
                stack("minecraft:sugar", 3, 1),
                stack("minecraft:stick", 5, 2),
                stack("minecraft:coal", 2, 4),
            ],
            vec![],
        );
        assert_eq!(memory.disc_in_chest().unwrap().1.slot, 0);
        assert_eq!(memory.brewing_ingredient().unwrap().1.id, "minecraft:sugar");
        let (recipe, _, stacks) = memory.recipe_at_hand().unwrap();
        assert_eq!(recipe.result, ("minecraft:torch", 4));
        assert_eq!(stacks.len(), 2);

        let poor = super::tests::memory(vec![stack("minecraft:wheat", 2, 0)], vec![]);
        assert!(poor.recipe_at_hand().is_none(), "two wheat make no bread");
    }

    #[test]
    fn the_blink_is_nine_flashes_and_leaves_each_lamp_as_it_was() {
        let ticks = blink_ticks();
        assert_eq!(ticks.len(), 18, "an even number of toggles");
        assert!(ticks.windows(2).all(|w| w[1] > w[0]));
        assert_eq!(ticks[7] - ticks[6], 12, "the fourth flash is long");
    }
}
