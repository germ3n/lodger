package craftkit.host.world;

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

		Approach(Mob mob, UUID player, double stop, long untilTick) {
			this.mob = mob;
			this.player = player;
			this.stop = stop;
			this.untilTick = untilTick;
		}
	}

	private final List<Approach> active = new ArrayList<>();
	private long tick;

	public void add(Mob mob, ServerPlayer player, float stop, float seconds) {
		active.removeIf(a -> a.mob == mob);
		active.add(new Approach(mob, player.getUUID(), Math.max(2, Math.min(stop, 16)), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));
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
				it.remove();
				continue;
			}
			if (mob.distanceTo(player) > approach.stop) {
				if (tick % 10 == 0 || mob.getNavigation().isDone()) {
					mob.getNavigation().moveTo(player, PACE);
				}
			} else {
				mob.getNavigation().stop();
			}
			mob.getLookControl().setLookAt(player);
		}
	}
}
