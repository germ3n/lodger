package craftkit.host.world;

/**
 * Which half a double chest is known by: pure, so JUnit checks it without the game. The lower x,
 * then the lower z, so the same half whichever of the two is clicked (the lodger kept the half
 * clicked, and a double chest was two records, each of half its slots; owner: "its saying my
 * chest holds nothing even tho theres stuff in there").
 */
public final class ChestHalves {
	private ChestHalves() {
	}

	/** Whether the half at (x, z) is the one a double chest is known by, against its other at (ox, oz). */
	public static boolean known(int x, int z, int ox, int oz) {
		return x < ox || (x == ox && z <= oz);
	}
}
