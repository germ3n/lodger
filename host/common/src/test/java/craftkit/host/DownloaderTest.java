package craftkit.host;

import com.sun.net.httpserver.HttpServer;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.io.OutputStream;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Duration;
import java.util.HexFormat;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DownloaderTest {
	private HttpServer server;

	private String serve(byte[] body, int status) throws IOException {
		server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
		server.createContext("/model.gguf", exchange -> {
			exchange.sendResponseHeaders(status, body.length);
			try (OutputStream out = exchange.getResponseBody()) {
				out.write(body);
			}
		});
		server.start();

		return "http://127.0.0.1:" + server.getAddress().getPort() + "/model.gguf";
	}

	@AfterEach
	void stop() {
		if (server != null) {
			server.stop(0);
		}
	}

	private static Resource resource(String url, Path path, String magic, String sha256) {
		return new Resource("model", "the model", url, path, 0, magic.getBytes(StandardCharsets.UTF_8), sha256, true, Resource.License.NONE);
	}

	private static String sha256(byte[] bytes) throws NoSuchAlgorithmException {
		return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
	}

	@Test
	void a_file_with_the_right_magic_is_saved_through_its_part_file_with_full_progress(@TempDir Path dir) throws IOException {
		byte[] body = "GGUF-model-bytes".getBytes(StandardCharsets.UTF_8);
		Path path = dir.resolve("models").resolve("qwen.gguf");
		Downloader downloader = new Downloader(resource(serve(body, 200), path, "GGUF", ""));

		downloader.run();

		assertEquals(Downloader.State.DONE, downloader.state(), downloader.error());
		assertArrayEquals(body, Files.readAllBytes(path));
		assertFalse(Files.exists(dir.resolve("models").resolve("qwen.gguf.part")));
		assertEquals(body.length, downloader.bytes());
		assertEquals(body.length, downloader.total(), "the content length became the total");
		assertEquals(1.0, downloader.fraction(), 1e-9);
	}

	@Test
	void a_file_that_is_not_what_the_magic_says_is_refused_and_nothing_is_left(@TempDir Path dir) throws IOException {
		Path path = dir.resolve("qwen.gguf");
		Downloader downloader = new Downloader(resource(serve("<html>not found</html>".getBytes(StandardCharsets.UTF_8), 200), path, "GGUF", ""));

		downloader.run();

		assertEquals(Downloader.State.FAILED, downloader.state());
		assertTrue(downloader.error().contains("GGUF"), downloader.error());
		assertFalse(Files.exists(path));
		assertFalse(Files.exists(dir.resolve("qwen.gguf.part")));
	}

	@Test
	void a_wrong_hash_is_refused_and_a_right_one_passes(@TempDir Path dir) throws Exception {
		byte[] body = "GGUF-hashed".getBytes(StandardCharsets.UTF_8);
		Path path = dir.resolve("qwen.gguf");

		Downloader wrong = new Downloader(resource(serve(body, 200), path, "GGUF", "0".repeat(64)));
		wrong.run();
		assertEquals(Downloader.State.FAILED, wrong.state());
		assertTrue(wrong.error().contains("SHA-256"), wrong.error());
		assertFalse(Files.exists(path));
		stop();

		Downloader right = new Downloader(resource(serve(body, 200), path, "GGUF", sha256(body)));
		right.run();
		assertEquals(Downloader.State.DONE, right.state(), right.error());
	}

	@Test
	void a_server_error_is_reported_with_its_status(@TempDir Path dir) throws IOException {
		Downloader downloader = new Downloader(resource(serve("gone".getBytes(StandardCharsets.UTF_8), 404), dir.resolve("qwen.gguf"), "", ""));

		downloader.run();

		assertEquals(Downloader.State.FAILED, downloader.state());
		assertTrue(downloader.error().contains("404"), downloader.error());
	}

	@Test
	void a_failed_download_never_replaces_a_good_file(@TempDir Path dir) throws IOException {
		Path path = dir.resolve("qwen.gguf");
		Files.write(path, "GGUF-good".getBytes(StandardCharsets.UTF_8));
		Downloader downloader = new Downloader(resource(serve("nope".getBytes(StandardCharsets.UTF_8), 200), path, "GGUF", ""));

		downloader.run();

		assertEquals(Downloader.State.FAILED, downloader.state());
		assertEquals("GGUF-good", Files.readString(path));
	}

	@Test
	void the_fraction_uses_the_size_hint_until_the_server_says_the_length() {
		Resource resource = new Resource("m", "t", "http://x/m", Path.of("/nowhere"), 1000, new byte[0], "", true, Resource.License.NONE);
		Downloader downloader = new Downloader(resource);

		assertEquals(0.0, downloader.fraction(), 1e-9);
		assertEquals(1000, downloader.total());
	}

	/** Requests' Range headers, in order, for the ranged server. */
	private final List<String> ranges = new CopyOnWriteArrayList<>();

	/**
	 * A server that honours `Range: bytes=N-` (unless `ignoreRange`), and on its first
	 * `stalls` requests sends `stallAfter` bytes and then goes quiet.
	 */
	private String serveRanged(byte[] body, int stalls, int stallAfter, boolean ignoreRange) throws IOException {
		AtomicInteger requests = new AtomicInteger();
		server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
		server.setExecutor(Executors.newCachedThreadPool());
		server.createContext("/model.gguf", exchange -> {
			String range = exchange.getRequestHeaders().getFirst("Range");
			ranges.add(range == null ? "" : range);
			int from = 0;
			if (range != null && !ignoreRange) {
				from = Integer.parseInt(range.substring("bytes=".length(), range.length() - 1));
			}
			if (from >= body.length && from > 0) {
				exchange.sendResponseHeaders(416, -1);
				exchange.close();
				return;
			}
			if (from > 0) {
				exchange.getResponseHeaders().add("Content-Range", "bytes " + from + "-" + (body.length - 1) + "/" + body.length);
			}
			exchange.sendResponseHeaders(from > 0 ? 206 : 200, body.length - from);
			OutputStream out = exchange.getResponseBody();
			try {
				if (requests.getAndIncrement() < stalls) {
					out.write(body, from, Math.min(stallAfter, body.length - from));
					out.flush();
					Thread.sleep(3_000);
				} else {
					out.write(body, from, body.length - from);
				}
				out.close();
			} catch (IOException | InterruptedException ignored) {
				// The client hung up on the quiet connection.
			}
		});
		server.start();

		return "http://127.0.0.1:" + server.getAddress().getPort() + "/model.gguf";
	}

	private static byte[] model(int size) {
		byte[] body = new byte[size];
		for (int i = 0; i < size; i++) {
			body[i] = (byte) (i * 31);
		}
		System.arraycopy("GGUF".getBytes(StandardCharsets.UTF_8), 0, body, 0, 4);

		return body;
	}

	private static Downloader quick(Resource resource) {
		return new Downloader(resource, Duration.ofMillis(300), Duration.ofMillis(10));
	}

	@Test
	void a_connection_that_goes_quiet_is_dropped_and_the_download_goes_on_from_where_it_stopped(@TempDir Path dir) throws Exception {
		byte[] body = model(200_000);
		Path path = dir.resolve("qwen.gguf");
		Downloader downloader = quick(resource(serveRanged(body, 1, 120_000, false), path, "GGUF", sha256(body)));

		downloader.run();

		assertEquals(Downloader.State.DONE, downloader.state(), downloader.error());
		assertArrayEquals(body, Files.readAllBytes(path));
		assertEquals(2, downloader.tries());
		assertEquals("", ranges.get(0));
		assertTrue(ranges.get(1).startsWith("bytes="), ranges.toString());
		assertTrue(Integer.parseInt(ranges.get(1).replaceAll("\\D", "")) > 0, "went on, not again: " + ranges);
	}

	@Test
	void a_part_file_left_by_a_cut_download_is_taken_up_at_the_next_start(@TempDir Path dir) throws Exception {
		byte[] body = model(100_000);
		Path path = dir.resolve("qwen.gguf");
		Files.write(dir.resolve("qwen.gguf.part"), java.util.Arrays.copyOf(body, 40_000));
		Downloader downloader = quick(resource(serveRanged(body, 0, 0, false), path, "GGUF", sha256(body)));

		downloader.run();

		assertEquals(Downloader.State.DONE, downloader.state(), downloader.error());
		assertEquals(List.of("bytes=40000-"), ranges);
		assertArrayEquals(body, Files.readAllBytes(path));
		assertEquals(body.length, downloader.total(), "what was there and what came");
	}

	@Test
	void a_server_that_ignores_the_range_gives_the_whole_file_from_the_start(@TempDir Path dir) throws Exception {
		byte[] body = model(50_000);
		Path path = dir.resolve("qwen.gguf");
		Files.write(dir.resolve("qwen.gguf.part"), java.util.Arrays.copyOf(body, 10_000));
		Downloader downloader = quick(resource(serveRanged(body, 0, 0, true), path, "GGUF", sha256(body)));

		downloader.run();

		assertEquals(Downloader.State.DONE, downloader.state(), downloader.error());
		assertArrayEquals(body, Files.readAllBytes(path), "not the old part twice");
	}

	@Test
	void a_whole_part_file_needs_only_its_sum(@TempDir Path dir) throws Exception {
		byte[] body = model(30_000);
		Path path = dir.resolve("qwen.gguf");
		Files.write(dir.resolve("qwen.gguf.part"), body);
		Downloader downloader = quick(resource(serveRanged(body, 0, 0, false), path, "GGUF", sha256(body)));

		downloader.run();

		assertEquals(Downloader.State.DONE, downloader.state(), downloader.error());
		assertArrayEquals(body, Files.readAllBytes(path));
	}

	@Test
	void a_network_that_fails_every_time_keeps_what_came_for_the_next_start(@TempDir Path dir) throws Exception {
		byte[] body = model(100_000);
		Path path = dir.resolve("qwen.gguf");
		Downloader downloader = quick(resource(serveRanged(body, 99, 5_000, false), path, "GGUF", ""));

		downloader.run();

		assertEquals(Downloader.State.FAILED, downloader.state());
		assertEquals(5, downloader.tries());
		assertTrue(downloader.error().contains("quiet"), downloader.error());
		assertEquals(25_000, Files.size(dir.resolve("qwen.gguf.part")), "five quiet pieces kept");
		assertFalse(Files.exists(path));
	}

	@Test
	void a_missing_file_is_not_asked_for_again(@TempDir Path dir) throws IOException {
		Downloader downloader = quick(resource(serve("gone".getBytes(StandardCharsets.UTF_8), 404), dir.resolve("qwen.gguf"), "", ""));

		downloader.run();

		assertEquals(1, downloader.tries());
		assertFalse(Files.exists(dir.resolve("qwen.gguf.part")));
	}
}

