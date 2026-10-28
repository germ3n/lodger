package craftkit.host.world;

/** The order a container's slots are tried in: pure, so JUnit checks it without the game. */
public final class Slots {
	private Slots() {
	}

	/** The slot asked for first (when it exists), then every other in order. */
	public static int[] order(int first, int size) {
		if (first < 0 || first >= size) {
			int[] all = new int[size];
			for (int i = 0; i < size; i++) {
				all[i] = i;
			}
			return all;
		}
		int[] order = new int[size];
		order[0] = first;
		for (int i = 0, at = 1; i < size; i++) {
			if (i != first) {
				order[at++] = i;
			}
		}

		return order;
	}
}
