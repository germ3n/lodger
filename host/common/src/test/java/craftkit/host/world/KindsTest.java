package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class KindsTest {
	@Test
	void a_kind_is_an_exact_id_or_a_suffix_after_a_star() {
		List<String> kinds = List.of("minecraft:chest", "*_stairs", "*_door");
		assertTrue(Kinds.wanted("minecraft:chest", kinds));
		assertFalse(Kinds.wanted("minecraft:trapped_chest", kinds), "exact means exact");
		assertTrue(Kinds.wanted("minecraft:spruce_stairs", kinds));
		assertTrue(Kinds.wanted("minecraft:iron_door", kinds));
		assertFalse(Kinds.wanted("minecraft:trapdoor", kinds), "a trapdoor is not a _door");
	}
}
