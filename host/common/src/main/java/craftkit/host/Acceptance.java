package craftkit.host;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashSet;
import java.util.Set;

/**
 * Which resources' terms have been accepted, in {@code config/craftkit/accepted.json}: a list of
 * {@code <mod id>:<resource id>}. A client's license screen adds to it; a dedicated server's
 * operator adds to it by hand, after the server logs the terms, so no one accepts anything on
 * their behalf. A resource with no license needs no acceptance.
 */
public final class Acceptance {
	private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

	private final Path file;
	private final Set<String> accepted = new LinkedHashSet<>();

	public Acceptance(Path file) {
		this.file = file;
		load();
	}

	public Path file() {
		return file;
	}

	public static String key(String modId, Resource resource) {
		return modId + ":" + resource.id();
	}

	/** Whether the terms of {@code resource} are accepted, or there are none to accept. */
	public boolean accepted(String modId, Resource resource) {
		return !resource.hasLicense() || accepted.contains(key(modId, resource));
	}

	/** Records the acceptance and writes the file. */
	public void accept(String modId, Resource resource) throws IOException {
		accepted.add(key(modId, resource));
		Files.createDirectories(file.getParent());
		JsonObject root = new JsonObject();
		JsonArray list = new JsonArray();
		accepted.forEach(list::add);
		root.add("accepted", list);
		Files.writeString(file, GSON.toJson(root) + System.lineSeparator());
	}

	private void load() {
		if (!Files.exists(file)) {
			return;
		}

		try {
			JsonObject root = JsonParser.parseString(Files.readString(file)).getAsJsonObject();
			if (root.has("accepted")) {
				for (JsonElement element : root.getAsJsonArray("accepted")) {
					accepted.add(element.getAsString());
				}
			}
		} catch (IOException | RuntimeException e) {
			Log.LOGGER.warn("could not read {}: {}; nothing counts as accepted", file, e.getMessage());
		}
	}
}
