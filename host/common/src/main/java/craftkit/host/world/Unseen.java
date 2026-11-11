package craftkit.host.world;

/**
 * A figure that moves only while no one looks at it (`while_unseen`, ABI 54), apart from the game
 * so JUnit checks it: whether it is in view, and the step it takes toward them when it is not.
 */
public final class Unseen {
	/** Within this of a player's view (the dot of their look and the way to it), it is seen: about seventy degrees off the middle, the screen's edge. */
	public static final double IN_VIEW = 0.35;
	/** A step every this many ticks while unseen, and how far. */
	public static final int EVERY_TICKS = 4;
	public static final double STEP = 0.9;
	/** Within this, along the ground, it has reached them: it strikes and is gone. */
	public static final double REACH = 1.6;

	private Unseen() {
	}

	/** Whether a figure {@code dot} off their view is seen, with nothing between (a wall hides it). */
	public static boolean seen(double dot, boolean clear) {
		return clear && dot > IN_VIEW;
	}

	/**
	 * Where a step from {@code (x, z)} toward {@code (px, pz)} lands, on the ground's plane, never
	 * nearer than {@link #REACH} short of them; null when it is within reach already.
	 */
	public static double[] step(double x, double z, double px, double pz) {
		double dx = px - x;
		double dz = pz - z;
		double away = Math.sqrt(dx * dx + dz * dz);
		if (away <= REACH) {
			return null;
		}
		double go = Math.min(STEP, away - REACH * 0.9);

		return new double[] {x + dx / away * go, z + dz / away * go};
	}
}
