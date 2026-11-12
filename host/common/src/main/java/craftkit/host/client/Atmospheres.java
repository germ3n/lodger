package craftkit.host.client;

import craftkit.host.net.HintPayload;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.fog.FogData;
import net.minecraft.client.renderer.state.level.SkyRenderState;
import net.minecraft.core.BlockPos;
import net.minecraft.core.GlobalPos;
import net.minecraft.util.ARGB;
import net.minecraft.world.level.MoonPhase;
import org.joml.Vector3f;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * What the hooks into the game's fog, sky, rain, compass and paintings ask while a hint is on
 * (the review's D and batch 11, owner): the atmospheres as real fog, eased and
 * stacked ({@link Atmosphere}), the stars gone, the moon full, the sky two hours on, the rain
 * red, a compass turned to a place, a painting upside down. Each hook is one line into here, so
 * a Minecraft update that moves a hook moves one line (docs/upgrading.md, The render hooks).
 */
public final class Atmospheres {
	private Atmospheres() {
	}

	private static float seconds(Hints.Active active, long now) {
		return (now - active.startedMillis()) / 1000f;
	}

	private static float total(Hints.Active active) {
		return (active.untilMillis() - active.startedMillis()) / 1000f;
	}

	/** The atmospheres on now, each with its weight and breath. */
	static List<Atmosphere.On> on() {
		long now = System.currentTimeMillis();
		List<Atmosphere.On> out = new ArrayList<>();
		for (Hints.Active active : Hints.active()) {
			Atmosphere.Fog fog = Atmosphere.of(active.hint().hint());
			if (fog != null) {
				float at = seconds(active, now);
				out.add(new Atmosphere.On(fog, Atmosphere.weight(at, total(active)), Atmosphere.breath(at)));
			}
		}

		return out;
	}

	/** How far in one kind of hint is, 0 to 1, eased, or 0 when it is not on. */
	static float weightOf(String hint) {
		long now = System.currentTimeMillis();
		float most = 0;
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals(hint)) {
				float at = seconds(active, now);
				most = Math.max(most, hint.equals("red_rain") ? Atmosphere.weightQuick(at, total(active)) : Atmosphere.weight(at, total(active)));
			}
		}

		return most;
	}

	/** The fog hook: the game's fog, closed and tinted by what is on. */
	public static void fog(FogData fog) {
		List<Atmosphere.On> on = on();
		if (on.isEmpty()) {
			return;
		}
		float[] start = Atmosphere.blend(fog.environmentalStart, fog.environmentalEnd, fog.color.x, fog.color.y, fog.color.z, on);
		float[] far = Atmosphere.blend(fog.renderDistanceStart, fog.renderDistanceEnd, fog.color.x, fog.color.y, fog.color.z, on);
		fog.environmentalStart = start[0];
		fog.environmentalEnd = start[1];
		fog.renderDistanceStart = far[0];
		fog.renderDistanceEnd = far[1];
		fog.skyEnd = Math.min(fog.skyEnd, start[1]);
		fog.cloudEnd = Math.min(fog.cloudEnd, start[1]);
		fog.color.set(start[2], start[3], start[4], fog.color.w);
	}

	/** The sky hook: the stars, the moon, the hour and the colour of the sky. */
	public static void sky(SkyRenderState sky) {
		float stars = weightOf("stars_out");
		if (stars > 0) {
			sky.starBrightness *= 1 - stars;
		}
		if (weightOf("moon_wrong") > 0) {
			sky.moonPhase = MoonPhase.FULL_MOON;
		}
		// The moon where the sun should be, at noon, the sun under the world (batch 16): from
		// the moment its minute is half in until it is half out.
		if (weightOf("wrong_sky") > 0.5f) {
			float sun = sky.sunAngle;
			sky.sunAngle = sky.moonAngle;
			sky.moonAngle = sun;
		}
		float later = weightOf("night_early");
		if (later > 0) {
			// Two hours of a twenty-four hour turn.
			float turn = (float) (Math.PI * 2 * 2 / 24) * later;
			sky.sunAngle += turn;
			sky.moonAngle += turn;
			sky.starAngle += turn;
		}
		List<Atmosphere.On> on = on();
		if (!on.isEmpty()) {
			float[] tinted = Atmosphere.blend(0, 1, sky.skyColor.x(), sky.skyColor.y(), sky.skyColor.z(), on);
			sky.skyColor = new Vector3f(tinted[2], tinted[3], tinted[4]);
		}
		if (later > 0) {
			sky.skyColor = new Vector3f(sky.skyColor.x() * (1 - 0.4f * later), sky.skyColor.y() * (1 - 0.4f * later), sky.skyColor.z() * (1 - 0.3f * later));
		}
	}

	/** The rain hook: a drop's colour, red while red rain is on, its alpha kept. */
	public static int rain(float alpha) {
		float red = weightOf("red_rain");
		if (red <= 0) {
			return ARGB.white(alpha);
		}
		int a = Math.round(Math.max(0, Math.min(1, alpha)) * 255);

		return ARGB.color(a, Math.round(255 - 115 * red), Math.round(255 - 245 * red), Math.round(255 - 245 * red));
	}

	/** The compass hook: where a compass points while `compass_points` is on, else what it would. */
	public static GlobalPos compass(GlobalPos would) {
		Minecraft minecraft = Minecraft.getInstance();
		if (minecraft.level == null) {
			return would;
		}
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("compass_points") && active.hint().pos().isPresent()) {
				return GlobalPos.of(minecraft.level.dimension(), active.hint().pos().get());
			}
		}

		return would;
	}

	/** Whether a painting at a place hangs upside down: `painting_flip` on there, until they have looked at it two seconds. */
	public static boolean flipped(double x, double y, double z) {
		for (Hints.Active active : Hints.active()) {
			HintPayload hint = active.hint();
			Optional<BlockPos> pos = hint.pos();
			if (hint.hint().equals("painting_flip") && pos.isPresent() && !Glitches.lookedLongEnough(active.id())
					&& pos.get().distToCenterSqr(x, y, z) <= 2.5) {
				return true;
			}
		}

		return false;
	}
}
