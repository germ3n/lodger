package craftkit.host;

import com.mojang.brigadier.Command;
import com.mojang.brigadier.CommandDispatcher;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.network.chat.Component;
import org.slf4j.Logger;

import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * The host: finds every content jar with a {@link Manifest}, loads its Rust mod, and answers
 * {@code /craftkit status}. One per process; each loader's entry point makes it and hooks it to
 * its own events.
 */
public final class Host {
	public static final String MOD_ID = "craftkit";
	public static final Logger LOGGER = Log.LOGGER;
	public static final String VERSION = Optional.ofNullable(Host.class.getPackage().getImplementationVersion()).orElse("dev");

	private final Platform platform;
	private final List<LoadedMod> mods;

	private Host(Platform platform, List<LoadedMod> mods) {
		this.platform = platform;
		this.mods = mods;
	}

	/** Loads every Rust mod the loader has, logging each, and registers the shutdown hook. */
	public static Host start(Platform platform) {
		String osName = System.getProperty("os.name");
		String osArch = System.getProperty("os.arch");
		Path configDir = platform.configDir();
		Path gameDir = platform.gameDir();
		List<LoadedMod> mods = new ArrayList<>();

		for (Platform.ContentJar jar : platform.jars()) {
			Optional<byte[]> manifest;
			try {
				manifest = jar.read(Manifest.PATH);
			} catch (IOException e) {
				LOGGER.warn("{}: could not read {}: {}", jar.id(), Manifest.PATH, e.getMessage());
				continue;
			}
			if (manifest.isEmpty()) {
				continue;
			}

			LoadedMod mod = LoadedMod.load(jar, manifest.get(), gameDir, configDir, osName, osArch);
			mods.add(mod);

			if (mod.loaded()) {
				LOGGER.info("loaded {} {} (sdk {}) from {}", mod.id(), mod.version(), mod.sdk(), Natives.platform(osName, osArch));

				for (String key : mod.unknownKeys()) {
					LOGGER.warn("{}: unknown config key {}, ignored", mod.id(), key);
				}

				if (mod.wroteConfig()) {
					LOGGER.info("{}: wrote {} with every default", mod.id(), mod.manifest().map(Manifest::config).orElse("its config"));
				}
			} else {
				LOGGER.warn("{}: not loaded: {}", mod.id(), mod.failure());
			}
		}

		if (mods.isEmpty()) {
			LOGGER.info("no content jars with {} are loaded", Manifest.PATH);
		}

		Host host = new Host(platform, List.copyOf(mods));
		Preparation.install(new Preparation(host.mods, new Acceptance(configDir.resolve("craftkit").resolve("accepted.json")), !platform.dedicatedServer()));
		// Mods are one per process (a mod may hold a model loaded once for every world), so they
		// are told the process is ending, not that a world closed.
		Runtime.getRuntime().addShutdownHook(new Thread(host::close, "craftkit-shutdown"));

		return host;
	}

	public List<LoadedMod> mods() {
		return mods;
	}

	/** The resources and loading every mod needs before play; the client and the server drive it their own way. */
	public Preparation preparation() {
		return Preparation.current().orElseThrow();
	}

	/**
	 * A dedicated server's way: fetch and load before the world opens, blocking its startup
	 * with progress in the log, and carry on without the mod if it fails. The client's way is
	 * the screen in {@code craftkit.host.client.PreparingScreen}.
	 */
	public void prepareBlocking() {
		Preparation preparation = preparation();

		if (!preparation.needed()) {
			return;
		}

		LOGGER.info("fetching and loading what the mods need before the world opens");
		preparation.run();

		Preparation.Progress progress = preparation.progress();
		if (progress.failed()) {
			LOGGER.error("not ready: {}. The server runs on; the mod goes without until it is fixed.", progress.error());
		}
	}

	public Platform platform() {
		return platform;
	}

	/** {@code /craftkit status}, for operators: what is loaded and what is not, and why. */
	public void registerCommands(CommandDispatcher<CommandSourceStack> dispatcher) {
		dispatcher.register(Commands.literal(MOD_ID)
				.requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))
				.then(Commands.literal("status").executes(context -> {
					CommandSourceStack source = context.getSource();
					source.sendSuccess(() -> Component.translatable("craftkit.status.header", VERSION, platform.name(), mods.size()), false);

					if (mods.isEmpty()) {
						source.sendSuccess(() -> Component.translatable("craftkit.status.none"), false);
					}

					for (LoadedMod mod : mods) {
						if (mod.loaded()) {
							source.sendSuccess(() -> Component.translatable("craftkit.status.loaded", mod.id(), mod.version(), mod.sdk(), Natives.platform()), false);
						} else {
							source.sendSuccess(() -> Component.translatable("craftkit.status.failed", mod.id(), mod.failure()), false);
						}
					}

					return Command.SINGLE_SUCCESS;
				})));
	}

	public void close() {
		for (LoadedMod mod : mods) {
			try {
				mod.close();
			} catch (RuntimeException e) {
				LOGGER.warn("{}: shutdown failed: {}", mod.id(), e.getMessage());
			}
		}
	}
}
