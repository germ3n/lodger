package craftkit.host.client;

import craftkit.host.net.HintPayload;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * The hints the client is showing right now, each until its time is up. {@link Glitches} reads
 * this list each tick and each frame and renders what each kind means; here a hint is kept,
 * timed, numbered (so a one-shot effect runs once per hint) and dropped, and logged once.
 */
public final class Hints {
	public record Active(long id, HintPayload hint, long startedMillis, long untilMillis) {
		/** How far through its time the hint is, zero to one. */
		public float progress(long now) {
			long span = Math.max(1, untilMillis - startedMillis);

			return Math.max(0, Math.min(1, (now - startedMillis) / (float) span));
		}
	}

	private static final List<Active> ACTIVE = new ArrayList<>();
	private static long next = 1;

	private Hints() {
	}

	public static synchronized void show(HintPayload hint) {
		long now = System.currentTimeMillis();
		ACTIVE.add(new Active(next++, hint, now, now + (long) (hint.seconds() * 1000)));
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

	/** Drops every hint: the player left the world. */
	public static synchronized void clear() {
		ACTIVE.clear();
	}
}
