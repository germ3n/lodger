package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SubtitlesTest {
	@Test
	void the_ghasts_cries_and_nothing_else() {
		for (String cry : new String[] {"entity.ghast.ambient", "entity.ghast.warn", "entity.ghast.scream", "entity.ghast.shoot", "entity.ghast.hurt"}) {
			assertTrue(Subtitles.renames("ghast_name", cry), cry);
		}
		assertFalse(Subtitles.renames("ghast_name", "entity.ghastling.ambient"), "not the little ones");
		assertFalse(Subtitles.renames("ghast_name", "entity.blaze.ambient"));
		assertFalse(Subtitles.renames("subtitle", "entity.ghast.ambient"), "another hint renames nothing");
	}
}
