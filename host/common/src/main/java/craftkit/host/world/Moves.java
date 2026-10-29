package craftkit.host.world;

import java.util.ArrayList;
import java.util.List;

/** Where a moved block may go, as offsets from where it was asked to go: pure, for JUnit. */
public final class Moves {
	private Moves() {
	}

	/** {0,0,0}, then ring by ring out to {@code around}, each ring at its level, one up, one down. */
	public static List<int[]> offsets(int around) {
		List<int[]> out = new ArrayList<>();
		out.add(new int[] {0, 0, 0});
		for (int r = 1; r <= around; r++) {
			for (int dy : new int[] {0, 1, -1}) {
				for (int dx = -r; dx <= r; dx++) {
					for (int dz = -r; dz <= r; dz++) {
						if (Math.max(Math.abs(dx), Math.abs(dz)) == r) {
							out.add(new int[] {dx, dy, dz});
						}
					}
				}
			}
		}

		return out;
	}
}
