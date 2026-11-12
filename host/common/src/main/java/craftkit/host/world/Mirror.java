package craftkit.host.world;

/**
 * Where a mirroring figure stands (ABI 33): where it was shown, moved as far as the player has
 * moved since, on the ground. Apart from the game so JUnit checks it.
 */
public final class Mirror {
	/**
	 * The least move sent, squared: a centimetre. A sneak is six a tick, so every step of theirs
	 * is its step too, and standing still sends nothing.
	 */
	public static final double LEAST_MOVE_SQ = 1e-4;

	private Mirror() {
	}

	/** Its place across the ground, x then z: where it was shown, plus how far they have come. */
	public static double[] to(double shownX, double shownZ, double fromX, double fromZ, double nowX, double nowZ) {
		return new double[] {shownX + (nowX - fromX), shownZ + (nowZ - fromZ)};
	}

	/** Whether it moves this tick: there is further than {@link #LEAST_MOVE_SQ} from here. */
	public static boolean moves(double atX, double atZ, double toX, double toZ) {
		double dx = toX - atX;
		double dz = toZ - atZ;

		return dx * dx + dz * dz > LEAST_MOVE_SQ;
	}
}
