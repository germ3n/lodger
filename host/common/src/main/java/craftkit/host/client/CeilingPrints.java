package craftkit.host.client;

import java.util.ArrayList;
import java.util.List;

/**
 * Where `prints_on_the_ceiling` steps: pure, so JUnit checks it without the game. From the start
 * to the end along the floor, a print every {@link #STRIDE} blocks, left and right of the line by
 * turns, each facing the way it walks; the client draws each on the ceiling over its place.
 */
public final class CeilingPrints {
	/** A step's length, in blocks. */
	public static final double STRIDE = 0.9;
	/** How far a print sits off the line, left or right. */
	public static final double SPLAY = 0.18;
	/** The most prints along a way, however long. */
	public static final int MOST = 40;

	/** One print: where over the floor, and the yaw it faces (0 is +z, 90 is -x). */
	public record Step(double x, double z, float yaw) {
	}

	private CeilingPrints() {
	}

	public static List<Step> steps(double fromX, double fromZ, double toX, double toZ) {
		double dx = toX - fromX;
		double dz = toZ - fromZ;
		double length = Math.sqrt(dx * dx + dz * dz);
		List<Step> steps = new ArrayList<>();
		if (length < 1e-6) {
			return steps;
		}
		double ux = dx / length;
		double uz = dz / length;
		float yaw = (float) (Math.toDegrees(Math.atan2(-ux, uz)));
		int count = (int) Math.min(MOST, Math.floor(length / STRIDE) + 1);
		for (int i = 0; i < count; i++) {
			double along = i * STRIDE;
			double side = (i % 2 == 0 ? 1 : -1) * SPLAY;
			steps.add(new Step(fromX + ux * along - uz * side, fromZ + uz * along + ux * side, yaw));
		}

		return steps;
	}
}
