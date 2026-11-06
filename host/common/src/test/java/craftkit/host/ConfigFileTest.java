package craftkit.host;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ConfigFileTest {
	@Test
	void a_missing_file_reads_as_empty_which_a_mod_takes_as_its_defaults(@TempDir Path dir) throws IOException {
		ConfigFile file = new ConfigFile(dir.resolve("lodger.json"));

		assertFalse(file.exists());
		assertEquals("", file.read());
	}

	@Test
	void the_effective_config_is_written_only_when_the_file_is_missing(@TempDir Path dir) throws IOException {
		ConfigFile file = new ConfigFile(dir.resolve("config").resolve("lodger.json"));

		assertTrue(file.writeIfMissing("{\"enabled\": true}"));
		assertEquals("{\"enabled\": true}" + System.lineSeparator(), Files.readString(file.path()));

		assertFalse(file.writeIfMissing("{\"enabled\": false}"), "a player's file is never rewritten");
		assertEquals("{\"enabled\": true}" + System.lineSeparator(), Files.readString(file.path()));
	}

	@Test
	void a_mod_that_asks_has_its_file_written_over(@TempDir Path dir) throws IOException {
		ConfigFile file = new ConfigFile(dir.resolve("config").resolve("lodger.json"));
		file.writeIfMissing("{\"preset\": \"balanced\"}");

		file.write("{\"preset\": \"calm\"}");

		assertEquals("{\"preset\": \"calm\"}" + System.lineSeparator(), Files.readString(file.path()));
	}
}
