package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * `throw_back` (ABI 56): what a player next throws or shoots comes back. A snowball of theirs, where
 * it lands, a snowball thrown back at them a moment on; an arrow of theirs that sticks, taken out
 * and dropped at their feet a moment on, the very arrow, and only one they could have picked up,
 * so nothing is had from nothing. Armed by player and kind until a game time; once.
 */
public final class Returns {
	/** How long after it lands it comes back, ticks. */
	static final long BACK_TICKS = 30;

	public enum Kind { SNOWBALL, ARROW }

	private record Armed(Kind kind, long until) {
	}

	private record Due(ServerLevel level, UUID player, Kind kind, Vec3 from, ItemStack arrow, long at) {
	}

	private static final Map<UUID, Armed> ARMED = new ConcurrentHashMap<>();
	private static final List<Due> DUE = new ArrayList<>();

	private Returns() {
	}

	public static void arm(UUID player, Kind kind, long until) {
		ARMED.put(player, new Armed(kind, until));
	}

	/** A thing of theirs landed: whether it is the one that comes back, taken to come back if so. */
	public static boolean landed(ServerLevel level, UUID owner, Kind kind, Vec3 at, ItemStack arrow) {
		Armed armed = ARMED.get(owner);
		if (armed == null || armed.kind() != kind) {
			return false;
		}
		ARMED.remove(owner);
		if (level.getGameTime() > armed.until()) {
			return false;
		}
		synchronized (DUE) {
			DUE.add(new Due(level, owner, kind, at, arrow == null ? ItemStack.EMPTY : arrow.copy(), level.getGameTime() + BACK_TICKS));
		}
		return true;
	}

	public static void tick(MinecraftServer server) {
		long now = server.overworld().getGameTime();
		ARMED.values().removeIf(a -> now > a.until());
		synchronized (DUE) {
			Iterator<Due> it = DUE.iterator();
			while (it.hasNext()) {
				Due due = it.next();
				if (due.level().getGameTime() < due.at()) {
					continue;
				}
				it.remove();
				if (!(due.level().getPlayerByUUID(due.player()) instanceof ServerPlayer player) || player.level() != due.level()) {
					continue;
				}
				if (due.kind() == Kind.SNOWBALL) {
					Vec3 from = due.from().add(0, 0.5, 0);
					var ball = new net.minecraft.world.entity.projectile.throwableitemprojectile.Snowball(due.level(), from.x, from.y, from.z,
							new ItemStack(net.minecraft.world.item.Items.SNOWBALL));
					Vec3 to = player.position().add(0, player.getBbHeight() * 0.6, 0).subtract(from);
					ball.shoot(to.x, to.y + Math.sqrt(to.x * to.x + to.z * to.z) * 0.1, to.z, 1.4f, 0.5f);
					due.level().addFreshEntity(ball);
				} else if (!due.arrow().isEmpty()) {
					var item = new net.minecraft.world.entity.item.ItemEntity(due.level(), player.getX(), player.getY() + 0.1, player.getZ(), due.arrow());
					item.setDeltaMovement(Vec3.ZERO);
					due.level().addFreshEntity(item);
				}
			}
		}
	}
}
