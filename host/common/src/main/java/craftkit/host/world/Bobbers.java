package craftkit.host.world;

import craftkit.host.Log;
import net.minecraft.core.Holder;
import net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket;
import net.minecraft.network.protocol.game.ClientboundSoundPacket;
import net.minecraft.network.syncher.EntityDataAccessor;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.entity.projectile.FishingHook;

import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * A bobber pulled under with nothing on it ({@code bobber}, ABI 38; `fish_on_line`, batch 16):
 * armed for a while, the first time the player's bobber sits in the water the client is told it
 * bites, so it splashes and goes under and is held there, while the server's own hook is kept
 * from any real bite: reeled in, it brings nothing.
 */
public final class Bobbers {
	/** The longest a bobber is waited for, and the longest it is held under. */
	public static final float MOST_SECONDS = 300;
	public static final float MOST_UNDER = 30;
	/** How often the bite is told again, so the bobber stays down. */
	private static final int AGAIN_EVERY = 12;

	private static final class Armed {
		final UUID player;
		final long untilTick;
		final int underTicks;
		FishingHook hook;
		long underUntil;

		Armed(UUID player, long untilTick, int underTicks) {
			this.player = player;
			this.untilTick = untilTick;
			this.underTicks = underTicks;
		}
	}

	private final List<Armed> armed = new ArrayList<>();
	private long tick;
	private static EntityDataAccessor<Boolean> biting;
	private static Field nibble;
	private static Field timeUntilHooked;
	private static Field timeUntilLured;
	private static boolean reflected;

	public void arm(ServerPlayer player, float seconds, float under) {
		armed.removeIf(a -> a.player.equals(player.getUUID()));
		armed.add(new Armed(player.getUUID(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20), (int) (Math.min(under, MOST_UNDER) * 20)));
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Armed> it = armed.iterator();
		while (it.hasNext()) {
			Armed each = it.next();
			ServerPlayer player = server.getPlayerList().getPlayer(each.player);
			if (player == null) {
				it.remove();
				continue;
			}
			if (each.hook == null) {
				FishingHook hook = player.fishing;
				if (each.untilTick < tick) {
					it.remove();
				} else if (hook != null && !hook.isRemoved() && hook.isInWater() && hook.getHookedIn() == null && hook.tickCount > 40) {
					each.hook = hook;
					each.underUntil = tick + each.underTicks;
					player.connection.send(new ClientboundSoundPacket(Holder.direct(SoundEvents.FISHING_BOBBER_SPLASH), SoundSource.NEUTRAL,
							hook.getX(), hook.getY(), hook.getZ(), 0.25f, 1 + (player.getRandom().nextFloat() - player.getRandom().nextFloat()) * 0.4f, player.getRandom().nextLong()));
				}
				continue;
			}
			FishingHook hook = each.hook;
			if (hook.isRemoved() || player.fishing != hook || each.underUntil < tick) {
				if (!hook.isRemoved()) {
					tell(player, hook, false);
				}
				it.remove();
				continue;
			}
			keepFromBiting(hook);
			if ((each.underUntil - tick) % AGAIN_EVERY == 0) {
				tell(player, hook, true);
			}
		}
	}

	/** The client told the bobber bites, or no longer. */
	private static void tell(ServerPlayer player, FishingHook hook, boolean bite) {
		if (!reflect()) {
			return;
		}
		player.connection.send(new ClientboundSetEntityDataPacket(hook.getId(), List.of(SynchedEntityData.DataValue.create(biting, bite))));
	}

	/** No real fish while it is held under: no nibble, nothing on its way. */
	private static void keepFromBiting(FishingHook hook) {
		if (!reflect()) {
			return;
		}
		try {
			nibble.setInt(hook, 0);
			timeUntilHooked.setInt(hook, 0);
			timeUntilLured.setInt(hook, Math.max(timeUntilLured.getInt(hook), 100));
		} catch (IllegalAccessException e) {
			// Reflected and opened: not thrown.
		}
	}

	@SuppressWarnings("unchecked")
	private static boolean reflect() {
		if (reflected) {
			return biting != null;
		}
		reflected = true;
		try {
			Field data = FishingHook.class.getDeclaredField("DATA_BITING");
			data.setAccessible(true);
			nibble = FishingHook.class.getDeclaredField("nibble");
			timeUntilHooked = FishingHook.class.getDeclaredField("timeUntilHooked");
			timeUntilLured = FishingHook.class.getDeclaredField("timeUntilLured");
			nibble.setAccessible(true);
			timeUntilHooked.setAccessible(true);
			timeUntilLured.setAccessible(true);
			biting = (EntityDataAccessor<Boolean>) data.get(null);
		} catch (ReflectiveOperationException | RuntimeException e) {
			Log.LOGGER.warn("the fishing hook's fields moved: no bobber pulled under ({})", e.toString());
			biting = null;
		}

		return biting != null;
	}
}
