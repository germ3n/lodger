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
		final int number;
		final int entityId;
		final UUID uuid;
		final GameProfile profile;
		final ItemStack held;
		Vec3 at;
		final List<Vec3> path;
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

	private final List<Figure> active = new ArrayList<>();
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
		remove(player, number);

		Vec3 at = ground(level, pos);
		UUID uuid = UUID.randomUUID();
		GameProfile profile = new GameProfile(uuid, skinOf.getGameProfile().name(), skinOf.getGameProfile().properties());
		Figure figure = new Figure(player, number, nextId++, uuid, profile, held, at, path, seconds > 0 ? tick + (long) (seconds * 20) : Long.MAX_VALUE, vanishWithin);
		figure.mirror = mirror;
		figure.shownAt = at;
		figure.playerAt = player.position();
		figure.onceSeen = onceSeen;
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

	/** Once a tick: the paths walked, the vanishing, the time. */
	public void tick() {
		tick++;
		Iterator<Figure> it = active.iterator();
		while (it.hasNext()) {
			Figure figure = it.next();
			boolean looked = looksAt(figure);
			if (looked) {
				figure.seen = true;
			}
			boolean gone = figure.player.isRemoved() || tick > figure.untilTick
					|| (figure.vanishWithin > 0 && figure.player.position().distanceTo(figure.at) < figure.vanishWithin)
					|| (figure.onceSeen && figure.seen && !looked);
			if (gone) {
				if (!figure.player.isRemoved()) {
					unsend(figure);
				}
				it.remove();
				continue;
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

			if (!figure.path.isEmpty() && tick % STEP_TICKS == 0 && figure.player.level() instanceof ServerLevel level) {
				figure.pathIndex++;
				if (figure.pathIndex >= figure.path.size()) {
					unsend(figure);
					it.remove();
					continue;
				}
				// A step: the entity is re-sent at the next point, which at the figures'
				// distances reads as a stride.
				figure.player.connection.send(new ClientboundRemoveEntitiesPacket(figure.entityId));
				figure.at = ground(level, figure.path.get(figure.pathIndex));
				sendEntity(figure);
			}
		}
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
		figure.player.connection.send(new ClientboundRemoveEntitiesPacket(figure.entityId));
		figure.player.connection.send(new ClientboundPlayerInfoRemovePacket(List.of(figure.uuid)));
	}

	public int count() {
		return active.size();
	}
}
