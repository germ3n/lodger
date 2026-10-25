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
	private final craftkit.host.world.Worlds worlds;

	private Host(Platform platform, List<LoadedMod> mods) {
		this.platform = platform;
		this.mods = mods;
		this.worlds = new craftkit.host.world.Worlds(mods, platform::sendToPlayer);
	}

	/** The per-tick exchange with every mod's worlds; the loaders hook its level and tick events. */
	public craftkit.host.world.Worlds worlds() {
		return worlds;
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

	/**
	 * Whether this player may use the debug screen and commands: an operator, or the owner of
	 * the singleplayer world they are in (who has no operator level with cheats off).
	 */
	public static boolean mayDebug(net.minecraft.server.level.ServerPlayer player) {
		net.minecraft.server.MinecraftServer server = player.level().getServer();
		if (server != null && server.isSingleplayerOwner(new net.minecraft.server.players.NameAndId(player.getGameProfile()))) {
			return true;
		}

		return Commands.hasPermission(Commands.LEVEL_GAMEMASTERS).test(player.createCommandSourceStack());
	}

	/**
	 * A debug question from a client ({@link craftkit.host.net.DebugAskPayload}), answered with a
	 * {@link craftkit.host.net.DebugTellPayload}. On the server thread.
	 */
	public void debugAsked(net.minecraft.server.level.ServerPlayer player, craftkit.host.net.DebugAskPayload ask) {
		com.google.gson.JsonObject answer;
		if (!mayDebug(player)) {
			answer = new com.google.gson.JsonObject();
			answer.addProperty("error", "operators only");
		} else {
			com.google.gson.JsonObject request;
			try {
				request = com.google.gson.JsonParser.parseString(ask.json().isEmpty() ? "{\"kind\":\"snapshot\"}" : ask.json()).getAsJsonObject();
			} catch (RuntimeException e) {
				request = new com.google.gson.JsonObject();
				request.addProperty("kind", "snapshot");
			}
			answer = worlds.debug(player, ask.mod(), request);
		}

		platform.sendToPlayer(player, new craftkit.host.net.DebugTellPayload(ask.mod(), answer.toString()));
	}

	/**
	 * {@code /craftkit status}, for operators: what is loaded and what is not, and why. And per
	 * mod, {@code /craftkit <id> debug} (opens the screen on the caller's client),
	 * {@code force <event>}, {@code stage <n>}, {@code action <id>} and {@code show}, the
	 * screen's buttons as chat commands for a dedicated server's console or a player without
	 * the key.
	 */
	public void registerCommands(CommandDispatcher<CommandSourceStack> dispatcher) {
		com.mojang.brigadier.builder.LiteralArgumentBuilder<CommandSourceStack> root = Commands.literal(MOD_ID)
				.requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS));

		for (LoadedMod mod : mods) {
			if (!mod.loaded()) {
				continue;
			}
			String id = mod.id();
			root.then(Commands.literal(id)
					.then(Commands.literal("debug").executes(context -> {
						net.minecraft.server.level.ServerPlayer player = context.getSource().getPlayer();
						if (player == null) {
							context.getSource().sendFailure(Component.translatable("craftkit.debug.no_screen"));
							return 0;
						}
						platform.sendToPlayer(player, new craftkit.host.net.DebugTellPayload(id, "{\"open\":true}"));
						return Command.SINGLE_SUCCESS;
					}))
					.then(Commands.literal("show").executes(context -> debugCommand(context.getSource(), id, "{\"kind\":\"snapshot\"}")))
					.then(Commands.literal("force").then(Commands.argument("event", com.mojang.brigadier.arguments.StringArgumentType.word())
							.executes(context -> debugCommand(context.getSource(), id, "{\"kind\":\"force\",\"event\":\"" + com.mojang.brigadier.arguments.StringArgumentType.getString(context, "event") + "\",\"player\":0}"))))
					.then(Commands.literal("stage").then(Commands.argument("stage", com.mojang.brigadier.arguments.IntegerArgumentType.integer(0, 6))
							.executes(context -> debugCommand(context.getSource(), id, "{\"kind\":\"set_stage\",\"stage\":" + com.mojang.brigadier.arguments.IntegerArgumentType.getInteger(context, "stage") + "}"))))
					.then(Commands.literal("action").then(Commands.argument("action", com.mojang.brigadier.arguments.StringArgumentType.word())
							.executes(context -> debugCommand(context.getSource(), id, "{\"kind\":\"action\",\"id\":\"" + com.mojang.brigadier.arguments.StringArgumentType.getString(context, "action") + "\"}")))));
		}

		dispatcher.register(root
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

	/** A debug request from chat: the mod's message, or its gauges for {@code show}, as chat lines. */
	private int debugCommand(CommandSourceStack source, String modId, String requestJson) {
		net.minecraft.server.level.ServerPlayer player = source.getPlayer();
		if (player == null) {
			source.sendFailure(Component.translatable("craftkit.debug.no_screen"));
			return 0;
		}

		com.google.gson.JsonObject answer = worlds.debug(player, modId, com.google.gson.JsonParser.parseString(requestJson).getAsJsonObject());
		if (answer.has("error") && !answer.get("error").getAsString().isEmpty()) {
			source.sendFailure(Component.literal(answer.get("error").getAsString()));
			return 0;
		}

		com.google.gson.JsonObject mod = answer.getAsJsonObject("mod");
		if (mod == null) {
			source.sendFailure(Component.literal("no answer from " + modId));
			return 0;
		}
		if (mod.has("message") && !mod.get("message").getAsString().isEmpty()) {
			source.sendSuccess(() -> Component.literal(mod.get("message").getAsString()), false);
		} else if (mod.has("gauges")) {
			for (com.google.gson.JsonElement gauge : mod.getAsJsonArray("gauges")) {
				com.google.gson.JsonObject g = gauge.getAsJsonObject();
				source.sendSuccess(() -> Component.literal(g.get("label").getAsString() + ": " + g.get("text").getAsString()), false);
			}
		}

		return Command.SINGLE_SUCCESS;
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
