package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.item.ItemStack;
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
 * wake in their own bed, their door open. Its floor runs on outward as barrier, so a door into the
 * dark leads nowhere and drops no one into the void; one left there by a quit is brought back
 * when next seen. Not a place to explore: a set piece, twice in a stay at most (owner).
 */
public final class OtherHouse {
	public static final ResourceKey<Level> KEY = ResourceKey.create(Registries.DIMENSION, Identifier.fromNamespaceAndPath("lodger", "other"));
	/** How far out from the house the barrier floor runs. */
	private static final int FLOOR_OUT = 24;
	/** The copy's height: a few blocks below the floor, a two-storey house above it. */
	private static final int BELOW = 3;
	private static final int ABOVE = 10;
	/** The figure in the copy, apart from the overworld's numbers. */
	private static final int FIGURE = 9_001;
	/** The longest anyone stays: three minutes. */
	private static final float MOST_SECONDS = 180;

	private record Visit(UUID player, BlockPos bed, BlockPos door, long untilTick) {
	}

	private final List<Visit> visits = new ArrayList<>();
	private long tick;

	/**
	 * Takes {@code player} into a copy of the house around {@code center} (half its width {@code
	 * radius}) for {@code seconds}, the figure at the copy's {@code door}, walking to a block from
	 * them when {@code comes}; back in their {@code bed} after. Why not, or the empty string.
	 */
	public String visit(ServerLevel home, ServerPlayer player, BlockPos center, int radius, BlockPos bed, BlockPos door, float seconds, boolean comes, Figures figures) {
		ServerLevel other = home.getServer().getLevel(KEY);
		if (other == null) {
			return "no lodger:other dimension (the content jar's data pack is off)";
		}
		if (player.level() != home) {
			return "they are not in the house's world";
		}
		if (visits.stream().anyMatch(v -> v.player.equals(player.getUUID()))) {
			return "they are there already";
		}
		radius = Math.max(4, Math.min(radius, 16));
		int floor = bed.getY() - 1;
		for (int x = center.getX() - radius; x <= center.getX() + radius; x++) {
			for (int z = center.getZ() - radius; z <= center.getZ() + radius; z++) {
				for (int y = floor - BELOW; y <= floor + ABOVE; y++) {
					BlockPos pos = new BlockPos(x, y, z);
					if (!home.isLoaded(pos)) {
						return "the house is not loaded";
					}
					BlockState state = home.getBlockState(pos);
					// In the dark: its lights left out.
					if (state.getLightEmission() > 0) {
						state = Blocks.AIR.defaultBlockState();
					}
					other.setBlock(pos, state, Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE);
				}
			}
		}
		// The floor on into the dark, unseen, so no one falls.
		for (int x = center.getX() - radius - FLOOR_OUT; x <= center.getX() + radius + FLOOR_OUT; x++) {
			for (int z = center.getZ() - radius - FLOOR_OUT; z <= center.getZ() + radius + FLOOR_OUT; z++) {
				BlockPos pos = new BlockPos(x, floor, z);
				if (other.getBlockState(pos).isAir()) {
					other.setBlock(pos, Blocks.BARRIER.defaultBlockState(), Block.UPDATE_CLIENTS);
				}
			}
		}

		if (player.isSleeping()) {
			player.stopSleepInBed(true, true);
		}
		player.teleportTo(other, bed.getX() + 0.5, bed.getY() + 0.6, bed.getZ() + 0.5, java.util.Set.of(), player.getYRot(), player.getXRot(), false);
		Vec3 inDoor = Vec3.atBottomCenterOf(door);
		List<Vec3> path = comes ? List.of(Vec3.atBottomCenterOf(bed).add(0, 0, 1)) : List.of();
		figures.show(other, player, FIGURE, player, inDoor, path, ItemStack.EMPTY, false, Math.min(seconds, MOST_SECONDS), 0f);
		visits.add(new Visit(player.getUUID(), bed, door, tick + (long) (Math.min(seconds, MOST_SECONDS) * 20)));

		return "";
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
			if (tick >= visit.untilTick || player.level().dimension() != KEY) {
				it.remove();
				home(server, player, visit.bed, visit.door);
			}
		}
		if (tick % 100 == 0) {
			ServerLevel other = server.getLevel(KEY);
			if (other != null) {
				for (ServerPlayer player : List.copyOf(other.players())) {
					if (visits.stream().noneMatch(v -> v.player.equals(player.getUUID()))) {
						home(server, player, null, null);
					}
				}
			}
		}
	}

	/** Back in their own bed with the door open; without a bed known, at their respawn or the world's. */
	private static void home(MinecraftServer server, ServerPlayer player, BlockPos bed, BlockPos door) {
		ServerLevel overworld = server.overworld();
		BlockPos to = bed;
		if (to == null) {
			ServerPlayer.RespawnConfig respawn = player.getRespawnConfig();
			to = respawn != null && respawn.respawnData().dimension() == Level.OVERWORLD ? respawn.respawnData().pos() : overworld.getRespawnData().pos();
		}
		player.teleportTo(overworld, to.getX() + 0.5, to.getY() + 0.6, to.getZ() + 0.5, java.util.Set.of(), player.getYRot(), player.getXRot(), false);
		if (door != null) {
			BlockState state = overworld.getBlockState(door);
			if (state.getBlock() instanceof DoorBlock doorBlock && !doorBlock.isOpen(state)) {
				doorBlock.setOpen(null, overworld, state, door, true);
			}
		}
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
