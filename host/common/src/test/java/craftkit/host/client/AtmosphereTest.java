package craftkit.host.client;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class AtmosphereTest {
	@Test
	void it_eases_in_over_most_of_a_minute_holds_and_eases_out() {
		float total = 240;
		assertEquals(0, Atmosphere.weight(0, total));
		float early = Atmosphere.weight(10, total);
		float later = Atmosphere.weight(40, total);
		assertTrue(early > 0 && early < 0.2f, "barely there at ten seconds: " + early);
		assertTrue(later > early && later < 1, "still coming at forty: " + later);
		assertEquals(1, Atmosphere.weight(120, total), "held");
		assertTrue(Atmosphere.weight(225, total) < 0.5f, "going in the last half minute");
		assertEquals(0, Atmosphere.weight(240, total));
	}

	@Test
	void a_short_one_eases_in_proportion() {
		// The old fog_close, a minute long: in over 18 seconds, out over 15.
		assertTrue(Atmosphere.weight(9, 60) > 0.4f && Atmosphere.weight(9, 60) < 0.6f);
		assertEquals(1, Atmosphere.weight(30, 60));
	}

	@Test
	void the_fog_closes_to_the_nearest_and_takes_each_colour_in_turn() {
		float[] none = Atmosphere.blend(10, 200, 0.6f, 0.7f, 0.9f, List.of());
		assertEquals(200, none[1]);

		float[] mist = Atmosphere.blend(10, 200, 0.6f, 0.7f, 0.9f, List.of(new Atmosphere.On(Atmosphere.MIST, 1, 1)));
		assertEquals(Atmosphere.MIST.end(), mist[1], 1e-4);
		assertEquals(Atmosphere.MIST.r(), mist[2], 1e-4);

		float[] both = Atmosphere.blend(10, 200, 0.6f, 0.7f, 0.9f,
				List.of(new Atmosphere.On(Atmosphere.MIST, 1, 1), new Atmosphere.On(Atmosphere.DARKNESS, 0.5f, 1)));
		assertTrue(both[1] < Atmosphere.MIST.end(), "darkness closes it further: " + both[1]);
		assertTrue(both[2] < Atmosphere.MIST.r(), "and darker: " + both[2]);
		assertTrue(both[0] <= both[1], "the start never past the end");
	}

	@Test
	void every_atmosphere_has_its_fog_and_fog_close_is_mist_now() {
		for (String hint : new String[] {"mist", "red_mist", "darkness", "dusk_noon"}) {
			assertTrue(Atmosphere.of(hint) != null, hint);
		}
		assertEquals(Atmosphere.MIST, Atmosphere.of("fog_close"));
		assertNull(Atmosphere.of("red_rain"), "red rain is the drops, not a fog");
	}

	@org.junit.jupiter.api.Test
	void red_rain_is_seen_within_seconds_where_a_mist_takes_most_of_a_minute() {
		org.junit.jupiter.api.Assertions.assertEquals(1f, Atmosphere.weightQuick(Atmosphere.QUICK_IN, 150), 1e-6);
		org.junit.jupiter.api.Assertions.assertTrue(Atmosphere.weight(Atmosphere.QUICK_IN, 150) < 0.2f, "the slow way still slow");
		org.junit.jupiter.api.Assertions.assertEquals(0f, Atmosphere.weightQuick(150, 150), 1e-6, "gone at its end");
	}
}

