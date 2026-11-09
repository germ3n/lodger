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
