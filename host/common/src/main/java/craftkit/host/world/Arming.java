package craftkit.host.world;

import java.util.Map;
import java.util.UUID;

/** A player armed until a game time, spent by the first use within it (`BlockBacks`); apart from the game so JUnit checks it. */
final class Arming {
	private Arming() {
	}

	/** Whether this player's use at this game time is the one armed: then spent; one past its time is dropped. */
	static boolean spend(Map<UUID, Long> armed, UUID player, long now) {
		Long until = armed.remove(player);

		return until != null && now <= until;
	}
}
