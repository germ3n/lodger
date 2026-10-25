package craftkit.host;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The loading steps that need no real library: a content jar is a map of files here. Loading a
 * real native through FFM is the dev-client item of the todo, since it needs the platform's
 * library built and a JVM allowed native access.
 */
class LoadedModTest {
	private record FakeJar(String id, Map<String, byte[]> files) implements Platform.ContentJar {
		@Override
		public Optional<byte[]> read(String resource) {
			return Optional.ofNullable(files.get(resource));
		}
	}

	private static byte[] utf8(String text) {
		return text.getBytes(StandardCharsets.UTF_8);
	}

	@Test
	void a_bad_manifest_is_a_failure_naming_the_manifest(@TempDir Path config) {
		LoadedMod mod = LoadedMod.load(new FakeJar("x", Map.of()), utf8("{ nope"), config, "Linux", "amd64");

		assertFalse(mod.loaded());
		assertTrue(mod.failure().startsWith("bad " + Manifest.PATH), mod.failure());
		assertEquals("x", mod.id(), "with no manifest, the jar's id names the failure");
	}

	@Test
	void a_jar_without_a_native_for_this_platform_says_which_file_it_looked_for(@TempDir Path config) {
		byte[] manifest = utf8("{ \"id\": \"demo\", \"library\": \"demo_core\" }");
		LoadedMod mod = LoadedMod.load(new FakeJar("demo", Map.of()), manifest, config, "Linux", "amd64");

		assertFalse(mod.loaded());
		assertTrue(mod.failure().contains("no native for linux-x86_64"), mod.failure());
		assertTrue(mod.failure().contains("natives/linux-x86_64/libdemo_core.so"), mod.failure());
		assertEquals("demo", mod.id());
	}

	@Test
	void a_file_that_is_not_a_library_fails_at_loading_and_leaves_the_extracted_file(@TempDir Path config) throws IOException {
		Map<String, byte[]> files = new HashMap<>();
		files.put("natives/" + Natives.platform() + "/" + Natives.fileName(Natives.os(System.getProperty("os.name")), "demo_core"), new byte[] {1, 2, 3});
		byte[] manifest = utf8("{ \"id\": \"demo\", \"library\": \"demo_core\" }");

		LoadedMod mod = LoadedMod.load(new FakeJar("demo", files), manifest, config, System.getProperty("os.name"), System.getProperty("os.arch"));

		assertFalse(mod.loaded());
		assertTrue(mod.failure().startsWith("could not load"), mod.failure());
		assertTrue(Files.exists(config.resolve("craftkit/natives/demo").resolve(Natives.platform())), "the native was extracted before loading failed");
	}
}
