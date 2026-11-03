package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.ItemStack;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * The one death the hard ending deals ({@code kill_player}, ABI 41): the player dies as if fallen
 * out of the world, and keeps everything. What they carry is taken into the world's data before
 * the blow, so nothing drops and nothing is lost if they quit on the death screen; it is given
 * back, slot for slot, once they stand alive again, with their experience. Never in creative or
 * spectator.
 */
public final class Deaths {
	/** The world data's key for a slot held over a death: the prefix, the player, the slot. */
	public static final String KEY = "craftkit_death:";

	private record Held(UUID player, WorldData data, int level, float progress, int total) {
	}

	private final List<Held> held = new ArrayList<>();
	private long tick;

	/** Answers why not, or null when they died. */
	public String kill(ServerPlayer player, WorldData data) {
		if (data == null) {
			return "no world data to hold their things in";
		}
		if (player.isCreative() || player.isSpectator()) {
			return "never in creative or spectator";
		}
		if (player.isDeadOrDying()) {
			return "already dead";
		}
		Inventory inventory = player.getInventory();
		for (int slot = 0; slot < inventory.getContainerSize(); slot++) {
			ItemStack stack = inventory.getItem(slot);
			if (!stack.isEmpty()) {
				data.keep(KEY + player.getUUID() + ":" + slot, stack.copy());
				inventory.setItem(slot, ItemStack.EMPTY);
			}
		}
		held.add(new Held(player.getUUID(), data, player.experienceLevel, player.experienceProgress, player.totalExperience));
		// No experience dropped: it comes back with the rest.
		player.experienceLevel = 0;
		player.experienceProgress = 0;
		player.totalExperience = 0;
		ServerLevel level = (ServerLevel) player.level();
		player.hurtServer(level, level.damageSources().fellOutOfWorld(), Float.MAX_VALUE);

		return null;
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Held> it = held.iterator();
		while (it.hasNext()) {
			Held each = it.next();
			ServerPlayer player = server.getPlayerList().getPlayer(each.player);
			if (player != null && !player.isDeadOrDying()) {
				giveBack(player, each.data);
				player.setExperienceLevels(each.level);
				player.experienceProgress = each.progress;
				player.totalExperience = each.total;
				it.remove();
			}
		}
	}

	/** What a world's data still holds of a player alive now (the server stopped before they respawned): given back. */
	public void sweep(MinecraftServer server, WorldData data) {
		if (tick % 100 != 0) {
			return;
		}
		for (ServerPlayer player : server.getPlayerList().getPlayers()) {
			if (!player.isDeadOrDying() && held.stream().noneMatch(h -> h.player.equals(player.getUUID()))
					&& !data.keysStarting(KEY + player.getUUID() + ":").isEmpty()) {
				giveBack(player, data);
			}
		}
	}

	private static void giveBack(ServerPlayer player, WorldData data) {
		Inventory inventory = player.getInventory();
		for (String key : data.keysStarting(KEY + player.getUUID() + ":")) {
			ItemStack stack = data.giveBack(key);
			if (stack == null) {
				continue;
			}
			int slot;
			try {
				slot = Integer.parseInt(key.substring(key.lastIndexOf(':') + 1));
			} catch (NumberFormatException e) {
				slot = -1;
			}
			if (slot >= 0 && slot < inventory.getContainerSize() && inventory.getItem(slot).isEmpty()) {
				inventory.setItem(slot, stack);
			} else if (!inventory.add(stack)) {
				var entity = new net.minecraft.world.entity.item.ItemEntity(player.level(), player.getX(), player.getY(), player.getZ(), stack, 0, 0, 0);
				entity.setUnlimitedLifetime();
				player.level().addFreshEntity(entity);
			}
		}
	}
}
