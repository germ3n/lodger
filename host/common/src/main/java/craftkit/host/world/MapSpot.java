package craftkit.host.world;

import java.util.Optional;

/** Where a place falls on a map, apart from the game so JUnit checks it ({@link MapMarks}). */
public final class MapSpot {
	private MapSpot() {
	}

	/**
	 * Where a place falls on a map, as the game places a marker (two to a pixel, from the map's
	 * middle); empty when it is off the map.
	 */
	public static Optional<byte[]> spot(int centerX, int centerZ, byte scale, double x, double z) {
		double size = 1 << scale;
		double dx = (x - centerX) / size;
		double dz = (z - centerZ) / size;
		if (Math.abs(dx) > 63 || Math.abs(dz) > 63) {
			return Optional.empty();
		}

		return Optional.of(new byte[] {(byte) (int) (dx * 2 + 0.5), (byte) (int) (dz * 2 + 0.5)});
	}
}
