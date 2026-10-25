package craftkit.host.world;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import craftkit.host.LoadedMod;
import craftkit.host.senses.Senses;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.Heightmap;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.TreeMap;

/**
 * The pass over a point's surroundings a mod asks for with the {@code surroundings} command,
 * sliced over ticks so it never shows in a tick profile: a column every four blocks within the
 * radius, read at the surface, counted into features the SDK's {@code Surroundings} names:
 * {@code water} (surface water, with how many columns), {@code lava}, {@code snow}, {@code sand},
 * {@code trees} (leaves overhead), {@code crops} (farmland), {@code lights} (torches and
 * lanterns), {@code beds}, {@code containers}, {@code doors}. First cut; villages, tunnels and
 * caves are the todo's.
 */
public final class Surroundings {
	/** Columns read a tick. */
	private static final int PER_TICK = 64;
	private static final int STEP = 4;

	private static final List<Pass> PASSES = new ArrayList<>();

	private static final class Pass {
		final ServerLevel level;
		final LoadedMod mod;
		final long id;
		final BlockPos around;
		final int radius;
		final Senses senses;
		int x;
		int z;
		final TreeMap<String, int[]> features = new TreeMap<>();

		Pass(ServerLevel level, LoadedMod mod, long id, BlockPos around, int radius, Senses senses) {
			this.level = level;
			this.mod = mod;
			this.id = id;
			this.around = around;
			this.radius = radius;
			this.senses = senses;
			this.x = -radius;
			this.z = -radius;
		}

		boolean step() {
			for (int done = 0; done < PER_TICK; done++) {
				if (z > radius) {
					return true;
				}

				column(around.offset(x, 0, z));
				x += STEP;
				if (x > radius) {
					x = -radius;
					z += STEP;
				}
			}

			return z > radius;
		}

		private void column(BlockPos at) {
			if (!level.isLoaded(at)) {
				return;
			}

			BlockPos surface = level.getHeightmapPos(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, at);
			BlockState top = level.getBlockState(surface.below());
			BlockState above = level.getBlockState(surface);
			BlockPos leaves = level.getHeightmapPos(Heightmap.Types.MOTION_BLOCKING, at);

			if (top.is(Blocks.WATER) || above.is(Blocks.WATER)) {
				count("water", surface);
			} else if (top.is(Blocks.LAVA)) {
				count("lava", surface);
			} else if (top.is(BlockTags.SAND)) {
				count("sand", surface);
			}

			if (top.is(Blocks.SNOW_BLOCK) || above.is(Blocks.SNOW) || top.is(Blocks.POWDER_SNOW)) {
				count("snow", surface);
			}

			if (leaves.getY() > surface.getY() && level.getBlockState(leaves.below()).is(BlockTags.LEAVES)) {
				count("trees", surface);
			}

			if (top.is(Blocks.FARMLAND)) {
				count("crops", surface);
			}

			for (int dy = -1; dy <= 3; dy++) {
				BlockState state = level.getBlockState(surface.offset(0, dy, 0));
				if (state.is(Blocks.TORCH) || state.is(Blocks.WALL_TORCH) || state.is(Blocks.LANTERN) || state.is(Blocks.SOUL_LANTERN)) {
					count("lights", surface);
				} else if (state.is(BlockTags.BEDS)) {
					count("beds", surface);
				} else if (state.is(Blocks.CHEST) || state.is(Blocks.BARREL) || state.is(BlockTags.SHULKER_BOXES)) {
					count("containers", surface);
				} else if (state.is(BlockTags.DOORS)) {
					count("doors", surface);
				}
			}
		}

		private void count(String kind, BlockPos at) {
			// Count, and the sum of positions for the center.
			int[] entry = features.computeIfAbsent(kind, k -> new int[4]);
			entry[0]++;
			entry[1] += at.getX();
			entry[2] += at.getY();
			entry[3] += at.getZ();
		}

		JsonObject result() {
			JsonObject result = new JsonObject();
			result.addProperty("id", id);
			result.add("around", Senses.blockPos(around));
			JsonArray array = new JsonArray();
			features.forEach((kind, entry) -> {
				JsonObject feature = new JsonObject();
				feature.addProperty("kind", kind);
				feature.add("pos", Senses.blockPos(new BlockPos(entry[1] / entry[0], entry[2] / entry[0], entry[3] / entry[0])));
				// Columns are STEP apart, so each stands for STEP squared blocks.
				feature.addProperty("size", (int) Math.sqrt((double) entry[0] * STEP * STEP));
				feature.addProperty("detail", entry[0] + " columns");
				array.add(feature);
			});
			result.add("features", array);

			return result;
		}
	}

	private Surroundings() {
	}

	public static void start(ServerLevel level, LoadedMod mod, long id, BlockPos around, int radius, Senses senses) {
		PASSES.add(new Pass(level, mod, id, around, Math.min(radius, 160), senses));
	}

	/** A slice of every running pass; a finished one hands its result to the senses. */
	public static void tick() {
		Iterator<Pass> it = PASSES.iterator();
		while (it.hasNext()) {
			Pass pass = it.next();
			if (pass.step()) {
				pass.senses.surrounded(pass.level, pass.result());
				it.remove();
			}
		}
	}

	/** Drops passes for a level that unloaded. */
	public static void forget(ServerLevel level) {
		PASSES.removeIf(pass -> pass.level == level);
	}
}
