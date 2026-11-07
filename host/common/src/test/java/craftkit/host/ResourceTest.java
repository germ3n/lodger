package craftkit.host;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ResourceTest {
	private static Resource parse(String json) {
		return Resource.parse(JsonParser.parseString(json).getAsJsonObject(), Path.of("/game"));
	}

	@Test
	void a_relative_path_is_under_the_game_directory_and_an_absolute_one_is_kept() {
		assertEquals(Path.of("/game/config/lodger/models/qwen.gguf"), parse("{\"id\":\"m\",\"url\":\"https://x/m\",\"path\":\"config/lodger/models/qwen.gguf\"}").path());
		assertEquals(Path.of("/models/q.gguf"), parse("{\"id\":\"m\",\"url\":\"https://x/m\",\"path\":\"/models/q.gguf\"}").path());
	}

	@Test
	void the_defaults_are_required_with_no_magic_or_hash_and_the_id_as_title() {
		Resource resource = parse("{\"id\":\"m\",\"url\":\"https://x/m\",\"path\":\"m\"}");

		assertTrue(resource.required());
		assertEquals("m", resource.title());
		assertArrayEquals(new byte[0], resource.magic());
		assertEquals("", resource.sha256());
		assertEquals(0, resource.sizeHint());
		assertTrue(!resource.hasLicense());
	}

	@Test
	void a_license_is_read_with_its_name_url_and_text() {
		Resource resource = parse("{\"id\":\"m\",\"url\":\"https://x/m\",\"path\":\"m\",\"license\":{\"name\":\"Apache License 2.0\",\"url\":\"https://x/LICENSE\",\"text\":\"terms\"}}");

		assertTrue(resource.hasLicense());
		assertEquals("Apache License 2.0", resource.license().name());
		assertEquals("terms", resource.license().text());
	}

	@Test
	void bad_resources_are_refused_by_name() {
		for (String[] bad : new String[][] {
				{"{\"url\":\"https://x/m\",\"path\":\"m\"}", "no id"},
				{"{\"id\":\"m\",\"url\":\"ftp://x/m\",\"path\":\"m\"}", "http"},
				{"{\"id\":\"m\",\"url\":\"https://x/m\"}", "no path"},
				{"{\"id\":\"m\",\"url\":\"https://x/m\",\"path\":\"m\",\"sha256\":\"abc\"}", "64 hex"},
		}) {
			IllegalArgumentException e = assertThrows(IllegalArgumentException.class, () -> parse(bad[0]), bad[0]);
			assertTrue(e.getMessage().contains(bad[1]), bad[0] + ": " + e.getMessage());
		}
	}

	@Test
	void a_resource_may_say_how_to_play_without_it_and_each_choice_its_note() {
		JsonObject json = JsonParser.parseString("""
				{"id": "model", "url": "https://x/m", "path": "m",
				 "without": {"model": {"enabled": false}},
				 "choices": [{"label": "small", "config": {}, "chosen": true, "note": "quick"}, {"label": "big", "config": {}}]}
				""").getAsJsonObject();
		Resource resource = Resource.parse(json, java.nio.file.Path.of("/game"));

		assertEquals(false, resource.without().getAsJsonObject("model").get("enabled").getAsBoolean());
		assertEquals("quick", resource.choices().get(0).note());
		assertEquals("", resource.choices().get(1).note(), "no note: none shown");
		assertEquals(null, Resource.parse(JsonParser.parseString("{\"id\": \"m\", \"url\": \"https://x/m\", \"path\": \"m\"}").getAsJsonObject(), java.nio.file.Path.of("/game")).without());
	}
}
