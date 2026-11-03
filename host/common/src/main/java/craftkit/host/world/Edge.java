package craftkit.host.world;

/**
 * Whether a step off a place is a safe fall (`shove`, batch 16), apart from the game so JUnit
 * checks it: the column a step out must be open at the feet and the head, and its ground must
 * be at least two and at most {@code most} blocks down, so the fall harms no one (the game's
 * fall harm starts past three blocks).
 */
public final class Edge {
	private Edge() {
	}

	/** One block of the column out from where they stand, by depth below their feet (0 is the feet's level). */
	public interface Column {
		/** Open: nothing to stand in the way (air, a flower, a torch). */
		boolean open(int depth);

		/** Solid enough to land on. */
		boolean ground(int depth);
	}

	/**
	 * The drop off the edge, 2 to {@code most}, or -1 when it is no safe edge: the way blocked
	 * at the feet or the head, no fall (ground at once), or too far, or water or something that
	 * is neither below (the column ends at a block neither open nor ground).
	 */
	public static int drop(Column column, int most) {
		if (!column.open(0) || !column.open(-1)) {
			return -1;
		}
		for (int depth = 1; depth <= most + 1; depth++) {
			if (column.ground(depth)) {
				int fall = depth - 1;
				return fall >= 2 && fall <= most ? fall : -1;
			}
			if (!column.open(depth)) {
				return -1;
			}
		}

		return -1;
	}
}
