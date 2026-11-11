package craftkit.host.client;

import com.mojang.authlib.GameProfile;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.PlayerInfo;
import net.minecraft.client.player.RemotePlayer;
import net.minecraft.world.entity.Entity;

import java.util.Map;
import java.util.UUID;

/**
 * A figure in their own skin on their client alone, made by the client (the server's figures
 * are packets from the host): a player entity of its own with a player entry of its own, so the
 * skin is theirs; gone with {@link #remove}. For the client's own glitches: the screenshot, the
 * third person.
 */
final class ClientFigure {
	final int id;
	final UUID uuid;
	final RemotePlayer entity;

	private ClientFigure(int id, UUID uuid, RemotePlayer entity) {
		this.id = id;
		this.uuid = uuid;
		this.entity = entity;
	}

	/** One at a place, facing {@code yaw}; null when the client moved its fields. */
	static ClientFigure add(Minecraft minecraft, int id, double x, double y, double z, float yaw) {
		try {
			var player = minecraft.player;
			UUID uuid = UUID.randomUUID();
			GameProfile profile = new GameProfile(uuid, player.getGameProfile().name(), player.getGameProfile().properties());
			playerInfos(minecraft).put(uuid, new PlayerInfo(profile, false));
			RemotePlayer figure = new RemotePlayer(minecraft.level, profile);
			figure.setId(id);
			figure.snapTo(x, y, z, yaw, 0);
			figure.setYHeadRot(yaw);
			figure.setYBodyRot(yaw);
			minecraft.level.addEntity(figure);
			return new ClientFigure(id, uuid, figure);
		} catch (ReflectiveOperationException | RuntimeException ignored) {
			return null;
		}
	}

	/** Where it stands now, facing {@code yaw}. */
	void move(double x, double y, double z, float yaw) {
		entity.snapTo(x, y, z, yaw, 0);
		entity.setYHeadRot(yaw);
		entity.setYBodyRot(yaw);
	}

	void remove(Minecraft minecraft) {
		if (minecraft.level != null) {
			minecraft.level.removeEntity(id, Entity.RemovalReason.DISCARDED);
		}
		try {
			if (minecraft.getConnection() != null) {
				playerInfos(minecraft).remove(uuid);
			}
		} catch (ReflectiveOperationException ignored) {
			// Nothing was put there.
		}
	}

	@SuppressWarnings("unchecked")
	private static Map<UUID, PlayerInfo> playerInfos(Minecraft minecraft) throws ReflectiveOperationException {
		var field = net.minecraft.client.multiplayer.ClientPacketListener.class.getDeclaredField("playerInfoMap");
		field.setAccessible(true);
		return (Map<UUID, PlayerInfo>) field.get(minecraft.getConnection());
	}
}
