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
}
