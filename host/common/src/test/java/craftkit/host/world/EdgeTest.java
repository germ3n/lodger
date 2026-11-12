package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class EdgeTest {
	/** A column from the head (-1) down: 'o' open, 'g' ground, 'w' water or anything else. */
	private static Edge.Column column(String head, String down) {
		return new Edge.Column() {
			private char at(int depth) {
				if (depth < 0) {
					return head.charAt(0);
				}
				return depth < down.length() ? down.charAt(depth) : 'o';
			}

			@Override
			public boolean open(int depth) {
				return at(depth) == 'o';
			}

			@Override
			public boolean ground(int depth) {
				return at(depth) == 'g';
			}
		};
	}

	@Test
	void a_higher_push_wants_more_health_and_never_leaves_less_than_a_heart() {
		assertEquals(4f, Edge.healthFor(3), "two hearts, as ever");
		for (int most = 2; most <= 6; most++) {
			float fall = Math.max(0, most - 3);
			assertEquals(true, Edge.healthFor(most) - fall >= 2, "a heart left after a fall of " + most);
		}
	}

	@Test
	void two_or_three_blocks_down_is_a_safe_edge() {
		// Out from their feet: the feet's level open, the block they stand on's level open, one
		// more open, then ground: they land two blocks down.
		assertEquals(2, Edge.drop(column("o", "ooog"), 3));
		assertEquals(3, Edge.drop(column("o", "oooog"), 3));
	}

	@Test
	void no_fall_too_far_a_wall_or_water_is_no_edge() {
		assertEquals(-1, Edge.drop(column("o", "og"), 3), "level ground out there");
		assertEquals(-1, Edge.drop(column("o", "oog"), 3), "a step down, no fall");
		assertEquals(-1, Edge.drop(column("o", "ooooog"), 3), "four down: too far");
		assertEquals(-1, Edge.drop(column("o", "oooooooo"), 3), "no ground in reach");
		assertEquals(-1, Edge.drop(column("g", "oog"), 3), "the head blocked");
		assertEquals(-1, Edge.drop(column("o", "gooog"), 3), "the feet blocked");
		assertEquals(-1, Edge.drop(column("o", "ooow"), 3), "water below");
	}
}
