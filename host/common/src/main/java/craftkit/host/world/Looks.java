package craftkit.host.world;

import net.minecraft.world.entity.Mob;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * Mobs told to look somewhere for a while: the look control is set every tick until the time is
 * up, since the game resets a mob's look each tick on its own.
 */
public final class Looks {
	private record Look(Mob mob, Vec3 at, long untilTick) {
	}

	private final List<Look> active = new ArrayList<>();
	private long tick;

	public void add(Mob mob, Vec3 at, float seconds) {
		active.add(new Look(mob, at, tick + (long) (seconds * 20)));
	}

	public void tick() {
		tick++;
		Iterator<Look> it = active.iterator();
		while (it.hasNext()) {
			Look look = it.next();
			if (look.untilTick() < tick || look.mob().isRemoved()) {
				it.remove();
				continue;
			}

			look.mob().getLookControl().setLookAt(look.at());
		}
	}

	public int count() {
		return active.size();
	}
}
