package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class CeilingPrintsTest {
	@Test
	void prints_step_from_the_door_to_the_bed_left_and_right_facing_their_way() {
		var steps = CeilingPrints.steps(0, 0, 0, 9);
		assertEquals(11, steps.size(), "a step every 0.9 blocks over nine");
		// Walking +z: they face yaw 0, and fall left and right of the line by turns.
		assertTrue(steps.stream().allMatch(s -> Math.abs(s.yaw()) < 1e-3));
		assertTrue(steps.get(0).x() * steps.get(1).x() < 0, "left, then right");
		assertTrue(steps.stream().allMatch(s -> Math.abs(Math.abs(s.x()) - CeilingPrints.SPLAY) < 1e-9));
		assertTrue(steps.get(10).z() <= 9.0 + 1e-9);
		// A long way is cut short, and no way has none.
		assertEquals(CeilingPrints.MOST, CeilingPrints.steps(0, 0, 200, 0).size());
		assertTrue(CeilingPrints.steps(3, 3, 3, 3).isEmpty());
		// Walking -x faces yaw 90.
		assertEquals(90f, CeilingPrints.steps(0, 0, -5, 0).get(0).yaw(), 1e-3);
	}
}
