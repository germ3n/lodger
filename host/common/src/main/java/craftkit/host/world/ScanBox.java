package craftkit.host.world;

/**
 * The blocks a scan reads: every {@code step}th block on each axis from the box's low corner
 * (ABI 23), so a wide box read sparsely stays under the host's limit, which counts the blocks
 * read, not the box. Pure, so JUnit checks it.
 */
public final class ScanBox {
	private ScanBox() {
	}

	/** The step as the host uses it: 0 and 1 are every block. */
	public static int step(int asked) {
		return Math.max(1, asked);
	}

	/** How many blocks a step reads along one axis from {@code a} to {@code b}, either way round. */
	public static long along(int a, int b, int step) {
		return Math.abs(b - a) / step(step) + 1;
	}

	/** How many blocks the scan reads. */
	public static long reads(int[] min, int[] max, int step) {
		return along(min[0], max[0], step) * along(min[1], max[1], step) * along(min[2], max[2], step);
	}

	/** The coordinates read along one axis, low to high. */
	public static int[] axis(int a, int b, int step) {
		int low = Math.min(a, b);
		int[] out = new int[(int) along(a, b, step)];
		for (int i = 0; i < out.length; i++) {
			out[i] = low + i * step(step);
		}

		return out;
	}
}
