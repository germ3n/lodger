package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.block.state.BlockState;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * `block_back` (ABI 57): the next block a player breaks within the armed time is there again on
 * their screen alone a blink, then gone, the world's own break untouched (`block_comes_back`).
 */
public final class BlockBacks {
	/** How long the broken block shows again, ticks. */
	static final long SHOWN_TICKS = 5;

	private record Shown(ServerPlayer player, ServerLevel level, BlockPos pos, long due) {
	}

	private static final Map<UUID, Long> ARMED = new ConcurrentHashMap<>();
	private static final List<Shown> SHOWN = new ArrayList<>();

	private BlockBacks() {
	}

	public static void arm(UUID player, long until) {
		ARMED.put(player, until);
	}

	/** A block they just broke: shown again to them alone a blink when they are armed. */
	public static void broke(ServerPlayer player, BlockPos pos, BlockState was) {
		if (ARMED.isEmpty() || !(player.level() instanceof ServerLevel level)) {
			return;
		}
		if (!Arming.spend(ARMED, player.getUUID(), level.getGameTime())) {
			return;
		}
		player.connection.send(new net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket(pos, was));
		synchronized (SHOWN) {
			SHOWN.add(new Shown(player, level, pos.immutable(), level.getGameTime() + SHOWN_TICKS));
		}
	}

	/** What was shown again, shown as it really is when due. */
	public static void tick(MinecraftServer server) {
		synchronized (SHOWN) {
			Iterator<Shown> it = SHOWN.iterator();
			while (it.hasNext()) {
				Shown shown = it.next();
				if (shown.level().getGameTime() >= shown.due()) {
					it.remove();
					if (!shown.player().isRemoved()) {
						shown.player().connection.send(new net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket(shown.level(), shown.pos()));
					}
				}
			}
		}
	}
}
