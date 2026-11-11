package craftkit.host.world;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import org.junit.jupiter.api.Test;

import java.util.Base64;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SavedMemoryTest {
	private static JsonObject encode(String base64) {
		return SavedMemory.CODEC.codec().encodeStart(JsonOps.INSTANCE, base64).getOrThrow().getAsJsonObject();
	}

	private static String decode(JsonElement saved) {
		return SavedMemory.CODEC.codec().parse(JsonOps.INSTANCE, saved).getOrThrow();
	}

	@Test
	void a_memory_past_an_nbt_string_s_limit_is_saved_as_bytes_and_read_back_whole() {
		// 68 KB of base64, as the world that opened with nothing saved: an NBT string holds
		// 65535 bytes at most, and its every save failed.
		byte[] raw = new byte[51_000];
		new Random(7).nextBytes(raw);
		String base64 = Base64.getEncoder().encodeToString(raw);
		assertTrue(base64.length() > 65_535);

		JsonObject saved = encode(base64);
		assertTrue(saved.get("memory").isJsonArray(), "the bytes as an array: " + saved.keySet());
		assertEquals(raw.length, saved.getAsJsonArray("memory").size());
		assertFalse(saved.has("bytes"), "no string beside it");
		assertEquals(base64, decode(saved));
	}

	@Test
	void an_older_save_s_base64_string_is_still_read() {
		JsonObject older = new JsonObject();
		older.addProperty("bytes", "AXsieCI6MX0=");
		assertEquals("AXsieCI6MX0=", decode(older));
	}

	@Test
	void a_new_world_saves_nothing_and_opens_empty() {
		assertEquals(0, encode("").size());
		assertEquals("", decode(new JsonObject()));
	}
}
