package craftkit.host.world;

/**
 * A figure drawn to the light a player carries (`drawn_to_light`, ABI 54), apart from the game so
 * JUnit checks it: what it does next, by whether they hold a light and how far off they are.
 */
public final class Drawn {
	/** Within this along the ground, it stops walking and runs at them. */
	public static final double RUSH_FROM = 20;
	/** How fast it walks at the light, blocks a tick: a quick walk, short of their run. */
	public static final double PER_TICK = 0.2;
	/** Lost, how long it stands looking about before it is gone, ticks. */
	public static final int LOST_TICKS = 30;

	/** What it does. */
	public enum Next {
		/** Toward their light, a step this tick. */
		COMES,
		/** Near enough: it runs at them. */
		RUSHES,
		/** No light in their hands: it has lost them, for good. */
		LOST
	}

	private Drawn() {
	}

	/** What it does next: once lost, lost for good, whatever they hold after. */
	public static Next next(boolean lit, double away, boolean lost) {
		if (lost || !lit) {
			return Next.LOST;
		}

		return away <= RUSH_FROM ? Next.RUSHES : Next.COMES;
	}
}
