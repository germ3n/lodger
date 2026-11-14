package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

class ShadowFollowTest {
	@Test
	void the_outline_lies_long_on_the_ground_away_from_the_sun() {
		// The head (1.65 up) of a shadow falling east lies 1.65 * STRETCH east of the feet.
		assertArrayEquals(new double[] {1.65 * ShadowFollow.STRETCH, 0}, ShadowFollow.ground(0, 1.65, 1, 0, 1), 1e-9);
		// Across goes to the side of the way it falls.
		assertArrayEquals(new double[] {0, 0.3}, ShadowFollow.ground(0.3, 0, 1, 0, 1), 1e-9);
		// Shrinking into the ground.
		assertArrayEquals(new double[] {0, 0}, ShadowFollow.ground(0.3, 1, 0, 1, 0), 1e-9);
		assertEquals(-1, ShadowFollow.fromSun(1000)[0], "morning: away from the eastern sun");
		assertEquals(1, ShadowFollow.fromSun(9000)[0], "afternoon: east");
	}
}
