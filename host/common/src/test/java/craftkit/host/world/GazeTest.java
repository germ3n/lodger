package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class GazeTest {
	@Test
	void a_glance_is_not_a_look_and_half_a_second_facing_it_is() {
		Gaze gaze = new Gaze();
		for (int i = 0; i < 9; i++) {
			gaze.tick(true, 30);
		}
		gaze.tick(false, 30);
		assertEquals(0, gaze.looks(), "nine ticks: a glance");

		for (int i = 0; i < 30; i++) {
			gaze.tick(true, 30);
		}
		assertEquals(1, gaze.looks());
		assertEquals(30, gaze.lookedTicks(), "the whole look counts, its first half second too");
	}

	@Test
	void facing_is_within_ten_degrees() {
		assertTrue(Gaze.facing(Math.cos(Math.toRadians(9))));
		assertFalse(Gaze.facing(Math.cos(Math.toRadians(12))));
	}

	@Test
	void walking_toward_it_is_the_distance_closed_and_the_detail_says_how_it_ended() {
		Gaze gaze = new Gaze();
		for (int d = 30; d >= 18; d--) {
			gaze.tick(true, d);
		}
		gaze.tick(false, 25);

		assertEquals(12.0, gaze.toward(), 1e-9);
		assertEquals("looked=13 looks=1 longest=13 toward=12.0 end=approached", gaze.detail("approached"));
	}
}
