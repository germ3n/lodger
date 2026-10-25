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

	private LoadedMod(String jarId, Optional<Manifest> manifest, Optional<Native> library, String failure, String name, String version, String sdk, List<String> unknownKeys, boolean wroteConfig) {
		this.jarId = jarId;
		this.manifest = manifest;
		this.library = library;
		this.failure = failure;
		this.name = name;
		this.version = version;
		this.sdk = sdk;
		this.unknownKeys = unknownKeys;
		this.wroteConfig = wroteConfig;
	}

	/** Loads the Rust mod a jar carries. Never throws: a failure is a value. */
	public static LoadedMod load(Platform.ContentJar jar, byte[] manifestBytes, Path configDir, String osName, String osArch) {
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
			return failed(jar.id(), Optional.of(manifest), "no native for " + Natives.platform(osName, osArch) + " in the jar (" + resource + ")");
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

		JsonObject reply = JsonParser.parseString(library.init(text)).getAsJsonObject();
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

		return new LoadedMod(jar.id(), Optional.of(manifest), Optional.of(library), "",
				info.get("name").getAsString(), info.get("version").getAsString(), info.get("sdk").getAsString(),
				List.copyOf(unknown), wrote);
	}

	private static LoadedMod failed(String jarId, Optional<Manifest> manifest, String why) {
		return new LoadedMod(jarId, manifest, Optional.empty(), why, "", "", "", List.of(), false);
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
