package craftkit.host.client.debug;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * What the debug screen shows, read out of a {@code DebugTellPayload}'s JSON: the host's panel
 * (this player's senses as meters and lines, the entities near, the last events taken in, the
 * last commands carried out) and the mod's {@code DebugReply} (its gauges, sections, events and
 * actions). Plain Java with no game classes, so it is tested without one and the screen only
 * lays it out.
 */
public final class DebugModel {
	/** One meter: a label, a fraction, and the words beside it. */
	public record Gauge(String label, float value, float max, String text) {
		public float fraction() {
			return max <= 0 ? 0 : Math.max(0, Math.min(1, value / max));
		}
	}

	public record Section(String title, List<String> lines) {
	}

	public record EventRow(String id, String words, String stages, boolean offered, String why, int times) {
	}

	public record ActionRow(String id, String label) {
	}

	public final List<String> mods = new ArrayList<>();
	public String error = "";
	public String message = "";
	/** The host's meters for the asking player. */
	public final List<Gauge> senses = new ArrayList<>();
	/** The host's lines: position, light, what they look at, the world's clock. */
	public final List<String> senseLines = new ArrayList<>();
	public final List<String> entities = new ArrayList<>();
	public final List<String> sensedEvents = new ArrayList<>();
	public final List<String> commands = new ArrayList<>();
	public final List<Gauge> gauges = new ArrayList<>();
	public final List<Section> sections = new ArrayList<>();
	public final List<EventRow> events = new ArrayList<>();
	public final List<ActionRow> actions = new ArrayList<>();

	public static DebugModel parse(String json) {
		DebugModel model = new DebugModel();
		JsonObject root;
		try {
			root = JsonParser.parseString(json).getAsJsonObject();
		} catch (RuntimeException e) {
			model.error = "unreadable answer: " + e.getMessage();
			return model;
		}

		if (root.has("mods")) {
			for (JsonElement id : root.getAsJsonArray("mods")) {
				model.mods.add(id.getAsString());
			}
		}
		if (root.has("error")) {
			model.error = root.get("error").getAsString();
		}
		if (root.has("host") && root.get("host").isJsonObject()) {
			model.readHost(root.getAsJsonObject("host"));
		}
		if (root.has("mod") && root.get("mod").isJsonObject()) {
			model.readMod(root.getAsJsonObject("mod"));
		}

		return model;
	}

	private void readHost(JsonObject host) {
		JsonObject player = host.has("player") ? host.getAsJsonObject("player") : new JsonObject();
		JsonObject move = player.has("movement") ? player.getAsJsonObject("movement") : new JsonObject();

		senses.add(new Gauge("pace", number(move, "pace"), 6, String.format(Locale.ROOT, "%.1f blocks/s", number(move, "pace"))));
		senses.add(new Gauge("still", number(move, "still_seconds"), 30, String.format(Locale.ROOT, "%.0fs standing still", number(move, "still_seconds"))));
		senses.add(new Gauge("stops", number(move, "stops"), 12, String.format(Locale.ROOT, "%.0f stops in the last minute", number(move, "stops"))));
		senses.add(new Gauge("turns", number(move, "turns"), 12, String.format(Locale.ROOT, "%.0f sharp turns", number(move, "turns"))));
		senses.add(new Gauge("look backs", number(move, "look_backs"), 6, String.format(Locale.ROOT, "%.0f looks behind", number(move, "look_backs"))));
		senses.add(new Gauge("backing away", number(move, "backing_away"), 10, String.format(Locale.ROOT, "%.1fs moving backwards", number(move, "backing_away"))));
		senses.add(new Gauge("sprint bursts", number(move, "sprint_bursts"), 8, String.format(Locale.ROOT, "%.0f sprint starts", number(move, "sprint_bursts"))));
		senses.add(new Gauge("fear", fear(move), 1, fearWords(fear(move))));
		senses.add(new Gauge("light", number(player, "light"), 15, String.format(Locale.ROOT, "block %.0f, sky %.0f", number(player, "light"), number(player, "sky_light"))));
		senses.add(new Gauge("health", number(player, "health"), 20, String.format(Locale.ROOT, "%.0f health, %.0f hunger", number(player, "health"), number(player, "hunger"))));
		float dayTime = number(host, "day_time");
		senses.add(new Gauge("day", dayTime, 24000, String.format(Locale.ROOT, "day %.0f, time %.0f%s", number(host, "day"), dayTime, dayTime >= 13000 && dayTime <= 23000 ? ", night" : "")));

		senseLines.add("player index " + (int) number(host, "index") + ", tick " + (long) number(host, "tick"));
		if (player.has("pos")) {
			JsonArray pos = player.getAsJsonArray("pos");
			senseLines.add(String.format(Locale.ROOT, "at %.1f %.1f %.1f, yaw %.0f, pitch %.0f", pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble(), number(player, "yaw"), number(player, "pitch")));
		}
		if (player.has("biome")) {
			senseLines.add(player.get("biome").getAsString() + " in " + text(player, "dimension") + (player.has("sky_light") && number(player, "sky_light") < 8 ? ", indoors to the mind" : ", outdoors to the mind"));
		}
		List<String> state = new ArrayList<>();
		for (String flag : new String[] {"sneaking", "sprinting", "swimming", "sleeping"}) {
			if (player.has(flag) && player.get(flag).getAsBoolean()) {
				state.add(flag);
			}
		}
		if (player.has("riding") && !player.get("riding").getAsString().isEmpty()) {
			state.add("riding " + player.get("riding").getAsString());
		}
		senseLines.add(state.isEmpty() ? "standing or walking" : String.join(", ", state));
		if (player.has("main_hand")) {
			JsonObject hand = player.getAsJsonObject("main_hand");
			senseLines.add("holding " + (hand.has("id") && !hand.get("id").getAsString().isEmpty() ? hand.get("id").getAsString() : "nothing"));
		}
		if (player.has("looking_at")) {
			JsonObject target = player.getAsJsonObject("looking_at");
			senseLines.add("looking at " + (target.has("block") ? target.get("block").getAsString() : target.has("entity") ? target.get("entity").getAsString() + " #" + (int) number(target, "id") : "?"));
		} else {
			senseLines.add("looking at nothing within 8 blocks");
		}
		senseLines.add((host.has("raining") && host.get("raining").getAsBoolean() ? (host.has("thundering") && host.get("thundering").getAsBoolean() ? "thundering" : "raining") : "clear") + ", " + (int) number(host, "players") + " player(s) in the level");
		senseLines.add("host timers: " + (int) number(host, "looks") + " looks, " + (int) number(host, "poses") + " poses, " + (int) number(host, "later") + " later");

		strings(host, "entities", entities);
		strings(host, "events", sensedEvents);
		strings(host, "commands", commands);
	}

	private void readMod(JsonObject mod) {
		message = text(mod, "message");
		if (mod.has("gauges")) {
			for (JsonElement element : mod.getAsJsonArray("gauges")) {
				JsonObject g = element.getAsJsonObject();
				gauges.add(new Gauge(text(g, "label"), number(g, "value"), number(g, "max"), text(g, "text")));
			}
		}
		if (mod.has("sections")) {
			for (JsonElement element : mod.getAsJsonArray("sections")) {
				JsonObject s = element.getAsJsonObject();
				List<String> lines = new ArrayList<>();
				strings(s, "lines", lines);
				sections.add(new Section(text(s, "title"), lines));
			}
		}
		if (mod.has("events")) {
			for (JsonElement element : mod.getAsJsonArray("events")) {
				JsonObject e = element.getAsJsonObject();
				events.add(new EventRow(text(e, "id"), text(e, "words"), text(e, "stages"), e.has("offered") && e.get("offered").getAsBoolean(), text(e, "why"), (int) number(e, "times")));
			}
		}
		if (mod.has("actions")) {
			for (JsonElement element : mod.getAsJsonArray("actions")) {
				JsonObject a = element.getAsJsonObject();
				actions.add(new ActionRow(text(a, "id"), text(a, "label")));
			}
		}
	}

	/**
	 * A fear reading from the movement, zero to one: look backs, backing away, stops and sprint
	 * bursts each add, as the mind reads them (docs/entity.md, Senses). A meter, not a truth.
	 */
	static float fear(JsonObject move) {
		float score = number(move, "look_backs") * 0.25f
				+ Math.min(number(move, "backing_away"), 6) * 0.08f
				+ Math.min(number(move, "stops"), 8) * 0.04f
				+ Math.min(number(move, "sprint_bursts"), 4) * 0.06f;

		return Math.max(0, Math.min(1, score));
	}

	static String fearWords(float fear) {
		if (fear < 0.15f) {
			return "calm";
		}
		if (fear < 0.4f) {
			return "watchful";
		}
		if (fear < 0.7f) {
			return "uneasy";
		}

		return "afraid";
	}

	/** The events whose id or words contain the filter, case blind; all of them for an empty filter. */
	public List<EventRow> eventsMatching(String filter) {
		String needle = filter.trim().toLowerCase(Locale.ROOT);
		List<EventRow> out = new ArrayList<>();
		for (EventRow row : events) {
			if (needle.isEmpty() || row.id().contains(needle) || row.words().toLowerCase(Locale.ROOT).contains(needle) || row.why().toLowerCase(Locale.ROOT).contains(needle)) {
				out.add(row);
			}
		}

		return out;
	}

	private static float number(JsonObject object, String key) {
		return object != null && object.has(key) && object.get(key).isJsonPrimitive() && object.get(key).getAsJsonPrimitive().isNumber() ? object.get(key).getAsFloat() : 0;
	}

	private static String text(JsonObject object, String key) {
		return object != null && object.has(key) && object.get(key).isJsonPrimitive() ? object.get(key).getAsString() : "";
	}

	private static void strings(JsonObject object, String key, List<String> into) {
		if (object.has(key) && object.get(key).isJsonArray()) {
			for (JsonElement element : object.getAsJsonArray(key)) {
				into.add(element.getAsString());
			}
		}
	}
}
