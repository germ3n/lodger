package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;

class SlotsTest {
	@Test
	void the_slot_asked_for_comes_first_then_the_rest_in_order() {
		assertArrayEquals(new int[] {2, 0, 1, 3}, Slots.order(2, 4));
		assertArrayEquals(new int[] {0, 1, 2}, Slots.order(0, 3));
		assertArrayEquals(new int[] {0, 1, 2}, Slots.order(9, 3), "a slot that is not there");
	}
}
