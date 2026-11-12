package craftkit.host.world;

/**
 * A tree brought down (`fell_tree`, ABI 54), apart from the game so JUnit checks it: where each of
 * its logs comes to lie, a log a block above the stump that many blocks out along the fall, as a
 * tree lies, and whether a place is too near the player to land a log on.
 */
public final class Felled {
	/** No log lands nearer the player than this, along the ground. */
	public static final double CLEAR = 2.0;

	private Felled() {
	}

	/** The way it falls, one step on the axis nearer {@code (dx, dz)}: {x, z}. */
	public static int[] way(double dx, double dz) {
		if (Math.abs(dx) >= Math.abs(dz)) {
			return new int[] {dx >= 0 ? 1 : -1, 0};
		}
		return new int[] {0, dz >= 0 ? 1 : -1};
	}

	/** Where the log {@code up} blocks over the stump lies: that many out along the way, at its own height then fallen. {dx, dz}. */
	public static int[] lies(int up, int[] way) {
		return new int[] {way[0] * up, way[1] * up};
	}

	/** Whether a block at {@code (x, z)} is too near a player at {@code (px, pz)} to land a log on. */
	public static boolean nearThem(int x, int z, double px, double pz) {
		double dx = x + 0.5 - px;
		double dz = z + 0.5 - pz;
		return Math.sqrt(dx * dx + dz * dz) < CLEAR;
	}
}
