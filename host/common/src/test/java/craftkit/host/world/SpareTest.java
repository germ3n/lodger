package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SpareTest {
	@Test
	void a_spared_hit_leaves_them_above_half_a_heart_after_the_game_scales_it() {
		// Hard: three health, a hit of four asked. Scaled half again it must land at most two.
		float harm = Spare.harm(4, 3, 1.5f);
		assertTrue(3 - harm * 1.5f >= 1 - 1e-6, "left with " + (3 - harm * 1.5f));
		assertEquals(4, Spare.harm(4, 20, 1.5f), "plenty of health: the hit as asked");
		assertEquals(0, Spare.harm(4, 1, 1.0f), "at half a heart: nothing");
		assertEquals(2, Spare.harm(4, 3, 1.0f), 1e-6);
	}
}
