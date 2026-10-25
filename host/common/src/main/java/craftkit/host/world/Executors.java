package craftkit.host.world;

import com.google.gson.JsonObject;
import craftkit.host.LoadedMod;
import craftkit.host.Log;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;

/**
 * Carries one command out against the level and answers its report. Each command checks what it
 * needs as it runs ({@code fits}, in the design's words) and reports {@code done} or why not; a
 * command the host does not know yet is reported as such rather than dropped, so a mod sees it.
 */
public final class Executors {
	/** Finds a player by the index the mod used, or null when they are gone. */
	public interface Players {
		ServerPlayer find(ServerLevel level, int index);
	}

	private Executors() {
	}

	public static JsonObject run(LoadedMod mod, ServerLevel level, JsonObject command, Players players) {
		long id = command.has("id") ? command.get("id").getAsLong() : 0;
		String kind = command.has("kind") ? command.get("kind").getAsString() : "";

		try {
			return switch (kind) {
				case "log" -> {
					Log.LOGGER.info("{}: {}", mod.id(), command.get("text").getAsString());
					yield report(id, true, "");
				}
				case "system_message" -> {
					ServerPlayer player = players.find(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					Worlds.message(player, command.get("text").getAsString());
					yield report(id, true, "");
				}
				default -> report(id, false, "the host does not carry out " + kind + " yet");
			};
		} catch (RuntimeException e) {
			return report(id, false, kind + ": " + e.getMessage());
		}
	}

	static JsonObject report(long id, boolean done, String reason) {
		JsonObject report = new JsonObject();
		report.addProperty("id", id);
		report.addProperty("done", done);
		report.addProperty("reason", reason);

		return report;
	}
}
