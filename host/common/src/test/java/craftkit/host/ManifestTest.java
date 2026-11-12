package craftkit.host;

import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ManifestTest {
	@Test
	void a_manifest_names_the_mod_its_library_and_its_config() {
		Manifest manifest = Manifest.parse("{ \"id\": \"lodger\", \"library\": \"lodger_core\", \"config\": \"lodger.json\" }");

		assertEquals(new Manifest("lodger", "lodger_core", "lodger.json"), manifest);
	}

	@Test
	void the_config_file_defaults_to_the_id() {
		assertEquals("tiny.json", Manifest.parse("{ \"id\": \"tiny\", \"library\": \"tiny\" }").config());
	}

	@Test
	void bad_manifests_are_refused_by_name() {
		for (String[] bad : new String[][] {
				{"{ nope", "not a JSON object"},
				{"[]", "not a JSON object"},
				{"{ \"library\": \"x\" }", "has no id"},
				{"{ \"id\": \"Lodger\", \"library\": \"x\" }", "id must be"},
				{"{ \"id\": \"lodger\" }", "has no library"},
				{"{ \"id\": \"lodger\", \"library\": \"../x\" }", "library must be"},
				{"{ \"id\": \"lodger\", \"library\": \"x\", \"config\": \"../x.json\" }", "config must be"},
		}) {
			IllegalArgumentException e = assertThrows(IllegalArgumentException.class, () -> Manifest.parse(bad[0]), bad[0]);
			assertTrue(e.getMessage().contains(bad[1]), bad[0] + ": " + e.getMessage());
		}
	}

	@Test
	void the_lodger_s_own_manifest_parses_and_names_its_crate() throws IOException {
		Repo.assumeLodger();
		String json = Files.readString(Repo.root().resolve("mod/src/main/resources/META-INF/craftkit.json"));
		Manifest manifest = Manifest.parse(json);

		assertEquals("lodger", manifest.id());
		assertEquals("lodger_core", manifest.library());
		assertEquals("lodger.json", manifest.config());

		String cargo = Files.readString(Repo.root().resolve("core/Cargo.toml"));
		assertTrue(cargo.contains("name = \"" + manifest.library() + "\""), "core/Cargo.toml's [lib] name is the manifest's library");
	}
}
