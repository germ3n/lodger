package craftkit.host.world;

import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Warped animals ({@code warp_mob}, ABI 25; owner): an animal frozen with its head
 * always on a player, perhaps too tall, perhaps upside down (named Dinnerbone, the name hidden),
 * perhaps a step closer each time the player looks away; then put back as it was, where it was.
 * A warped mob carries {@link #TAG} and a tag of how it was ({@link WarpMath#tag}), so one saved
 * while warped is put back the next time it is seen: never an animal left wrong in a world.
 */
public final class Warps {
	public static final String TAG = "craftkit_warped";
	/** The longest warp, whatever a mod asks. */
	public static final float MOST_SECONDS = 120;
	/** Within this many blocks of the player, it is over at once. */
	private static final double ENDS_WITHIN = 6;
	/** It steps closer no nearer than this. */
	private static final double CREEPS_TO = 8;
	private static final int CREEP_EVERY = 30;
	private static final int SWEEP_EVERY = 100;

	private static final class Warp {
		final Mob mob;
		final UUID player;
		final long untilTick;
		final boolean flip;
		final boolean creep;
		final float bodyYaw;
		/** Never still (owner): its head snapped aside till this tick, and when next. */
		long snapUntil;
		float snapYaw;
		long nextTwitch;
		long nextSound;

		Warp(Mob mob, UUID player, long untilTick, boolean flip, boolean creep) {
			this.mob = mob;
			this.player = player;
			this.untilTick = untilTick;
			this.flip = flip;
			this.creep = creep;
			this.bodyYaw = mob.getYRot();
		}
	}

	private final List<Warp> active = new ArrayList<>();
	private long tick;

	/** Warps a mob for a player; one already warped is not warped again. */
	public boolean add(Mob mob, ServerPlayer player, float seconds, double scale, boolean flip, boolean creep, Vec3 to) {
		if (mob.entityTags().contains(TAG) || mob.entityTags().contains(Holds.TAG)) {
			return false;
		}
		boolean named = mob.getCustomName() != null;
		AttributeInstance size = mob.getAttribute(Attributes.SCALE);
		double was = size == null ? 1.0 : size.getBaseValue();
		mob.addTag(TAG);
		mob.addTag(WarpMath.tag(new WarpMath.Was(mob.getX(), mob.getY(), mob.getZ(), mob.getYRot(), was, mob.isNoAi())));
		mob.setNoAi(true);
		mob.getNavigation().stop();
		mob.setDeltaMovement(Vec3.ZERO);
		if (size != null && scale > 0 && scale != 1.0) {
			size.setBaseValue(Math.min(scale, 3.0));
		}
		boolean flipped = flip && !named;
		if (flipped) {
			mob.setCustomName(Component.literal("Dinnerbone"));
			mob.setCustomNameVisible(false);
		}
		if (to != null) {
			mob.teleportTo(to.x, to.y, to.z);
		}
		active.add(new Warp(mob, player.getUUID(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20), flipped, creep));

		return true;
	}

	/** A mob's own ambient sound (the game keeps the getter to itself), or null. */
	private static net.minecraft.sounds.SoundEvent ambient(Mob mob) {
		for (Class<?> c = mob.getClass(); c != null && c != Object.class; c = c.getSuperclass()) {
			try {
				java.lang.reflect.Method m = c.getDeclaredMethod("getAmbientSound");
				m.setAccessible(true);
				return (net.minecraft.sounds.SoundEvent) m.invoke(mob);
			} catch (NoSuchMethodException ignored) {
				// Up a class.
			} catch (ReflectiveOperationException | RuntimeException e) {
				return null;
			}
		}
		return null;
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Warp> it = active.iterator();
		while (it.hasNext()) {
			Warp warp = it.next();
			Mob mob = warp.mob;
			if (mob.isRemoved()) {
				it.remove();
				continue;
			}
			ServerPlayer player = server.getPlayerList().getPlayer(warp.player);
			boolean here = player != null && player.level() == mob.level();
			if (warp.untilTick < tick || (here && player.distanceTo(mob) <= ENDS_WITHIN)) {
				restore(mob, warp.flip);
				it.remove();
				continue;
			}
			if (!here) {
				continue;
			}
			// The head on them, but for a snap aside now and then; the body as it stood, but for a
			// shudder.
			Vec3 to = player.getEyePosition().subtract(mob.getEyePosition());
			var random = mob.getRandom();
			if (tick >= warp.nextTwitch) {
				warp.snapYaw = Facing.yaw(to.x, to.z) + (random.nextBoolean() ? 1 : -1) * (60 + random.nextFloat() * 110);
				warp.snapUntil = tick + 3 + random.nextInt(4);
				warp.nextTwitch = tick + 25 + random.nextInt(60);
				mob.teleportTo(mob.getX() + (random.nextDouble() - 0.5) * 0.12, mob.getY(), mob.getZ() + (random.nextDouble() - 0.5) * 0.12);
			}
			if (tick < warp.snapUntil) {
				mob.setYHeadRot(warp.snapYaw);
				mob.setXRot((random.nextFloat() - 0.5f) * 80);
			} else {
				mob.setYHeadRot(Facing.yaw(to.x, to.z));
				mob.setXRot(Facing.pitch(to.x, to.y, to.z));
			}
			mob.setYBodyRot(warp.bodyYaw);
			// Its own voice, slowed, to them alone.
			if (tick >= warp.nextSound) {
				warp.nextSound = tick + 100 + random.nextInt(120);
				net.minecraft.sounds.SoundEvent voice = ambient(mob);
				if (voice != null) {
					player.connection.send(new net.minecraft.network.protocol.game.ClientboundSoundPacket(net.minecraft.core.Holder.direct(voice),
							net.minecraft.sounds.SoundSource.HOSTILE, mob.getX(), mob.getY(), mob.getZ(), 1.0f, 0.45f + random.nextFloat() * 0.1f, random.nextLong()));
				}
			}
			if (warp.creep && tick % CREEP_EVERY == 0 && player.distanceTo(mob) > CREEPS_TO) {
				Vec3 view = player.getViewVector(1f);
				Vec3 at = mob.position().subtract(player.getEyePosition());
				if (!WarpMath.lookedAt(view.x, view.y, view.z, at.x, at.y, at.z)) {
					double[] step = WarpMath.stepToward(mob.getX(), mob.getZ(), player.getX(), player.getZ());
					net.minecraft.core.BlockPos ground = net.minecraft.core.BlockPos.containing(step[0], mob.getY() - 0.5, step[1]);
					ServerLevel level = (ServerLevel) mob.level();
					if (level.getBlockState(ground).isFaceSturdy(level, ground, net.minecraft.core.Direction.UP)
							&& level.getBlockState(ground.above()).getCollisionShape(level, ground.above()).isEmpty()) {
						mob.teleportTo(step[0], mob.getY(), step[1]);
					}
				}
			}
		}

		if (tick % SWEEP_EVERY == 0) {
			for (ServerLevel level : server.getAllLevels()) {
				for (Entity entity : level.getAllEntities()) {
					if (entity instanceof Mob mob && mob.entityTags().contains(TAG) && active.stream().noneMatch(w -> w.mob == mob)) {
						restore(mob, "Dinnerbone".equals(mob.getCustomName() == null ? "" : mob.getCustomName().getString()));
					}
				}
			}
		}
	}

	/** Puts back every warped mob in a level: it is about to be saved and closed. */
	public void restoreIn(ServerLevel level) {
		active.removeIf(warp -> {
			if (warp.mob.level() == level) {
				restore(warp.mob, warp.flip);
				return true;
			}
			return false;
		});
	}

	private static void restore(Mob mob, boolean flipped) {
		Optional<WarpMath.Was> was = Optional.empty();
		String fromTag = null;
		for (String tag : mob.entityTags()) {
			if (tag.startsWith(WarpMath.FROM)) {
				fromTag = tag;
				was = WarpMath.read(tag);
			}
		}
		if (was.isPresent()) {
			WarpMath.Was w = was.get();
			AttributeInstance size = mob.getAttribute(Attributes.SCALE);
			if (size != null) {
				size.setBaseValue(w.scale());
			}
			mob.setNoAi(w.noAi());
			mob.teleportTo(w.x(), w.y(), w.z());
			mob.setYRot(w.yaw());
			mob.setYHeadRot(w.yaw());
			mob.setYBodyRot(w.yaw());
		} else {
			mob.setNoAi(false);
		}
		if (flipped) {
			mob.setCustomName(null);
			mob.setCustomNameVisible(false);
		}
		mob.removeTag(TAG);
		if (fromTag != null) {
			mob.removeTag(fromTag);
		}
	}

	public int count() {
		return active.size();
	}
}
