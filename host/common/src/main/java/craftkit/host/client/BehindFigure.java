package craftkit.host.client;

import net.minecraft.client.CameraType;
import net.minecraft.client.Minecraft;

/**
 * `only_in_third_person`: while the hint lasts, a figure in their skin stands right behind them
 * whenever their camera is out of their head (F5), facing the way they face, moving as they
 * move; in first person there is nothing there.
 */
public final class BehindFigure {
	private static final int ENTITY_ID = -0x4C4F4448;
	/** How far behind them it stands. */
	private static final double BEHIND = 1.4;

	private static ClientFigure figure;

	private BehindFigure() {
	}

	/** Once a client tick while the hint lasts. */
	static void tick(Minecraft minecraft) {
		var player = minecraft.player;
		boolean outside = minecraft.options.getCameraType() != CameraType.FIRST_PERSON;
		if (!outside) {
			clear(minecraft);
			return;
		}
		double yaw = Math.toRadians(player.getYRot());
		double x = player.getX() + Math.sin(yaw) * BEHIND;
		double z = player.getZ() - Math.cos(yaw) * BEHIND;
		if (figure == null) {
			figure = ClientFigure.add(minecraft, ENTITY_ID, x, player.getY(), z, player.getYRot());
		} else {
			figure.move(x, player.getY(), z, player.getYRot());
		}
	}

	/** The hint over, or first person again: nothing there. */
	static void clear(Minecraft minecraft) {
		if (figure != null) {
			figure.remove(minecraft);
			figure = null;
		}
	}
}
