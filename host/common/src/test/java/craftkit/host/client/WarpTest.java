package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class WarpTest {
	@Test
	void each_part_is_bent_its_own_way_the_same_every_frame_and_within_bounds() {
		assertArrayEquals(Warp.of(1234, 3), Warp.of(1234, 3), "fixed: it never flickers");
		boolean differ = false;
		for (int part = 1; part < 12; part++) {
			float[] w = Warp.of(77, part);
			assertTrue(w[1] >= 0.55f && w[1] <= 2.4f, "along: " + w[1]);
			assertTrue(w[0] >= 0.6f && w[0] <= 1.5f && w[2] >= 0.6f && w[2] <= 1.5f, "across");
			for (int r = 3; r < 6; r++) {
				assertTrue(Math.abs(w[r]) <= 0.8f, "turned at most 0.8: " + w[r]);
			}
			differ |= w[1] != Warp.of(77, 1)[1];
		}
		assertTrue(differ, "parts differ from each other");
		float[] body = Warp.body(5);
		assertTrue(body[0] > 0 && body[1] > 0);
	}
}
