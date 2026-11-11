package craftkit.host.client;

/**
 * How a warped entity's model goes wrong, apart from the game so JUnit checks it (owner: "more
 * like a stretching that makes sense, and like shaking"). One stretch of the whole body, tall and
 * thin or low and wide, its volume kept, grown in over a few seconds and breathing slowly; a
 * tremor through every part; a sharp twitch of the head now and then; and the head tilting over
 * to an angle a neck cannot hold. The same entity goes wrong the same way each time.
 */
public final class Warp {
	/** How long the stretch takes to grow in, milliseconds. */
	public static final long GROW_MS = 2500;
	/** How far the body stretches at full growth: tall to this, or low to {@link #LOW}. */
	public static final float TALL = 1.8f;
	public static final float LOW = 0.6f;
	/** The breath: how much the stretch swells and settles, and how slowly. */
	public static final float BREATH = 0.05f;
	public static final long BREATH_MS = 1900;
	/** The tremor's turn at most each way, radians. */
	public static final float TREMOR = 0.06f;
	/** The head's tilt at full growth, and a twitch's snap on top of it, radians. */
	public static final float TILT = 0.9f;
	public static final float TWITCH = 0.7f;
	/** A twitch every this many milliseconds or so, lasting {@link #TWITCH_MS}. */
	public static final long TWITCH_EVERY_MS = 2800;
	public static final long TWITCH_MS = 140;

	private Warp() {
	}

	/** How far grown in, zero to one, eased: slow to start, slow to settle. */
	public static float grown(long sinceMs) {
		float t = Math.max(0, Math.min(1, sinceMs / (float) GROW_MS));

		return t * t * (3 - 2 * t);
	}

	/** Whether this entity goes tall and thin (else low and wide), the same each time. */
	public static boolean tall(int id) {
		return unit(mix(id * 0xD6E8FEB86659FD93L)) < 0.6f;
	}

	/**
	 * The whole body's scale, {x and z, y}: stretched along its height by the growth and the
	 * breath, narrowed or widened across so its volume is kept (a body stretched, not a part
	 * broken off).
	 */
	public static float[] body(int id, long sinceMs, long nowMs) {
		return body(id, sinceMs, nowMs, 0);
	}

	/** {@link #body(int, long, long)} grown to {@code height} (0 for the entity's own way, {@link #tall}). */
	public static float[] body(int id, long sinceMs, long nowMs, float height) {
		float g = grown(sinceMs);
		float full = height > 0 ? height : tall(id) ? TALL : LOW;
		float breath = 1 + BREATH * g * (float) Math.sin(2 * Math.PI * (nowMs + id * 131L) / BREATH_MS);
		float y = (1 + (full - 1) * g) * breath;
		float across = (float) (1 / Math.sqrt(y));

		return new float[] {across, y};
	}

	/** The tremor of one part now, {x, y, z} turns: fast and uneven, never twice the same. */
	public static float[] tremor(int id, int part, long nowMs, long sinceMs) {
		float g = grown(sinceMs);
		long seed = mix(id * 0x9E3779B97F4A7C15L + part * 0xC2B2AE3D27D4EB4FL);
		float[] turn = new float[3];
		for (int axis = 0; axis < 3; axis++) {
			double phase = unit(mix(seed + axis)) * 2 * Math.PI;
			double fast = Math.sin(nowMs * 0.083 + phase);
			double uneven = Math.sin(nowMs * 0.029 + phase * 1.7);
			turn[axis] = (float) (TREMOR * g * fast * (0.55 + 0.45 * uneven));
		}

		return turn;
	}

	/** The head's turn about its forward axis now: the tilt grown in, and a twitch's snap when one comes. */
	public static float headTilt(int id, long nowMs, long sinceMs) {
		float g = grown(sinceMs);
		float side = unit(mix(id * 0x2545F4914F6CDD1DL)) < 0.5f ? -1 : 1;
		long offset = (long) (unit(mix(id * 0x94D049BB133111EBL)) * TWITCH_EVERY_MS);
		long into = Math.floorMod(nowMs + offset, TWITCH_EVERY_MS);
		float twitch = into < TWITCH_MS ? (float) Math.sin(Math.PI * into / TWITCH_MS) : 0;

		return side * g * (TILT + TWITCH * twitch);
	}

	private static long mix(long z) {
		z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
		z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
		return z ^ (z >>> 31);
	}

	private static float unit(long z) {
		return (z >>> 40) / (float) (1L << 24);
	}
}
