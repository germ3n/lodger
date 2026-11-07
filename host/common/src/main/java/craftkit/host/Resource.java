package craftkit.host;

import com.google.gson.JsonObject;

import java.nio.file.Path;
import java.util.HexFormat;

/**
 * A file a mod needs the host to fetch, as its {@code init} reply declared it (the SDK's
 * {@code Resource}): the lodger's model, for one. Required resources gate play (see
 * {@link Preparation}); optional ones download in the background.
 */
public record Resource(String id, String title, String url, Path path, long sizeHint, byte[] magic, String sha256, boolean required, License license, java.util.List<Choice> choices,
		JsonObject without) {
	/**
	 * Another form of the resource a player may pick on the preparing screen (the model's sizes):
	 * a label, the change to the mod's config that picks it, and whether it is the one now.
	 */
	public record Choice(String label, com.google.gson.JsonObject config, boolean chosen, String note) {
	}

	public Resource(String id, String title, String url, Path path, long sizeHint, byte[] magic, String sha256, boolean required, License license) {
		this(id, title, url, path, sizeHint, magic, sha256, required, license, java.util.List.of(), null);
	}

	/** The terms a resource comes under, shown before its first download. */
	public record License(String name, String url, String text) {
		public static final License NONE = new License("", "", "");

		public boolean isEmpty() {
			return name.isEmpty() && url.isEmpty() && text.isEmpty();
		}

		static License parse(JsonObject json) {
			if (json == null) {
				return NONE;
			}

			return new License(
					json.has("name") ? json.get("name").getAsString() : "",
					json.has("url") ? json.get("url").getAsString() : "",
					json.has("text") ? json.get("text").getAsString() : "");
		}
	}

	public boolean hasLicense() {
		return !license.isEmpty();
	}

	public static Resource parse(JsonObject json, Path gameDir) {
		String id = required(json, "id");
		String url = required(json, "url");
		if (!(url.startsWith("https://") || url.startsWith("http://"))) {
			throw new IllegalArgumentException(id + ": url must start with http:// or https://");
		}

		Path path = Path.of(required(json, "path"));
		if (!path.isAbsolute()) {
			path = gameDir.resolve(path);
		}

		String sha256 = json.has("sha256") ? json.get("sha256").getAsString() : "";
		if (!sha256.isEmpty()) {
			if (sha256.length() != 64) {
				throw new IllegalArgumentException(id + ": sha256 must be 64 hex characters");
			}
			HexFormat.of().parseHex(sha256);
		}

		return new Resource(
				id,
				json.has("title") ? json.get("title").getAsString() : id,
				url,
				path.toAbsolutePath().normalize(),
				json.has("size_hint") ? json.get("size_hint").getAsLong() : 0,
				json.has("magic") ? json.get("magic").getAsString().getBytes(java.nio.charset.StandardCharsets.UTF_8) : new byte[0],
				sha256.toLowerCase(java.util.Locale.ROOT),
				!json.has("required") || json.get("required").getAsBoolean(),
				License.parse(json.has("license") && json.get("license").isJsonObject() ? json.getAsJsonObject("license") : null),
				choices(json),
				// The config change that plays without it (ABI 46): the screen's Play without it.
				json.has("without") && json.get("without").isJsonObject() ? json.getAsJsonObject("without") : null);
	}

	private static java.util.List<Choice> choices(JsonObject json) {
		java.util.List<Choice> out = new java.util.ArrayList<>();
		if (json.has("choices") && json.get("choices").isJsonArray()) {
			for (com.google.gson.JsonElement element : json.getAsJsonArray("choices")) {
				JsonObject c = element.getAsJsonObject();
				out.add(new Choice(c.get("label").getAsString(), c.has("config") && c.get("config").isJsonObject() ? c.getAsJsonObject("config") : new JsonObject(), c.has("chosen") && c.get("chosen").getAsBoolean(),
						c.has("note") ? c.get("note").getAsString() : ""));
			}
		}

		return java.util.List.copyOf(out);
	}

	private static String required(JsonObject json, String key) {
		if (!json.has(key) || !json.get(key).isJsonPrimitive()) {
			throw new IllegalArgumentException("a resource has no " + key);
		}

		return json.get(key).getAsString();
	}

	/** Whether the file is already there. */
	public boolean present() {
		return java.nio.file.Files.isRegularFile(path);
	}
}
