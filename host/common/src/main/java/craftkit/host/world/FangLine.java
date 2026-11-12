package craftkit.host.world;

import java.util.ArrayList;
import java.util.List;

/**
 * Where `fangs`' line goes (ABI 56): from a place straight toward the player, a fang every block
 * and a quarter, the last a block and a half short of them. Apart from the game so JUnit checks
 * it.
 */
public final class FangLine {
	/** The way between two fangs, blocks, as an evoker's line. */
	public static final double STEP = 1.25;
	/** How far short of the player the last stands. */
	public static final double SHORT = 1.5;
	/** The ticks between one fang and the next. */
	public static final int GAP_TICKS = 2;
	/** The most in a line. */
	public static final int MOST = 16;

	private FangLine() {
	}

	/** Each fang's x and z, the first at the start: none when the player is too near for one. */
	public static List<double[]> cells(double fromX, double fromZ, double toX, double toZ) {
		double dx = toX - fromX;
		double dz = toZ - fromZ;
		double length = Math.sqrt(dx * dx + dz * dz);
		List<double[]> out = new ArrayList<>();
		if (length <= SHORT) {
			return out;
		}
		for (int i = 0; i < MOST && i * STEP <= length - SHORT; i++) {
			double f = i * STEP / length;
			out.add(new double[] {fromX + dx * f, fromZ + dz * f});
		}
		return out;
	}
}
