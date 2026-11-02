package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.network.protocol.game.ClientboundSoundPacket;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.level.block.SoundType;
import net.minecraft.world.level.block.state.BlockState;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * A second set of steps behind a player (`footsteps`, ABI 36; `footsteps_follow`, batch 16):
 * each step they take is heard again a moment later, a few blocks back along their way, in the
 * sound of the block there, sent to them alone. They stop when the player stops.
 */
public final class Footsteps {
	/** The longest a follow lasts, whatever a mod asks. */
	public static final float MOST_SECONDS = 120;

	private static final class Follow {
		final UUID player;
		final long untilTick;
		final double behind;
		final int delay;
		final Stride stride = new Stride();
		double lastX;
		double lastZ;
		boolean started;

		Follow(UUID player, long untilTick, double behind, int delay) {
			this.player = player;
			this.untilTick = untilTick;
			this.behind = behind;
			this.delay = delay;
		}
	}

	private record Step(UUID player, long atTick, double x, double y, double z) {
	}

	private final List<Follow> active = new ArrayList<>();
	private final List<Step> due = new ArrayList<>();
	private long tick;

	public void add(ServerPlayer player, float seconds, float behind, int delay) {
		active.removeIf(f -> f.player.equals(player.getUUID()));
		active.add(new Follow(player.getUUID(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20), Math.max(1, Math.min(behind, 8)), Math.max(0, Math.min(delay, 40))));
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Follow> it = active.iterator();
		while (it.hasNext()) {
			Follow follow = it.next();
			ServerPlayer player = server.getPlayerList().getPlayer(follow.player);
			if (player == null || follow.untilTick < tick || player.isDeadOrDying()) {
				it.remove();
				continue;
			}
			double x = player.getX();
			double z = player.getZ();
			if (follow.started) {
				double dx = x - follow.lastX;
				double dz = z - follow.lastZ;
				// A teleport or a ride is no step.
				if (dx * dx + dz * dz < 1 && player.getVehicle() == null
						&& follow.stride.moved(dx, dz, player.onGround(), player.isShiftKeyDown())) {
					double[] at = Stride.behind(x, z, dx, dz, follow.behind);
					due.add(new Step(follow.player, tick + follow.delay, at[0], player.getY(), at[1]));
				}
			}
			follow.lastX = x;
			follow.lastZ = z;
			follow.started = true;
		}

		Iterator<Step> steps = due.iterator();
		while (steps.hasNext()) {
			Step step = steps.next();
			if (step.atTick > tick) {
				continue;
			}
			steps.remove();
			ServerPlayer player = server.getPlayerList().getPlayer(step.player);
			if (player == null) {
				continue;
			}
			BlockPos under = BlockPos.containing(step.x, step.y - 0.2, step.z);
			BlockState state = player.level().getBlockState(under);
			if (state.isAir()) {
				state = player.level().getBlockState(under.below());
			}
			if (state.isAir()) {
				continue;
			}
			SoundType type = state.getSoundType();
			player.connection.send(new ClientboundSoundPacket(Holder.direct(type.getStepSound()), SoundSource.HOSTILE,
					step.x, step.y, step.z, type.getVolume() * 0.3f, type.getPitch() * 0.95f, player.level().getRandom().nextLong()));
		}
	}

	/** Every follow of this player stopped. */
	public void stop(UUID player) {
		active.removeIf(f -> f.player.equals(player));
		due.removeIf(s -> s.player.equals(player));
	}
}
