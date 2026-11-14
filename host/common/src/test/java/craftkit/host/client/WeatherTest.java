package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;

class WeatherTest {
	@Test
	void the_dry_column_is_the_one_at_their_right_hand() {
		// Facing south (+z), their right is west.
		assertArrayEquals(new int[] {-1, 0}, Weather.beside(0.5, 0.5, 0));
		// Facing west (-x), their right is north.
		assertArrayEquals(new int[] {0, -1}, Weather.beside(0.5, 0.5, 90));
		// Facing north, east.
		assertArrayEquals(new int[] {1, 0}, Weather.beside(0.5, 0.5, 180));
	}
}
