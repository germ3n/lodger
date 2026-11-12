package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

class WorthTest {
	@Test
	void the_rare_and_hard_won_first_and_dirt_not_at_all() {
		assertTrue(Worth.of("minecraft:netherite_ingot", false) > Worth.of("minecraft:diamond_sword", false));
		assertTrue(Worth.of("minecraft:diamond_sword", false) > Worth.of("minecraft:iron_ingot", false));
		assertTrue(Worth.of("minecraft:iron_pickaxe", true) > Worth.of("minecraft:iron_pickaxe", false), "enchanted is worth more");
		assertTrue(Worth.of("minecraft:elytra", false) > Worth.of("minecraft:emerald", false));
		assertTrue(Worth.of("minecraft:dirt", false) < Worth.LEAST, "not worth taking");
		assertTrue(Worth.of("minecraft:cobblestone", false) < Worth.LEAST);
		assertTrue(Worth.of("minecraft:gold_ingot", false) >= Worth.LEAST);
	}
}
