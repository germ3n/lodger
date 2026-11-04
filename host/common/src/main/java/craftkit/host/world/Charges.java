package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Mob;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * Animals turned on a player ({@code charge}, ABI 27; owner, 2026-11-09: "passive mobs aggro"):
 * each runs at the player and hits while it touches them, once a second, so many times at most,
 * and with {@code spare} never below one half heart: a nip at stage 3, real harm from 4, never
 * a kill. Nothing about the animal is changed, so nothing needs putting back.
 */
public final class Charges {
	/** The longest a charge lasts, whatever a mod asks. */
	public static final float MOST_SECONDS = 60;
	private static final double REACH = 1.6;
	private static final int HIT_EVERY = 20;

	private static final class Charge {
		final Mob mob;
		final UUID player;
		final float damage;
		final boolean spare;
		final long untilTick;
		int hitsLeft;
		long nextHit;

		Charge(Mob mob, UUID player, float damage, int hits, boolean spare, long untilTick) {
			this.mob = mob;
			this.player = player;
			this.damage = damage;
			this.hitsLeft = hits;
			this.spare = spare;
			this.untilTick = untilTick;
		}
	}

	private final List<Charge> active = new ArrayList<>();
	private long tick;

	public void add(Mob mob, ServerPlayer player, float damage, int hits, float seconds, boolean spare) {
		active.removeIf(c -> c.mob == mob);
		active.add(new Charge(mob, player.getUUID(), damage, hits, spare, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Charge> it = active.iterator();
		while (it.hasNext()) {
			Charge charge = it.next();
			Mob mob = charge.mob;
			ServerPlayer player = server.getPlayerList().getPlayer(charge.player);
			if (mob.isRemoved() || player == null || player.level() != mob.level() || player.isDeadOrDying()
					|| charge.hitsLeft <= 0 || charge.untilTick < tick) {
				mob.getNavigation().stop();
				it.remove();
				continue;
			}
			mob.getNavigation().moveTo(player, 1.6);
			mob.getLookControl().setLookAt(player);
			if (mob.distanceTo(player) <= REACH && tick >= charge.nextHit) {
				// Spared, as the blow lands: the game scales a mob's hit by the difficulty after
				// this (half again on hard), so the room left is divided by that first (until
				// 2026-11-10 a spared hit could kill on hard). Never spared is never asked for.
				float scale = switch (player.level().getDifficulty()) {
					case HARD -> 1.5f;
					default -> 1.0f;
				};
				float harm = Spare.harm(charge.damage, player.getHealth(), scale);
				if (harm > 0) {
					player.hurtServer((ServerLevel) mob.level(), mob.level().damageSources().mobAttack(mob), harm);
				}
				charge.hitsLeft--;
				charge.nextHit = tick + HIT_EVERY;
			}
		}
	}

	/** Everything for every player, forgotten: the world is closing, and none of it is the next one's. */
	public void clear() {
		active.clear();
	}
}
