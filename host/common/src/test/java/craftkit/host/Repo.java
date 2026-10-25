package craftkit.host;

import java.nio.file.Path;

/** The repository's root, for tests that read docs/, sdk/ or mod/; build-logic passes it in. */
final class Repo {
	private Repo() {
	}

	static Path root() {
		String property = System.getProperty("craftkit.repo");

		if (property == null) {
			return Path.of("..", "..").toAbsolutePath().normalize();
		}

		return Path.of(property);
	}
}
