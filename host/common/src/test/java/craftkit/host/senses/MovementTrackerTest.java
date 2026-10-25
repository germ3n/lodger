package craftkit.host.senses;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MovementTrackerTest {
	private static MovementTracker walk(int ticks, double speedPerTick, float yaw) {
		MovementTracker tracker = new MovementTracker();
		double z = 0;
		for (int i = 0; i < ticks; i++) {
			z += speedPerTick;
			tracker.sample(0, 64, z, yaw, false);
		}
		return tracker;
	}

	@Test
	void a_steady_walk_has_a_pace_and_no_stops_or_turns() {
		MovementTracker tracker = walk(200, 0.2, 0);

		assertEquals(4.0, tracker.pace(), 0.05, "0.2 blocks a tick is 4 a second");
		assertEquals(0, tracker.stops());
		assertEquals(0, tracker.turns());
		assertEquals(0, tracker.lookBacks());
		assertEquals(0, tracker.stillSeconds(), 1e-6);
	}

	@Test
	void standing_still_counts_seconds_and_one_stop() {
		MovementTracker tracker = walk(40, 0.2, 0);
		// Six seconds still: longer than the five the pace is read over.
		for (int i = 0; i < 120; i++) {
			tracker.sample(0, 64, 8, 0, false);
		}

		assertEquals(1, tracker.stops());
		assertEquals(6.0, tracker.stillSeconds(), 1e-6);
		assertEquals(0.0, tracker.pace(), 0.01, "nothing moved in the last five seconds");
	}

	@Test
	void a_turn_and_back_within_two_seconds_is_a_look_back() {
		MovementTracker tracker = walk(20, 0.2, 0);
		for (int i = 1; i <= 5; i++) {
			tracker.sample(0, 64, 4, 36f * i, false);
		}
		for (int i = 0; i < 10; i++) {
			tracker.sample(0, 64, 4, 180, false);
		}
		for (int i = 1; i <= 5; i++) {
			tracker.sample(0, 64, 4, 180 - 36f * i, false);
		}

		assertEquals(2, tracker.turns());
		assertEquals(1, tracker.lookBacks());
	}

	@Test
	void walking_against_the_facing_is_backing_away() {
		MovementTracker tracker = new MovementTracker();
		double z = 0;
		for (int i = 0; i < 40; i++) {
			z += 0.1;
			tracker.sample(0, 64, z, 180, false);
		}

		assertEquals(39 / 20f, tracker.backingAway(), 1e-6);
	}

	@Test
	void sprint_starts_are_bursts_and_old_ones_expire() {
		MovementTracker tracker = new MovementTracker();
		for (int i = 0; i < 10; i++) {
			tracker.sample(0, 64, i * 0.1, 0, false);
		}
		for (int i = 0; i < 10; i++) {
			tracker.sample(0, 64, 1 + i * 0.3, 0, true);
		}
		assertEquals(1, tracker.sprintBursts());

		for (int i = 0; i < 61 * 20; i++) {
			tracker.sample(0, 64, 4 + i * 0.1, 0, false);
		}
		assertEquals(0, tracker.sprintBursts(), "a minute later it is forgotten");
	}

	@Test
	void yaw_differences_wrap_around() {
		assertEquals(20f, MovementTracker.yawDelta(10, -10), 1e-6);
		assertEquals(-20f, MovementTracker.yawDelta(170, -170), 1e-6);
		assertTrue(Math.abs(MovementTracker.yawDelta(179, -179)) < 3);
	}
}
