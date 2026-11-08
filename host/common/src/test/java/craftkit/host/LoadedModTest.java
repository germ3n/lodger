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
	void a_status_with_an_empty_error_field_is_a_status_not_a_failure() {
		LoadedMod.ModStatus loaded = LoadedMod.parseStatus("{\"ready\":true,\"line\":\"The model is loaded\",\"loading\":false,\"error\":\"\"}");
		assertTrue(loaded.ready());
		assertEquals("The model is loaded", loaded.line());

		LoadedMod.ModStatus failing = LoadedMod.parseStatus("{\"ready\":false,\"line\":\"The model failed to load: bad file\",\"loading\":false,\"error\":\"bad file\"}");
		assertFalse(failing.ready());
		assertEquals("bad file", failing.error());

		LoadedMod.ModStatus refused = LoadedMod.parseStatus("{\"error\":\"the mod is not initialized\"}");
		assertFalse(refused.ready());
		assertEquals("the mod is not initialized", refused.line());

		LoadedMod.ModStatus odd = LoadedMod.parseStatus("{\"ok\":true}");
		assertFalse(odd.ready());
		assertTrue(odd.line().startsWith("status answered"));
	}

	@Test
	void an_ok_answer_has_no_error_and_an_error_answer_has_one() {
		assertEquals("", LoadedMod.errorOf("{\"ok\":true}"));
		assertEquals("nope", LoadedMod.errorOf("{\"error\":\"nope\"}"));
		assertEquals("", LoadedMod.errorOf("[]"));
	}

	@Test
	void a_bad_manifest_is_a_failure_naming_the_manifest(@TempDir Path config) {
		LoadedMod mod = LoadedMod.load(new FakeJar("x", Map.of()), utf8("{ nope"), config.getParent(), config, "Linux", "amd64");

		assertFalse(mod.loaded());
		assertTrue(mod.failure().startsWith("bad " + Manifest.PATH), mod.failure());
		assertEquals("x", mod.id(), "with no manifest, the jar's id names the failure");
	}

	@Test
	void a_jar_without_a_native_for_this_platform_says_which_file_it_looked_for(@TempDir Path config) {
		byte[] manifest = utf8("{ \"id\": \"demo\", \"library\": \"demo_core\" }");
		LoadedMod mod = LoadedMod.load(new FakeJar("demo", Map.of()), manifest, config.getParent(), config, "Linux", "amd64");

		assertFalse(mod.loaded());
		assertTrue(mod.failure().contains("no native library for this computer (linux-x86_64") && mod.failure().contains("it carries none"), mod.failure());
		assertTrue(mod.failure().contains("natives/linux-x86_64/libdemo_core.so"), mod.failure());
		assertEquals("demo", mod.id());
	}

	@Test
	void a_file_that_is_not_a_library_fails_at_loading_and_leaves_the_extracted_file(@TempDir Path config) throws IOException {
		Map<String, byte[]> files = new HashMap<>();
		files.put("natives/" + Natives.platform() + "/" + Natives.fileName(Natives.os(System.getProperty("os.name")), "demo_core"), new byte[] {1, 2, 3});
		byte[] manifest = utf8("{ \"id\": \"demo\", \"library\": \"demo_core\" }");

		LoadedMod mod = LoadedMod.load(new FakeJar("demo", files), manifest, config.getParent(), config, System.getProperty("os.name"), System.getProperty("os.arch"));

		assertFalse(mod.loaded());
		assertTrue(mod.failure().startsWith("could not load"), mod.failure());
		assertTrue(Files.exists(config.resolve("craftkit/natives/demo").resolve(Natives.platform())), "the native was extracted before loading failed");
	}

	@Test
	void a_linux_too_old_for_the_native_says_its_glibc() {
		assertTrue(LoadedMod.glibcHint("Linux", "2.28").contains("glibc 2.28"));
		org.junit.jupiter.api.Assertions.assertEquals("", LoadedMod.glibcHint("Linux", "2.34"));
		org.junit.jupiter.api.Assertions.assertEquals("", LoadedMod.glibcHint("Linux", "2.39"));
		org.junit.jupiter.api.Assertions.assertEquals("", LoadedMod.glibcHint("Windows 11", "2.28"), "not Linux");
		org.junit.jupiter.api.Assertions.assertEquals("", LoadedMod.glibcHint("Linux", ""), "not asked");
	}

	@Test
	void a_native_that_wants_vulkan_says_what_to_install() {
		RuntimeException linux = new RuntimeException("libvulkan.so.1: cannot open shared object file");
		org.junit.jupiter.api.Assertions.assertTrue(LoadedMod.vulkanHint(linux, "Linux", false).contains("apt install libvulkan1"));
		RuntimeException windows = new RuntimeException("Can't find dependent libraries", new RuntimeException("vulkan-1.dll"));
		org.junit.jupiter.api.Assertions.assertTrue(LoadedMod.vulkanHint(windows, "Windows 11", false).contains("graphics driver"));
		org.junit.jupiter.api.Assertions.assertEquals("", LoadedMod.vulkanHint(new RuntimeException("no such file"), "Linux", false));
		// As Java says it: no reason given; the loader tried by itself is missing.
		RuntimeException bare = new IllegalArgumentException("Cannot open library: /x/liblodger_core.so");
		org.junit.jupiter.api.Assertions.assertTrue(LoadedMod.vulkanHint(bare, "Linux", true).contains("apt install libvulkan1"));
		org.junit.jupiter.api.Assertions.assertFalse(LoadedMod.vulkanLoaderMissing("Mac OS X"), "a Mac's natives use Metal");
	}
}
