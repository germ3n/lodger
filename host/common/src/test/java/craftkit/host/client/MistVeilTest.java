package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MistVeilTest {
	@Test
	void a_veil_only_with_a_shader_pack_on_as_thick_as_the_mist_has_grown() {
		assertEquals(0f, MistVeil.alpha("mist", 1, false), "the game's fog does it without one");
		assertEquals(MistVeil.MIST, MistVeil.alpha("mist", 1, true), 1e-6);
		assertEquals(MistVeil.MIST / 2, MistVeil.alpha("mist", 0.5f, true), 1e-6, "grown in");
		assertTrue(MistVeil.alpha("fog_close", 1, true) > MistVeil.alpha("mist", 1, true), "the closing fog thicker");
		assertEquals(MistVeil.RED, MistVeil.colour("red_mist"));
		assertEquals(0f, MistVeil.alpha("rain", 1, true));
	}
}
