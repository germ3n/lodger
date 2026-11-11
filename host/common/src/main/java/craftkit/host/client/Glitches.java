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
 *   <li>{@code red_rain}: the drops red ({@link Atmospheres#rain}), no screen cast.</li>
 *   <li>{@code mist}, {@code red_mist}, {@code darkness}, {@code dusk_noon}: real fog ({@link Atmospheres#fog}),
 *       with drifting motes in the mists.</li>
 *   <li>{@code stars_out}, {@code moon_wrong}, {@code night_early}, {@code wrong_sky}: the sky ({@link Atmospheres#sky}).</li>
 *   <li>{@code compass_points}, {@code painting_flip}: a compass turned, a painting upside down
 *       until looked at two seconds.</li>
 *   <li>{@code hunger_shake}, {@code heart_skip}, {@code subtitle}: the HUD, once.</li>
 *   <li>{@code name_called}: the name in {@code detail} in the subtitles, from the hint's place, once.</li>
 *   <li>{@code ghast_name}: while it lasts, every ghast's sound reads in the subtitles as the name in {@code detail}.</li>
 *   <li>{@code crosshair_gone}: no crosshair while it lasts.</li>
 *   <li>{@code hotbar_slip}: the selected slot jumps to the item named in {@code detail}, once.</li>
 *   <li>{@code camera_nudge}: the view turns two degrees toward the hint's place, once.</li>
 *   <li>{@code fov_pull}: the field of view narrows over the hint's time and snaps back.</li>
 *   <li>{@code fog_close}: mist rolling in, the real fog, (it was a dark ring).</li>
 *   <li>{@code snow_indoors}, {@code rain_indoors}: snow or rain particles fall around the player.</li>
 *   <li>{@code eyes_open}: lying in bed, the game's fade to sleep held off while it lasts.</li>
 * </ul>
 */
public final class Glitches {
	private static final Set<Long> DONE_ONCE = new HashSet<>();
	/** Ticks each `painting_flip` has been looked at straight. */
	private static final java.util.Map<Long, Integer> LOOKED = new java.util.HashMap<>();
	/** Their saturation before `hunger_shake`, to put back. */
	private static Float saturationBefore;
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
			// Their saturation was the last world's player's: nothing to put back here.
			saturationBefore = null;
			Hints.clear();
			DONE_ONCE.clear();
			craftkit.host.client.debug.DebugHud.leave();
			return;
		}

		boolean pullingFov = false;
		boolean shaking = false;
		long now = System.currentTimeMillis();
		for (Hints.Active active : Hints.active()) {
			HintPayload hint = active.hint();
			switch (hint.hint()) {
				case "cut_music" -> minecraft.getMusicManager().stopPlaying();
				// The hunger bar shakes while saturation is nothing; it is put back after.
				case "hunger_shake" -> {
					shaking = true;
					if (saturationBefore == null) {
						saturationBefore = minecraft.player.getFoodData().getSaturationLevel();
					}
					minecraft.player.getFoodData().setSaturation(0);
				}
				// The hearts blink as they do when hurt, with no harm.
				case "heart_skip" -> once(active, () -> {
					try {
						var hud = minecraft.gui.hud;
						var ticks = net.minecraft.client.gui.Hud.class.getDeclaredField("tickCount");
						var blink = net.minecraft.client.gui.Hud.class.getDeclaredField("healthBlinkTime");
						ticks.setAccessible(true);
						blink.setAccessible(true);
						blink.setLong(hud, ticks.getInt(hud) + 20L);
					} catch (ReflectiveOperationException ignored) {
						// The HUD moved its fields: no blink, nothing else lost.
					}
				});
				// Lying in bed with their eyes open: the game's own fade to sleep kept at nothing,
				// so they see what comes to the bed (the host holds the night meanwhile).
				case "eyes_open" -> {
					if (minecraft.player.isSleeping()) {
						try {
							var counter = net.minecraft.world.entity.player.Player.class.getDeclaredField("sleepCounter");
							counter.setAccessible(true);
							counter.setInt(minecraft.player, 0);
						} catch (ReflectiveOperationException ignored) {
							// The field moved: the screen fades as it would, nothing else lost.
						}
					}
				}
				// "Footsteps" in the subtitles, behind them, and no sound.
				case "subtitle" -> once(active, () -> {
					var events = minecraft.getSoundManager().getSoundEvent(net.minecraft.resources.Identifier.parse("minecraft:block.stone.step"));
					if (events != null) {
						subtitle(minecraft, events, minecraft.player.position().subtract(minecraft.player.getViewVector(1f).scale(3)));
					}
				});
				// Their own name in the subtitles, from where the hint says, and no sound (batch 16).
				case "name_called" -> once(active, () -> {
					Vec3 at = hint.pos().map(Vec3::atCenterOf).orElse(minecraft.player.position());
					String name = hint.detail().isEmpty() ? minecraft.player.getName().getString() : hint.detail();
					subtitle(minecraft, new net.minecraft.client.sounds.WeighedSoundEvents(net.minecraft.resources.Identifier.parse("minecraft:entity.player.breath"), null) {
						@Override
						public net.minecraft.network.chat.Component getSubtitle() {
							return net.minecraft.network.chat.Component.literal(name);
						}
					}, at);
				});
				// A painting stays upside down until they have looked straight at it two seconds.
				case "painting_flip" -> hint.pos().ifPresent(pos -> {
					if (minecraft.crosshairPickEntity instanceof net.minecraft.world.entity.decoration.painting.Painting painting
							&& pos.distToCenterSqr(painting.position()) <= 2.5) {
						LOOKED.merge(active.id(), 1, Integer::sum);
					}
				});
				// The mists have body: motes drifting near the ground around them.
				case "mist", "fog_close", "red_mist" -> {
					float w = Atmospheres.weightOf(hint.hint());
					var random = minecraft.level.getRandom();
					int count = (int) (w * 3);
					for (int i = 0; i < count; i++) {
						double x = minecraft.player.getX() + random.nextDouble() * 20 - 10;
						double y = minecraft.player.getY() + random.nextDouble() * 2.5;
						double z = minecraft.player.getZ() + random.nextDouble() * 20 - 10;
						if (hint.hint().equals("red_mist")) {
							minecraft.level.addParticle(new net.minecraft.core.particles.DustParticleOptions(0x6E0A0A, 1.6f), x, y, z, 0.01, 0, 0.01);
						} else {
							minecraft.level.addParticle(net.minecraft.core.particles.ParticleTypes.WHITE_ASH, x, y, z, 0.02, 0, 0.02);
						}
					}
				}
				// Splashes crossing the water toward the shore they stand on, like steps (batch 10).
				case "ripples" -> hint.pos().ifPresent(pos -> {
					float t = active.progress(now);
					Vec3 from = Vec3.atCenterOf(pos);
					Vec3 toward = minecraft.player.position().subtract(from);
					Vec3 way = new Vec3(toward.x, 0, toward.z).normalize();
					Vec3 at = from.subtract(way.scale(10 * (1 - t)));
					if (minecraft.level.getGameTime() % 4 == 0) {
						for (int i = 0; i < 4; i++) {
							minecraft.level.addParticle(net.minecraft.core.particles.ParticleTypes.SPLASH,
									at.x + (minecraft.level.getRandom().nextDouble() - 0.5) * 0.6, pos.getY() + 1.0,
									at.z + (minecraft.level.getRandom().nextDouble() - 0.5) * 0.6, 0, 0.05, 0);
						}
					}
				});
				// A bobber's wake at one spot, no bobber, no one fishing (batch 10).
				case "bobber" -> hint.pos().ifPresent(pos -> {
					if (minecraft.level.getRandom().nextInt(6) == 0) {
						minecraft.level.addParticle(net.minecraft.core.particles.ParticleTypes.FISHING,
								pos.getX() + 0.5 + (minecraft.level.getRandom().nextDouble() - 0.5) * 0.3, pos.getY() + 1.0,
								pos.getZ() + 0.5 + (minecraft.level.getRandom().nextDouble() - 0.5) * 0.3, 0, 0, 0);
					}
					if (minecraft.level.getRandom().nextInt(80) == 0) {
						minecraft.level.addParticle(net.minecraft.core.particles.ParticleTypes.SPLASH, pos.getX() + 0.5, pos.getY() + 1.0, pos.getZ() + 0.5, 0, 0.1, 0);
					}
				});
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
				// Not while a screen is open: the options save the field of view as it stands, and
				// a narrowed one would be kept for good (until now it could be).
				case "fov_pull" -> {
					if (minecraft.gui.screen() != null) {
						break;
					}
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
		if (!shaking && saturationBefore != null) {
			minecraft.player.getFoodData().setSaturation(saturationBefore);
			saturationBefore = null;
		}

		// A one-shot's id is dropped once its hint is gone, so the set stays small.
		if (now - lastSeen > 10_000) {
			lastSeen = now;
			Set<Long> live = new HashSet<>();
			Hints.active().forEach(a -> live.add(a.id()));
			DONE_ONCE.retainAll(live);
			LOOKED.keySet().retainAll(live);
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
				// Dusk at noon drains the light from what fog cannot reach: a grey veil, eased.
				case "dusk_noon" -> fill(graphics, width, height, 0x14121A, 0.35f * Atmospheres.weightOf("dusk_noon"));
				default -> {
				}
			}
		}
	}

	/** Whether a `painting_flip` has been looked at straight for two seconds. */
	public static boolean lookedLongEnough(long id) {
		return LOOKED.getOrDefault(id, 0) >= 40;
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

	/**
	 * A sound's subtitle as the overlay shows it (the subtitle hook): while a `ghast_name` hint
	 * is on, a ghast's sound reads as the name in its detail; anything else as it was.
	 */
	public static net.minecraft.network.chat.Component subtitleOf(String soundPath, net.minecraft.network.chat.Component subtitle) {
		if (subtitle == null) {
			return null;
		}
		for (Hints.Active active : Hints.active()) {
			HintPayload hint = active.hint();
			if (Subtitles.renames(hint.hint(), soundPath) && !hint.detail().isEmpty()) {
				return net.minecraft.network.chat.Component.literal(hint.detail());
			}
		}

		return subtitle;
	}

	/** A line in the subtitles, as if a sound of `events` played at `at`, and no sound. */
	private static void subtitle(Minecraft minecraft, net.minecraft.client.sounds.WeighedSoundEvents events, Vec3 at) {
		try {
			var field = net.minecraft.client.gui.Hud.class.getDeclaredField("subtitleOverlay");
			field.setAccessible(true);
			var overlay = (net.minecraft.client.gui.components.SubtitleOverlay) field.get(minecraft.gui.hud);
			var instance = new net.minecraft.client.resources.sounds.SimpleSoundInstance(
					net.minecraft.sounds.SoundEvent.createVariableRangeEvent(net.minecraft.resources.Identifier.parse("minecraft:block.stone.step")),
					net.minecraft.sounds.SoundSource.BLOCKS, 1f, 1f, minecraft.level.getRandom(), at.x, at.y, at.z);
			overlay.onPlaySound(instance, events, 16f);
		} catch (ReflectiveOperationException ignored) {
			// The HUD moved its fields: no subtitle, nothing else lost.
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
