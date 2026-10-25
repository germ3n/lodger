package craftkit.host;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import java.util.ServiceLoader;

/**
 * What the host needs from a loader. Fabric and NeoForge each provide one through
 * {@code META-INF/services/craftkit.host.Platform}; everything else in the host is loader-blind.
 */
public interface Platform {
	/** {@code fabric} or {@code neoforge}, for the log and the status line. */
	String name();

	/** The game's config directory, where {@code <mod>.json} files and {@code craftkit/natives/} live. */
	Path configDir();

	/** Every loaded jar, so the host can find the ones that carry a Rust mod. */
	List<ContentJar> jars();

	/**
	 * A loaded jar as the host reads it: its id and a way to a file inside it. Bytes, not a path,
	 * because the loaders differ in how a jar's contents are reachable (Fabric hands out a
	 * {@code Path} into the jar's file system, NeoForge a {@code JarContents}) and the host only
	 * ever reads a file whole: a manifest of a hundred bytes or a native it copies out anyway.
	 */
	interface ContentJar {
		String id();

		/** The bytes of a resource inside the jar, or empty when the jar has no such file. */
		Optional<byte[]> read(String resource) throws IOException;
	}

	static Platform load() {
		return ServiceLoader.load(Platform.class).findFirst()
				.orElseThrow(() -> new IllegalStateException("no craftkit.host.Platform service on the classpath"));
	}
}
