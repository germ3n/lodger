package craftkit.host.world;

import java.util.Locale;

/**
 * How a figure was looked at while it stood there, apart from the game so JUnit checks it: a
 * look is facing it within ten degrees for half a second or more; the time of every such look
 * adds up, and how far they walked toward it is the distance closed from where they first stood
 * (owner's design: "facing within 10 degrees for half a second, how long, whether moved toward").
 * Reported once, as the figure goes ({@code figure_seen}, ABI 47).
 */
public final class Gaze {
	/** cos 10 degrees: a dot product of the view and the way to it above this is facing it. */
	public static final double FACING = Math.cos(Math.toRadians(10));
	/** Half a second: a glance shorter than this is not a look. */
	public static final int LEAST_TICKS = 10;

	private int run;
	private int lookedTicks;
	private int looks;
	private int longestRun;
	private double firstDistance = -1;
	private double closest = Double.MAX_VALUE;

	/** One tick: whether they face it within ten degrees, and how far they are from it. */
	public void tick(boolean facing, double distance) {
		if (firstDistance < 0) {
			firstDistance = distance;
		}
		closest = Math.min(closest, distance);
		if (facing) {
			run++;
			if (run == LEAST_TICKS) {
				looks++;
				lookedTicks += LEAST_TICKS;
			} else if (run > LEAST_TICKS) {
				lookedTicks++;
			}
			longestRun = Math.max(longestRun, run);
		} else {
			run = 0;
		}
	}

	/** Whether the view {@code dot} the way to it faces it within ten degrees. */
	public static boolean facing(double dot) {
		return dot > FACING;
	}

	public int lookedTicks() {
		return lookedTicks;
	}

	public int looks() {
		return looks;
	}

	/** How far they came toward it, in blocks, from where they first stood; 0 if they never did. */
	public double toward() {
		return firstDistance < 0 ? 0 : Math.max(0, firstDistance - closest);
	}

	/**
	 * The event's detail: {@code looked=<ticks> looks=<n> longest=<ticks> toward=<blocks>
	 * end=<how>}, where how is {@code approached} (they came within its distance and it went),
	 * {@code looked_away} (once seen, gone when they looked away), {@code timed} (its time ran
	 * out), {@code walked} (it reached its path's end) or {@code gone} (they left, died or went
	 * elsewhere).
	 */
	public String detail(String end) {
		return String.format(Locale.ROOT, "looked=%d looks=%d longest=%d toward=%.1f end=%s",
				lookedTicks, looks, longestRun >= LEAST_TICKS ? longestRun : 0, toward(), end);
	}
}
