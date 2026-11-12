package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.state.BlockState;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * Redstone gone dead in a box for a while (`dead_redstone`, ABI 54): the pressure plates,
 * buttons and levers in it still press, click and flip, but give no power ({@code
 * craftkit.host.mixin}'s signal hooks ask {@link #dead}). As it begins and ends, what they power is
 * told again, so a lamp a lever keeps lit goes dark at once and comes back after.
 */
public final class DeadRedstone {
	/** The longest redstone is held dead, seconds. */
	public static final float MOST_SECONDS = 300;

	private record Box(ServerLevel level, BlockPos min, BlockPos max, long until) {
		boolean holds(BlockPos pos) {
			return pos.getX() >= min.getX() && pos.getX() <= max.getX() && pos.getY() >= min.getY() && pos.getY() <= max.getY()
					&& pos.getZ() >= min.getZ() && pos.getZ() <= max.getZ();
		}
	}

	private static final List<Box> BOXES = new ArrayList<>();
	private static long tick;

	private DeadRedstone() {
	}

	/** Whether the source at {@code pos} gives no power now. */
	public static synchronized boolean dead(BlockPos pos) {
		if (BOXES.isEmpty()) {
			return false;
		}
		for (Box box : BOXES) {
			if (box.holds(pos)) {
				return true;
			}
		}
		return false;
	}

	/** The redstone in a box dead for {@code seconds}. */
	public static synchronized void hold(ServerLevel level, BlockPos min, BlockPos max, float seconds) {
		Box box = new Box(level, min.immutable(), max.immutable(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20));
		BOXES.add(box);
		retell(box);
	}

	public static void tick(MinecraftServer server) {
		List<Box> ended = new ArrayList<>();
		synchronized (DeadRedstone.class) {
			tick++;
			Iterator<Box> it = BOXES.iterator();
			while (it.hasNext()) {
				Box box = it.next();
				if (box.until() < tick) {
					it.remove();
					ended.add(box);
				}
			}
		}
		ended.forEach(DeadRedstone::retell);
	}

	/** Everything forgotten: the world is closing. */
	public static synchronized void clear() {
		BOXES.clear();
	}

	/** What the box's sources power, told again: its plates', buttons' and levers' neighbours. */
	private static void retell(Box box) {
		for (BlockPos pos : BlockPos.betweenClosed(box.min(), box.max())) {
			BlockState state = box.level().getBlockState(pos);
			if (state.getBlock() instanceof net.minecraft.world.level.block.BasePressurePlateBlock
					|| state.getBlock() instanceof net.minecraft.world.level.block.ButtonBlock
					|| state.getBlock() instanceof net.minecraft.world.level.block.LeverBlock) {
				BlockPos at = pos.immutable();
				box.level().updateNeighborsAt(at, state.getBlock());
				for (Direction d : Direction.values()) {
					box.level().updateNeighborsAt(at.relative(d), state.getBlock());
				}
			}
		}
	}
}
