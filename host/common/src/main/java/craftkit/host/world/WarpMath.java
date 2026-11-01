package craftkit.host.world;

import java.util.Locale;
import java.util.Optional;

/**
 * The arithmetic of a warped animal, apart from the world so JUnit checks it: the tag that
 * carries where and how it was (so one saved while warped is put back when next seen), whether
 * a player looks at it, and the step it takes toward them while they do not.
 */
public final class WarpMath {
	/** The prefix of the tag that holds how a warped mob was. */
	public static final String FROM = "craftkit_warp_from:";
	/** Within this cone of a player's view, it is looked at: about 35 degrees off the middle. */
	public static final double LOOKED_AT = 0.82;
	/** How far one step closer goes. */
	public static final double STEP = 1.5;

	private WarpMath() {
	}

	/** How a mob was before it was warped. */
	public record Was(double x, double y, double z, float yaw, double scale, boolean noAi) {
	}

	public static String tag(Was was) {
		return FROM + String.format(Locale.ROOT, "%.3f,%.3f,%.3f,%.2f,%.4f,%b", was.x(), was.y(), was.z(), was.yaw(), was.scale(), was.noAi());
	}

	/** The tag read back, or empty for one that is not a warp's or is broken. */
	public static Optional<Was> read(String tag) {
		if (!tag.startsWith(FROM)) {
			return Optional.empty();
		}
		String[] parts = tag.substring(FROM.length()).split(",");
		if (parts.length != 6) {
			return Optional.empty();
		}
		try {
			return Optional.of(new Was(Double.parseDouble(parts[0]), Double.parseDouble(parts[1]), Double.parseDouble(parts[2]),
					Float.parseFloat(parts[3]), Double.parseDouble(parts[4]), Boolean.parseBoolean(parts[5])));
		} catch (NumberFormatException e) {
			return Optional.empty();
		}
	}

	/** Whether a view (a unit vector) points at something {@code to} away (any length). */
	public static boolean lookedAt(double vx, double vy, double vz, double tx, double ty, double tz) {
		double length = Math.sqrt(tx * tx + ty * ty + tz * tz);
		if (length < 1e-6) {
			return true;
		}

		return (vx * tx + vy * ty + vz * tz) / length >= LOOKED_AT;
	}

	/** Where one step from {@code (x, z)} toward {@code (px, pz)} lands, on the ground's plane. */
	public static double[] stepToward(double x, double z, double px, double pz) {
		double dx = px - x;
		double dz = pz - z;
		double length = Math.sqrt(dx * dx + dz * dz);
		if (length <= STEP) {
			return new double[] {x, z};
		}

		return new double[] {x + dx / length * STEP, z + dz / length * STEP};
	}
}
