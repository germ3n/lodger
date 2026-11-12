package craftkit.host;

import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The versions pinned in one place agree with the others, as ~/chainv4's tests/toolchains.rs
 * does for its toolchains: a version moved in gradle.properties and not in the docs fails here.
 */
class VersionsTest {
	private static Properties properties(String path) throws IOException {
		Properties properties = new Properties();
		properties.load(Files.newBufferedReader(Repo.root().resolve(path)));

		return properties;
	}

	private static String group(String text, String regex) {
		Matcher matcher = Pattern.compile(regex).matcher(text);
		assertTrue(matcher.find(), regex);

		return matcher.group(1);
	}

	@Test
	void the_host_s_abi_version_is_the_sdk_s() throws IOException {
		String sdk = Files.readString(Repo.root().resolve("sdk/src/lib.rs"));

		assertEquals(Integer.parseInt(group(sdk, "pub const ABI_VERSION: u32 = (\\d+);")), Native.ABI_VERSION);
	}

	@Test
	void the_host_and_the_sdk_carry_one_version() throws IOException {
		String sdk = Files.readString(Repo.root().resolve("sdk/Cargo.toml"));

		assertEquals(properties("host/gradle.properties").getProperty("version"), group(sdk, "(?m)^version = \"([^\"]+)\""));
	}

	@Test
	void the_lodger_s_jar_and_its_mind_carry_one_version() throws IOException {
		Repo.assumeLodger();
		String core = Files.readString(Repo.root().resolve("core/Cargo.toml"));

		assertEquals(properties("mod/gradle.properties").getProperty("version"), group(core, "(?m)^version = \"([^\"]+)\""));
	}

	@Test
	void the_lodger_s_jar_builds_for_the_minecraft_the_host_does() throws IOException {
		Repo.assumeLodger();
		Properties host = properties("host/gradle.properties");
		Properties mod = properties("mod/gradle.properties");

		for (String key : new String[] {"minecraft_version", "minecraft_version_range", "fabric_loader_version", "neoforge_loader_version_range"}) {
			assertEquals(host.getProperty(key), mod.getProperty(key), key);
		}
	}

	@Test
	void the_docs_name_the_versions_gradle_builds_with() throws IOException {
		Repo.assumeLodger();
		Properties host = properties("host/gradle.properties");
		String development = Files.readString(Repo.root().resolve("docs/development.md"));
		String readme = Files.readString(Repo.root().resolve("README.md"));

		assertTrue(readme.contains("Minecraft Java " + host.getProperty("minecraft_version")), "the README names Minecraft " + host.getProperty("minecraft_version"));

		for (String key : new String[] {"minecraft_version", "fabric_loader_version", "fabric_version", "loom_version", "neoforge_version", "moddev_version", "neo_form_version"}) {
			assertTrue(development.contains(host.getProperty(key)), "development.md names " + key + " " + host.getProperty(key));
		}

		String wrapper = Files.readString(Repo.root().resolve("host/gradle/wrapper/gradle-wrapper.properties"));
		assertTrue(development.contains(group(wrapper, "gradle-([0-9.]+)-bin.zip")), "development.md names the Gradle version");
	}

	@Test
	void the_ci_uses_the_same_rust_as_the_toolchain_file_and_java_25() throws IOException {
		Repo.assumeLodger();
		Path root = Repo.root();
		String toolchain = group(Files.readString(root.resolve("rust-toolchain.toml")), "channel = \"([^\"]+)\"");
		String ci = Files.readString(root.resolve(".github/workflows/ci.yml"));

		assertTrue(ci.contains("toolchain: " + toolchain), "ci.yml installs Rust " + toolchain);
		assertTrue(ci.contains("java-version: '25'"), "ci.yml installs Java 25");
	}
}
