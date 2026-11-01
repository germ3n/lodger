package craftkit.host.world;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.function.Predicate;

/**
 * Things to do at a tick, in the order they were added: the hidden blocks to put back. Pure, so
 * JUnit checks the timing apart from the world.
 */
public final class Due<T> {
	private record Entry<T>(T thing, long at) {
	}

	private final List<Entry<T>> entries = new ArrayList<>();

	/** Adds a thing due at a tick. */
	public void add(T thing, long at) {
		entries.add(new Entry<>(thing, at));
	}

	/** Takes out and answers what is due at or before a tick. */
	public List<T> due(long tick) {
		return take(e -> e.at() <= tick);
	}

	/** Takes out and answers everything that matches, due or not: a level closing. */
	public List<T> all(Predicate<T> which) {
		return take(e -> which.test(e.thing()));
	}

	public int size() {
		return entries.size();
	}

	private List<T> take(Predicate<Entry<T>> which) {
		List<T> out = new ArrayList<>();
		Iterator<Entry<T>> it = entries.iterator();
		while (it.hasNext()) {
			Entry<T> entry = it.next();
			if (which.test(entry)) {
				out.add(entry.thing());
				it.remove();
			}
		}

		return out;
	}
}
