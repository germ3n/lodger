package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.DoorBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * The other house (`other_house`, ABI 49): a copy of the target's house, block for block and its
 * lights left out, in the content jar's void dimension `lodger:other` (a black sky, no mobs, no
 * day, no sleep). They wake in it, in the copy's bed, the figure in its doorway; after a while they
 * wake in their own bed, their door open. Not a place to explore: a set piece, twice in a stay at
 * most (owner).
 *
 * <p>Safe by construction: the copy always stands at one place in the dimension, cleared before
 * each copy, so nothing is left of the last; an unseen barrier floor runs on outward with barrier
 * walls at its edge and a barrier layer under the copy, so no door into the dark and no hole dug
 * drops anyone into the void; and they are in adventure mode while there, so nothing of the copy
 * can be mined and carried home (a house of diamond blocks was a second house of diamonds). Their
 * game mode goes back as they leave, and one left there by a quit or a crash is brought back
 * with theirs when next seen (kept on them as a tag).
 */
public final class OtherHouse {
	public static final ResourceKey<Level> KEY = ResourceKey.create(Registries.DIMENSION, Identifier.fromNamespaceAndPath("lodger", "other"));
	/** Where the copy's floor-level centre always stands in the dimension. */
	private static final BlockPos ANCHOR = new BlockPos(0, 100, 0);
	/** The house's half width copied, at most; the floor and its walls run this much further. */
	private static final int MOST_RADIUS = 16;
	private static final int FLOOR_OUT = 24;
	private static final int WALL_HIGH = 4;
	/** The copy's height: a few blocks below the floor, a two-storey house above it. */
	private static final int BELOW = 3;
	private static final int ABOVE = 10;
	/** The figure in the copy, apart from the overworld's numbers. */
	private static final int FIGURE = 9_001;
	/** The longest anyone stays: three minutes. */
	private static final float MOST_SECONDS = 180;
	/** The game mode they had, kept on them while there: survives a quit. */
	private static final String MODE_TAG = "craftkit_other_mode_";

	private record Visit(UUID player, BlockPos bed, BlockPos door, long untilTick) {
	}

	private final List<Visit> visits = new ArrayList<>();
	private long tick;

	/** Where a block of the house stands in the copy. */
	private static BlockPos inCopy(BlockPos pos, BlockPos center, int floor) {
		return ANCHOR.offset(pos.getX() - center.getX(), pos.getY() - floor, pos.getZ() - center.getZ());
	}

	/**
	 * Takes {@code player} into a copy of the house around {@code center} (half its width {@code
	 * radius}) for {@code seconds}, the figure at the copy's {@code door}, walking at them when
	 * {@code comes}; back in their {@code bed} after. Why not, or the empty string.
	 */
	public String visit(ServerLevel home, ServerPlayer player, BlockPos center, int radius, BlockPos bed, BlockPos door, float seconds, boolean comes, Figures figures) {
		ServerLevel other = home.getServer().getLevel(KEY);
		if (other == null) {
			return "no lodger:other dimension (the content jar's data pack is off)";
		}
		if (player.level() != home) {
			return "they are not in the house's world";
		}
		if (player.isCreative() || player.isSpectator()) {
			return "never in creative or spectator";
		}
		if (visits.stream().anyMatch(v -> v.player.equals(player.getUUID()))) {
			return "they are there already";
		}
		radius = Math.max(4, Math.min(radius, MOST_RADIUS));
		int floor = bed.getY() - 1;
		// All of it loaded before a block is written: a failure leaves nothing half copied.
		for (int x = center.getX() - radius; x <= center.getX() + radius; x += 8) {
			for (int z = center.getZ() - radius; z <= center.getZ() + radius; z += 8) {
				if (!home.isLoaded(new BlockPos(x, floor, z))) {
					return "the house is not loaded";
				}
			}
		}
		if (!home.isLoaded(new BlockPos(center.getX() + radius, floor, center.getZ() + radius))) {
			return "the house is not loaded";
		}

		// The last copy gone: the whole place cleared, so nothing of an older house stays.
		int span = MOST_RADIUS + FLOOR_OUT + 1;
		for (int x = -span; x <= span; x++) {
			for (int z = -span; z <= span; z++) {
				for (int y = -BELOW - 1; y <= ABOVE + WALL_HIGH; y++) {
					BlockPos pos = ANCHOR.offset(x, y, z);
					if (!other.getBlockState(pos).isAir()) {
						other.setBlock(pos, Blocks.AIR.defaultBlockState(), Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE);
					}
				}
			}
		}
		// The house, its lights left out: in the dark.
		for (int x = center.getX() - radius; x <= center.getX() + radius; x++) {
			for (int z = center.getZ() - radius; z <= center.getZ() + radius; z++) {
				for (int y = floor - BELOW; y <= floor + ABOVE; y++) {
					BlockPos pos = new BlockPos(x, y, z);
					BlockState state = home.getBlockState(pos);
					// In the dark: its lights left out; and nothing that goes off (TNT, a respawn
					// anchor), whose blast would drop the copy's blocks to carry home.
					if (state.getLightEmission() > 0 || state.is(Blocks.TNT) || state.is(Blocks.RESPAWN_ANCHOR)) {
						state = Blocks.AIR.defaultBlockState();
					}
					if (!state.isAir()) {
						other.setBlock(inCopy(pos, center, floor), state, Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE);
					}
				}
			}
		}
		// Unseen, so no one falls: a floor on into the dark with walls at its edge, and a layer
		// under the copy for one who digs down.
		int out = radius + FLOOR_OUT;
		for (int x = -out; x <= out; x++) {
			for (int z = -out; z <= out; z++) {
				barrier(other, ANCHOR.offset(x, 0, z));
				barrier(other, ANCHOR.offset(x, -BELOW - 1, z));
				// Walls the whole height, from the layer under the copy to above its roof: a swim
				// down or a flight over found the void.
				if (Math.abs(x) == out || Math.abs(z) == out) {
					for (int y = -BELOW - 1; y <= ABOVE + WALL_HIGH; y++) {
						barrier(other, ANCHOR.offset(x, y, z));
					}
				}
			}
		}

		if (player.isSleeping()) {
			player.stopSleepInBed(true, true);
		}
		BlockPos copyBed = inCopy(bed, center, floor);
		GameType mode = player.gameMode.getGameModeForPlayer();
		if (!player.teleportTo(other, copyBed.getX() + 0.5, copyBed.getY() + 0.6, copyBed.getZ() + 0.5, java.util.Set.of(), player.getYRot(), player.getXRot(), false)) {
			return "the move was refused";
		}
		// Nothing of the copy mined and carried home.
		player.addTag(MODE_TAG + mode.getName());
		player.setGameMode(GameType.ADVENTURE);

		Vec3 inDoor = Vec3.atBottomCenterOf(inCopy(door, center, floor));
		List<Vec3> path = new ArrayList<>();
		if (comes) {
			// A block at a time from the doorway to a block from the bed.
			Vec3 to = Vec3.atBottomCenterOf(copyBed);
			double length = inDoor.distanceTo(to);
			for (double d = 1; d < length - 1; d += 1) {
				path.add(inDoor.add(to.subtract(inDoor).normalize().scale(d)));
			}
		}
		figures.show(other, player, FIGURE, player, inDoor, path, ItemStack.EMPTY, false, Math.min(seconds, MOST_SECONDS), 0f);
		visits.add(new Visit(player.getUUID(), bed, door, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));

		return "";
	}

	private static void barrier(ServerLevel level, BlockPos pos) {
		if (level.getBlockState(pos).isAir()) {
			level.setBlock(pos, Blocks.BARRIER.defaultBlockState(), Block.UPDATE_CLIENTS);
		}
	}

	/** Once a tick: a visit over sends them home; anyone else there is sent home too. */
	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Visit> it = visits.iterator();
		while (it.hasNext()) {
			Visit visit = it.next();
			ServerPlayer player = server.getPlayerList().getPlayer(visit.player);
			if (player == null) {
				// Gone with a quit: brought back by the sweep when next seen.
				it.remove();
				continue;
			}
			// Out of the box all the same (a pearl, a flight): home at once, never into the void.
			boolean out = player.level().dimension() == KEY && (player.getY() < ANCHOR.getY() - BELOW - 2
					|| Math.abs(player.getX() - ANCHOR.getX()) > MOST_RADIUS + FLOOR_OUT + 1
					|| Math.abs(player.getZ() - ANCHOR.getZ()) > MOST_RADIUS + FLOOR_OUT + 1
					|| player.getY() > ANCHOR.getY() + ABOVE + WALL_HIGH + 8);
			if (tick >= visit.untilTick || player.level().dimension() != KEY || out) {
				it.remove();
				home(server, player, visit.bed, visit.door);
			}
		}
		if (tick % 20 == 0) {
			ServerLevel other = server.getLevel(KEY);
			if (other != null) {
				for (ServerPlayer player : List.copyOf(other.players())) {
					if (visits.stream().noneMatch(v -> v.player.equals(player.getUUID()))) {
						home(server, player, null, null);
					}
				}
			}
			// One sent home elsewhere (a death, a portal) keeps their mode tag: put it back.
			for (ServerPlayer player : server.getPlayerList().getPlayers()) {
				if (player.level().dimension() != KEY && visits.stream().noneMatch(v -> v.player.equals(player.getUUID()))) {
					restoreMode(player);
				}
			}
		}
	}

	/** Their game mode as it was before the visit, from the tag kept on them. */
	private static void restoreMode(ServerPlayer player) {
		for (String tag : List.copyOf(player.entityTags())) {
			if (tag.startsWith(MODE_TAG)) {
				player.removeTag(tag);
				GameType mode = GameType.byName(tag.substring(MODE_TAG.length()), GameType.SURVIVAL);
				if (player.gameMode.getGameModeForPlayer() == GameType.ADVENTURE) {
					player.setGameMode(mode);
				}
			}
		}
	}

	/** Back in their own bed with the door open; without a bed known, at their respawn or the world's. */
	private static void home(MinecraftServer server, ServerPlayer player, BlockPos bed, BlockPos door) {
		ServerLevel overworld = server.overworld();
		List<ItemStack> left = leftBehind(server);
		BlockPos to = bed;
		if (to == null) {
			ServerPlayer.RespawnConfig respawn = player.getRespawnConfig();
			to = respawn != null && respawn.respawnData().dimension() == Level.OVERWORLD ? respawn.respawnData().pos() : overworld.getRespawnData().pos();
		}
		player.teleportTo(overworld, to.getX() + 0.5, to.getY() + 0.6, to.getZ() + 0.5, java.util.Set.of(), player.getYRot(), player.getXRot(), false);
		restoreMode(player);
		// What they left there comes home with them: in their inventory, or at their feet.
		for (ItemStack stack : left) {
			if (!player.getInventory().add(stack) && !stack.isEmpty()) {
				net.minecraft.world.entity.item.ItemEntity dropped = new net.minecraft.world.entity.item.ItemEntity(overworld, player.getX(), player.getY(), player.getZ(), stack);
				dropped.setUnlimitedLifetime();
				overworld.addFreshEntity(dropped);
			}
		}
		if (door != null) {
			BlockState state = overworld.getBlockState(door);
			if (state.getBlock() instanceof DoorBlock doorBlock && !doorBlock.isOpen(state)) {
				doorBlock.setOpen(null, overworld, state, door, true);
			}
		}
	}

	/**
	 * What is left in the copy, taken out: what they put in its chests and what they dropped.
	 * The next copy's clearing spilled it into a place torn down, and nothing came home.
	 */
	private static List<ItemStack> leftBehind(MinecraftServer server) {
		List<ItemStack> left = new ArrayList<>();
		ServerLevel other = server.getLevel(KEY);
		if (other == null) {
			return left;
		}
		int span = MOST_RADIUS + FLOOR_OUT + 1;
		net.minecraft.world.phys.AABB box = new net.minecraft.world.phys.AABB(ANCHOR.getX() - span, ANCHOR.getY() - BELOW - 2, ANCHOR.getZ() - span,
				ANCHOR.getX() + span + 1, ANCHOR.getY() + ABOVE + WALL_HIGH + 2, ANCHOR.getZ() + span + 1);
		for (net.minecraft.world.entity.item.ItemEntity item : other.getEntitiesOfClass(net.minecraft.world.entity.item.ItemEntity.class, box)) {
			left.add(item.getItem().copy());
			item.discard();
		}
		for (int x = -MOST_RADIUS; x <= MOST_RADIUS; x++) {
			for (int z = -MOST_RADIUS; z <= MOST_RADIUS; z++) {
				for (int y = -BELOW; y <= ABOVE; y++) {
					BlockPos pos = ANCHOR.offset(x, y, z);
					if (other.getBlockEntity(pos) instanceof net.minecraft.world.Container container) {
						for (int i = 0; i < container.getContainerSize(); i++) {
							ItemStack stack = container.removeItemNoUpdate(i);
							if (!stack.isEmpty()) {
								left.add(stack);
							}
						}
					}
				}
			}
		}
		return left;
	}

	/** Everyone there sent home: the server is stopping. */
	public void allHome(MinecraftServer server) {
		ServerLevel other = server.getLevel(KEY);
		if (other != null) {
			for (ServerPlayer player : List.copyOf(other.players())) {
				Visit visit = visits.stream().filter(v -> v.player.equals(player.getUUID())).findFirst().orElse(null);
				home(server, player, visit == null ? null : visit.bed, visit == null ? null : visit.door);
			}
		}
		visits.clear();
	}
}
