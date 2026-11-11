package craftkit.host.client;

/** How much of the experience bar `xp_slips` leaves showing, apart from the game so JUnit checks it. */
public final class XpSlip {
	/** The bar's width in GUI pixels, as the game draws it. */
	public static final int WIDTH = 182;

	private XpSlip() {
	}

	/** The pixels from the left still showing, the hint this far through (0 to 1): all to none over its first half, then none. */
	public static int kept(float progress) {
		float drained = Math.min(1, Math.max(0, progress * 2));

		return Math.round(WIDTH * (1 - drained));
	}
}
