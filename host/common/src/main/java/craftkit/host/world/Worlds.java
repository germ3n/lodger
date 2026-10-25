package craftkit.host.world;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import craftkit.host.LoadedMod;
import craftkit.host.Log;
import craftkit.host.Native;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.Base64;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

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
	/** Player indices, stable for the server's life: events and commands name players by them. */
	private final Map<UUID, Integer> indices = new LinkedHashMap<>();
	private int ticks;

	private record Open(long handle, WorldData data) {
	}

	public Worlds(List<LoadedMod> mods) {
		this.mods = mods;
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
		Map<LoadedMod, Open> perMod = open.remove(level);
		if (perMod == null) {
			return;
		}

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

		open.forEach((level, perMod) -> {
			String observations = observe(level).toString();
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
					reports.add(Executors.run(mod, level, element.getAsJsonObject(), this::player));
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

	/** The player with this index, if online, for the executors. */
	private ServerPlayer player(ServerLevel level, int index) {
		for (ServerPlayer player : level.players()) {
			if (indexOf(player) == index) {
				return player;
			}
		}

		return null;
	}

	private int indexOf(ServerPlayer player) {
		return indices.computeIfAbsent(player.getUUID(), uuid -> indices.size());
	}

	private JsonObject observe(ServerLevel level) {
		JsonObject observations = new JsonObject();
		JsonArray players = new JsonArray();

		for (ServerPlayer player : level.players()) {
			JsonObject p = new JsonObject();
			p.addProperty("index", indexOf(player));
			p.addProperty("name", player.getName().getString());
			p.addProperty("uuid", player.getUUID().toString());
			p.add("pos", vec(player.position()));
			p.addProperty("yaw", player.getYRot());
			p.addProperty("pitch", player.getXRot());
			p.add("velocity", vec(player.getDeltaMovement()));
			p.addProperty("sneaking", player.isShiftKeyDown());
			p.addProperty("sprinting", player.isSprinting());
			p.addProperty("swimming", player.isSwimming());
			p.addProperty("sleeping", player.isSleeping());
			p.addProperty("riding", player.getVehicle() == null ? "" : BuiltInRegistries.ENTITY_TYPE.getKey(player.getVehicle().getType()).toString());
			p.addProperty("health", player.getHealth());
			p.addProperty("hunger", player.getFoodData().getFoodLevel());
			p.addProperty("air", player.getAirSupply());
			p.addProperty("level", player.experienceLevel);
			p.add("main_hand", stack(player.getMainHandItem()));
			p.add("off_hand", stack(player.getOffhandItem()));
			p.addProperty("dimension", level.dimension().identifier().toString());
			players.add(p);
		}

		observations.add("players", players);

		return observations;
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

	static JsonArray vec(Vec3 v) {
		JsonArray array = new JsonArray();
		array.add(v.x);
		array.add(v.y);
		array.add(v.z);

		return array;
	}

	static JsonObject stack(ItemStack stack) {
		JsonObject object = new JsonObject();
		object.addProperty("id", stack.isEmpty() ? "" : BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
		object.addProperty("count", stack.isEmpty() ? 0 : stack.getCount());

		return object;
	}

	/** A system message to a player, for the executors. */
	static void message(ServerPlayer player, String text) {
		player.sendSystemMessage(Component.literal(text));
	}
}
