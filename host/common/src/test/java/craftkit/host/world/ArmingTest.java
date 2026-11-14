package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ArmingTest {
	@Test
	void the_first_break_within_the_time_is_the_one_and_then_it_is_spent() {
		UUID them = UUID.randomUUID();
		Map<UUID, Long> armed = new HashMap<>();
		armed.put(them, 100L);
		assertTrue(Arming.spend(armed, them, 50));
		assertFalse(Arming.spend(armed, them, 60), "spent");
		armed.put(them, 100L);
		assertFalse(Arming.spend(armed, them, 101), "past its time");
		assertFalse(armed.containsKey(them));
		assertFalse(Arming.spend(armed, UUID.randomUUID(), 10), "never armed");
	}
}
