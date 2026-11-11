package craftkit.host.world;

import net.minecraft.core.component.DataComponents;
import net.minecraft.network.protocol.game.ClientboundMapItemDataPacket;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.saveddata.maps.MapDecoration;
import net.minecraft.world.level.saveddata.maps.MapDecorationTypes;
import net.minecraft.world.level.saveddata.maps.MapId;
import net.minecraft.world.level.saveddata.maps.MapItemSavedData;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

/**
 * A player's marker on the maps one player holds, at a place of the mod's (`map_mark`, ABI
 * 54): for them alone, their map's own markers sent again with it a few times a second, since the
 * game sends its own over it whenever they change. Nothing in the map's data changes.
 */
public final class MapMarks {
	/** The longest a mark stays. */
	public static final float MOST_SECONDS = 600;

	private record Mark(Vec3 at, long untilTick) {
	}

	private final Map<UUID, Mark> marks = new HashMap<>();
	private long tick;

	public void mark(ServerPlayer player, Vec3 at, float seconds) {
		marks.put(player.getUUID(), new Mark(at, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));
	}

	public void tick(MinecraftServer server) {
		tick++;
		if (marks.isEmpty() || tick % 5 != 0) {
			return;
		}
		marks.entrySet().removeIf(e -> e.getValue().untilTick() < tick);
		for (var entry : marks.entrySet()) {
			ServerPlayer player = server.getPlayerList().getPlayer(entry.getKey());
			if (player == null) {
				continue;
			}
			for (InteractionHand hand : InteractionHand.values()) {
				send(player, player.getItemInHand(hand), entry.getValue().at());
			}
		}
	}

	private static void send(ServerPlayer player, ItemStack stack, Vec3 at) {
		MapId id = stack.get(DataComponents.MAP_ID);
		if (id == null) {
			return;
		}
		MapItemSavedData data = net.minecraft.world.item.MapItem.getSavedData(id, player.level());
		if (data == null) {
			return;
		}
		Optional<byte[]> spot = MapSpot.spot(data.centerX, data.centerZ, data.scale, at.x, at.z);
		if (spot.isEmpty()) {
			return;
		}
		List<MapDecoration> decorations = new ArrayList<>();
		data.getDecorations().forEach(decorations::add);
		decorations.add(new MapDecoration(MapDecorationTypes.PLAYER, spot.get()[0], spot.get()[1], (byte) 8, Optional.empty()));
		player.connection.send(new ClientboundMapItemDataPacket(id, data.scale, data.locked, Optional.of(decorations), Optional.empty()));
	}

	public void clear() {
		marks.clear();
	}
}
