package craftkit.host.world;

import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import craftkit.host.Host;
import net.minecraft.resources.Identifier;
import net.minecraft.world.level.saveddata.SavedData;
import net.minecraft.world.level.saveddata.SavedDataType;

/**
 * One mod's bytes for one level, kept in the level's saved data under {@code craftkit_<mod id>},
 * as the base64 the ABI hands out, so the host never decodes them. The mod owns what they mean.
 */
public final class WorldData extends SavedData {
	private String bytes;

	public WorldData() {
		this("");
	}

	public WorldData(String bytes) {
		this.bytes = bytes;
	}

	public static SavedDataType<WorldData> type(String modId) {
		Codec<WorldData> codec = RecordCodecBuilder.create(instance -> instance
				.group(Codec.STRING.fieldOf("bytes").forGetter(data -> data.bytes))
				.apply(instance, WorldData::new));

		java.util.function.Supplier<WorldData> fresh = WorldData::new;

		return new SavedDataType<>(Identifier.fromNamespaceAndPath(Host.MOD_ID, modId), fresh, codec, null);
	}

	/** The saved bytes as base64, empty for a new world. */
	public String bytes() {
		return bytes;
	}

	public void set(String base64) {
		if (!base64.equals(bytes)) {
			bytes = base64;
			setDirty();
		}
	}
}
