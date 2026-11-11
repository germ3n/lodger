package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class DoorOutTest {
	@Test
	void the_way_out_is_the_side_open_to_the_sky() {
		assertEquals(1, DoorOut.side(true, false));
		assertEquals(-1, DoorOut.side(false, true));
	}

	@Test
	void a_door_open_both_ways_or_roofed_both_ways_tells_nothing() {
		assertEquals(0, DoorOut.side(true, true));
		assertEquals(0, DoorOut.side(false, false));
	}
}
