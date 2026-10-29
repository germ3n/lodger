package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

class MovesTest {
	@Test
	void the_spot_asked_comes_first_then_the_nearest_ring_level_up_and_down() {
		assertEquals(1, Moves.offsets(0).size(), "no search: the spot alone");
		List<int[]> one = Moves.offsets(1);
		assertArrayEquals(new int[] {0, 0, 0}, one.get(0));
		assertEquals(1 + 8 * 3, one.size(), "the first ring, at three heights");
		assertEquals(0, one.get(1)[1], "its own level first");
		assertEquals(1 + 8 * 3 + 16 * 3, Moves.offsets(2).size());
	}
}
