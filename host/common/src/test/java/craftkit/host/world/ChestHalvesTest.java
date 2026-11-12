package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ChestHalvesTest {
	@Test
	void a_double_chest_is_known_by_the_same_half_whichever_is_clicked() {
		// Side by side along x: the west half.
		assertTrue(ChestHalves.known(-403, -374, -402, -374));
		assertFalse(ChestHalves.known(-402, -374, -403, -374));
		// Along z: the north half.
		assertTrue(ChestHalves.known(5, 1, 5, 2));
		assertFalse(ChestHalves.known(5, 2, 5, 1));
		// Exactly one of the two, either way round.
		for (int[] pair : new int[][] {{0, 0, 1, 0}, {3, 7, 3, 6}, {-1, -1, -2, -1}}) {
			assertTrue(ChestHalves.known(pair[0], pair[1], pair[2], pair[3]) != ChestHalves.known(pair[2], pair[3], pair[0], pair[1]));
		}
	}
}
