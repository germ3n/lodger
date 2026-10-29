package craftkit.host.world;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import craftkit.host.LoadedMod;
import craftkit.host.Log;
import craftkit.host.net.HintPayload;
import craftkit.host.senses.Senses;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundSoundPacket;
import net.minecraft.resources.Identifier;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvent;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.Container;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.DoorBlock;
import net.minecraft.world.level.block.LeverBlock;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.SignBlockEntity;
import net.minecraft.world.level.block.entity.SignText;
import net.minecraft.world.level.block.entity.SignTextSlot;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.phys.Vec3;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.BiConsumer;

/**
 * Carries one command out against the level and answers its report. Each command checks what it
 * needs as it runs ({@code fits}, in the design's words) and reports {@code done} or why not; a
 * command the host does not know is reported as such rather than dropped, so a mod sees it.
 *
 * <p>With {@link Senses}, this is the Minecraft-facing edge of the host: a new game version is
 * absorbed here and the protocol never moves for it.
 */
public final class Executors {
	/** The most blocks one scan may cover: 40 by 40 by 25, in one tick. */
	private static final long SCAN_LIMIT = 40_000;

	/** What an executor needs beyond the level: the players by index, the senses to answer into, the hint sender. */
	public record Context(Senses senses, BiConsumer<ServerPlayer, HintPayload> sendHint, Looks looks, Later later, Poses poses, Figures figures, Holds holds) {
	}

	private Executors() {
	}

	public static JsonObject run(LoadedMod mod, ServerLevel level, JsonObject command, Context context, WorldData data) {
		long id = command.has("id") ? command.get("id").getAsLong() : 0;
		String kind = command.has("kind") ? command.get("kind").getAsString() : "";

		try {
			return switch (kind) {
				case "log" -> {
					Log.LOGGER.info("{}: {}", mod.id(), command.get("text").getAsString());
					yield report(id, true, "");
				}
				case "system_message" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					player.sendSystemMessage(Component.literal(command.get("text").getAsString()));
					yield report(id, true, "");
				}
				case "play_sound" -> playSound(level, command, id, context);
				case "hint" -> hint(level, command, id, context);
				case "scan" -> scan(level, command, id, context);
				case "surroundings" -> {
					Surroundings.start(level, mod, id, blockPos(command.getAsJsonArray("around")), command.get("radius").getAsInt(), context.senses());
					yield report(id, true, "started; the answer comes as an observation");
				}
				case "toggle_block" -> toggleBlock(level, blockPos(command.getAsJsonArray("pos")), id);
				case "move_block" -> moveBlock(level, command, id);
				case "set_block" -> setBlock(level, command, id);
				case "open_lid" -> openLid(level, blockPos(command.getAsJsonArray("pos")), command.get("seconds").getAsFloat(), id, context);
				case "rotate_frame" -> rotateFrame(level, command, id);
				case "swap_painting" -> swapPainting(level, command.get("entity").getAsInt(), id);
				case "pose_stand" -> poseStand(level, command, id, context);
				case "name_entity" -> nameEntity(level, command.get("entity").getAsInt(), command.get("name").getAsString(), id);
				case "dye_sheep" -> dyeSheep(level, command.get("entity").getAsInt(), command.get("colour").getAsString(), id);
				case "move_entity" -> moveEntity(level, command, id, context);
				case "push_entity" -> pushEntity(level, command, id);
				case "rotate_block" -> rotateBlock(level, blockPos(command.getAsJsonArray("pos")), id);
				case "break_block" -> breakBlock(level, blockPos(command.getAsJsonArray("pos")), id);
				case "set_frame_item" -> setFrameItem(level, command, id);
				case "equip_entity" -> equipEntity(level, command, id);
				case "damage_item" -> damageItem(level, command, id);
				case "figure" -> figure(level, command, id, context);
				case "remove_figure" -> {
					ServerPlayer player = context.senses().player(level, command.has("player") ? command.get("player").getAsInt() : 0);
					boolean removed = player != null && context.figures().remove(player, command.get("figure").getAsInt());
					yield report(id, removed, removed ? "" : "no such figure");
				}
				case "player_list" -> playerList(level, command, id, context);
				case "game_message" -> gameMessage(level, command, id, context);
				case "move_item" -> moveItem(level, command, id);
				case "take_item" -> takeItem(level, command, id, data);
				case "put_item" -> putItem(level, command, id, data);
				case "drop_item" -> dropItem(level, command, id, data);
				case "rename_item" -> renameItem(level, command, id);
				case "sign_book" -> signBook(level, command, id);
				case "mark_map" -> markMap(level, command, id);
				case "set_sign" -> setSign(level, command, id);
				case "cancel_sleep" -> cancelSleep(level, command, id, context);
				case "look_at" -> lookAt(level, command, id, context);
				case "hold" -> hold(level, command, id, context);
				case "move_player" -> movePlayer(level, command, id, context);
				case "block_sound" -> blockSound(level, command, id, context);
				default -> report(id, false, "the host does not carry out " + kind + " yet");
			};
		} catch (RuntimeException e) {
			return report(id, false, kind + ": " + e.getMessage());
		}
	}

	private static JsonObject playSound(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}

		Optional<Holder.Reference<SoundEvent>> sound = BuiltInRegistries.SOUND_EVENT.get(Identifier.parse(command.get("sound").getAsString()));
		if (sound.isEmpty()) {
			return report(id, false, "no such sound");
		}

		JsonArray pos = command.getAsJsonArray("pos");
		player.connection.send(new ClientboundSoundPacket(sound.get(), SoundSource.AMBIENT,
				pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble(),
				command.get("volume").getAsFloat(), command.get("pitch").getAsFloat(), level.getRandom().nextLong()));

		return report(id, true, "");
	}

	private static JsonObject blockSound(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		String which = command.get("block").getAsString();
		BlockState state;
		if (which.equals("under")) {
			state = level.getBlockState(player.blockPosition().below());
		} else if (which.equals("held")) {
			state = player.getMainHandItem().getItem() instanceof net.minecraft.world.item.BlockItem held
					? held.getBlock().defaultBlockState()
					: Blocks.STONE.defaultBlockState();
		} else {
			Optional<Holder.Reference<Block>> block = BuiltInRegistries.BLOCK.get(Identifier.parse(which));
			if (block.isEmpty()) {
				return report(id, false, "no such block");
			}
			state = block.get().value().defaultBlockState();
		}
		if (state.isAir()) {
			state = Blocks.STONE.defaultBlockState();
		}
		net.minecraft.world.level.block.SoundType type = state.getSoundType();
		SoundEvent sound = switch (command.get("of").getAsString()) {
			case "place" -> type.getPlaceSound();
			case "break" -> type.getBreakSound();
			case "hit" -> type.getHitSound();
			default -> type.getStepSound();
		};

		JsonArray pos = command.getAsJsonArray("pos");
		player.connection.send(new ClientboundSoundPacket(Holder.direct(sound), SoundSource.BLOCKS,
				pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble(),
				command.get("volume").getAsFloat() * type.getVolume(), command.get("pitch").getAsFloat() * type.getPitch(), level.getRandom().nextLong()));

		return report(id, true, "");
	}

	private static JsonObject hint(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}

		Optional<BlockPos> pos = command.has("pos") && !command.get("pos").isJsonNull() ? Optional.of(blockPos(command.getAsJsonArray("pos"))) : Optional.empty();
		context.sendHint().accept(player, new HintPayload(command.get("hint").getAsString(), pos, command.get("seconds").getAsFloat(), command.has("detail") ? command.get("detail").getAsString() : ""));

		return report(id, true, "");
	}

	private static JsonObject scan(ServerLevel level, JsonObject command, long id, Context context) {
		BlockPos min = blockPos(command.getAsJsonArray("min"));
		BlockPos max = blockPos(command.getAsJsonArray("max"));
		long volume = (long) (Math.abs(max.getX() - min.getX()) + 1) * (Math.abs(max.getY() - min.getY()) + 1) * (Math.abs(max.getZ() - min.getZ()) + 1);
		if (volume > SCAN_LIMIT) {
			return report(id, false, "the box holds " + volume + " blocks; at most " + SCAN_LIMIT);
		}

		List<String> wanted = new ArrayList<>();
		for (JsonElement kind : command.getAsJsonArray("kinds")) {
			wanted.add(kind.getAsString());
		}

		java.util.Map<String, Integer> counts = new java.util.TreeMap<>();
		JsonArray positions = new JsonArray();
		for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
			String block = Senses.blockId(level.getBlockState(pos));
			counts.merge(block, 1, Integer::sum);
			if (wanted.contains(block)) {
				JsonArray entry = new JsonArray();
				entry.add(block);
				entry.add(Senses.blockPos(pos));
				positions.add(entry);
			}
		}

		JsonObject answer = new JsonObject();
		answer.addProperty("id", id);
		JsonArray countsArray = new JsonArray();
		counts.forEach((block, count) -> {
			JsonArray entry = new JsonArray();
			entry.add(block);
			entry.add(count);
			countsArray.add(entry);
		});
		answer.add("counts", countsArray);
		answer.add("positions", positions);
		context.senses().scanned(level, answer);

		return report(id, true, "");
	}

	private static JsonObject toggleBlock(ServerLevel level, BlockPos pos, long id) {
		BlockState state = level.getBlockState(pos);
		Block block = state.getBlock();

		if (block instanceof DoorBlock door) {
			door.setOpen(null, level, state, pos, !door.isOpen(state));
			return report(id, true, "");
		}

		if (block instanceof LeverBlock lever) {
			lever.pull(state, level, pos, null);
			return report(id, true, "");
		}

		if (state.hasProperty(BlockStateProperties.OPEN)) {
			level.setBlock(pos, state.cycle(BlockStateProperties.OPEN), Block.UPDATE_ALL);
			return report(id, true, "");
		}

		// Campfires and candles: lit or not. Never a furnace, whose LIT the game owns.
		if (state.hasProperty(BlockStateProperties.LIT) && (Senses.blockId(state).contains("campfire") || Senses.blockId(state).contains("candle") || Senses.blockId(state).contains("redstone_lamp"))) {
			level.setBlock(pos, state.cycle(BlockStateProperties.LIT), Block.UPDATE_ALL);
			return report(id, true, "");
		}

		return report(id, false, Senses.blockId(state) + " has nothing to toggle");
	}

	/** The block ids a {@code set_block} may replace, beyond air; empty for air alone. */
	private static List<String> replaceable(JsonObject command) {
		List<String> ids = new ArrayList<>();
		if (command.has("replace") && command.get("replace").isJsonArray()) {
			for (JsonElement id : command.getAsJsonArray("replace")) {
				ids.add(id.getAsString());
			}
		}

		return ids;
	}

	private static JsonObject openLid(ServerLevel level, BlockPos pos, float seconds, long id, Context context) {
		BlockState state = level.getBlockState(pos);
		if (!(level.getBlockEntity(pos) instanceof net.minecraft.world.level.block.entity.ChestBlockEntity)) {
			return report(id, false, "no chest there");
		}

		// Block event 1 is the chest's opener count: the clients animate the lid from it.
		level.blockEvent(pos, state.getBlock(), 1, 1);
		level.playSound(null, pos, net.minecraft.sounds.SoundEvents.CHEST_OPEN, SoundSource.BLOCKS, 0.5f, level.getRandom().nextFloat() * 0.1f + 0.9f);
		context.later().at(seconds, () -> {
			if (level.getBlockEntity(pos) instanceof net.minecraft.world.level.block.entity.ChestBlockEntity) {
				level.blockEvent(pos, level.getBlockState(pos).getBlock(), 1, 0);
				level.playSound(null, pos, net.minecraft.sounds.SoundEvents.CHEST_CLOSE, SoundSource.BLOCKS, 0.5f, level.getRandom().nextFloat() * 0.1f + 0.9f);
			}
		});

		return report(id, true, "");
	}

	private static JsonObject rotateFrame(ServerLevel level, JsonObject command, long id) {
		if (!(level.getEntity(command.get("entity").getAsInt()) instanceof net.minecraft.world.entity.decoration.ItemFrame frame)) {
			return report(id, false, "no item frame with that id here");
		}
		if (frame.getItem().isEmpty()) {
			return report(id, false, "the frame is empty");
		}

		int rotation = command.has("rotation") && !command.get("rotation").isJsonNull()
				? command.get("rotation").getAsInt() % 8
				: (frame.getRotation() + 1) % 8;
		frame.setRotation(rotation);

		return report(id, true, "");
	}

	private static JsonObject swapPainting(ServerLevel level, int entityId, long id) {
		if (!(level.getEntity(entityId) instanceof net.minecraft.world.entity.decoration.painting.Painting painting)) {
			return report(id, false, "no painting with that id here");
		}

		Holder<net.minecraft.world.entity.decoration.painting.PaintingVariant> current = painting.getVariant();
		net.minecraft.core.Registry<net.minecraft.world.entity.decoration.painting.PaintingVariant> registry = level.registryAccess().lookupOrThrow(net.minecraft.core.registries.Registries.PAINTING_VARIANT);
		List<Holder.Reference<net.minecraft.world.entity.decoration.painting.PaintingVariant>> same = registry.listElements()
				.filter(variant -> variant.value().width() == current.value().width() && variant.value().height() == current.value().height() && !variant.equals(current))
				.toList();
		if (same.isEmpty()) {
			return report(id, false, "no other painting of that size");
		}

		// A new painting in the old one's place: the variant is final on the entity.
		Holder<net.minecraft.world.entity.decoration.painting.PaintingVariant> chosen = same.get(level.getRandom().nextInt(same.size()));
		net.minecraft.world.entity.decoration.painting.Painting replacement = new net.minecraft.world.entity.decoration.painting.Painting(level, painting.getPos(), painting.getDirection(), chosen);
		painting.discard();
		level.addFreshEntity(replacement);

		return report(id, true, "");
	}

	private static JsonObject poseStand(ServerLevel level, JsonObject command, long id, Context context) {
		if (!(level.getEntity(command.get("entity").getAsInt()) instanceof net.minecraft.world.entity.decoration.ArmorStand stand)) {
			return report(id, false, "no armour stand with that id here");
		}

		if (command.has("follow") && !command.get("follow").isJsonNull()) {
			ServerPlayer player = context.senses().player(level, command.get("follow").getAsInt());
			if (player == null) {
				return report(id, false, "the player is not here");
			}
			context.poses().add(stand, player, command.get("seconds").getAsFloat());

			return report(id, true, "");
		}

		if (command.has("at") && !command.get("at").isJsonNull()) {
			JsonArray at = command.getAsJsonArray("at");
			stand.setHeadPose(Poses.headToward(stand, new Vec3(at.get(0).getAsDouble(), at.get(1).getAsDouble(), at.get(2).getAsDouble())));

			return report(id, true, "");
		}

		return report(id, false, "neither a place to face nor a player to follow");
	}

	private static JsonObject nameEntity(ServerLevel level, int entityId, String name, long id) {
		Entity entity = level.getEntity(entityId);
		if (entity == null) {
			return report(id, false, "no entity with that id here");
		}

		entity.setCustomName(Component.literal(name));
		entity.setCustomNameVisible(true);

		return report(id, true, "");
	}

	private static JsonObject dyeSheep(ServerLevel level, int entityId, String colour, long id) {
		if (!(level.getEntity(entityId) instanceof net.minecraft.world.entity.animal.sheep.Sheep sheep)) {
			return report(id, false, "no sheep with that id here");
		}

		net.minecraft.world.item.DyeColor dye = net.minecraft.world.item.DyeColor.byName(colour, null);
		if (dye == null) {
			return report(id, false, "no colour called " + colour);
		}

		sheep.setColor(dye);

		return report(id, true, "");
	}

	private static JsonObject moveEntity(ServerLevel level, JsonObject command, long id, Context context) {
		int entityId = command.get("entity").getAsInt();
		JsonArray to = command.getAsJsonArray("to");
		double x = to.get(0).getAsDouble();
		double y = to.get(1).getAsDouble();
		double z = to.get(2).getAsDouble();

		// A player index wrapped from the top names the player; a player moves two blocks at most
		// (waking a block from the bed), never farther.
		if (entityId > Integer.MAX_VALUE - 64) {
			ServerPlayer player = context.senses().player(level, Integer.MAX_VALUE - entityId);
			if (player == null) {
				return report(id, false, "the player is not here");
			}
			if (player.position().distanceTo(new Vec3(x, y, z)) > 2.5) {
				return report(id, false, "a player is moved two blocks at most");
			}
			player.teleportTo(x, y, z);
			return report(id, true, "");
		}

		Entity entity = level.getEntity(entityId);
		if (entity == null || entity instanceof ServerPlayer) {
			return report(id, false, "no movable entity with that id here");
		}
		entity.teleportTo(x, y, z);

		return report(id, true, "");
	}

	private static JsonObject figure(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer skinOf = context.senses().player(level, command.get("skin_of").getAsInt());
		if (skinOf == null) {
			return report(id, false, "the player is not here");
		}

		JsonArray pos = command.getAsJsonArray("pos");
		List<Vec3> path = new ArrayList<>();
		if (command.has("path")) {
			for (JsonElement point : command.getAsJsonArray("path")) {
				JsonArray p = point.getAsJsonArray();
				path.add(new Vec3(p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble()));
			}
		}
		ItemStack held = ItemStack.EMPTY;
		if (command.has("held")) {
			JsonObject item = command.getAsJsonObject("held");
			String itemId = item.has("id") ? item.get("id").getAsString() : "";
			if (!itemId.isEmpty()) {
				Optional<Holder.Reference<Item>> kind = BuiltInRegistries.ITEM.get(Identifier.parse(itemId));
				if (kind.isPresent()) {
					held = new ItemStack(kind.get().value(), 1);
				}
			}
		}

		// The figure is shown to the player whose skin it wears: it is for them alone.
		context.figures().show(level, skinOf, command.get("figure").getAsInt(), skinOf,
				new Vec3(pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble()),
				path, held, command.has("name_shown") && command.get("name_shown").getAsBoolean(),
				command.has("seconds") ? command.get("seconds").getAsFloat() : 0,
				command.has("vanish_within") ? command.get("vanish_within").getAsFloat() : 0);

		return report(id, true, "");
	}

	/** What a {@code break_block} may break: things that drop as themselves and that a player rebuilds in a moment. */
	private static boolean breakable(BlockState state) {
		String id = Senses.blockId(state);
		return state.is(net.minecraft.tags.BlockTags.BEDS) || id.endsWith("torch") || id.endsWith("_carpet") || id.endsWith("glass_pane")
				|| state.is(net.minecraft.tags.BlockTags.FLOWERS) || state.is(net.minecraft.tags.BlockTags.ALL_SIGNS) || id.endsWith("lantern");
	}

	private static JsonObject breakBlock(ServerLevel level, BlockPos pos, long id) {
		BlockState state = level.getBlockState(pos);
		if (state.isAir()) {
			return report(id, false, "nothing there");
		}
		if (!breakable(state)) {
			return report(id, false, Senses.blockId(state) + " is not something it breaks");
		}

		// As a player's hand would: the block's own drop, both halves of a bed.
		level.destroyBlock(pos, true);

		return report(id, true, "");
	}

	private static JsonObject rotateBlock(ServerLevel level, BlockPos pos, long id) {
		BlockState state = level.getBlockState(pos);
		if (state.isAir()) {
			return report(id, false, "nothing there");
		}

		BlockState turned = state.rotate(net.minecraft.world.level.block.Rotation.CLOCKWISE_90);
		if (turned.equals(state)) {
			return report(id, false, Senses.blockId(state) + " does not turn");
		}

		level.setBlock(pos, turned, Block.UPDATE_ALL);

		return report(id, true, "");
	}

	private static JsonObject setFrameItem(ServerLevel level, JsonObject command, long id) {
		if (!(level.getEntity(command.get("entity").getAsInt()) instanceof net.minecraft.world.entity.decoration.ItemFrame frame)) {
			return report(id, false, "no item frame with that id here");
		}

		JsonObject item = command.getAsJsonObject("item");
		String itemId = item.has("id") ? item.get("id").getAsString() : "";
		if (itemId.isEmpty()) {
			frame.setItem(ItemStack.EMPTY);
			return report(id, true, "");
		}

		Optional<Holder.Reference<Item>> kind = BuiltInRegistries.ITEM.get(Identifier.parse(itemId));
		if (kind.isEmpty()) {
			return report(id, false, "no such item");
		}
		frame.setItem(new ItemStack(kind.get().value(), Math.max(1, item.get("count").getAsInt())));

		return report(id, true, "");
	}

	private static JsonObject equipEntity(ServerLevel level, JsonObject command, long id) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		if (!(entity instanceof net.minecraft.world.entity.LivingEntity living) || entity instanceof ServerPlayer) {
			return report(id, false, "no entity that wears things with that id here");
		}

		JsonObject item = command.getAsJsonObject("item");
		Optional<Holder.Reference<Item>> kind = BuiltInRegistries.ITEM.get(Identifier.parse(item.get("id").getAsString()));
		if (kind.isEmpty()) {
			return report(id, false, "no such item");
		}
		ItemStack stack = new ItemStack(kind.get().value(), Math.max(1, item.get("count").getAsInt()));
		net.minecraft.world.entity.EquipmentSlot slot = living.getEquipmentSlotForItem(stack);
		if (!living.getItemBySlot(slot).isEmpty()) {
			return report(id, false, "that slot is worn already");
		}
		living.setItemSlot(slot, stack);

		return report(id, true, "");
	}

	private static JsonObject damageItem(ServerLevel level, JsonObject command, long id) {
		Container container = container(level, blockPos(command.getAsJsonArray("pos")));
		if (container == null) {
			return report(id, false, "no container there");
		}

		int slot = command.get("slot").getAsInt();
		if (slot < 0 || slot >= container.getContainerSize()) {
			return report(id, false, "no such slot");
		}
		ItemStack stack = container.getItem(slot);
		if (stack.isEmpty() || !stack.isDamageableItem()) {
			return report(id, false, "nothing with wear in that slot");
		}

		// Worn, never broken: the damage stops one short of the item's end.
		int amount = command.get("amount").getAsInt();
		stack.setDamageValue(Math.min(stack.getMaxDamage() - 1, stack.getDamageValue() + Math.max(0, amount)));
		container.setChanged();

		return report(id, true, "");
	}

	private static JsonObject pushEntity(ServerLevel level, JsonObject command, long id) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		if (entity == null || entity instanceof ServerPlayer) {
			return report(id, false, "no pushable entity with that id here");
		}

		JsonArray v = command.getAsJsonArray("velocity");
		entity.setDeltaMovement(v.get(0).getAsDouble(), v.get(1).getAsDouble(), v.get(2).getAsDouble());
		entity.needsSync = true;

		return report(id, true, "");
	}

	private static JsonObject playerList(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}

		java.util.UUID ghost = java.util.UUID.randomUUID();
		com.mojang.authlib.GameProfile profile = new com.mojang.authlib.GameProfile(ghost, player.getGameProfile().name());
		var entry = new net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket.Entry(
				ghost, profile, true, 0, net.minecraft.world.level.GameType.SURVIVAL, null, true, 0, null);
		var packet = new net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket(
				java.util.EnumSet.of(net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket.Action.ADD_PLAYER,
						net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket.Action.UPDATE_LISTED,
						net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket.Action.UPDATE_LATENCY),
				List.of());
		// The packet's entries are built from players; a ghost has none, so it is set by hand.
		packet = withEntries(packet, List.of(entry));
		player.connection.send(packet);
		context.later().at(command.get("seconds").getAsFloat(), () -> {
			if (!player.isRemoved()) {
				player.connection.send(new net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket(List.of(ghost)));
			}
		});

		return report(id, true, "");
	}

	/** A player-info packet with these entries, written through its own codec since the entries field is private. */
	static net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket withEntries(
			net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket packet,
			List<net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket.Entry> entries) {
		try {
			java.lang.reflect.Field field = null;
			for (java.lang.reflect.Field candidate : packet.getClass().getDeclaredFields()) {
				if (List.class.isAssignableFrom(candidate.getType())) {
					field = candidate;
				}
			}
			if (field == null) {
				throw new IllegalStateException("no entries field");
			}
			field.setAccessible(true);
			field.set(packet, entries);

			return packet;
		} catch (ReflectiveOperationException e) {
			throw new IllegalStateException("could not set the player list entries", e);
		}
	}

	private static JsonObject gameMessage(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}

		String message = command.get("message").getAsString();
		String text = command.has("text") ? command.get("text").getAsString() : "";
		Component name = player.getName();
		switch (message) {
			case "joined" -> player.sendSystemMessage(Component.translatable("multiplayer.player.joined", name).withStyle(net.minecraft.ChatFormatting.YELLOW));
			case "left" -> player.sendSystemMessage(Component.translatable("multiplayer.player.left", name).withStyle(net.minecraft.ChatFormatting.YELLOW));
			case "says" -> player.sendSystemMessage(Component.translatable("chat.type.text", name, Component.literal(text)));
			case "whisper" -> player.sendSystemMessage(Component.translatable("commands.message.display.incoming", name, Component.literal(text)).withStyle(net.minecraft.ChatFormatting.GRAY, net.minecraft.ChatFormatting.ITALIC));
			case "sleepers" -> player.connection.send(new net.minecraft.network.protocol.game.ClientboundSetActionBarTextPacket(Component.translatable("sleep.players_sleeping", 1, 2)));
			case "bed_occupied" -> player.connection.send(new net.minecraft.network.protocol.game.ClientboundSetActionBarTextPacket(Component.translatable("block.minecraft.bed.occupied")));
			default -> {
				return report(id, false, "no message kind called " + message);
			}
		}

		return report(id, true, "");
	}

	private static JsonObject moveBlock(ServerLevel level, JsonObject command, long id) {
		BlockPos from = blockPos(command.getAsJsonArray("from"));
		BlockPos to = blockPos(command.getAsJsonArray("to"));
		int around = command.has("around") ? Math.clamp(command.get("around").getAsInt(), 0, 8) : 0;
		BlockState state = level.getBlockState(from);
		if (state.isAir()) {
			return report(id, false, "nothing at the source");
		}

		// A single-block container goes with what it holds; a chest may be half of two.
		BlockEntity entity = level.getBlockEntity(from);
		Container holds = entity instanceof Container c && !(state.getBlock() instanceof net.minecraft.world.level.block.ChestBlock) ? c : null;
		if (entity != null && holds == null) {
			return report(id, false, "a block with contents is not moved this way");
		}

		BlockPos at = null;
		for (int[] offset : Moves.offsets(around)) {
			BlockPos candidate = to.offset(offset[0], offset[1], offset[2]);
			// A torch is moved as a player would move it: never to where nothing holds it; a box
			// moved around sits on something solid.
			if (candidate.equals(from) || !level.getBlockState(candidate).isAir() || !state.canSurvive(level, candidate)) {
				continue;
			}
			if (around > 0 && !level.getBlockState(candidate.below()).isFaceSturdy(level, candidate.below(), Direction.UP)) {
				continue;
			}
			at = candidate;
			break;
		}
		if (at == null) {
			return report(id, false, around > 0 ? "no room around the destination" : "the destination is not empty, or nothing holds it there");
		}

		List<ItemStack> contents = new ArrayList<>();
		if (holds != null) {
			for (int slot = 0; slot < holds.getContainerSize(); slot++) {
				contents.add(holds.getItem(slot).copy());
				holds.setItem(slot, ItemStack.EMPTY);
			}
		}
		level.setBlock(from, Blocks.AIR.defaultBlockState(), Block.UPDATE_ALL);
		level.setBlock(at, state, Block.UPDATE_ALL);
		if (holds != null && level.getBlockEntity(at) instanceof Container moved) {
			for (int slot = 0; slot < Math.min(contents.size(), moved.getContainerSize()); slot++) {
				moved.setItem(slot, contents.get(slot));
			}
			moved.setChanged();
		}

		return report(id, true, "");
	}

	private static JsonObject setBlock(ServerLevel level, JsonObject command, long id) {
		BlockPos pos = blockPos(command.getAsJsonArray("pos"));
		List<String> replace = replaceable(command);
		int settle = command.has("settle") ? command.get("settle").getAsInt() : 0;
		String wallId = command.has("wall") ? command.get("wall").getAsString() : "";
		boolean rest = command.has("rest") && command.get("rest").getAsBoolean();
		boolean only = command.has("only") && command.get("only").getAsBoolean();

		Optional<Holder.Reference<Block>> block = BuiltInRegistries.BLOCK.get(Identifier.parse(blockId(command)));
		Optional<Holder.Reference<Block>> wall = wallId.isEmpty() ? Optional.empty() : BuiltInRegistries.BLOCK.get(Identifier.parse(wallId));
		if (block.isEmpty() || (!wallId.isEmpty() && wall.isEmpty())) {
			return report(id, false, "no such block");
		}

		// The nearest spot first, down before up, standing before against a wall: a torch on the
		// path goes on the ground below the slope before it goes on the hill beside it.
		String why = "nothing holds it there";
		for (int dy : Settle.order(settle)) {
			BlockPos at = pos.above(dy);
			String taken = taken(level, at, replace, only);
			if (taken != null) {
				why = dy == 0 ? taken : why;
				continue;
			}
			if (rest && !level.getBlockState(at.below()).isFaceSturdy(level, at.below(), Direction.UP)) {
				why = dy == 0 ? "nothing under it" : why;
				continue;
			}
			BlockState standing = block.get().value().defaultBlockState();
			if (standing.canSurvive(level, at)) {
				level.setBlock(at, standing, Block.UPDATE_ALL);
				return report(id, true, "");
			}
		}
		if (wall.isPresent()) {
			for (int dy : Settle.order(settle)) {
				BlockPos at = pos.above(dy);
				if (taken(level, at, replace, only) != null) {
					continue;
				}
				BlockState onWall = againstWall(level, at, wall.get().value().defaultBlockState());
				if (onWall != null) {
					level.setBlock(at, onWall, Block.UPDATE_ALL);
					return report(id, true, "");
				}
			}
		}

		return report(id, false, why);
	}

	private static String blockId(JsonObject command) {
		return command.get("block").getAsString();
	}

	/**
	 * Why a block may not go at {@code pos}, or null when it may. Nothing a player built is ever
	 * destroyed: a block goes where there is nothing, or over one of the blocks the mod named,
	 * which it keeps to natural ground.
	 */
	private static String taken(ServerLevel level, BlockPos pos, List<String> replace, boolean only) {
		BlockState there = level.getBlockState(pos);
		if (only && !replace.contains(Senses.blockId(there))) {
			return there.isAir() ? "nothing to replace there" : "the place holds " + Senses.blockId(there);
		}
		if (!there.isAir() && !replace.contains(Senses.blockId(there))) {
			return "the place holds " + Senses.blockId(there);
		}
		if (level.getBlockEntity(pos) != null) {
			return "a block with contents is never replaced";
		}

		return null;
	}

	/** The wall block facing out from the first side that holds it, or null if none does. */
	private static BlockState againstWall(ServerLevel level, BlockPos pos, BlockState wall) {
		if (!wall.hasProperty(BlockStateProperties.HORIZONTAL_FACING)) {
			return wall.canSurvive(level, pos) ? wall : null;
		}
		for (Direction facing : Direction.Plane.HORIZONTAL) {
			BlockState state = wall.setValue(BlockStateProperties.HORIZONTAL_FACING, facing);
			if (state.canSurvive(level, pos)) {
				return state;
			}
		}

		return null;
	}

	private static JsonObject moveItem(ServerLevel level, JsonObject command, long id) {
		Container from = container(level, blockPos(command.getAsJsonArray("from")));
		Container to = container(level, blockPos(command.getAsJsonArray("to")));
		if (from == null || to == null) {
			return report(id, false, "no container at " + (from == null ? "the source" : "the destination"));
		}

		int fromSlot = command.get("from_slot").getAsInt();
		int toSlot = command.get("to_slot").getAsInt();
		int count = command.get("count").getAsInt();
		if (fromSlot < 0 || fromSlot >= from.getContainerSize() || toSlot < 0 || toSlot >= to.getContainerSize()) {
			return report(id, false, "no such slot");
		}

		// What is there now, up to the count: a chest changes while no one watches it.
		ItemStack source = from.getItem(fromSlot);
		if (source.isEmpty()) {
			// The slot asked is empty: the first that is not (a hopper no one has looked in).
			for (int candidate : Slots.order(fromSlot, from.getContainerSize())) {
				if (!from.getItem(candidate).isEmpty()) {
					fromSlot = candidate;
					source = from.getItem(candidate);
					break;
				}
			}
		}
		if (source.isEmpty()) {
			return report(id, false, "nothing in the source");
		}
		count = Math.min(count, source.getCount());

		// The slot asked for, else a matching stack with room, else the first empty one.
		int slot = -1;
		for (int candidate : Slots.order(toSlot, to.getContainerSize())) {
			if (to == from && candidate == fromSlot) {
				continue;
			}
			ItemStack there = to.getItem(candidate);
			if (there.isEmpty() || (ItemStack.isSameItemSameComponents(there, source) && there.getCount() + count <= there.getMaxStackSize())) {
				slot = candidate;
				break;
			}
		}
		if (slot < 0) {
			return report(id, false, "the destination is full");
		}

		ItemStack moved = source.split(count);
		ItemStack target = to.getItem(slot);
		if (target.isEmpty()) {
			to.setItem(slot, moved);
		} else {
			target.grow(moved.getCount());
		}
		from.setChanged();
		to.setChanged();

		return report(id, true, "");
	}

	private static JsonObject takeItem(ServerLevel level, JsonObject command, long id, WorldData data) {
		Container from = container(level, blockPos(command.getAsJsonArray("from")));
		if (from == null) {
			return report(id, false, "no container at the source");
		}

		int slot = command.get("slot").getAsInt();
		int count = command.get("count").getAsInt();
		if (slot < 0 || slot >= from.getContainerSize()) {
			return report(id, false, "no such slot");
		}

		ItemStack source = from.getItem(slot);
		if (source.isEmpty() || source.getCount() < count) {
			return report(id, false, "the slot has too few");
		}

		// What it takes it holds: the very stack, kept with the world, when the mod names a key.
		ItemStack taken = source.split(count);
		if (command.has("keep") && !command.get("keep").getAsString().isEmpty()) {
			data.keep(command.get("keep").getAsString(), taken);
		}
		from.setChanged();

		return report(id, true, "");
	}

	private static JsonObject signBook(ServerLevel level, JsonObject command, long id) {
		Container in = container(level, blockPos(command.getAsJsonArray("pos")));
		int slot = command.get("slot").getAsInt();
		if (in == null || slot < 0 || slot >= in.getContainerSize() || !in.getItem(slot).is(net.minecraft.world.item.Items.WRITABLE_BOOK)) {
			return report(id, false, "no book and quill there");
		}
		List<net.minecraft.server.network.Filterable<Component>> pages = new ArrayList<>();
		for (JsonElement page : command.getAsJsonArray("pages")) {
			pages.add(net.minecraft.server.network.Filterable.passThrough(Component.literal(page.getAsString())));
		}
		String title = command.get("title").getAsString();
		ItemStack written = in.getItem(slot).transmuteCopy(net.minecraft.world.item.Items.WRITTEN_BOOK, 1);
		written.set(net.minecraft.core.component.DataComponents.WRITTEN_BOOK_CONTENT,
				new net.minecraft.world.item.component.WrittenBookContent(net.minecraft.server.network.Filterable.passThrough(title.isEmpty() ? " " : title), "", 0, pages, true));
		in.setItem(slot, written);
		in.setChanged();

		return report(id, true, "");
	}

	private static JsonObject markMap(ServerLevel level, JsonObject command, long id) {
		Container in = container(level, blockPos(command.getAsJsonArray("pos")));
		int slot = command.get("slot").getAsInt();
		if (in == null || slot < 0 || slot >= in.getContainerSize() || !in.getItem(slot).is(net.minecraft.world.item.Items.FILLED_MAP)) {
			return report(id, false, "no map there");
		}
		net.minecraft.world.level.saveddata.maps.MapItemSavedData.addTargetDecoration(in.getItem(slot), blockPos(command.getAsJsonArray("at")), "craftkit", net.minecraft.world.level.saveddata.maps.MapDecorationTypes.TARGET_X);
		in.setChanged();

		return report(id, true, "");
	}

	private static JsonObject renameItem(ServerLevel level, JsonObject command, long id) {
		Container in = container(level, blockPos(command.getAsJsonArray("pos")));
		if (in == null) {
			return report(id, false, "no container there");
		}
		int slot = command.get("slot").getAsInt();
		if (slot < 0 || slot >= in.getContainerSize() || in.getItem(slot).isEmpty()) {
			return report(id, false, "nothing in that slot");
		}
		in.getItem(slot).set(net.minecraft.core.component.DataComponents.CUSTOM_NAME, Component.literal(command.get("name").getAsString()));
		in.setChanged();

		return report(id, true, "");
	}

	/** The stack a give-back command means: the kept one when it names a key, else one made from its item. */
	private static ItemStack stackOf(JsonObject command, WorldData data) {
		if (command.has("kept") && !command.get("kept").getAsString().isEmpty()) {
			return data.giveBack(command.get("kept").getAsString());
		}
		JsonObject item = command.getAsJsonObject("item");
		Optional<Holder.Reference<Item>> kind = BuiltInRegistries.ITEM.get(Identifier.parse(item.get("id").getAsString()));

		return kind.map(k -> new ItemStack(k.value(), item.get("count").getAsInt())).orElse(null);
	}

	private static JsonObject putItem(ServerLevel level, JsonObject command, long id, WorldData data) {
		Container into = container(level, blockPos(command.getAsJsonArray("into")));
		if (into == null) {
			return report(id, false, "no container at the destination");
		}

		ItemStack stack = stackOf(command, data);
		if (stack == null || stack.isEmpty()) {
			return report(id, false, "no such item, or nothing kept under that name");
		}
		// As it goes back: a little more wear (never broken by it), a name.
		int wear = command.has("wear") ? command.get("wear").getAsInt() : 0;
		if (wear > 0 && stack.isDamageableItem()) {
			stack.setDamageValue(Math.min(stack.getMaxDamage() - 1, stack.getDamageValue() + wear));
		}
		if (command.has("name") && !command.get("name").getAsString().isEmpty()) {
			stack.set(net.minecraft.core.component.DataComponents.CUSTOM_NAME, Component.literal(command.get("name").getAsString()));
		}
		int slot = -1;
		if (command.has("slot") && !command.get("slot").isJsonNull()) {
			slot = command.get("slot").getAsInt();
			if (slot < 0 || slot >= into.getContainerSize()) {
				return report(id, false, "no such slot");
			}
			if (!into.getItem(slot).isEmpty()) {
				slot = -1;
			}
		}

		if (slot < 0) {
			for (int candidate = 0; candidate < into.getContainerSize(); candidate++) {
				if (into.getItem(candidate).isEmpty()) {
					slot = candidate;
					break;
				}
			}
		}

		if (slot < 0) {
			// Never lost: what does not fit goes back to where it was kept.
			if (command.has("kept") && !command.get("kept").getAsString().isEmpty()) {
				data.keep(command.get("kept").getAsString(), stack);
			}
			return report(id, false, "the container is full");
		}

		into.setItem(slot, stack);
		into.setChanged();

		return report(id, true, "");
	}

	private static JsonObject dropItem(ServerLevel level, JsonObject command, long id, WorldData data) {
		ItemStack stack = stackOf(command, data);
		if (stack == null || stack.isEmpty()) {
			return report(id, false, "no such item, or nothing kept under that name");
		}

		JsonArray pos = command.getAsJsonArray("pos");
		ItemEntity entity = new ItemEntity(level, pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble(), stack, 0, 0, 0);
		entity.setUnlimitedLifetime();
		level.addFreshEntity(entity);

		return report(id, true, "");
	}

	private static JsonObject setSign(ServerLevel level, JsonObject command, long id) {
		BlockPos pos = blockPos(command.getAsJsonArray("pos"));
		int settle = command.has("settle") ? command.get("settle").getAsInt() : 0;
		SignBlockEntity sign = null;
		for (int dy : Settle.order(settle)) {
			if (level.getBlockEntity(pos.above(dy)) instanceof SignBlockEntity found) {
				sign = found;
				break;
			}
		}
		if (sign == null) {
			return report(id, false, "no sign there");
		}

		List<Component> lines = new ArrayList<>();
		for (JsonElement line : command.getAsJsonArray("lines")) {
			lines.add(Component.literal(line.getAsString()));
		}
		while (lines.size() < SignText.LINES) {
			lines.add(Component.empty());
		}

		SignText text = new SignText(lines.subList(0, SignText.LINES), lines.subList(0, SignText.LINES), net.minecraft.world.item.DyeColor.BLACK, false);
		sign.setText(text, SignTextSlot.FRONT);

		return report(id, true, "");
	}

	private static JsonObject cancelSleep(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}

		if (player.isSleeping()) {
			player.stopSleepInBed(true, true);
			player.connection.send(new net.minecraft.network.protocol.game.ClientboundSetActionBarTextPacket(Component.translatable("block.minecraft.bed.not_safe")));
		} else {
			context.senses().refuseSleep(player, command.get("seconds").getAsFloat());
		}

		return report(id, true, "");
	}

	private static JsonObject lookAt(ServerLevel level, JsonObject command, long id, Context context) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		Mob mob = entity == null ? null : Senses.mob(entity);
		if (mob == null) {
			return report(id, false, "no mob with that id here");
		}

		JsonArray at = command.getAsJsonArray("at");
		context.looks().add(mob, new Vec3(at.get(0).getAsDouble(), at.get(1).getAsDouble(), at.get(2).getAsDouble()), command.get("seconds").getAsFloat());

		return report(id, true, "");
	}

	/** The farthest a mod may move a player in one go. */
	static final double MOST_MOVE = 32;

	private static JsonObject movePlayer(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		JsonArray to = command.getAsJsonArray("to");
		Vec3 target = new Vec3(to.get(0).getAsDouble(), to.get(1).getAsDouble(), to.get(2).getAsDouble());
		if (target.distanceTo(player.position()) > MOST_MOVE) {
			return report(id, false, "farther than " + (int) MOST_MOVE + " blocks");
		}
		for (int up = 0; up <= 2; up++) {
			Vec3 at = target.add(0, up, 0);
			if (level.noCollision(player, player.getDimensions(player.getPose()).makeBoundingBox(at))) {
				player.teleportTo(at.x, at.y, at.z);
				return report(id, true, "");
			}
		}

		return report(id, false, "no room to stand there");
	}

	private static JsonObject hold(ServerLevel level, JsonObject command, long id, Context context) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		Mob mob = entity == null ? null : Senses.mob(entity);
		if (mob == null) {
			return report(id, false, "no mob with that id here");
		}
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}

		context.holds().add(mob, player, command.get("seconds").getAsFloat());

		return report(id, true, "");
	}

	private static Container container(ServerLevel level, BlockPos pos) {
		BlockEntity entity = level.getBlockEntity(pos);

		return entity instanceof Container container ? container : null;
	}

	static BlockPos blockPos(JsonArray array) {
		return new BlockPos(array.get(0).getAsInt(), array.get(1).getAsInt(), array.get(2).getAsInt());
	}

	static JsonObject report(long id, boolean done, String reason) {
		JsonObject report = new JsonObject();
		report.addProperty("id", id);
		report.addProperty("done", done);
		report.addProperty("reason", reason);

		return report;
	}
}
