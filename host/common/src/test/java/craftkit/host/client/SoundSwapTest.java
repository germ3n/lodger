package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class SoundSwapTest {
	@Test
	void a_named_sound_plays_as_the_other_and_nothing_else_is_touched() {
		String detail = "from=*.step;to=minecraft:block.wood.step";
		assertEquals("minecraft:block.wood.step", SoundSwap.swapped("block.grass.step", detail));
		assertEquals("minecraft:block.wood.step", SoundSwap.swapped("block.gravel.step", detail));
		assertNull(SoundSwap.swapped("block.wood.step", detail), "what it turns into is left alone");
		assertNull(SoundSwap.swapped("entity.zombie.step", detail), "a mob's steps are not theirs");
		assertNull(SoundSwap.swapped("block.grass.break", detail));
		assertEquals("minecraft:block.bell.use", SoundSwap.swapped("block.stone.break", "from=block.stone.break,block.deepslate.break;to=minecraft:block.bell.use"));
		assertNull(SoundSwap.swapped("block.grass.step", "to=minecraft:block.wood.step"), "nothing named, nothing swapped");
	}
}
