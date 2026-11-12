package craftkit.host.world;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Things set off one after another toward a player, the farthest first: pure, so JUnit checks
 * it without the game (`tip_dripleaves`'s order, as if something walked toward them).
 */
public final class Toward {
	private Toward() {
	}

	/** The indices of {@code points} ({@code {x, z}} each), farthest from (px, pz) first, at most {@code most}, of those nearest. */
	public static List<Integer> order(List<double[]> points, double px, double pz, int most) {
		List<Integer> indices = new ArrayList<>();
		for (int i = 0; i < points.size(); i++) {
			indices.add(i);
		}
		Comparator<Integer> near = Comparator.comparingDouble(i -> Math.hypot(points.get(i)[0] - px, points.get(i)[1] - pz));
		indices.sort(near);
		List<Integer> kept = new ArrayList<>(indices.subList(0, Math.min(most, indices.size())));
		kept.sort(near.reversed());

		return kept;
	}
}
