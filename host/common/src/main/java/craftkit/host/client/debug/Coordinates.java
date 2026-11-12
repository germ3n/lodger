package craftkit.host.client.debug;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * The first block position written in a line of the panel, {@code (440,64,-32)} or
 * {@code (440,ground,-32)}, so the line can be clicked to go there. Pure, tested without the game.
 */
public final class Coordinates {
	private static final Pattern SPOT = Pattern.compile("\\((-?\\d+),\\s?(-?\\d+|ground),\\s?(-?\\d+)\\)");

	/** A spot: {@code y} is empty for "ground", the surface there. */
	public record Spot(int x, Optional<Integer> y, int z) {
		public String json() {
			return "[" + x + "," + y.map(String::valueOf).orElse("null") + "," + z + "]";
		}
	}

	/** A spot found in a line, from {@code start} to {@code end} in its characters. */
	public record Found(int start, int end, Spot spot) {
	}

	/** The keys a command's place goes under, the one that says most first. */
	private static final String[] PLACES = {"pos", "at", "to", "from", "min", "around", "door", "bed", "center"};

	private Coordinates() {
	}

	/** Every spot in a line, in order: each its own link (owner: "EVERY place with coordinates"). */
	public static List<Found> findAll(String line) {
		List<Found> found = new ArrayList<>();
		Matcher m = SPOT.matcher(line);
		while (m.find()) {
			Optional<Integer> y = m.group(2).equals("ground") ? Optional.empty() : Optional.of(Integer.parseInt(m.group(2)));
			found.add(new Found(m.start(), m.end(), new Spot(Integer.parseInt(m.group(1)), y, Integer.parseInt(m.group(3)))));
		}

		return found;
	}

	/** A spot as the panel writes it, {@code (440,64,-32)}, or {@code (440,ground,-32)} for a y below -1000. */
	public static String words(double x, double y, double z) {
		return "(" + (int) Math.floor(x) + "," + (y <= -1000 ? "ground" : String.valueOf((int) Math.floor(y))) + "," + (int) Math.floor(z) + ")";
	}

	/** Where a command acts, as {@code " at (x,y,z)"}, or empty when it names no place. */
	public static String placeOf(JsonObject command) {
		for (String key : PLACES) {
			JsonElement value = command.get(key);
			if (value != null && value.isJsonArray()) {
				JsonArray p = value.getAsJsonArray();
				if (p.size() == 3 && p.get(0).isJsonPrimitive() && p.get(0).getAsJsonPrimitive().isNumber()
						&& p.get(1).isJsonPrimitive() && p.get(2).isJsonPrimitive()) {
					return " at " + words(p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble());
				}
			}
		}

		return "";
	}

	public static Optional<Spot> find(String line) {
		Matcher m = SPOT.matcher(line);
		if (!m.find()) {
			return Optional.empty();
		}
		Optional<Integer> y = m.group(2).equals("ground") ? Optional.empty() : Optional.of(Integer.parseInt(m.group(2)));

		return Optional.of(new Spot(Integer.parseInt(m.group(1)), y, Integer.parseInt(m.group(3))));
	}
}
