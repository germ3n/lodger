package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

class SettleTest {
	@Test
	void a_block_is_tried_where_it_was_asked_and_then_nearest_first_down_before_up() {
		assertArrayEquals(new int[] {0}, Settle.order(0));
		assertArrayEquals(new int[] {0, -1, 1, -2, 2, -3, 3}, Settle.order(3));
	}

	@Test
	void a_mod_cannot_make_the_host_search_far_or_backwards() {
		assertEquals(Settle.MOST * 2 + 1, Settle.order(200).length);
		assertArrayEquals(new int[] {0}, Settle.order(-4));
	}

	@Test
	void a_y_below_minus_a_thousand_counts_from_the_ground() {
		// The ground's own block, the air on it, the block under it, on ground at 61 and at 80.
		assertEquals(61, Settle.fromGround(61, Settle.GROUND));
		assertEquals(62, Settle.fromGround(61, Settle.GROUND + 1));
		assertEquals(59, Settle.fromGround(61, Settle.GROUND - 2));
		assertEquals(83, Settle.fromGround(80, Settle.GROUND + 3));
	}
}
