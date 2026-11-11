package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class JumpArcTest {
	@Test
	void a_jump_rises_to_a_block_and_a_quarter_and_lands() {
		assertEquals(0, JumpArc.height(0));
		assertEquals(JumpArc.TOP, JumpArc.height(JumpArc.AIRBORNE / 2), 1e-9);
		assertEquals(0, JumpArc.height(JumpArc.AIRBORNE));
		assertTrue(JumpArc.height(3) > 0 && JumpArc.height(3) < JumpArc.TOP);
		assertTrue(Figures.JUMP_EVERY > JumpArc.AIRBORNE, "it lands before the next");
	}
}
