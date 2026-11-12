package craftkit.host.client.debug;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class CoordinatesTest {
	@Test
	void the_first_spot_in_a_line_is_found_and_ground_is_the_surface() {
		Coordinates.Spot spot = Coordinates.find("#3 close_door at (-72,69,28)").orElseThrow();
		assertEquals(-72, spot.x());
		assertEquals(Optional.of(69), spot.y());
		assertEquals("[-72,69,28]", spot.json());

		Coordinates.Spot far = Coordinates.find("chose far_figure at (440,ground,-32) then (1,2,3)").orElseThrow();
		assertTrue(far.y().isEmpty());
		assertEquals("[440,null,-32]", far.json());
	}

	@Test
	void a_line_without_a_spot_has_none() {
		assertTrue(Coordinates.find("nothing planned").isEmpty());
		assertTrue(Coordinates.find("a day (1.5) later").isEmpty());
	}

	@Test
	void every_spot_in_a_line_is_found_where_it_is_written() {
		String line = "move_block from (1,64,2) to (3,ground,-4)";
		var found = Coordinates.findAll(line);
		assertEquals(2, found.size());
		assertEquals("(1,64,2)", line.substring(found.get(0).start(), found.get(0).end()));
		assertEquals("(3,ground,-4)", line.substring(found.get(1).start(), found.get(1).end()));
		assertTrue(found.get(1).spot().y().isEmpty());
		assertTrue(Coordinates.findAll("nothing here").isEmpty());
	}

	@Test
	void a_command_s_place_is_written_as_a_spot_the_panel_can_click() {
		assertEquals("(-73,64,2)", Coordinates.words(-72.5, 64.9, 2.1));
		assertEquals("(5,ground,6)", Coordinates.words(5, -10_000, 6));

		JsonObject setBlock = JsonParser.parseString("{\"kind\":\"set_block\",\"pos\":[10,-9999,-3]}").getAsJsonObject();
		assertEquals(" at (10,ground,-3)", Coordinates.placeOf(setBlock));
		JsonObject figure = JsonParser.parseString("{\"kind\":\"figure\",\"at\":[1.5,70.0,2.5],\"to\":[9,9,9]}").getAsJsonObject();
		assertEquals(" at (1,70,2)", Coordinates.placeOf(figure), "the first place it names");
		JsonObject hint = JsonParser.parseString("{\"kind\":\"hint\",\"player\":0}").getAsJsonObject();
		assertEquals("", Coordinates.placeOf(hint));
		// The written spot is found again, as the panel reads it.
		assertEquals(Optional.of(64), Coordinates.find("set_block" + Coordinates.placeOf(
				JsonParser.parseString("{\"pos\":[1,64,2]}").getAsJsonObject())).orElseThrow().y());
	}
}
