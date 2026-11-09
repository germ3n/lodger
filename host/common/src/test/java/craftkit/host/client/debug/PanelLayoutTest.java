package craftkit.host.client.debug;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class PanelLayoutTest {
	@Test
	void the_panel_is_a_third_of_a_wide_screen_and_all_of_a_narrow_one() {
		assertEquals(640, PanelLayout.left(960), "a third of 960 is 320");
		assertEquals(170, PanelLayout.left(400), "never under 230 wide");
		assertEquals(0, PanelLayout.left(200), "never past the left edge");
		assertEquals(533, PanelLayout.units(960), "320 pixels at 0.6 are 533 units");
	}

	@Test
	void a_click_is_read_in_the_panel_s_units() {
		assertEquals(0, PanelLayout.toPanel(640, 640));
		assertEquals(100, PanelLayout.toPanel(700, 640), 1e-4);
		assertEquals(50, PanelLayout.toPanel(30, 0), 1e-4, "y has no left");
	}
}
