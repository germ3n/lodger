package craftkit.host.world;

/** How full a boss bar is this far into its time ({@link BossBars}): pure, for JUnit. */
public final class BarFill {
	private BarFill() {
	}

	/** 0 to 1, this far into its time. */
	public static float of(long elapsed, long total) {
		if (total <= 0) {
			return 1;
		}
		return Math.max(0, Math.min(1, (float) elapsed / total));
	}
}
