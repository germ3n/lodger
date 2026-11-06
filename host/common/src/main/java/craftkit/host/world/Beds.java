package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.block.BedBlock;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BedPart;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.storage.LevelData;

/**
 * Beds as two blocks (ABI 26): moved one block to the side ({@code shift_bed}), doubled beside
 * ({@code bed_beside}), recoloured ({@code swap_block} over a bed). A bed's halves go together:
 * one half set alone breaks the other, so the first is set without telling its neighbours. A
 * player whose respawn point was the moved bed respawns at it where it went.
 */
public final class Beds {
	/** Without neighbour updates or drops: half a bed set alone must not break the other. */
	private static final int QUIET = Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE;

	private Beds() {
	}

	/** A bed's two halves, foot first, or null when the block is no bed. */
	public static BlockPos[] halves(ServerLevel level, BlockPos pos) {
		BlockState state = level.getBlockState(pos);
		if (!(state.getBlock() instanceof BedBlock)) {
			return null;
		}
		Direction facing = state.getValue(BlockStateProperties.HORIZONTAL_FACING);
		BlockPos foot = state.getValue(BlockStateProperties.BED_PART) == BedPart.FOOT ? pos : pos.relative(facing.getOpposite());

		return new BlockPos[] {foot.immutable(), foot.relative(facing).immutable()};
	}

	/** The side of a bed with room for both halves and solid floor under them, or null. */
	private static Direction roomBeside(ServerLevel level, BlockPos[] bed) {
		Direction facing = level.getBlockState(bed[0]).getValue(BlockStateProperties.HORIZONTAL_FACING);
		for (Direction side : new Direction[] {facing.getClockWise(), facing.getCounterClockWise()}) {
			boolean room = true;
			for (BlockPos half : bed) {
				BlockPos at = half.relative(side);
				room &= level.getBlockState(at).isAir() && level.getBlockState(at.below()).isFaceSturdy(level, at.below(), Direction.UP);
			}
			if (room) {
				return side;
			}
		}

		return null;
	}

	/** Moves a bed one block to its side; why not, or the empty string. */
	public static String shift(ServerLevel level, BlockPos pos) {
		BlockPos[] bed = halves(level, pos);
		if (bed == null) {
			return "no bed there";
		}
		BlockState foot = level.getBlockState(bed[0]);
		BlockState head = level.getBlockState(bed[1]);
		if (foot.getValue(BlockStateProperties.OCCUPIED) || head.getValue(BlockStateProperties.OCCUPIED)) {
			return "someone lies in it";
		}
		Direction side = roomBeside(level, bed);
		if (side == null) {
			return "no room beside it";
		}
		level.setBlock(bed[1], Blocks.AIR.defaultBlockState(), QUIET);
		level.setBlock(bed[0], Blocks.AIR.defaultBlockState(), Block.UPDATE_ALL);
		level.setBlock(bed[0].relative(side), foot, QUIET);
		level.setBlock(bed[1].relative(side), head, Block.UPDATE_ALL);
		// Everyone online, wherever they are: one in the Nether whose bed this is woke at the
		// world's spawn when only this level's players were asked.
		for (ServerPlayer player : level.getServer().getPlayerList().getPlayers()) {
			ServerPlayer.RespawnConfig respawn = player.getRespawnConfig();
			if (respawn == null || !respawn.respawnData().dimension().equals(level.dimension())) {
				continue;
			}
			BlockPos at = respawn.respawnData().pos();
			if (at.equals(bed[0]) || at.equals(bed[1])) {
				LevelData.RespawnData moved = LevelData.RespawnData.of(level.dimension(), at.relative(side), respawn.respawnData().yaw(), respawn.respawnData().pitch());
				player.setRespawnPosition(new ServerPlayer.RespawnConfig(moved, respawn.forced()), false);
			}
		}

		return "";
	}

	/** A bed of another block beside a bed, the same facing; why not, or the empty string. */
	public static String beside(ServerLevel level, BlockPos pos, Block block) {
		BlockPos[] bed = halves(level, pos);
		if (bed == null) {
			return "no bed there";
		}
		if (!(block instanceof BedBlock)) {
			return "that is no bed";
		}
		Direction side = roomBeside(level, bed);
		if (side == null) {
			return "no room beside it";
		}
		Direction facing = level.getBlockState(bed[0]).getValue(BlockStateProperties.HORIZONTAL_FACING);
		BlockState foot = block.defaultBlockState().setValue(BlockStateProperties.HORIZONTAL_FACING, facing).setValue(BlockStateProperties.BED_PART, BedPart.FOOT);
		level.setBlock(bed[0].relative(side), foot, QUIET);
		level.setBlock(bed[1].relative(side), foot.setValue(BlockStateProperties.BED_PART, BedPart.HEAD), Block.UPDATE_ALL);

		return "";
	}

	/** Both halves of a bed as another bed block, every property kept; why not, or the empty string. */
	public static String recolour(ServerLevel level, BlockPos pos, Block block) {
		BlockPos[] bed = halves(level, pos);
		if (bed == null || !(block instanceof BedBlock)) {
			return "no bed there, or that is no bed";
		}
		BlockState foot = copied(level.getBlockState(bed[0]), block.defaultBlockState());
		BlockState head = copied(level.getBlockState(bed[1]), block.defaultBlockState());
		level.setBlock(bed[0], foot, QUIET);
		level.setBlock(bed[1], head, Block.UPDATE_ALL);

		return "";
	}

	private static BlockState copied(BlockState from, BlockState to) {
		BlockState out = to;
		for (net.minecraft.world.level.block.state.properties.Property<?> property : from.getProperties()) {
			out = copy(from, out, property);
		}

		return out;
	}

	private static <T extends Comparable<T>> BlockState copy(BlockState from, BlockState to, net.minecraft.world.level.block.state.properties.Property<T> property) {
		return to.hasProperty(property) ? to.setValue(property, from.getValue(property)) : to;
	}
}
