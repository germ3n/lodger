package craftkit.host.client;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class HandPrintTest {
	@Test
	void two_hands_lie_on_the_pane_across_the_way_they_look_a_touch_toward_them() {
		// They look north (-z) at it: the hands spread along x, set a touch south, toward them.
		List<double[]> points = HandPrint.points(0, -3);
		assertTrue(points.size() > 30, "two hands of dots: " + points.size());
		for (double[] p : points) {
			assertEquals(HandPrint.TOWARD, p[2], 1e-9, "on one plane, toward them");
			assertTrue(Math.abs(p[0]) < 0.45 && p[1] > 0.3 && p[1] < 0.9, "on the pane: " + p[0] + ", " + p[1]);
		}
		assertTrue(points.stream().anyMatch(p -> p[0] < 0) && points.stream().anyMatch(p -> p[0] > 0), "two hands, side by side");
		assertTrue(HandPrint.points(0, 0).isEmpty(), "inside the pane: nothing");
	}
}
