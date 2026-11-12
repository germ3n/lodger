package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.UUID;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class RefusalsTest {
	@Test
	void one_feeding_refused_within_its_time_and_no_more() {
		UUID player = UUID.randomUUID();
		assertFalse(Refusals.take(player, 10), "none armed");
		Refusals.arm(player, 100);
		assertTrue(Refusals.take(player, 50));
		assertFalse(Refusals.take(player, 60), "once");
	}

	@Test
	void past_its_time_it_lapses() {
		UUID player = UUID.randomUUID();
		Refusals.arm(player, 100);
		assertFalse(Refusals.take(player, 101));
		assertFalse(Refusals.take(player, 50), "taken when lapsed");
	}
}
