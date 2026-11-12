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

	/** A villager running in at a door (`villager_flees_in`, ABI 54): it, the door, where it falls, and its time. */
	private record FleeingIn(net.minecraft.world.entity.npc.villager.Villager villager, ServerLevel level, net.minecraft.core.BlockPos door,
			net.minecraft.world.phys.Vec3 inside, long fromTick) {
	}

	private final java.util.List<FleeingIn> fleeing = new java.util.ArrayList<>();
	/** When it bangs, when the door opens, and how long it may take to get in. */
	static final int[] BANGS = {0, 12, 24};
	static final int OPENS = 40;
	static final int GETS_IN_BY = 140;

	/**
	 * A villager out of nowhere at {@code outside}, banging on the door at {@code door} as if
	 * something were after it; the door opens, it runs in, and at {@code inside} it drops dead.
	 * No one's to blame (no killer, no gossip), and it drops nothing (ABI 54).
	 */
	public boolean fleeIn(ServerLevel level, net.minecraft.core.BlockPos door, net.minecraft.world.phys.Vec3 outside, net.minecraft.world.phys.Vec3 inside) {
		if (!(net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(net.minecraft.resources.Identifier.parse("minecraft:villager")).create(level, net.minecraft.world.entity.EntitySpawnReason.MOB_SUMMONED)
				instanceof net.minecraft.world.entity.npc.villager.Villager villager)) {
			return false;
		}
		net.minecraft.world.phys.Vec3 to = net.minecraft.world.phys.Vec3.atCenterOf(door).subtract(outside);
		float yaw = (float) Math.toDegrees(Math.atan2(-to.x, to.z));
		villager.snapTo(outside.x, outside.y, outside.z, yaw, 0);
		villager.setYHeadRot(yaw);
		villager.skipDropExperience();
		level.addFreshEntity(villager);
		fleeing.add(new FleeingIn(villager, level, door.immutable(), inside, tick));
		return true;
	}

	private void tickFleeing() {
		java.util.Iterator<FleeingIn> it = fleeing.iterator();
		while (it.hasNext()) {
			FleeingIn f = it.next();
			var villager = f.villager();
			if (villager.isRemoved()) {
				it.remove();
				continue;
			}
			long t = tick - f.fromTick();
			for (int bang : BANGS) {
				if (t == bang) {
					f.level().playSound(null, f.door(), net.minecraft.sounds.SoundEvents.ZOMBIE_ATTACK_WOODEN_DOOR, net.minecraft.sounds.SoundSource.BLOCKS, 0.8f, 1.3f);
					f.level().playSound(null, villager.blockPosition(), net.minecraft.sounds.SoundEvents.VILLAGER_HURT, net.minecraft.sounds.SoundSource.NEUTRAL, 1f, 1.1f);
				}
			}
			if (t == OPENS) {
				var state = f.level().getBlockState(f.door());
				if (state.getBlock() instanceof net.minecraft.world.level.block.DoorBlock door) {
					door.setOpen(null, f.level(), state, f.door(), true);
				}
			}
			if (t >= OPENS && t % 10 == 0) {
				villager.getNavigation().moveTo(f.inside().x, f.inside().y, f.inside().z, 1.4);
			}
			boolean there = villager.position().distanceTo(f.inside()) < 1.3;
			if (t >= OPENS && (there || t >= GETS_IN_BY)) {
				villager.kill(f.level());
				it.remove();
			}
		}
	}

	/** A pack circling a player ({@link Ring}): who, its mobs, and its time. */
	private record Circling(net.minecraft.server.level.ServerPlayer player, java.util.List<net.minecraft.world.entity.Mob> pack, long fromTick, long ticks) {
	}

	private final java.util.List<Circling> circlings = new java.util.ArrayList<>();

	/**
	 * {@code count} of {@code type} placed round {@code player} for {@code seconds}, circling them
	 * nearer each round, a growl now and then, then breaking off and running into the dark
	 * (`circle`, ABI 54). They are placed for a while as any are, and hurt no one. False when none
	 * could be placed.
	 */
	public boolean circle(ServerLevel level, net.minecraft.server.level.ServerPlayer player, net.minecraft.world.entity.EntityType<?> type, int count, float seconds) {
		java.util.List<net.minecraft.world.entity.Mob> pack = new java.util.ArrayList<>();
		for (int i = 0; i < Math.min(count, 8); i++) {
			double a = Ring.angle(i, count, 0);
			double x = player.getX() + Math.cos(a) * Ring.FROM;
			double z = player.getZ() + Math.sin(a) * Ring.FROM;
			net.minecraft.core.BlockPos column = net.minecraft.core.BlockPos.containing(x, 0, z);
			int y = level.getHeight(net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, column.getX(), column.getZ());
			if (!(type.create(level, net.minecraft.world.entity.EntitySpawnReason.MOB_SUMMONED) instanceof net.minecraft.world.entity.Mob mob)) {
				continue;
			}
			mob.snapTo(x, y, z, 0, 0);
			level.addFreshEntity(mob);
			add(mob, seconds + 2);
			pack.add(mob);
		}
		if (pack.isEmpty()) {
			return false;
		}
		circlings.add(new Circling(player, pack, tick, (long) (Math.min(seconds, 60) * 20)));
		return true;
	}

	/** Each pack a tick on: round its player, nearer; then off into the dark. */
	private void tickCircling() {
		java.util.Iterator<Circling> it = circlings.iterator();
		while (it.hasNext()) {
			Circling c = it.next();
			long since = tick - c.fromTick();
			if (since > c.ticks() || c.player().isRemoved()) {
				it.remove();
				continue;
			}
			if (since % 10 != 0) {
				continue;
			}
			double t = since / (double) c.ticks();
			for (int i = 0; i < c.pack().size(); i++) {
				net.minecraft.world.entity.Mob mob = c.pack().get(i);
				if (mob.isRemoved()) {
					continue;
				}
				double x;
				double z;
				if (Ring.running(t)) {
					double ax = mob.getX() - c.player().getX();
					double az = mob.getZ() - c.player().getZ();
					double length = Math.max(0.1, Math.sqrt(ax * ax + az * az));
					x = c.player().getX() + ax / length * 40;
					z = c.player().getZ() + az / length * 40;
					mob.getNavigation().moveTo(x, mob.getY(), z, 1.8);
				} else {
					double a = Ring.angle(i, c.pack().size(), since / 20.0);
					double r = Ring.radius(t);
					x = c.player().getX() + Math.cos(a) * r;
					z = c.player().getZ() + Math.sin(a) * r;
					mob.getNavigation().moveTo(x, c.player().getY(), z, 1.3);
					mob.getLookControl().setLookAt(c.player(), 30, 30);
				}
			}
			// A growl from one of them now and then, a low one.
			if (!Ring.running(t) && since % 40 == 0) {
				net.minecraft.world.entity.Mob growler = c.pack().get((int) (since / 40 % c.pack().size()));
				net.minecraft.core.registries.BuiltInRegistries.SOUND_EVENT.get(net.minecraft.resources.Identifier.parse("minecraft:entity.wolf.growl"))
						.ifPresent(growl -> growler.playSound(growl.value(), 1f, 0.8f));
			}
		}
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
		tickCircling();
		DeadRedstone.tick(server);
		tickFleeing();
		for (Entity entity : due.due(tick)) {
			active.remove(entity);
			// Removed already (killed, or unloaded with its chunk and seen again as another
			// entity, which the sweep takes): what it carried was dropped or saved with it, and
			// dropping it again here made two of everything.
			if (!entity.isRemoved()) {
				gone(entity);
			}
		}
		if (Sweeps.due(Sweeps.Sweep.TEMPORARIES, tick)) {
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
