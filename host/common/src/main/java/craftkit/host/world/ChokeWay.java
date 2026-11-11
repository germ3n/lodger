package craftkit.host.world;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * The way a choke's figure comes to the bed, apart from the game so JUnit checks it
 * ({@link Figures#choke}): the spots beside a bed it may stand on, best first, and its slow
 * walk there.
 */
public final class ChokeWay {
	/** The farthest it walks: from farther it first stands this far off along the way. */
	public static final double MOST_WALK = 7;
	/** A stride each half second: a slow creep. */
	public static final double STEP = 0.6;

	private ChokeWay() {
	}

	/**
	 * The spots beside a bed, best first: by the pillow on either side, the side nearer
	 * {@code fromX}, {@code fromZ} first; then beside its foot the same way; then past its foot.
	 * The head's block and the way the bed faces (foot to head) given; each spot {x, z}.
	 */
	public static List<int[]> bedside(int headX, int headZ, int faceX, int faceZ, double fromX, double fromZ) {
		int footX = headX - faceX;
		int footZ = headZ - faceZ;
		// A quarter turn either way: the bed's two long sides.
		int sideX = -faceZ;
		int sideZ = faceX;
		Comparator<int[]> nearer = Comparator.comparingDouble(p -> {
			double dx = p[0] + 0.5 - fromX;
			double dz = p[1] + 0.5 - fromZ;
			return dx * dx + dz * dz;
		});
		List<int[]> head = new ArrayList<>(List.of(new int[] {headX + sideX, headZ + sideZ}, new int[] {headX - sideX, headZ - sideZ}));
		List<int[]> foot = new ArrayList<>(List.of(new int[] {footX + sideX, footZ + sideZ}, new int[] {footX - sideX, footZ - sideZ}));
		head.sort(nearer);
		foot.sort(nearer);
		List<int[]> spots = new ArrayList<>(head);
		spots.addAll(foot);
		spots.add(new int[] {footX - faceX, footZ - faceZ});

		return spots;
	}

	/**
	 * The walk from one place to another, a point a stride apart and the last one the place
	 * itself; begun at most {@link #MOST_WALK} off along the line. Each point {x, y, z}.
	 */
	public static List<double[]> walk(double fromX, double fromY, double fromZ, double toX, double toY, double toZ) {
		double dx = toX - fromX;
		double dy = toY - fromY;
		double dz = toZ - fromZ;
		double length = Math.sqrt(dx * dx + dz * dz);
		double start = Math.max(0, length - MOST_WALK);
		int strides = (int) Math.ceil((length - start) / STEP);
		List<double[]> points = new ArrayList<>();
		for (int i = 0; i <= strides; i++) {
			double along = length < 1e-6 ? 1 : Math.min(1, (start + i * STEP) / length);
			points.add(new double[] {fromX + dx * along, fromY + dy * along, fromZ + dz * along});
		}
		double[] last = points.get(points.size() - 1);
		if (last[0] != toX || last[1] != toY || last[2] != toZ) {
			points.add(new double[] {toX, toY, toZ});
		}

		return points;
	}
}
