package craftkit.host;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class AcceptanceTest {
	private static Resource licensed() {
		return new Resource("model", "t", "https://x/m", Path.of("/nowhere"), 0, new byte[0], "", true, new Resource.License("Apache License 2.0", "https://x/LICENSE", "terms"));
	}

	private static Resource free() {
		return new Resource("free", "t", "https://x/f", Path.of("/nowhere"), 0, new byte[0], "", true, Resource.License.NONE);
	}

	@Test
	void a_resource_without_terms_needs_no_acceptance_and_one_with_terms_does(@TempDir Path dir) {
		Acceptance acceptance = new Acceptance(dir.resolve("accepted.json"));

		assertTrue(acceptance.accepted("lodger", free()));
		assertFalse(acceptance.accepted("lodger", licensed()));
	}

	@Test
	void an_acceptance_is_written_and_read_back_by_mod_and_resource(@TempDir Path dir) throws IOException {
		Path file = dir.resolve("craftkit").resolve("accepted.json");
		new Acceptance(file).accept("lodger", licensed());

		assertTrue(Files.readString(file).contains("\"lodger:model\""));
		Acceptance again = new Acceptance(file);
		assertTrue(again.accepted("lodger", licensed()));
		assertFalse(again.accepted("other", licensed()), "another mod's same resource id is not accepted");
	}

	@Test
	void a_broken_file_counts_nothing_as_accepted(@TempDir Path dir) throws IOException {
		Path file = dir.resolve("accepted.json");
		Files.writeString(file, "{ nope");

		assertFalse(new Acceptance(file).accepted("lodger", licensed()));
	}
}
