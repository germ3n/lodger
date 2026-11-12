package craftkit.host.client.debug;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

/**
 * Whether the debug panel pauses the game while it is open (owner: "pauses the game when its
 * open, should be toggleable"): off at first, as the panel was made to watch the world go on
 * beside it; toggled from the panel's header and kept in {@code config/craftkit/debug.properties}.
 * The game pauses a singleplayer world alone (not one opened to LAN), as its own pause does.
 */
public final class DebugPause {
	private static final String KEY = "pause_when_open";
	private static Boolean on;

	private DebugPause() {
	}

	private static Path file() {
		return net.minecraft.client.Minecraft.getInstance().gameDirectory.toPath().resolve("config").resolve("craftkit").resolve("debug.properties");
	}

	/** Whether the open panel pauses the game. */
	public static boolean on() {
		if (on == null) {
			on = read(file());
		}
		return on;
	}

	/** The other way, and kept. */
	public static void toggle() {
		on = !on();
		write(file(), on);
	}

	static boolean read(Path file) {
		Properties properties = new Properties();
		try (Reader reader = Files.newBufferedReader(file)) {
			properties.load(reader);
		} catch (IOException missing) {
			return false;
		}
		return Boolean.parseBoolean(properties.getProperty(KEY, "false"));
	}

	static void write(Path file, boolean value) {
		Properties properties = new Properties();
		properties.setProperty(KEY, Boolean.toString(value));
		try {
			Files.createDirectories(file.getParent());
			try (Writer writer = Files.newBufferedWriter(file)) {
				properties.store(writer, "The debug panel");
			}
		} catch (IOException unwritable) {
			// Kept for this session alone.
		}
	}
}
