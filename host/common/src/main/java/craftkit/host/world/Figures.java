package craftkit.host.world;

import com.mojang.authlib.GameProfile;
import net.minecraft.core.BlockPos;
import net.minecraft.network.protocol.game.ClientboundAddEntityPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket;
import net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket;
import net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.EnumSet;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * The figures: a player-shaped thing in a player's own skin that only that player sees. No
 * entity exists on the server; the figure is packets to one client (a player-list entry with
 * the player's own profile, so the skin is theirs, then a player entity with a new id), which
 * is what makes it invisible to everyone else and to every other mod. It faces the player,
 * holds what the command says, walks its path a point every half second by being re-sent, and
 * is taken back when its time is up, when the player comes within its vanishing distance, when
 * the mod removes it, or when the player leaves.
 */
public final class Figures {
	/** Entity ids far above anything the server hands out. */
	private static final int FIRST_ID = Integer.MAX_VALUE / 2;
	private static final int STEP_TICKS = 10;

	private static final class Figure {
		final ServerPlayer player;
		/** The level it was shown in: a player in another one sees it no more. */
		net.minecraft.world.level.Level level;
		final int number;
		final int entityId;
		final UUID uuid;
		final GameProfile profile;
		ItemStack held;
		Vec3 at;
		List<Vec3> path;
		/** Stands where its path ends instead of going: a choke's figure at the bed (ABI 54). */
		boolean stays;
		/**
		 * Glides along its path a little each tick, the client easing it between and moving its
		 * legs, instead of a re-send at each point every half second, which read as a stutter
		 * up close (the owner: "the walking up to the bed is very laggy").
		 */
		boolean glides;
		int pathIndex;
		long untilTick;
		final double vanishWithin;
		/** Mirrors the player from where both stood when shown (ABI 33). */
		boolean mirror;
		Vec3 shownAt;
		Vec3 playerAt;
		/** Gone once it leaves their view after they have seen it (ABI 33). */
		boolean onceSeen;
		boolean seen;
		/** Crouches as they crouch (ABI 35). */
		boolean copyCrouch;
		boolean crouched;
		/** Swings now and then, as at the rock (ABI 35). */
		boolean swinging;
		/** How it is looked at, told to the mind as it goes (ABI 47). */
		final Gaze gaze = new Gaze();

		Figure(ServerPlayer player, int number, int entityId, UUID uuid, GameProfile profile, ItemStack held, Vec3 at, List<Vec3> path, long untilTick, double vanishWithin) {
			this.player = player;
			this.number = number;
			this.entityId = entityId;
			this.uuid = uuid;
			this.profile = profile;
			this.held = held;
			this.at = at;
			this.path = path;
			this.untilTick = untilTick;
			this.vanishWithin = vanishWithin;
		}
	}

	/** A figure gone, and how it was looked at: the senses tell the mind (`figure_seen`). */
	public record Sighting(ServerPlayer player, net.minecraft.world.level.Level level, net.minecraft.core.BlockPos at, String detail) {
	}

	private final List<Figure> active = new ArrayList<>();
	private final List<Sighting> sightings = new ArrayList<>();

	/** Everything forgotten: the world is closing, and its players and levels with it. */
	public void clear() {
		active.clear();
		sightings.clear();
		digs.clear();
		chokes.clear();
	}

	/** The figures gone since the last ask, each with how it was looked at. */
	public List<Sighting> takeSightings() {
		List<Sighting> taken = List.copyOf(sightings);
		sightings.clear();
		return taken;
	}
	private int nextId = FIRST_ID;
	private long tick;

	/**
	 * Shows a figure to {@code player} wearing {@code skinOf}'s skin, or moves the one with
	 * this number. A y below -1000 stands it on the ground at that column.
	 */
	public void show(ServerLevel level, ServerPlayer player, int number, ServerPlayer skinOf, Vec3 pos, List<Vec3> path, ItemStack held, boolean nameShown, float seconds, float vanishWithin) {
		show(level, player, number, skinOf, pos, path, held, nameShown, seconds, vanishWithin, false, false);
	}

	/** {@link #show} mirroring the player, or gone once seen and looked away from (ABI 33). */
	public void show(ServerLevel level, ServerPlayer player, int number, ServerPlayer skinOf, Vec3 pos, List<Vec3> path, ItemStack held, boolean nameShown, float seconds, float vanishWithin, boolean mirror, boolean onceSeen) {
		show(level, player, number, skinOf, pos, path, held, nameShown, seconds, vanishWithin, mirror, onceSeen, false, false);
	}

	/** {@link #show} copying their crouch and swinging now and then (ABI 35). */
	public void show(ServerLevel level, ServerPlayer player, int number, ServerPlayer skinOf, Vec3 pos, List<Vec3> path, ItemStack held, boolean nameShown, float seconds, float vanishWithin, boolean mirror, boolean onceSeen, boolean copyCrouch, boolean swinging) {
		remove(player, number);

		Vec3 at = ground(level, pos);
		UUID uuid = UUID.randomUUID();
		GameProfile profile = new GameProfile(uuid, skinOf.getGameProfile().name(), skinOf.getGameProfile().properties());
		Figure figure = new Figure(player, number, nextId++, uuid, profile, held, at, path, seconds > 0 ? tick + (long) (seconds * 20) : Long.MAX_VALUE, vanishWithin);
		figure.level = level;
		figure.mirror = mirror;
		figure.shownAt = at;
		figure.playerAt = player.position();
		figure.onceSeen = onceSeen;
		figure.copyCrouch = copyCrouch;
		figure.swinging = swinging;
		active.add(figure);
		send(level, figure, nameShown);
	}

	public boolean remove(ServerPlayer player, int number) {
		Iterator<Figure> it = active.iterator();
		boolean removed = false;
		while (it.hasNext()) {
			Figure figure = it.next();
			if (figure.player == player && figure.number == number) {
				unsend(figure);
				it.remove();
				removed = true;
			}
		}

		return removed;
	}

	/** Once a tick: the paths walked, the vanishing, the time, the digs, the chokes. */
	public void tick() {
		tick++;
		tickDigs();
		tickChokes();
		Iterator<Figure> it = active.iterator();
		while (it.hasNext()) {
			Figure figure = it.next();
			boolean looked = looksAt(figure);
			if (looked) {
				figure.seen = true;
			}
			if (figure.player.level() == figure.level) {
				figure.gaze.tick(Gaze.facing(viewDot(figure)), figure.player.position().distanceTo(figure.at));
			}
			// Dead and respawned, or gone to another dimension: the client may keep its world
			// (a respawn in the same one does), so the figure is taken back through the
			// connection, which the new player shares (until now it stayed there).
			boolean left = figure.player.isRemoved() || figure.player.level() != figure.level;
			boolean approached = figure.vanishWithin > 0 && figure.player.position().distanceTo(figure.at) < figure.vanishWithin;
			boolean lookedAway = figure.onceSeen && figure.seen && !looked;
			boolean timed = tick > figure.untilTick;
			if (left || timed || approached || lookedAway) {
				unsend(figure);
				it.remove();
				String end = left ? "gone" : approached ? "approached" : lookedAway ? "looked_away" : "timed";
				// Told to the world it was shown in, not the one they went to.
				sightings.add(new Sighting(figure.player, figure.level, net.minecraft.core.BlockPos.containing(figure.at), figure.gaze.detail(end)));
				continue;
			}

			// Crouching as they crouch; swinging at the rock now and then.
			if (figure.copyCrouch && figure.player.isShiftKeyDown() != figure.crouched) {
				figure.crouched = figure.player.isShiftKeyDown();
				pose(figure, figure.crouched ? net.minecraft.world.entity.Pose.CROUCHING : net.minecraft.world.entity.Pose.STANDING);
			}
			if (figure.swinging && (tick + figure.entityId) % 70 == 0) {
				figure.player.connection.send(new net.minecraft.network.protocol.game.ClientboundSwingAnimationPacket(figure.entityId,
						net.minecraft.world.InteractionHand.MAIN_HAND, net.minecraft.world.item.component.SwingAnimation.DEFAULT));
			}

			// Mirroring: where it was shown, moved as far as the player has moved, a quarter
			// second at a time.
			if (figure.mirror && tick % 5 == 0) {
				Vec3 moved = figure.player.position().subtract(figure.playerAt);
				Vec3 to = figure.shownAt.add(moved.x, 0, moved.z);
				if (to.distanceToSqr(figure.at) > 0.04 && figure.player.level() instanceof ServerLevel level) {
					figure.player.connection.send(new ClientboundRemoveEntitiesPacket(figure.entityId));
					figure.at = ground(level, new Vec3(to.x, -10_000, to.z));
					sendEntity(figure);
				}
			}

			if (figure.glides && !figure.path.isEmpty()) {
				glide(figure);
				continue;
			}
			if (!figure.path.isEmpty() && tick % STEP_TICKS == 0 && figure.player.level() instanceof ServerLevel level) {
				// The path's points in order, its first one first: the index used to move on before
				// it was read, so the first point was skipped and a path of one ended at once.
				if (figure.pathIndex >= figure.path.size() && figure.stays) {
					figure.path = List.of();
					continue;
				}
				if (figure.pathIndex >= figure.path.size()) {
					unsend(figure);
					it.remove();
					sightings.add(new Sighting(figure.player, figure.level, net.minecraft.core.BlockPos.containing(figure.at), figure.gaze.detail("walked")));
					continue;
				}
				// A step: the entity is re-sent at the next point, which at the figures'
				// distances reads as a stride.
				figure.player.connection.send(new ClientboundRemoveEntitiesPacket(figure.entityId));
				figure.at = ground(level, figure.path.get(figure.pathIndex));
				figure.pathIndex++;
				sendEntity(figure);
			}
		}
	}

	/** A gliding figure a tick along its path, its place and facing sent; it stops at the end. */
	private static void glide(Figure figure) {
		double left = ChokeWay.PER_TICK;
		while (left > 1e-9 && figure.pathIndex < figure.path.size()) {
			Vec3 next = figure.path.get(figure.pathIndex);
			double gap = figure.at.distanceTo(next);
			if (gap <= left) {
				figure.at = next;
				figure.pathIndex++;
				left -= gap;
			} else {
				figure.at = figure.at.add(next.subtract(figure.at).scale(left / gap));
				left = 0;
			}
		}
		if (figure.pathIndex >= figure.path.size()) {
			figure.path = List.of();
		}
		Vec3 to = figure.player.position().subtract(figure.at);
		float yaw = (float) Math.toDegrees(Math.atan2(-to.x, to.z));
		float pitch = (float) -Math.toDegrees(Math.atan2(to.y, Math.sqrt(to.x * to.x + to.z * to.z)));
		figure.player.connection.send(new net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket(figure.entityId,
				net.minecraft.world.entity.PositionPath.of(figure.at), yaw, pitch, true));
		var head = new net.minecraft.network.FriendlyByteBuf(io.netty.buffer.Unpooled.buffer());
		head.writeVarInt(figure.entityId);
		head.writeByte(net.minecraft.util.Mth.floor(yaw * 256f / 360f));
		figure.player.connection.send(net.minecraft.network.protocol.game.ClientboundRotateHeadPacket.STREAM_CODEC.decode(head));
	}

	/** The figure's pose to its player: crouching or standing. */
	@SuppressWarnings("unchecked")
	private static void pose(Figure figure, net.minecraft.world.entity.Pose pose) {
		try {
			var field = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_POSE");
			field.setAccessible(true);
			var accessor = (net.minecraft.network.syncher.EntityDataAccessor<net.minecraft.world.entity.Pose>) field.get(null);
			figure.player.connection.send(new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(figure.entityId,
					List.of(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(accessor, pose))));
		} catch (ReflectiveOperationException ignored) {
			// The pose field moved: the figure stands, nothing else lost.
		}
	}

	/** One figure's dig through the rock, on its player's client alone (ABI 35). */
	private static final class Dig {
		final ServerPlayer player;
		final int number;
		final ServerLevel level;
		final net.minecraft.core.Direction way;
		BlockPos at;
		int steps;
		final int depth;
		final boolean through;
		long nextTick;
		/** The blocks shown as air to the player, to give back. */
		final List<BlockPos> opened = new ArrayList<>();
		/** Hidden in the hill, waiting to come out of its far side. */
		boolean inside;

		Dig(ServerPlayer player, int number, ServerLevel level, net.minecraft.core.Direction way, BlockPos at, int depth, boolean through, long nextTick) {
			this.player = player;
			this.number = number;
			this.level = level;
			this.way = way;
			this.at = at;
			this.depth = depth;
			this.through = through;
			this.nextTick = nextTick;
		}
	}

	private final List<Dig> digs = new ArrayList<>();

	/**
	 * The figure walks from a place along a way to the first rock face (stone at its feet and
	 * head, within 24 blocks) and digs in; with {@code through}, it comes out of the far face a
	 * minute later. False when there is no rock face ahead.
	 */
	public boolean digIn(ServerLevel level, ServerPlayer player, int number, BlockPos from, net.minecraft.core.Direction way, boolean through) {
		BlockPos ground = BlockPos.containing(ground(level, new Vec3(from.getX() + 0.5, -10_000, from.getZ() + 0.5)));
		BlockPos at = ground;
		for (int i = 0; i < 24; i++) {
			BlockPos next = at.relative(way);
			BlockPos onGround = BlockPos.containing(ground(level, Vec3.atBottomCenterOf(next).with(net.minecraft.core.Direction.Axis.Y, -10_000)));
			if (rock(level, next.atY(at.getY())) && rock(level, next.atY(at.getY()).above())) {
				show(level, player, number, player, Vec3.atBottomCenterOf(at), List.of(), player.getMainHandItem().copy(), false, 0, 0);
				digs.add(new Dig(player, number, level, way, at, 6, through, tick + 30));
				return true;
			}
			at = onGround;
		}

		return false;
	}

	private static boolean rock(ServerLevel level, BlockPos pos) {
		var state = level.getBlockState(pos);

		return !state.isAir() && state.isSolidRender() && state.getBlock().getExplosionResistance() >= 3;
	}

	private void tickDigs() {
		Iterator<Dig> it = digs.iterator();
		while (it.hasNext()) {
			Dig dig = it.next();
			if (dig.player.isRemoved() || dig.player.level() != dig.level) {
				// The rock it opened, closed again on their client before it goes, when the
				// client still shows that world (a respawn in the same dimension); another
				// dimension's client dropped it, and those places are not this world's there.
				var now = dig.player.connection == null ? null : dig.player.connection.player;
				if (now != null && now.level() == dig.level) {
					for (BlockPos open : new ArrayList<>(dig.opened)) {
						giveBack(dig, open);
					}
				}
				remove(dig.player, dig.number);
				it.remove();
				continue;
			}
			if (tick < dig.nextTick) {
				continue;
			}
			if (dig.inside) {
				// Out of the far face: the first air along the way, within thirty blocks.
				BlockPos at = dig.at;
				for (int i = 0; i < 30; i++) {
					at = at.relative(dig.way);
					if (dig.level.getBlockState(at).isAir() && dig.level.getBlockState(at.above()).isAir()) {
						show(dig.level, dig.player, dig.number, dig.player, Vec3.atBottomCenterOf(at), List.of(Vec3.atCenterOf(at.relative(dig.way, 2)).with(net.minecraft.core.Direction.Axis.Y, -10_000)), dig.player.getMainHandItem().copy(), false, 6, 0);
						break;
					}
				}
				it.remove();
				continue;
			}
			// A step in: the rock ahead opened for them, the figure in it, the rock behind back.
			BlockPos next = dig.at.relative(dig.way);
			for (BlockPos open : List.of(next, next.above())) {
				dig.player.connection.send(new net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket(open, net.minecraft.world.level.block.Blocks.AIR.defaultBlockState()));
				dig.opened.add(open);
			}
			dig.player.playSound(net.minecraft.sounds.SoundEvents.STONE_BREAK, 0.6f, 0.9f);
			show(dig.level, dig.player, dig.number, dig.player, Vec3.atBottomCenterOf(next), List.of(), dig.player.getMainHandItem().copy(), false, 0, 0);
			BlockPos behind = dig.at;
			if (dig.steps > 0) {
				giveBack(dig, behind);
			}
			dig.at = next;
			dig.steps++;
			dig.nextTick = tick + 25;
			if (dig.steps >= dig.depth) {
				remove(dig.player, dig.number);
				for (BlockPos open : new ArrayList<>(dig.opened)) {
					giveBack(dig, open);
				}
				if (dig.through) {
					dig.inside = true;
					dig.nextTick = tick + 1_200;
				} else {
					it.remove();
				}
			}
		}
	}

	/** The real block at a place, sent back to the player: the rock closes. */
	private static void giveBack(Dig dig, BlockPos pos) {
		for (BlockPos p : List.of(pos, pos.above())) {
			dig.player.connection.send(new net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket(dig.level, p));
			dig.opened.remove(p);
		}
	}

	/** The player's view dotted with the way to the figure's chest: 1 is straight at it. */
	private static double viewDot(Figure figure) {
		Vec3 to = figure.at.add(0, 1.4, 0).subtract(figure.player.getEyePosition());
		double length = to.length();

		return length < 1e-3 ? 1 : figure.player.getViewVector(1f).dot(to.scale(1 / length));
	}

	/** Whether the player looks at a figure: within about 25 degrees of their view. */
	private static boolean looksAt(Figure figure) {
		Vec3 to = figure.at.add(0, 1.4, 0).subtract(figure.player.getEyePosition());
		double length = to.length();

		return length < 1e-3 || figure.player.getViewVector(1f).dot(to.scale(1 / length)) > 0.9;
	}

	/**
	 * The figure shown to a player with this number swings its arm; with damage above 0 the
	 * player is hit for it, the hurt's own jolt and sound, never below half a heart when
	 * {@code spare} (ABI 33). False when no such figure is shown.
	 */
	public boolean swing(ServerPlayer player, int number, float damage, boolean spare) {
		for (Figure figure : active) {
			if (figure.player == player && figure.number == number) {
				player.connection.send(new net.minecraft.network.protocol.game.ClientboundSwingAnimationPacket(figure.entityId,
						net.minecraft.world.InteractionHand.MAIN_HAND, net.minecraft.world.item.component.SwingAnimation.DEFAULT));
				if (damage > 0 && player.level() instanceof ServerLevel level) {
					float harm = spare ? Math.min(damage, Math.max(0, player.getHealth() - 1)) : damage;
					player.playSound(net.minecraft.sounds.SoundEvents.PLAYER_ATTACK_STRONG, 1f, 0.9f);
					if (harm > 0) {
						player.hurtServer(level, level.damageSources().generic(), harm);
					}
				}
				return true;
			}
		}

		return false;
	}

	/** A player held asleep while a figure chokes them (ABI 54). */
	private static final class Choke {
		final ServerPlayer player;
		final int number;
		final ItemStack item;
		final long fromTick;
		final long untilTick;
		final float damage;
		final boolean spare;
		boolean began;
		/** The unseen thing at their neck the lead's rope is tied to, once it is out; -1 before. */
		int knot = -1;

		Choke(ServerPlayer player, int number, ItemStack item, long fromTick, long untilTick, float damage, boolean spare) {
			this.player = player;
			this.number = number;
			this.item = item;
			this.fromTick = fromTick;
			this.untilTick = untilTick;
			this.damage = damage;
			this.spare = spare;
		}
	}

	/** The longest a choke squeezes. */
	static final float MOST_CHOKE_SECONDS = 6;

	private final List<Choke> chokes = new ArrayList<>();

	/**
	 * Hold {@code player}, asleep, in bed while the figure {@code number} walks from where it
	 * stands to their bed's side ({@link ChokeWay}), then for {@code seconds} let it squeeze
	 * once a second (ABI 54). It stays where its walk ends. False when they are not asleep or
	 * no such figure is shown.
	 */
	public boolean choke(ServerPlayer player, int number, ItemStack item, float seconds, float damage, boolean spare) {
		if (!player.isSleeping() || !(player.level() instanceof ServerLevel level)) {
			return false;
		}
		Figure figure = find(player, number);
		if (figure == null) {
			return false;
		}
		Vec3 to = bedside(level, player, figure.at);
		List<Vec3> walk = new ArrayList<>();
		for (double[] p : ChokeWay.walk(figure.at.x, figure.at.y, figure.at.z, to.x, to.y, to.z)) {
			walk.add(new Vec3(p[0], p[1], p[2]));
		}
		figure.path = walk;
		figure.pathIndex = 0;
		figure.stays = true;
		figure.glides = true;
		// It glides from the next tick on; it squeezes once at the bed.
		long arrives = tick + 1 + ChokeWay.ticks(ChokeWay.walk(figure.at.x, figure.at.y, figure.at.z, to.x, to.y, to.z));
		long squeezes = Math.round(Math.max(1, Math.min(seconds, MOST_CHOKE_SECONDS)) * 20);
		figure.untilTick = Math.max(figure.untilTick, arrives + squeezes + 40);
		chokes.removeIf(choke -> choke.player == player);
		chokes.add(new Choke(player, number, item, arrives, arrives + squeezes, damage, spare));

		return true;
	}

	/**
	 * Where the figure stands at the bed they lie in: the first of {@link ChokeWay#bedside}
	 * with room for it (nothing solid at its feet or head, the floor solid); where it is when
	 * none has.
	 */
	private static Vec3 bedside(ServerLevel level, ServerPlayer player, Vec3 from) {
		BlockPos head = player.getSleepingPos().orElse(null);
		if (head == null) {
			return from;
		}
		var state = level.getBlockState(head);
		if (!state.hasProperty(net.minecraft.world.level.block.HorizontalDirectionalBlock.FACING)) {
			return from;
		}
		var facing = state.getValue(net.minecraft.world.level.block.HorizontalDirectionalBlock.FACING);
		for (int[] spot : ChokeWay.bedside(head.getX(), head.getZ(), facing.getStepX(), facing.getStepZ(), from.x, from.z)) {
			BlockPos feet = new BlockPos(spot[0], head.getY(), spot[1]);
			if (open(level, feet) && open(level, feet.above()) && !open(level, feet.below())) {
				return new Vec3(feet.getX() + 0.5, feet.getY(), feet.getZ() + 0.5);
			}
		}

		return from;
	}

	private static boolean open(ServerLevel level, BlockPos pos) {
		return level.getBlockState(pos).getCollisionShape(level, pos).isEmpty();
	}

	private Figure find(ServerPlayer player, int number) {
		for (Figure figure : active) {
			if (figure.player == player && figure.number == number) {
				return figure;
			}
		}

		return null;
	}

	/** Each choke a tick on: their sleep held, the squeezes, its end when they leave the bed. */
	private void tickChokes() {
		Iterator<Choke> it = chokes.iterator();
		while (it.hasNext()) {
			Choke choke = it.next();
			ServerPlayer player = choke.player;
			// Out of bed, by Leave Bed or anything else: it lets go, and it is gone.
			if (player.isRemoved() || !player.isSleeping() || !(player.level() instanceof ServerLevel level)) {
				untie(player, choke);
				remove(player, choke.number);
				it.remove();
				continue;
			}
			if (tick >= choke.untilTick) {
				untie(player, choke);
				remove(player, choke.number);
				player.stopSleepInBed(true, true);
				it.remove();
				continue;
			}
			// The night does not pass under them: their sleep never grows deep enough.
			holdSleep(player);
			Figure figure = find(player, choke.number);
			if (tick < choke.fromTick) {
				continue;
			}
			// At the bed: it takes the item out.
			if (!choke.began) {
				choke.began = true;
				if (figure != null) {
					figure.held = choke.item;
					player.connection.send(new ClientboundSetEquipmentPacket(figure.entityId, List.of(com.mojang.datafixers.util.Pair.of(EquipmentSlot.MAINHAND, figure.held))));
					choke.knot = nextId++;
					tie(level, player, figure, choke.knot);
				}
				player.addEffect(new net.minecraft.world.effect.MobEffectInstance(net.minecraft.world.effect.MobEffects.DARKNESS,
						(int) (choke.untilTick - tick) + 40, 0, false, false));
			}
			if ((tick - choke.fromTick) % 20 == 0) {
				squeeze(level, player, figure, choke);
			}
		}
	}

	/** How far down the bed from their eyes the rope is tied, and how far below them. */
	static final double ROPE_DOWN_THE_BED = 0.65;
	static final double ROPE_BELOW = 0.15;

	/**
	 * The lead's rope from the figure's hand to their neck and chest, as the game draws a leashed
	 * mob's: an invisible, silent silverfish sent to them alone, its lead held by the figure. The
	 * held lead alone was hard to make out at the bedside (the owner). Tied down the bed from
	 * their eyes: tied where they lie (at their eyes), the rope ran straight at the camera, seen
	 * end on as a dot ("i still cant see the lead").
	 */
	@SuppressWarnings("unchecked")
	private static void tie(ServerLevel level, ServerPlayer player, Figure figure, int knot) {
		var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.get(net.minecraft.resources.Identifier.parse("minecraft:silverfish"));
		if (type.isEmpty()) {
			return;
		}
		Vec3 neck = player.position().add(0, -ROPE_BELOW, 0);
		BlockPos head = player.getSleepingPos().orElse(null);
		if (head != null && level.getBlockState(head).hasProperty(net.minecraft.world.level.block.HorizontalDirectionalBlock.FACING)) {
			// The bed faces foot to head: down it is the other way.
			var facing = level.getBlockState(head).getValue(net.minecraft.world.level.block.HorizontalDirectionalBlock.FACING);
			neck = neck.add(-facing.getStepX() * ROPE_DOWN_THE_BED, 0, -facing.getStepZ() * ROPE_DOWN_THE_BED);
		}
		player.connection.send(new ClientboundAddEntityPacket(knot, UUID.randomUUID(), neck.x, neck.y, neck.z, 0, 0, type.get().value(), 0, Vec3.ZERO, 0));
		try {
			var flags = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_SHARED_FLAGS_ID");
			var silent = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_SILENT");
			var floats = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_NO_GRAVITY");
			flags.setAccessible(true);
			silent.setAccessible(true);
			floats.setAccessible(true);
			player.connection.send(new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(knot, List.of(
					// Invisible: the shared flags' sixth bit.
					net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
							(net.minecraft.network.syncher.EntityDataAccessor<Byte>) flags.get(null), (byte) 0x20),
					net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
							(net.minecraft.network.syncher.EntityDataAccessor<Boolean>) silent.get(null), true),
					net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
							(net.minecraft.network.syncher.EntityDataAccessor<Boolean>) floats.get(null), true))));
		} catch (ReflectiveOperationException ignored) {
			// The fields moved: a silverfish they may see at their neck is worse than no rope.
			player.connection.send(new ClientboundRemoveEntitiesPacket(knot));
			return;
		}
		var link = new net.minecraft.network.FriendlyByteBuf(io.netty.buffer.Unpooled.buffer());
		link.writeInt(knot);
		link.writeInt(figure.entityId);
		player.connection.send(net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket.STREAM_CODEC.decode(link));
	}

	/** The rope gone with the thing it was tied to. */
	private static void untie(ServerPlayer player, Choke choke) {
		if (choke.knot >= 0 && player.connection != null) {
			player.connection.send(new ClientboundRemoveEntitiesPacket(choke.knot));
		}
	}

	/** One squeeze: its arm, the harm without waking them, a drowning gasp and the jolt. */
	private static void squeeze(ServerLevel level, ServerPlayer player, Figure figure, Choke choke) {
		if (figure != null) {
			player.connection.send(new net.minecraft.network.protocol.game.ClientboundSwingAnimationPacket(figure.entityId,
					net.minecraft.world.InteractionHand.MAIN_HAND, net.minecraft.world.item.component.SwingAnimation.DEFAULT));
		}
		float health = player.getHealth();
		float harm = choke.spare ? Spare.harm(choke.damage, health, 1f) : choke.damage;
		var drowning = level.damageSources().drown();
		if (harm >= health) {
			// Not spared and the last of them: a real death, which wakes them.
			player.hurtServer(level, drowning, harm);
			return;
		}
		if (harm > 0) {
			player.setHealth(health - harm);
		}
		// The hurt's jolt and the drowning sound, without the hurt that would wake them.
		level.broadcastDamageEvent(player, drowning);
	}

	/** Their sleep timer back to nothing, so the night waits (the game skips it at 100). */
	private static void holdSleep(ServerPlayer player) {
		try {
			var field = net.minecraft.world.entity.player.Player.class.getDeclaredField("sleepCounter");
			field.setAccessible(true);
			field.setInt(player, 0);
		} catch (ReflectiveOperationException ignored) {
			// The field moved: the night may pass under them, nothing else lost.
		}
	}

	private static Vec3 ground(ServerLevel level, Vec3 pos) {
		if (pos.y > -1000) {
			return pos;
		}
		BlockPos column = BlockPos.containing(pos.x, 0, pos.z);
		if (!level.isLoaded(column)) {
			return new Vec3(pos.x, level.getSeaLevel() + 1, pos.z);
		}
		// Leaves are not ground: a figure in a forest stands under the trees, not on them.
		BlockPos surface = level.getHeightmapPos(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, column);

		return new Vec3(pos.x, surface.getY(), pos.z);
	}

	private void send(ServerLevel level, Figure figure, boolean nameShown) {
		var entry = new ClientboundPlayerInfoUpdatePacket.Entry(figure.uuid, figure.profile, false, 0, GameType.SURVIVAL, null, nameShown, 0, null);
		var info = new ClientboundPlayerInfoUpdatePacket(EnumSet.of(ClientboundPlayerInfoUpdatePacket.Action.ADD_PLAYER, ClientboundPlayerInfoUpdatePacket.Action.UPDATE_LISTED), List.of());
		Executors.withEntries(info, List.of(entry));
		figure.player.connection.send(info);
		sendEntity(figure);
	}

	private void sendEntity(Figure figure) {
		// Facing the player: yaw from the figure to them.
		Vec3 to = figure.player.position().subtract(figure.at);
		float yaw = (float) Math.toDegrees(Math.atan2(-to.x, to.z));
		float pitch = (float) -Math.toDegrees(Math.atan2(to.y, Math.sqrt(to.x * to.x + to.z * to.z)));
		// The player type by its id: 26.3 keeps the entity types in the registry, not as constants.
		EntityType<?> playerType = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.get(net.minecraft.resources.Identifier.parse("minecraft:player"))
				.map(holder -> (EntityType<?>) holder.value())
				.orElseThrow(() -> new IllegalStateException("no player entity type"));
		figure.player.connection.send(new ClientboundAddEntityPacket(figure.entityId, figure.uuid, figure.at.x, figure.at.y, figure.at.z, pitch, yaw, playerType, 0, Vec3.ZERO, yaw));
		if (!figure.held.isEmpty()) {
			figure.player.connection.send(new ClientboundSetEquipmentPacket(figure.entityId, List.of(com.mojang.datafixers.util.Pair.of(EquipmentSlot.MAINHAND, figure.held))));
		}
	}

	private void unsend(Figure figure) {
		if (figure.player.connection == null) {
			return;
		}
		figure.player.connection.send(new ClientboundRemoveEntitiesPacket(figure.entityId));
		figure.player.connection.send(new ClientboundPlayerInfoRemovePacket(List.of(figure.uuid)));
	}

	public int count() {
		return active.size();
	}
}
