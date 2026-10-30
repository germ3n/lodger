package craftkit.host.client;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ListLayoutTest {
	@Test
	void each_group_gets_a_heading_and_the_rows_follow_it() {
		List<ListLayout.Place> places = ListLayout.places(List.of("Model", "Model", "Pacing"));
		assertEquals(new ListLayout.Place(16, true), places.get(0));
		assertEquals(new ListLayout.Place(40, false), places.get(1));
		assertEquals(new ListLayout.Place(80, true), places.get(2), "a heading before the new group");
		assertEquals(104, ListLayout.height(places));
	}

	@Test
	void the_scroll_stays_in_range_and_a_row_shows_only_when_whole() {
		assertEquals(0, ListLayout.clamp(-30, 300, 200));
		assertEquals(100, ListLayout.clamp(500, 300, 200), "no further than the last row showing");
		assertEquals(0, ListLayout.clamp(40, 150, 200), "a short list does not scroll");
		assertTrue(ListLayout.shows(40, 20, 32, 200));
		assertFalse(ListLayout.shows(190, 20, 32, 200), "cut off at the bottom");
		assertFalse(ListLayout.shows(20, 20, 32, 200), "cut off at the top");
	}
}
