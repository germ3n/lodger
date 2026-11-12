package craftkit.host.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SilhouetteTest {
	@Test
	void a_person_s_outline_head_body_arms_and_legs_with_a_gap_between() {
		assertTrue(Silhouette.inside(0, 1.65), "the head");
		assertTrue(Silhouette.inside(0, 1.1), "the body");
		assertTrue(Silhouette.inside(0.32, 1.0), "an arm");
		assertTrue(Silhouette.inside(0.12, 0.3), "a leg");
		assertFalse(Silhouette.inside(0, 0.3), "between the legs");
		assertFalse(Silhouette.inside(0.45, 1.65), "beside the head");
		assertFalse(Silhouette.inside(0, 2.0), "over the head");
		var points = Silhouette.points();
		assertTrue(points.size() > 100 && points.size() < 400, "dots enough to read it: " + points.size());
		assertTrue(points.stream().allMatch(p -> Math.abs(p[0]) <= 0.5 && p[1] >= 0 && p[1] <= 1.95));
	}
}
