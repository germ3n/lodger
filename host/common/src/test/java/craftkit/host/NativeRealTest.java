package craftkit.host;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The real lodger library through the real FFM binding, when the repository has it built
 * ({@code cargo build --release -p lodger-core}) and, for the model parts, the model at
 * {@code core/models/qwen.gguf}. Skipped otherwise, so the CI's host job, which has neither,
 * still passes; the dev machine runs it.
 */
class NativeRealTest {
	private static Path library() {
		String os = Natives.os(System.getProperty("os.name"));
		return Repo.root().resolve("target").resolve("release").resolve(Natives.fileName(os, "lodger_core"));
	}

	@Test
	void the_real_library_answers_its_abi_version_info_and_init(@TempDir Path game) throws Exception {
		Assumptions.assumeTrue(Files.isRegularFile(library()), "no release library built");

		try (Native lodger = Native.load(library())) {
			assertEquals(Native.ABI_VERSION, lodger.abiVersion());
			JsonObject info = JsonParser.parseString(lodger.info()).getAsJsonObject();
			assertEquals("lodger", info.get("id").getAsString());

			JsonObject init = new JsonObject();
			init.addProperty("config", "");
			init.addProperty("game_dir", game.toString());
			init.addProperty("config_dir", game.resolve("config").toString());
			init.addProperty("id", "lodger");
			JsonObject reply = JsonParser.parseString(lodger.init(init.toString())).getAsJsonObject();
			assertFalse(reply.has("error"), reply.toString());
			assertEquals("model", reply.getAsJsonArray("resources").get(0).getAsJsonObject().get("id").getAsString());

			JsonObject status = JsonParser.parseString(lodger.status()).getAsJsonObject();
			assertFalse(status.get("ready").getAsBoolean());
			assertTrue(status.get("line").getAsString().contains("not downloaded"), status.toString());
			lodger.shutdown();
		}
	}

	/** The server's own layout: an empty config, the model at the default path under the game directory. */
	@Test
	void with_the_model_at_its_default_path_warm_loads_it_too(@TempDir Path game) throws Exception {
		Path model = Repo.root().resolve("core").resolve("models").resolve("qwen.gguf");
		Assumptions.assumeTrue(Files.isRegularFile(library()), "no release library built");
		Assumptions.assumeTrue(Files.isRegularFile(model), "no model at core/models/qwen.gguf");
		Path at = game.resolve("config/lodger/models/qwen2.5-0.5b-q4_k_m.gguf");
		Files.createDirectories(at.getParent());
		Files.createSymbolicLink(at, model);

		try (Native lodger = Native.load(library())) {
			JsonObject init = new JsonObject();
			init.addProperty("config", "");
			init.addProperty("game_dir", game.toString());
			init.addProperty("config_dir", game.resolve("config").toString());
			init.addProperty("id", "lodger");
			JsonObject reply = JsonParser.parseString(lodger.init(init.toString())).getAsJsonObject();
			assertFalse(reply.has("error"), reply.toString());
			String before = lodger.status();
			assertTrue(before.contains("\"ready\":false"), before);
			assertTrue(before.contains("not loaded yet"), before);

			String warmed = lodger.warm();
			assertEquals("", LoadedMod.errorOf(warmed), warmed);
			String after = lodger.status();
			assertTrue(after.contains("\"ready\":true"), after);
			lodger.shutdown();
		}
	}

	@Test
	void with_the_real_model_warm_loads_it_and_status_says_so(@TempDir Path game) throws Exception {
		Path model = Repo.root().resolve("core").resolve("models").resolve("qwen.gguf");
		Assumptions.assumeTrue(Files.isRegularFile(library()), "no release library built");
		Assumptions.assumeTrue(Files.isRegularFile(model), "no model at core/models/qwen.gguf");

		try (Native lodger = Native.load(library())) {
			JsonObject init = new JsonObject();
			init.addProperty("config", "{ \"model\": { \"path\": \"" + model.toString().replace("\\", "\\\\") + "\" } }");
			init.addProperty("game_dir", game.toString());
			init.addProperty("config_dir", game.resolve("config").toString());
			init.addProperty("id", "lodger");
			JsonObject reply = JsonParser.parseString(lodger.init(init.toString())).getAsJsonObject();
			assertFalse(reply.has("error"), reply.toString());

			String warmed = lodger.warm();
			assertEquals("", LoadedMod.errorOf(warmed), warmed);
			JsonObject status = JsonParser.parseString(lodger.status()).getAsJsonObject();
			assertTrue(status.get("ready").getAsBoolean(), status.toString());
			assertEquals("The model is loaded", status.get("line").getAsString(), status.toString());
			lodger.shutdown();
		}
	}
}
