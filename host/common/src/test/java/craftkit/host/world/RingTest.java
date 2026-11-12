package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class RingTest {
	@Test
	void they_circle_nearer_each_round_and_then_run() {
		assertEquals(Ring.FROM, Ring.radius(0), 1e-9);
		assertTrue(Ring.radius(0.4) < Ring.radius(0.2), "nearer as it goes");
		assertEquals(Ring.TO, Ring.radius(Ring.CIRCLING), 1e-9);
		assertFalse(Ring.running(0.5));
		assertTrue(Ring.running(0.9));
		assertTrue(Ring.angle(0, 4, 1) > Ring.angle(0, 4, 0), "round they go");
		assertEquals(Math.PI / 2, Ring.angle(1, 4, 0), 1e-9, "spread round the ring");
	}
}
