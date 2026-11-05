package craftkit.host.world;

import com.mojang.math.Transformation;
import craftkit.host.senses.Senses;
import net.minecraft.core.BlockPos;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.particles.BlockParticleOption;
import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerBossEvent;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.BossEvent;
import net.minecraft.world.Difficulty;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.damagesource.CombatRules;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.Display;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntitySpawnReason;
import net.minecraft.world.entity.EntityTypes;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.entity.decoration.Mannequin;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.component.ResolvableProfile;
import net.minecraft.world.item.component.SwingAnimation;
import net.minecraft.world.item.enchantment.Enchantments;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.phys.Vec3;
import org.joml.Matrix4f;
import org.joml.Quaternionf;
import org.joml.Vector3f;

import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * The confrontation, the fourth ending ({@code confront}, ABI 43; owner, 2026-11-10: "a fourth
 * ending: you confront it ... rise out of the ground ... deforming and bending its limbs, turning
 * grotesque, in between stages"). One fight a player, under one boss bar in their name, run tick
 * by tick here:
 *
 * <ol>
 *   <li><b>The rising.</b> The ground before its door cracks and it rises out of it, lying flat,
 *       then upright; the bar fills as it rises.</li>
 *   <li><b>Waves.</b> It watches from its roof while warped animals come at them, three waves
 *       (none on peaceful).</li>
 *   <li><b>Bend 1.</b> It drops onto all fours, crawls at them flat and fast, stands too tall,
 *       its head backwards a beat.</li>
 *   <li><b>The duel.</b> A mannequin in their skin and a copy of their best gear: strafing,
 *       sprint-hitting, pearling behind them, twitching a block aside, upside down a second at a
 *       time.</li>
 *   <li><b>Bend 2.</b> At three fifths it freezes and shakes, and copies peel off it; one is real
 *       (a hit on a copy puffs it away).</li>
 *   <li><b>Bend 3.</b> At seven twentieths they crawl together and sink; its blocks rise out of
 *       the ground and snap together into a body.</li>
 *   <li><b>The construct.</b> Block limbs bending the wrong way, slams and sweeps; at each third an
 *       arm torn off and grown back as two, the torso stretched, the head twisted round.</li>
 *   <li><b>The end.</b> It falls apart into its blocks; the figure stands where it was, looks at
 *       them and sinks. Its lantern, its head and its weapon are theirs, with what it held.</li>
 * </ol>
 *
 * Its hits are scaled to their armour and the difficulty; a hit that would kill goes through
 * {@link Deaths}, so they die once keeping everything. Every entity of a fight carries {@link #TAG}
 * and is taken away, dropping nothing, when the fight ends, its level closes, or it is seen again
 * after a quit.
 */
public final class Fights {
	public static final String TAG = "craftkit_fight";
	/** How far from where it began they may go before it is a flight. */
	private static final double ARENA = 80;
	/** The longest a fight lasts. */
	private static final long MOST_TICKS = 20 * 60 * 12;
	/** The mannequin's own health, never spent: the fight keeps its own. */
	private static final float HIDDEN_HEALTH = 1000;

	private enum Phase { RISING, WAVES, BEND1, DUEL, BEND2, COPIES, BEND3, CONSTRUCT, END }

	/** One block of the construct: a limb or the torso or the head, hung from its pivot. */
	private static final class Part {
		final Display.BlockDisplay display;
		/** The block it shows (the display's own getter is private). */
		BlockState state;
		final String role;
		final Vector3f size;
		/** Where it rose from, while it assembles. */
		Vec3 from;
		float grown = 1;
		boolean gone;

		Part(Display.BlockDisplay display, String role, Vector3f size) {
			this.display = display;
			this.role = role;
			this.size = size;
		}
	}

	private static final class Fight {
		final UUID player;
		final ServerLevel level;
		final WorldData data;
		final Vec3 at;
		final Vec3 roof;
		final List<String> blocks;
		final ServerBossEvent bar;
		final float max;
		float pool;
		final float hit;
		Phase phase = Phase.RISING;
		long phaseTick;
		long tick;
		Mannequin real;
		final List<Mannequin> copies = new ArrayList<>();
		final List<Mob> animals = new ArrayList<>();
		final List<Part> parts = new ArrayList<>();
		int wave;
		long nextAttack;
		long nextPearl;
		long nextTwitch;
		long flippedUntil;
		long slamAt;
		int strafe = 1;
		int deformed;
		int dropped;
		/** They died to it (the one death, keeping everything). */
		boolean lost;
		ItemStack weapon = ItemStack.EMPTY;

		Fight(UUID player, ServerLevel level, WorldData data, Vec3 at, Vec3 roof, List<String> blocks, String name, float max, float hit) {
			this.player = player;
			this.level = level;
			this.data = data;
			this.at = at;
			this.roof = roof;
			this.blocks = blocks;
			this.max = max;
			this.pool = max;
			this.hit = hit;
			this.bar = new ServerBossEvent(UUID.randomUUID(), Component.literal(name), BossEvent.BossBarColor.PURPLE, BossEvent.BossBarOverlay.NOTCHED_10);
		}
	}

	private final List<Fight> fights = new ArrayList<>();
	private final Charges charges;
	private final Deaths deaths;
	private final Senses senses;
	private long tick;

	public Fights(Charges charges, Deaths deaths, Senses senses) {
		this.charges = charges;
		this.deaths = deaths;
		this.senses = senses;
	}

	/** Begins a fight; answers why not, or null. */
	public String start(ServerLevel level, ServerPlayer player, BlockPos at, Vec3 roof, List<String> blocks, WorldData data) {
		if (fights.stream().anyMatch(f -> f.player.equals(player.getUUID()))) {
			return "a fight is on already";
		}
		if (player.isCreative() || player.isSpectator()) {
			return "never in creative or spectator";
		}
		Gear gear = Gear.of(player);
		float diff = switch (level.getDifficulty()) {
			case PEACEFUL -> 0.6f;
			case EASY -> 0.75f;
			case HARD -> 1.3f;
			default -> 1f;
		};
		float max = (float) (150 * (0.6 + gear.armor / 20.0 + gear.damage / 20.0) * diff);
		float hit = (float) (3 + gear.armor * 0.35);
		Vec3 door = Vec3.atBottomCenterOf(at);
		Vec3 top = roof.y <= -1000 ? ground(level, roof) : roof;
		Fight fight = new Fight(player.getUUID(), level, data, door, top, blocks, player.getGameProfile().name(), max, hit);
		fight.weapon = gear.weapon;
		fight.bar.addPlayer(player);
		fight.bar.setProgress(0);
		fight.real = mannequin(level, player, ground(level, door).add(0, -2.2, 0), gear);
		fight.real.setPose(Pose.SWIMMING);
		fight.real.noPhysics = true;
		fight.real.setNoGravity(true);
		fights.add(fight);
		level.playSound(null, door.x, door.y, door.z, SoundEvents.STONE_BREAK, SoundSource.HOSTILE, 1.5f, 0.5f);

		return null;
	}

	public void tick(MinecraftServer server) {
		tick++;
		Iterator<Fight> it = fights.iterator();
		while (it.hasNext()) {
			Fight fight = it.next();
			fight.tick++;
			ServerPlayer player = server.getPlayerList().getPlayer(fight.player);
			String over = null;
			if (player == null || player.level() != fight.level || player.position().distanceTo(fight.at) > ARENA || fight.tick > MOST_TICKS) {
				over = "fight_fled";
			} else if (player.isDeadOrDying()) {
				over = "fight_lost";
			} else {
				try {
					over = step(fight, player);
				} catch (RuntimeException e) {
					craftkit.host.Log.LOGGER.warn("the fight stopped: {}", e.toString());
					over = "fight_fled";
				}
			}
			if (over != null) {
				clear(fight);
				it.remove();
				ServerPlayer told = player != null ? player : null;
				if (told != null) {
					senses.noticedFight(told, over);
				}
			}
		}
		if (tick % 100 == 0) {
			for (ServerLevel level : server.getAllLevels()) {
				for (Entity entity : level.getAllEntities()) {
					if (entity.entityTags().contains(TAG) && fights.stream().noneMatch(f -> owns(f, entity))) {
						entity.discard();
					}
				}
			}
		}
	}

	/** Every fight in a level ended: it is about to be saved and closed. */
	public void endIn(ServerLevel level) {
		fights.removeIf(f -> {
			if (f.level == level) {
				clear(f);
				return true;
			}
			return false;
		});
	}

	/** Every fight ended: the world is closing. */
	public void clearAll() {
		fights.forEach(Fights::clear);
		fights.clear();
	}

	// ---- one tick of a fight

	private String step(Fight f, ServerPlayer player) {
		long in = f.tick - f.phaseTick;
		Mannequin m = f.real;
		switch (f.phase) {
			case RISING -> {
				// Out of the ground, lying flat, then bent upright at the waist.
				m.snapTo(m.getX(), m.getY() + 0.04, m.getZ(), yawTo(m.position(), player.position()), 0);
				if (in % 6 == 0) {
					dust(f.level, f.at, Blocks.DIRT.defaultBlockState(), 12);
					f.level.playSound(null, f.at.x, f.at.y, f.at.z, SoundEvents.GRAVEL_BREAK, SoundSource.HOSTILE, 1f, 0.6f);
				}
				f.bar.setProgress(Math.min(1, in / 70f));
				if (in == 55) {
					m.setPose(Pose.STANDING);
				}
				if (in >= 70) {
					m.noPhysics = false;
					m.setNoGravity(false);
					m.setYHeadRot(yawTo(m.position(), player.position()));
					next(f, f.level.getDifficulty() == Difficulty.PEACEFUL ? Phase.BEND1 : Phase.WAVES);
					if (f.phase == Phase.WAVES) {
						m.snapTo(f.roof.x, f.roof.y, f.roof.z, 0, 0);
						m.setNoGravity(true);
						m.noPhysics = true;
					}
				}
			}
			case WAVES -> {
				face(m, player.position());
				f.animals.removeIf(a -> a.isRemoved() || a.isDeadOrDying());
				if (f.animals.isEmpty() || in > 20 * 40) {
					if (f.wave >= 3) {
						f.animals.forEach(Entity::discard);
						f.animals.clear();
						m.noPhysics = false;
						m.setNoGravity(false);
						next(f, Phase.BEND1);
					} else {
						wave(f, player);
						f.phaseTick = f.tick;
					}
				}
			}
			case BEND1 -> {
				// On all fours, flat and fast at them; then up, too tall, its head backwards.
				if (in == 0) {
					m.snapTo(f.at.x, ground(f.level, f.at).y, f.at.z, 0, 0);
					m.setPose(Pose.SWIMMING);
					f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.PLAYER_BIG_FALL, SoundSource.HOSTILE, 1f, 0.5f);
				}
				if (in < 40) {
					chase(m, player.position(), 0.34, 0);
				} else if (in == 40) {
					m.setPose(Pose.STANDING);
					scale(m, 1.3);
				} else if (in < 60) {
					m.setYHeadRot(m.getYRot() + 180);
				} else {
					next(f, Phase.DUEL);
				}
			}
			case DUEL, COPIES -> {
				damageTaken(f, player);
				fightAs(f, m, player, 1);
				for (Iterator<Mannequin> it = f.copies.iterator(); it.hasNext(); ) {
					Mannequin copy = it.next();
					if (copy.getHealth() < HIDDEN_HEALTH) {
						// A copy: a puff, and gone.
						f.level.sendParticles(ParticleTypes.LARGE_SMOKE, copy.getX(), copy.getY() + 1, copy.getZ(), 20, 0.3, 0.6, 0.3, 0.02);
						f.level.playSound(null, copy.getX(), copy.getY(), copy.getZ(), SoundEvents.FIRE_EXTINGUISH, SoundSource.HOSTILE, 1f, 0.6f);
						copy.discard();
						it.remove();
						continue;
					}
					fightAs(f, copy, player, 0.5f);
				}
				if (f.phase == Phase.DUEL && f.pool <= f.max * 0.6f) {
					next(f, Phase.BEND2);
				} else if (f.phase == Phase.COPIES && f.pool <= f.max * 0.35f) {
					next(f, Phase.BEND3);
				}
			}
			case BEND2 -> {
				// It freezes and shakes; then the copies peel off it and walk out.
				if (in < 30) {
					m.setDeltaMovement(Vec3.ZERO);
					m.snapTo(m.getX() + (f.level.getRandom().nextDouble() - 0.5) * 0.3, m.getY(), m.getZ() + (f.level.getRandom().nextDouble() - 0.5) * 0.3, m.getYRot() + (f.level.getRandom().nextFloat() - 0.5f) * 40, 0);
				} else if (in == 30) {
					int n = 2 + switch (f.level.getDifficulty()) {
						case HARD -> 2;
						case NORMAL -> 1;
						default -> 0;
					};
					for (int i = 0; i < n; i++) {
						Mannequin copy = mannequin(f.level, player, m.position(), Gear.of(player));
						scale(copy, 1.3);
						copy.setYRot(i * 360f / n);
						copy.setYHeadRot(copy.getYRot() + 180);
						f.copies.add(copy);
					}
					f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.PLAYER_HURT, SoundSource.HOSTILE, 1f, 0.4f);
				} else if (in < 50) {
					for (Mannequin copy : f.copies) {
						Vec3 out = Vec3.directionFromRotation(0, copy.getYRot()).scale(0.15);
						copy.setDeltaMovement(out.x, copy.getDeltaMovement().y, out.z);
					}
				} else {
					next(f, Phase.COPIES);
				}
			}
			case BEND3 -> {
				// They crawl together and sink; its blocks rise and snap into a body.
				List<Mannequin> all = new ArrayList<>(f.copies);
				all.add(m);
				if (in == 0) {
					all.forEach(c -> c.setPose(Pose.SWIMMING));
				}
				if (in < 30) {
					for (Mannequin c : all) {
						if (c != m) {
							chase(c, m.position(), 0.25, 0);
						}
					}
				} else if (in < 60) {
					for (Mannequin c : all) {
						c.noPhysics = true;
						c.setNoGravity(true);
						c.snapTo(c.getX(), c.getY() - 0.06, c.getZ(), c.getYRot(), 0);
					}
					if (in % 5 == 0) {
						dust(f.level, m.position(), Blocks.DIRT.defaultBlockState(), 10);
					}
				} else if (in == 60) {
					f.copies.forEach(Entity::discard);
					f.copies.clear();
					m.setInvisible(true);
					m.setPose(Pose.STANDING);
					scale(m, 2.4);
					m.snapTo(m.getX(), ground(f.level, m.position()).y, m.getZ(), m.getYRot(), 0);
					assemble(f);
				} else if (in < 110) {
					m.noPhysics = false;
					m.setNoGravity(false);
					animate(f, (in - 60) / 50f, 0);
				} else {
					next(f, Phase.CONSTRUCT);
				}
			}
			case CONSTRUCT -> {
				damageTaken(f, player);
				float left = f.pool / (f.max * 0.35f);
				int should = left < 0.33f ? 3 : left < 0.66f ? 2 : left < 0.99f ? 1 : 0;
				while (f.deformed < should) {
					deform(f);
				}
				// It walks at them slowly; now and then it raises its arms and slams the ground.
				double dist = m.position().distanceTo(player.position());
				long slam = f.slamAt == 0 ? -1 : f.tick - f.slamAt;
				if (slam < 0 && dist < 6 && f.tick >= f.nextAttack) {
					f.slamAt = f.tick;
					slam = 0;
				}
				if (slam >= 0 && slam < 16) {
					m.setDeltaMovement(0, m.getDeltaMovement().y, 0);
					face(m, player.position());
					if (slam == 13) {
						slamDown(f, player);
					}
				} else {
					f.slamAt = 0;
					chase(m, player.position(), 0.16, 3.0);
					if (dist < 4.5 && f.tick >= f.nextAttack && f.level.getRandom().nextInt(3) == 0) {
						hurt(f, player, f.hit * 0.8f);
						m.swing(InteractionHand.MAIN_HAND, SwingAnimation.DEFAULT, true);
						f.nextAttack = f.tick + 30;
					}
				}
				animate(f, 1, slam >= 0 && slam < 16 ? slam : -1);
				if (f.pool <= 0) {
					next(f, Phase.END);
				}
			}
			case END -> {
				// It falls apart; the figure stands where it was, looks at them, and sinks.
				if (in == 0) {
					for (Part part : f.parts) {
						if (!part.gone) {
							drop(f, part.state, part.display.position());
							part.display.discard();
						}
					}
					f.parts.clear();
					f.bar.setProgress(0);
					m.setInvisible(false);
					scale(m, 1.0);
					m.setItemSlot(EquipmentSlot.MAINHAND, ItemStack.EMPTY);
					f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.WITHER_BREAK_BLOCK, SoundSource.HOSTILE, 1f, 0.6f);
				}
				if (in < 40) {
					m.setDeltaMovement(0, m.getDeltaMovement().y, 0);
					face(m, player.position());
				} else if (in < 90) {
					m.noPhysics = true;
					m.setNoGravity(true);
					m.snapTo(m.getX(), m.getY() - 0.04, m.getZ(), m.getYRot(), 0);
				} else {
					reward(f, player);
					return "fight_won";
				}
			}
		}
		if (f.phase.ordinal() >= Phase.DUEL.ordinal() && f.phase != Phase.END) {
			f.bar.setProgress(Math.max(0, f.pool / f.max));
		}
		return f.lost ? "fight_lost" : null;
	}

	private static void next(Fight f, Phase phase) {
		f.phase = phase;
		f.phaseTick = f.tick;
	}

	/** What a fighter took since the last tick, out of the fight's own health. */
	private static void damageTaken(Fight f, ServerPlayer player) {
		Mannequin m = f.real;
		float lost = HIDDEN_HEALTH - m.getHealth();
		if (lost <= 0) {
			return;
		}
		m.setHealth(HIDDEN_HEALTH);
		// Blocking with their own shield: the hit is caught.
		if (m.isBlocking()) {
			f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.SHIELD_BLOCK.value(), SoundSource.HOSTILE, 1f, 1f);
			return;
		}
		f.pool = Math.max(0, f.pool - lost);
		// The construct knocks its blocks loose as it is hit.
		if (f.phase == Phase.CONSTRUCT && f.dropped < 12 && lost >= 3 && !f.blocks.isEmpty()) {
			String id = f.blocks.get(f.dropped % f.blocks.size());
			drop(f, net.minecraft.core.registries.BuiltInRegistries.BLOCK.get(net.minecraft.resources.Identifier.parse(id))
					.map(b -> b.value().defaultBlockState()).orElse(Blocks.COBBLESTONE.defaultBlockState()), m.position().add(0, 2, 0));
			f.dropped++;
		}
	}

	/** One fighter's turn in the duel: strafing, sprint-hitting, pearling, twitching, flipping. */
	private void fightAs(Fight f, Mannequin m, ServerPlayer player, float strength) {
		Vec3 to = player.position().subtract(m.position());
		double dist = to.horizontalDistance();
		Vec3 dir = new Vec3(to.x, 0, to.z).normalize();
		Vec3 side = new Vec3(-dir.z, 0, dir.x).scale(f.strafe);
		if (f.tick % 50 == 0) {
			f.strafe = -f.strafe;
		}
		Vec3 want;
		boolean ready = f.tick >= f.nextAttack;
		if (ready && dist > 2.4) {
			want = dir.scale(0.3);
			m.setSprinting(true);
		} else if (dist < 3) {
			want = side.scale(0.17).add(dir.scale(-0.04));
			m.setSprinting(false);
		} else {
			want = dir.scale(0.2).add(side.scale(0.08));
			m.setSprinting(false);
		}
		move(m, want);
		face(m, player.position());
		if (dist <= 2.8 && ready) {
			m.swing(InteractionHand.MAIN_HAND, SwingAnimation.DEFAULT, true);
			hurt(f, player, f.hit * strength);
			f.nextAttack = f.tick + 14 + f.level.getRandom().nextInt(8);
		}
		if (m != f.real) {
			return;
		}
		// Now and then behind them, as by a pearl.
		if (f.tick >= f.nextPearl && (dist > 9 || f.level.getRandom().nextInt(200) == 0)) {
			Vec3 behind = player.position().subtract(Vec3.directionFromRotation(0, player.getYRot()).scale(2.5));
			if (f.level.noCollision(m, m.getBoundingBox().move(behind.subtract(m.position())))) {
				f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.ENDER_PEARL_THROW, SoundSource.HOSTILE, 1f, 1f);
				m.snapTo(behind.x, behind.y, behind.z, m.getYRot(), 0);
				f.level.sendParticles(ParticleTypes.PORTAL, behind.x, behind.y + 1, behind.z, 30, 0.3, 0.6, 0.3, 0.1);
				f.level.playSound(null, behind.x, behind.y, behind.z, SoundEvents.PLAYER_TELEPORT, SoundSource.HOSTILE, 1f, 1f);
			}
			f.nextPearl = f.tick + 160 + f.level.getRandom().nextInt(100);
		}
		// A twitch: a block aside and back.
		if (f.tick >= f.nextTwitch) {
			Vec3 aside = m.position().add(side.scale(1.0));
			if (f.level.noCollision(m, m.getBoundingBox().move(aside.subtract(m.position())))) {
				m.snapTo(aside.x, aside.y, aside.z, m.getYRot() + 25, 0);
			}
			f.nextTwitch = f.tick + 70 + f.level.getRandom().nextInt(60);
		}
		// Upside down a second at a time.
		if (f.flippedUntil == 0 && f.level.getRandom().nextInt(180) == 0) {
			m.setCustomName(Component.literal("Dinnerbone"));
			m.setCustomNameVisible(false);
			f.flippedUntil = f.tick + 20;
		} else if (f.flippedUntil != 0 && f.tick >= f.flippedUntil) {
			m.setCustomName(null);
			f.flippedUntil = 0;
		}
		// Their shield, raised now and then, if they carry one.
		if (!m.getOffhandItem().isEmpty() && !m.isBlocking() && f.level.getRandom().nextInt(120) == 0) {
			m.startUsingItem(InteractionHand.OFF_HAND);
		} else if (m.isBlocking() && f.level.getRandom().nextInt(25) == 0) {
			m.stopUsingItem();
		}
	}

	/** A hit on them, scaled as the game scales a mob's; one that would kill is the one death that keeps everything. */
	private void hurt(Fight f, ServerPlayer player, float raw) {
		DamageSource source = f.level.damageSources().mobAttack(f.real);
		float scaled = switch (f.level.getDifficulty()) {
			case PEACEFUL -> 0;
			case EASY -> Math.min(raw / 2 + 1, raw);
			case HARD -> raw * 1.5f;
			default -> raw;
		};
		float after = CombatRules.getDamageAfterAbsorb(player, scaled, source, (float) player.getArmorValue(), (float) player.getAttributeValue(Attributes.ARMOR_TOUGHNESS));
		if (after >= player.getHealth() + player.getAbsorptionAmount()) {
			if (deaths.kill(player, f.data) == null) {
				f.lost = true;
			}
			return;
		}
		player.hurtServer(f.level, source, raw);
	}

	private void slamDown(Fight f, ServerPlayer player) {
		Mannequin m = f.real;
		f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.IRON_GOLEM_ATTACK, SoundSource.HOSTILE, 1.5f, 0.5f);
		f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.GENERIC_EXPLODE.value(), SoundSource.HOSTILE, 0.6f, 1.4f);
		dust(f.level, m.position(), f.level.getBlockState(m.blockPosition().below()), 60);
		if (player.position().distanceTo(m.position()) < 4.5) {
			hurt(f, player, f.hit * 1.3f);
			Vec3 away = player.position().subtract(m.position()).normalize();
			player.push(away.x * 0.8, 0.5, away.z * 0.8);
			player.connection.send(new net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket(player));
		}
		f.nextAttack = f.tick + 70;
	}

	/** A wave of warped animals at them from the edge of the arena. */
	private void wave(Fight f, ServerPlayer player) {
		f.wave++;
		int n = 2 + switch (f.level.getDifficulty()) {
			case HARD -> 3;
			case NORMAL -> 2;
			default -> 1;
		};
		var kinds = List.of(EntityTypes.COW, EntityTypes.PIG, EntityTypes.SHEEP);
		for (int i = 0; i < n; i++) {
			double angle = Math.PI * 2 * i / n + f.wave;
			Vec3 at = ground(f.level, player.position().add(Math.cos(angle) * 14, 0, Math.sin(angle) * 14));
			Mob animal = (Mob) kinds.get((i + f.wave) % kinds.size()).create(f.level, EntitySpawnReason.EVENT);
			if (animal == null) {
				continue;
			}
			animal.snapTo(at.x, at.y, at.z, 0, 0);
			animal.addTag(TAG);
			if (i % 2 == 0) {
				animal.setCustomName(Component.literal("Dinnerbone"));
				animal.setCustomNameVisible(false);
			} else {
				var size = animal.getAttribute(Attributes.SCALE);
				if (size != null) {
					size.setBaseValue(1.7);
				}
			}
			f.level.addFreshEntity(animal);
			f.animals.add(animal);
			charges.add(animal, player, 3, 4, 40, true);
		}
		f.level.playSound(null, player.getX(), player.getY(), player.getZ(), SoundEvents.RAVAGER_ROAR, SoundSource.HOSTILE, 0.7f, 0.5f);
	}

	// ---- the construct

	/** Its blocks rise out of the ground around it, to snap together over two and a half seconds. */
	private static void assemble(Fight f) {
		String[][] layout = {
				{"torso", "0"}, {"head", "4"}, {"thigh_l", "1"}, {"shin_l", "1"}, {"thigh_r", "1"}, {"shin_r", "1"},
				{"arm_l", "2"}, {"fore_l", "3"}, {"arm_r", "2"}, {"fore_r", "3"}, {"torch", "5"},
		};
		for (String[] part : layout) {
			add(f, part[0], f.blocks.get(Math.min(Integer.parseInt(part[1]), f.blocks.size() - 1)));
		}
	}

	private static void add(Fight f, String role, String block) {
		Display.BlockDisplay display = EntityTypes.BLOCK_DISPLAY.create(f.level, EntitySpawnReason.EVENT);
		if (display == null) {
			return;
		}
		BlockState state = net.minecraft.core.registries.BuiltInRegistries.BLOCK.get(net.minecraft.resources.Identifier.parse(block))
				.map(b -> b.value().defaultBlockState()).orElse(Blocks.COBBLESTONE.defaultBlockState());
		if (role.equals("torch")) {
			state = Blocks.TORCH.defaultBlockState();
		}
		Reflect.blockState(display, state);
		double angle = f.level.getRandom().nextDouble() * Math.PI * 2;
		Vec3 from = ground(f.level, f.real.position().add(Math.cos(angle) * 5, 0, Math.sin(angle) * 5)).add(0, -1, 0);
		display.snapTo(from.x, from.y, from.z, 0, 0);
		display.addTag(TAG);
		f.level.addFreshEntity(display);
		Part part = new Part(display, role, sizeOf(role));
		part.state = state;
		part.from = from;
		f.parts.add(part);
	}

	private static Vector3f sizeOf(String role) {
		return switch (role) {
			case "torso" -> new Vector3f(1.4f, 1.6f, 0.8f);
			case "head" -> new Vector3f(0.9f, 0.9f, 0.9f);
			case "thigh_l", "thigh_r", "shin_l", "shin_r" -> new Vector3f(0.55f, 1.0f, 0.55f);
			case "torch" -> new Vector3f(0.5f, 0.5f, 0.5f);
			default -> new Vector3f(0.45f, 0.95f, 0.45f);
		};
	}

	/**
	 * Every part placed and posed: hung from its pivot, the knees and elbows bending the wrong
	 * way as it walks, the arms raised for a slam ({@code slam} its tick, or -1), its deformations
	 * kept. {@code assembled} below 1 brings each part up from where it rose.
	 */
	private static void animate(Fight f, float assembled, long slam) {
		Mannequin m = f.real;
		float yaw = m.getYRot();
		float walk = (float) Math.sin(f.tick * 0.25) * (float) Math.min(1, m.getDeltaMovement().horizontalDistance() * 8);
		float stretch = f.deformed >= 2 ? 1.6f : 1f;
		float hip = 2.0f;
		float top = hip + 1.6f * stretch;
		float raise = slam < 0 ? 0 : slam < 13 ? -2.6f * slam / 13f : 0.5f;
		for (Part part : f.parts) {
			if (part.gone) {
				continue;
			}
			Vector3f pivot;
			Quaternionf turn = new Quaternionf();
			Vector3f size = new Vector3f(part.size).mul(part.grown);
			boolean right = part.role.endsWith("_r") || part.role.endsWith("_r2");
			float s = right ? 1 : -1;
			switch (part.role) {
				case "torso" -> {
					pivot = new Vector3f(0, top, 0);
					size.y *= stretch;
				}
				case "head" -> {
					pivot = new Vector3f(0, top + 0.9f, 0);
					turn.rotateY(f.deformed >= 3 ? (float) Math.PI + f.tick * 0.05f : 0);
				}
				case "torch" -> pivot = new Vector3f(0, top + 1.4f, 0);
				case "thigh_l", "thigh_r" -> {
					pivot = new Vector3f(0.4f * s, hip, 0);
					turn.rotateX(walk * s * 0.6f);
				}
				case "shin_l", "shin_r" -> {
					float thigh = walk * s * 0.6f;
					pivot = new Vector3f(0.4f * s, hip - (float) Math.cos(thigh), (float) Math.sin(thigh) * -1);
					// The knee bends the wrong way: forward.
					turn.rotateX(thigh - 0.9f - Math.abs(walk) * 0.5f);
				}
				case "arm_l", "arm_r", "arm_l2", "arm_r2" -> {
					float low = part.role.endsWith("2") ? 0.7f : 0;
					pivot = new Vector3f(0.95f * s, top - 0.1f - low, 0);
					turn.rotateX(-walk * s * 0.5f + raise).rotateZ(s * 0.15f);
				}
				default -> {
					// A forearm, at the end of its arm: the elbow bending the wrong way.
					float low = part.role.endsWith("2") ? 0.7f : 0;
					float upper = -walk * s * 0.5f + raise;
					pivot = new Vector3f(0.95f * s + s * 0.14f, top - 0.1f - low - (float) Math.cos(upper) * 0.95f, (float) Math.sin(upper) * -0.95f);
					turn.rotateX(upper + 1.0f);
				}
			}
			// Rotated with the body, and placed relative to its feet.
			Vector3f world = new Vector3f(pivot).rotateY((float) Math.toRadians(-yaw));
			Vec3 target = m.position().add(world.x, world.y, world.z);
			Vec3 at = assembled >= 1 ? target : part.from.lerp(target, Math.max(0, assembled));
			part.display.snapTo(at.x, at.y, at.z, yaw, 0);
			Matrix4f matrix = new Matrix4f().rotate(turn).scale(size).translate(-0.5f, -1f, -0.5f);
			Reflect.transformation(part.display, new Transformation(matrix));
		}
	}

	/** One more deformation, at each third: an arm torn off and grown back as two; the torso stretched; the head twisted round. */
	private static void deform(Fight f) {
		f.deformed++;
		Mannequin m = f.real;
		f.level.playSound(null, m.getX(), m.getY(), m.getZ(), SoundEvents.ZOMBIE_BREAK_WOODEN_DOOR, SoundSource.HOSTILE, 1.5f, 0.5f);
		if (f.deformed == 1) {
			String block = f.blocks.get(Math.min(2, f.blocks.size() - 1));
			for (Part part : f.parts) {
				if (part.role.equals("arm_l") || part.role.equals("fore_l")) {
					drop(f, part.state, part.display.position());
					part.display.discard();
					part.gone = true;
				}
			}
			for (String role : new String[] {"arm_l", "fore_l", "arm_l2", "fore_l2"}) {
				add(f, role, block);
				f.parts.getLast().from = f.real.position();
				f.parts.getLast().grown = 1;
			}
		}
	}

	private static void drop(Fight f, BlockState state, Vec3 at) {
		ItemStack stack = new ItemStack(state.getBlock().asItem());
		if (stack.isEmpty()) {
			return;
		}
		ItemEntity item = new ItemEntity(f.level, at.x, at.y, at.z, stack);
		item.setUnlimitedLifetime();
		f.level.addFreshEntity(item);
	}

	// ---- the end

	/** Its lantern, its head, its weapon, and what it held, theirs. */
	private static void reward(Fight f, ServerPlayer player) {
		String name = player.getGameProfile().name();
		ItemStack lantern = new ItemStack(Items.SOUL_LANTERN);
		lantern.set(DataComponents.CUSTOM_NAME, Component.literal(name + "'s light"));
		ItemStack head = new ItemStack(Items.PLAYER_HEAD);
		head.set(DataComponents.PROFILE, ResolvableProfile.createResolved(player.getGameProfile()));
		head.set(DataComponents.CUSTOM_NAME, Component.literal("the lodger"));
		ItemStack weapon = f.weapon.isEmpty() ? new ItemStack(Items.IRON_SWORD) : f.weapon.copy();
		weapon.set(DataComponents.CUSTOM_NAME, Component.literal("what it learned"));
		var enchantments = f.level.registryAccess().lookupOrThrow(Registries.ENCHANTMENT);
		var key = weapon.is(Items.BOW) || weapon.is(Items.CROSSBOW) ? Enchantments.POWER : Enchantments.SHARPNESS;
		var holder = enchantments.getOrThrow(key);
		int had = weapon.getEnchantments().getLevel(holder);
		weapon.enchant(holder, Math.min(5, had + 1));
		List<ItemStack> given = new ArrayList<>(List.of(lantern, head, weapon));
		ItemStack held = f.data == null ? null : f.data.giveBack("held");
		if (held != null) {
			given.add(held);
		}
		for (ItemStack stack : given) {
			if (!player.getInventory().add(stack)) {
				ItemEntity item = new ItemEntity(f.level, player.getX(), player.getY(), player.getZ(), stack);
				item.setUnlimitedLifetime();
				f.level.addFreshEntity(item);
			}
		}
		f.level.playSound(null, player.getX(), player.getY(), player.getZ(), SoundEvents.UI_TOAST_CHALLENGE_COMPLETE, SoundSource.PLAYERS, 1f, 0.8f);
	}

	private static void clear(Fight f) {
		f.bar.removeAllPlayers();
		f.animals.forEach(Entity::discard);
		f.copies.forEach(Entity::discard);
		f.parts.forEach(p -> p.display.discard());
		if (f.real != null) {
			f.real.discard();
		}
	}

	private static boolean owns(Fight f, Entity entity) {
		return entity == f.real || f.copies.contains(entity) || f.animals.contains(entity) || f.parts.stream().anyMatch(p -> p.display == entity);
	}

	// ---- helpers

	/** Their armour, their best weapon and how hard it hits, their shield. */
	private record Gear(int armor, double damage, ItemStack weapon, ItemStack shield, List<ItemStack> worn) {
		static Gear of(ServerPlayer player) {
			ItemStack best = ItemStack.EMPTY;
			double most = 1;
			for (int i = 0; i < player.getInventory().getContainerSize(); i++) {
				ItemStack stack = player.getInventory().getItem(i);
				double d = weaponDamage(stack);
				if (d > most) {
					most = d;
					best = stack;
				}
			}
			ItemStack shield = player.getOffhandItem().is(Items.SHIELD) ? player.getOffhandItem().copy() : ItemStack.EMPTY;
			List<ItemStack> worn = new ArrayList<>();
			for (EquipmentSlot slot : new EquipmentSlot[] {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET}) {
				worn.add(player.getItemBySlot(slot).copy());
			}
			return new Gear(player.getArmorValue(), most, best.copy(), shield, worn);
		}
	}

	/** How hard a weapon hits, by what it is made of. */
	static double weaponDamage(ItemStack stack) {
		String id = net.minecraft.core.registries.BuiltInRegistries.ITEM.getKey(stack.getItem()).getPath();
		if (!(id.endsWith("_sword") || id.endsWith("_axe") || id.equals("mace") || id.equals("trident"))) {
			return 0;
		}
		if (id.equals("mace") || id.equals("trident")) {
			return 8;
		}
		return id.startsWith("netherite") ? 8 : id.startsWith("diamond") ? 7 : id.startsWith("iron") ? 6 : id.startsWith("stone") || id.startsWith("copper") ? 5 : 4;
	}

	/** A mannequin in their skin and a copy of their gear, at a place, its own health never spent. */
	private static Mannequin mannequin(ServerLevel level, ServerPlayer player, Vec3 at, Gear gear) {
		Mannequin m = EntityTypes.MANNEQUIN.create(level, EntitySpawnReason.EVENT);
		if (m == null) {
			throw new IllegalStateException("no mannequin");
		}
		m.setComponent(DataComponents.PROFILE, ResolvableProfile.createResolved(player.getGameProfile()));
		var health = m.getAttribute(Attributes.MAX_HEALTH);
		if (health != null) {
			health.setBaseValue(HIDDEN_HEALTH);
		}
		m.setHealth(HIDDEN_HEALTH);
		m.setItemSlot(EquipmentSlot.MAINHAND, gear.weapon.copy());
		m.setItemSlot(EquipmentSlot.OFFHAND, gear.shield.copy());
		EquipmentSlot[] slots = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
		for (int i = 0; i < slots.length; i++) {
			m.setItemSlot(slots[i], gear.worn.get(i).copy());
		}
		m.snapTo(at.x, at.y, at.z, 0, 0);
		m.addTag(TAG);
		level.addFreshEntity(m);
		return m;
	}

	private static void scale(Mannequin m, double scale) {
		var size = m.getAttribute(Attributes.SCALE);
		if (size != null) {
			size.setBaseValue(scale);
		}
	}

	/** Moves toward a place at a speed, stopping {@code within} of it, jumping what it bumps into. */
	private static void chase(Mannequin m, Vec3 to, double speed, double within) {
		Vec3 d = to.subtract(m.position());
		Vec3 flat = new Vec3(d.x, 0, d.z);
		if (flat.length() <= within) {
			move(m, Vec3.ZERO);
		} else {
			move(m, flat.normalize().scale(speed));
		}
		face(m, to);
	}

	private static void move(Mannequin m, Vec3 want) {
		double y = m.getDeltaMovement().y;
		if (m.onGround() && m.horizontalCollision && want.lengthSqr() > 0) {
			y = 0.42;
		}
		m.setDeltaMovement(want.x, y, want.z);
	}

	private static void face(Entity e, Vec3 to) {
		float yaw = yawTo(e.position(), to);
		e.setYRot(yaw);
		e.setYHeadRot(yaw);
		if (e instanceof net.minecraft.world.entity.LivingEntity living) {
			living.setYBodyRot(yaw);
		}
	}

	private static float yawTo(Vec3 from, Vec3 to) {
		return Facing.yaw(to.x - from.x, to.z - from.z);
	}

	private static Vec3 ground(ServerLevel level, Vec3 at) {
		BlockPos top = level.getHeightmapPos(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, BlockPos.containing(at.x, 0, at.z));
		return new Vec3(at.x, top.getY(), at.z);
	}

	private static void dust(ServerLevel level, Vec3 at, BlockState state, int count) {
		level.sendParticles(new BlockParticleOption(ParticleTypes.BLOCK, state), at.x, at.y + 0.2, at.z, count, 0.6, 0.2, 0.6, 0.1);
	}

	/** The block displays' private setters, by name (26.3). */
	private static final class Reflect {
		private static Method blockState;
		private static Method transformation;
		private static Method duration;

		static void blockState(Display.BlockDisplay display, BlockState state) {
			try {
				if (blockState == null) {
					blockState = Display.BlockDisplay.class.getDeclaredMethod("setBlockState", BlockState.class);
					blockState.setAccessible(true);
				}
				blockState.invoke(display, state);
			} catch (ReflectiveOperationException e) {
				throw new IllegalStateException("the block display's block cannot be set: " + e);
			}
		}

		static void transformation(Display display, Transformation t) {
			try {
				if (transformation == null) {
					transformation = Display.class.getDeclaredMethod("setTransformation", Transformation.class);
					transformation.setAccessible(true);
					duration = Display.class.getDeclaredMethod("setTransformationInterpolationDuration", int.class);
					duration.setAccessible(true);
				}
				transformation.invoke(display, t);
				duration.invoke(display, 2);
			} catch (ReflectiveOperationException e) {
				throw new IllegalStateException("the block display cannot be posed: " + e);
			}
		}
	}
}
