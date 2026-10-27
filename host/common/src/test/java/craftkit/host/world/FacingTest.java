package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class FacingTest {
	@Test
	void a_mob_turns_the_game_s_way_toward_what_it_faces() {
		assertEquals(0, Facing.yaw(0, 5), 1e-4, "+z is yaw 0");
		assertEquals(90, Math.floorMod(Math.round(Facing.yaw(-5, 0)), 360), "-x is yaw 90");
		assertEquals(270, Math.floorMod(Math.round(Facing.yaw(5, 0)), 360), "+x is yaw 270, or -90");
		assertEquals(180, Math.floorMod(Math.round(Facing.yaw(0, -5)), 360));
	}

	@Test
	void it_looks_up_at_a_player_above_and_down_at_one_below() {
		assertEquals(-45, Facing.pitch(3, 3, 0), 1e-4, "up is negative");
		assertEquals(45, Facing.pitch(0, -4, 4), 1e-4);
		assertEquals(0, Facing.pitch(10, 0, 0), 1e-4);
	}
}
