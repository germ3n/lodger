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
 * cargo names a cdylib. A shared library cannot be loaded from inside a jar, so the right one is
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

	/** The six platforms a content jar carries a native for (the CI's matrix). */
	public static final java.util.List<String> PLATFORMS = java.util.List.of(
			"windows-x86_64", "windows-aarch64", "linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64");

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

	public static String sha256(byte[] bytes) {
		try {
			return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
		} catch (NoSuchAlgorithmException e) {
			throw new IllegalStateException("SHA-256 is always there", e);
		}
	}
}
