package craftkit.host.world;

/**
 * A pack circling a player (`circle`, ABI 54), apart from the game so JUnit checks it: how far
 * out each is as the time goes, nearer each round, and where on the ring.
 */
public final class Ring {
	/** How far out they start, and how near they come. */
	public static final double FROM = 12;
	public static final double TO = 4;
	/** Of the time, the part spent circling; then they run. */
	public static final double CIRCLING = 0.8;
	/** How far round the ring they go a second, radians. */
	public static final double TURN_PER_SECOND = 0.6;

	private Ring() {
	}

	/** How far out at {@code t} of the time (0 to 1): from {@link #FROM} to {@link #TO} while circling. */
	public static double radius(double t) {
		double circling = Math.min(1, Math.max(0, t / CIRCLING));
		return FROM + (TO - FROM) * circling;
	}

	/** Whether at {@code t} of the time they have broken off and run. */
	public static boolean running(double t) {
		return t >= CIRCLING;
	}

	/** The angle of the {@code i}th of {@code n} at {@code seconds} in. */
	public static double angle(int i, int n, double seconds) {
		return 2 * Math.PI * i / Math.max(1, n) + seconds * TURN_PER_SECOND;
	}
}
