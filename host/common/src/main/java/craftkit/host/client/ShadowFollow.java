package craftkit.host.client;

/**
 * `shadow_follows`' shadow, apart from the game so JUnit checks it: a person's outline
 * ({@link Silhouette}) laid flat on the ground from its feet, long along the way a shadow falls
 * (away from the sun, or from the light they hold), and where on their trail it stands.
 */
public final class ShadowFollow {
	/** How much longer than the person the shadow lies. */
	public static final double STRETCH = 1.6;
	/** How far back along their trail it walks, ticks. */
	public static final int BEHIND_TICKS = 50;

	private ShadowFollow() {
	}

	/** An outline point {across, up} on the ground, {x, z} from the feet, the shadow falling along (dx, dz). */
	public static double[] ground(double across, double up, double dx, double dz, double scale) {
		double length = Math.hypot(dx, dz);
		double ax = length == 0 ? 1 : dx / length;
		double az = length == 0 ? 0 : dz / length;

		return new double[] {(ax * up * STRETCH - az * across) * scale, (az * up * STRETCH + ax * across) * scale};
	}

	/** The way a shadow falls by day: away from the sun, west before noon and east after (the sun rises in the east). */
	public static double[] fromSun(long dayTime) {
		long t = Math.floorMod(dayTime, 24000L);

		return t < 6000 ? new double[] {-1, 0} : new double[] {1, 0};
	}
}
