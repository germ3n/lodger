package craftkit.host.client.debug;

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

	private Coordinates() {
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
