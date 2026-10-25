package craftkit.host;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * The host's logger, in a class with no Minecraft imports, so the parts that run in plain JUnit
 * tests (the downloader, the acceptance file, the preparation) can log without loading the game.
 */
public final class Log {
	public static final Logger LOGGER = LoggerFactory.getLogger(Host.MOD_ID);

	private Log() {
	}
}
