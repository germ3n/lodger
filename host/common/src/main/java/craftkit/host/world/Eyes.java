package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.projectile.EyeOfEnder;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

/**
 * Eyes of ender sent home (`eye_home`, ABI 54): while a player is armed, the next eye they throw
 * flies toward a place of the mod's (their house) instead of the stronghold, once.
 */
public final class Eyes {
	/** The longest an arming waits for a throw. */
	public static final float MOST_SECONDS = 1800;

	private record Armed(Vec3 home, long untilTick) {
	}

	private final Map<UUID, Armed> armed = new HashMap<>();
	/** The eyes already seen, so one is turned once and an old one never. */
	private final Set<UUID> seen = new HashSet<>();
	private long tick;

	/** The next eye {@code player} throws, within {@code seconds}, flies toward {@code home}. */
	public void arm(ServerPlayer player, Vec3 home, float seconds) {
		armed.put(player.getUUID(), new Armed(home, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));
	}

	public void tick(MinecraftServer server) {
		tick++;
		if (armed.isEmpty()) {
			seen.clear();
			return;
		}
		armed.entrySet().removeIf(e -> e.getValue().untilTick() < tick);
		for (var entry : Map.copyOf(armed).entrySet()) {
			ServerPlayer player = server.getPlayerList().getPlayer(entry.getKey());
			if (player == null) {
				continue;
			}
			for (EyeOfEnder eye : player.level().getEntitiesOfClass(EyeOfEnder.class, new AABB(player.blockPosition()).inflate(4))) {
				if (!seen.add(eye.getUUID())) {
					continue;
				}
				// Thrown this moment: near their hand, its life just begun.
				if (eye.tickCount <= 2) {
					eye.signalTo(entry.getValue().home());
					armed.remove(entry.getKey());
					break;
				}
			}
		}
	}

	public void clear() {
		armed.clear();
		seen.clear();
	}
}
