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
	/** The points of its way this far apart. */
	public static final double STEP = 0.6;
	/** How far it glides a tick: a slow creep, a block a second. */
	public static final double PER_TICK = 0.05;

	/** At the bed, before it squeezes: it stands, looks at the lead in its hands, then at them. */
	public static final int STANDS = 20;
	public static final int LOOKS_AT_IT = 40;
	public static final int LOOKS_AT_THEM = 25;
	public static final int BEFORE_SQUEEZING = STANDS + LOOKS_AT_IT + LOOKS_AT_THEM;

	/**
	 * Its beats at the bed (the owner: "make it so we see it like pulling it out and looking at
	 * it and then us and then choking us").
	 */
	public enum Beat {
		/** Arrived, empty-handed, facing them. */
		STANDS,
		/** The lead out, its rope between its hands, its head down at it. */
		LOOKS_AT_IT,
		/** Its head up again, at them. */
		LOOKS_AT_THEM,
		/** The rope at their neck, a squeeze a second. */
		CHOKES
	}

	private ChokeWay() {
	}

	/** The beat this many ticks after it reached the bed. */
	public static Beat beat(long since) {
		if (since < STANDS) {
			return Beat.STANDS;
		}
		if (since < STANDS + LOOKS_AT_IT) {
			return Beat.LOOKS_AT_IT;
		}
		if (since < BEFORE_SQUEEZING) {
			return Beat.LOOKS_AT_THEM;
		}

		return Beat.CHOKES;
	}

	/**
	 * Where its other hand holds the rope's far end, standing at {@code x, y, z} facing {@code
	 * yaw} (degrees, as the game's): a little out and in front of its left side, the lead in its
	 * right, as the game puts a player's held rope (right is the game's local -x). {x, y, z}.
	 */
	public static double[] otherHand(double x, double y, double z, float yaw) {
		double localX = 0.35;
		double localY = 0.85;
		double localZ = 0.3;
		// The game's Vec3.yRot(-yaw): a quarter turn is yaw 90, facing west.
		double a = -Math.toRadians(yaw);
		double cos = Math.cos(a);
		double sin = Math.sin(a);

		return new double[] {x + localX * cos + localZ * sin, y + localY, z + localZ * cos - localX * sin};
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

	/** The ticks a walk takes at {@link #PER_TICK}, whole, from its first point to its last. */
	public static int ticks(List<double[]> walk) {
		double length = 0;
		for (int i = 1; i < walk.size(); i++) {
			double[] a = walk.get(i - 1);
			double[] b = walk.get(i);
			length += Math.sqrt((b[0] - a[0]) * (b[0] - a[0]) + (b[1] - a[1]) * (b[1] - a[1]) + (b[2] - a[2]) * (b[2] - a[2]));
		}

		return (int) Math.ceil(length / PER_TICK - 1e-9);
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
