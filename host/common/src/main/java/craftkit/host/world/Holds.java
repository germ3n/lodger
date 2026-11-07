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

	/** On a creeping mob: where it was held, `x,y,z`, so one saved mid-creep goes back there. */
	public static final String FROM = "craftkit_held_from:";
	/** A creeping mob comes no nearer than this. */
	private static final double CREEPS_TO = 6;
	private static final int CREEP_EVERY = 30;

	/** The hits a held mob takes unmoved before it wakes: the first it ignores, the second frees it. */
	public static final int HITS_TO_WAKE = 2;

	private static final class Hold {
		final Mob mob;
		final UUID player;
		final long untilTick;
		/** The point it faces in place of the player, or null (ABI 27: the house). */
		final Vec3 at;
		/** Within this many blocks of the player it turns to them after all (the packed villagers); 0 never. */
		final double turnWithin;
		/** Where it was held, when it creeps (ABI 37); null when it does not. */
		final Vec3 from;
		float health;
		int hits;

		Hold(Mob mob, UUID player, long untilTick, Vec3 at, double turnWithin, boolean creep) {
			this.mob = mob;
			this.player = player;
			this.untilTick = untilTick;
			this.at = at;
			this.turnWithin = turnWithin;
			this.from = creep ? mob.position() : null;
			this.health = mob.getHealth();
		}

		Mob mob() {
			return mob;
		}
	}

	private final List<Hold> active = new ArrayList<>();
	private long tick;

	/** Holds a mob for a player; a mob already held is held for the new time. */
	public void add(Mob mob, ServerPlayer player, float seconds) {
		add(mob, player, seconds, null);
	}

	/** Holds a mob, facing {@code at} when given, else the player. */
	public void add(Mob mob, ServerPlayer player, float seconds, Vec3 at) {
		add(mob, player, seconds, at, 0);
	}

	/** Holds a mob facing {@code at}, turning to the player when they come within {@code turnWithin}. */
	public void add(Mob mob, ServerPlayer player, float seconds, Vec3 at, double turnWithin) {
		add(mob, player, seconds, at, turnWithin, false);
	}

	/**
	 * Holds a mob; with {@code creep} it comes a step nearer along the ground each moment the
	 * player looks away, to six blocks, and is back where it was held the moment they look at
	 * it (`hostiles_close`, batch 16), and back there when let go.
	 */
	public void add(Mob mob, ServerPlayer player, float seconds, Vec3 at, double turnWithin, boolean creep) {
		Hold before = active.stream().filter(h -> h.mob() == mob).findFirst().orElse(null);
		if (before != null && before.from != null) {
			mob.teleportTo(before.from.x, before.from.y, before.from.z);
		}
		active.removeIf(h -> h.mob() == mob);
		mob.entityTags().stream().filter(t -> t.startsWith(FROM)).toList().forEach(mob::removeTag);
		if (creep) {
			mob.addTag(FROM + mob.getX() + "," + mob.getY() + "," + mob.getZ());
		}
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
		active.add(new Hold(mob, player.getUUID(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20), at, turnWithin, creep));
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
			// Hit once it stands unmoved, still staring; hit again and that one wakes, alone
			// (owner: "a mix of" breaking the spell and staying frozen).
			if (mob.getHealth() < hold.health) {
				hold.hits++;
			}
			hold.health = mob.getHealth();
			if (hold.untilTick < tick || hold.hits >= HITS_TO_WAKE) {
				release(mob);
				it.remove();
				continue;
			}
			ServerPlayer player = server.getPlayerList().getPlayer(hold.player);
			boolean near = player != null && player.level() == mob.level() && hold.turnWithin > 0 && player.distanceTo(mob) <= hold.turnWithin;
			if (hold.at != null || (player != null && player.level() == mob.level())) {
				Vec3 to = (hold.at != null && !near ? hold.at : player.getEyePosition()).subtract(mob.getEyePosition());
				float yaw = Facing.yaw(to.x, to.z);
				mob.setYRot(yaw);
				mob.setYHeadRot(yaw);
				mob.setYBodyRot(yaw);
				mob.setXRot(Facing.pitch(to.x, to.y, to.z));
			}
			if (hold.from != null && player != null && player.level() == mob.level()) {
				creep(hold, mob, player);
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

	/** Looked at, back where it was held; looked away from, now and then a step nearer. */
	private void creep(Hold hold, Mob mob, ServerPlayer player) {
		Vec3 view = player.getViewVector(1f);
		Vec3 to = mob.position().subtract(player.getEyePosition());
		if (WarpMath.lookedAt(view.x, view.y, view.z, to.x, to.y, to.z)) {
			if (mob.position().distanceToSqr(hold.from) > 0.01) {
				mob.teleportTo(hold.from.x, hold.from.y, hold.from.z);
			}
			return;
		}
		if (tick % CREEP_EVERY != 0 || player.distanceTo(mob) <= CREEPS_TO) {
			return;
		}
		double[] step = WarpMath.stepToward(mob.getX(), mob.getZ(), player.getX(), player.getZ());
		ServerLevel level = (ServerLevel) mob.level();
		// The ground there, a block up or down from where it stands, with room for it.
		for (int dy : new int[] {0, 1, -1}) {
			double y = Math.floor(mob.getY()) + dy;
			net.minecraft.core.BlockPos feet = net.minecraft.core.BlockPos.containing(step[0], y, step[1]);
			if (level.getBlockState(feet.below()).isFaceSturdy(level, feet.below(), net.minecraft.core.Direction.UP)
					&& level.noCollision(mob, mob.getDimensions(mob.getPose()).makeBoundingBox(new Vec3(step[0], y, step[1])))) {
				mob.teleportTo(step[0], y, step[1]);
				return;
			}
		}
	}

	private static void release(Mob mob) {
		for (String tag : mob.entityTags().stream().filter(t -> t.startsWith(FROM)).toList()) {
			String[] at = tag.substring(FROM.length()).split(",");
			try {
				mob.teleportTo(Double.parseDouble(at[0]), Double.parseDouble(at[1]), Double.parseDouble(at[2]));
			} catch (RuntimeException ignored) {
				// A tag not of ours: left where it stands.
			}
			mob.removeTag(tag);
		}
		mob.setNoAi(mob.entityTags().contains(TAG_STILL));
		mob.removeTag(TAG);
		mob.removeTag(TAG_STILL);
	}

	public int count() {
		return active.size();
	}
}
