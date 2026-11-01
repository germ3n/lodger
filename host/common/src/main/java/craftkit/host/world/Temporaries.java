package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;

/**
 * Entities placed for a while ({@code place_entity} with seconds, ABI 27): one more animal in the
 * pen till noon. Each is gone at its time, at once when its level is about to be saved and
 * closed, and, carrying {@link #TAG}, gone when next seen after a quit: never one left behind.
 */
public final class Temporaries {
	public static final String TAG = "craftkit_temporary";
	/** The longest an entity stays, whatever a mod asks: ten minutes. */
	public static final float MOST_SECONDS = 600;
	private static final int SWEEP_EVERY = 100;

	private final Due<Entity> due = new Due<>();
	private final java.util.Set<Entity> active = new java.util.HashSet<>();
	private long tick;

	public void add(Entity entity, float seconds) {
		entity.addTag(TAG);
		active.add(entity);
		due.add(entity, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20));
	}

	public void tick(MinecraftServer server) {
		tick++;
		for (Entity entity : due.due(tick)) {
			active.remove(entity);
			entity.discard();
		}
		if (tick % SWEEP_EVERY == 0) {
			for (ServerLevel level : server.getAllLevels()) {
				for (Entity entity : level.getAllEntities()) {
					if (entity.entityTags().contains(TAG) && !active.contains(entity)) {
						entity.discard();
					}
				}
			}
		}
	}

	/** Takes away everything placed for a while in a level: it is about to be saved and closed. */
	public void goneIn(ServerLevel level) {
		for (Entity entity : due.all(e -> e.level() == level)) {
			active.remove(entity);
			entity.discard();
		}
	}
}
