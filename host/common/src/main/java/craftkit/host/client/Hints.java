package craftkit.host.client;

import craftkit.host.net.HintPayload;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * The hints the client is showing right now, each until its time is up. The renderers (a later
 * phase) read this list each frame; today a hint is kept, timed and dropped, and logged once, so
 * the path from a mod's command to the client is in place.
 */
public final class Hints {
	public record Active(HintPayload hint, long untilMillis) {
	}

	private static final List<Active> ACTIVE = new ArrayList<>();

	private Hints() {
	}

	public static synchronized void show(HintPayload hint) {
		ACTIVE.add(new Active(hint, System.currentTimeMillis() + (long) (hint.seconds() * 1000)));
		craftkit.host.Log.LOGGER.info("hint {} for {}s{}", hint.hint(), hint.seconds(), hint.pos().map(p -> " at " + p.toShortString()).orElse(""));
	}

	/** The hints still running; expired ones fall away as this is read. */
	public static synchronized List<Active> active() {
		long now = System.currentTimeMillis();
		Iterator<Active> it = ACTIVE.iterator();
		while (it.hasNext()) {
			if (it.next().untilMillis() < now) {
				it.remove();
			}
		}

		return List.copyOf(ACTIVE);
	}

	/** Whether a hint of this kind is showing. */
	public static boolean showing(String kind) {
		return active().stream().anyMatch(active -> active.hint().hint().equals(kind));
	}
}
