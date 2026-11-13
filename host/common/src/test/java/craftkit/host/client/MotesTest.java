package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MotesTest {
	@Test
	void a_detail_is_read_pair_by_pair_and_what_it_cannot_read_stays_as_it_was() {
		Motes.Spec spec = Motes.parse("p=minecraft:campfire_cosy_smoke;n=2.5;r=1;h=3;v=0,0.05,0;every=4");
		assertEquals("minecraft:campfire_cosy_smoke", spec.particle());
		assertEquals(2.5f, spec.perTick());
		assertEquals(1, spec.spread());
		assertEquals(3, spec.height());
		assertArrayEquals(new double[] {0, 0.05, 0}, spec.velocity());
		assertEquals(4, spec.every());
		assertNull(spec.to());

		Motes.Spec dust = Motes.parse("p=dust:7A1A1A:9;n=lots;v=1,2;junk");
		assertEquals("dust", dust.particle());
		assertEquals(0x7A1A1A, dust.colour());
		assertEquals(4f, dust.size(), "kept to a size the game draws");
		assertEquals(1f, dust.perTick(), "a count it cannot read is one");
		assertArrayEquals(new double[] {0, 0, 0}, dust.velocity(), "two numbers are no velocity");
	}

	@Test
	void the_centre_stands_at_the_place_moves_toward_another_or_keeps_to_the_player() {
		double[] from = {0, 64, 0};
		double[] player = {10, 70, -5};
		assertArrayEquals(new double[] {0, 66, 0}, Motes.centre(Motes.parse("o=0,2,0"), from, player, 0.5f));
		assertArrayEquals(new double[] {5, 64, 10}, Motes.centre(Motes.parse("to=10,64,20"), from, player, 0.5f));
		assertArrayEquals(new double[] {10, 64, 20}, Motes.centre(Motes.parse("to=10,64,20"), from, player, 3f), "never past where it goes");
		assertArrayEquals(new double[] {10, 64, -5}, Motes.centre(Motes.parse("at=player;o=0,-6,0"), from, player, 0.2f));
	}

	@Test
	void so_many_a_tick_with_the_fraction_as_a_chance_and_none_between_beats() {
		Motes.Spec spec = Motes.parse("n=2.5;every=2");
		assertEquals(0, Motes.count(spec, 1, 0.0));
		assertEquals(3, Motes.count(spec, 2, 0.1));
		assertEquals(2, Motes.count(spec, 2, 0.9));
		assertTrue(Motes.count(Motes.parse("n=0.2"), 0, 0.5) == 0);
	}
}
