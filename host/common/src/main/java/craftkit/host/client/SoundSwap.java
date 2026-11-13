package craftkit.host.client;

import java.util.Arrays;

/**
 * The `sound_swap` hint's rule, apart from the game so JUnit checks it (`wrong_footing`,
 * docs/events_todo.md, 322): while it is on, a sound the detail names plays as another. The
 * detail is {@code from=a,b;to=id}: {@code from} the sound paths swapped ({@code *.step} for every
 * step but the one it turns into), {@code to} the sound played instead.
 */
public final class SoundSwap {
	private SoundSwap() {
	}

	/** The sound to play instead of the one at {@code path} (`block.grass.step`), or null to leave it be. */
	public static String swapped(String path, String detail) {
		String from = "";
		String to = "";
		for (String pair : detail.split(";")) {
			int eq = pair.indexOf('=');
			if (eq > 0) {
				String key = pair.substring(0, eq).trim();
				String value = pair.substring(eq + 1).trim();
				if (key.equals("from")) {
					from = value;
				} else if (key.equals("to")) {
					to = value;
				}
			}
		}
		if (to.isEmpty() || from.isEmpty()) {
			return null;
		}
		String toPath = to.contains(":") ? to.substring(to.indexOf(':') + 1) : to;
		if (path.equals(toPath)) {
			return null;
		}
		boolean named = Arrays.stream(from.split(",")).map(String::trim).anyMatch(f ->
				f.equals(path) || (f.startsWith("*") && path.endsWith(f.substring(1)) && path.startsWith("block.")));

		return named ? to : null;
	}
}
