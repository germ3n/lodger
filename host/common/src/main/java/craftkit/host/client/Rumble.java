package craftkit.host.client;

/**
 * The view's shake in a `rumble`: pure, so JUnit checks it without the game. Hard at first and
 * dying away to nothing at its end, never more than {@link #MOST} degrees from where they look,
 * and every tick's offset is taken back the next, so their aim is where it was after.
 */
public final class Rumble {
	/** The most the view is turned, in degrees, at the rumble's start. */
	public static final float MOST = 1.6f;

	private Rumble() {
	}

	/** The offset from their own yaw and pitch at a point of the rumble (0 to 1) on a tick. */
	public static float[] offset(float progress, long tick) {
		float left = Math.max(0f, 1f - progress);
		float amplitude = MOST * left * left;
		return new float[] {
				(float) (Math.sin(tick * 2.3) * amplitude),
				(float) (Math.cos(tick * 3.1) * amplitude * 0.6),
		};
	}
}
