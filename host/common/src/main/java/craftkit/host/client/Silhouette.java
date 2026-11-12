package craftkit.host.client;

import java.util.ArrayList;
import java.util.List;

/**
 * A person's outline as points on a wall, for `shadow_on_the_wall`: pure, so JUnit checks it
 * without the game. Each point is {@code {across, up}} in blocks, across from -0.5 to 0.5 about
 * the middle, up from the feet (0) to the top of the head (about 1.9), a dot every
 * {@link #STEP} filling the head, the body, the arms hanging and the legs.
 */
public final class Silhouette {
	public static final double STEP = 0.08;

	private Silhouette() {
	}

	public static List<double[]> points() {
		List<double[]> points = new ArrayList<>();
		for (double up = 0; up <= 1.95; up += STEP) {
			for (double across = -0.5; across <= 0.5; across += STEP) {
				if (inside(across, up)) {
					points.add(new double[] {across, up});
				}
			}
		}

		return points;
	}

	/** Whether a point is inside the outline. */
	static boolean inside(double across, double up) {
		double a = Math.abs(across);
		// The head, a round of a quarter block about 1.65 up.
		if (Math.hypot(across, up - 1.65) <= 0.24) {
			return true;
		}
		// The body, shoulders to hips.
		if (up >= 0.75 && up <= 1.42 && a <= 0.22) {
			return true;
		}
		// The arms, hanging at the sides.
		if (up >= 0.7 && up <= 1.4 && a > 0.26 && a <= 0.38) {
			return true;
		}
		// The legs, a gap between.
		return up >= 0 && up < 0.75 && a >= 0.04 && a <= 0.2;
	}
}
