package craftkit.host;

import java.nio.file.Path;

/** The repository's root, for tests that read docs/, sdk/ or mod/; build-logic passes it in. */
final class Repo {
	private Repo() {
	}

	/**
	 * Skips a test about the lodger (its jar, its manifest, its docs and CI) where the lodger is
	 * not beside the host: in the craftkit repository exported from this one
	 * (`scripts/export_craftkit.py`), which holds the SDK and the host alone. Here it always is.
	 */
	static void assumeLodger() {
		org.junit.jupiter.api.Assumptions.assumeTrue(java.nio.file.Files.isDirectory(root().resolve("core")) && java.nio.file.Files.isDirectory(root().resolve("mod")),
				"the lodger is not beside the host here");
	}

	static Path root() {
		String property = System.getProperty("craftkit.repo");

		if (property == null) {
			return Path.of("..", "..").toAbsolutePath().normalize();
		}

		return Path.of(property);
	}
}
