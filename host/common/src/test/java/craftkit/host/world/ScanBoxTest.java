package craftkit.host.world;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

class ScanBoxTest {
	@Test
	void every_block_is_read_without_a_step() {
		assertEquals(17L * 9 * 17, ScanBox.reads(new int[] {-8, 61, -8}, new int[] {8, 69, 8}, 0));
		assertEquals(ScanBox.reads(new int[] {0, 0, 0}, new int[] {4, 4, 4}, 1), ScanBox.reads(new int[] {0, 0, 0}, new int[] {4, 4, 4}, 0));
	}

	@Test
	void a_step_reads_every_nth_block_from_the_low_corner() {
		assertArrayEquals(new int[] {-6, -3, 0, 3, 6}, ScanBox.axis(-6, 7, 3));
		assertArrayEquals(new int[] {2, 4}, ScanBox.axis(5, 2, 2), "either way round");
		// A cave read: 97 by 41 by 97 read every third block is under the 40,000 limit.
		long reads = ScanBox.reads(new int[] {-48, 20, -48}, new int[] {48, 60, 48}, 3);
		assertEquals(33L * 14 * 33, reads);
		assertEquals(true, reads <= 40_000);
	}
}
