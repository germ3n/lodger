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
 *   <li>{@code eyes_in_the_dark}: two small lights side by side at the hint's place, blinking, at
 *       eye height; gone for good once they come within five blocks.</li>
 *   <li>{@code hearts_drop}: the right half of their hearts drawn empty while it lasts, then as
 *       they are; no harm.</li>
 *   <li>{@code xp_slips}: their experience bar empties from the right over the first half of the
 *       hint, stays empty, then is as it was: the empty bar drawn over it, nothing lost.</li>
 *   <li>{@code f3_line}: one more line on the debug screen while it lasts, "Lodger: watching"
 *       ({@code DebugEntryPositionMixin}).</li>
 *   <li>{@code only_in_third_person}: in third person (F5) a figure in their skin right behind
 *       them while it lasts, in first person nothing ({@link BehindFigure}).</li>
 *   <li>{@code in_the_screenshot}: the next screenshot they take while it lasts has a figure in
 *       their skin far ahead in it ({@link ScreenshotFigure}), once.</li>
 *   <li>{@code handprints}: two hands of pale dust on the glass at the hint's place, facing
 *       them, renewed while it lasts ({@link HandPrint}).</li>
 *   <li>{@code false_death}: the death screen over the game while it lasts (a second), its red
 *       wash, "You Died!" twice the size and their score, no buttons; then the game as it was.</li>
 *   <li>{@code inventory_stare}: in the inventory, their figure stares straight out instead of
 *       following the mouse, while it lasts ({@code InventoryScreenMixin}).</li>
 *   <li>{@code rumble}: a deep boom under them, the view shaking and dying away ({@link Rumble}),
 *       dust falling from the ceiling over them indoors.</li>
 *   <li>{@code red_trail}: red specks on the floor from the hint's place to the one in its
 *       detail (the door to the bed), renewed while it lasts.</li>
 *   <li>{@code cold_breath}: their breath showing in puffs before their face.</li>
 *   <li>{@code clock_wrong}: a clock shows the time half a day off ({@code ClockTimeMixin}).</li>
 *   <li>{@code phantom_item}: the first time they open their inventory while it lasts, a paper
 *       named "behind you" in an empty slot for three seconds ({@code InventoryScreenMixin}).</li>
 *   <li>{@code rustle_in_the_pack}: the first time they open their inventory while it lasts,
 *       a bundle's rustle, once, though they carry none ({@code InventoryScreenMixin}).</li>
 *   <li>{@code border_closes}: the world border on their client drawn close round them, then
 *       put back as it was.</li>
 *   <li>{@code shadow_on_the_wall}: while they hold a light indoors, a person's shadow on the
 *       wall behind them ({@link Silhouette}), gone for good once they turn to it.</li>
 *   <li>{@code nightmare}: one frame of their own face, the figure's, filling the screen.</li>
 *   <li>{@code static}: a second of grey noise over the screen and a hiss.</li>
 *   <li>{@code elder_face}: the elder guardian's face over the screen and its cry, as the game
 *       shows it, no curse.</li>
 *   <li>{@code boom_far}: an explosion at the hint's place, far off: the burst seen there, the
 *       boom from that way, a flash on the screen.</li>
 * </ul>
 */
public final class Glitches {
	/** Each `footprints` hint's prints, read once. */
	private static final java.util.Map<Long, java.util.List<FootPrint.Print>> PRINTS = new java.util.HashMap<>();

	/** The top of the ground at a column near {@code y}: the highest block with something to stand on and air over it, three up to four down; NaN for none. */
	private static double groundAt(net.minecraft.client.multiplayer.ClientLevel level, double x, double y, double z) {
		int bx = net.minecraft.util.Mth.floor(x);
		int bz = net.minecraft.util.Mth.floor(z);
		for (int by = net.minecraft.util.Mth.floor(y) + 3; by >= net.minecraft.util.Mth.floor(y) - 4; by--) {
			net.minecraft.core.BlockPos at = new net.minecraft.core.BlockPos(bx, by, bz);
			if (!level.getBlockState(at).getCollisionShape(level, at).isEmpty() && level.getBlockState(at.above()).getCollisionShape(level, at.above()).isEmpty()) {
				return level.getBlockState(at).getCollisionShape(level, at).max(net.minecraft.core.Direction.Axis.Y) + by;
			}
		}
		return Double.NaN;
	}

	private static final Set<Long> DONE_ONCE = new HashSet<>();
	/** Ticks each `painting_flip` has been looked at straight. */
	private static final java.util.Map<Long, Integer> LOOKED = new java.util.HashMap<>();
	/** Their saturation before `hunger_shake`, to put back. */
	private static Float saturationBefore;
	private static Integer fovBefore;
	private static long lastSeen;
	/** The rumble's offset on their view now, taken back as it changes and when it ends. */
	private static float shakenYaw;
	private static float shakenPitch;
	/** The world border as it was before `border_closes` drew it close, to put back. */
	private static double[] borderBefore;
	/** When each `phantom_item` was first seen in the inventory. */
	private static final java.util.Map<Long, Long> PHANTOM_SHOWN = new java.util.HashMap<>();

	private Glitches() {
	}

	/** Whether the debug screen has its one more line (`f3_line`). */
	public static boolean f3Line() {
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("f3_line")) {
				return true;
			}
		}

		return false;
	}

	/** Whether their next screenshot is to have the figure in it (`in_the_screenshot`). */
	public static boolean screenshotArmed() {
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("in_the_screenshot") && !DONE_ONCE.contains(active.id())) {
				return true;
			}
		}

		return false;
	}

	/** The screenshot with the figure in it taken: once a hint. */
	public static void screenshotTaken() {
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("in_the_screenshot")) {
				DONE_ONCE.add(active.id());
			}
		}
	}

	/** Whether their figure in the inventory stares straight out (`inventory_stare`). */
	public static boolean inventoryStare() {
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("inventory_stare")) {
				return true;
			}
		}

		return false;
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

		ScreenshotFigure.tick(minecraft);
		boolean behind = false;
		boolean pullingFov = false;
		boolean shaking = false;
		boolean rumbling = false;
		boolean bordered = false;
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
				case "only_in_third_person" -> {
					behind = true;
					BehindFigure.tick(minecraft);
				}
				// Two glowing points a hand apart across the way they look, blinking now and then;
				// gone once they come close.
				case "eyes_in_the_dark" -> hint.pos().ifPresent(pos -> {
					var eye = minecraft.player.getEyePosition();
					double cx = pos.getX() + 0.5;
					double cy = pos.getY() + 0.6;
					double cz = pos.getZ() + 0.5;
					if (DONE_ONCE.contains(active.id()) || eye.distanceToSqr(cx, cy, cz) < 25) {
						DONE_ONCE.add(active.id());
						return;
					}
					long t = minecraft.level.getGameTime();
					if (t % 3 != 0 || t % 50 < 5) {
						return;
					}
					double dx = cx - eye.x;
					double dz = cz - eye.z;
					double length = Math.max(1e-6, Math.sqrt(dx * dx + dz * dz));
					double ax = -dz / length * 0.13;
					double az = dx / length * 0.13;
					for (int side = -1; side <= 1; side += 2) {
						minecraft.level.addParticle(net.minecraft.core.particles.ParticleTypes.GLOW, cx + ax * side, cy, cz + az * side, 0, 0, 0);
					}
				});
				// Footprints round the house, dark on the ground, renewed as the dust fades: those
				// within twenty-four blocks, each on the ground found under it.
				// Every eighth tick: never a `return` here, which left the whole tick, the hints
				// after it in the list and the restores at its end, seven ticks in eight.
				case "footprints" -> {
					if (minecraft.level.getGameTime() % 8 == 0) {
						drawPrints(minecraft, active, hint);
					}
				}
				// Two hands pressed on the glass from the house's side, renewed as the dust fades.
				case "handprints" -> hint.pos().ifPresent(pos -> {
					if (minecraft.level.getGameTime() % 6 != 0) {
						return;
					}
					var eye = minecraft.player.getEyePosition();
					double cx = pos.getX() + 0.5;
					double cz = pos.getZ() + 0.5;
					for (double[] p : HandPrint.points(cx - eye.x, cz - eye.z)) {
						minecraft.level.addParticle(new net.minecraft.core.particles.DustParticleOptions(0xE6E2DA, 0.35f),
								cx + p[0], pos.getY() + p[1], cz + p[2], 0, 0, 0);
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
					// Banks of it low over the ground, big soft drifts (seen with a shader pack on
					// too, which hides the game's fog), and the motes in it.
					int banks = MistVeil.banks(w, shadersOn());
					for (int i = 0; i < banks; i++) {
						double x = minecraft.player.getX() + random.nextDouble() * 28 - 14;
						double y = minecraft.player.getY() + random.nextDouble() * 1.6 - 0.3;
						double z = minecraft.player.getZ() + random.nextDouble() * 28 - 14;
						int colour = hint.hint().equals("red_mist") ? 0x7A1A1A : 0xD8DCE0;
						minecraft.level.addParticle(new net.minecraft.core.particles.DustParticleOptions(colour, 4.0f), x, y, z, 0.02, 0, 0.02);
					}
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
				// A deep boom under them, the view shaking, dust from the ceiling indoors.
				case "rumble" -> {
					rumbling = true;
					once(active, () -> minecraft.level.playLocalSound(minecraft.player.getX(), minecraft.player.getY() - 4, minecraft.player.getZ(),
							net.minecraft.sounds.SoundEvents.GENERIC_EXPLODE.value(), SoundSource.AMBIENT, 1.4f, 0.3f, false));
					float[] shake = Rumble.offset(active.progress(now), minecraft.level.getGameTime());
					minecraft.player.setYRot(minecraft.player.getYRot() - shakenYaw + shake[0]);
					minecraft.player.setXRot(net.minecraft.util.Mth.clamp(minecraft.player.getXRot() - shakenPitch + shake[1], -90f, 90f));
					shakenYaw = shake[0];
					shakenPitch = shake[1];
					rumbleDust(minecraft, active.progress(now));
				}
				// Far off, the burst seen where the hint is and the boom from that way.
				case "boom_far" -> once(active, () -> hint.pos().ifPresent(pos -> {
					Vec3 at = Vec3.atCenterOf(pos);
					Vec3 way = at.subtract(minecraft.player.position());
					Vec3 heard = minecraft.player.position().add(new Vec3(way.x, 0, way.z).normalize().scale(14));
					minecraft.level.playLocalSound(heard.x, heard.y, heard.z, net.minecraft.sounds.SoundEvents.GENERIC_EXPLODE.value(), SoundSource.AMBIENT, 1.6f, 0.55f, false);
					minecraft.level.addAlwaysVisibleParticle(net.minecraft.core.particles.ParticleTypes.EXPLOSION_EMITTER, true, at.x, at.y + 3, at.z, 0, 0, 0);
					for (int i = 0; i < 12; i++) {
						var random = minecraft.level.getRandom();
						minecraft.level.addAlwaysVisibleParticle(net.minecraft.core.particles.ParticleTypes.LARGE_SMOKE, true,
								at.x + random.nextDouble() * 4 - 2, at.y + 3 + random.nextDouble() * 3, at.z + random.nextDouble() * 4 - 2, 0, 0.1, 0);
					}
				}));
				// A person's shadow on the wall behind them, cast by the light they hold.
				case "shadow_on_the_wall" -> {
					if (!DONE_ONCE.contains(active.id()) && minecraft.level.getGameTime() % 3 == 0) {
						shadow(minecraft, active);
					}
				}
				// Red specks from the door to the bed, on the floor, every half second.
				case "red_trail" -> {
					if (minecraft.level.getGameTime() % 10 == 0) {
						hint.pos().ifPresent(from -> redTrail(minecraft, from, hint.detail()));
					}
				}
				// A puff of breath before their face every two seconds.
				case "cold_breath" -> {
					if (minecraft.level.getGameTime() % 40 == 0) {
						Vec3 look = minecraft.player.getViewVector(1f);
						Vec3 mouth = minecraft.player.getEyePosition().add(look.scale(0.4)).add(0, -0.1, 0);
						for (int i = 0; i < 3; i++) {
							minecraft.level.addParticle(net.minecraft.core.particles.ParticleTypes.WHITE_SMOKE, mouth.x, mouth.y, mouth.z,
									look.x * 0.03, 0.005, look.z * 0.03);
						}
					}
				}
				// The world's edge close round them, its wall and its red, then as it was.
				case "border_closes" -> {
					bordered = true;
					var border = minecraft.level.getWorldBorder();
					if (borderBefore == null) {
						borderBefore = new double[] {border.getCenterX(), border.getCenterZ(), border.getSize()};
						border.setCenter(minecraft.player.getX(), minecraft.player.getZ());
						border.setSize(14);
					}
				}
				case "static" -> once(active, () -> minecraft.level.playLocalSound(minecraft.player.getX(), minecraft.player.getEyeY(), minecraft.player.getZ(),
						net.minecraft.sounds.SoundEvents.FIRE_EXTINGUISH, SoundSource.AMBIENT, 0.6f, 0.5f, false));
				// The face and the cry the game shows for an elder guardian's curse; no curse.
				case "elder_face" -> once(active, () -> {
					minecraft.level.addParticle(net.minecraft.core.particles.ParticleTypes.ELDER_GUARDIAN,
							minecraft.player.getX(), minecraft.player.getY(), minecraft.player.getZ(), 0, 0, 0);
					minecraft.level.playLocalSound(minecraft.player.getX(), minecraft.player.getY(), minecraft.player.getZ(),
							net.minecraft.sounds.SoundEvents.ELDER_GUARDIAN_CURSE, SoundSource.HOSTILE, 1f, 1f, false);
				});
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

		if (!behind) {
			BehindFigure.clear(minecraft);
		}
		if (!bordered && borderBefore != null) {
			var border = minecraft.level.getWorldBorder();
			border.setCenter(borderBefore[0], borderBefore[1]);
			border.setSize(borderBefore[2]);
			borderBefore = null;
		}
		if (!rumbling && (shakenYaw != 0 || shakenPitch != 0)) {
			minecraft.player.setYRot(minecraft.player.getYRot() - shakenYaw);
			minecraft.player.setXRot(net.minecraft.util.Mth.clamp(minecraft.player.getXRot() - shakenPitch, -90f, 90f));
			shakenYaw = 0;
			shakenPitch = 0;
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
			PRINTS.keySet().retainAll(live);
			PHANTOM_SHOWN.keySet().retainAll(live);
		}
	}

	/** Once a frame, over the HUD: the screen glitches as translucent fills. */
	public static void render(GuiGraphicsExtractor graphics, int width, int height) {
		long now = System.currentTimeMillis();
		for (Hints.Active active : Hints.active()) {
			float progress = active.progress(now);
			switch (active.hint().hint()) {
				// The right five of their hearts drawn as empty containers over them; no harm.
				case "hearts_drop" -> {
					var player = Minecraft.getInstance().player;
					if (player == null || player.isCreative() || player.isSpectator()) {
						break;
					}
					int x = width / 2 - 91;
					int y = height - 39;
					for (int heart = 5; heart < 10; heart++) {
						graphics.blitSprite(net.minecraft.client.renderer.RenderPipelines.GUI_TEXTURED,
								net.minecraft.resources.Identifier.withDefaultNamespace("hud/heart/container"), x + heart * 8, y, 9, 9);
					}
				}
				// Their experience bar emptied on their screen: the empty bar over the right of it,
				// spreading left; their experience is untouched.
				case "xp_slips" -> {
					var player = Minecraft.getInstance().player;
					if (player == null || player.isCreative() || player.isSpectator()) {
						break;
					}
					int kept = XpSlip.kept(progress);
					int x = width / 2 - 91;
					int y = height - 29;
					graphics.blitSprite(net.minecraft.client.renderer.RenderPipelines.GUI_TEXTURED,
							net.minecraft.resources.Identifier.withDefaultNamespace("hud/experience_bar_background"),
							XpSlip.WIDTH, 5, kept, 0, x + kept, y, XpSlip.WIDTH - kept, 5);
				}
				// The death screen as the game draws it: its wash, its title twice the size, the
				// score; nothing died.
				case "false_death" -> {
					graphics.fillGradient(0, 0, width, height, 0x60500000, 0xA0803030);
					var font = Minecraft.getInstance().font;
					var pose = graphics.pose();
					pose.pushMatrix();
					pose.scale(2, 2);
					graphics.centeredText(font, net.minecraft.network.chat.Component.translatable("deathScreen.title"), width / 4, 30, 0xFFFFFFFF);
					pose.popMatrix();
					int score = Minecraft.getInstance().player == null ? 0 : Minecraft.getInstance().player.getScore();
					graphics.centeredText(font, net.minecraft.network.chat.Component.translatable("deathScreen.score.value",
							net.minecraft.network.chat.Component.literal(Integer.toString(score)).withStyle(net.minecraft.ChatFormatting.YELLOW)), width / 2, 100, 0xFFFFFFFF);
				}
				// With a shader pack on, which draws its own fog and leaves the game's unused, the
				// mists are a veil over the screen instead (`MistVeil`), as thick as they have grown.
				case "mist", "fog_close", "red_mist" -> {
					String kind = active.hint().hint();
					float alpha = MistVeil.alpha(kind, Atmospheres.weightOf(kind), shadersOn());
					if (alpha > 0) {
						fill(graphics, width, height, MistVeil.colour(kind), alpha);
					}
				}
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
				// Their own face, the figure's, over the whole screen for a frame or two.
				case "nightmare" -> {
					var player = Minecraft.getInstance().player;
					if (player != null) {
						fill(graphics, width, height, 0x000000, 1f);
						int size = Math.max(width, height);
						net.minecraft.client.gui.components.PlayerFaceExtractor.extractRenderState(graphics, player.getSkin(),
								(width - size) / 2, (height - size) / 2, size);
					}
				}
				// Grey noise, new each frame, in blocks of four.
				case "static" -> {
					var random = java.util.concurrent.ThreadLocalRandom.current();
					for (int y = 0; y < height; y += 4) {
						for (int x = 0; x < width; x += 4) {
							int grey = random.nextInt(256);
							graphics.fill(x, y, x + 4, y + 4, 0x90000000 | grey << 16 | grey << 8 | grey);
						}
					}
				}
				// The far burst's glow on the sky, a moment.
				case "boom_far" -> {
					if (progress < 0.12f) {
						fill(graphics, width, height, 0xFFB060, 0.18f * (1 - progress / 0.12f));
					}
				}
				// Dusk at noon drains the light from what fog cannot reach: a grey veil, eased.
				case "dusk_noon" -> fill(graphics, width, height, 0x14121A, 0.35f * Atmospheres.weightOf("dusk_noon"));
				default -> {
				}
			}
		}
	}

	/**
	 * The shadow behind them: on the first wall straight back from them, within eight blocks,
	 * dark dust in a person's outline facing them; only while they hold a light, and gone for
	 * good once they look round at it.
	 */
	private static void shadow(Minecraft minecraft, Hints.Active active) {
		var player = minecraft.player;
		String held = craftkit.host.senses.Senses.itemId(player.getMainHandItem()) + " " + craftkit.host.senses.Senses.itemId(player.getOffhandItem());
		if (!held.contains("torch") && !held.contains("lantern")) {
			return;
		}
		Vec3 look = player.getViewVector(1f);
		Vec3 back = new Vec3(-look.x, 0, -look.z).normalize();
		Vec3 feet = player.position();
		Vec3 wall = null;
		for (double d = 1; d <= 8; d += 0.25) {
			Vec3 at = feet.add(back.scale(d)).add(0, 1, 0);
			net.minecraft.core.BlockPos block = net.minecraft.core.BlockPos.containing(at);
			if (!minecraft.level.getBlockState(block).getCollisionShape(minecraft.level, block).isEmpty()) {
				wall = feet.add(back.scale(d - 0.3));
				break;
			}
		}
		if (wall == null) {
			return;
		}
		Vec3 toWall = wall.subtract(player.getEyePosition());
		if (look.dot(toWall.normalize()) > 0.3) {
			DONE_ONCE.add(active.id());
			return;
		}
		Vec3 across = new Vec3(-back.z, 0, back.x);
		for (double[] p : Silhouette.points()) {
			minecraft.level.addParticle(new net.minecraft.core.particles.DustParticleOptions(0x050505, 0.9f),
					wall.x + across.x * p[0], feet.y + p[1], wall.z + across.z * p[0], 0, 0, 0);
		}
	}

	/** Red specks on the floor along the way from `from` to the place in `to` ("x,y,z"). */
	private static void redTrail(Minecraft minecraft, net.minecraft.core.BlockPos from, String to) {
		String[] parts = to.split(",");
		if (parts.length != 3) {
			return;
		}
		Vec3 a = Vec3.atBottomCenterOf(from);
		Vec3 b;
		try {
			b = new Vec3(Integer.parseInt(parts[0]) + 0.5, Integer.parseInt(parts[1]), Integer.parseInt(parts[2]) + 0.5);
		} catch (NumberFormatException e) {
			return;
		}
		double length = Math.min(40, a.distanceTo(b));
		var random = minecraft.level.getRandom();
		for (double d = 0; d <= length; d += 0.7) {
			Vec3 at = a.lerp(b, length == 0 ? 0 : d / length);
			double x = at.x + random.nextDouble() * 0.3 - 0.15;
			double z = at.z + random.nextDouble() * 0.3 - 0.15;
			double ground = groundAt(minecraft.level, x, at.y, z);
			if (!Double.isNaN(ground)) {
				minecraft.level.addParticle(new net.minecraft.core.particles.DustParticleOptions(0x8A0A0A, 0.5f), x, ground + 0.02, z, 0, 0, 0);
			}
		}
	}

	/** Whether a clock is to show the time half a day off (`clock_wrong`). */
	public static boolean clockWrong() {
		return Hints.showing("clock_wrong");
	}

	/**
	 * `phantom_item`: in the inventory, the first time it is opened while the hint lasts, a
	 * paper named "behind you" drawn in an empty slot of theirs for three seconds, its name
	 * under the mouse; then gone, as if it never was.
	 */
	public static void phantomItem(GuiGraphicsExtractor graphics, net.minecraft.world.inventory.AbstractContainerMenu menu, int left, int top, int mouseX, int mouseY) {
		long now = System.currentTimeMillis();
		for (Hints.Active active : Hints.active()) {
			if (!active.hint().hint().equals("phantom_item")) {
				continue;
			}
			long shown = PHANTOM_SHOWN.computeIfAbsent(active.id(), id -> now);
			if (now - shown > 3000) {
				continue;
			}
			net.minecraft.world.inventory.Slot empty = null;
			for (net.minecraft.world.inventory.Slot slot : menu.slots) {
				if (slot.container instanceof net.minecraft.world.entity.player.Inventory && slot.getContainerSlot() >= 9
						&& slot.getContainerSlot() < 36 && !slot.hasItem() && (empty == null || slot.getContainerSlot() == 22)) {
					empty = slot;
				}
			}
			if (empty == null) {
				return;
			}
			var paper = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.PAPER);
			paper.set(net.minecraft.core.component.DataComponents.CUSTOM_NAME, net.minecraft.network.chat.Component.literal("behind you"));
			int x = left + empty.x;
			int y = top + empty.y;
			graphics.item(paper, x, y);
			if (mouseX >= x && mouseX < x + 16 && mouseY >= y && mouseY < y + 16) {
				graphics.setTooltipForNextFrame(Minecraft.getInstance().font, paper, mouseX, mouseY);
			}
			return;
		}
	}

	/** `rustle_in_the_pack`: the inventory open, a bundle's rustle once a hint. */
	public static void inventoryOpen() {
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("rustle_in_the_pack")) {
				once(active, () -> {
					var minecraft = Minecraft.getInstance();
					if (minecraft.player != null && minecraft.level != null) {
						minecraft.level.playLocalSound(minecraft.player.getX(), minecraft.player.getY(), minecraft.player.getZ(),
								net.minecraft.sounds.SoundEvents.BUNDLE_INSERT, SoundSource.PLAYERS, 0.8f, 0.8f, false);
					}
				});
			}
		}
	}

	/** Dust down from the ceiling over them, while the rumble is strong: indoors alone. */
	private static void rumbleDust(Minecraft minecraft, float progress) {
		if (progress > 0.7f || minecraft.level.getGameTime() % 2 != 0) {
			return;
		}
		net.minecraft.core.BlockPos head = minecraft.player.blockPosition().above(2);
		var random = minecraft.level.getRandom();
		for (int up = 0; up < 6; up++) {
			net.minecraft.core.BlockPos ceiling = head.above(up);
			var state = minecraft.level.getBlockState(ceiling);
			if (!state.isAir()) {
				for (int i = 0; i < 3; i++) {
					minecraft.level.addParticle(new net.minecraft.core.particles.BlockParticleOption(net.minecraft.core.particles.ParticleTypes.FALLING_DUST, state),
							minecraft.player.getX() + random.nextDouble() * 6 - 3, ceiling.getY() - 0.05, minecraft.player.getZ() + random.nextDouble() * 6 - 3, 0, 0, 0);
				}
				return;
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

	/** The footprints a `footprints` hint holds, dark dust on the ground under each within 24 blocks. */
	private static void drawPrints(Minecraft minecraft, Hints.Active active, craftkit.host.net.HintPayload hint) {
		var eye = minecraft.player.position();
		for (FootPrint.Print print : PRINTS.computeIfAbsent(active.id(), id -> FootPrint.read(hint.detail()))) {
			double dx = print.x() - eye.x;
			double dz = print.z() - eye.z;
			if (dx * dx + dz * dz > 24 * 24) {
				continue;
			}
			double ground = groundAt(minecraft.level, print.x(), print.y(), print.z());
			if (Double.isNaN(ground)) {
				continue;
			}
			for (double[] d : FootPrint.dots(print.yaw())) {
				minecraft.level.addParticle(new net.minecraft.core.particles.DustParticleOptions(0x2E2218, 0.4f),
						print.x() + d[0], ground + 0.02, print.z() + d[1], 0, 0, 0);
			}
		}
	}

	/** Whether Iris has a shader pack on, and when it was last asked. */
	private static boolean shaders;
	private static long shadersAsked;

	/**
	 * Whether Iris has a shader pack on, asked through its API when it is there (false without
	 * it), at most every two seconds: a mist draws its veil every frame, and reflection each frame
	 * cost for nothing.
	 */
	private static boolean shadersOn() {
		long now = System.currentTimeMillis();
		if (now - shadersAsked < 2000) {
			return shaders;
		}
		shadersAsked = now;
		try {
			Class<?> api = Class.forName("net.irisshaders.iris.api.v0.IrisApi");
			Object instance = api.getMethod("getInstance").invoke(null);
			shaders = (boolean) api.getMethod("isShaderPackInUse").invoke(instance);
		} catch (ReflectiveOperationException | LinkageError absent) {
			shaders = false;
		}
		return shaders;
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
