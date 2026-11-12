package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FangLineTest {
	@Test
	void the_line_comes_straight_at_them_and_stops_short() {
		List<double[]> line = FangLine.cells(0, 0, 10, 0);
		assertEquals(0, line.get(0)[0], 1e-9);
		double last = line.get(line.size() - 1)[0];
		assertTrue(last <= 10 - FangLine.SHORT, "short of them: " + last);
		assertTrue(last > 10 - FangLine.SHORT - FangLine.STEP, "and no shorter than a step more");
		assertTrue(line.stream().allMatch(c -> c[1] == 0), "straight");
	}

	@Test
	void none_when_they_are_too_near_and_never_more_than_the_most() {
		assertTrue(FangLine.cells(0, 0, 1, 0).isEmpty());
		assertEquals(FangLine.MOST, FangLine.cells(0, 0, 100, 0).size());
	}
}
