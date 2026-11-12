package craftkit.host.world;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Who has an animal refuse their food next (`refuse_food`, ABI 56): by player, until a game time.
 * The first feeding after is refused once and the mark taken; past its time it lapses.
 */
public final class Refusals {
	private static final Map<UUID, Long> UNTIL = new ConcurrentHashMap<>();

	private Refusals() {
	}

	/** The next feeding by this player before game time {@code until} is refused. */
	public static void arm(UUID player, long until) {
		UNTIL.put(player, until);
	}

	/** Whether this feeding is the one refused, taking the mark if so. */
	public static boolean take(UUID player, long now) {
		Long until = UNTIL.get(player);
		if (until == null) {
			return false;
		}
		UNTIL.remove(player);

		return now <= until;
	}
}
