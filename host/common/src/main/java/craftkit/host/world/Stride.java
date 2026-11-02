package craftkit.host.world;

/**
 * A walker's steps, counted as the game counts its own (a step each 1/0.6 blocks walked on the
 * ground, none while crouching or in the air), apart from the game so JUnit checks it: what
 * {@link Footsteps} follows a player with (`footsteps_follow`, batch 16).
 */
public final class Stride {
	/** The ground walked between two steps, as the game's own step sounds. */
	public static final double STEP = 1 / 0.6;

	private double walked;

	/** Moved `dx`, `dz` this tick: whether that finishes a step. */
	public boolean moved(double dx, double dz, boolean onGround, boolean crouching) {
		if (!onGround || crouching) {
			return false;
		}
		walked += Math.sqrt(dx * dx + dz * dz);
		if (walked >= STEP) {
			walked -= STEP * Math.floor(walked / STEP);
			return true;
		}

		return false;
	}

	/** Where a step `behind` blocks back along the way they went lands, from where they are. */
	public static double[] behind(double x, double z, double dx, double dz, double behind) {
		double length = Math.sqrt(dx * dx + dz * dz);
		if (length < 1e-6) {
			return new double[] {x, z};
		}

		return new double[] {x - dx / length * behind, z - dz / length * behind};
	}
}
