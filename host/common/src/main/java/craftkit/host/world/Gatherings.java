package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.npc.villager.AbstractVillager;
import net.minecraft.world.entity.npc.villager.Villager;
import net.minecraft.world.item.trading.MerchantOffer;
import net.minecraft.world.level.block.DoorBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * The village emptied into one house till dawn ({@code gather_villagers}, ABI 31; owner,
 * 2026-11-09): each villager walked in and held still facing the back wall, turning to the
 * player who comes near, and put back where it stood at its time or at once when the level
 * closes. Beside it, the trades of what the lodger took ({@code offer_trade}): one sale each,
 * then the trade is taken away, so a restock never sells it twice.
 */
public final class Gatherings {
	/** The longest a village stays empty, whatever a mod asks. */
	public static final float MOST_SECONDS = 600;
	/** Within this many blocks, the packed villagers turn to the player. */
	public static final double TURN_WITHIN = 5;

	private record Gathered(AbstractVillager villager, ServerLevel level, Vec3 from, long untilTick) {
	}

	/** On a gathered villager: where it stood, `x,y,z`. */
	public static final String FROM = "craftkit_gathered_from:";
	/** On a villager selling what the lodger took: one sale. */
	public static final String SELLING = "craftkit_selling";

	private record Sale(AbstractVillager villager, MerchantOffer offer) {
	}

	private final List<Gathered> gathered = new ArrayList<>();
	private final List<Sale> sales = new ArrayList<>();
	private long tick;

	/** Packs the villagers near a village's middle into the house behind a door; how many went in. */
	public int gather(ServerLevel level, BlockPos center, BlockPos door, float seconds, ServerPlayer player, Holds holds) {
		BlockState state = level.getBlockState(door);
		if (!(state.getBlock() instanceof DoorBlock doorBlock)) {
			return 0;
		}
		Direction facing = state.getValue(BlockStateProperties.HORIZONTAL_FACING);
		// Inside is the side under a roof.
		Direction in = !level.canSeeSky(door.relative(facing).above()) ? facing : facing.getOpposite();
		List<BlockPos> spots = new ArrayList<>();
		for (int depth = 1; depth <= 4; depth++) {
			for (int across = -1; across <= 1; across++) {
				BlockPos at = door.relative(in, depth).relative(in.getClockWise(), across);
				if (level.getBlockState(at).getCollisionShape(level, at).isEmpty()
						&& level.getBlockState(at.above()).getCollisionShape(level, at.above()).isEmpty()
						&& level.getBlockState(at.below()).isFaceSturdy(level, at.below(), Direction.UP)) {
					spots.add(at);
				}
			}
		}
		if (spots.isEmpty()) {
			return 0;
		}
		List<Villager> villagers = level.getEntitiesOfClass(Villager.class, new AABB(center).inflate(48));
		long until = tick + (long) (Math.min(seconds, MOST_SECONDS) * 20);
		Vec3 wall = Vec3.atCenterOf(door.relative(in, 8));
		for (int i = 0; i < villagers.size(); i++) {
			Villager villager = villagers.get(i);
			BlockPos spot = spots.get(i % spots.size());
			// Crowded, never into the walls: a quarter block more for each round, at most a third.
			double crowd = Math.min(0.33, (i / spots.size()) * 0.25);
			gathered.add(new Gathered(villager, level, villager.position(), until));
			// Where it stood, kept on it: one saved while gathered goes back when next seen.
			villager.addTag(FROM + villager.getX() + "," + villager.getY() + "," + villager.getZ());
			villager.teleportTo(spot.getX() + 0.5 + crowd, spot.getY(), spot.getZ() + 0.5 - crowd);
			holds.add(villager, player, Math.min(seconds, MOST_SECONDS), wall, TURN_WITHIN);
		}
		doorBlock.setOpen(null, level, level.getBlockState(door), door, false);

		return villagers.size();
	}

	/** A trade of one stack for an emerald, watched so it sells once. */
	public void sell(AbstractVillager villager, net.minecraft.world.item.ItemStack stack) {
		MerchantOffer offer = new MerchantOffer(new net.minecraft.world.item.trading.ItemCost(net.minecraft.world.item.Items.EMERALD, 1), stack, 1, 0, 0.05f);
		villager.getOffers().add(offer);
		villager.addTag(SELLING);
		sales.add(new Sale(villager, offer));
	}

	/** The lodger's trade on a villager: one emerald for a thing, one use. */
	private static boolean ours(MerchantOffer offer) {
		return offer.getMaxUses() == 1 && offer.getBaseCostA().is(net.minecraft.world.item.Items.EMERALD)
				&& offer.getBaseCostA().getCount() == 1 && offer.getCostB().isEmpty();
	}

	public void tick(MinecraftServer server) {
		tick++;
		if (tick % 100 == 0) {
			sweep(server);
		}
		Iterator<Gathered> it = gathered.iterator();
		while (it.hasNext()) {
			Gathered g = it.next();
			if (g.villager().isRemoved() || g.untilTick() < tick) {
				putBack(g);
				it.remove();
			}
		}
		sales.removeIf(sale -> {
			AbstractVillager villager = sale.villager();
			if (villager.isRemoved()) {
				// Killed before it sold: what it was selling drops where it fell, never lost.
				if (villager.isDeadOrDying() && !sale.offer().isOutOfStock() && villager.level() instanceof ServerLevel level) {
					var item = new net.minecraft.world.entity.item.ItemEntity(level, villager.getX(), villager.getY() + 0.5, villager.getZ(), sale.offer().getResult().copy());
					item.setUnlimitedLifetime();
					level.addFreshEntity(item);
				}
				return true;
			}
			if (sale.offer().isOutOfStock()) {
				villager.getOffers().remove(sale.offer());
				villager.removeTag(SELLING);
				return true;
			}
			return false;
		});
	}

	/** Puts every villager of a level back: the level is about to be saved and closed. */
	public void putBackIn(ServerLevel level) {
		gathered.removeIf(g -> {
			if (g.level() == level) {
				putBack(g);
				return true;
			}
			return false;
		});
	}

	private static void putBack(Gathered g) {
		if (!g.villager().isRemoved()) {
			g.villager().teleportTo(g.from().x, g.from().y, g.from().z);
			clearFrom(g.villager());
		}
	}

	private static void clearFrom(net.minecraft.world.entity.Entity villager) {
		villager.entityTags().stream().filter(t -> t.startsWith(FROM)).toList().forEach(villager::removeTag);
	}

	/**
	 * Villagers saved while gathered (their chunk unloaded as the player went away): back where
	 * they stood when next seen (until 2026-11-10 they stayed in the house).
	 */
	private void sweep(MinecraftServer server) {
		for (ServerLevel level : server.getAllLevels()) {
			for (net.minecraft.world.entity.Entity entity : level.getAllEntities()) {
				if (!(entity instanceof Villager villager)) {
					continue;
				}
				// A sale saved with it (its chunk unloaded, the game quit): watched again, so it
				// sells once; sold, taken away, so a restock never sells it twice (until
				// 2026-11-10 a restock sold the very thing again and again).
				if (villager.entityTags().contains(SELLING) && sales.stream().noneMatch(s -> s.villager() == villager)) {
					for (MerchantOffer offer : List.copyOf(villager.getOffers())) {
						if (ours(offer)) {
							if (offer.getUses() > 0 || offer.isOutOfStock()) {
								villager.getOffers().remove(offer);
							} else {
								sales.add(new Sale(villager, offer));
							}
						}
					}
					if (sales.stream().noneMatch(s -> s.villager() == villager)) {
						villager.removeTag(SELLING);
					}
				}
				if (gathered.stream().anyMatch(g -> g.villager() == villager)) {
					continue;
				}
				for (String tag : villager.entityTags().stream().filter(t -> t.startsWith(FROM)).toList()) {
					String[] at = tag.substring(FROM.length()).split(",");
					try {
						villager.teleportTo(Double.parseDouble(at[0]), Double.parseDouble(at[1]), Double.parseDouble(at[2]));
					} catch (RuntimeException ignored) {
						// Not one of ours: left where it stands.
					}
					villager.removeTag(tag);
				}
			}
		}
	}
}
