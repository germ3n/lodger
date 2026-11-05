package craftkit.host.client;

/** How one warped entity's part is bent, by a fixed draw from its id and the part's place, apart from the game so JUnit checks it. */
public final class Warp {
	private Warp() {
	}

	/** x, y, z scales and x, y, z turns (radians) for one part. */
	public static float[] of(int id, int part) {
		long seed = mix(id * 0x9E3779B97F4A7C15L + part * 0xC2B2AE3D27D4EB4FL);
		float a = unit(seed);
		float b = unit(mix(seed + 1));
		float c = unit(mix(seed + 2));
		float d = unit(mix(seed + 3));
		float e = unit(mix(seed + 4));
		float f = unit(mix(seed + 5));
		// Long or squat (0.55 to 2.4 along its length), thin or thick (0.6 to 1.5 across), and
		// turned up to 0.8 radians wrong each way.
		float along = 0.55f + 1.85f * a * a;
		float across = 0.6f + 0.9f * b;
		return new float[] {across, along, 0.6f + 0.9f * c, (d - 0.5f) * 1.6f, (e - 0.5f) * 1.6f, (f - 0.5f) * 1.6f};
	}

	/** The whole body: x and z, then y; tall and thin or low and wide. */
	public static float[] body(int id) {
		float t = unit(mix(id * 0xD6E8FEB86659FD93L));
		return t < 0.5f ? new float[] {0.75f, 1.45f} : new float[] {1.3f, 0.7f};
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
