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
}
