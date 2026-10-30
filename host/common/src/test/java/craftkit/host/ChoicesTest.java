package craftkit.host;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ChoicesTest {
	@Test
	void a_resource_reads_its_choices_and_one_is_chosen() {
		Resource resource = Resource.parse(JsonParser.parseString("""
				{"id": "model", "url": "https://x/m", "path": "/m.gguf", "choices": [
				  {"label": "Qwen2.5 0.5B, 491 MB", "config": {"model": {"size": "0.5b"}}, "chosen": false},
				  {"label": "Qwen2.5 1.5B, 1117 MB", "config": {"model": {"size": "1.5b"}}, "chosen": true}
				]}
				""").getAsJsonObject(), Path.of("/game"));

		assertEquals(2, resource.choices().size());
		assertTrue(resource.choices().get(1).chosen());
		assertEquals("0.5b", resource.choices().get(0).config().getAsJsonObject("model").get("size").getAsString());
	}

	@Test
	void a_choice_is_merged_into_the_config_and_the_rest_of_it_kept() throws Exception {
		Path dir = Files.createTempDirectory("craftkit-config");
		Path file = dir.resolve("lodger.json");
		Files.writeString(file, "{\"preset\": \"calm\", \"model\": {\"threads\": 4, \"size\": \"1.5b\"}}");

		JsonObject change = JsonParser.parseString("{\"model\": {\"size\": \"3b\"}}").getAsJsonObject();
		new ConfigFile(file).merge(change);

		JsonObject after = JsonParser.parseString(Files.readString(file)).getAsJsonObject();
		assertEquals("3b", after.getAsJsonObject("model").get("size").getAsString());
		assertEquals(4, after.getAsJsonObject("model").get("threads").getAsInt(), "the rest of model kept");
		assertEquals("calm", after.get("preset").getAsString());
	}
}
