package craftkit.host;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * A mod's config file on disk. The mod owns its schema and defaults; the host only reads the
 * text, hands it over, and writes the effective config back when the file was missing, so a
 * player finds every key with its default after the first run and the host never rewrites a
 * player's edits.
 */
public final class ConfigFile {
	private final Path path;

	public ConfigFile(Path path) {
		this.path = path;
	}

	public Path path() {
		return path;
	}

	/** The file's text, or the empty string when there is none, which a mod reads as its defaults. */
	public String read() throws IOException {
		if (!Files.exists(path)) {
			return "";
		}

		return Files.readString(path, StandardCharsets.UTF_8);
	}

	public boolean exists() {
		return Files.exists(path);
	}

	/**
	 * Merges a change into the file: objects key by key, anything else replaced, the rest of
	 * the file as it was. The preparing screen's model choice writes `{"model":{"size":"3b"}}`.
	 */
	public void merge(com.google.gson.JsonObject change) throws IOException {
		String text = read();
		com.google.gson.JsonObject config = text.isBlank() ? new com.google.gson.JsonObject() : com.google.gson.JsonParser.parseString(text).getAsJsonObject();
		merge(config, change);
		Files.createDirectories(path.getParent());
		Files.writeString(path, new com.google.gson.GsonBuilder().setPrettyPrinting().create().toJson(config) + System.lineSeparator(), StandardCharsets.UTF_8);
	}

	/** Takes keys out of the file, each by its path's parts; a section left empty goes too. */
	public void remove(java.util.List<java.util.List<String>> paths) throws IOException {
		if (paths.isEmpty() || !exists()) {
			return;
		}
		String text = read();
		com.google.gson.JsonObject config = text.isBlank() ? new com.google.gson.JsonObject() : com.google.gson.JsonParser.parseString(text).getAsJsonObject();
		for (java.util.List<String> key : paths) {
			remove(config, key);
		}
		Files.writeString(path, new com.google.gson.GsonBuilder().setPrettyPrinting().create().toJson(config) + System.lineSeparator(), StandardCharsets.UTF_8);
	}

	static void remove(com.google.gson.JsonObject config, java.util.List<String> path) {
		if (path.isEmpty()) {
			return;
		}
		if (path.size() == 1) {
			config.remove(path.get(0));
			return;
		}
		if (config.get(path.get(0)) instanceof com.google.gson.JsonObject inner) {
			remove(inner, path.subList(1, path.size()));
			if (inner.isEmpty()) {
				config.remove(path.get(0));
			}
		}
	}

	/** A change merged into another, for gathering several before one write. */
	public static void mergeInto(com.google.gson.JsonObject into, com.google.gson.JsonObject change) {
		merge(into, change);
	}

	static void merge(com.google.gson.JsonObject into, com.google.gson.JsonObject change) {
		for (String key : change.keySet()) {
			com.google.gson.JsonElement value = change.get(key);
			if (value.isJsonObject() && into.has(key) && into.get(key).isJsonObject()) {
				merge(into.getAsJsonObject(key), value.getAsJsonObject());
			} else {
				into.add(key, value.deepCopy());
			}
		}
	}

	/** Writes the effective config over the file: the mod asked, its preset changed (ABI 45). */
	public void write(String effectiveJson) throws IOException {
		Files.createDirectories(path.getParent());
		Files.writeString(path, effectiveJson + System.lineSeparator(), StandardCharsets.UTF_8);
	}

	/** Writes the effective config, only when no file is there. */
	public boolean writeIfMissing(String effectiveJson) throws IOException {
		if (Files.exists(path)) {
			return false;
		}

		Files.createDirectories(path.getParent());
		Files.writeString(path, effectiveJson + System.lineSeparator(), StandardCharsets.UTF_8);

		return true;
	}
}
