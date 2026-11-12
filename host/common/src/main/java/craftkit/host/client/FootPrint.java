package craftkit.host.client;

import java.util.ArrayList;
import java.util.List;

/**
 * The footprints round the house (`footprints` hint, ABI 54), apart from the game so JUnit
 * checks it: the hint's detail read back, and the dots of one print: a sole and a heel, toes at
 * its front, pointing the way it walks. Each dot {x, z} from the print's middle.
 */
public final class FootPrint {
	/** A dot's spacing. */
	static final double DOT = 0.045;

	private FootPrint() {
	}

	/** One print: where, and the way it points (degrees, the game's yaw). */
	public record Print(double x, double y, double z, float yaw) {
	}

	/** The hint's detail, {@code x,y,z,yaw;...}, read back; a broken entry is skipped. */
	public static List<Print> read(String detail) {
		List<Print> prints = new ArrayList<>();
		for (String entry : detail.split(";")) {
			String[] parts = entry.split(",");
			if (parts.length != 4) {
				continue;
			}
			try {
				prints.add(new Print(Double.parseDouble(parts[0]), Double.parseDouble(parts[1]), Double.parseDouble(parts[2]), Float.parseFloat(parts[3])));
			} catch (NumberFormatException ignored) {
				// Not a print: skipped.
			}
		}
		return prints;
	}

	/** The dots of a print pointing {@code yaw}: the sole, the heel behind it, toes in front. */
	public static List<double[]> dots(float yaw) {
		double a = Math.toRadians(yaw);
		// The game's facing: yaw 0 points +z, 90 points -x.
		double fx = -Math.sin(a);
		double fz = Math.cos(a);
		double sx = fz;
		double sz = -fx;
		List<double[]> dots = new ArrayList<>();
		double[][] shape = {
			{0, -3}, {-1, -3}, {1, -3}, {0, -2}, {-1, -2}, {1, -2},
			{0, 0}, {-1, 0}, {1, 0}, {0, 1}, {-1, 1}, {1, 1}, {0, 2}, {-1, 2}, {1, 2},
			{-1.5, 3.5}, {-0.5, 3.8}, {0.5, 3.8}, {1.5, 3.4}};
		for (double[] d : shape) {
			dots.add(new double[] {(d[0] * sx + d[1] * fx) * DOT, (d[0] * sz + d[1] * fz) * DOT});
		}
		return dots;
	}
}
