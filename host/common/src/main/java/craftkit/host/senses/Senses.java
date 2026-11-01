package craftkit.host.senses;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.Container;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.entity.TamableAnimal;
import net.minecraft.world.entity.monster.Enemy;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.LightLayer;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.HitResult;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * What the host sees, gathered into the SDK's {@code Observations} once a tick per level.
 *
 * <p>Two kinds of input: the loaders' events (a player joined, used a block, broke one, slept,
 * died, typed in chat), which the loader hooks push in through the {@code noticed...} methods
 * and which drain into the next tick's observations; and what is read off the level each tick:
 * every player's state and movement, every entity within 64 blocks of a player once a second,
 * the weather once a second. Nothing here is more than the server knows anyway.
 *
 * <p>This class is the Minecraft-facing edge of the host, with {@code world/Executors} and the
 * client side: a new game version is absorbed here, and the protocol never moves for it
 * ({@code docs/development.md}, Updating Minecraft).
 */
public final class Senses {
	/** How far around a player entities and blocks are looked at. */
	public static final double REACH = 64;
	private static final double LOOK_RANGE = 8;
	/** `-Dcraftkit.debug=true` logs every event the senses take in, for tracing what reaches a mod. */
	public static final boolean DEBUG = Boolean.getBoolean("craftkit.debug");

	private final Map<UUID, Integer> indices = new LinkedHashMap<>();
	private final Map<UUID, MovementTracker> movement = new HashMap<>();
	private final Map<UUID, Boolean> wasSleeping = new HashMap<>();
	private final Map<UUID, String> wasRiding = new HashMap<>();
	private final Map<UUID, Boolean> hadContainerOpen = new HashMap<>();
	/** The container each player opened last, so its close can say what it holds now. */
	private final Map<UUID, BlockPos> openedAt = new HashMap<>();
	/** Players whose next sleep attempt is refused, and until which tick. */
	private final Map<UUID, Long> refuseSleepUntil = new HashMap<>();
	private final Map<ServerLevel, List<JsonObject>> events = new HashMap<>();
	private final Map<ServerLevel, List<JsonObject>> chat = new HashMap<>();
	private final Map<ServerLevel, List<JsonObject>> scans = new HashMap<>();
	private final Map<ServerLevel, JsonObject> surroundings = new HashMap<>();
	private boolean modsSent;
	private final List<JsonObject> mods = new ArrayList<>();
	private long tick;
	/** The last events taken in, newest last, for the debug screen. */
	private final java.util.ArrayDeque<String> recentEvents = new java.util.ArrayDeque<>();
	private static final int MOST_RECENT = 30;
	/** The last observation of each player, kept for the debug screen's meters. */
	private final Map<UUID, JsonObject> lastSeen = new HashMap<>();

	/** The index the SDK names a player by, stable for the server's life. */
	public int indexOf(ServerPlayer player) {
		return indices.computeIfAbsent(player.getUUID(), uuid -> indices.size());
	}

	/** The player with this index in this level, or null. */
	public ServerPlayer player(ServerLevel level, int index) {
		for (ServerPlayer player : level.players()) {
			if (indexOf(player) == index) {
				return player;
			}
		}

		return null;
	}

	/** The other mods loaded, told once, on the first tick. */
	public void mods(List<String[]> idsAndVersions) {
		for (String[] mod : idsAndVersions) {
			JsonObject object = new JsonObject();
			object.addProperty("id", mod[0]);
			object.addProperty("version", mod[1]);
			mods.add(object);
		}
	}

	// ---- what the loaders' hooks tell us

	public void noticedJoined(ServerPlayer player) {
		event(player, "joined", null, "", null);
	}

	public void noticedLeft(ServerPlayer player) {
		event(player, "left", null, "", null);
	}

	/** What the host does with a chest's fresh loot, just unpacked: a mod's offering (`LootOffers`). */
	public interface LootHook {
		/** The item placed, "" for an offering spent with nothing to give, or null for none. */
		String touched(ServerPlayer player, Container chest);
	}

	private LootHook lootHook = (player, chest) -> null;

	public void lootHook(LootHook hook) {
		lootHook = hook;
	}

	/** A block used: if it holds a container, that is a container opened, with its contents. */
	public void noticedUsedBlock(ServerPlayer player, BlockPos pos) {
		BlockEntity entity = player.level().getBlockEntity(pos);

		// Fresh loot: unpacked now, as the game would on opening, so an offering can join it
		// before the chest is seen (the hook runs before the block's own use).
		if (entity instanceof net.minecraft.world.RandomizableContainer loot && loot.getLootTable() != null && entity instanceof Container chest) {
			loot.unpackLootTable(player);
			String touched = lootHook.touched(player, chest);
			if (touched != null) {
				event(player, "loot_touched", pos, touched, null);
			}
		}

		if (DEBUG) {
			craftkit.host.Log.LOGGER.info("sense: used {} at {} ({})", blockId(player.level().getBlockState(pos)), pos.toShortString(), entity == null ? "no block entity" : entity.getClass().getSimpleName());
		}

		if (player.level().getBlockState(pos).getBlock() instanceof net.minecraft.world.level.block.DoorBlock) {
			event(player, "used_door", pos, blockId(player.level().getBlockState(pos)), null);
		}
		if (entity instanceof Container container) {
			openedAt.put(player.getUUID(), pos.immutable());
			event(player, "opened_container", pos, blockId(player.level().getBlockState(pos)), contents(container));
		}
	}

	public void noticedPlacedBlock(ServerPlayer player, BlockPos pos, BlockState state) {
		event(player, "placed_block", pos, blockId(state), null);
	}

	public void noticedBrokeBlock(ServerPlayer player, BlockPos pos, BlockState state) {
		event(player, "broke_block", pos, blockId(state), null);
	}

	public void noticedSleepRefused(ServerPlayer player, BlockPos pos, String why) {
		event(player, "sleep_refused", pos, why, null);
	}

	public void noticedDied(ServerPlayer player) {
		event(player, "died", player.blockPosition(), "", null);
	}

	public void noticedRespawned(ServerPlayer player) {
		event(player, "respawned", player.blockPosition(), "", null);
	}

	public void noticedChangedDimension(ServerPlayer player, String from, String to) {
		event(player, "changed_dimension", null, to, null);
	}

	public void noticedHit(ServerPlayer player, Entity target) {
		event(player, "hit", target.blockPosition(), entityId(target), null);
	}

	/** A player killed something: the attacker of the killing blow was the player. */
	public void noticedKilled(ServerPlayer player, Entity victim) {
		event(player, "killed", victim.blockPosition(), entityId(victim), null);
	}

	public void noticedChat(ServerPlayer player, String text) {
		JsonObject line = new JsonObject();
		line.addProperty("player", indexOf(player));
		line.addProperty("text", text);
		chat.computeIfAbsent(player.level() instanceof ServerLevel level ? level : null, l -> new ArrayList<>()).add(line);
	}

	/** An answered scan, for the next tick's observations. */
	public void scanned(ServerLevel level, JsonObject scan) {
		scans.computeIfAbsent(level, l -> new ArrayList<>()).add(scan);
	}

	/** A finished surroundings pass, for the next tick's observations. */
	public void surrounded(ServerLevel level, JsonObject result) {
		surroundings.put(level, result);
	}

	// ---- sleep refusal, armed by the cancel_sleep command

	public void refuseSleep(ServerPlayer player, float seconds) {
		refuseSleepUntil.put(player.getUUID(), tick + (long) (seconds * MovementTracker.TPS));
	}

	/** Whether this player's sleep attempt is to be refused now; consumes the arming. */
	public boolean shouldRefuseSleep(ServerPlayer player) {
		Long until = refuseSleepUntil.get(player.getUUID());

		if (until == null) {
			return false;
		}

		if (tick > until) {
			refuseSleepUntil.remove(player.getUUID());

			return false;
		}

		refuseSleepUntil.remove(player.getUUID());

		return true;
	}

	// ---- the per-tick observation

	/** Everything seen in this level since the last call, as the SDK's {@code Observations}. */
	public JsonObject observe(ServerLevel level, long serverTick) {
		tick = serverTick;
		JsonObject observations = new JsonObject();
		JsonArray players = new JsonArray();
		List<ServerPlayer> present = level.players();
		boolean second = serverTick % MovementTracker.TPS == 0;

		for (ServerPlayer player : present) {
			players.add(observePlayer(level, player));
		}

		observations.add("players", players);

		if (second && !present.isEmpty()) {
			observations.add("entities", observeEntities(level, present));
			JsonObject world = new JsonObject();
			world.addProperty("raining", level.isRaining());
			world.addProperty("thundering", level.isThundering());
			world.addProperty("moon_phase", 0);
			observations.add("world", world);
		}

		List<JsonObject> queued = events.remove(level);
		if (queued != null) {
			JsonArray array = new JsonArray();
			queued.forEach(array::add);
			observations.add("events", array);
		}

		List<JsonObject> lines = chat.remove(level);
		if (lines != null) {
			JsonArray array = new JsonArray();
			lines.forEach(array::add);
			observations.add("chat", array);
		}

		List<JsonObject> answered = scans.remove(level);
		if (answered != null) {
			JsonArray array = new JsonArray();
			answered.forEach(array::add);
			observations.add("scans", array);
		}

		JsonObject around = surroundings.remove(level);
		if (around != null) {
			observations.add("surroundings", around);
		}

		if (!modsSent && !mods.isEmpty()) {
			JsonArray array = new JsonArray();
			mods.forEach(array::add);
			observations.add("mods", array);
			modsSent = true;
		}

		return observations;
	}

	private JsonObject observePlayer(ServerLevel level, ServerPlayer player) {
		UUID uuid = player.getUUID();
		Vec3 pos = player.position();
		MovementTracker tracker = movement.computeIfAbsent(uuid, u -> new MovementTracker());
		tracker.sample(pos.x, pos.y, pos.z, player.getYRot(), player.isSprinting());

		// Things read off state changes, so no loader needs an event for them.
		boolean sleeping = player.isSleeping();
		Boolean slept = wasSleeping.put(uuid, sleeping);
		if (slept != null && slept != sleeping) {
			event(player, sleeping ? "slept_began" : "slept_ended", player.blockPosition(), "", null);
		}

		String riding = player.getVehicle() == null ? "" : entityId(player.getVehicle());
		String rode = wasRiding.put(uuid, riding);
		if (rode != null && !rode.equals(riding)) {
			event(player, riding.isEmpty() ? "dismounted" : "mounted", player.blockPosition(), riding.isEmpty() ? rode : riding, null);
		}

		boolean containerOpen = player.containerMenu != player.inventoryMenu;
		Boolean hadOpen = hadContainerOpen.put(uuid, containerOpen);
		if (hadOpen != null && hadOpen && !containerOpen) {
			// The contents as the player left them: what they took or put in while it was open.
			BlockPos at = openedAt.remove(uuid);
			BlockEntity entity = at == null ? null : player.level().getBlockEntity(at);
			if (entity instanceof Container container) {
				event(player, "closed_container", at, blockId(player.level().getBlockState(at)), contents(container));
			} else {
				event(player, "closed_container", null, "", null);
			}
		}

		JsonObject p = new JsonObject();
		p.addProperty("index", indexOf(player));
		p.addProperty("name", player.getName().getString());
		p.addProperty("uuid", uuid.toString());
		p.add("pos", vec(pos));
		p.addProperty("yaw", player.getYRot());
		p.addProperty("pitch", player.getXRot());
		p.add("velocity", vec(player.getDeltaMovement()));
		p.addProperty("sneaking", player.isShiftKeyDown());
		p.addProperty("sprinting", player.isSprinting());
		p.addProperty("swimming", player.isSwimming());
		p.addProperty("sleeping", sleeping);
		p.addProperty("riding", riding);
		p.addProperty("health", player.getHealth());
		p.addProperty("hunger", player.getFoodData().getFoodLevel());
		p.addProperty("air", player.getAirSupply());
		p.addProperty("level", player.experienceLevel);
		p.add("main_hand", stack(player.getMainHandItem()));
		p.add("off_hand", stack(player.getOffhandItem()));
		JsonArray armor = new JsonArray();
		for (EquipmentSlot slot : new EquipmentSlot[] {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET}) {
			armor.add(stack(player.getItemBySlot(slot)));
		}
		p.add("armor", armor);
		JsonArray effects = new JsonArray();
		for (MobEffectInstance effect : player.getActiveEffects()) {
			effects.add(effect.getEffect().unwrapKey().map(key -> key.identifier().toString()).orElse("?"));
		}
		p.add("effects", effects);
		p.addProperty("dimension", level.dimension().identifier().toString());
		BlockPos at = player.blockPosition();
		p.addProperty("biome", level.getBiome(at).unwrapKey().map(key -> key.identifier().toString()).orElse("?"));
		p.addProperty("light", level.getLightEngine().getLayerListener(LightLayer.BLOCK).getLightValue(at));
		p.addProperty("sky_light", level.getLightEngine().getLayerListener(LightLayer.SKY).getLightValue(at));

		JsonObject looking = lookingAt(level, player);
		if (looking != null) {
			p.add("looking_at", looking);
		}

		JsonObject move = new JsonObject();
		move.addProperty("pace", tracker.pace());
		move.addProperty("stops", tracker.stops());
		move.addProperty("turns", tracker.turns());
		move.addProperty("look_backs", tracker.lookBacks());
		move.addProperty("backing_away", tracker.backingAway());
		move.addProperty("sprint_bursts", tracker.sprintBursts());
		move.addProperty("still_seconds", tracker.stillSeconds());
		p.add("movement", move);
		lastSeen.put(uuid, p);

		return p;
	}

	private JsonObject lookingAt(ServerLevel level, ServerPlayer player) {
		Vec3 eye = player.getEyePosition();
		Vec3 view = player.getViewVector(1.0f);
		Entity nearest = null;
		double nearestDistance = LOOK_RANGE;
		for (Entity entity : level.getEntities(player, player.getBoundingBox().inflate(LOOK_RANGE), e -> e != player && !(e instanceof net.minecraft.world.entity.item.ItemEntity))) {
			Vec3 to = entity.position().add(0, entity.getBbHeight() / 2, 0).subtract(eye);
			double distance = to.length();
			if (distance < nearestDistance && to.normalize().dot(view) > Math.cos(Math.toRadians(5))) {
				nearest = entity;
				nearestDistance = distance;
			}
		}

		if (nearest != null) {
			JsonObject target = new JsonObject();
			target.addProperty("kind", "entity");
			target.addProperty("id", nearest.getId());
			target.addProperty("entity", entityId(nearest));

			return target;
		}

		HitResult hit = player.pick(LOOK_RANGE, 1.0f, false);
		if (hit instanceof BlockHitResult block && hit.getType() == HitResult.Type.BLOCK) {
			JsonObject target = new JsonObject();
			target.addProperty("kind", "block");
			target.add("pos", blockPos(block.getBlockPos()));
			target.addProperty("block", blockId(level.getBlockState(block.getBlockPos())));

			return target;
		}

		return null;
	}

	private JsonArray observeEntities(ServerLevel level, List<ServerPlayer> players) {
		JsonArray array = new JsonArray();
		Map<Integer, Entity> seen = new HashMap<>();
		for (ServerPlayer player : players) {
			for (Entity entity : level.getEntities(player, player.getBoundingBox().inflate(REACH), e -> !(e instanceof ServerPlayer))) {
				seen.putIfAbsent(entity.getId(), entity);
			}
		}

		for (Entity entity : seen.values()) {
			JsonObject e = new JsonObject();
			e.addProperty("id", entity.getId());
			e.addProperty("kind", entityId(entity));
			e.add("pos", vec(entity.position()));
			e.addProperty("name", entity.hasCustomName() && entity.getCustomName() != null ? entity.getCustomName().getString() : "");
			e.addProperty("owner", entity instanceof TamableAnimal tamed && tamed.getOwnerReference() != null ? tamed.getOwnerReference().getUUID().toString() : "");
			e.addProperty("hostile", entity instanceof Enemy);
			// Tied by a lead to a fence post (ABI 27, `horse_moved`).
			e.addProperty("leashed", entity instanceof net.minecraft.world.entity.Leashable leashable && leashable.isLeashed()
					&& leashable.getLeashHolder() instanceof net.minecraft.world.entity.decoration.LeashFenceKnotEntity);

			if (entity instanceof LivingEntity living) {
				Vec3 view = living.getViewVector(1.0f);
				for (ServerPlayer player : players) {
					Vec3 to = player.getEyePosition().subtract(living.getEyePosition());
					if (to.length() < REACH && to.normalize().dot(view) > Math.cos(Math.toRadians(10))) {
						e.addProperty("looking_at_player", indexOf(player));
						break;
					}
				}
			}

			array.add(e);
		}

		return array;
	}

	private void event(ServerPlayer player, String kind, BlockPos pos, String detail, JsonArray contents) {
		JsonObject event = new JsonObject();
		event.addProperty("player", indexOf(player));
		event.addProperty("kind", kind);
		if (pos != null) {
			event.add("pos", blockPos(pos));
		}
		event.addProperty("detail", detail);
		if (contents != null) {
			event.add("contents", contents);
		}

		if (player.level() instanceof ServerLevel level) {
			events.computeIfAbsent(level, l -> new ArrayList<>()).add(event);
		}

		recentEvents.addLast(String.format("%d %s%s%s", tick, kind, pos == null ? "" : " at " + pos.toShortString(), detail.isEmpty() ? "" : " " + detail));
		while (recentEvents.size() > MOST_RECENT) {
			recentEvents.removeFirst();
		}

		if (DEBUG) {
			craftkit.host.Log.LOGGER.info("sense: {}", event);
		}
	}

	/**
	 * The senses' own page for the debug screen: this player's last observation as meters and
	 * lines, the entities near them by kind, the world's state, the last events taken in. The
	 * client lays it out; see {@code client/debug/DebugModel}.
	 */
	public JsonObject panel(ServerLevel level, ServerPlayer player) {
		JsonObject panel = new JsonObject();
		JsonObject seen = lastSeen.getOrDefault(player.getUUID(), new JsonObject());
		panel.add("player", seen);
		panel.addProperty("index", indexOf(player));
		panel.addProperty("tick", tick);
		long time = level.getLevelData().getGameTime();
		panel.addProperty("day_time", time % 24000L);
		panel.addProperty("day", time / 24000L);
		panel.addProperty("raining", level.isRaining());
		panel.addProperty("thundering", level.isThundering());
		panel.addProperty("players", level.players().size());

		java.util.Map<String, Integer> kinds = new java.util.TreeMap<>();
		for (Entity entity : level.getEntities(player, player.getBoundingBox().inflate(REACH), e -> !(e instanceof ServerPlayer))) {
			kinds.merge(entityId(entity).replace("minecraft:", ""), 1, Integer::sum);
		}
		JsonArray near = new JsonArray();
		kinds.entrySet().stream()
				.sorted((a, b) -> b.getValue() - a.getValue())
				.limit(12)
				.forEach(entry -> near.add(entry.getValue() + " " + entry.getKey()));
		panel.add("entities", near);

		JsonArray events = new JsonArray();
		recentEvents.descendingIterator().forEachRemaining(events::add);
		panel.add("events", events);

		return panel;
	}

	static JsonArray contents(Container container) {
		JsonArray array = new JsonArray();
		for (int slot = 0; slot < container.getContainerSize(); slot++) {
			ItemStack stack = container.getItem(slot);
			if (!stack.isEmpty()) {
				JsonObject entry = stack(stack);
				entry.addProperty("slot", slot);
				array.add(entry);
			}
		}

		return array;
	}

	public static String blockId(BlockState state) {
		return BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString();
	}

	public static String entityId(Entity entity) {
		return BuiltInRegistries.ENTITY_TYPE.getKey(entity.getType()).toString();
	}

	public static JsonArray vec(Vec3 v) {
		JsonArray array = new JsonArray();
		array.add(v.x);
		array.add(v.y);
		array.add(v.z);

		return array;
	}

	public static JsonArray blockPos(BlockPos pos) {
		JsonArray array = new JsonArray();
		array.add(pos.getX());
		array.add(pos.getY());
		array.add(pos.getZ());

		return array;
	}

	/** `minecraft:iron_pickaxe`, or empty for nothing; the id the protocol carries. */
	public static String itemId(ItemStack stack) {
		return stack.isEmpty() ? "" : BuiltInRegistries.ITEM.getKey(stack.getItem()).toString();
	}

	public static JsonObject stack(ItemStack stack) {
		JsonObject object = new JsonObject();
		object.addProperty("id", stack.isEmpty() ? "" : BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
		object.addProperty("count", stack.isEmpty() ? 0 : stack.getCount());

		return object;
	}

	/** A mob's look, for the look_at command. Null for an entity that cannot turn its head. */
	public static Mob mob(Entity entity) {
		return entity instanceof Mob mob ? mob : null;
	}

	/** The box {@code REACH} blocks around a point. */
	public static AABB around(Vec3 center, double radius) {
		return new AABB(center.x - radius, center.y - radius, center.z - radius, center.x + radius, center.y + radius, center.z + radius);
	}
}
