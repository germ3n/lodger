package craftkit.host.world;

/** A hit spared, apart from the game so JUnit checks it ({@link Charges}). */
public final class Spare {
	private Spare() {
	}

	/**
	 * The harm a hit asks for, spared: what the game's scaling by {@code scale} (half again on
	 * hard) still leaves them above one half heart's worth of health, never more than asked.
	 */
	public static float harm(float damage, float health, float scale) {
		return Math.min(damage, Math.max(0, (health - 1) / scale));
	}

	/**
	 * How long they may burn, spared: fire takes a point a second (more at the start of a
	 * second), so a point more is kept off; at most {@code most}, nothing at two health or less.
	 */
	public static float fireSeconds(float health, float most) {
		return (float) Math.max(0, Math.min(most, Math.floor(health - 2)));
	}

	/**
	 * An arrow's harm, spared: the game rounds its hit up from its speed at impact times its
	 * base, and an arrow falling from above can come in at a quarter over the 1.6 it was shot at,
	 * so a quarter more and a whole point of rounding are kept off (a spared arrow could kill).
	 */
	public static float arrowHarm(float damage, float health) {
		return harm(damage, health - 1, 1.25f);
	}
}
