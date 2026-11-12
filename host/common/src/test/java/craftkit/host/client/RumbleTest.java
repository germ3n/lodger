package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class RumbleTest {
	@Test
	void the_shake_is_hard_at_first_bounded_and_nothing_at_its_end() {
		float early = 0;
		float late = 0;
		for (long tick = 0; tick < 40; tick++) {
			float[] start = Rumble.offset(0.05f, tick);
			float[] end = Rumble.offset(0.9f, tick);
			assertTrue(Math.abs(start[0]) <= Rumble.MOST && Math.abs(start[1]) <= Rumble.MOST);
			early = Math.max(early, Math.abs(start[0]));
			late = Math.max(late, Math.abs(end[0]));
		}
		assertTrue(early > 1f, "felt at the start: " + early);
		assertTrue(late < 0.05f, "nearly still at the end: " + late);
		float[] over = Rumble.offset(1f, 7);
		assertEquals(0f, over[0], 1e-6);
		assertEquals(0f, over[1], 1e-6);
	}
}
