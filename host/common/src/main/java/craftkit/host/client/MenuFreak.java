package craftkit.host.client;

/**
 * The pause menu going wrong (`menu_freaks`, owner: "opening the main menu it kinda freaks out,
 * like buttons vibrate"), apart from the game so JUnit checks it: how far each button is shaken
 * as the time goes, hardest at first and settling, and when the first button's words read wrong.
 */
public final class MenuFreak {
	/** How long it goes on, milliseconds, and how far a button shakes at first, pixels. */
	public static final long LASTS_MS = 4000;
	public static final int MOST_SHAKE = 3;
	/** When the first button reads wrong, and for how long, milliseconds. */
	public static final long WRONG_FROM_MS = 900;
	public static final long WRONG_FOR_MS = 600;
	/** What it reads then. */
	public static final String WRONG = "stay";

	private MenuFreak() {
	}

	/** A button's shake now, {dx, dy}: uneven, every button its own, less as it settles; none once it is over. */
	public static int[] shake(long sinceMs, int button) {
		if (sinceMs < 0 || sinceMs >= LASTS_MS) {
			return new int[] {0, 0};
		}
		double left = 1 - sinceMs / (double) LASTS_MS;
		double reach = MOST_SHAKE * left;
		double t = sinceMs / 16.0 + button * 7.3;
		int dx = (int) Math.round(Math.sin(t * 1.9) * Math.cos(t * 0.7) * reach);
		int dy = (int) Math.round(Math.cos(t * 2.3) * Math.sin(t * 1.1) * reach);
		return new int[] {dx, dy};
	}

	/** Whether the first button reads wrong now. */
	public static boolean wrong(long sinceMs) {
		return sinceMs >= WRONG_FROM_MS && sinceMs < WRONG_FROM_MS + WRONG_FOR_MS;
	}

	/** Whether it is over. */
	public static boolean over(long sinceMs) {
		return sinceMs >= LASTS_MS;
	}
}
