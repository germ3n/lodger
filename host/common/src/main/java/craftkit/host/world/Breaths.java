package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

/**
 * Their breath drawn down faster (`drain_air`, ABI 54): while it lasts, each tick a player is
 * under water their air goes down by one more, twice as fast. The drowning itself is the game's.
 */
public final class Breaths {
	/** The longest a drawing lasts. */
	public static final float MOST_SECONDS = 60;

	private final Map<UUID, Long> drawn = new HashMap<>();
	private long tick;

	public void drain(ServerPlayer player, float seconds) {
		drawn.put(player.getUUID(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20));
	}

	public void tick(MinecraftServer server) {
		tick++;
		drawn.entrySet().removeIf(e -> e.getValue() < tick);
		for (UUID uuid : drawn.keySet()) {
			ServerPlayer player = server.getPlayerList().getPlayer(uuid);
			if (player != null && player.isUnderWater() && player.getAirSupply() > -19) {
				player.setAirSupply(player.getAirSupply() - 1);
			}
		}
	}

	public void clear() {
		drawn.clear();
	}
}
