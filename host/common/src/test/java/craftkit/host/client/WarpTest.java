package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class WarpTest {
	@Test
	void the_stretch_grows_in_eased_and_is_whole_at_its_time() {
		assertEquals(0f, Warp.grown(0));
		assertEquals(1f, Warp.grown(Warp.GROW_MS));
		assertEquals(1f, Warp.grown(Warp.GROW_MS * 10));
		assertTrue(Warp.grown(Warp.GROW_MS / 10) < 0.1f, "slow to start");
		float[] start = Warp.body(5, 0, 0);
		assertEquals(1f, start[0], 1e-6);
		assertEquals(1f, start[1], 1e-6, "as it was until it grows");
	}

	@Test
	void the_body_stretches_whole_its_volume_kept_tall_or_low() {
		boolean sawTall = false;
		boolean sawLow = false;
		for (int id = 0; id < 64; id++) {
			for (long now = 0; now < 4000; now += 333) {
				float[] b = Warp.body(id, Warp.GROW_MS, now);
				assertEquals(1f, b[0] * b[0] * b[1], 1e-4, "volume kept: " + b[0] + " " + b[1]);
				if (Warp.tall(id)) {
					assertTrue(b[1] > 1.6f && b[0] < 0.8f, "tall and thin");
					sawTall = true;
				} else {
					assertTrue(b[1] < 0.7f && b[0] > 1.15f, "low and wide");
					sawLow = true;
				}
			}
		}
		assertTrue(sawTall && sawLow);
		assertNotEquals(Warp.body(7, Warp.GROW_MS, 0)[1], Warp.body(7, Warp.GROW_MS, 500)[1], "it breathes");
		float[] asked = Warp.body(7, Warp.GROW_MS, 0, 2.0f);
		assertEquals(2.0f, asked[1], 2.0f * Warp.BREATH + 1e-4, "to the height the mind chose");
		assertEquals(1f, asked[0] * asked[0] * asked[1], 1e-4);
	}

	@Test
	void every_part_trembles_a_little_and_the_head_tilts_and_twitches() {
		boolean moved = false;
		float last = Warp.tremor(9, 2, 0, Warp.GROW_MS)[0];
		for (long now = 0; now < 2000; now += 17) {
			float[] t = Warp.tremor(9, 2, now, Warp.GROW_MS);
			for (float turn : t) {
				assertTrue(Math.abs(turn) <= Warp.TREMOR + 1e-6, "a tremor, not a break: " + turn);
			}
			moved |= t[0] != last;
			last = t[0];
		}
		assertTrue(moved, "it shakes");
		assertEquals(0f, Warp.tremor(9, 2, 100, 0)[0], 1e-6, "still before it grows");

		float most = 0;
		float least = Float.MAX_VALUE;
		for (long now = 0; now < Warp.TWITCH_EVERY_MS; now += 5) {
			float tilt = Math.abs(Warp.headTilt(3, now, Warp.GROW_MS));
			most = Math.max(most, tilt);
			least = Math.min(least, tilt);
		}
		assertEquals(Warp.TILT, least, 1e-4, "tilted over all the time");
		assertTrue(most > Warp.TILT + Warp.TWITCH * 0.9f, "and a twitch's snap once a while: " + most);
	}
}
