package craftkit.host.world;

/**
 * Where a {@code set_block} with {@code settle} looks for a spot its block can stand on: pure,
 * so JUnit can check it without the game.
 */
public final class Settle {
	/** The farthest a {@code set_block} may look, whatever the mod asks. */
	public static final int MOST = 8;

	private Settle() {
	}

	/** 0, -1, 1, -2, 2 ... out to {@code settle} (clamped to 0..{@link #MOST}): the nearest heights first, down before up. */
	public static int[] order(int settle) {
		int n = Math.clamp(settle, 0, MOST);
		int[] order = new int[n * 2 + 1];
		for (int i = 1; i <= n; i++) {
			order[i * 2 - 1] = -i;
			order[i * 2] = i;
		}

		return order;
	}
}
