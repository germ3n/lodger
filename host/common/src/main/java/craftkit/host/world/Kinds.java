package craftkit.host.world;

import java.util.List;

/** Whether a block id is one a scan asked for: pure, for JUnit. */
public final class Kinds {
	private Kinds() {
	}

	/** An id asked for exactly, or one ending in what follows a leading {@code *} ({@code *_stairs}). */
	public static boolean wanted(String block, List<String> kinds) {
		for (String kind : kinds) {
			if (kind.startsWith("*") ? block.endsWith(kind.substring(1)) : block.equals(kind)) {
				return true;
			}
		}

		return false;
	}
}
