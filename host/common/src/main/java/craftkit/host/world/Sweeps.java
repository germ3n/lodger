package craftkit.host.world;

/**
 * When each whole-level sweep runs, apart from the game so JUnit checks it: every one of them
 * once in {@link #EVERY} ticks, each on its own tick of the five seconds, so no two read every
 * entity of every level in the same tick (they all fell on one, with the saves, a spike every five
 * seconds; found by the performance review, docs/performance.md).
 */
public final class Sweeps {
	public static final int EVERY = 100;

	/** The sweeps, each its own offset into the five seconds. */
	public enum Sweep {
		HOLDS(0), DEATHS(10), WARPS(20), TEMPORARIES(40), FIGHTS(60), GATHERINGS(80);

		final int offset;

		Sweep(int offset) {
			this.offset = offset;
		}
	}

	private Sweeps() {
	}

	/** Whether this sweep runs on this tick. */
	public static boolean due(Sweep sweep, long tick) {
		return Math.floorMod(tick - sweep.offset, EVERY) == 0;
	}
}
