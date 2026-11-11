package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ChokeWayTest {
	@Test
	void it_stands_by_the_pillow_on_the_side_nearer_it_then_by_the_foot_then_past_it() {
		// The head at 0,0, the bed facing north (foot to head: z -1), the foot at 0,1; it comes
		// from the east.
		List<int[]> spots = ChokeWay.bedside(0, 0, 0, -1, 6.5, 0.5);
		assertEquals(5, spots.size());
		assertArrayEquals(new int[] {1, 0}, spots.get(0), "by the pillow, on its side");
		assertArrayEquals(new int[] {-1, 0}, spots.get(1));
		assertArrayEquals(new int[] {1, 1}, spots.get(2), "then beside the foot");
		assertArrayEquals(new int[] {-1, 1}, spots.get(3));
		assertArrayEquals(new int[] {0, 2}, spots.get(4), "then past the foot");

		// From the west, the west side first.
		assertArrayEquals(new int[] {-1, 0}, ChokeWay.bedside(0, 0, 0, -1, -6.5, 0.5).get(0));
		// A bed facing east (x +1): its sides are north and south.
		List<int[]> east = ChokeWay.bedside(0, 0, 1, 0, 0.5, 6.5);
		assertArrayEquals(new int[] {0, 1}, east.get(0));
		assertArrayEquals(new int[] {-2, 0}, east.get(4), "past the foot, to the west");
	}

	@Test
	void it_walks_a_stride_at_a_time_and_from_no_farther_than_seven_blocks() {
		List<double[]> near = ChokeWay.walk(0, 64, 0, 3, 64, 0);
		assertArrayEquals(new double[] {0, 64, 0}, near.get(0), 1e-9);
		assertArrayEquals(new double[] {3, 64, 0}, near.get(near.size() - 1), 1e-9);
		for (int i = 1; i < near.size(); i++) {
			double stride = near.get(i)[0] - near.get(i - 1)[0];
			assertTrue(stride > 0 && stride <= ChokeWay.STEP + 1e-9, "stride " + stride);
		}

		List<double[]> far = ChokeWay.walk(0, 64, 0, 20, 64, 0);
		assertEquals(13, far.get(0)[0], 1e-9, "begun seven blocks off");
		assertArrayEquals(new double[] {20, 64, 0}, far.get(far.size() - 1), 1e-9);
		assertTrue(far.size() <= 14, "at most a dozen strides: " + far.size());

		List<double[]> there = ChokeWay.walk(2, 64, 2, 2, 64, 2);
		assertEquals(1, there.size(), "already there: it stands");
	}

	@Test
	void its_glide_takes_a_second_a_block_and_seven_seconds_at_most() {
		assertEquals(60, ChokeWay.ticks(ChokeWay.walk(0, 64, 0, 3, 64, 0)));
		assertEquals(140, ChokeWay.ticks(ChokeWay.walk(0, 64, 0, 20, 64, 0)), "begun seven blocks off");
		assertEquals(0, ChokeWay.ticks(ChokeWay.walk(2, 64, 2, 2, 64, 2)));
	}

	@Test
	void at_the_bed_it_stands_looks_at_the_lead_then_at_them_then_chokes() {
		assertEquals(ChokeWay.Beat.STANDS, ChokeWay.beat(0));
		assertEquals(ChokeWay.Beat.LOOKS_AT_IT, ChokeWay.beat(ChokeWay.STANDS));
		assertEquals(ChokeWay.Beat.LOOKS_AT_THEM, ChokeWay.beat(ChokeWay.STANDS + ChokeWay.LOOKS_AT_IT));
		assertEquals(ChokeWay.Beat.CHOKES, ChokeWay.beat(ChokeWay.BEFORE_SQUEEZING));
		assertTrue(ChokeWay.LOOKS_AT_IT >= 30, "long enough to see it look at the lead");
	}

	@Test
	void the_rope_s_far_end_is_in_its_left_hand_whichever_way_it_faces() {
		// Facing south (yaw 0, +z): its left is east (+x), the rope a little in front.
		double[] south = ChokeWay.otherHand(0, 64, 0, 0);
		assertTrue(south[0] > 0.2 && south[2] > 0.1, south[0] + ", " + south[2]);
		assertEquals(64.85, south[1], 1e-9);
		// Facing west (yaw 90, -x): its left is south (+z), in front is west.
		double[] west = ChokeWay.otherHand(0, 64, 0, 90);
		assertTrue(west[2] > 0.2 && west[0] < -0.1, west[0] + ", " + west[2]);
	}

	@Test
	void it_looks_from_the_foot_of_the_bed_then_walks_round_to_the_pillow_and_then_chokes() {
		// The bed facing north (z -1) from its head at 0,0: the foot at 0,1, past it 0,2.
		assertArrayEquals(new int[] {0, 2}, ChokeWay.foot(0, 0, 0, -1));
		assertArrayEquals(ChokeWay.bedside(0, 0, 0, -1, 0.5, 6.5).get(4), ChokeWay.foot(0, 0, 0, -1), "past the foot");
		long toBed = 40;
		assertEquals(ChokeWay.Beat.STANDS, ChokeWay.beat(0, toBed));
		assertEquals(ChokeWay.Beat.LOOKS_AT_IT, ChokeWay.beat(ChokeWay.STANDS, toBed));
		assertEquals(ChokeWay.Beat.LOOKS_AT_THEM, ChokeWay.beat(ChokeWay.BEFORE_SQUEEZING - 1, toBed));
		assertEquals(ChokeWay.Beat.TO_THE_BED, ChokeWay.beat(ChokeWay.BEFORE_SQUEEZING, toBed), "the looking done at the foot, then the walk");
		assertEquals(ChokeWay.Beat.TO_THE_BED, ChokeWay.beat(ChokeWay.BEFORE_SQUEEZING + toBed - 1, toBed));
		assertEquals(ChokeWay.Beat.CHOKES, ChokeWay.beat(ChokeWay.BEFORE_SQUEEZING + toBed, toBed), "only then at the pillow");
		assertEquals(ChokeWay.Beat.CHOKES, ChokeWay.beat(ChokeWay.BEFORE_SQUEEZING, 0), "no walk: at once");
	}
}

