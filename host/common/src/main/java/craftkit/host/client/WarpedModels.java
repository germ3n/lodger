package craftkit.host.client;

import net.minecraft.client.model.Model;
import net.minecraft.client.model.geom.ModelPart;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.WeakHashMap;

/**
 * Warped animals and villagers, their own models bent (owner: "stretching the actual model";
 * then "more like a stretching that makes sense, and like shaking"): the server names them in a
 * `warp_models` hint, and as each of their models is posed for drawing it goes wrong as {@link
 * Warp} has it: the whole body stretched, its feet kept on the ground, every part trembling, the
 * head tilted over and twitching. It grows in from the hint's start. Nothing else is touched.
 */
public final class WarpedModels {
	private WarpedModels() {
	}

	/** Each render state's entity, as it was taken from it this frame. */
	private static final Map<Object, Integer> IDS = new WeakHashMap<>();

	public static void remember(Object state, int id) {
		IDS.put(state, id);
	}

	/** A warp as a hint names it: when it began, and the height it grows to (0: the entity's own way, {@link Warp#tall}). */
	private record Named(long started, float height) {
	}

	private static Map<Integer, Named> cached = Map.of();
	private static long cachedAt;

	/**
	 * The ids the active `warp_models` hints name (`id` or `id:height`), each with when its hint
	 * began, read again a few times a second, not every model.
	 */
	private static Map<Integer, Named> warped() {
		long now = System.currentTimeMillis();
		if (now - cachedAt < 250) {
			return cached;
		}
		cachedAt = now;
		Map<Integer, Named> ids = new HashMap<>();
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("warp_models")) {
				for (String id : active.hint().detail().split(",")) {
					try {
						String[] parts = id.trim().split(":");
						float height = parts.length > 1 ? Float.parseFloat(parts[1]) : 0;
						ids.merge(Integer.parseInt(parts[0]), new Named(active.startedMillis(), height),
								(a, b) -> a.started() <= b.started() ? a : b);
					} catch (NumberFormatException ignored) {
						// Not an id: skipped.
					}
				}
			}
		}
		cached = ids;
		return ids;
	}

	/** Model units from the root's pivot down to the feet: the game draws a body 24 of them tall. */
	private static final float FEET = 24f;

	/** After the model is posed: a warped entity bent. */
	public static void warp(Model<?> model, Object state) {
		Integer id = IDS.get(state);
		Named named = id == null ? null : warped().get(id);
		if (named == null) {
			return;
		}
		long now = System.currentTimeMillis();
		long since = now - named.started();
		List<ModelPart> parts = model.allParts();
		for (int i = 1; i < parts.size(); i++) {
			float[] t = Warp.tremor(id, i, now, since);
			ModelPart part = parts.get(i);
			part.xRot += t[0];
			part.yRot += t[1];
			part.zRot += t[2];
		}
		ModelPart root = model.root();
		if (root.hasChild("head")) {
			root.getChild("head").zRot += Warp.headTilt(id, now, since);
		}
		float[] body = Warp.body(id, since, now, named.height());
		root.xScale *= body[0];
		root.yScale *= body[1];
		root.zScale *= body[0];
		// Stretched from the root's pivot, over the body: moved so the feet stay where they stood.
		root.y += FEET * (1 - body[1]);
	}
}
