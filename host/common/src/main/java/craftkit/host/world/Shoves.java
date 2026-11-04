package craftkit.host.world;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.network.protocol.game.ClientboundSoundPacket;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * A push off an edge, armed for a while ({@code shove}, ABI 37; `shove`, batch 16): the first
 * moment the player stands at a drop of two to three blocks onto solid ground, or on a ladder
 * with ground that near below, not crouching, with more than two hearts and not flying, they
 * are pushed off it once, and a step sounds right behind them. The fall is never more than
 * {@code most} blocks ({@link Edge}), so it never harms, let alone kills.
 */
public final class Shoves {
	/** The longest a push waits for its edge, whatever a mod asks. */
	public static final float MOST_SECONDS = 300;
	private static final int MOST_DROP = 3;
	/** How hard: a stumble, enough to clear the edge, no more. */
	private static final double PUSH = 0.45;

	private record Armed(UUID player, long untilTick, int most) {
	}

	private final List<Armed> armed = new ArrayList<>();
	private long tick;

	public void arm(ServerPlayer player, float seconds, int most) {
		armed.removeIf(a -> a.player.equals(player.getUUID()));
		armed.add(new Armed(player.getUUID(), tick + (long) (Math.min(seconds, MOST_SECONDS) * 20), Math.max(2, Math.min(most, MOST_DROP))));
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Armed> it = armed.iterator();
		while (it.hasNext()) {
			Armed each = it.next();
			ServerPlayer player = server.getPlayerList().getPlayer(each.player);
			if (player == null || each.untilTick < tick) {
				it.remove();
				continue;
			}
			if (tick % 5 != 0 || player.isShiftKeyDown() || player.getHealth() <= 4 || player.getAbilities().flying
					|| player.isSpectator() || player.isPassenger() || player.isInWater()
					|| !(player.onGround() || player.onClimbable())) {
				continue;
			}
			Direction way = edge(player, each.most);
			if (way == null) {
				continue;
			}
			it.remove();
			Vec3 push = new Vec3(way.getStepX() * PUSH, 0, way.getStepZ() * PUSH);
			player.setDeltaMovement(push);
			player.connection.send(new ClientboundSetEntityMotionPacket(player));
			Vec3 behind = player.position().subtract(push.normalize().scale(0.8));
			BlockState under = player.level().getBlockState(player.blockPosition().below());
			var type = (under.isAir() ? net.minecraft.world.level.block.Blocks.STONE.defaultBlockState() : under).getSoundType();
			player.connection.send(new ClientboundSoundPacket(Holder.direct(type.getStepSound()), SoundSource.HOSTILE,
					behind.x, behind.y, behind.z, type.getVolume(), type.getPitch() * 0.9f, player.level().getRandom().nextLong()));
		}
	}

	/** The way off a safe edge from where they stand, the way they face first, or null. */
	private static Direction edge(ServerPlayer player, int most) {
		ServerLevel level = (ServerLevel) player.level();
		BlockPos feet = player.blockPosition();
		Direction facing = player.getDirection();
		List<Direction> ways = new ArrayList<>(List.of(facing, facing.getClockWise(), facing.getCounterClockWise(), facing.getOpposite()));
		for (Direction way : ways) {
			// Off a ladder, away from the wall it hangs on; on the ground, any edge near enough
			// to step off: the block out from their feet's own.
			BlockPos out = feet.relative(way);
			if (!player.onClimbable()) {
				Vec3 at = player.position();
				double toEdge = switch (way) {
					case EAST -> feet.getX() + 1 - at.x;
					case WEST -> at.x - feet.getX();
					case SOUTH -> feet.getZ() + 1 - at.z;
					default -> at.z - feet.getZ();
				};
				if (toEdge > 0.45) {
					continue;
				}
			}
			int drop = Edge.drop(new Edge.Column() {
				@Override
				public boolean open(int depth) {
					BlockPos pos = out.below(depth);
					BlockState state = level.getBlockState(pos);
					return state.getCollisionShape(level, pos).isEmpty() && state.getFluidState().isEmpty();
				}

				@Override
				public boolean ground(int depth) {
					BlockPos pos = out.below(depth);
					return level.getBlockState(pos).isFaceSturdy(level, pos, Direction.UP);
				}
			}, most);
			if (drop > 0) {
				return way;
			}
		}

		return null;
	}

	/** Everything for every player, forgotten: the world is closing, and none of it is the next one's. */
	public void clear() {
		armed.clear();
	}
}
