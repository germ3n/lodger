package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class DrawnTest {
	@Test
	void it_comes_at_their_light_and_runs_at_them_near_enough() {
		assertEquals(Drawn.Next.COMES, Drawn.next(true, 45, false));
		assertEquals(Drawn.Next.RUSHES, Drawn.next(true, Drawn.RUSH_FROM, false));
	}

	@Test
	void the_light_put_away_it_has_lost_them_for_good() {
		assertEquals(Drawn.Next.LOST, Drawn.next(false, 45, false));
		assertEquals(Drawn.Next.LOST, Drawn.next(false, 5, false), "even near");
		assertEquals(Drawn.Next.LOST, Drawn.next(true, 45, true), "a light held again finds nothing");
	}
}
