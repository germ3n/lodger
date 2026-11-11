package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class UnseenTest {
	@Test
	void it_is_seen_on_their_screen_with_nothing_between_and_not_behind_them_or_a_wall() {
		assertTrue(Unseen.seen(0.9, true));
		assertTrue(Unseen.seen(0.5, true), "at the screen's side still seen");
		assertFalse(Unseen.seen(-0.2, true), "behind them");
		assertFalse(Unseen.seen(0.95, false), "a wall between");
	}

	@Test
	void unseen_it_steps_straight_at_them_and_stops_within_reach() {
		double[] at = Unseen.step(0, 0, 10, 0);
		assertEquals(Unseen.STEP, at[0], 1e-9);
		assertEquals(0, at[1], 1e-9);
		double x = 0;
		int steps = 0;
		while (true) {
			double[] next = Unseen.step(x, 0, 10, 0);
			if (next == null) {
				break;
			}
			assertTrue(next[0] > x, "always nearer");
			assertTrue(10 - next[0] >= Unseen.REACH * 0.9 - 1e-9, "never into them");
			x = next[0];
			assertTrue(++steps < 100);
		}
		assertNull(Unseen.step(9, 0, 10, 0), "within reach: no step, the strike");
	}
}
