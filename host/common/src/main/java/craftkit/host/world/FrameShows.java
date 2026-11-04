package craftkit.host.world;

import craftkit.host.Log;
import net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket;
import net.minecraft.network.syncher.EntityDataAccessor;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.decoration.ItemFrame;
import net.minecraft.world.item.ItemStack;

import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * Item frames showing one thing to one player ({@code frames_show}, ABI 38; `item_frames_face`,
 * batch 16): the frames' item as that player's client sees it is told to be another, for a while,
 * then what each really holds again. Nothing in a frame changes; a player who leaves and comes
 * back sees them as they are.
 */
public final class FrameShows {
	public static final float MOST_SECONDS = 600;

	private record Show(UUID player, List<ItemFrame> frames, long untilTick) {
	}

	private final List<Show> active = new ArrayList<>();
	private long tick;
	private static EntityDataAccessor<ItemStack> item;
	private static boolean reflected;

	/** Answers how many frames show it. */
	public int show(ServerPlayer player, List<ItemFrame> frames, ItemStack stack, float seconds) {
		if (!reflect() || frames.isEmpty()) {
			return 0;
		}
		for (ItemFrame frame : frames) {
			player.connection.send(new ClientboundSetEntityDataPacket(frame.getId(), List.of(SynchedEntityData.DataValue.create(item, stack.copy()))));
		}
		active.add(new Show(player.getUUID(), frames, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));

		return frames.size();
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Show> it = active.iterator();
		while (it.hasNext()) {
			Show show = it.next();
			ServerPlayer player = server.getPlayerList().getPlayer(show.player);
			if (player == null) {
				it.remove();
				continue;
			}
			if (show.untilTick < tick) {
				for (ItemFrame frame : show.frames) {
					if (!frame.isRemoved()) {
						player.connection.send(new ClientboundSetEntityDataPacket(frame.getId(), List.of(SynchedEntityData.DataValue.create(item, frame.getItem().copy()))));
					}
				}
				it.remove();
			}
		}
	}

	@SuppressWarnings("unchecked")
	private static boolean reflect() {
		if (reflected) {
			return item != null;
		}
		reflected = true;
		try {
			Field data = ItemFrame.class.getDeclaredField("DATA_ITEM");
			data.setAccessible(true);
			item = (EntityDataAccessor<ItemStack>) data.get(null);
		} catch (ReflectiveOperationException | RuntimeException e) {
			Log.LOGGER.warn("the item frame's field moved: no frames shown ({})", e.toString());
			item = null;
		}

		return item != null;
	}

	/** Everything for every player, forgotten: the world is closing, and none of it is the next one's. */
	public void clear() {
		active.clear();
	}
}
