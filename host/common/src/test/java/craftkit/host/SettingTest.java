package craftkit.host;

import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class SettingTest {
	private static Setting setting(String json) {
		return Setting.parse(JsonParser.parseString(json).getAsJsonObject());
	}

	@Test
	void a_choice_cycles_a_toggle_flips_and_a_number_steps_within_its_range() {
		Setting size = setting("{\"key\":\"model.size\",\"kind\":\"choice\",\"options\":[\"0.5b\",\"1.5b\",\"3b\"],\"value\":\"3b\"}");
		assertEquals("0.5b", size.next(new JsonPrimitive("3b"), true).getAsString(), "round to the first");
		assertEquals("1.5b", size.next(new JsonPrimitive("3b"), false).getAsString());
		Setting chat = setting("{\"key\":\"events.chat\",\"kind\":\"toggle\",\"value\":true}");
		assertEquals(false, chat.next(new JsonPrimitive(true), true).getAsBoolean());
		Setting share = setting("{\"key\":\"model.gpu_share\",\"kind\":\"number\",\"min\":0,\"max\":1,\"step\":0.1,\"value\":0.5}");
		assertEquals(0.6, share.next(new JsonPrimitive(0.5), true).getAsDouble(), 1e-9);
		assertEquals(1.0, share.next(new JsonPrimitive(1.0), true).getAsDouble(), 1e-9, "never past the max");
		assertEquals(0.0, share.next(new JsonPrimitive(0.0), false).getAsDouble(), 1e-9);
	}

	@Test
	void a_dotted_key_becomes_a_nested_change() {
		Setting size = setting("{\"key\":\"model.size\",\"kind\":\"choice\",\"options\":[\"3b\"],\"value\":\"3b\"}");
		assertEquals("{\"model\":{\"size\":\"0.5b\"}}", size.change(new JsonPrimitive("0.5b")).toString());
		assertEquals("on", Setting.show(new JsonPrimitive(true)));
	}
}
