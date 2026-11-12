package craftkit.host.client;

/**
 * The veil a mist draws over the screen while a shader pack is on (owner: "mist doesnt seem to do
 * anything"), apart from the game so JUnit checks it: a shader pack draws its own fog and leaves
 * the game's, which the mists move, unused; so with one on, the mists are drawn over everything
 * instead, a pale veil (a red one for the red mist), as thick as the mist has grown.
 */
public final class MistVeil {
	/** The veil at full weight: the mist, the closing fog, the red mist. */
	public static final float MIST = 0.28f;
	public static final float FOG_CLOSE = 0.45f;
	public static final float RED_MIST = 0.30f;
	public static final int PALE = 0xC4C8CC;
	public static final int RED = 0x701010;

	private MistVeil() {
	}

	/** The veil's alpha for a mist of this kind grown to {@code weight} (0 to 1); 0 with no shader pack on, the game's fog doing it then. */
	public static float alpha(String kind, float weight, boolean shaders) {
		if (!shaders) {
			return 0;
		}
		float full = switch (kind) {
			case "fog_close" -> FOG_CLOSE;
			case "red_mist" -> RED_MIST;
			case "mist" -> MIST;
			default -> 0;
		};
		return full * Math.max(0, Math.min(1, weight));
	}

	/**
	 * How many banks of mist drift low over the ground this tick: only with a shader pack on, as
	 * its veil's body, the game's own fog showing the mist without one (the banks over the
	 * game's fog, too, were a whiteout and smoke over the ground, owner: "weird smoke/steam
	 * particles all over the ground").
	 */
	public static int banks(float weight, boolean shaders) {
		return shaders ? (int) (Math.max(0, Math.min(1, weight)) * 6) : 0;
	}

	/** The veil's colour for a mist of this kind. */
	public static int colour(String kind) {
		return kind.equals("red_mist") ? RED : PALE;
	}
}
