package craftkit.host.world;

/** Which way an entity turns to face a point: pure, so JUnit checks it without the game. */
public final class Facing {
	private Facing() {
	}

	/** The yaw, in the game's degrees (0 faces +z, 90 faces -x), that looks along dx, dz. */
	public static float yaw(double dx, double dz) {
		return (float) (Math.toDegrees(Math.atan2(dz, dx)) - 90.0);
	}

	/** The pitch (down is positive) that looks dy up over a horizontal run of dx, dz. */
	public static float pitch(double dx, double dy, double dz) {
		return (float) -Math.toDegrees(Math.atan2(dy, Math.sqrt(dx * dx + dz * dz)));
	}
}
