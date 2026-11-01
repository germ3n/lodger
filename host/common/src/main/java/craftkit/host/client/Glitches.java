package craftkit.host.client;

import craftkit.host.net.HintPayload;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.phys.Vec3;

import java.util.HashSet;
import java.util.Set;

/**
 * What each hint kind does on the client, for the target alone, with nothing in the world
 * changed ({@code docs/entity.md}, The sky and the light, The screen, Sounds). The loaders call
 * {@link #tick} once a client tick and {@link #render} once a frame over the HUD; the vanilla
 * crosshair layer asks {@link #hideCrosshair}.
 *
 * <ul>
 *   <li>{@code cut_music}: the music stops and stays stopped.</li>
 *   <li>{@code silence}: ambient, weather and hostile sounds are stopped as they start.</li>
 *   <li>{@code flicker}: the screen darkens in flickers for a few seconds.</li>
 *   <li>{@code lights_fail}: the screen goes dark, then back.</li>
 *   <li>{@code sun_blink}: black for a blink, twice.</li>
 *   <li>{@code lightning_silent}: a white flash, no thunder.</li>
 *   <li>{@code red_rain}: a red cast while it lasts.</li>
 *   <li>{@code crosshair_gone}: no crosshair while it lasts.</li>
 *   <li>{@code hotbar_slip}: the selected slot jumps to the item named in {@code detail}, once.</li>
 *   <li>{@code camera_nudge}: the view turns two degrees toward the hint's place, once.</li>
 *   <li>{@code fov_pull}: the field of view narrows over the hint's time and snaps back.</li>
 *   <li>{@code fog_close}: a dark ring closes in from the edges of the screen for a minute.</li>
 *   <li>{@code snow_indoors}, {@code rain_indoors}: snow or rain particles fall around the player.</li>
 * </ul>
 */
public final class Glitches {
	private static final Set<Long> DONE_ONCE = new HashSet<>();
	private static Integer fovBefore;
	private static long lastSeen;

	private Glitches() {
	}

	/** Once a client tick, in a world. */
	public static void tick(Minecraft minecraft) {
		if (minecraft.player == null || minecraft.level == null) {
			if (fovBefore != null) {
				minecraft.options.fov().set(fovBefore);
				fovBefore = null;
			}
			Hints.clear();
			DONE_ONCE.clear();
			return;
		}

		boolean pullingFov = false;
		long now = System.currentTimeMillis();
		for (Hints.Active active : Hints.active()) {
			HintPayload hint = active.hint();
			switch (hint.hint()) {
				case "cut_music" -> minecraft.getMusicManager().stopPlaying();
				// One of their advancements, its toast again, as if just earned.
				case "advancement_toast" -> once(active, () -> {
					var holder = minecraft.getConnection() == null ? null
							: minecraft.getConnection().getAdvancements().get(net.minecraft.resources.Identifier.parse(hint.detail()));
					if (holder != null) {
						minecraft.gui.toastManager().addToast(new net.minecraft.client.gui.components.toasts.AdvancementToast(holder));
					}
				});
				case "silence" -> {
					minecraft.getSoundManager().stop(null, SoundSource.AMBIENT);
					minecraft.getSoundManager().stop(null, SoundSource.WEATHER);
					minecraft.getSoundManager().stop(null, SoundSource.HOSTILE);
				}
				case "hotbar_slip" -> once(active, () -> {
					var inventory = minecraft.player.getInventory();
					for (int slot = 0; slot < 9; slot++) {
						if (!inventory.getItem(slot).isEmpty() && craftkit.host.senses.Senses.itemId(inventory.getItem(slot)).equals(hint.detail())) {
							inventory.setSelectedSlot(slot);
							break;
						}
					}
				});
				case "camera_nudge" -> once(active, () -> hint.pos().ifPresent(pos -> {
					Vec3 to = Vec3.atCenterOf(pos).subtract(minecraft.player.position());
					float wanted = (float) Math.toDegrees(Math.atan2(-to.x, to.z));
					float delta = wrap(wanted - minecraft.player.getYRot());
					minecraft.player.setYRot(minecraft.player.getYRot() + Math.signum(delta) * Math.min(2, Math.abs(delta)));
				}));
				case "snow_indoors", "rain_indoors" -> {
					var random = minecraft.level.getRandom();
					var particle = hint.hint().equals("snow_indoors")
							? net.minecraft.core.particles.ParticleTypes.SNOWFLAKE
							: net.minecraft.core.particles.ParticleTypes.FALLING_WATER;
					for (int i = 0; i < 6; i++) {
						minecraft.level.addParticle(particle,
								minecraft.player.getX() + random.nextDouble() * 6 - 3,
								minecraft.player.getY() + 2.5 + random.nextDouble(),
								minecraft.player.getZ() + random.nextDouble() * 6 - 3,
								0, -0.05, 0);
					}
				}
				case "fov_pull" -> {
					pullingFov = true;
					if (fovBefore == null) {
						fovBefore = minecraft.options.fov().get();
					}
					int narrowed = (int) (fovBefore - 10 * active.progress(now));
					minecraft.options.fov().set(Math.max(30, narrowed));
				}
				default -> {
				}
			}
		}

		if (!pullingFov && fovBefore != null) {
			minecraft.options.fov().set(fovBefore);
			fovBefore = null;
		}

		// A one-shot's id is dropped once its hint is gone, so the set stays small.
		if (now - lastSeen > 10_000) {
			lastSeen = now;
			Set<Long> live = new HashSet<>();
			Hints.active().forEach(a -> live.add(a.id()));
			DONE_ONCE.retainAll(live);
		}
	}

	/** Once a frame, over the HUD: the screen glitches as translucent fills. */
	public static void render(GuiGraphicsExtractor graphics, int width, int height) {
		long now = System.currentTimeMillis();
		for (Hints.Active active : Hints.active()) {
			float progress = active.progress(now);
			switch (active.hint().hint()) {
				case "flicker" -> {
					// Dark for a few frames at a time, at an irregular beat.
					long phase = (now / 40) % 7;
					if (phase == 0 || phase == 3 || (phase == 5 && (now / 40) % 3 == 0)) {
						fill(graphics, width, height, 0x000000, 0.55f);
					}
				}
				case "lights_fail" -> {
					// Down fast, dark for most of it, back in the last fifth.
					float alpha = progress < 0.1f ? progress * 8.5f : progress > 0.8f ? (1 - progress) * 4.25f : 0.85f;
					fill(graphics, width, height, 0x050008, alpha);
				}
				case "sun_blink" -> {
					if ((progress > 0.1f && progress < 0.25f) || (progress > 0.55f && progress < 0.65f)) {
						fill(graphics, width, height, 0x000000, 0.92f);
					}
				}
				case "lightning_silent" -> fill(graphics, width, height, 0xFFFFFF, progress < 0.3f ? 0.8f : 0.8f * (1 - progress));
				case "fog_close" -> {
					// A ring from the edges, darkest at the rim, deepest in the middle minute.
					float depth = Math.min(progress * 6, Math.min(1, (1 - progress) * 6));
					int rim = (int) (Math.min(width, height) * 0.35f * depth);
					int a = (int) (200 * depth);
					graphics.fillGradient(0, 0, width, rim, (a << 24) | 0x05050A, 0x05050A);
					graphics.fillGradient(0, height - rim, width, height, 0x05050A, (a << 24) | 0x05050A);
					graphics.fill(0, 0, rim / 2, height, ((a / 2) << 24) | 0x05050A);
					graphics.fill(width - rim / 2, 0, width, height, ((a / 2) << 24) | 0x05050A);
				}
				case "red_rain" -> {
					float alpha = Math.min(progress * 5, Math.min(1, (1 - progress) * 5)) * 0.22f;
					fill(graphics, width, height, 0x7A0A0A, alpha);
				}
				default -> {
				}
			}
		}
	}

	/** Whether the vanilla crosshair is to be left out this frame. */
	public static boolean hideCrosshair() {
		return Hints.showing("crosshair_gone");
	}

	private static void fill(GuiGraphicsExtractor graphics, int width, int height, int rgb, float alpha) {
		int a = Math.max(0, Math.min(255, (int) (alpha * 255)));
		graphics.fill(0, 0, width, height, (a << 24) | rgb);
	}

	private static void once(Hints.Active active, Runnable run) {
		if (DONE_ONCE.add(active.id())) {
			run.run();
		}
	}

	private static float wrap(float degrees) {
		degrees %= 360;
		if (degrees >= 180) {
			degrees -= 360;
		}
		if (degrees < -180) {
			degrees += 360;
		}

		return degrees;
	}
}
