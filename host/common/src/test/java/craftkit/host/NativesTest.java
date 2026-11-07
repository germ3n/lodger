package craftkit.host;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class NativesTest {
	@Test
	void platforms_are_named_as_a_content_jar_lays_them_out() {
		assertEquals("macos-aarch64", Natives.platform("Mac OS X", "aarch64"));
		assertEquals("macos-x86_64", Natives.platform("Mac OS X", "x86_64"));
		assertEquals("linux-x86_64", Natives.platform("Linux", "amd64"));
		assertEquals("linux-aarch64", Natives.platform("Linux", "aarch64"));
		assertEquals("windows-x86_64", Natives.platform("Windows 11", "amd64"));
		assertEquals("windows-aarch64", Natives.platform("Windows 11", "aarch64"));
		// Not one a jar carries: named as it is, so the player is told, never mistaken for x86_64.
		assertEquals("linux-riscv64", Natives.platform("Linux", "riscv64"));
		assertEquals("windows-x86", Natives.platform("Windows 10", "x86"));
		assertEquals(6, Natives.PLATFORMS.size());
		assertTrue(Natives.PLATFORMS.contains("macos-x86_64"), "Intel Macs too");
	}

	@Test
	void the_library_is_named_as_cargo_names_a_cdylib() {
		assertEquals("natives/macos-aarch64/liblodger_core.dylib", Natives.resourcePath("Mac OS X", "aarch64", "lodger_core"));
		assertEquals("natives/linux-x86_64/liblodger_core.so", Natives.resourcePath("Linux", "amd64", "lodger_core"));
		assertEquals("natives/windows-x86_64/lodger_core.dll", Natives.resourcePath("Windows 10", "amd64", "lodger_core"));
	}

	@Test
	void a_compressed_native_is_unpacked_and_a_plain_one_left_as_it_is(@TempDir Path dir) throws IOException {
		byte[] plain = new byte[100_000];
		for (int i = 0; i < plain.length; i++) {
			plain[i] = (byte) (i % 7);
		}
		java.io.ByteArrayOutputStream packed = new java.io.ByteArrayOutputStream();
		try (var out = new org.tukaani.xz.XZOutputStream(packed, new org.tukaani.xz.LZMA2Options())) {
			out.write(plain);
		}

		assertArrayEquals(plain, Natives.unpacked(packed.toByteArray()));
		assertArrayEquals(plain, Natives.unpacked(plain), "no xz magic: as it is");
		Path target = Natives.extract(packed.toByteArray(), dir.resolve("natives"), "Linux", "amd64", "demo");
		assertArrayEquals(plain, Files.readAllBytes(target), "written unpacked");
	}

	@Test
	void a_native_is_extracted_once_and_rewritten_only_when_its_bytes_change(@TempDir Path dir) throws IOException {
		Path target = Natives.extract(new byte[] {1, 2, 3}, dir.resolve("natives"), "Linux", "amd64", "demo");

		assertEquals(dir.resolve("natives/linux-x86_64/libdemo.so"), target);
		assertArrayEquals(new byte[] {1, 2, 3}, Files.readAllBytes(target));
		assertFalse(Files.exists(target.resolveSibling("libdemo.so.part")));

		var before = Files.getLastModifiedTime(target);
		Natives.extract(new byte[] {1, 2, 3}, dir.resolve("natives"), "Linux", "amd64", "demo");
		assertEquals(before, Files.getLastModifiedTime(target), "the same bytes are not rewritten");

		Natives.extract(new byte[] {9}, dir.resolve("natives"), "Linux", "amd64", "demo");
		assertArrayEquals(new byte[] {9}, Files.readAllBytes(target));
	}
}
