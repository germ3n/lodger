package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TowardTest {
	@Test
	void the_nearest_few_set_off_farthest_first_coming_toward_them() {
		List<double[]> points = List.of(new double[] {1, 0}, new double[] {9, 0}, new double[] {4, 0}, new double[] {20, 0}, new double[] {2, 0});
		// The four nearest of five, the farthest of them first: 9, 4, 2, 1.
		assertEquals(List.of(1, 2, 4, 0), Toward.order(points, 0, 0, 4));
		assertEquals(List.of(), Toward.order(List.of(), 0, 0, 4));
		assertEquals(List.of(0), Toward.order(points, 0, 0, 1));
	}
}
