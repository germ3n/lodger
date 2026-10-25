package craftkit.host;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Duration;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.atomic.AtomicReference;

/**
 * Fetches one {@link Resource}, reporting progress.
 *
 * <p>The file is written under a temporary name and renamed when complete, so a failed download
 * never replaces a good file; the first bytes must match the resource's magic, so an HTML error
 * page fails at once rather than after half a gigabyte; and the SHA-256 is checked when the
 * resource names one. Progress is two atomics the screen and the server log read from any
 * thread. One {@code Downloader} runs one download, on the thread {@link #run} is called from.
 */
public final class Downloader {
	private static final int CHUNK = 1 << 20;
	private static final Duration CONNECT = Duration.ofSeconds(15);

	public enum State {
		PENDING, RUNNING, DONE, FAILED
	}

	private final Resource resource;
	private final AtomicLong bytes = new AtomicLong();
	private final AtomicLong total = new AtomicLong();
	private final AtomicReference<State> state = new AtomicReference<>(State.PENDING);
	private final AtomicReference<String> error = new AtomicReference<>("");
	private volatile boolean cancelled;

	public Downloader(Resource resource) {
		this.resource = resource;
		this.total.set(resource.sizeHint());
	}

	public Resource resource() {
		return resource;
	}

	public State state() {
		return state.get();
	}

	public String error() {
		return error.get();
	}

	public long bytes() {
		return bytes.get();
	}

	public long total() {
		return total.get();
	}

	/** 0 to 1, or 0 while the size is unknown. */
	public double fraction() {
		long t = total.get();

		return t <= 0 ? 0 : Math.min(1.0, (double) bytes.get() / t);
	}

	/** Asks the download to stop at the next chunk; the part file is removed. */
	public void cancel() {
		cancelled = true;
	}

	/** Downloads, blocking. Afterwards {@link #state()} is DONE or FAILED and {@link #error()} says why. */
	public void run() {
		state.set(State.RUNNING);
		Path path = resource.path();
		Path part = path.resolveSibling(path.getFileName() + ".part");

		try {
			Files.createDirectories(path.getParent());

			try (OutputStream out = Files.newOutputStream(part)) {
				fetch(out);
			}

			Files.move(part, path, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
			state.set(State.DONE);
		} catch (IOException | InterruptedException | RuntimeException e) {
			try {
				Files.deleteIfExists(part);
			} catch (IOException ignored) {
				// The part file is harmless; the next run overwrites it.
			}

			error.set(e.getMessage() == null ? e.getClass().getSimpleName() : e.getMessage());
			state.set(State.FAILED);
		}
	}

	private void fetch(OutputStream out) throws IOException, InterruptedException {
		HttpClient client = HttpClient.newBuilder()
				.followRedirects(HttpClient.Redirect.NORMAL)
				.connectTimeout(CONNECT)
				.build();
		HttpRequest request = HttpRequest.newBuilder(URI.create(resource.url())).GET().build();
		HttpResponse<InputStream> response = client.send(request, HttpResponse.BodyHandlers.ofInputStream());

		if (response.statusCode() / 100 != 2) {
			throw new IOException("the server answered " + response.statusCode());
		}

		response.headers().firstValueAsLong("content-length").ifPresent(total::set);
		MessageDigest digest = sha256();
		byte[] magic = resource.magic();
		byte[] head = new byte[magic.length];
		int headRead = 0;
		byte[] buffer = new byte[CHUNK];

		try (InputStream in = response.body()) {
			while (true) {
				if (cancelled) {
					throw new IOException("cancelled");
				}

				int read = in.read(buffer);
				if (read < 0) {
					break;
				}

				if (headRead < magic.length) {
					int take = Math.min(magic.length - headRead, read);
					System.arraycopy(buffer, 0, head, headRead, take);
					headRead += take;

					if (headRead == magic.length && !Arrays.equals(head, magic)) {
						throw new IOException("not a " + new String(magic, java.nio.charset.StandardCharsets.UTF_8) + " file");
					}
				}

				out.write(buffer, 0, read);
				digest.update(buffer, 0, read);
				bytes.addAndGet(read);
			}
		}

		if (headRead < magic.length) {
			throw new IOException("not a " + new String(magic, java.nio.charset.StandardCharsets.UTF_8) + " file");
		}

		if (!resource.sha256().isEmpty()) {
			String got = HexFormat.of().formatHex(digest.digest());

			if (!got.equals(resource.sha256())) {
				throw new IOException("the file's SHA-256 is " + got + ", expected " + resource.sha256());
			}
		}
	}

	private static MessageDigest sha256() {
		try {
			return MessageDigest.getInstance("SHA-256");
		} catch (NoSuchAlgorithmException e) {
			throw new IllegalStateException("SHA-256 is always there", e);
		}
	}
}
