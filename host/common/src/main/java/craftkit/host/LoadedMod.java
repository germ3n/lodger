package craftkit.host;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * One Rust mod as the host sees it: loaded, with what it said at startup, or not, with why.
 *
 * <p>Loading goes: read the manifest, find and extract this platform's native, bind it, check the
 * ABI version, ask its info, hand it its config, and keep the answer. Any step failing leaves
 * {@link #failure()} set and the jar's assets still load as any resource pack would.
 */
public final class LoadedMod {
	private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

	private final String jarId;
	private final Optional<Manifest> manifest;
	private final Optional<Native> library;
	private final String failure;
	private final String name;
	private final String version;
	private final String sdk;
	private final List<String> unknownKeys;
	private final boolean wroteConfig;
	private volatile List<Resource> resources;
	/** Config values of the wrong type the mod mended or left at their defaults, in words. */
	private volatile List<String> warnings = List.of();
	/** The settings the mod offers on the settings screen. */
	private volatile List<Setting> settings = List.of();
	/** Where the mod was loaded from and reads its config, for {@link #reconfigure}. */
	private Path gameDir;
	private Path configDir;

	private LoadedMod(String jarId, Optional<Manifest> manifest, Optional<Native> library, String failure, String name, String version, String sdk, List<String> unknownKeys, boolean wroteConfig, List<Resource> resources) {
		this.jarId = jarId;
		this.manifest = manifest;
		this.library = library;
		this.failure = failure;
		this.name = name;
		this.version = version;
		this.sdk = sdk;
		this.unknownKeys = unknownKeys;
		this.wroteConfig = wroteConfig;
		this.resources = resources;
	}

	/** Loads the Rust mod a jar carries. Never throws: a failure is a value. */
	public static LoadedMod load(Platform.ContentJar jar, byte[] manifestBytes, Path gameDir, Path configDir, String osName, String osArch) {
		Manifest manifest;
		try {
			manifest = Manifest.parse(new String(manifestBytes, StandardCharsets.UTF_8));
		} catch (IllegalArgumentException e) {
			return failed(jar.id(), Optional.empty(), "bad " + Manifest.PATH + ": " + e.getMessage());
		}

		String resource = Natives.resourcePath(osName, osArch, manifest.library());
		Optional<byte[]> inJar;
		try {
			inJar = jar.read(resource);
		} catch (IOException e) {
			return failed(jar.id(), Optional.of(manifest), "could not read " + resource + ": " + e.getMessage());
		}
		if (inJar.isEmpty()) {
			// Which platforms it does carry, so the player knows what to look for.
			List<String> carried = new ArrayList<>();
			for (String platform : Natives.PLATFORMS) {
				String[] p = platform.split("-", 2);
				try {
					if (jar.read("natives/" + platform + "/" + Natives.fileName(p[0], manifest.library())).isPresent()) {
						carried.add(platform);
					}
				} catch (IOException ignored) {
					// Not readable: not counted.
				}
			}
			return failed(jar.id(), Optional.of(manifest), "this jar has no native library for this computer (" + Natives.platform(osName, osArch) + ", " + resource + "); it carries "
					+ (carried.isEmpty() ? "none" : String.join(", ", carried)));
		}

		Path extracted;
		try {
			extracted = Natives.extract(inJar.get(), configDir.resolve("craftkit").resolve("natives").resolve(manifest.id()), osName, osArch, manifest.library());
		} catch (IOException e) {
			return failed(jar.id(), Optional.of(manifest), "could not extract the native: " + e.getMessage());
		}

		Native library;
		try {
			library = Native.load(extracted);
		} catch (RuntimeException e) {
			return failed(jar.id(), Optional.of(manifest), "could not load " + extracted + ": " + e.getMessage());
		}

		int abi = library.abiVersion();
		if (abi != Native.ABI_VERSION) {
			library.close();
			return failed(jar.id(), Optional.of(manifest), "the library speaks ABI " + abi + ", this host ABI " + Native.ABI_VERSION + "; rebuild the mod against the matching craftkit");
		}

		JsonObject info = JsonParser.parseString(library.info()).getAsJsonObject();
		if (info.has("error")) {
			library.close();
			return failed(jar.id(), Optional.of(manifest), "info failed: " + info.get("error").getAsString());
		}

		if (!manifest.id().equals(info.get("id").getAsString())) {
			library.close();
			return failed(jar.id(), Optional.of(manifest), "the manifest says " + manifest.id() + " but the library says " + info.get("id").getAsString());
		}

		ConfigFile config = new ConfigFile(configDir.resolve(manifest.config()));
		String text;
		try {
			text = config.read();
		} catch (IOException e) {
			library.close();
			return failed(jar.id(), Optional.of(manifest), "could not read " + config.path() + ": " + e.getMessage());
		}

		JsonObject init = new JsonObject();
		init.addProperty("config", text);
		init.addProperty("game_dir", gameDir.toAbsolutePath().toString());
		init.addProperty("config_dir", configDir.toAbsolutePath().toString());
		init.addProperty("id", manifest.id());
		JsonObject reply = JsonParser.parseString(library.init(init.toString())).getAsJsonObject();
		if (reply.has("error")) {
			library.close();
			return failed(jar.id(), Optional.of(manifest), "init refused: " + reply.get("error").getAsString());
		}

		List<String> unknown = new ArrayList<>();
		if (reply.has("unknown")) {
			for (JsonElement key : reply.getAsJsonArray("unknown")) {
				unknown.add(key.getAsString());
			}
		}

		boolean wrote = false;
		try {
			wrote = config.writeIfMissing(GSON.toJson(reply.get("config")));
		} catch (IOException e) {
			// Not fatal: the mod runs on what it read; the player just has no file to edit yet.
			unknown.add("(could not write " + config.path() + ": " + e.getMessage() + ")");
		}

		List<Resource> resources = new ArrayList<>();
		if (reply.has("resources")) {
			for (JsonElement element : reply.getAsJsonArray("resources")) {
				try {
					resources.add(Resource.parse(element.getAsJsonObject(), gameDir));
				} catch (IllegalArgumentException e) {
					library.close();
					return failed(jar.id(), Optional.of(manifest), "bad resource: " + e.getMessage());
				}
			}
		}

		LoadedMod made = new LoadedMod(jar.id(), Optional.of(manifest), Optional.of(library), "",
				info.get("name").getAsString(), info.get("version").getAsString(), info.get("sdk").getAsString(),
				List.copyOf(unknown), wrote, List.copyOf(resources));
		made.gameDir = gameDir;
		made.configDir = configDir;
		made.settings = settingsOf(reply);
		made.warnings = warningsOf(reply);

		return made;
	}

	/**
	 * A change to the mod's config, written to its file, and the mod initialised again with it:
	 * its resources are what the new config asks for. Before any world is open alone (the
	 * preparing screen's model choice). Answers an error, or the empty string.
	 */
	public synchronized String reconfigure(com.google.gson.JsonObject change) {
		if (library.isEmpty() || manifest.isEmpty() || configDir == null) {
			return "the mod is not loaded";
		}
		ConfigFile config = new ConfigFile(configDir.resolve(manifest.get().config()));
		JsonObject reply;
		// What the file said before: put back if the mod refuses the change, so the next launch
		// is not refused too (until 2026-11-10 the refused change stayed in the file).
		String before;
		try {
			before = config.exists() ? config.read() : null;
		} catch (IOException e) {
			return e.getMessage();
		}
		try {
			config.merge(change);
			JsonObject init = new JsonObject();
			init.addProperty("config", config.read());
			init.addProperty("game_dir", gameDir.toAbsolutePath().toString());
			init.addProperty("config_dir", configDir.toAbsolutePath().toString());
			init.addProperty("id", manifest.get().id());
			// What the file said before the change: a preset picked over a file of every key
			// takes the old preset's values out (ABI 45).
			init.addProperty("config_before", before == null ? "" : before);
			reply = JsonParser.parseString(library.get().init(init.toString())).getAsJsonObject();
			if (reply.has("rewrite") && reply.get("rewrite").getAsBoolean() && reply.has("config")) {
				config.write(GSON.toJson(reply.get("config")));
			}
		} catch (IOException | RuntimeException e) {
			restore(config, before);
			return e.getMessage();
		}
		if (reply.has("error")) {
			restore(config, before);
			return reply.get("error").getAsString();
		}
		List<Resource> fresh = new ArrayList<>();
		try {
			if (reply.has("resources")) {
				for (JsonElement element : reply.getAsJsonArray("resources")) {
					fresh.add(Resource.parse(element.getAsJsonObject(), gameDir));
				}
			}
		} catch (RuntimeException e) {
			return "the mod's resources: " + e.getMessage();
		}
		resources = List.copyOf(fresh);
		settings = settingsOf(reply);
		warnings = warningsOf(reply);
		for (String warning : warnings) {
			Log.LOGGER.warn("{}: config: {}", manifest.get().id(), warning);
		}

		return "";
	}

	/** The config file as it was before a refused change; written back, or left if it cannot be. */
	private static void restore(ConfigFile config, String before) {
		if (before == null) {
			return;
		}
		try {
			java.nio.file.Files.writeString(config.path(), before, StandardCharsets.UTF_8);
		} catch (IOException e) {
			Log.LOGGER.warn("the config could not be put back: {}", e.getMessage());
		}
	}

	/** The reply's config warnings (ABI 22); none from a mod that sends none. */
	static List<String> warningsOf(JsonObject reply) {
		List<String> out = new ArrayList<>();
		if (reply.has("warnings") && reply.get("warnings").isJsonArray()) {
			reply.getAsJsonArray("warnings").forEach(w -> out.add(w.getAsString()));
		}

		return List.copyOf(out);
	}

	/** Config values the mod mended or left at their defaults, for the log. */
	public List<String> warnings() {
		return warnings;
	}

	private static List<Setting> settingsOf(JsonObject reply) {
		List<Setting> out = new ArrayList<>();
		if (reply.has("settings") && reply.get("settings").isJsonArray()) {
			for (JsonElement element : reply.getAsJsonArray("settings")) {
				out.add(Setting.parse(element.getAsJsonObject()));
			}
		}

		return List.copyOf(out);
	}

	/** What the settings screen shows for this mod; none for a mod that offers none. */
	public List<Setting> settings() {
		return settings;
	}

	private static LoadedMod failed(String jarId, Optional<Manifest> manifest, String why) {
		return new LoadedMod(jarId, manifest, Optional.empty(), why, "", "", "", List.of(), false, List.of());
	}

	/** The files the mod needs fetched. */
	public List<Resource> resources() {
		return resources;
	}

	/** Loads what the mod loads before play, blocking. Empty when it loaded; the error otherwise. Call on a worker thread. */
	public String warm() {
		if (library.isEmpty()) {
			return failure;
		}

		return errorOf(library.get().warm());
	}

	/** The mod's own account of whether it is ready, from any thread. */
	public ModStatus status() {
		if (library.isEmpty()) {
			return new ModStatus(false, failure, false, failure);
		}

		return parseStatus(library.get().status());
	}

	/**
	 * Reads a {@code status} answer. A {@code Status} has {@code ready} and carries its own
	 * {@code error} field, empty when nothing is wrong; a failure answer has {@code error} alone.
	 * Reading every {@code error} key as a failure once made a loaded model look unready and
	 * would have kept the title screen gated for good.
	 */
	static ModStatus parseStatus(String raw) {
		JsonObject answer = JsonParser.parseString(raw).getAsJsonObject();

		if (answer.has("ready")) {
			return new ModStatus(
					answer.get("ready").getAsBoolean(),
					answer.has("line") ? answer.get("line").getAsString() : "",
					answer.has("loading") && answer.get("loading").getAsBoolean(),
					answer.has("error") ? answer.get("error").getAsString() : "");
		}

		String why = answer.has("error") ? answer.get("error").getAsString() : "status answered " + raw;

		return new ModStatus(false, why, false, why);
	}

	/** What a mod says of itself: ready to play, one line why or why not. */
	public record ModStatus(boolean ready, String line, boolean loading, String error) {
	}

	/** The error in an answer, or the empty string when the answer is not an error. */
	public static String errorOf(String answer) {
		JsonElement element = JsonParser.parseString(answer);

		if (element.isJsonObject() && element.getAsJsonObject().has("error") && element.getAsJsonObject().get("error").isJsonPrimitive()) {
			return element.getAsJsonObject().get("error").getAsString();
		}

		return "";
	}

	/** The library, for the world loop. Empty when the mod did not load. */
	public Optional<Native> library() {
		return library;
	}

	public String id() {
		return manifest.map(Manifest::id).orElse(jarId);
	}

	public boolean loaded() {
		return library.isPresent();
	}

	/** Why the mod is not loaded, or the empty string when it is. */
	public String failure() {
		return failure;
	}

	public String name() {
		return name;
	}

	public String version() {
		return version;
	}

	public String sdk() {
		return sdk;
	}

	public List<String> unknownKeys() {
		return unknownKeys;
	}

	public boolean wroteConfig() {
		return wroteConfig;
	}

	public Optional<Manifest> manifest() {
		return manifest;
	}

	/** Tells the mod the process is ending and unloads the library. */
	public void close() {
		library.ifPresent(l -> {
			try {
				l.shutdown();
			} finally {
				l.close();
			}
		});
	}
}
