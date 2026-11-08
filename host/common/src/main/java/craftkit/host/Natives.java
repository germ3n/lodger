package craftkit.host;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.Locale;

/**
 * Where a mod's shared library is inside its jar and on disk, for this machine.
 *
 * <p>A content jar carries one native per platform under {@code natives/<os>-<arch>/}, named as
 * cargo names a cdylib, xz-compressed with {@code .xz} after the name in a release build (half
 * the size of the jar's own deflate) or plain in a developer's. A shared library cannot be
 * loaded from inside a jar, so the right one is
 * extracted to {@code config/craftkit/natives/<mod>/<os>-<arch>/} and loaded from there; one
 * already there with the same SHA-256 is left alone, and a new one is written beside and moved
 * into place so a crash mid-write never leaves a half library to load next time.
 */
public final class Natives {
	private Natives() {
	}

	/** {@code macos}, {@code linux} or {@code windows}, from {@code os.name}. */
	public static String os(String osName) {
		String name = osName.toLowerCase(Locale.ROOT);

		if (name.contains("mac") || name.contains("darwin")) {
			return "macos";
		}

		if (name.contains("win")) {
			return "windows";
		}

		return "linux";
	}

	/** {@code aarch64} or {@code x86_64}, from {@code os.arch}. */
	/**
	 * Why this CPU cannot run the native, or empty: every x86_64 native is built for x86-64-v3
	 * (AVX2 and FMA, `.cargo/config.toml`), and an older CPU died on its first instruction with
	 * the JVM, no reason given. Asked of the system itself: Linux's {@code /proc/cpuinfo}, the
	 * Mac's {@code hw.optional.avx2_0}, Windows' {@code IsProcessorFeaturePresent}. Where it
	 * cannot be asked, nothing is said and the load goes on.
	 */
	public static String cpuMissing(String osName, String osArch) {
		if (!arch(osArch).equals("x86_64")) {
			return "";
		}
		Boolean avx2 = switch (os(osName)) {
			case "linux" -> {
				try {
					yield cpuinfoHasAvx2(java.nio.file.Files.readString(java.nio.file.Path.of("/proc/cpuinfo")));
				} catch (java.io.IOException | RuntimeException e) {
					yield null;
				}
			}
			case "macos" -> sysctlFlag("hw.optional.avx2_0");
			default -> windowsFeature(40); // PF_AVX2_INSTRUCTIONS_AVAILABLE
		};
		return Boolean.FALSE.equals(avx2) ? MISSING_AVX2 : "";
	}

	static final String MISSING_AVX2 = "this CPU has no AVX2, which the lodger's native needs (Intel processors since 2013, AMD since 2015)";

	/** Whether {@code /proc/cpuinfo}'s first processor lists both avx2 and fma. */
	static boolean cpuinfoHasAvx2(String cpuinfo) {
		for (String line : cpuinfo.split("\n")) {
			if (line.startsWith("flags")) {
				java.util.Set<String> flags = new java.util.HashSet<>(java.util.Arrays.asList(line.substring(line.indexOf(':') + 1).trim().split("\\s+")));
				return flags.contains("avx2") && flags.contains("fma");
			}
		}
		// No flags line (an emulator, a container hiding it): not known to be missing.
		return true;
	}

	private static Boolean sysctlFlag(String name) {
		try (java.lang.foreign.Arena arena = java.lang.foreign.Arena.ofConfined()) {
			java.lang.foreign.Linker linker = java.lang.foreign.Linker.nativeLinker();
			var found = linker.defaultLookup().find("sysctlbyname");
			if (found.isEmpty()) {
				return null;
			}
			var call = linker.downcallHandle(found.get(), java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_INT,
					java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.ADDRESS,
					java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));
			var value = arena.allocate(java.lang.foreign.ValueLayout.JAVA_INT);
			var size = arena.allocate(java.lang.foreign.ValueLayout.JAVA_LONG);
			size.set(java.lang.foreign.ValueLayout.JAVA_LONG, 0, 4);
			int result = (int) call.invokeExact(arena.allocateFrom(name), value, size, java.lang.foreign.MemorySegment.NULL, 0L);
			// No such name: not known, and the load goes on.
			return result == 0 ? value.get(java.lang.foreign.ValueLayout.JAVA_INT, 0) != 0 : null;
		} catch (Throwable e) {
			return null;
		}
	}

	private static Boolean windowsFeature(int feature) {
		try (java.lang.foreign.Arena arena = java.lang.foreign.Arena.ofConfined()) {
			java.lang.foreign.Linker linker = java.lang.foreign.Linker.nativeLinker();
			var kernel = java.lang.foreign.SymbolLookup.libraryLookup("kernel32", arena);
			var found = kernel.find("IsProcessorFeaturePresent");
			if (found.isEmpty()) {
				return null;
			}
			var call = linker.downcallHandle(found.get(), java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_INT, java.lang.foreign.ValueLayout.JAVA_INT));
			return (int) call.invokeExact(feature) != 0;
		} catch (Throwable e) {
			return null;
		}
	}

	public static String arch(String osArch) {
		String arch = osArch.toLowerCase(Locale.ROOT);

		if (arch.equals("aarch64") || arch.equals("arm64")) {
			return "aarch64";
		}
		if (arch.equals("amd64") || arch.equals("x86_64") || arch.equals("x64")) {
			return "x86_64";
		}

		// Anything else by its own name (x86, riscv64, ppc64le): no jar carries it, and the
		// player is told so, where it once passed for x86_64 and failed to load obscurely.
		return arch;
	}

	/** The six platforms a content jar carries a native for (the CI's natives). */
	public static final java.util.List<String> PLATFORMS = java.util.List.of(
			"windows-x86_64", "windows-aarch64", "linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64");

	/** After a native's name in the jar when it is xz-compressed. */
	public static final String XZ = ".xz";

	/** {@code <os>-<arch>}, the folder name under {@code natives/}. */
	public static String platform(String osName, String osArch) {
		return os(osName) + "-" + arch(osArch);
	}

	public static String platform() {
		return platform(System.getProperty("os.name"), System.getProperty("os.arch"));
	}

	/** The file name cargo gives the cdylib {@code library} on {@code os}. */
	public static String fileName(String os, String library) {
		return switch (os) {
			case "macos" -> "lib" + library + ".dylib";
			case "windows" -> library + ".dll";
			default -> "lib" + library + ".so";
		};
	}

	/** The resource path inside a content jar for this platform's native of {@code library}. */
	public static String resourcePath(String osName, String osArch, String library) {
		return "natives/" + platform(osName, osArch) + "/" + fileName(os(osName), library);
	}

	/**
	 * Writes the native's {@code bytes} (read out of the jar) to {@code targetDir/<os>-<arch>/<file>}
	 * unless the same bytes are already there, and answers the path to load.
	 */
	public static Path extract(byte[] bytes, Path targetDir, String osName, String osArch, String library) throws IOException {
		bytes = unpacked(bytes);
		Path target = targetDir.resolve(platform(osName, osArch)).resolve(fileName(os(osName), library));

		if (Files.exists(target) && sha256(Files.readAllBytes(target)).equals(sha256(bytes))) {
			return target;
		}

		Files.createDirectories(target.getParent());
		Path part = target.resolveSibling(target.getFileName() + ".part");
		Files.write(part, bytes);
		Files.move(part, target, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);

		return target;
	}

	/** The bytes as they are, or unpacked when they begin with xz's magic. */
	public static byte[] unpacked(byte[] bytes) throws IOException {
		if (bytes.length < 6 || (bytes[0] & 0xFF) != 0xFD || bytes[1] != '7' || bytes[2] != 'z' || bytes[3] != 'X' || bytes[4] != 'Z' || bytes[5] != 0) {
			return bytes;
		}
		try (var in = new org.tukaani.xz.XZInputStream(new java.io.ByteArrayInputStream(bytes))) {
			return in.readAllBytes();
		}
	}

	public static String sha256(byte[] bytes) {
		try {
			return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
		} catch (NoSuchAlgorithmException e) {
			throw new IllegalStateException("SHA-256 is always there", e);
		}
	}
}
