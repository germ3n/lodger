package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class WarpMathTest {
	@Test
	void how_it_was_goes_into_its_tag_and_comes_back() {
		WarpMath.Was was = new WarpMath.Was(-132.5, 64.0, 17.25, 90.5f, 1.0, false);
		String tag = WarpMath.tag(was);

		assertTrue(tag.startsWith(WarpMath.FROM));
		assertEquals(was, WarpMath.read(tag).orElseThrow());
		assertTrue(WarpMath.read("craftkit_held").isEmpty());
		assertTrue(WarpMath.read(WarpMath.FROM + "1,2,x").isEmpty(), "a broken tag is no warp");
	}

	@Test
	void it_is_looked_at_inside_the_cone_and_not_beside_or_behind() {
		assertTrue(WarpMath.lookedAt(0, 0, 1, 0, 0, 10));
		assertTrue(WarpMath.lookedAt(0, 0, 1, 3, 0, 10), "a little off the middle");
		assertFalse(WarpMath.lookedAt(0, 0, 1, 10, 0, 10), "45 degrees off");
		assertFalse(WarpMath.lookedAt(0, 0, 1, 0, 0, -10), "behind");
	}

	@Test
	void a_step_closer_is_a_step_and_never_onto_them() {
		assertArrayEquals(new double[] {1.5, 0}, WarpMath.stepToward(0, 0, 20, 0), 1e-9);
		assertArrayEquals(new double[] {0, 0}, WarpMath.stepToward(0, 0, 1, 0), 1e-9);
	}
}
