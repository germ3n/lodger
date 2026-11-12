package craftkit.host.world;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.Container;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.item.ItemStack;

/**
 * Entities placed for a while ({@code place_entity} with seconds, ABI 27): one more animal in the
 * pen till noon. Each is gone at its time, at once when its level is about to be saved and
 * closed, and, carrying {@link #TAG}, gone when next seen after a quit: never one left behind.
 */
public final class Temporaries {
	public static final String TAG = "craftkit_temporary";
	/** On a temporary hostile set on a player (ABI 54): its hits never kill, its death drops nothing. */
	public static final String SPARES = "craftkit_spares";
	/** The longest an entity stays, whatever a mod asks: ten minutes. */
	public static final float MOST_SECONDS = 600;
	private static final int SWEEP_EVERY = 100;

	private final Due<Entity> due = new Due<>();
	private final java.util.Set<Entity> active = new java.util.HashSet<>();
	private long tick;

	public void add(Entity entity, float seconds) {
		entity.addTag(TAG);
		// Nothing of value from what is only here a while (the rule: it makes nothing worth more
		// than paper): it cannot be hurt, so it never dies to drop its meat or leather, the
		// squid its ink; a sheep comes shorn, a chicken lays no egg in its time, and an adult
		// cannot breed. What it carries of theirs is still dropped when it goes (`gone`).
		entity.setPermanentlyInvulnerable(true);
		if (entity instanceof LivingEntity living) {
			living.skipDropExperience();
		}
		if (entity instanceof net.minecraft.world.entity.animal.sheep.Sheep sheep) {
			sheep.setSheared(true);
		}
		if (entity instanceof net.minecraft.world.entity.animal.chicken.Chicken chicken) {
			chicken.eggTime = (int) (MOST_SECONDS * 20) + 200;
		}
		if (entity instanceof net.minecraft.world.entity.AgeableMob ageable && ageable.getAge() >= 0) {
			ageable.setAge((int) (MOST_SECONDS * 20) + 200);
		}
		active.add(entity);
		due.add(entity, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20));
	}

	/**
	 * A hostile placed for a while and set on {@code player} (ABI 54): it may be hurt and killed,
	 * gone then with nothing dropped ({@link #vanishes}); its hits never kill ({@link #spares});
	 * its attack at {@code harm} when above 0, before the difficulty's scaling.
	 */
	public void addHostile(net.minecraft.world.entity.Mob mob, net.minecraft.server.level.ServerPlayer player, float seconds, float harm) {
		mob.addTag(TAG);
		mob.addTag(SPARES);
		mob.skipDropExperience();
		mob.setPersistenceRequired();
		var attack = mob.getAttribute(net.minecraft.world.entity.ai.attributes.Attributes.ATTACK_DAMAGE);
		if (attack != null && harm > 0) {
			attack.setBaseValue(harm);
		}
		mob.setTarget(player);
		if (mob instanceof net.minecraft.world.entity.NeutralMob neutral) {
			neutral.setPersistentAngerTarget(net.minecraft.world.entity.EntityReference.of(player));
			neutral.startPersistentAngerTimer();
		}
		active.add(mob);
		due.add(mob, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20));
	}

	/** Whether a hit from this source is a spared hostile's: such a hit never kills. */
	public static boolean spares(net.minecraft.world.damagesource.DamageSource source) {
		Entity by = source.getEntity();
		return by != null && by.entityTags().contains(SPARES);
	}

	/**
	 * Whether a dying entity is a spared hostile, to vanish rather than die: a puff, nothing
	 * dropped, no experience. The loaders cancel its death when it is.
	 */
	public static boolean vanishes(Entity entity) {
		if (!entity.entityTags().contains(SPARES) || !(entity.level() instanceof ServerLevel level)) {
			return false;
		}
		level.broadcastEntityEvent(entity, (byte) 60);
		entity.discard();
		return true;
	}

	public void tick(MinecraftServer server) {
		tick++;
		for (Entity entity : due.due(tick)) {
			active.remove(entity);
			// Removed already (killed, or unloaded with its chunk and seen again as another
			// entity, which the sweep takes): what it carried was dropped or saved with it, and
			// dropping it again here made two of everything.
			if (!entity.isRemoved()) {
				gone(entity);
			}
		}
		if (tick % SWEEP_EVERY == 0) {
			active.removeIf(Entity::isRemoved);
			for (ServerLevel level : server.getAllLevels()) {
				// Gathered first, then taken: the level's entity list is live, and an entity
				// discarded or an item dropped while walking it could skip or repeat entries.
				java.util.List<Entity> left = new java.util.ArrayList<>();
				for (Entity entity : level.getAllEntities()) {
					if (entity.entityTags().contains(TAG) && !active.contains(entity)) {
						left.add(entity);
					}
				}
				left.forEach(Temporaries::gone);
			}
		}
	}

	/** Takes away everything placed for a while in a level: it is about to be saved and closed. */
	public void goneIn(ServerLevel level) {
		for (Entity entity : due.all(e -> e.level() == level)) {
			active.remove(entity);
			if (!entity.isRemoved()) {
				gone(entity);
			}
		}
	}

	/**
	 * Gone, but what it carries stays: what it wears (a stack of the player's it was given, a
	 * saddle, armour) and what it holds in an inventory (a donkey's chest), dropped where it
	 * stood as items that do not despawn (until now they went with it).
	 */
	private static void gone(Entity entity) {
		if (entity.level() instanceof ServerLevel level) {
			java.util.List<ItemStack> carried = new java.util.ArrayList<>();
			if (entity instanceof LivingEntity living) {
				for (EquipmentSlot slot : EquipmentSlot.values()) {
					ItemStack worn = living.getItemBySlot(slot);
					if (!worn.isEmpty()) {
						carried.add(worn.copy());
						living.setItemSlot(slot, ItemStack.EMPTY);
					}
				}
			}
			Container inventory = entity instanceof net.minecraft.world.entity.npc.InventoryCarrier carrier ? carrier.getInventory() : horseInventory(entity);
			if (inventory != null) {
				for (int i = 0; i < inventory.getContainerSize(); i++) {
					ItemStack held = inventory.removeItemNoUpdate(i);
					if (!held.isEmpty()) {
						carried.add(held);
					}
				}
			}
			for (ItemStack stack : carried) {
				ItemEntity item = new ItemEntity(level, entity.getX(), entity.getY() + 0.5, entity.getZ(), stack);
				item.setUnlimitedLifetime();
				level.addFreshEntity(item);
			}
		}
		entity.discard();
	}

	/** A horse's, donkey's or llama's inventory, its saddle and chest, or null. */
	private static Container horseInventory(Entity entity) {
		if (!(entity instanceof net.minecraft.world.entity.animal.equine.AbstractHorse)) {
			return null;
		}
		try {
			java.lang.reflect.Field field = net.minecraft.world.entity.animal.equine.AbstractHorse.class.getDeclaredField("inventory");
			field.setAccessible(true);
			return (Container) field.get(entity);
		} catch (ReflectiveOperationException | RuntimeException e) {
			return null;
		}
	}
}
