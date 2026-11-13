package craftkit.host.client;

import java.util.Locale;

/**
 * The `motes` hint's spec, apart from the game so JUnit checks it (docs/events_todo.md, 321 to
 * 401): particles drawn on the target's client alone, a few a tick, around a centre that stands
 * at the hint's place, at the player, or moves from the place to another over the hint's time.
 * One hint for the smoke over a roof, the dust crossing a field, the lights over a swamp, the
 * sparks at a rod, the rain on them alone, rather than a hint each.
 *
 * <p>The detail is {@code key=value} pairs split by {@code ;}: {@code p} the particle (an id, or
 * {@code dust:RRGGBB:size}), {@code n} how many a tick (a fraction is a chance), {@code r} the
 * spread across, {@code h} the spread up, {@code v} the velocity {@code x,y,z}, {@code to} where
 * the centre goes {@code x,y,z}, {@code at=player} to stand at them, {@code o} an offset
 * {@code x,y,z}, {@code every} ticks between draws.
 */
public final class Motes {
	public record Spec(String particle, int colour, float size, float perTick, double spread, double height,
			double[] velocity, double[] to, boolean atPlayer, double[] offset, int every) {
	}

	private Motes() {
	}

	private static double[] three(String text) {
		String[] parts = text.split(",");
		if (parts.length != 3) {
			return null;
		}
		try {
			return new double[] {Double.parseDouble(parts[0]), Double.parseDouble(parts[1]), Double.parseDouble(parts[2])};
		} catch (NumberFormatException e) {
			return null;
		}
	}

	private static double number(String text, double otherwise) {
		try {
			return Double.parseDouble(text);
		} catch (NumberFormatException e) {
			return otherwise;
		}
	}

	/** The spec a detail stands for; anything it cannot read is left at its default. */
	public static Spec parse(String detail) {
		String particle = "minecraft:smoke";
		int colour = 0xFFFFFF;
		float size = 1;
		float perTick = 1;
		double spread = 0.5;
		double height = 0.5;
		double[] velocity = {0, 0, 0};
		double[] to = null;
		boolean atPlayer = false;
		double[] offset = {0, 0, 0};
		int every = 1;
		for (String pair : detail.split(";")) {
			int eq = pair.indexOf('=');
			if (eq <= 0) {
				continue;
			}
			String key = pair.substring(0, eq).trim().toLowerCase(Locale.ROOT);
			String value = pair.substring(eq + 1).trim();
			switch (key) {
				case "p" -> {
					if (value.startsWith("dust:")) {
						String[] parts = value.split(":");
						particle = "dust";
						if (parts.length > 1) {
							try {
								colour = Integer.parseInt(parts[1], 16) & 0xFFFFFF;
							} catch (NumberFormatException ignored) {
								// The default colour.
							}
						}
						if (parts.length > 2) {
							size = (float) Math.max(0.1, Math.min(4, number(parts[2], 1)));
						}
					} else {
						particle = value;
					}
				}
				case "n" -> perTick = (float) Math.max(0, Math.min(20, number(value, 1)));
				case "r" -> spread = Math.max(0, Math.min(32, number(value, 0.5)));
				case "h" -> height = Math.max(0, Math.min(32, number(value, 0.5)));
				case "v" -> {
					double[] v = three(value);
					if (v != null) {
						velocity = v;
					}
				}
				case "to" -> to = three(value);
				case "at" -> atPlayer = value.equals("player");
				case "o" -> {
					double[] o = three(value);
					if (o != null) {
						offset = o;
					}
				}
				case "every" -> every = (int) Math.max(1, Math.min(200, number(value, 1)));
				default -> {
				}
			}
		}

		return new Spec(particle, colour, size, perTick, spread, height, velocity, to, atPlayer, offset, every);
	}

	/** Where the motes gather now: at the player, or from the hint's place toward {@code to} as its time runs. */
	public static double[] centre(Spec spec, double[] from, double[] player, float progress) {
		double[] base;
		if (spec.atPlayer()) {
			base = player.clone();
		} else if (spec.to() != null) {
			double t = Math.max(0, Math.min(1, progress));
			base = new double[] {from[0] + (spec.to()[0] - from[0]) * t, from[1] + (spec.to()[1] - from[1]) * t, from[2] + (spec.to()[2] - from[2]) * t};
		} else {
			base = from.clone();
		}

		return new double[] {base[0] + spec.offset()[0], base[1] + spec.offset()[1], base[2] + spec.offset()[2]};
	}

	/** How many to draw this tick: the whole of {@code perTick}, and one more by its fraction against a draw from 0 to 1. */
	public static int count(Spec spec, long tick, double draw) {
		if (tick % spec.every() != 0) {
			return 0;
		}
		int whole = (int) Math.floor(spec.perTick());

		return whole + (draw < spec.perTick() - whole ? 1 : 0);
	}
}
