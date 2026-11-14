package craftkit.host.client;

/** The weather hook's arithmetic, apart from the game so JUnit checks it (`dry_beside_you`). */
public final class Weather {
	private Weather() {
	}

	/** The column a block to their right as they face (the game's yaw: 0 south, 90 west), {x, z}. */
	public static int[] beside(double x, double z, float yaw) {
		double r = Math.toRadians(yaw);

		return new int[] {(int) Math.floor(x - Math.cos(r)), (int) Math.floor(z - Math.sin(r))};
	}
}
