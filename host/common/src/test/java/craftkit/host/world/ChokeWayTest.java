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
}
