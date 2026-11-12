package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SweepsTest {
	@Test
	void each_sweep_runs_once_in_five_seconds_and_never_two_on_one_tick() {
		for (long tick = 0; tick < 1000; tick++) {
			int due = 0;
			for (Sweeps.Sweep sweep : Sweeps.Sweep.values()) {
				if (Sweeps.due(sweep, tick)) {
					due++;
				}
			}
			assertTrue(due <= 1, "two sweeps on tick " + tick);
		}
		for (Sweeps.Sweep sweep : Sweeps.Sweep.values()) {
			int runs = 0;
			for (long tick = 0; tick < Sweeps.EVERY; tick++) {
				if (Sweeps.due(sweep, tick)) {
					runs++;
				}
			}
			assertEquals(1, runs, sweep + " once in five seconds");
		}
	}
}
