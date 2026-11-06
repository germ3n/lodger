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
import java.nio.file.StandardOpenOption;
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
 * page fails at once rather than after half a gigabyte; and the SHA-256 is checked over the whole
 * file when the resource names one. Progress is two atomics the screen and the server log read
 * from any thread. One {@code Downloader} runs one download, on the thread {@link #run} is called
 * from.
 *
 * <p>A connection that goes quiet is a broken one: thirty seconds without a byte and it is
 * closed, and the download goes on from where it stopped (an HTTP range), up to five tries with
 * a pause between. A download cut off by the network keeps its part file, so the next start goes
 * on from it too; only a cancel, a file that is not what the magic says, or a wrong sum removes
 * it. The 4B model's download hung at 2059 of 2381 MB with no end until 2026-11-10.
 */
public final class Downloader {
	private static final int CHUNK = 1 << 20;
	private static final Duration CONNECT = Duration.ofSeconds(15);
	private static final int TRIES = 5;

	public enum State {
		PENDING, RUNNING, DONE, FAILED
	}

	/** A failure that no try again will mend: the wrong file, the wrong sum, a 4xx, a cancel. */
	static final class Final extends IOException {
		Final(String message) {
			super(message);
		}
	}

	private final Resource resource;
	private final Duration stall;
	private final Duration pause;
	private final AtomicLong bytes = new AtomicLong();
	private final AtomicLong total = new AtomicLong();
	private final AtomicReference<State> state = new AtomicReference<>(State.PENDING);
	private final AtomicReference<String> error = new AtomicReference<>("");
	private volatile boolean cancelled;
	private volatile int tries;

	public Downloader(Resource resource) {
		this(resource, Duration.ofSeconds(30), Duration.ofSeconds(2));
	}

	/** With a shorter quiet time and pause, for the tests. */
	Downloader(Resource resource, Duration stall, Duration pause) {
		this.resource = resource;
		this.stall = stall;
		this.pause = pause;
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

	/** How many connections it took, for the log and the tests. */
	public int tries() {
		return tries;
	}

	/** 0 to 1, or 0 while the size is unknown. */
	public double fraction() {
		long t = total.get();

		return t <= 0 ? 0 : Math.min(1.0, (double) bytes.get() / t);
	}

	/** Asks the download to stop at the next chunk; the part file is kept, for the next start to go on from (Quit used to throw away a download nearly done). */
	public void cancel() {
		cancelled = true;
	}

	/** Downloads, blocking. Afterwards {@link #state()} is DONE or FAILED and {@link #error()} says why. */
	public void run() {
		state.set(State.RUNNING);
		Path path = resource.path();
		Path part = path.resolveSibling(path.getFileName() + ".part");
		boolean keepPart = false;

		try {
			Files.createDirectories(path.getParent());
			IOException last = null;
			for (int attempt = 0; attempt < TRIES; attempt++) {
				if (cancelled) {
					throw new Final("cancelled");
				}
				if (attempt > 0) {
					Thread.sleep(pause.toMillis() * attempt);
				}
				try {
					tries++;
					fetch(part);
					last = null;
					break;
				} catch (Final e) {
					throw e;
				} catch (IOException e) {
					last = e;
				}
			}
			if (last != null) {
				// The network gave out every time: what came is kept for the next start.
				keepPart = true;
				throw last;
			}
			check(part);
			Files.move(part, path, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
			state.set(State.DONE);
		} catch (IOException | InterruptedException | RuntimeException e) {
			if (!keepPart && !cancelled) {
				try {
					Files.deleteIfExists(part);
				} catch (IOException ignored) {
					// The part file is harmless; the next run starts it again.
				}
			}

			error.set(e.getMessage() == null ? e.getClass().getSimpleName() : e.getMessage());
			state.set(State.FAILED);
		}
	}

	/** One connection: from the part file's end when there is one, to the file's end. */
	private void fetch(Path part) throws IOException, InterruptedException {
		long have = Files.exists(part) ? Files.size(part) : 0;
		bytes.set(have);
		// Closed after the try, with its selector thread (until 2026-11-10 one was left per try).
		try (HttpClient client = HttpClient.newBuilder()
				.followRedirects(HttpClient.Redirect.NORMAL)
				.connectTimeout(CONNECT)
				.build()) {
			fetchWith(client, part, have);
		}
	}

	private void fetchWith(HttpClient client, Path part, long have) throws IOException, InterruptedException {
		// The answer's head within the stall time: a server that takes the connection and says
		// nothing used to hold the download, with no Try again, for good.
		HttpRequest.Builder request = HttpRequest.newBuilder(URI.create(resource.url())).timeout(stall).GET();
		if (have > 0) {
			request.header("Range", "bytes=" + have + "-");
		}
		HttpResponse<InputStream> response = client.send(request.build(), HttpResponse.BodyHandlers.ofInputStream());
		int status = response.statusCode();

		if (status == 416 && have > 0) {
			// Nothing after the end: the part file is whole already.
			response.body().close();
			return;
		}
		if (status / 100 == 4) {
			response.body().close();
			throw new Final("the server answered " + status);
		}
		if (status / 100 != 2) {
			response.body().close();
			throw new IOException("the server answered " + status);
		}
		boolean resumed = status == 206 && have > 0;
		if (!resumed) {
			have = 0;
			bytes.set(0);
		}
		long length = response.headers().firstValueAsLong("content-length").orElse(-1);
		if (length >= 0) {
			total.set(have + length);
		}

		byte[] magic = resource.magic();
		byte[] head = new byte[magic.length];
		int headRead = 0;
		if (resumed && magic.length > 0) {
			try (InputStream old = Files.newInputStream(part)) {
				headRead = old.readNBytes(head, 0, magic.length);
			}
			if (headRead == magic.length && !Arrays.equals(head, magic)) {
				throw new Final(notMagic());
			}
		}

		AtomicLong lastByte = new AtomicLong(System.nanoTime());
		InputStream in = response.body();
		Thread watch = Thread.ofVirtual().unstarted(() -> {
			try {
				while (!Thread.currentThread().isInterrupted()) {
					Thread.sleep(Math.max(10, stall.toMillis() / 5));
					if (System.nanoTime() - lastByte.get() > stall.toNanos()) {
						in.close();
						return;
					}
				}
			} catch (InterruptedException | IOException ignored) {
				// Done watching.
			}
		});
		watch.start();
		byte[] buffer = new byte[CHUNK];

		try (in; OutputStream out = resumed
				? Files.newOutputStream(part, StandardOpenOption.APPEND)
				: Files.newOutputStream(part, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING, StandardOpenOption.WRITE)) {
			while (true) {
				if (cancelled) {
					throw new Final("cancelled");
				}

				int read;
				try {
					read = in.read(buffer);
				} catch (IOException e) {
					throw new IOException("the connection went quiet or broke at " + bytes.get() + " bytes", e);
				}
				if (read < 0) {
					break;
				}
				lastByte.set(System.nanoTime());

				if (headRead < magic.length) {
					int take = Math.min(magic.length - headRead, read);
					System.arraycopy(buffer, 0, head, headRead, take);
					headRead += take;

					if (headRead == magic.length && !Arrays.equals(head, magic)) {
						throw new Final(notMagic());
					}
				}

				out.write(buffer, 0, read);
				bytes.addAndGet(read);
			}
		} finally {
			watch.interrupt();
		}

		if (headRead < magic.length) {
			throw new Final(notMagic());
		}
		if (length >= 0 && bytes.get() < total.get()) {
			throw new IOException("the connection ended at " + bytes.get() + " of " + total.get() + " bytes");
		}
	}

	/** The whole file's sum, when the resource names one. */
	private void check(Path part) throws IOException {
		if (resource.sha256().isEmpty()) {
			return;
		}
		MessageDigest digest = sha256();
		byte[] buffer = new byte[CHUNK];
		try (InputStream in = Files.newInputStream(part)) {
			int read;
			while ((read = in.read(buffer)) >= 0) {
				digest.update(buffer, 0, read);
			}
		}
		String got = HexFormat.of().formatHex(digest.digest());
		if (!got.equals(resource.sha256())) {
			throw new Final("the file's SHA-256 is " + got + ", expected " + resource.sha256());
		}
	}

	private String notMagic() {
		return "not a " + new String(resource.magic(), java.nio.charset.StandardCharsets.UTF_8) + " file";
	}

	private static MessageDigest sha256() {
		try {
			return MessageDigest.getInstance("SHA-256");
		} catch (NoSuchAlgorithmException e) {
			throw new IllegalStateException("SHA-256 is always there", e);
		}
	}
}
