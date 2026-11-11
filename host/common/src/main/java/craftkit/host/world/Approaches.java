package craftkit.host.world;

import net.minecraft.world.item.ItemStack;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Mob;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * Animals walking up to a player together and stopping a few blocks off, facing them
 * ({@code approach}, ABI 37; `pack`, batch 16). Only their path and their look are steered,
 * tick by tick; their AI stays on and nothing about them is changed, so nothing is put back.
 */
public final class Approaches {
	/** The longest an approach lasts, whatever a mod asks. */
	public static final float MOST_SECONDS = 90;
	/** A walk, not a run. */
	private static final double PACE = 1.0;

	private static final class Approach {
		final Mob mob;
		final UUID player;
		final double stop;
		final long untilTick;
		/** How fast against its own walk (ABI 44): 1 a walk, 0.35 a slow, slow one. */
		final double pace;
		/** Once there: `stare`, `leave`, `hit`, or "" to stand facing them (ABI 44). */
		final String then;
		final float damage;
		/** With `give`: what it brings, and where it is kept if it never gets there (ABI 51). */
		ItemStack give = ItemStack.EMPTY;
		WorldData data;
		String kept = "";
		/** When it got there, 0 before; and once going, where to. */
		long arrived;
		net.minecraft.world.phys.Vec3 away;

		Approach(Mob mob, UUID player, double stop, long untilTick, double pace, String then, float damage) {
			this.mob = mob;
			this.player = player;
			this.stop = stop;
			this.untilTick = untilTick;
			this.pace = pace;
			this.then = then;
			this.damage = damage;
		}
	}

	private final List<Approach> active = new ArrayList<>();
	private long tick;

	public void add(Mob mob, ServerPlayer player, float stop, float seconds) {
		add(mob, player, stop, seconds, 1, "", 0);
	}

	/**
	 * One walking at them to bring them {@code give}, dropped at their feet as it gets there and
	 * then going its way; kept again under {@code kept} if it never gets there (ABI 51).
	 */
	public void addGiving(Mob mob, ServerPlayer player, float stop, float seconds, float pace, ItemStack give, WorldData data, String kept) {
		add(mob, player, stop, seconds, pace, "give", 0);
		Approach approach = active.get(active.size() - 1);
		approach.give = give;
		approach.data = data;
		approach.kept = kept;
	}

	/**
	 * One walking to a place instead of at them, and once there (or after twenty seconds) as it
	 * was (ABI 54): a pet goes back to its owner of itself.
	 */
	public void addTo(Mob mob, ServerPlayer player, net.minecraft.world.phys.Vec3 to, float seconds) {
		add(mob, player, 1.5f, seconds, 1, "", 0);
		Approach approach = active.get(active.size() - 1);
		approach.away = to;
		approach.arrived = tick;
	}

	/** What it carried back where it was kept: it never got there. */
	private static void keepAgain(Approach approach) {
		if (!approach.give.isEmpty() && approach.data != null) {
			approach.data.keep(approach.kept, approach.give);
			approach.give = ItemStack.EMPTY;
		}
	}

	/** One walking at them at `pace`, and then what it does once there (ABI 44). */
	public void add(Mob mob, ServerPlayer player, float stop, float seconds, float pace, String then, float damage) {
		// A mob walks one way at a time: an earlier walk of it ends here, what it was bringing
		// kept again (until now the book it carried was gone).
		active.removeIf(a -> {
			if (a.mob == mob) {
				keepAgain(a);
				return true;
			}
			return false;
		});
		active.add(new Approach(mob, player.getUUID(), Math.max(1.2, Math.min(stop, 16)), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20),
				Math.max(0.2, Math.min(pace, 1.5)), then, Math.max(0, Math.min(damage, 6))));
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Approach> it = active.iterator();
		while (it.hasNext()) {
			Approach approach = it.next();
			Mob mob = approach.mob;
			ServerPlayer player = server.getPlayerList().getPlayer(approach.player);
			if (mob.isRemoved() || player == null || player.level() != mob.level() || approach.untilTick < tick
					|| mob.entityTags().contains(Holds.TAG) || mob.entityTags().contains(Warps.TAG)) {
				mob.getNavigation().stop();
				keepAgain(approach);
				it.remove();
				continue;
			}
			// Going its way: to where it turned, then free.
			if (approach.away != null) {
				if (tick % 10 == 0 || mob.getNavigation().isDone()) {
					mob.getNavigation().moveTo(approach.away.x, approach.away.y, approach.away.z, 1.0);
				}
				if (mob.position().distanceTo(approach.away) < 2 || tick - approach.arrived > 20 * 20) {
					mob.getNavigation().stop();
					it.remove();
				}
				continue;
			}
			if (approach.arrived == 0 && mob.distanceTo(player) > approach.stop) {
				if (tick % 10 == 0 || mob.getNavigation().isDone()) {
					mob.getNavigation().moveTo(player, approach.pace * PACE);
				}
			} else {
				mob.getNavigation().stop();
				if (approach.arrived == 0) {
					approach.arrived = tick;
				}
				long there = tick - approach.arrived;
				switch (approach.then) {
					case "hit" -> {
						if (there == 10) {
							mob.swing(net.minecraft.world.InteractionHand.MAIN_HAND, net.minecraft.world.item.component.SwingAnimation.DEFAULT, true);
							float scale = player.level().getDifficulty() == net.minecraft.world.Difficulty.HARD ? 1.5f : 1f;
							float harm = Spare.harm(approach.damage, player.getHealth(), scale);
							if (harm > 0) {
								player.hurtServer((net.minecraft.server.level.ServerLevel) mob.level(), mob.level().damageSources().mobAttack(mob), harm);
							}
						}
						if (there >= 30) {
							approach.away = awayFrom(mob, player);
						}
					}
					case "leave" -> {
						if (there >= 30) {
							approach.away = awayFrom(mob, player);
						}
					}
					case "give" -> {
						// At their feet, for good, and it goes its way.
						if (there == 10 && !approach.give.isEmpty() && mob.level() instanceof net.minecraft.server.level.ServerLevel level) {
							net.minecraft.world.entity.item.ItemEntity dropped = new net.minecraft.world.entity.item.ItemEntity(level, player.getX(), player.getY() + 0.3, player.getZ(), approach.give);
							dropped.setUnlimitedLifetime();
							level.addFreshEntity(dropped);
							approach.give = ItemStack.EMPTY;
						}
						if (there >= 40) {
							approach.away = awayFrom(mob, player);
						}
					}
					case "stare" -> {
						if (there >= 100) {
							it.remove();
							continue;
						}
					}
					default -> {
					}
				}
			}
			mob.getLookControl().setLookAt(player);
		}
	}

	/** Sixteen blocks on, straight away from them. */
	private static net.minecraft.world.phys.Vec3 awayFrom(Mob mob, ServerPlayer player) {
		net.minecraft.world.phys.Vec3 way = mob.position().subtract(player.position()).multiply(1, 0, 1).normalize();
		return mob.position().add(way.scale(16));
	}

	/** Everything for every player, forgotten: the world is closing, and none of it is the next one's. */
	public void clear() {
		// What one was bringing is kept again, never lost with the world's close.
		active.forEach(Approaches::keepAgain);
		active.clear();
	}
}
