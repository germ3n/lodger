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
import java.util.HexFormat;

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
}
