package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

class StrideTest {
	@Test
	void a_step_each_stride_walked_none_crouching_or_in_the_air() {
		Stride stride = new Stride();
		int steps = 0;
		// Twelve blocks at a walk, a fifth of a block a tick (not ten: six strides is ten to the
		// last float, a step either side of it).
		for (int i = 0; i < 60; i++) {
			if (stride.moved(0.2, 0, true, false)) {
				steps++;
			}
		}
		assertEquals(7, steps, "a step each 1/0.6 blocks");

		Stride quiet = new Stride();
		for (int i = 0; i < 50; i++) {
			assertFalse(quiet.moved(0.2, 0, true, true), "crouching");
			assertFalse(quiet.moved(0.2, 0, false, false), "in the air");
		}
		assertFalse(quiet.moved(0, 0, true, false), "standing still");
	}

	@Test
	void the_step_behind_is_back_along_their_way() {
		assertArrayEquals(new double[] {7, 0}, Stride.behind(10, 0, 0.2, 0, 3), 1e-9);
		assertArrayEquals(new double[] {5, 5}, Stride.behind(5, 5, 0, 0, 3), 1e-9, "standing: where they are");
	}
}
