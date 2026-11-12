package craftkit.host.world;

import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import craftkit.host.Host;
import net.minecraft.resources.Identifier;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.saveddata.SavedData;
import net.minecraft.world.level.saveddata.SavedDataType;

/**
 * One mod's bytes for one level, kept in the level's saved data under {@code craftkit_<mod id>},
 * as raw bytes ({@link SavedMemory}; the ABI hands them out as base64). The mod owns what they mean.
 * Beside them, the items the mod took and keeps ({@code take_item}'s {@code keep}), as the game
 * saves items, components and all, so what is given back is the very thing taken.
 */
public final class WorldData extends SavedData {
	private String bytes;
	private final java.util.Map<String, ItemStack> kept = new java.util.HashMap<>();

	public WorldData() {
		this("", java.util.Map.of());
	}

	public WorldData(String bytes, java.util.Map<String, ItemStack> kept) {
		this.bytes = bytes;
		this.kept.putAll(kept);
	}

	public static SavedDataType<WorldData> type(String modId) {
		Codec<WorldData> codec = RecordCodecBuilder.create(instance -> instance
				.group(
						SavedMemory.CODEC.forGetter(data -> data.bytes),
						Codec.unboundedMap(Codec.STRING, ItemStack.CODEC).optionalFieldOf("kept", java.util.Map.of()).forGetter(data -> data.kept))
				.apply(instance, WorldData::new));

		java.util.function.Supplier<WorldData> fresh = WorldData::new;

		return new SavedDataType<>(Identifier.fromNamespaceAndPath(Host.MOD_ID, modId), fresh, codec, null);
	}

	/** The saved bytes as base64, empty for a new world. */
	public String bytes() {
		return bytes;
	}

	/** Keeps a taken stack under a key; a stack already kept there is added to if the same. */
	public void keep(String key, ItemStack stack) {
		ItemStack had = kept.get(key);
		if (had != null && ItemStack.isSameItemSameComponents(had, stack) && had.getCount() + stack.getCount() <= had.getMaxStackSize()) {
			had.grow(stack.getCount());
		} else if (had != null && ItemStack.isSameItemSameComponents(had, stack)) {
			// A full stack: the rest beside it. A stack saved over 99 threw at every save and
			// stopped the world saving at all.
			int room = had.getMaxStackSize() - had.getCount();
			had.grow(room);
			ItemStack rest = stack.copy();
			rest.shrink(room);
			int n = 1;
			while (kept.containsKey(key + "_" + n)) {
				n++;
			}
			kept.put(key + "_" + n, rest);
		} else if (had == null) {
			kept.put(key, stack.copy());
		} else {
			// Another item under that name already: kept beside it, never over it (a second
			// piece under one name used to destroy the first).
			int n = 1;
			while (kept.containsKey(key + "_" + n)) {
				n++;
			}
			kept.put(key + "_" + n, stack.copy());
		}
		setDirty();
	}

	/** Whether something is kept under a key. */
	public boolean has(String key) {
		return kept.containsKey(key);
	}

	/** Whether what is kept under a key is the same item as {@code stack}, so the two stack. */
	public boolean holdsLike(String key, ItemStack stack) {
		ItemStack had = kept.get(key);
		return had != null && ItemStack.isSameItemSameComponents(had, stack);
	}

	/** The stack kept under a key, taken out, or null. */
	public ItemStack giveBack(String key) {
		ItemStack stack = kept.remove(key);
		if (stack != null) {
			setDirty();
		}
		return stack;
	}

	/** The keys kept that begin with {@code prefix}, in order. */
	public java.util.List<String> keysStarting(String prefix) {
		return kept.keySet().stream().filter(k -> k.startsWith(prefix)).sorted().toList();
	}

	public void set(String base64) {
		if (!base64.equals(bytes)) {
			bytes = base64;
			setDirty();
		}
	}
}
