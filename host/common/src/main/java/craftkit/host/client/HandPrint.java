package craftkit.host.client;

import java.util.ArrayList;
import java.util.List;

/**
 * The shape `handprints` lays on a pane, apart from the game so JUnit checks it: two hands side
 * by side, each a palm and five fingers of dots, on the pane's middle plane across the way they
 * look at it (from {@code dx}, {@code dz}, the pane's middle less their eyes), set a touch toward
 * them so the glass hides none of it. Each point {x, y, z} from the pane block's middle and its
 * bottom.
 */
public final class HandPrint {
	/** A dot's spacing, and how far toward them off the pane's middle. */
	static final double DOT = 0.035;
	static final double TOWARD = 0.08;

	private HandPrint() {
	}

	public static List<double[]> points(double dx, double dz) {
		double length = Math.sqrt(dx * dx + dz * dz);
		if (length < 1e-6) {
			return List.of();
		}
		// Across the way they look, and toward them.
		double ax = -dz / length;
		double az = dx / length;
		double tx = -dx / length * TOWARD;
		double tz = -dz / length * TOWARD;
		List<double[]> points = new ArrayList<>();
		for (double side : new double[] {-0.17, 0.17}) {
			// The palm: four by four.
			for (int u = -2; u <= 1; u++) {
				for (int v = 0; v < 4; v++) {
					points.add(at(ax, az, tx, tz, side + (u + 0.5) * DOT, 0.42 + v * DOT));
				}
			}
			// Five fingers up from it, the thumb lower and out.
			double[][] fingers = {{-2.0, 4, 3}, {-0.7, 4, 4}, {0.6, 4, 4}, {1.9, 4, 3}, {(side < 0 ? 3.2 : -3.2), 1, 2}};
			for (double[] finger : fingers) {
				for (int v = 0; v < finger[2]; v++) {
					points.add(at(ax, az, tx, tz, side + finger[0] * DOT, 0.42 + (finger[1] + v) * DOT));
				}
			}
		}

		return points;
	}

	private static double[] at(double ax, double az, double tx, double tz, double across, double up) {
		return new double[] {ax * across + tx, up, az * across + tz};
	}
}
