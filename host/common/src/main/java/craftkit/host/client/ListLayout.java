package craftkit.host.client;

import java.util.ArrayList;
import java.util.List;

/**
 * Where the rows of a scrolling list sit: a heading before each new group, rows a fixed height
 * apart, and the scroll kept between the top and the last row's showing. Pure, for JUnit.
 */
public final class ListLayout {
	public static final int ROW = 24;
	public static final int HEADING = 16;

	/** A row's place: its top in the list's own space, and whether a group heading comes first. */
	public record Place(int top, boolean heading) {
	}

	private ListLayout() {
	}

	/** One place for each row, given each row's group in order. */
	public static List<Place> places(List<String> groups) {
		List<Place> out = new ArrayList<>();
		int y = 0;
		String group = null;
		for (String g : groups) {
			boolean heading = !g.equals(group);
			if (heading) {
				group = g;
				y += HEADING;
			}
			out.add(new Place(y, heading));
			y += ROW;
		}

		return out;
	}

	/** The whole list's height. */
	public static int height(List<Place> places) {
		return places.isEmpty() ? 0 : places.get(places.size() - 1).top() + ROW;
	}

	/** A scroll kept from above the top and past the end. */
	public static int clamp(int scroll, int height, int view) {
		return Math.max(0, Math.min(scroll, Math.max(0, height - view)));
	}

	/** Whether a row of `size` at screen `y` shows wholly between `top` and `bottom`. */
	public static boolean shows(int y, int size, int top, int bottom) {
		return y >= top && y + size <= bottom;
	}
}
