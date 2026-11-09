package craftkit.host.client;

import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class ConfigTextTest {
	@Test
	void a_typed_value_is_read_back_as_the_kind_the_key_had() {
		assertEquals(new JsonPrimitive(48L), ConfigText.parse(new JsonPrimitive(48), "48"));
		assertNull(ConfigText.parse(new JsonPrimitive(48), "4.5"), "a whole number stays whole");
		assertEquals(new JsonPrimitive(0.25), ConfigText.parse(new JsonPrimitive(0.7), "0.25"));
		assertNull(ConfigText.parse(new JsonPrimitive(0.7), "lots"));
		assertEquals(new JsonPrimitive("4b"), ConfigText.parse(new JsonPrimitive("1.5b"), "4b"));
		assertEquals(JsonParser.parseString("[1.0, 2.0]"), ConfigText.parse(JsonParser.parseString("[40.0]"), "[1.0, 2.0]"));
		assertNull(ConfigText.parse(JsonParser.parseString("[]"), "7"), "a list stays a list");
		assertEquals("4b", ConfigText.text(new JsonPrimitive("4b")), "a string bare");
		assertEquals("[40.0,40.0]", ConfigText.text(JsonParser.parseString("[40.0, 40.0]")));
	}
}
