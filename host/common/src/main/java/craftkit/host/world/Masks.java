package craftkit.host.world;

import net.minecraft.core.component.DataComponents;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.component.ResolvableProfile;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * Mobs wearing a player's own head (`wear_face`, ABI 54): on its head while it lasts, never
 * dropped (its drop chance nothing), and taken off again, what it wore before put back, when the
 * time is up; gone with it if it dies.
 */
public final class Masks {
	/** The longest a mob wears it. */
	public static final float MOST_SECONDS = 600;

	private record Mask(Mob mob, ItemStack before, float chanceBefore, long untilTick) {
	}

	private final List<Mask> worn = new ArrayList<>();
	private long tick;

	/** {@code mob} wears {@code player}'s head for {@code seconds}. */
	public void put(Mob mob, ServerPlayer player, float seconds) {
		worn.removeIf(m -> m.mob() == mob);
		ItemStack head = new ItemStack(Items.PLAYER_HEAD);
		head.set(DataComponents.PROFILE, ResolvableProfile.createResolved(player.getGameProfile()));
		worn.add(new Mask(mob, mob.getItemBySlot(EquipmentSlot.HEAD).copy(), mob.getDropChances().byEquipment(EquipmentSlot.HEAD),
				tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));
		mob.setItemSlot(EquipmentSlot.HEAD, head);
		mob.setDropChance(EquipmentSlot.HEAD, 0f);
	}

	public void tick() {
		tick++;
		Iterator<Mask> it = worn.iterator();
		while (it.hasNext()) {
			Mask mask = it.next();
			if (mask.mob().isRemoved() || !mask.mob().isAlive()) {
				it.remove();
				continue;
			}
			if (tick >= mask.untilTick()) {
				takeOff(mask);
				it.remove();
			}
		}
	}

	private static void takeOff(Mask mask) {
		mask.mob().setItemSlot(EquipmentSlot.HEAD, mask.before());
		mask.mob().setDropChance(EquipmentSlot.HEAD, mask.chanceBefore());
	}

	/** The world closing: every head taken off, so none is saved on a mob. */
	public void clear() {
		worn.forEach(Masks::takeOff);
		worn.clear();
	}
}
