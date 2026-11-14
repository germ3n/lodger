package craftkit.host.client;

import java.util.ArrayList;
import java.util.List;

/**
 * `tally_on_the_wall`'s strokes as points on a wall's face, apart from the game so JUnit checks
 * it: {@code {across, up}} in blocks about the face's middle, each group four upright strokes and
 * a fifth across them, as a prisoner counts days; at most {@link #MOST} strokes.
 */
public final class Tally {
	public static final int MOST = 40;
	static final double STEP = 0.05;
	static final double GAP = 0.09;
	static final double GROUP = 0.5;
	static final double HIGH = 0.4;

	private Tally() {
	}

	public static List<double[]> points(int count) {
		int n = Math.max(0, Math.min(MOST, count));
		int groups = (n + 4) / 5;
		int perRow = 4;
		List<double[]> points = new ArrayList<>();
		for (int g = 0; g < groups; g++) {
			int row = g / perRow;
			int col = g % perRow;
			double left = -0.9 + col * GROUP;
			double bottom = 0.6 - row * (HIGH + 0.15);
			int inGroup = Math.min(5, n - g * 5);
			for (int s = 0; s < Math.min(4, inGroup); s++) {
				for (double up = 0; up <= HIGH; up += STEP) {
					points.add(new double[] {left + s * GAP, bottom + up});
				}
			}
			if (inGroup == 5) {
				for (double t = 0; t <= 1; t += 0.08) {
					points.add(new double[] {left - 0.05 + t * (3 * GAP + 0.1), bottom + 0.05 + t * (HIGH - 0.1)});
				}
			}
		}

		return points;
	}
}
