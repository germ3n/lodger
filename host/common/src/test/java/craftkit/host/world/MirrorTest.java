package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MirrorTest {
	@Test
	void it_moves_as_far_as_they_have_from_where_it_was_shown() {
		assertArrayEquals(new double[] {13, -2}, Mirror.to(10, 0, 100, 50, 103, 48), 1e-9);
		assertArrayEquals(new double[] {10, 0}, Mirror.to(10, 0, 100, 50, 100, 50), 1e-9);
	}

	@Test
	void every_tick_of_a_walk_or_a_sneak_moves_it_and_standing_still_does_not() {
		// A tick's walk (0.216) and a tick's sneak (0.065): each one sent, so it glides with them.
		assertTrue(Mirror.moves(0, 0, 0.216, 0));
		assertTrue(Mirror.moves(0, 0, 0, 0.065));
		assertFalse(Mirror.moves(5, 5, 5.001, 5.001));
	}
}
