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
			double crowd = (i / spots.size()) * 0.25;
			gathered.add(new Gathered(villager, level, villager.position(), until));
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
		sales.add(new Sale(villager, offer));
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Gathered> it = gathered.iterator();
		while (it.hasNext()) {
			Gathered g = it.next();
			if (g.villager().isRemoved() || g.untilTick() < tick) {
				putBack(g);
				it.remove();
			}
		}
		sales.removeIf(sale -> {
			if (sale.villager().isRemoved()) {
				return true;
			}
			if (sale.offer().isOutOfStock()) {
				sale.villager().getOffers().remove(sale.offer());
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
		}
	}
}
