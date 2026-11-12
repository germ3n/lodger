package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MenuFreakTest {
	@Test
	void the_buttons_shake_hardest_at_first_settle_and_one_reads_wrong_a_moment() {
		int most = 0;
		boolean moved = false;
		for (long t = 0; t < 600; t += 16) {
			int[] s = MenuFreak.shake(t, 2);
			most = Math.max(most, Math.max(Math.abs(s[0]), Math.abs(s[1])));
			moved |= s[0] != 0 || s[1] != 0;
		}
		assertTrue(moved, "it shakes");
		assertTrue(most <= MenuFreak.MOST_SHAKE);
		assertArrayEquals(new int[] {0, 0}, MenuFreak.shake(MenuFreak.LASTS_MS, 2), "still once over");
		assertFalse(MenuFreak.wrong(0));
		assertTrue(MenuFreak.wrong(MenuFreak.WRONG_FROM_MS + 10));
		assertFalse(MenuFreak.wrong(MenuFreak.WRONG_FROM_MS + MenuFreak.WRONG_FOR_MS));
		assertTrue(MenuFreak.over(MenuFreak.LASTS_MS));
	}
}
