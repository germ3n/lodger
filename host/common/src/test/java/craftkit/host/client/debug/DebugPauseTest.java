package craftkit.host.client.debug;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DebugPauseTest {
	@Test
	void off_at_first_and_kept_once_toggled(@TempDir Path dir) {
		Path file = dir.resolve("config").resolve("craftkit").resolve("debug.properties");
		assertFalse(DebugPause.read(file), "off with no file");
		DebugPause.write(file, true);
		assertTrue(DebugPause.read(file));
		DebugPause.write(file, false);
		assertFalse(DebugPause.read(file));
	}
}
