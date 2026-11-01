package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

import java.util.List;

/**
 * Blocks taken away for a while and put back as they were ({@code hide_block}, ABI 24): a
 * player's torches out behind them for a minute. Each is put back at its time, or at once when
 * its level is about to be saved and closed, so a quit never leaves a light gone; one is not
 * put back over what filled the empty place meanwhile, nor where it can no longer stand.
 */
public final class Hidden {
	/** The longest a block stays hidden, whatever a mod asks: ten minutes. */
	public static final float MOST_SECONDS = 600;

	private record Gone(ServerLevel level, BlockPos pos, BlockState state) {
	}

	private final Due<Gone> due = new Due<>();
	private long tick;

	/** Takes the block at a place away until {@code seconds} from now. */
	public void hide(ServerLevel level, BlockPos pos, float seconds) {
		BlockState state = level.getBlockState(pos);
		level.setBlock(pos, net.minecraft.world.level.block.Blocks.AIR.defaultBlockState(), Block.UPDATE_ALL);
		due.add(new Gone(level, pos.immutable(), state), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20));
	}

	public void tick() {
		tick++;
		putBack(due.due(tick));
	}

	/** Puts back everything hidden in a level: it is about to be saved and closed. */
	public void putBackIn(ServerLevel level) {
		putBack(due.all(gone -> gone.level() == level));
	}

	private static void putBack(List<Gone> gone) {
		for (Gone g : gone) {
			if (g.level().getBlockState(g.pos()).isAir() && g.state().canSurvive(g.level(), g.pos())) {
				g.level().setBlock(g.pos(), g.state(), Block.UPDATE_ALL);
			}
		}
	}

	public int count() {
		return due.size();
	}
}
