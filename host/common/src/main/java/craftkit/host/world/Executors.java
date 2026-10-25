package craftkit.host.world;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import craftkit.host.LoadedMod;
import craftkit.host.Log;
import craftkit.host.net.HintPayload;
import craftkit.host.senses.Senses;
import net.minecraft.core.BlockPos;
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
	public record Context(Senses senses, BiConsumer<ServerPlayer, HintPayload> sendHint, Looks looks) {
	}

	private Executors() {
	}

	public static JsonObject run(LoadedMod mod, ServerLevel level, JsonObject command, Context context) {
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
				case "move_block" -> moveBlock(level, blockPos(command.getAsJsonArray("from")), blockPos(command.getAsJsonArray("to")), id);
				case "set_block" -> setBlock(level, blockPos(command.getAsJsonArray("pos")), command.get("block").getAsString(), id);
				case "move_item" -> moveItem(level, command, id);
				case "take_item" -> takeItem(level, command, id);
				case "put_item" -> putItem(level, command, id);
				case "drop_item" -> dropItem(level, command, id);
				case "set_sign" -> setSign(level, command, id);
				case "cancel_sleep" -> cancelSleep(level, command, id, context);
				case "look_at" -> lookAt(level, command, id, context);
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

		return report(id, false, Senses.blockId(state) + " has nothing to toggle");
	}

	private static JsonObject moveBlock(ServerLevel level, BlockPos from, BlockPos to, long id) {
		BlockState state = level.getBlockState(from);
		if (state.isAir()) {
			return report(id, false, "nothing at the source");
		}

		if (level.getBlockEntity(from) != null) {
			return report(id, false, "a block with contents is not moved this way");
		}

		if (!level.getBlockState(to).isAir()) {
			return report(id, false, "the destination is not empty");
		}

		level.setBlock(from, Blocks.AIR.defaultBlockState(), Block.UPDATE_ALL);
		level.setBlock(to, state, Block.UPDATE_ALL);

		return report(id, true, "");
	}

	private static JsonObject setBlock(ServerLevel level, BlockPos pos, String blockId, long id) {
		Optional<Holder.Reference<Block>> block = BuiltInRegistries.BLOCK.get(Identifier.parse(blockId));
		if (block.isEmpty()) {
			return report(id, false, "no such block");
		}

		// Nothing is ever destroyed: a block goes only where there is nothing.
		if (!level.getBlockState(pos).isAir() && !block.get().value().defaultBlockState().isAir()) {
			return report(id, false, "the place is not empty");
		}

		level.setBlock(pos, block.get().value().defaultBlockState(), Block.UPDATE_ALL);

		return report(id, true, "");
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

		ItemStack source = from.getItem(fromSlot);
		if (source.isEmpty() || source.getCount() < count) {
			return report(id, false, "the source slot has too few");
		}

		ItemStack target = to.getItem(toSlot);
		if (!target.isEmpty() && !(ItemStack.isSameItemSameComponents(target, source) && target.getCount() + count <= target.getMaxStackSize())) {
			return report(id, false, "the destination slot is taken");
		}

		ItemStack moved = source.split(count);
		if (target.isEmpty()) {
			to.setItem(toSlot, moved);
		} else {
			target.grow(count);
		}
		from.setChanged();
		to.setChanged();

		return report(id, true, "");
	}

	private static JsonObject takeItem(ServerLevel level, JsonObject command, long id) {
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

		source.shrink(count);
		from.setChanged();

		return report(id, true, "");
	}

	private static JsonObject putItem(ServerLevel level, JsonObject command, long id) {
		Container into = container(level, blockPos(command.getAsJsonArray("into")));
		if (into == null) {
			return report(id, false, "no container at the destination");
		}

		JsonObject item = command.getAsJsonObject("item");
		Optional<Holder.Reference<Item>> kind = BuiltInRegistries.ITEM.get(Identifier.parse(item.get("id").getAsString()));
		if (kind.isEmpty()) {
			return report(id, false, "no such item");
		}

		ItemStack stack = new ItemStack(kind.get().value(), item.get("count").getAsInt());
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
			return report(id, false, "the container is full");
		}

		into.setItem(slot, stack);
		into.setChanged();

		return report(id, true, "");
	}

	private static JsonObject dropItem(ServerLevel level, JsonObject command, long id) {
		JsonObject item = command.getAsJsonObject("item");
		Optional<Holder.Reference<Item>> kind = BuiltInRegistries.ITEM.get(Identifier.parse(item.get("id").getAsString()));
		if (kind.isEmpty()) {
			return report(id, false, "no such item");
		}

		JsonArray pos = command.getAsJsonArray("pos");
		ItemEntity entity = new ItemEntity(level, pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble(), new ItemStack(kind.get().value(), item.get("count").getAsInt()), 0, 0, 0);
		entity.setUnlimitedLifetime();
		level.addFreshEntity(entity);

		return report(id, true, "");
	}

	private static JsonObject setSign(ServerLevel level, JsonObject command, long id) {
		BlockEntity entity = level.getBlockEntity(blockPos(command.getAsJsonArray("pos")));
		if (!(entity instanceof SignBlockEntity sign)) {
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
