package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FelledTest {
	@Test
	void it_falls_along_one_axis_each_log_its_height_out_and_never_onto_them() {
		assertArrayEquals(new int[] {1, 0}, Felled.way(5, 2));
		assertArrayEquals(new int[] {0, -1}, Felled.way(1, -4));
		assertArrayEquals(new int[] {4, 0}, Felled.lies(4, new int[] {1, 0}));
		assertTrue(Felled.nearThem(3, 0, 3.5, 0.9));
		assertFalse(Felled.nearThem(6, 0, 3.5, 0.5));
	}
}
