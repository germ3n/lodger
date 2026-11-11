package craftkit.host.world;

/** A player's jump as a height over the ticks since it left the ground, apart from the game so JUnit checks it. */
public final class JumpArc {
	/** The ticks it is off the ground: a player's jump lands in about twelve. */
	public static final long AIRBORNE = 12;
	/** Its top, a player's: a block and a quarter. */
	public static final double TOP = 1.25;

	private JumpArc() {
	}

	/** Its height this many ticks into the jump; nothing on the ground. */
	public static double height(long tick) {
		if (tick <= 0 || tick >= AIRBORNE) {
			return 0;
		}
		double t = (double) tick / AIRBORNE;

		return TOP * 4 * t * (1 - t);
	}
}
