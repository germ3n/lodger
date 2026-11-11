package craftkit.host.client;

import com.mojang.authlib.GameProfile;
import com.mojang.blaze3d.pipeline.RenderTarget;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.PlayerInfo;
import net.minecraft.client.player.RemotePlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.levelgen.Heightmap;

import java.io.File;
import java.util.Map;
import java.util.UUID;
import java.util.function.Consumer;

/**
 * `in_the_screenshot`: the screenshot they take is held back a tick, a figure in their skin set
 * far ahead in their view on their client alone, the picture taken with it in, and the figure
 * gone; once while the hint lasts. On their screen a frame at most.
 */
public final class ScreenshotFigure {
	/** Its number among the client's entities, far from any the server hands out. */
	private static final int ENTITY_ID = -0x4C4F4447;
	/** How far ahead of them it stands. */
	private static final double AHEAD = 14;

	private static File dir;
	private static RenderTarget target;
	private static Consumer<Component> done;
	/** The ticks since the figure was set there: 0 when none. */
	private static int since;
	private static UUID uuid;
	/** The screenshot taken again, here: the game's own, not to be held back. */
	private static boolean taking;

	private ScreenshotFigure() {
	}

	/** Whether the game's screenshot is held back for one with the figure in it. */
	public static boolean hold(File screenshotDir, RenderTarget renderTarget, Consumer<Component> message) {
		Minecraft minecraft = Minecraft.getInstance();
		if (taking || since > 0 || minecraft.player == null || minecraft.level == null || !Glitches.screenshotArmed()) {
			return false;
		}
		dir = screenshotDir;
		target = renderTarget;
		done = message;
		if (!place(minecraft)) {
			return false;
		}
		since = 1;
		Glitches.screenshotTaken();

		return true;
	}

	/** Once a client tick: a frame drawn with the figure in it, the picture, and the figure gone. */
	public static void tick(Minecraft minecraft) {
		if (since == 0) {
			return;
		}
		since++;
		if (since < 3) {
			return;
		}
		taking = true;
		try {
			net.minecraft.client.Screenshot.grab(dir, target, done);
		} finally {
			taking = false;
			remove(minecraft);
			since = 0;
		}
	}

	private static boolean place(Minecraft minecraft) {
		try {
			var player = minecraft.player;
			var level = minecraft.level;
			double yaw = Math.toRadians(player.getYRot());
			double x = player.getX() - Math.sin(yaw) * AHEAD;
			double z = player.getZ() + Math.cos(yaw) * AHEAD;
			int y = level.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, (int) Math.floor(x), (int) Math.floor(z));
			uuid = UUID.randomUUID();
			GameProfile profile = new GameProfile(uuid, player.getGameProfile().name(), player.getGameProfile().properties());
			// Their skin by a player entry of its own, as the server's figures have one.
			playerInfos(minecraft).put(uuid, new PlayerInfo(profile, false));
			RemotePlayer figure = new RemotePlayer(level, profile);
			figure.setId(ENTITY_ID);
			float facing = player.getYRot() + 180;
			figure.snapTo(x, y, z, facing, 0);
			figure.setYHeadRot(facing);
			figure.setYBodyRot(facing);
			level.addEntity(figure);
			return true;
		} catch (ReflectiveOperationException | RuntimeException ignored) {
			// The client moved its fields: the screenshot as the game takes it.
			return false;
		}
	}

	private static void remove(Minecraft minecraft) {
		if (minecraft.level != null) {
			minecraft.level.removeEntity(ENTITY_ID, Entity.RemovalReason.DISCARDED);
		}
		try {
			if (uuid != null && minecraft.getConnection() != null) {
				playerInfos(minecraft).remove(uuid);
			}
		} catch (ReflectiveOperationException ignored) {
			// Nothing was put there.
		}
		uuid = null;
	}

	@SuppressWarnings("unchecked")
	private static Map<UUID, PlayerInfo> playerInfos(Minecraft minecraft) throws ReflectiveOperationException {
		var field = net.minecraft.client.multiplayer.ClientPacketListener.class.getDeclaredField("playerInfoMap");
		field.setAccessible(true);
		return (Map<UUID, PlayerInfo>) field.get(minecraft.getConnection());
	}
}
