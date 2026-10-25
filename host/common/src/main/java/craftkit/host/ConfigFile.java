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
