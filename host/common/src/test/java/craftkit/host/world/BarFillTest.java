package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class BarFillTest {
	@Test
	void a_bar_fills_from_empty_to_full_over_its_time() {
		assertEquals(0f, BarFill.of(0, 1200), 1e-6);
		assertEquals(0.5f, BarFill.of(600, 1200), 1e-6);
		assertEquals(1f, BarFill.of(1300, 1200), 1e-6, "never past full");
		assertEquals(0f, BarFill.of(-5, 1200), 1e-6);
		assertEquals(1f, BarFill.of(3, 0), 1e-6);
	}
}
