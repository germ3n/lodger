package craftkit.host.client;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FootPrintTest {
	@Test
	void the_detail_reads_back_and_a_broken_entry_is_skipped() {
		List<FootPrint.Print> prints = FootPrint.read("1.5,64.0,-3.2,90;bad;2,64,2,0");
		assertEquals(2, prints.size());
		assertEquals(-3.2, prints.get(0).z(), 1e-9);
		assertEquals(90f, prints.get(0).yaw(), 1e-6);
	}

	@Test
	void a_print_points_its_way_toes_in_front() {
		// Yaw 0 points +z: the toes are the dots farthest along z.
		List<double[]> dots = FootPrint.dots(0);
		double toes = dots.stream().mapToDouble(d -> d[1]).max().orElse(0);
		double heel = dots.stream().mapToDouble(d -> d[1]).min().orElse(0);
		assertTrue(toes > 0.1 && heel < -0.1);
		// Yaw 90 points -x.
		double west = FootPrint.dots(90).stream().mapToDouble(d -> d[0]).min().orElse(0);
		assertTrue(west < -0.1);
	}
}
