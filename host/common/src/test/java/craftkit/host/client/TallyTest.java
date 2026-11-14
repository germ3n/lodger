package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class TallyTest {
	@Test
	void strokes_in_fives_the_fifth_across_and_never_past_the_most() {
		assertEquals(0, Tally.points(0).size());
		int one = Tally.points(1).size();
		assertEquals(4 * one, Tally.points(4).size(), "four uprights");
		assertTrue(Tally.points(5).size() > Tally.points(4).size(), "the fifth across them");
		assertEquals(Tally.points(Tally.MOST).size(), Tally.points(Tally.MOST + 30).size());
		for (double[] p : Tally.points(Tally.MOST)) {
			assertTrue(Math.abs(p[0]) <= 1.2 && p[1] > -3 && p[1] < 1.1, "on the wall: " + p[0] + "," + p[1]);
		}
	}
}
