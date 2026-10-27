package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * Mobs held where they stand, facing a player, attacking no one, for a while ({@code hold}):
 * their AI is off and the host turns them toward the player's eyes each tick, then gives the AI
 * back as it was. A held mob carries {@link #TAG}, so one saved while held (the chunk unloaded,
 * the game quit) is let go the next time it is seen: never a mob left frozen in a world.
 */
public final class Holds {
	/** On every held mob; with {@link #TAG_STILL} too if its AI was already off. */
	public static final String TAG = "craftkit_held";
	public static final String TAG_STILL = "craftkit_held_still";
	/** The longest hold, whatever a mod asks: most of a night. */
	public static final float MOST_SECONDS = 600;
	private static final int SWEEP_EVERY = 100;

	private record Hold(Mob mob, UUID player, long untilTick) {
	}

	private final List<Hold> active = new ArrayList<>();
	private long tick;

	/** Holds a mob for a player; a mob already held is held for the new time. */
	public void add(Mob mob, ServerPlayer player, float seconds) {
		active.removeIf(h -> h.mob() == mob);
		if (!mob.entityTags().contains(TAG)) {
			if (mob.isNoAi()) {
				mob.addTag(TAG_STILL);
			}
			mob.addTag(TAG);
		}
		mob.setNoAi(true);
		mob.setTarget(null);
		mob.getNavigation().stop();
		mob.setDeltaMovement(Vec3.ZERO);
		active.add(new Hold(mob, player.getUUID(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Hold> it = active.iterator();
		while (it.hasNext()) {
			Hold hold = it.next();
			Mob mob = hold.mob();
			if (mob.isRemoved()) {
				it.remove();
				continue;
			}
			if (hold.untilTick() < tick) {
				release(mob);
				it.remove();
				continue;
			}
			ServerPlayer player = server.getPlayerList().getPlayer(hold.player());
			if (player != null && player.level() == mob.level()) {
				Vec3 to = player.getEyePosition().subtract(mob.getEyePosition());
				float yaw = Facing.yaw(to.x, to.z);
				mob.setYRot(yaw);
				mob.setYHeadRot(yaw);
				mob.setYBodyRot(yaw);
				mob.setXRot(Facing.pitch(to.x, to.y, to.z));
			}
		}

		if (tick % SWEEP_EVERY == 0) {
			for (ServerLevel level : server.getAllLevels()) {
				for (Entity entity : level.getAllEntities()) {
					if (entity instanceof Mob mob && mob.entityTags().contains(TAG) && active.stream().noneMatch(h -> h.mob() == mob)) {
						release(mob);
					}
				}
			}
		}
	}

	/** Lets every mob in a level go: it is about to be saved and closed. */
	public void releaseIn(ServerLevel level) {
		active.removeIf(hold -> {
			if (hold.mob().level() == level) {
				release(hold.mob());
				return true;
			}
			return false;
		});
	}

	private static void release(Mob mob) {
		mob.setNoAi(mob.entityTags().contains(TAG_STILL));
		mob.removeTag(TAG);
		mob.removeTag(TAG_STILL);
	}

	public int count() {
		return active.size();
	}
}
