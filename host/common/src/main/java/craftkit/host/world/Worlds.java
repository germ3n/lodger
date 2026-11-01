package craftkit.host.world;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import craftkit.host.LoadedMod;
import craftkit.host.Log;
import craftkit.host.Native;
import craftkit.host.net.HintPayload;
import craftkit.host.senses.Senses;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;

import java.util.ArrayList;
import java.util.Base64;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.function.BiConsumer;

/**
 * The per-tick exchange between the game and every loaded mod's worlds: open a world for each
 * mod when a level loads, with its bytes from saved data; each tick gather what the level's
 * players are doing and send it, tick, carry out the commands, report; save every few seconds
 * and on unload; close on unload. All on the server thread.
 *
 * <p>Phase 2 (see {@code docs/todo.md}): the players' basic state is observed and {@code Log},
 * {@code SystemMessage} and {@code PlaySound} are carried out; the senses proper and the rest of
 * the executors are phase 3.
 */
public final class Worlds {
	/** How often a world's bytes go to saved data: every 100 ticks, five seconds. */
	private static final int SAVE_EVERY = 100;

	private final List<LoadedMod> mods;
	private final Map<ServerLevel, Map<LoadedMod, Open>> open = new HashMap<>();
	private final Senses senses = new Senses();
	private final Looks looks = new Looks();
	private final Holds holds = new Holds();
	private final Hidden hidden = new Hidden();
	private final Warps warps = new Warps();
	private final LootOffers loot = new LootOffers();
	private final Later later = new Later();
	private final Poses poses = new Poses();
	private final Figures figures = new Figures();
	private final BiConsumer<ServerPlayer, HintPayload> sendHint;
	private int ticks;
	/** The last commands carried out and how they went, newest last, for the debug screen. */
	private final java.util.ArrayDeque<String> recentCommands = new java.util.ArrayDeque<>();
	private static final int MOST_RECENT = 40;

	private record Open(long handle, WorldData data) {
	}

	/** @param sendHint how this loader sends a hint payload to a player. */
	public Worlds(List<LoadedMod> mods, BiConsumer<ServerPlayer, HintPayload> sendHint) {
		this.mods = mods;
		this.sendHint = sendHint;
		senses.lootHook((player, chest) -> loot.apply(player, chest, modId -> {
			Map<LoadedMod, Open> perMod = open.get((ServerLevel) player.level());
			if (perMod == null) {
				return null;
			}
			return perMod.entrySet().stream().filter(e -> e.getKey().id().equals(modId)).map(e -> e.getValue().data()).findFirst().orElse(null);
		}));
	}

	/** What the loaders' hooks feed. */
	public Senses senses() {
		return senses;
	}

	public void levelLoaded(MinecraftServer server, ServerLevel level) {
		Map<LoadedMod, Open> perMod = new HashMap<>();

		for (LoadedMod mod : mods) {
			if (mod.library().isEmpty()) {
				continue;
			}

			WorldData data = level.getDataStorage().computeIfAbsent(WorldData.type(mod.id()));
			byte[] saved = data.bytes().isEmpty() ? new byte[0] : Base64.getDecoder().decode(data.bytes());
			JsonObject info = new JsonObject();
			info.addProperty("id", level.dimension().identifier().toString());
			info.addProperty("seed", level.getSeed());
			info.addProperty("name", server.getWorldData().getLevelName());

			JsonObject answer = JsonParser.parseString(mod.library().get().worldOpen(info.toString(), saved)).getAsJsonObject();
			if (answer.has("error")) {
				Log.LOGGER.warn("{}: could not open {}: {}", mod.id(), info.get("id").getAsString(), answer.get("error").getAsString());
				continue;
			}

			perMod.put(mod, new Open(answer.get("handle").getAsLong(), data));
			Log.LOGGER.info("{}: opened {} ({} saved bytes)", mod.id(), info.get("id").getAsString(), saved.length);
		}

		open.put(level, perMod);
	}

	public void levelUnloaded(ServerLevel level) {
		// No mob is saved frozen: what the level holds is let go before it closes.
		holds.releaseIn(level);
		// No light is saved gone: what the level hides is put back before it closes.
		hidden.putBackIn(level);
		warps.restoreIn(level);
		Map<LoadedMod, Open> perMod = open.remove(level);
		if (perMod == null) {
			return;
		}

		Surroundings.forget(level);
		perMod.forEach((mod, world) -> {
			save(mod, world);
			String error = LoadedMod.errorOf(mod.library().get().worldClose(world.handle()));
			if (!error.isEmpty()) {
				Log.LOGGER.warn("{}: close: {}", mod.id(), error);
			}
		});
	}

	/** Saves and closes everything: the server is stopping. */
	public void closeAll() {
		for (ServerLevel level : new ArrayList<>(open.keySet())) {
			levelUnloaded(level);
		}
	}

	public void tick(MinecraftServer server) {
		ticks++;
		long realMillis = System.currentTimeMillis();
		looks.tick();
		holds.tick(server);
		hidden.tick();
		warps.tick(server);
		later.tick();
		poses.tick();
		figures.tick();
		Surroundings.tick();
		Executors.Context context = new Executors.Context(senses, sendHint, looks, later, poses, figures, holds, loot, hidden, warps);

		open.forEach((level, perMod) -> {
			String observations = senses.observe(level, server.getTickCount()).toString();
			String now = now(server, level, realMillis).toString();

			perMod.forEach((mod, world) -> {
				Native library = mod.library().get();
				String error = LoadedMod.errorOf(library.worldObserve(world.handle(), observations));
				if (!error.isEmpty()) {
					Log.LOGGER.warn("{}: observe: {}", mod.id(), error);
					return;
				}

				JsonElement commands = JsonParser.parseString(library.worldTick(world.handle(), now));
				if (commands.isJsonObject() && commands.getAsJsonObject().has("error")) {
					Log.LOGGER.warn("{}: tick: {}", mod.id(), commands.getAsJsonObject().get("error").getAsString());
					return;
				}

				JsonArray reports = new JsonArray();
				for (JsonElement element : commands.getAsJsonArray()) {
					JsonObject report = Executors.run(mod, level, element.getAsJsonObject(), context, world.data());
					reports.add(report);
					remember(mod, element.getAsJsonObject(), report);
				}

				if (!reports.isEmpty()) {
					error = LoadedMod.errorOf(library.worldReport(world.handle(), reports.toString()));
					if (!error.isEmpty()) {
						Log.LOGGER.warn("{}: report: {}", mod.id(), error);
					}
				}

				if (ticks % SAVE_EVERY == 0) {
					save(mod, world);
				}
			});
		});
	}

	private void remember(LoadedMod mod, JsonObject command, JsonObject report) {
		String kind = command.has("kind") ? command.get("kind").getAsString() : "?";
		String line = mod.id() + ": " + kind + (report.get("done").getAsBoolean() ? " done" : " not done, " + report.get("reason").getAsString());
		if (Senses.DEBUG) {
			Log.LOGGER.info("did: {} {}", line, command);
		}
		recentCommands.addLast(line);
		while (recentCommands.size() > MOST_RECENT) {
			recentCommands.removeFirst();
		}
	}

	/** The mods with an open world, by id, for the debug screen's picker. */
	public List<String> openModIds(ServerLevel level) {
		Map<LoadedMod, Open> perMod = open.get(level);
		if (perMod == null) {
			return List.of();
		}

		return perMod.keySet().stream().map(LoadedMod::id).sorted().toList();
	}

	/**
	 * The debug screen's question about one mod in the player's level: the host's own panel
	 * (the senses as they read this player, the last commands) and the mod's {@code DebugReply},
	 * as one JSON object. On the server thread. An unknown mod or a closed world is an error.
	 */
	public JsonObject debug(ServerPlayer player, String modId, JsonObject request) {
		JsonObject answer = new JsonObject();
		if (!(player.level() instanceof ServerLevel level)) {
			answer.addProperty("error", "the player is in no level");
			return answer;
		}

		JsonArray ids = new JsonArray();
		openModIds(level).forEach(ids::add);
		answer.add("mods", ids);
		answer.add("host", hostPanel(level, player));

		Map<LoadedMod, Open> perMod = open.get(level);
		LoadedMod mod = perMod == null ? null : perMod.keySet().stream().filter(m -> m.id().equals(modId)).findFirst().orElse(null);
		if (mod == null) {
			answer.addProperty("error", modId.isEmpty() ? "" : "no open world for " + modId + " here");
			return answer;
		}

		// A force names the player by the index the senses gave them, which the client does not know.
		if ("force".equals(request.has("kind") ? request.get("kind").getAsString() : "")) {
			request.addProperty("player", senses.indexOf(player));
		}

		JsonElement reply = JsonParser.parseString(mod.library().get().worldDebug(perMod.get(mod).handle(), request.toString()));
		if (reply.isJsonObject() && reply.getAsJsonObject().has("error") && !reply.getAsJsonObject().has("gauges")) {
			answer.addProperty("error", reply.getAsJsonObject().get("error").getAsString());
		} else {
			answer.add("mod", reply);
		}

		return answer;
	}

	private JsonObject hostPanel(ServerLevel level, ServerPlayer player) {
		JsonObject panel = senses.panel(level, player);
		JsonArray commands = new JsonArray();
		recentCommands.descendingIterator().forEachRemaining(commands::add);
		panel.add("commands", commands);
		panel.addProperty("looks", looks.count());
		panel.addProperty("later", later.count());
		panel.addProperty("poses", poses.count());
		panel.addProperty("figures", figures.count());

		return panel;
	}

	private void save(LoadedMod mod, Open world) {
		JsonObject answer = JsonParser.parseString(mod.library().get().worldSave(world.handle())).getAsJsonObject();

		if (answer.has("error")) {
			Log.LOGGER.warn("{}: save: {}", mod.id(), answer.get("error").getAsString());
			return;
		}

		world.data().set(answer.get("bytes").getAsString());

		if (ticks <= SAVE_EVERY) {
			Log.LOGGER.info("{}: first save, {} base64 characters", mod.id(), answer.get("bytes").getAsString().length());
		}
	}

	private static JsonObject now(MinecraftServer server, ServerLevel level, long realMillis) {
		JsonObject now = new JsonObject();
		now.addProperty("tick", server.getTickCount());
		// 26.3 keeps the clock as game time on the level data; the time of day is its remainder.
		long time = level.getLevelData().getGameTime();
		now.addProperty("day_time", time % 24000L);
		now.addProperty("day", time / 24000L);
		now.addProperty("real_millis", realMillis);

		return now;
	}
}
