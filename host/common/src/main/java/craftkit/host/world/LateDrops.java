package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.Block;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * `late_drop` (ABI 56): the next block a player breaks near them drops what it drops a moment
 * late, as if held. Armed by player until a game time; the first break near them within it is
 * the one, every stack it drops that same tick held back, then let fall where it would have.
 */
public final class LateDrops {
	/** How near the player the break must be, blocks, and how long its drops are held, ticks. */
	static final double NEAR = 8;
	static final long HELD_TICKS = 30;

	private record Armed(long until, long[] firedAt) {
	}

	private record Held(ServerLevel level, BlockPos pos, ItemStack stack, long due) {
	}

	private static final Map<UUID, Armed> ARMED = new ConcurrentHashMap<>();
	private static final List<Held> HELD = new ArrayList<>();
	private static boolean releasing;

	private LateDrops() {
	}

	public static void arm(UUID player, long until) {
		ARMED.put(player, new Armed(until, new long[] {-1}));
	}

	/** Whether this drop is held back, and taken to be let fall later if so. */
	public static boolean hold(ServerLevel level, BlockPos pos, ItemStack stack) {
		if (releasing || ARMED.isEmpty()) {
			return false;
		}
		long now = level.getGameTime();
		for (Map.Entry<UUID, Armed> entry : ARMED.entrySet()) {
			var player = level.getPlayerByUUID(entry.getKey());
			Armed armed = entry.getValue();
			if (player == null || now > armed.until() || player.position().distanceTo(net.minecraft.world.phys.Vec3.atCenterOf(pos)) > NEAR) {
				continue;
			}
			// The first break's, and the rest of what it drops this same tick.
			if (armed.firedAt()[0] < 0) {
				armed.firedAt()[0] = now;
			} else if (armed.firedAt()[0] != now) {
				continue;
			}
			synchronized (HELD) {
				HELD.add(new Held(level, pos.immutable(), stack.copy(), now + HELD_TICKS));
			}
			return true;
		}

		return false;
	}

	/** The held drops let fall when due; marks spent or past their time gone. */
	public static void tick(MinecraftServer server) {
		long now = server.overworld().getGameTime();
		ARMED.values().removeIf(a -> (a.firedAt()[0] >= 0 && now > a.firedAt()[0]) || now > a.until());
		synchronized (HELD) {
			Iterator<Held> it = HELD.iterator();
			while (it.hasNext()) {
				Held held = it.next();
				if (held.level().getGameTime() >= held.due()) {
					it.remove();
					releasing = true;
					try {
						Block.popResource(held.level(), held.pos(), held.stack());
					} finally {
						releasing = false;
					}
				}
			}
		}
	}
}
