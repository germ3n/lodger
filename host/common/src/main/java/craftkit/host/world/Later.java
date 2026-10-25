package craftkit.host.world;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * Things the host does a few ticks on, on the server thread: a chest lid closed after its
 * moment, a player-list entry taken away after its minute. Each is a tick and a runnable;
 * {@link #tick()} runs the ones whose tick has come.
 */
public final class Later {
	private record Pending(long atTick, Runnable run) {
	}

	private final List<Pending> pending = new ArrayList<>();
	private long tick;

	public void at(float seconds, Runnable run) {
		pending.add(new Pending(tick + Math.max(1, (long) (seconds * 20)), run));
	}

	public void tick() {
		tick++;
		Iterator<Pending> it = pending.iterator();
		while (it.hasNext()) {
			Pending p = it.next();
			if (p.atTick() <= tick) {
				it.remove();
				p.run().run();
			}
		}
	}

	public int count() {
		return pending.size();
	}
}
