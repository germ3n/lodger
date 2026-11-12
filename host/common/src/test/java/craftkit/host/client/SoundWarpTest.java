package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class SoundWarpTest {
	@Test
	void four_ways_and_only_the_slow_one_pitches_down() {
		assertEquals(SoundWarp.Way.HALL, SoundWarp.of("hall"));
		assertEquals(SoundWarp.Way.BROKEN, SoundWarp.of(" broken "));
		assertNull(SoundWarp.of("loud"));
		assertEquals(0.55f, SoundWarp.pitch(SoundWarp.Way.SLOW, 1f), 1e-6);
		assertEquals(1.2f, SoundWarp.pitch(SoundWarp.Way.HALL, 1.2f), 1e-6);
		assertEquals(1f, SoundWarp.pitch(null, 1f), 1e-6);
	}
}
