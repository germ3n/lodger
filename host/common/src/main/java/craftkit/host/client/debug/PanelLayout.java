package craftkit.host.client.debug;

/**
 * Where the debug panel sits and how a click finds its rows: pure arithmetic, so JUnit checks
 * it without the game. The panel is a strip down the right of the screen, {@link #SHARE} of it
 * wide and at least {@link #LEAST_WIDTH} GUI pixels, drawn at {@link #SCALE} into units of its
 * own.
 */
public final class PanelLayout {
	/** The panel's text and rows against the game's GUI scale: small enough to read more. */
	public static final float SCALE = 0.6f;
	public static final float SHARE = 1f / 3f;
	public static final int LEAST_WIDTH = 230;

	private PanelLayout() {
	}

	/** Where the panel begins on a screen this wide, in GUI pixels: never past the left edge. */
	public static int left(int width) {
		int panel = Math.min(width, Math.max(LEAST_WIDTH, Math.round(width * SHARE)));

		return width - panel;
	}

	/** How wide the panel is in its own units, on a screen this wide. */
	public static int units(int width) {
		return (int) ((width - left(width)) / SCALE);
	}

	/** A GUI pixel's place in the panel's units, x or y; x measured from the panel's left. */
	public static double toPanel(double gui, int left) {
		return (gui - left) / SCALE;
	}
}
