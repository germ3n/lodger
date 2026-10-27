package craftkit.host.client.debug;

import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class CoordinatesTest {
	@Test
	void the_first_spot_in_a_line_is_found_and_ground_is_the_surface() {
		Coordinates.Spot spot = Coordinates.find("#3 close_door at (-72,69,28)").orElseThrow();
		assertEquals(-72, spot.x());
		assertEquals(Optional.of(69), spot.y());
		assertEquals("[-72,69,28]", spot.json());

		Coordinates.Spot far = Coordinates.find("chose far_figure at (440,ground,-32) then (1,2,3)").orElseThrow();
		assertTrue(far.y().isEmpty());
		assertEquals("[440,null,-32]", far.json());
	}

	@Test
	void a_line_without_a_spot_has_none() {
		assertTrue(Coordinates.find("nothing planned").isEmpty());
		assertTrue(Coordinates.find("a day (1.5) later").isEmpty());
	}
}
