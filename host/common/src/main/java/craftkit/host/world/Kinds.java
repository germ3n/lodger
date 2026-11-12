package craftkit.host.world;

import java.util.List;

/** Whether a block id is one a scan asked for: pure, for JUnit. */
public final class Kinds {
	private Kinds() {
	}

	/** An id asked for exactly, or one ending in what follows a leading {@code *} ({@code *_stairs}). */
	public static boolean wanted(String block, List<String> kinds) {
		for (String kind : kinds) {
			// The suffix matched in place: a substring for every block and every kind was made,
			// forty thousand blocks a scan.
			boolean match = kind.startsWith("*")
					? block.length() >= kind.length() - 1 && block.regionMatches(block.length() - (kind.length() - 1), kind, 1, kind.length() - 1)
					: block.equals(kind);
			if (match) {
				return true;
			}
		}

		return false;
	}
}
