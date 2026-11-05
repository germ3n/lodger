package craftkit.host.client;

import net.minecraft.client.model.Model;
import net.minecraft.client.model.geom.ModelPart;

import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.WeakHashMap;

/**
 * The fight's warped animals, their own models bent (owner, 2026-11-10: "stretching the actual
 * model"): the server names them in a `warp_models` hint, and as each of their models is posed
 * for drawing every part is stretched or squashed and turned wrong, by its own fixed draw so it
 * never flickers, the whole body taller or lower. Nothing else is touched.
 */
public final class WarpedModels {
	private WarpedModels() {
	}

	/** Each render state's entity, as it was taken from it this frame. */
	private static final Map<Object, Integer> IDS = new WeakHashMap<>();

	public static void remember(Object state, int id) {
		IDS.put(state, id);
	}

	/** The ids the active `warp_models` hints name. */
	private static Set<Integer> warped() {
		Set<Integer> ids = new HashSet<>();
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("warp_models")) {
				for (String id : active.hint().detail().split(",")) {
					try {
						ids.add(Integer.parseInt(id.trim()));
					} catch (NumberFormatException ignored) {
						// Not an id: skipped.
					}
				}
			}
		}
		return ids;
	}

	/** After the model is posed: a warped entity's parts bent. */
	public static void warp(Model<?> model, Object state) {
		Integer id = IDS.get(state);
		if (id == null || !warped().contains(id)) {
			return;
		}
		List<ModelPart> parts = model.allParts();
		for (int i = 1; i < parts.size(); i++) {
			float[] w = Warp.of(id, i);
			ModelPart part = parts.get(i);
			part.xScale *= w[0];
			part.yScale *= w[1];
			part.zScale *= w[2];
			part.xRot += w[3];
			part.yRot += w[4];
			part.zRot += w[5];
		}
		if (!parts.isEmpty()) {
			float[] body = Warp.body(id);
			parts.getFirst().xScale *= body[0];
			parts.getFirst().yScale *= body[1];
			parts.getFirst().zScale *= body[0];
		}
	}
}
