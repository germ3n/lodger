package craftkit.host.world;

/** Which way an entity turns to face a point: pure, so JUnit checks it without the game. */
public final class Facing {
	private Facing() {
	}

	/** The yaw, in the game's degrees (0 faces +z, 90 faces -x), that looks along dx, dz. */
	public static float yaw(double dx, double dz) {
		return (float) (Math.toDegrees(Math.atan2(dz, dx)) - 90.0);
	}

	/**
	 * Of the four sides north, south, west, east (0 to 3), the one that points most nearly along
	 * dx, dz among those {@code holds} says may be taken; -1 when none may. A wall torch turned to
	 * point at a bed (`turn_torches`).
	 */
	public static int nearestSide(double dx, double dz, boolean[] holds) {
		int[][] ways = {{0, -1}, {0, 1}, {-1, 0}, {1, 0}};
		int best = -1;
		double most = Double.NEGATIVE_INFINITY;
		for (int i = 0; i < 4; i++) {
			double along = ways[i][0] * dx + ways[i][1] * dz;
			if (holds[i] && along > most) {
				most = along;
				best = i;
			}
		}

		return best;
	}

	/** The pitch (down is positive) that looks dy up over a horizontal run of dx, dz. */
	public static float pitch(double dx, double dy, double dz) {
		return (float) -Math.toDegrees(Math.atan2(dy, Math.sqrt(dx * dx + dz * dz)));
	}
}
