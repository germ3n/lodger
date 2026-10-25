package craftkit.host;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.google.gson.JsonSyntaxException;

import java.util.regex.Pattern;

/**
 * {@code META-INF/craftkit.json} in a content jar: what makes a jar a Rust mod.
 *
 * <pre>{ "id": "lodger", "library": "lodger_core", "config": "lodger.json" }</pre>
 *
 * <ul>
 * <li>{@code id}: the mod id, the same one the jar's {@code fabric.mod.json} and
 * {@code neoforge.mods.toml} name; lowercase letters, digits and underscores.</li>
 * <li>{@code library}: the cargo library name, so the file is {@code lib<library>.dylib},
 * {@code lib<library>.so} or {@code <library>.dll} under {@code natives/<os>-<arch>/}.</li>
 * <li>{@code config}: the file under the config directory the mod's config text is read from and
 * written back to. Optional; defaults to {@code <id>.json}.</li>
 * </ul>
 */
public record Manifest(String id, String library, String config) {
	public static final String PATH = "META-INF/craftkit.json";
	private static final Pattern ID = Pattern.compile("[a-z][a-z0-9_]{1,63}");
	private static final Pattern LIBRARY = Pattern.compile("[A-Za-z0-9_]{1,64}");

	public static Manifest parse(String json) {
		JsonObject object;
		try {
			object = JsonParser.parseString(json).getAsJsonObject();
		} catch (JsonSyntaxException | IllegalStateException e) {
			throw new IllegalArgumentException(PATH + " is not a JSON object: " + e.getMessage());
		}

		String id = required(object, "id");
		if (!ID.matcher(id).matches()) {
			throw new IllegalArgumentException(PATH + ": id must be lowercase letters, digits and underscores, got " + id);
		}

		String library = required(object, "library");
		if (!LIBRARY.matcher(library).matches()) {
			throw new IllegalArgumentException(PATH + ": library must be a cargo library name, got " + library);
		}

		String config = object.has("config") ? object.get("config").getAsString() : id + ".json";
		if (config.contains("/") || config.contains("\\") || config.contains("..")) {
			throw new IllegalArgumentException(PATH + ": config must be a file name, got " + config);
		}

		return new Manifest(id, library, config);
	}

	private static String required(JsonObject object, String key) {
		if (!object.has(key) || !object.get(key).isJsonPrimitive()) {
			throw new IllegalArgumentException(PATH + " has no " + key);
		}

		return object.get(key).getAsString();
	}
}
