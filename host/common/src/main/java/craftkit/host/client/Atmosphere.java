package craftkit.host.client;

import java.util.List;

/**
 * The atmospheres' arithmetic, apart from the renderer so JUnit checks it (the review's D, owner:
 * "not abrupt like other mods, make it good and spooky"): how strongly one is on at a
 * moment (in over most of a minute, held with a slow breath, out over half a minute), and how the
 * game's own fog and its colour blend toward each one that is on, several at once.
 */
public final class Atmosphere {
	private Atmosphere() {
	}

	/** One atmosphere's fog: where it starts and ends, in blocks, and its colour, 0 to 1. */
	public record Fog(float start, float end, float r, float g, float b) {
	}

	/** Mist: pale grey, close; 22 blocks out (13 was a whiteout without a shader pack, owner: "i can hardly see anything"). */
	public static final Fog MIST = new Fog(0, 22, 0.74f, 0.76f, 0.80f);
	/** Red mist: deep red, a little farther. */
	public static final Fog RED_MIST = new Fog(0, 20, 0.42f, 0.04f, 0.04f);
	/** A sandstorm: sand-coloured, close. */
	public static final Fog SANDSTORM = new Fog(0, 16, 0.80f, 0.68f, 0.47f);
	/** Darkness: black, a few blocks. */
	public static final Fog DARKNESS = new Fog(0, 7, 0.0f, 0.0f, 0.0f);
	/** Dusk at noon: the light drained, grey-violet, the distance gone. */
	public static final Fog DUSK = new Fog(4, 44, 0.22f, 0.20f, 0.27f);

	/**
	 * Whether a hint turns this compass to its place: `compass_points` any compass, as it did;
	 * `recovery_compass_lies` a recovery compass alone, which then points not at where they died.
	 */
	public static boolean pointsAway(String hint, boolean recovery) {
		// `compass_steadies` (ABI 56): a compass held still on a place in the Nether, as if it worked there.
		return hint.equals("compass_points") || (!recovery && hint.equals("compass_steadies")) || (recovery && hint.equals("recovery_compass_lies"));
	}

	/** The fog an atmosphere's hint stands for, or null for none. The old `fog_close` is mist now. */
	public static Fog of(String hint) {
		return switch (hint) {
			case "mist", "fog_close" -> MIST;
			case "red_mist" -> RED_MIST;
			case "darkness" -> DARKNESS;
			case "sandstorm" -> SANDSTORM;
			case "dusk_noon" -> DUSK;
			default -> null;
		};
	}

	/** How far in, 0 to 1, `elapsed` seconds into an atmosphere `total` seconds long: eased in over most of a minute, out over half of one. */
	public static float weight(float elapsed, float total) {
		if (elapsed <= 0 || elapsed >= total) {
			return 0;
		}
		float in = Math.min(50, total * 0.3f);
		float out = Math.min(35, total * 0.25f);
		if (elapsed < in) {
			return smooth(elapsed / in);
		}
		if (elapsed > total - out) {
			return smooth((total - elapsed) / out);
		}

		return 1;
	}

	/**
	 * {@link #weight} for one that should be seen at once, red rain's: in over {@link #QUICK_IN}
	 * seconds (owner: it eased in over most of a minute and seemed to do nothing), out as any.
	 */
	public static float weightQuick(float elapsed, float total) {
		if (elapsed <= 0 || elapsed >= total) {
			return 0;
		}
		float in = Math.min(QUICK_IN, total * 0.3f);
		float out = Math.min(35, total * 0.25f);
		if (elapsed < in) {
			return smooth(elapsed / in);
		}
		if (elapsed > total - out) {
			return smooth((total - elapsed) / out);
		}
		return 1;
	}

	/** How fast a quick atmosphere comes in, seconds. */
	public static final float QUICK_IN = 8;

	/** The slow breath of a held atmosphere: its edge comes and goes a tenth, over nine seconds. */
	public static float breath(float elapsed) {
		return 1 + 0.1f * (float) Math.sin(elapsed * Math.PI * 2 / 9);
	}

	/** An atmosphere on, and how strongly. */
	public record On(Fog fog, float weight, float breath) {
	}

	/**
	 * The game's fog (start, end, colour) blended toward each atmosphere that is on, in turn:
	 * each pulls the fog as it stands by its weight, closer and toward its colour, so two on
	 * together close it further than either alone (they stack). Answers {start, end, r, g, b}.
	 */
	public static float[] blend(float start, float end, float r, float g, float b, List<On> on) {
		float outStart = start;
		float outEnd = end;
		float[] colour = {r, g, b};
		for (On each : on) {
			float w = Math.max(0, Math.min(1, each.weight()));
			if (w == 0) {
				continue;
			}
			float wantEnd = each.fog().end() * each.breath();
			outEnd = Math.min(outEnd, lerp(outEnd, wantEnd, w));
			outStart = Math.min(outStart, lerp(outStart, each.fog().start(), w));
			colour[0] = lerp(colour[0], each.fog().r(), w);
			colour[1] = lerp(colour[1], each.fog().g(), w);
			colour[2] = lerp(colour[2], each.fog().b(), w);
		}
		outStart = Math.min(outStart, outEnd);

		return new float[] {outStart, outEnd, colour[0], colour[1], colour[2]};
	}

	private static float lerp(float a, float b, float t) {
		return a + (b - a) * t;
	}

	private static float smooth(float t) {
		float c = Math.max(0, Math.min(1, t));

		return c * c * (3 - 2 * c);
	}
}
