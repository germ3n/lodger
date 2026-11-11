package craftkit.host.world;

/**
 * Which side of a door is out, for a scan's {@code outs}: the side along its facing open to the
 * sky, the other roofed. Game-free, so it is tested on its own.
 */
public final class DoorOut {
	/** How many blocks out from the door each side is looked at for the sky: a porch roofs one. */
	public static final int LOOK = 3;

	private DoorOut() {
	}

	/**
	 * The side out: 1 for the door's facing, -1 for the opposite, 0 when both or neither is open,
	 * which says nothing about which is the house.
	 */
	public static int side(boolean facingOpen, boolean oppositeOpen) {
		if (facingOpen == oppositeOpen) {
			return 0;
		}
		return facingOpen ? 1 : -1;
	}
}
