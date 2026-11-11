package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MapSpotTest {
	@Test
	void a_place_falls_on_the_map_two_to_a_pixel_from_its_middle() {
		// A map of scale 0 centred on 64, 64: a block is a pixel, a marker two units to one.
		assertArrayEquals(new byte[] {0, 0}, MapSpot.spot(64, 64, (byte) 0, 64, 64).orElseThrow());
		// The game rounds as it does: (int) (d * 2 + 0.5), so ten blocks north is -19, not -20.
		assertArrayEquals(new byte[] {20, -19}, MapSpot.spot(64, 64, (byte) 0, 74, 54).orElseThrow());
		// Scale 2: four blocks a pixel.
		assertArrayEquals(new byte[] {10, 0}, MapSpot.spot(0, 0, (byte) 2, 20, 0).orElseThrow());
		assertTrue(MapSpot.spot(64, 64, (byte) 0, 64 + 70, 64).isEmpty(), "off the map");
	}
}
