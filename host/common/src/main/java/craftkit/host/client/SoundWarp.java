package craftkit.host.client;

/**
 * How every sound goes wrong while a `sound_warp` hint is on (owner: "sound distortion effects,
 * like DSP"), apart from the game so JUnit checks it: the four ways, read from the hint's detail,
 * and how much each one pitches a sound.
 */
public final class SoundWarp {
	/** The ways: slowed and deep, a long echo in a hall too big, muffled as through a wall, crunched and broken. */
	public enum Way {
		SLOW, HALL, MUFFLED, BROKEN
	}

	/** How far down a slowed sound is pitched, as a tape dragging. */
	public static final float SLOW_PITCH = 0.55f;

	private SoundWarp() {
	}

	/** The way a hint's detail names, or null for none. */
	public static Way of(String detail) {
		return switch (detail.trim()) {
			case "slow" -> Way.SLOW;
			case "hall" -> Way.HALL;
			case "muffled" -> Way.MUFFLED;
			case "broken" -> Way.BROKEN;
			default -> null;
		};
	}

	/** A sound's pitch under a way: lower when slowed, else as it was. */
	public static float pitch(Way way, float pitch) {
		return way == Way.SLOW ? pitch * SLOW_PITCH : pitch;
	}
}
