package craftkit.host.client.debug;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class PanelLayoutTest {
	@Test
	void the_panel_is_a_third_of_a_wide_screen_and_all_of_a_narrow_one() {
		assertEquals(640, PanelLayout.left(960), "a third of 960 is 320");
		assertEquals(170, PanelLayout.left(400), "never under 230 wide");
		assertEquals(0, PanelLayout.left(200), "never past the left edge");
		assertEquals(426, PanelLayout.units(960), "320 pixels at three quarters are 426 units");
	}

	@Test
	void a_click_is_read_in_the_panel_s_units() {
		assertEquals(0, PanelLayout.toPanel(640, 640));
		assertEquals(100, PanelLayout.toPanel(715, 640), 1e-9);
		assertEquals(40, PanelLayout.toPanel(30, 0), 1e-9, "y has no left");
	}
}
