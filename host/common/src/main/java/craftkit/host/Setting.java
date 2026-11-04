package craftkit.host;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.ArrayList;
import java.util.List;

/**
 * One setting a mod offers on the settings screen (`InitReply.settings`): a config key (dotted
 * for a nested one), its label and group, how it changes (`choice`, `toggle`, `number`) and its
 * value now. Pure, so JUnit checks it.
 */
public record Setting(String key, String label, String group, String kind, List<String> options, double min, double max, double step, JsonElement value) {
	public static Setting parse(JsonObject json) {
		List<String> options = new ArrayList<>();
		if (json.has("options") && json.get("options").isJsonArray()) {
			json.getAsJsonArray("options").forEach(o -> options.add(o.getAsString()));
		}
		return new Setting(
				json.get("key").getAsString(),
				json.has("label") ? json.get("label").getAsString() : json.get("key").getAsString(),
				json.has("group") ? json.get("group").getAsString() : "",
				json.has("kind") ? json.get("kind").getAsString() : "choice",
				List.copyOf(options),
				json.has("min") ? json.get("min").getAsDouble() : 0,
				json.has("max") ? json.get("max").getAsDouble() : 0,
				json.has("step") ? json.get("step").getAsDouble() : 1,
				json.has("value") ? json.get("value") : com.google.gson.JsonNull.INSTANCE);
	}

	/** The value after one press: the next option, the other state, or a step up (`up`) or down, kept in range. */
	public JsonElement next(JsonElement now, boolean up) {
		// A value of the wrong shape, or a choice with nothing to choose, stays as it is: a
		// press never throws on the render thread (until 2026-11-10 it could).
		if (now == null || !now.isJsonPrimitive()
				|| (kind.equals("toggle") && !now.getAsJsonPrimitive().isBoolean())
				|| (kind.equals("number") && (!now.getAsJsonPrimitive().isNumber() || step <= 0))
				|| (!kind.equals("toggle") && !kind.equals("number") && options.isEmpty())) {
			return now == null ? com.google.gson.JsonNull.INSTANCE : now;
		}
		return switch (kind) {
			case "toggle" -> new com.google.gson.JsonPrimitive(!now.getAsBoolean());
			case "number" -> {
				double stepped = now.getAsDouble() + (up ? step : -step);
				double kept = Math.max(min, Math.min(max, Math.round(stepped / step) * step));
				// A whole step keeps the number whole: `2`, never `2.0`, which a mind reading a
				// count refuses, and the mod would not load (the dev client, 2026-11-09).
				yield step == Math.rint(step)
						? new com.google.gson.JsonPrimitive(Math.round(kept))
						: new com.google.gson.JsonPrimitive(Math.round(kept * 1000) / 1000.0);
			}
			default -> {
				int at = options.indexOf(now.getAsString());
				int n = options.size();
				yield new com.google.gson.JsonPrimitive(options.get(((at + (up ? 1 : -1)) % n + n) % n));
			}
		};
	}

	/** A change to the config setting this key to `value`, nested by its dots: `{"model":{"size":"3b"}}`. */
	public JsonObject change(JsonElement value) {
		JsonObject root = new JsonObject();
		JsonObject at = root;
		String[] parts = key.split("\\.");
		for (int i = 0; i < parts.length - 1; i++) {
			JsonObject next = new JsonObject();
			at.add(parts[i], next);
			at = next;
		}
		at.add(parts[parts.length - 1], value);

		return root;
	}

	/** The value as the screen shows it. */
	public static String show(JsonElement value) {
		if (value.isJsonPrimitive() && value.getAsJsonPrimitive().isBoolean()) {
			return value.getAsBoolean() ? "on" : "off";
		}

		return value.isJsonPrimitive() ? value.getAsString() : value.toString();
	}
}
