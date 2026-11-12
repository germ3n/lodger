package craftkit.host.client;

import net.minecraft.client.Minecraft;
import net.minecraft.world.level.levelgen.Heightmap;

/**
 * `spyglass_figure`: while they look through a spyglass, far off on the ground where they look,
 * a figure in their skin looking back, on their client alone; the moment they lower it, it is
 * gone, and once seen it is not there again while the hint lasts.
 */
public final class SpyglassFigure {
	/** Its number among the client's entities, far from the server's and the screenshot's. */
	private static final int ENTITY_ID = -0x4C4F4448;
	/** How far off it stands: farther than the eye picks out a figure without the glass. */
	static final double FAR = 72;

	private static ClientFigure figure;
	/** The hint whose figure was seen and is done. */
	private static long seen = -1;

	private SpyglassFigure() {
	}

	/** Once a tick while the hint lasts. */
	static void tick(Minecraft minecraft, long hint) {
		var player = minecraft.player;
		if (seen == hint) {
			clear(minecraft);
			return;
		}
		if (!player.isScoping()) {
			if (figure != null) {
				clear(minecraft);
				seen = hint;
			}
			return;
		}
		if (figure == null) {
			double yaw = Math.toRadians(player.getYRot());
			double x = player.getX() - Math.sin(yaw) * FAR;
			double z = player.getZ() + Math.cos(yaw) * FAR;
			int y = minecraft.level.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, (int) Math.floor(x), (int) Math.floor(z));
			figure = ClientFigure.add(minecraft, ENTITY_ID, x, y, z, player.getYRot() + 180);
		}
	}

	/** Gone: the glass lowered, the hint over, or the world left. */
	static void clear(Minecraft minecraft) {
		if (figure != null) {
			figure.remove(minecraft);
			figure = null;
		}
	}
}
