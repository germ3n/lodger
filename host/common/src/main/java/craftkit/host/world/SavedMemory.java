package craftkit.host.world;

import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;

import java.nio.ByteBuffer;
import java.util.Base64;
import java.util.Optional;

/** How {@link WorldData} saves a mod's bytes, apart from the game so JUnit checks it. */
public final class SavedMemory {
	private SavedMemory() {
	}

	/**
	 * The mod's bytes as the save holds them: raw, in a byte array (NBT's own, as large as the
	 * memory grows), or an older save's base64 string. An NBT string holds 65535 bytes at most:
	 * a long stay's memory outgrew it, its every save failed and the game wrote it empty, so the
	 * world opened with nothing (its stage, its plans, all it knew). Written as the array alone.
	 */
	public static final MapCodec<String> CODEC = Codec.mapPair(
					Codec.BYTE_BUFFER.optionalFieldOf("memory"),
					Codec.STRING.optionalFieldOf("bytes", ""))
			.xmap(pair -> pair.getFirst().map(SavedMemory::base64).orElse(pair.getSecond()),
					base64 -> Pair.of(base64.isEmpty() ? Optional.<ByteBuffer>empty() : Optional.of(ByteBuffer.wrap(Base64.getDecoder().decode(base64))), ""));

	private static String base64(ByteBuffer buffer) {
		byte[] raw = new byte[buffer.remaining()];
		buffer.duplicate().get(raw);

		return Base64.getEncoder().encodeToString(raw);
	}
}
