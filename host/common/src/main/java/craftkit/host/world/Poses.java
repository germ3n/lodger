package craftkit.host.world;

import net.minecraft.core.Rotations;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * Armour stands told to follow a player with their heads for a while ({@code pose_stand} with
 * {@code follow}): each tick the head turns toward the player, except while that player looks
 * straight at the stand, when it holds still, the way the docs' {@code stand_watch} has it.
 */
public final class Poses {
	private record Watch(ArmorStand stand, ServerPlayer player, long untilTick) {
	}

	private final List<Watch> active = new ArrayList<>();
	private long tick;

	public void add(ArmorStand stand, ServerPlayer player, float seconds) {
		active.add(new Watch(stand, player, tick + (long) (seconds * 20)));
	}

	public void tick() {
		tick++;
		Iterator<Watch> it = active.iterator();
		while (it.hasNext()) {
			Watch watch = it.next();
			if (watch.untilTick() < tick || watch.stand().isRemoved() || watch.player().isRemoved()) {
				it.remove();
				continue;
			}

			if (lookingAt(watch.player(), watch.stand())) {
				continue;
			}

			watch.stand().setHeadPose(headToward(watch.stand(), watch.player().getEyePosition()));
		}
	}

	/** The head rotation that points a stand's face at {@code at}. */
	public static Rotations headToward(ArmorStand stand, Vec3 at) {
		Vec3 eye = stand.position().add(0, stand.getBbHeight() * 0.9, 0);
		Vec3 to = at.subtract(eye);
		double flat = Math.sqrt(to.x * to.x + to.z * to.z);
		float pitch = (float) -Math.toDegrees(Math.atan2(to.y, flat));
		float yaw = (float) Math.toDegrees(Math.atan2(-to.x, to.z)) - stand.getYRot();

		return new Rotations(pitch, yaw, 0);
	}

	private static boolean lookingAt(ServerPlayer player, ArmorStand stand) {
		Vec3 to = stand.position().add(0, stand.getBbHeight() / 2, 0).subtract(player.getEyePosition());

		return to.length() < 16 && to.normalize().dot(player.getViewVector(1.0f)) > Math.cos(Math.toRadians(8));
	}

	public int count() {
		return active.size();
	}
}
