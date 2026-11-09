package craftkit.host.client;

import com.google.gson.JsonElement;
import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;

/** How the settings' All keys tab shows a config value in a field and reads it back. */
final class ConfigText {
	private ConfigText() {
	}

	/** A value as its field shows it: a string bare, a number or a list as JSON. */
	static String text(JsonElement value) {
		if (value.isJsonPrimitive() && value.getAsJsonPrimitive().isString()) {
			return value.getAsString();
		}
		return value.toString();
	}

	/**
	 * A field's text read back as the kind of value the key had: a yes or no, a whole number or
	 * not, a string, a list. Null when it is not one (and the row is said).
	 */
	static JsonElement parse(JsonElement was, String text) {
		try {
			if (was.isJsonPrimitive()) {
				JsonPrimitive primitive = was.getAsJsonPrimitive();
				if (primitive.isBoolean()) {
					return text.equals("true") || text.equals("false") ? new JsonPrimitive(Boolean.parseBoolean(text)) : null;
				}
				if (primitive.isNumber()) {
					String trimmed = text.trim();
					boolean whole = !primitive.toString().contains(".") && !primitive.toString().contains("e");
					return whole ? new JsonPrimitive(Long.parseLong(trimmed)) : new JsonPrimitive(Double.parseDouble(trimmed));
				}
				return new JsonPrimitive(text);
			}
			JsonElement parsed = JsonParser.parseString(text);
			return parsed.isJsonArray() == was.isJsonArray() ? parsed : null;
		} catch (RuntimeException e) {
			return null;
		}
	}
}
