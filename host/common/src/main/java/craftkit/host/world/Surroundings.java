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
		/** Chunks whose structures were looked up already: a structure start is per chunk. */
		final java.util.Set<Long> chunksChecked = new java.util.HashSet<>();

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

		/** The structures the pass names, by the tag that groups them and the plain word the mod reads. */
		private static final java.util.Map<net.minecraft.tags.TagKey<net.minecraft.world.level.levelgen.structure.Structure>, String> STRUCTURES = java.util.Map.of(
				net.minecraft.tags.StructureTags.VILLAGE, "village",
				net.minecraft.tags.StructureTags.MINESHAFT, "mineshaft",
				net.minecraft.tags.StructureTags.RUINED_PORTAL, "ruined_portal",
				net.minecraft.tags.StructureTags.SHIPWRECK, "shipwreck",
				net.minecraft.tags.StructureTags.OCEAN_RUIN, "ocean_ruin");

		private void column(BlockPos at) {
			if (!level.isLoaded(at)) {
				return;
			}

			structures(at);

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
				if (state.is(Blocks.BELL)) {
					count("bell", surface);
				} else if (state.is(Blocks.TORCH) || state.is(Blocks.WALL_TORCH) || state.is(Blocks.LANTERN) || state.is(Blocks.SOUL_LANTERN)) {
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

		/** Once per chunk: which named structures stand here, counted at their bounding box's middle. */
		private void structures(BlockPos at) {
			long chunk = new net.minecraft.world.level.ChunkPos(at.getX() >> 4, at.getZ() >> 4).pack();
			if (!chunksChecked.add(chunk)) {
				return;
			}

			var registry = level.registryAccess().lookupOrThrow(net.minecraft.core.registries.Registries.STRUCTURE);
			STRUCTURES.forEach((tag, word) -> registry.get(tag).ifPresent(set -> {
				net.minecraft.world.level.levelgen.structure.StructureStart start = level.structureManager().getStructureAt(at, set);
				if (start.isValid()) {
					BlockPos center = start.getBoundingBox().getCenter();
					int[] entry = features.computeIfAbsent(word, k -> new int[4]);
					if (entry[0] == 0) {
						entry[0] = 1;
						entry[1] = center.getX();
						entry[2] = center.getY();
						entry[3] = center.getZ();
					}
				}
			}));
		}

		private void count(String kind, BlockPos at) {
			// Count, and the sum of positions for the center.
			int[] entry = features.computeIfAbsent(kind, k -> new int[4]);
			entry[0]++;
			entry[1] += at.getX();
			entry[2] += at.getY();
			entry[3] += at.getZ();
		}

		/** How far the house is read around the pass's middle: the bed, most days. */
		private static final int HOUSE = 8;
		private static final int MOST_OF_A_KIND = 32;

		/**
		 * The house around the middle, block by block, once: each door (its lower half), light,
		 * chest and bed head as a feature of its own, {@code house_door} and so on, so a mod
		 * knows the house a player sleeps in whoever built it (a village house, generated
		 * torches). About 2,600 blocks, once a day.
		 */
		private void house(JsonArray array) {
			java.util.Map<String, Integer> counts = new java.util.HashMap<>();
			for (BlockPos at : BlockPos.betweenClosed(around.offset(-HOUSE, -3, -HOUSE), around.offset(HOUSE, 5, HOUSE))) {
				if (!level.isLoaded(at)) {
					continue;
				}
				BlockState state = level.getBlockState(at);
				String kind = null;
				if (state.is(BlockTags.DOORS) && state.getValue(net.minecraft.world.level.block.DoorBlock.HALF) == net.minecraft.world.level.block.state.properties.DoubleBlockHalf.LOWER) {
					kind = "house_door";
				} else if (state.is(Blocks.TORCH) || state.is(Blocks.WALL_TORCH) || state.is(Blocks.SOUL_TORCH) || state.is(Blocks.SOUL_WALL_TORCH) || state.is(Blocks.LANTERN) || state.is(Blocks.SOUL_LANTERN)) {
					kind = "house_light";
				} else if (state.is(Blocks.CHEST) || state.is(Blocks.BARREL) || state.is(Blocks.TRAPPED_CHEST)) {
					kind = "house_chest";
				} else if (state.is(BlockTags.BEDS) && state.getValue(net.minecraft.world.level.block.BedBlock.PART) == net.minecraft.world.level.block.state.properties.BedPart.HEAD) {
					kind = "house_bed";
				}
				if (kind == null || counts.merge(kind, 1, Integer::sum) > MOST_OF_A_KIND) {
					continue;
				}
				JsonObject feature = new JsonObject();
				feature.addProperty("kind", kind);
				feature.add("pos", Senses.blockPos(at.immutable()));
				feature.addProperty("size", 1);
				feature.addProperty("detail", Senses.blockId(state));
				array.add(feature);
			}
		}

		JsonObject result() {
			JsonObject result = new JsonObject();
			result.addProperty("id", id);
			result.add("around", Senses.blockPos(around));
			JsonArray array = new JsonArray();
			house(array);
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
