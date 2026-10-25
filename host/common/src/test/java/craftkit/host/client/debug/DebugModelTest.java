package craftkit.host.client.debug;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DebugModelTest {
	private static final String ANSWER = """
			{
			  "mods": ["lodger"],
			  "host": {
			    "index": 0, "tick": 400, "day_time": 14000, "day": 3, "raining": true, "thundering": false, "players": 1,
			    "looks": 2, "poses": 0, "later": 1,
			    "player": {
			      "pos": [1.5, 64.0, 2.5], "yaw": 90.0, "pitch": 0.0, "light": 4, "sky_light": 2, "health": 20, "hunger": 18,
			      "biome": "minecraft:plains", "dimension": "minecraft:overworld", "sneaking": true,
			      "main_hand": {"id": "minecraft:iron_pickaxe", "count": 1, "slot": 0},
			      "looking_at": {"kind": "block", "pos": [1, 64, 3], "block": "minecraft:chest"},
			      "movement": {"pace": 1.5, "stops": 2, "turns": 1, "look_backs": 3, "backing_away": 2.0, "sprint_bursts": 1, "still_seconds": 0.0}
			    },
			    "entities": ["3 cow", "1 item_frame"],
			    "events": ["399 opened_container at 1, 64, 3 minecraft:chest"],
			    "commands": ["lodger: toggle_block done"]
			  },
			  "mod": {
			    "message": "",
			    "gauges": [{"label": "stage", "value": 2, "max": 5, "text": "stage 2 of 5"}],
			    "sections": [{"title": "Plan", "lines": ["nothing planned"]}],
			    "events": [
			      {"id": "open_door", "words": "open the door", "stages": "1 to 5", "offered": false, "why": "the target is home and awake", "times": 1},
			      {"id": "knock", "words": "knock on the door", "stages": "2 to 5", "offered": true, "why": "", "times": 0}
			    ],
			    "actions": [{"id": "plan_now", "label": "Decide now"}]
			  }
			}
			""";

	@Test
	void the_host_panel_becomes_meters_and_lines() {
		DebugModel model = DebugModel.parse(ANSWER);

		assertEquals("", model.error);
		assertEquals(java.util.List.of("lodger"), model.mods);
		DebugModel.Gauge pace = model.senses.get(0);
		assertEquals("pace", pace.label());
		assertEquals(1.5f, pace.value());
		assertEquals(0.25f, pace.fraction());
		DebugModel.Gauge fear = model.senses.stream().filter(g -> g.label().equals("fear")).findFirst().orElseThrow();
		assertTrue(fear.value() > 0.7f, "three look backs and backing away read as afraid: " + fear.value());
		assertEquals("afraid", fear.text());
		assertTrue(model.senseLines.stream().anyMatch(l -> l.contains("indoors to the mind")), model.senseLines.toString());
		assertTrue(model.senseLines.stream().anyMatch(l -> l.contains("looking at minecraft:chest")), model.senseLines.toString());
		assertTrue(model.senseLines.stream().anyMatch(l -> l.startsWith("raining")), model.senseLines.toString());
		assertEquals(2, model.entities.size());
		assertEquals(1, model.sensedEvents.size());
		assertEquals(1, model.commands.size());
	}

	@Test
	void the_mod_reply_becomes_gauges_sections_events_and_actions_and_events_filter() {
		DebugModel model = DebugModel.parse(ANSWER);

		assertEquals("stage 2 of 5", model.gauges.get(0).text());
		assertEquals(0.4f, model.gauges.get(0).fraction());
		assertEquals("Plan", model.sections.get(0).title());
		assertEquals(2, model.events.size());
		assertFalse(model.events.get(0).offered());
		assertEquals("the target is home and awake", model.events.get(0).why());
		assertEquals(1, model.events.get(0).times());
		assertEquals("Decide now", model.actions.get(0).label());

		assertEquals(1, model.eventsMatching("knock").size());
		assertEquals(1, model.eventsMatching("awake").size(), "the why is searched too");
		assertEquals(2, model.eventsMatching("  ").size());
	}

	@Test
	void an_error_answer_and_a_broken_one_read_as_errors_not_exceptions() {
		assertEquals("operators only", DebugModel.parse("{\"error\":\"operators only\"}").error);
		assertTrue(DebugModel.parse("{ nope").error.startsWith("unreadable answer"));
		assertEquals(0, DebugModel.parse("{}").gauges.size());
		assertEquals(11, DebugModel.parse("{\"host\":{}}").senses.size(), "an empty host still has every meter, at zero");
	}

	@Test
	void fear_is_bounded_and_worded() {
		com.google.gson.JsonObject calm = new com.google.gson.JsonObject();
		assertEquals(0f, DebugModel.fear(calm));
		assertEquals("calm", DebugModel.fearWords(0f));
		com.google.gson.JsonObject panicked = new com.google.gson.JsonObject();
		panicked.addProperty("look_backs", 20);
		panicked.addProperty("backing_away", 60);
		assertEquals(1f, DebugModel.fear(panicked));
		assertEquals("watchful", DebugModel.fearWords(0.3f));
		assertEquals("uneasy", DebugModel.fearWords(0.5f));
	}
}
