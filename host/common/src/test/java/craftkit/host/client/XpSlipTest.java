package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class XpSlipTest {
	@Test
	void the_bar_empties_over_the_first_half_and_stays_empty() {
		assertEquals(XpSlip.WIDTH, XpSlip.kept(0));
		assertEquals(XpSlip.WIDTH / 2, XpSlip.kept(0.25f));
		assertEquals(0, XpSlip.kept(0.5f));
		assertEquals(0, XpSlip.kept(0.9f));
	}
}
