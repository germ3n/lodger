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
	void a_press_on_a_setting_of_the_wrong_shape_changes_nothing_and_never_throws() {
		Setting empty = setting("{\"key\":\"x\",\"value\":\"a\"}");
		assertEquals("a", empty.next(new JsonPrimitive("a"), true).getAsString(), "a choice with no options");
		Setting none = setting("{\"key\":\"x\",\"kind\":\"toggle\"}");
		assertEquals(com.google.gson.JsonNull.INSTANCE, none.next(com.google.gson.JsonNull.INSTANCE, true), "no value");
		Setting count = setting("{\"key\":\"x\",\"kind\":\"number\",\"min\":0,\"max\":4,\"step\":0,\"value\":1}");
		assertEquals(1, count.next(new JsonPrimitive(1), true).getAsInt(), "a step of nothing");
		Setting flag = setting("{\"key\":\"x\",\"kind\":\"toggle\",\"value\":\"yes\"}");
		assertEquals("yes", flag.next(new JsonPrimitive("yes"), true).getAsString(), "a toggle that is not a boolean");
	}

	@Test
	void a_whole_step_writes_a_whole_number() {
		Setting plans = setting("{\"key\":\"brain.most_plans\",\"kind\":\"number\",\"min\":1,\"max\":4,\"step\":1,\"value\":3}");
		assertEquals("2", plans.next(new JsonPrimitive(3), false).toString(), "a count, not 2.0");
		assertEquals("{\"brain\":{\"most_plans\":4}}", plans.change(plans.next(new JsonPrimitive(3), true)).toString());
		Setting share = setting("{\"key\":\"model.gpu_share\",\"kind\":\"number\",\"min\":0,\"max\":1,\"step\":0.1,\"value\":0.9}");
		assertEquals("1.0", share.next(new JsonPrimitive(0.9), true).toString(), "a share stays a fraction");
	}

	@Test
	void a_dotted_key_becomes_a_nested_change() {
		Setting size = setting("{\"key\":\"model.size\",\"kind\":\"choice\",\"options\":[\"3b\"],\"value\":\"3b\"}");
		assertEquals("{\"model\":{\"size\":\"0.5b\"}}", size.change(new JsonPrimitive("0.5b")).toString());
		assertEquals("on", Setting.show(new JsonPrimitive(true)));
	}
}
