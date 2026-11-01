package craftkit.host.world;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

class DueTest {
	@Test
	void what_is_due_comes_out_once_in_order_and_the_rest_waits() {
		Due<String> due = new Due<>();
		due.add("first torch", 20);
		due.add("second torch", 35);
		due.add("third torch", 20);

		assertEquals(List.of(), due.due(19));
		assertEquals(List.of("first torch", "third torch"), due.due(20));
		assertEquals(List.of(), due.due(20), "once");
		assertEquals(1, due.size());
		assertEquals(List.of("second torch"), due.due(1_000));
	}

	@Test
	void a_closing_level_takes_out_its_own_whenever_they_are_due() {
		Due<String> due = new Due<>();
		due.add("overworld torch", 1_200);
		due.add("nether lantern", 1_200);

		assertEquals(List.of("overworld torch"), due.all(s -> s.startsWith("overworld")));
		assertEquals(1, due.size());
	}
}
