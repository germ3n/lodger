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
	public record Context(Senses senses, BiConsumer<ServerPlayer, HintPayload> sendHint, Looks looks, Later later, Poses poses, Figures figures, Holds holds, LootOffers loot, Hidden hidden, Warps warps, Temporaries temporaries, Charges charges, Gatherings gatherings, Footsteps footsteps, Approaches approaches, Shoves shoves, Bobbers bobbers, FrameShows frameShows, Deaths deaths, Fights fights) {
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
				case "equip_entity" -> equipEntity(level, command, id, data);
				case "damage_item" -> damageItem(level, command, id);
				case "figure" -> figure(level, command, id, context);
				case "swap_block" -> swapBlock(level, command, id);
				case "jukebox" -> jukebox(level, command, id);
				case "prints" -> prints(level, command, id);
				case "turn_page" -> turnPage(level, command, id);
				case "brew" -> brew(level, command, id);
				case "strip" -> strip(level, command, id);
				case "shift_bed" -> done(id, Beds.shift(level, blockPos(command.getAsJsonArray("pos"))));
				case "bed_beside" -> bedBeside(level, command, id);
				case "hide_block" -> hideBlock(level, command, id, context);
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
				case "warp_mob" -> warpMob(level, command, id, context);
				case "place_entity" -> placeEntity(level, command, id, context, data);
				case "release_bees" -> releaseBees(level, command, id, context);
				case "move_leash" -> moveLeash(level, command, id);
				case "charge" -> charge(level, command, id, context);
				case "gather_villagers" -> gatherVillagers(level, command, id, context);
				case "light_portal" -> lightPortal(level, command, id);
				case "place_blueprint" -> placeBlueprint(level, command, id);
				case "figure_swing" -> figureSwing(level, command, id, context);
				case "dig_in" -> digIn(level, command, id, context);
				case "offer_trade" -> offerTrade(level, command, id, context, data);
				case "move_player" -> movePlayer(level, command, id, context);
				case "set_state" -> setState(level, command, id);
				case "grow" -> grow(level, command, id);
				case "arm_loot" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					context.loot().arm(player, mod.id(), command);
					yield report(id, true, "");
				}
				case "block_sound" -> blockSound(level, command, id, context);
				case "approach" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					int walking = 0;
					for (var each : command.getAsJsonArray("entities")) {
						Entity entity = level.getEntity(each.getAsInt());
						Mob mob = entity == null ? null : Senses.mob(entity);
						if (mob != null && !mob.entityTags().contains(Holds.TAG) && !mob.entityTags().contains(Warps.TAG)) {
							context.approaches().add(mob, player, command.get("stop").getAsFloat(), command.get("seconds").getAsFloat(),
									command.has("pace") && !command.get("pace").isJsonNull() ? command.get("pace").getAsFloat() : 1f,
									command.has("then") ? command.get("then").getAsString() : "",
									command.has("damage") ? command.get("damage").getAsFloat() : 0f);
							walking++;
						}
					}
					yield report(id, walking > 0, walking > 0 ? "walking " + walking : "none of them here");
				}
				case "shoot" -> shoot(level, command, id, context);
				case "confront" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					JsonArray roof = command.getAsJsonArray("roof");
					List<String> blocks = new ArrayList<>();
					command.getAsJsonArray("blocks").forEach(b -> blocks.add(b.getAsString()));
					if (blocks.isEmpty()) {
						blocks.add("minecraft:cobblestone");
					}
					String why = context.fights().start(level, player, blockPos(command.getAsJsonArray("at")),
							new Vec3(roof.get(0).getAsDouble(), roof.get(1).getAsDouble(), roof.get(2).getAsDouble()), blocks, data);
					yield report(id, why == null, why == null ? "the fight is on" : why);
				}
				case "splash_potion" -> splashPotion(level, command, id, context);
				case "put_potion" -> putPotion(level, command, id);
				case "change_potion" -> changePotion(level, command, id);
				case "effect_cloud" -> effectCloud(level, command, id);
				case "kill_player" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					String why = context.deaths().kill(player, data);
					yield report(id, why == null, why == null ? "" : why);
				}
				case "spill_container" -> spillContainer(level, command, id);
				case "carry_block" -> {
					Entity entity = level.getEntity(command.get("entity").getAsInt());
					if (!(entity instanceof net.minecraft.world.entity.monster.Enderman enderman)) {
						yield report(id, false, "no enderman with that id here");
					}
					Optional<Holder.Reference<Block>> block = BuiltInRegistries.BLOCK.get(Identifier.parse(command.get("block").getAsString()));
					if (block.isEmpty()) {
						yield report(id, false, "no such block");
					}
					// Never in place of one it carries (which may be from their building).
					if (enderman.getCarriedBlock() != null) {
						yield report(id, false, "it carries a block already");
					}
					enderman.setCarriedBlock(block.get().value().defaultBlockState());
					yield report(id, true, "");
				}
				case "bobber" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					context.bobbers().arm(player, command.get("seconds").getAsFloat(), command.get("under").getAsFloat());
					yield report(id, true, "armed");
				}
				case "frames_show" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					Optional<Holder.Reference<net.minecraft.world.item.Item>> item = BuiltInRegistries.ITEM.get(Identifier.parse(command.get("item").getAsString()));
					if (item.isEmpty()) {
						yield report(id, false, "no such item");
					}
					List<net.minecraft.world.entity.decoration.ItemFrame> frames = new ArrayList<>();
					for (var each : command.getAsJsonArray("frames")) {
						if (level.getEntity(each.getAsInt()) instanceof net.minecraft.world.entity.decoration.ItemFrame frame) {
							frames.add(frame);
						}
					}
					int shown = context.frameShows().show(player, frames, new net.minecraft.world.item.ItemStack(item.get()), command.get("seconds").getAsFloat());
					yield report(id, shown > 0, shown > 0 ? "shown in " + shown : "no frame of those here");
				}
				case "shove" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					context.shoves().arm(player, command.get("seconds").getAsFloat(), command.get("most_drop").getAsInt());
					yield report(id, true, "armed");
				}
				case "footsteps" -> {
					ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
					if (player == null) {
						yield report(id, false, "the player is not here");
					}
					context.footsteps().add(player, command.get("seconds").getAsFloat(), command.get("behind").getAsFloat(), command.get("delay").getAsInt());
					yield report(id, true, "");
				}
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
		int step = command.has("step") ? command.get("step").getAsInt() : 0;
		boolean dark = command.has("dark") && command.get("dark").getAsBoolean();
		int most = command.has("most") ? command.get("most").getAsInt() : 0;
		int[] lo = {min.getX(), min.getY(), min.getZ()};
		int[] hi = {max.getX(), max.getY(), max.getZ()};
		long volume = ScanBox.reads(lo, hi, step);
		if (volume > SCAN_LIMIT) {
			return report(id, false, "the scan reads " + volume + " blocks; at most " + SCAN_LIMIT);
		}

		List<String> wanted = new ArrayList<>();
		for (JsonElement kind : command.getAsJsonArray("kinds")) {
			wanted.add(kind.getAsString());
		}

		java.util.Map<String, Integer> counts = new java.util.TreeMap<>();
		JsonArray positions = new JsonArray();
		BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
		for (int x : ScanBox.axis(lo[0], hi[0], step)) {
			for (int y : ScanBox.axis(lo[1], hi[1], step)) {
				for (int z : ScanBox.axis(lo[2], hi[2], step)) {
					pos.set(x, y, z);
					String block = Senses.blockId(level.getBlockState(pos));
					counts.merge(block, 1, Integer::sum);
					if (!Kinds.wanted(block, wanted) || (most > 0 && positions.size() >= most)) {
						continue;
					}
					if (dark && (level.getBrightness(net.minecraft.world.level.LightLayer.SKY, pos) > 0
							|| level.getBrightness(net.minecraft.world.level.LightLayer.BLOCK, pos) > 0)) {
						continue;
					}
					JsonArray entry = new JsonArray();
					entry.add(block);
					entry.add(Senses.blockPos(pos.immutable()));
					positions.add(entry);
				}
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

		// A bell rings, swinging, as if struck (batch 9).
		if (block instanceof net.minecraft.world.level.block.BellBlock bell) {
			boolean rang = bell.attemptToRing(level, pos, state.getValue(net.minecraft.world.level.block.BellBlock.FACING));
			return report(id, rang, rang ? "" : "the bell would not ring");
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
		return replaceable(command, "replace");
	}

	/** The block ids a command lists under a key. */
	private static List<String> replaceable(JsonObject command, String key) {
		List<String> ids = new ArrayList<>();
		if (command.has(key) && command.get(key).isJsonArray()) {
			for (JsonElement id : command.getAsJsonArray(key)) {
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
				command.has("vanish_within") ? command.get("vanish_within").getAsFloat() : 0,
				command.has("mirror") && command.get("mirror").getAsBoolean(),
				command.has("once_seen") && command.get("once_seen").getAsBoolean(),
				command.has("copy_crouch") && command.get("copy_crouch").getAsBoolean(),
				command.has("swinging") && command.get("swinging").getAsBoolean());

		return report(id, true, "");
	}

	/** Another block in a block's place, every property they share kept: a wall torch's facing, a lantern's hanging (ABI 24). */
	private static JsonObject swapBlock(ServerLevel level, JsonObject command, long id) {
		BlockPos pos = blockPos(command.getAsJsonArray("pos"));
		BlockState there = level.getBlockState(pos);
		if (!replaceable(command, "only").contains(Senses.blockId(there))) {
			return report(id, false, "the place holds " + Senses.blockId(there));
		}
		Optional<Holder.Reference<Block>> block = BuiltInRegistries.BLOCK.get(Identifier.parse(blockId(command)));
		if (block.isEmpty()) {
			return report(id, false, "no such block");
		}
		// A bed is two blocks: both halves go at once, or the other breaks.
		if (there.getBlock() instanceof net.minecraft.world.level.block.BedBlock) {
			return done(id, Beds.recolour(level, pos, block.get().value()));
		}
		BlockState swapped = block.get().value().defaultBlockState();
		for (net.minecraft.world.level.block.state.properties.Property<?> property : there.getProperties()) {
			swapped = copy(there, swapped, property);
		}
		level.setBlock(pos, swapped, Block.UPDATE_ALL);

		return report(id, true, "");
	}

	private static <T extends Comparable<T>> BlockState copy(BlockState from, BlockState to, net.minecraft.world.level.block.state.properties.Property<T> property) {
		return to.hasProperty(property) ? to.setValue(property, from.getValue(property)) : to;
	}

	private static JsonObject bedBeside(ServerLevel level, JsonObject command, long id) {
		Optional<Holder.Reference<Block>> block = BuiltInRegistries.BLOCK.get(Identifier.parse(blockId(command)));
		if (block.isEmpty()) {
			return report(id, false, "no such block");
		}

		return done(id, Beds.beside(level, blockPos(command.getAsJsonArray("pos")), block.get().value()));
	}

	/** Footprints in the snow from one block to another: a layer off at each step, left and right of the line (ABI 30). */
	private static JsonObject prints(ServerLevel level, JsonObject command, long id) {
		BlockPos from = blockPos(command.getAsJsonArray("from"));
		BlockPos to = blockPos(command.getAsJsonArray("to"));
		double dx = to.getX() - from.getX();
		double dz = to.getZ() - from.getZ();
		double length = Math.sqrt(dx * dx + dz * dz);
		if (length < 2) {
			return report(id, false, "too short a way");
		}
		double ux = dx / length;
		double uz = dz / length;
		int made = 0;
		for (int i = 1; i <= Math.min(40, (int) length); i++) {
			// Left foot, right foot: half a block either side of the line.
			double side = (i % 2 == 0 ? 0.45 : -0.45);
			int x = (int) Math.floor(from.getX() + 0.5 + ux * i - uz * side);
			int z = (int) Math.floor(from.getZ() + 0.5 + uz * i + ux * side);
			for (int y = from.getY() + 3; y >= from.getY() - 4; y--) {
				BlockPos at = new BlockPos(x, y, z);
				BlockState state = level.getBlockState(at);
				if (state.is(net.minecraft.world.level.block.Blocks.SNOW)) {
					int layers = state.getValue(net.minecraft.world.level.block.SnowLayerBlock.LAYERS);
					level.setBlock(at, layers > 1
							? state.setValue(net.minecraft.world.level.block.SnowLayerBlock.LAYERS, layers - 1)
							: net.minecraft.world.level.block.Blocks.AIR.defaultBlockState(), Block.UPDATE_ALL);
					made++;
					break;
				}
				if (!state.isAir()) {
					break;
				}
			}
		}

		return made >= 3 ? report(id, true, "") : report(id, false, "no snow along the way");
	}

	/** An empty jukebox plays their disc from a chest; one playing stops, its disc back (ABI 29). */
	private static JsonObject jukebox(ServerLevel level, JsonObject command, long id) {
		if (!(level.getBlockEntity(blockPos(command.getAsJsonArray("pos"))) instanceof net.minecraft.world.level.block.entity.JukeboxBlockEntity jukebox)) {
			return report(id, false, "no jukebox there");
		}
		Container from = container(level, blockPos(command.getAsJsonArray("from")));
		if (from == null) {
			return report(id, false, "no container to take from or give back to");
		}
		if (!jukebox.getTheItem().isEmpty()) {
			ItemStack disc = jukebox.getTheItem().copy();
			for (int slot = 0; slot < from.getContainerSize(); slot++) {
				if (from.getItem(slot).isEmpty()) {
					jukebox.setTheItem(ItemStack.EMPTY);
					from.setItem(slot, disc);
					return report(id, true, "stopped");
				}
			}
			jukebox.popOutTheItem();
			return report(id, true, "stopped");
		}
		int slot = command.get("slot").getAsInt();
		if (slot < 0 || slot >= from.getContainerSize() || !from.getItem(slot).has(net.minecraft.core.component.DataComponents.JUKEBOX_PLAYABLE)) {
			return report(id, false, "no disc in that slot");
		}
		jukebox.setTheItem(from.removeItem(slot, 1));

		return report(id, true, "");
	}

	/** The book on a lectern open to another page (ABI 29). */
	private static JsonObject turnPage(ServerLevel level, JsonObject command, long id) {
		if (!(level.getBlockEntity(blockPos(command.getAsJsonArray("pos"))) instanceof net.minecraft.world.level.block.entity.LecternBlockEntity lectern) || !lectern.hasBook()) {
			return report(id, false, "no book on a lectern there");
		}
		try {
			var count = net.minecraft.world.level.block.entity.LecternBlockEntity.class.getDeclaredMethod("getPageCount", ItemStack.class);
			count.setAccessible(true);
			int pages = (int) count.invoke(null, lectern.getBook());
			if (pages < 2) {
				return report(id, false, "the book has one page");
			}
			int to = (lectern.getPage() + 1 + level.getRandom().nextInt(pages - 1)) % pages;
			var set = net.minecraft.world.level.block.entity.LecternBlockEntity.class.getDeclaredMethod("setPage", int.class);
			set.setAccessible(true);
			set.invoke(lectern, to);
		} catch (ReflectiveOperationException e) {
			return report(id, false, "the page would not turn: " + e.getMessage());
		}

		return report(id, true, "");
	}

	/** An ingredient into a brewing stand that holds potions, a blaze powder when it has no fuel: the game brews (ABI 29). */
	private static JsonObject brew(ServerLevel level, JsonObject command, long id) {
		if (!(level.getBlockEntity(blockPos(command.getAsJsonArray("stand"))) instanceof net.minecraft.world.level.block.entity.BrewingStandBlockEntity stand)) {
			return report(id, false, "no brewing stand there");
		}
		boolean potions = !stand.getItem(0).isEmpty() || !stand.getItem(1).isEmpty() || !stand.getItem(2).isEmpty();
		if (!potions || !stand.getItem(3).isEmpty()) {
			return report(id, false, potions ? "the stand is brewing already" : "no potions in the stand");
		}
		Container from = container(level, blockPos(command.getAsJsonArray("from")));
		int slot = command.get("slot").getAsInt();
		if (from == null || slot < 0 || slot >= from.getContainerSize() || from.getItem(slot).isEmpty()) {
			return report(id, false, "no ingredient in that slot");
		}
		stand.setItem(3, from.removeItem(slot, 1));
		if (stand.getItem(4).isEmpty()) {
			for (int i = 0; i < from.getContainerSize(); i++) {
				if (from.getItem(i).is(net.minecraft.world.item.Items.BLAZE_POWDER)) {
					stand.setItem(4, from.removeItem(i, 1));
					break;
				}
			}
		}
		stand.setChanged();

		return report(id, true, "");
	}

	/** One enchantment, never a curse, gone from an enchanted item in a container, as a grindstone would (ABI 29). */
	private static JsonObject strip(ServerLevel level, JsonObject command, long id) {
		Container from = container(level, blockPos(command.getAsJsonArray("from")));
		if (from == null) {
			return report(id, false, "no container there");
		}
		for (int slot = 0; slot < from.getContainerSize(); slot++) {
			ItemStack stack = from.getItem(slot);
			var enchantments = stack.getOrDefault(net.minecraft.core.component.DataComponents.ENCHANTMENTS, net.minecraft.world.item.enchantment.ItemEnchantments.EMPTY);
			var removable = enchantments.keySet().stream().filter(h -> !h.is(net.minecraft.tags.EnchantmentTags.CURSE)).toList();
			if (removable.isEmpty()) {
				continue;
			}
			var chosen = removable.get(level.getRandom().nextInt(removable.size()));
			net.minecraft.world.item.enchantment.EnchantmentHelper.updateEnchantments(stack, m -> m.removeIf(h -> h.equals(chosen)));
			from.setChanged();
			return report(id, true, "");
		}

		return report(id, false, "nothing enchanted there");
	}

	/** A report from a why-not: done when it is empty. */
	private static JsonObject done(long id, String whyNot) {
		return report(id, whyNot.isEmpty(), whyNot);
	}

	/** A block taken away for a while and put back as it was (ABI 24); only one of {@code only}, never one with contents. */
	private static JsonObject hideBlock(ServerLevel level, JsonObject command, long id, Context context) {
		BlockPos pos = blockPos(command.getAsJsonArray("pos"));
		BlockState there = level.getBlockState(pos);
		if (there.isAir() || !replaceable(command, "only").contains(Senses.blockId(there))) {
			return report(id, false, "the place holds " + Senses.blockId(there));
		}
		if (level.getBlockEntity(pos) != null) {
			return report(id, false, "a block with contents is never hidden");
		}
		context.hidden().hide(level, pos, command.get("seconds").getAsFloat());

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

		// The very stack from a container's slot, only into an empty frame (ABI 28), or traded
		// for the frame's own with `swap` (ABI 40).
		if (command.has("from") && command.get("from").isJsonArray()) {
			boolean swap = command.has("swap") && command.get("swap").getAsBoolean();
			if (!frame.getItem().isEmpty() && !swap) {
				return report(id, false, "the frame holds something");
			}
			Container from = container(level, blockPos(command.getAsJsonArray("from")));
			int slot = command.has("slot") ? command.get("slot").getAsInt() : 0;
			if (from == null || slot < 0 || slot >= from.getContainerSize() || from.getItem(slot).isEmpty()) {
				return report(id, false, "nothing in that slot");
			}
			ItemStack had = frame.getItem().copy();
			ItemStack one = from.getItem(slot).getCount() == 1 || had.isEmpty() ? from.removeItem(slot, 1) : null;
			if (one == null) {
				return report(id, false, "a stack of more than one, and the frame's has nowhere to go");
			}
			frame.setItem(one);
			if (!had.isEmpty()) {
				from.setItem(slot, had);
			}
			from.setChanged();
			return report(id, true, "");
		}

		JsonObject item = command.getAsJsonObject("item");
		String itemId = item.has("id") ? item.get("id").getAsString() : "";
		Optional<Holder.Reference<Item>> kind = itemId.isEmpty() ? Optional.empty() : BuiltInRegistries.ITEM.get(Identifier.parse(itemId));
		if (!itemId.isEmpty() && kind.isEmpty()) {
			return report(id, false, "no such item");
		}
		// What the frame held is never lost: it lands where asked, or at the frame (ABI 40;
		// until then it was deleted).
		ItemStack had = frame.getItem().copy();
		frame.setItem(kind.map(k -> new ItemStack(k.value(), Math.max(1, item.get("count").getAsInt()))).orElse(ItemStack.EMPTY));
		if (!had.isEmpty()) {
			Vec3 at = frame.position();
			if (command.has("drop_at") && command.get("drop_at").isJsonArray()) {
				JsonArray p = command.getAsJsonArray("drop_at");
				at = new Vec3(p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble());
			}
			ItemEntity dropped = new ItemEntity(level, at.x, at.y, at.z, had, 0, 0, 0);
			dropped.setUnlimitedLifetime();
			level.addFreshEntity(dropped);
		}

		return report(id, true, kind.isPresent() || had.isEmpty() ? "" : "dropped " + BuiltInRegistries.ITEM.getKey(had.getItem()));
	}

	private static JsonObject equipEntity(ServerLevel level, JsonObject command, long id, WorldData data) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		if (!(entity instanceof net.minecraft.world.entity.LivingEntity living) || entity instanceof ServerPlayer) {
			return report(id, false, "no entity that wears things with that id here");
		}

		// The very stack kept (ABI 27: the scarecrow wears their own helmet), or a plain one.
		ItemStack stack = stackOf(command, data);
		if (stack == null || stack.isEmpty()) {
			return report(id, false, "no such item, or nothing kept under that name");
		}
		net.minecraft.world.entity.EquipmentSlot slot = living.getEquipmentSlotForItem(stack);
		if (!living.getItemBySlot(slot).isEmpty()) {
			// A kept stack goes back where it was kept (until 2026-11-10 it was lost here).
			if (command.has("kept") && !command.get("kept").getAsString().isEmpty()) {
				data.keep(command.get("kept").getAsString(), stack);
			}
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
		// Taken out through the connection even after a respawn, which shares it (until
		// 2026-11-10 a death left the ghost in the list until they rejoined).
		context.later().at(command.get("seconds").getAsFloat(), () -> {
			if (player.connection != null) {
				player.connection.send(new net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket(List.of(ghost)));
			}
		});

		return report(id, true, "");
	}

	/** The game's death lines that name the one who died and no one else. */
	private static final List<String> DEATHS = List.of("death.fell.accident.generic", "death.attack.drown", "death.attack.inWall",
			"death.attack.outOfWorld", "death.attack.generic", "death.attack.starve", "death.attack.freeze");

	/** A player's own entry in their list with this latency: -1 shows no connection. */
	private static net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket latency(ServerPlayer player, int latency) {
		var entry = new net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket.Entry(
				player.getUUID(), player.getGameProfile(), true, latency, player.gameMode.getGameModeForPlayer(), null, true, 0, null);
		var packet = new net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket(
				java.util.EnumSet.of(net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket.Action.UPDATE_LATENCY), List.of());

		return withEntries(packet, List.of(entry));
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
			// One of theirs again, as if just earned: the chat line, and the toast on their client.
			case "advancement" -> {
				List<net.minecraft.advancements.AdvancementHolder> had = new ArrayList<>();
				for (net.minecraft.advancements.AdvancementHolder holder : level.getServer().getAdvancements().getAllAdvancements()) {
					var display = holder.value().display();
					if (display.isPresent() && display.get().announceToChat() && !display.get().hidden()
							&& player.getAdvancements().getOrStartProgress(holder).isDone()) {
						had.add(holder);
					}
				}
				if (had.isEmpty()) {
					return report(id, false, "they have no advancement to show");
				}
				var holder = had.get((int) (Math.abs(text.hashCode() + id) % had.size()));
				player.sendSystemMessage(holder.value().display().get().type().createAnnouncement(holder, player));
				context.sendHint().accept(player, new craftkit.host.net.HintPayload("advancement_toast", Optional.empty(), 1f, holder.id().toString()));
			}
			// A death message about them while they live (ABI 28): only the game's own one-name lines.
			case "death" -> {
				if (!DEATHS.contains(text)) {
					return report(id, false, "no such death message");
				}
				player.sendSystemMessage(Component.translatable(text, name));
			}
			// Their own entry in the list shows no connection for a minute.
			case "no_ping" -> {
				player.connection.send(latency(player, -1));
				context.later().at(60f, () -> {
					if (!player.isRemoved()) {
						player.connection.send(latency(player, player.connection.latency()));
					}
				});
			}
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
			// Inside the world: past its top or bottom every block reads as air and nothing
			// can be set there (until 2026-11-10 the block and its contents were lost).
			if (level.isOutsideBuildHeight(candidate)) {
				continue;
			}
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
			BlockState standing = withState(block.get().value().defaultBlockState(), command.has("state") ? command.get("state").getAsString() : "");
			if (standing.canSurvive(level, at)) {
				// What was there is the mod's now, as a player's dig is: said, so it can count it.
				String was = Senses.blockId(level.getBlockState(at));
				level.setBlock(at, standing, Block.UPDATE_ALL);
				return report(id, true, level.getBlockState(at).isAir() && !was.equals("minecraft:air") ? "replaced " + was : "");
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
		// With no key the mod spends it (a sapling planted, ladders laid, the makings of what it
		// crafts): the docs of `take_item` say so.
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
		if (command.has("one_line") && command.get("one_line").getAsBoolean()) {
			// One line of theirs, the last with text, reads the word; the rest kept (ABI 28).
			List<Component> had = sign.getText(SignTextSlot.FRONT).getMessages(false);
			int last = -1;
			for (int i = 0; i < SignText.LINES && i < had.size(); i++) {
				lines.add(had.get(i));
				if (!had.get(i).getString().isBlank()) {
					last = i;
				}
			}
			if (last < 0) {
				return report(id, false, "the sign has no text");
			}
			lines.set(last, Component.literal(command.getAsJsonArray("lines").get(0).getAsString()));
		} else {
			for (JsonElement line : command.getAsJsonArray("lines")) {
				lines.add(Component.literal(line.getAsString()));
			}
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

	/** A block's state with `name=value,name=value` applied; what does not fit is left as it was. */
	static BlockState withState(BlockState state, String properties) {
		for (String pair : properties.split(",")) {
			String[] kv = pair.split("=", 2);
			if (kv.length == 2) {
				state = withProperty(state, kv[0].trim(), kv[1].trim());
			}
		}

		return state;
	}

	private static <T extends Comparable<T>> BlockState withProperty(BlockState state, String name, String value) {
		for (net.minecraft.world.level.block.state.properties.Property<?> property : state.getProperties()) {
			if (property.getName().equals(name)) {
				@SuppressWarnings("unchecked")
				net.minecraft.world.level.block.state.properties.Property<T> typed = (net.minecraft.world.level.block.state.properties.Property<T>) property;
				T now = state.getValue(typed);
				T next;
				if (value.equals("@next")) {
					java.util.List<T> values = new java.util.ArrayList<>(typed.getPossibleValues());
					next = values.get((values.indexOf(now) + 1) % values.size());
				} else if (value.equals("@up")) {
					// One higher, never round to the bottom: a full composter stays full (ABI 29).
					java.util.List<T> values = new java.util.ArrayList<>(typed.getPossibleValues());
					java.util.Collections.sort(values);
					int at = values.indexOf(now);
					next = at + 1 < values.size() ? values.get(at + 1) : now;
				} else {
					next = typed.getValue(value).orElse(now);
				}
				return state.setValue(typed, next);
			}
		}

		return state;
	}

	/** Bone meal, as a player uses it, up to `tries` times, until the block is something else. */
	private static JsonObject grow(ServerLevel level, JsonObject command, long id) {
		BlockPos pos = blockPos(command.getAsJsonArray("pos"));
		int tries = Math.clamp(command.get("tries").getAsInt(), 1, 16);
		BlockState was = level.getBlockState(pos);
		for (int i = 0; i < tries; i++) {
			BlockState state = level.getBlockState(pos);
			if (state.getBlock() != was.getBlock()) {
				break;
			}
			if (!(state.getBlock() instanceof net.minecraft.world.level.block.BonemealableBlock grows)
					|| !grows.isValidBonemealTarget(level, pos, state, net.minecraft.world.level.block.BonemealSource.INTERACTION)) {
				return report(id, false, "it will not grow");
			}
			if (grows.isBonemealSuccess(level, level.getRandom(), pos, state, net.minecraft.world.level.block.BonemealSource.INTERACTION)) {
				grows.performBonemeal(level, level.getRandom(), pos, state, net.minecraft.world.level.block.BonemealSource.INTERACTION);
			}
		}

		return level.getBlockState(pos).getBlock() != was.getBlock() ? report(id, true, "") : report(id, false, "it did not grow this time");
	}

	private static JsonObject setState(ServerLevel level, JsonObject command, long id) {
		BlockPos pos = blockPos(command.getAsJsonArray("pos"));
		BlockState state = level.getBlockState(pos);
		String property = command.get("property").getAsString();
		String value = command.get("value").getAsString();
		// A bed turned round where it stands: its two halves swap, facing the other way, both
		// set before either is checked, so it never breaks half way.
		if (value.equals("@turn") && state.getBlock() instanceof net.minecraft.world.level.block.BedBlock) {
			Direction facing = state.getValue(net.minecraft.world.level.block.BedBlock.FACING);
			boolean isHead = state.getValue(net.minecraft.world.level.block.BedBlock.PART) == net.minecraft.world.level.block.state.properties.BedPart.HEAD;
			BlockPos head = isHead ? pos : pos.relative(facing);
			BlockPos foot = isHead ? pos.relative(facing.getOpposite()) : pos;
			BlockState headState = level.getBlockState(head);
			BlockState footState = level.getBlockState(foot);
			if (!(headState.getBlock() instanceof net.minecraft.world.level.block.BedBlock) || !(footState.getBlock() instanceof net.minecraft.world.level.block.BedBlock)) {
				return report(id, false, "not a whole bed");
			}
			Direction turned = facing.getOpposite();
			level.setBlock(foot, headState.setValue(net.minecraft.world.level.block.BedBlock.FACING, turned), Block.UPDATE_CLIENTS);
			level.setBlock(head, footState.setValue(net.minecraft.world.level.block.BedBlock.FACING, turned), Block.UPDATE_CLIENTS);
			return report(id, true, "");
		}
		BlockState changed = withState(state, property + "=" + value);
		if (changed == state) {
			return report(id, false, "the block there has no " + property + " to change");
		}
		level.setBlock(pos, changed, Block.UPDATE_ALL);
		// A door is two blocks: its other half changes with it.
		if (state.hasProperty(BlockStateProperties.DOUBLE_BLOCK_HALF)) {
			BlockPos other = state.getValue(BlockStateProperties.DOUBLE_BLOCK_HALF) == net.minecraft.world.level.block.state.properties.DoubleBlockHalf.LOWER ? pos.above() : pos.below();
			BlockState half = level.getBlockState(other);
			if (half.is(state.getBlock())) {
				level.setBlock(other, withState(half, property + "=" + changed.getValue(findProperty(changed, property))), Block.UPDATE_ALL);
			}
		}

		return report(id, true, "");
	}

	private static net.minecraft.world.level.block.state.properties.Property<?> findProperty(BlockState state, String name) {
		return state.getProperties().stream().filter(p -> p.getName().equals(name)).findFirst().orElseThrow();
	}

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
		// A warped mob is the warp's to put back: held too, the two would leave it frozen for
		// good (until 2026-11-10).
		if (mob.entityTags().contains(Warps.TAG)) {
			return report(id, false, "the mob is warped");
		}
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}

		Vec3 at = null;
		if (command.has("at") && command.get("at").isJsonArray()) {
			JsonArray p = command.getAsJsonArray("at");
			at = new Vec3(p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble());
		}
		boolean creep = command.has("creep") && command.get("creep").getAsBoolean();
		context.holds().add(mob, player, command.get("seconds").getAsFloat(), at, 0, creep);

		return report(id, true, "");
	}

	/** What {@code place_entity} may place: an armour stand, or a farm animal. */
	private static final List<String> PLACEABLE = List.of("minecraft:armor_stand", "minecraft:cow", "minecraft:pig", "minecraft:sheep",
			"minecraft:chicken", "minecraft:goat", "minecraft:horse", "minecraft:donkey", "minecraft:mule", "minecraft:llama",
			"minecraft:rabbit", "minecraft:mooshroom", "minecraft:camel", "minecraft:squid");

	/** An armour stand or a farm animal placed (ABI 27): for a while when seconds are given, wearing a kept stack when named. */
	private static JsonObject placeEntity(ServerLevel level, JsonObject command, long id, Context context, WorldData data) {
		String of = command.get("of").getAsString();
		if (!PLACEABLE.contains(of)) {
			return report(id, false, "not something it places");
		}
		Optional<Holder.Reference<net.minecraft.world.entity.EntityType<?>>> type = BuiltInRegistries.ENTITY_TYPE.get(Identifier.parse(of));
		if (type.isEmpty()) {
			return report(id, false, "no such entity");
		}
		JsonArray p = command.getAsJsonArray("pos");
		Vec3 pos = new Vec3(p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble());
		BlockPos at = BlockPos.containing(pos);
		if (!level.getBlockState(at).getCollisionShape(level, at).isEmpty()) {
			return report(id, false, "no room there");
		}
		Entity entity = type.get().value().create(level, net.minecraft.world.entity.EntitySpawnReason.MOB_SUMMONED);
		if (entity == null) {
			return report(id, false, "it could not be made");
		}
		float yaw = command.has("yaw") ? command.get("yaw").getAsFloat() : 0;
		entity.snapTo(pos.x, pos.y, pos.z, yaw, 0);
		entity.setYHeadRot(yaw);
		if (command.has("kept") && !command.get("kept").getAsString().isEmpty() && entity instanceof net.minecraft.world.entity.LivingEntity living) {
			ItemStack worn = data.giveBack(command.get("kept").getAsString());
			if (worn != null && !worn.isEmpty()) {
				living.setItemSlot(living.getEquipmentSlotForItem(worn), worn);
			}
		}
		level.addFreshEntity(entity);
		float seconds = command.has("seconds") ? command.get("seconds").getAsFloat() : 0;
		if (seconds > 0) {
			context.temporaries().add(entity, seconds);
		}
		// Still, facing a player, for its time (ABI 30: the shape under the water).
		if (command.has("face") && !command.get("face").isJsonNull() && entity instanceof net.minecraft.world.entity.Mob mob) {
			ServerPlayer facing = context.senses().player(level, command.get("face").getAsInt());
			if (facing != null) {
				context.holds().add(mob, facing, seconds > 0 ? seconds : 20);
			}
		}

		return report(id, true, "");
	}

	/** A hive's bees out, held still around a place for a while, then home (ABI 27). */
	private static JsonObject releaseBees(ServerLevel level, JsonObject command, long id, Context context) {
		BlockPos hive = blockPos(command.getAsJsonArray("hive"));
		if (!(level.getBlockEntity(hive) instanceof net.minecraft.world.level.block.entity.BeehiveBlockEntity beehive) || beehive.isEmpty()) {
			return report(id, false, "no hive with bees there");
		}
		beehive.emptyAllLivingFromHive(null, level.getBlockState(hive), net.minecraft.world.level.block.entity.BeehiveBlockEntity.BeeReleaseStatus.BEE_RELEASED);
		JsonArray p = command.getAsJsonArray("to");
		Vec3 to = new Vec3(p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble());
		ServerPlayer nearest = level.getNearestPlayer(to.x, to.y, to.z, 64, false) instanceof ServerPlayer sp ? sp : null;
		List<net.minecraft.world.entity.animal.bee.Bee> bees = level.getEntitiesOfClass(net.minecraft.world.entity.animal.bee.Bee.class,
				new net.minecraft.world.phys.AABB(hive).inflate(3));
		if (bees.isEmpty() || nearest == null) {
			return report(id, false, bees.isEmpty() ? "no bee came out" : "no player near");
		}
		for (int i = 0; i < bees.size(); i++) {
			double angle = i * Math.PI * 2 / bees.size();
			bees.get(i).teleportTo(to.x + Math.cos(angle) * 1.2, to.y + (i % 2) * 0.4, to.z + Math.sin(angle) * 1.2);
			context.holds().add(bees.get(i), nearest, command.get("seconds").getAsFloat());
		}

		return report(id, true, "");
	}

	/** An animal on a lead tied to another fence post within eight blocks of its own (ABI 27). */
	private static JsonObject moveLeash(ServerLevel level, JsonObject command, long id) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		if (!(entity instanceof net.minecraft.world.entity.Leashable leashable) || !leashable.isLeashed()
				|| !(leashable.getLeashHolder() instanceof net.minecraft.world.entity.decoration.LeashFenceKnotEntity knot)) {
			return report(id, false, "no animal tied to a post with that id here");
		}
		BlockPos post = knot.getPos();
		// The nearest other post: a fence within eight blocks, two up or down.
		BlockPos other = null;
		for (BlockPos at : BlockPos.betweenClosed(post.offset(-8, -2, -8), post.offset(8, 2, 8))) {
			if (!at.equals(post) && level.getBlockState(at).is(net.minecraft.tags.BlockTags.FENCES)
					&& (other == null || at.distSqr(post) < other.distSqr(post))) {
				other = at.immutable();
			}
		}
		if (other == null) {
			return report(id, false, "no other post near");
		}
		leashable.setLeashedTo(net.minecraft.world.entity.decoration.LeashFenceKnotEntity.getOrCreateKnot(level, other), true);
		boolean stillHeld = !level.getEntitiesOfClass(net.minecraft.world.entity.Mob.class, knot.getBoundingBox().inflate(12),
				m -> m instanceof net.minecraft.world.entity.Leashable l && l.getLeashHolder() == knot).isEmpty();
		if (!stillHeld) {
			knot.discard();
		}

		return report(id, true, "");
	}

	/** The figure digs into the rock ahead, on its player's client alone (ABI 35). */
	private static JsonObject digIn(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		JsonArray way = command.getAsJsonArray("way");
		Direction direction = Direction.getApproximateNearest(way.get(0).getAsInt(), 0, way.get(1).getAsInt());
		boolean dug = context.figures().digIn(level, player, command.get("figure").getAsInt(), blockPos(command.getAsJsonArray("from")), direction,
				command.has("through") && command.get("through").getAsBoolean());

		return dug ? report(id, true, "") : report(id, false, "no rock face ahead");
	}

	/** The figure shown to a player swings once, perhaps a hit, never below half a heart with spare (ABI 33). */
	private static JsonObject figureSwing(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		boolean swung = context.figures().swing(player, command.get("figure").getAsInt(), command.get("damage").getAsFloat(),
				command.has("spare") && command.get("spare").getAsBoolean());

		return swung ? report(id, true, "") : report(id, false, "no figure shown with that number");
	}

	/**
	 * Blocks laid relative to a place (ABI 34): the ground found when the place's y is below
	 * -1000 (the highest block that stops movement, leaves left out, water counting), and each
	 * block only into air or over one of {@code replace}, never over contents. A door's lower
	 * half goes in without telling its neighbours, so it does not break before its upper half.
	 * Reports {@code base <y> placed <n>}.
	 */
	private static JsonObject placeBlueprint(ServerLevel level, JsonObject command, long id) {
		JsonArray at = command.getAsJsonArray("at");
		int x = at.get(0).getAsInt();
		int y = at.get(1).getAsInt();
		int z = at.get(2).getAsInt();
		if (y <= -1000) {
			BlockPos column = new BlockPos(x, 0, z);
			if (!level.isLoaded(column)) {
				return report(id, false, "the place is not loaded");
			}
			y = level.getHeightmapPos(net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, column).getY() - 1;
		}
		List<String> replace = replaceable(command);
		int placed = 0;
		for (JsonElement element : command.getAsJsonArray("parts")) {
			JsonObject part = element.getAsJsonObject();
			JsonArray rel = part.getAsJsonArray("at");
			BlockPos pos = new BlockPos(x + rel.get(0).getAsInt(), y + rel.get(1).getAsInt(), z + rel.get(2).getAsInt());
			if (!level.isLoaded(pos) || taken(level, pos, replace, false) != null) {
				continue;
			}
			Optional<Holder.Reference<Block>> block = BuiltInRegistries.BLOCK.get(Identifier.parse(part.get("block").getAsString()));
			if (block.isEmpty()) {
				continue;
			}
			BlockState state = withState(block.get().value().defaultBlockState(), part.has("state") ? part.get("state").getAsString() : "");
			boolean lowerHalf = state.hasProperty(BlockStateProperties.DOUBLE_BLOCK_HALF)
					&& state.getValue(BlockStateProperties.DOUBLE_BLOCK_HALF) == net.minecraft.world.level.block.state.properties.DoubleBlockHalf.LOWER;
			level.setBlock(pos, state, lowerHalf ? Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE : Block.UPDATE_ALL);
			placed++;
		}

		return report(id, placed > 0, "base " + y + " placed " + placed);
	}

	/** A place from a command, on the ground there (a block and a half up) when its y is below -1000. */
	private static Vec3 grounded(ServerLevel level, JsonArray p) {
		double x = p.get(0).getAsDouble();
		double y = p.get(1).getAsDouble();
		double z = p.get(2).getAsDouble();
		if (y <= -1000) {
			y = level.getHeightmapPos(net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, BlockPos.containing(x, 0, z)).getY() + 1.5;
		}
		return new Vec3(x, y, z);
	}

	/** The effects a splash, a cloud or its brew may carry: none that harms (ABI 42). */
	private static final java.util.Set<String> SPLASHES = java.util.Set.of("darkness", "slowness", "blindness", "nausea", "levitation");
	private static final java.util.Set<String> CLOUDS = java.util.Set.of("slowness", "weakness");
	/** The effects its brew may hold: none that harms or carries them anywhere (until 2026-11-10 a list of the harmful let levitation and the rest through). */
	private static final java.util.Set<String> BREWS = java.util.Set.of("night_vision", "darkness", "invisibility", "slowness", "weakness", "nausea", "glowing", "luck", "unluck", "water_breathing", "fire_resistance");
	/** The kinds a potion of theirs may become: none that harms. */
	private static final java.util.Set<String> SAFE_POTIONS = java.util.Set.of("night_vision", "invisibility", "water_breathing", "fire_resistance", "leaping", "swiftness", "slow_falling");

	private static Optional<Holder.Reference<net.minecraft.world.effect.MobEffect>> effect(String name) {
		return BuiltInRegistries.MOB_EFFECT.get(Identifier.withDefaultNamespace(name));
	}

	/** A real arrow from the dark, aimed as a skeleton aims; spared, never picked up, staying where it lands (ABI 42). */
	private static JsonObject shoot(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		Vec3 from = grounded(level, command.getAsJsonArray("from"));
		JsonArray a = command.getAsJsonArray("at");
		Vec3 at = new Vec3(a.get(0).getAsDouble(), a.get(1).getAsDouble(), a.get(2).getAsDouble());
		// Aimed where they are now, not where they stood when it was asked a second ago: at their
		// chest, or a block and a third beside their head on the side it shoots from.
		Vec3 eye = player.getEyePosition();
		if (command.get("damage").getAsFloat() > 0) {
			at = player.position().add(0, 1.1, 0);
		} else {
			Vec3 across = new Vec3(eye.z - from.z, 0, from.x - eye.x).normalize();
			Vec3 side = at.subtract(eye).dot(across) >= 0 ? across : across.scale(-1);
			at = eye.add(side.scale(1.3)).add(0, 0.1, 0);
		}
		var arrow = new net.minecraft.world.entity.projectile.arrow.Arrow(level, from.x, from.y, from.z, new ItemStack(net.minecraft.world.item.Items.ARROW), null);
		arrow.pickup = net.minecraft.world.entity.projectile.arrow.AbstractArrow.Pickup.CREATIVE_ONLY;
		float damage = Spare.arrowHarm(Math.max(0, command.get("damage").getAsFloat()), player.getHealth());
		// An arrow's harm is its speed times its base: at 1.6, the base is the harm over 1.6.
		arrow.setBaseDamage(damage / 1.6);
		double dx = at.x - from.x;
		double dz = at.z - from.z;
		double dy = at.y - from.y;
		arrow.shoot(dx, dy + Math.sqrt(dx * dx + dz * dz) * 0.2, dz, 1.6f, 0);
		// It stays where it lands, to be found: never gone after a minute in the ground.
		try {
			java.lang.reflect.Field life = net.minecraft.world.entity.projectile.arrow.AbstractArrow.class.getDeclaredField("life");
			life.setAccessible(true);
			// Saved as a short: the least a short holds, about half an hour in the ground.
			life.setInt(arrow, Short.MIN_VALUE);
		} catch (ReflectiveOperationException | RuntimeException ignored) {
			// The field moved: it goes after a minute, as any arrow does.
		}
		level.addFreshEntity(arrow);
		level.playSound(null, from.x, from.y, from.z, net.minecraft.sounds.SoundEvents.ARROW_SHOOT, net.minecraft.sounds.SoundSource.HOSTILE, 0.8f, 1f);

		return report(id, true, damage > 0 ? "aimed at them, " + damage : "aimed past them");
	}

	/** A splash potion of one effect thrown at their feet from the dark (ABI 42). */
	private static JsonObject splashPotion(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		String name = command.get("effect").getAsString();
		var effect = effect(name);
		if (!SPLASHES.contains(name) || effect.isEmpty()) {
			return report(id, false, "not an effect it throws");
		}
		float seconds = Math.max(0.5f, Math.min(command.get("seconds").getAsFloat(), name.equals("levitation") ? 2 : 15));
		ItemStack potion = new ItemStack(net.minecraft.world.item.Items.SPLASH_POTION);
		potion.set(net.minecraft.core.component.DataComponents.POTION_CONTENTS, new net.minecraft.world.item.alchemy.PotionContents(Optional.empty(), Optional.empty(),
				List.of(new net.minecraft.world.effect.MobEffectInstance(effect.get(), (int) (seconds * 20))), Optional.empty()));
		Vec3 from = grounded(level, command.getAsJsonArray("from"));
		var thrown = new net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownSplashPotion(level, from.x, from.y, from.z, potion);
		double dx = player.getX() - from.x;
		double dz = player.getZ() - from.z;
		thrown.shoot(dx, player.getY() - from.y + Math.sqrt(dx * dx + dz * dz) * 0.2, dz, 0.75f, 0);
		level.addFreshEntity(thrown);

		return report(id, true, "");
	}

	/** Its brew, put into a container, named; nothing in it that harms (ABI 42). */
	private static JsonObject putPotion(ServerLevel level, JsonObject command, long id) {
		Container into = container(level, blockPos(command.getAsJsonArray("into")));
		if (into == null) {
			return report(id, false, "no container there");
		}
		List<net.minecraft.world.effect.MobEffectInstance> effects = new ArrayList<>();
		for (JsonElement each : command.getAsJsonArray("effects")) {
			JsonArray pair = each.getAsJsonArray();
			String name = pair.get(0).getAsString();
			var effect = effect(name);
			if (!BREWS.contains(name) || effect.isEmpty()) {
				return report(id, false, "not an effect it brews: " + name);
			}
			effects.add(new net.minecraft.world.effect.MobEffectInstance(effect.get(), (int) (Math.max(1, Math.min(pair.get(1).getAsFloat(), 600)) * 20)));
		}
		ItemStack potion = new ItemStack(net.minecraft.world.item.Items.POTION);
		potion.set(net.minecraft.core.component.DataComponents.POTION_CONTENTS, new net.minecraft.world.item.alchemy.PotionContents(Optional.empty(), Optional.of(0x2B2234), effects, Optional.empty()));
		potion.set(net.minecraft.core.component.DataComponents.CUSTOM_NAME, Component.literal(command.get("name").getAsString()));
		for (int slot = 0; slot < into.getContainerSize(); slot++) {
			if (into.getItem(slot).isEmpty()) {
				into.setItem(slot, potion);
				into.setChanged();
				return report(id, true, "");
			}
		}

		return report(id, false, "the container is full");
	}

	/** A potion of theirs, another kind, its name kept (ABI 42). */
	private static JsonObject changePotion(ServerLevel level, JsonObject command, long id) {
		Container in = container(level, blockPos(command.getAsJsonArray("pos")));
		int slot = command.get("slot").getAsInt();
		if (in == null || slot < 0 || slot >= in.getContainerSize()) {
			return report(id, false, "no such slot");
		}
		ItemStack stack = in.getItem(slot);
		var had = stack.get(net.minecraft.core.component.DataComponents.POTION_CONTENTS);
		String into = command.get("into").getAsString();
		var kind = BuiltInRegistries.POTION.get(Identifier.withDefaultNamespace(into));
		if (had == null || !SAFE_POTIONS.contains(into) || kind.isEmpty()) {
			return report(id, false, "no potion there, or not a kind it turns one into");
		}
		// Another kind than it is: the next safe one when asked for its own.
		var asked = kind.get();
		if (had.potion().filter(p -> p.equals(asked)).isPresent()) {
			List<String> safe = new ArrayList<>(new java.util.TreeSet<>(SAFE_POTIONS));
			String other = safe.get((safe.indexOf(into) + 1) % safe.size());
			kind = BuiltInRegistries.POTION.get(Identifier.withDefaultNamespace(other));
			if (kind.isEmpty()) {
				return report(id, false, "no other kind");
			}
		}
		Component name = stack.getHoverName();
		stack.set(net.minecraft.core.component.DataComponents.POTION_CONTENTS, new net.minecraft.world.item.alchemy.PotionContents(kind.get()));
		stack.set(net.minecraft.core.component.DataComponents.CUSTOM_NAME, name.copy().withStyle(s -> s.withItalic(false)));
		in.setChanged();

		return report(id, true, "");
	}

	/** A lingering cloud of one effect, gone as it first touches someone (ABI 42). */
	private static JsonObject effectCloud(ServerLevel level, JsonObject command, long id) {
		String name = command.get("effect").getAsString();
		var effect = effect(name);
		if (!CLOUDS.contains(name) || effect.isEmpty()) {
			return report(id, false, "not an effect it leaves");
		}
		JsonArray p = command.getAsJsonArray("pos");
		float radius = Math.max(0.5f, Math.min(command.get("radius").getAsFloat(), 3));
		var cloud = new net.minecraft.world.entity.AreaEffectCloud(level, p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble());
		cloud.setRadius(radius);
		cloud.setRadiusOnUse(-radius);
		cloud.setWaitTime(0);
		cloud.setDuration((int) (Math.min(command.get("seconds").getAsFloat(), 1200) * 20));
		cloud.addEffect(new net.minecraft.world.effect.MobEffectInstance(effect.get(), 200));
		level.addFreshEntity(cloud);

		return report(id, true, "");
	}

	/** Every stack of a container dropped at a place, kept from despawning; with {@code remove} the empty container gone (ABI 41). */
	private static JsonObject spillContainer(ServerLevel level, JsonObject command, long id) {
		BlockPos from = blockPos(command.getAsJsonArray("from"));
		Container container = container(level, from);
		if (container == null) {
			return report(id, false, "no container there");
		}
		JsonArray to = command.getAsJsonArray("to");
		int spilled = 0;
		for (int slot = 0; slot < container.getContainerSize(); slot++) {
			ItemStack stack = container.removeItemNoUpdate(slot);
			if (!stack.isEmpty()) {
				ItemEntity entity = new ItemEntity(level, to.get(0).getAsDouble(), to.get(1).getAsDouble(), to.get(2).getAsDouble(), stack, 0, 0, 0);
				entity.setUnlimitedLifetime();
				level.addFreshEntity(entity);
				spilled++;
			}
		}
		container.setChanged();
		if (command.has("remove") && command.get("remove").getAsBoolean()) {
			level.removeBlockEntity(from);
			level.setBlock(from, Blocks.AIR.defaultBlockState(), Block.UPDATE_ALL);
		}

		return report(id, true, "spilled " + spilled);
	}

	/** A Nether portal lit in an empty obsidian frame within six blocks, as flint and steel would (ABI 32). */
	private static JsonObject lightPortal(ServerLevel level, JsonObject command, long id) {
		BlockPos near = blockPos(command.getAsJsonArray("near"));
		for (BlockPos at : BlockPos.betweenClosed(near.offset(-6, -6, -6), near.offset(6, 6, 6))) {
			if (!level.getBlockState(at).isAir()) {
				continue;
			}
			for (Direction.Axis axis : new Direction.Axis[] {Direction.Axis.X, Direction.Axis.Z}) {
				var shape = net.minecraft.world.level.portal.PortalShape.findEmptyPortalShape(level, at.immutable(), axis);
				if (shape.isPresent()) {
					shape.get().createPortalBlocks(level);
					level.playSound(null, at, net.minecraft.sounds.SoundEvents.FLINTANDSTEEL_USE, net.minecraft.sounds.SoundSource.BLOCKS, 1f, 1f);
					return report(id, true, "");
				}
			}
		}

		return report(id, false, "no empty frame there");
	}

	/** The village emptied into one house till dawn (ABI 31). */
	private static JsonObject gatherVillagers(ServerLevel level, JsonObject command, long id, Context context) {
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		int packed = context.gatherings().gather(level, blockPos(command.getAsJsonArray("center")), blockPos(command.getAsJsonArray("door")),
				command.get("seconds").getAsFloat(), player, context.holds());

		return packed > 0 ? report(id, true, "") : report(id, false, "no villager, or no room behind that door");
	}

	/** What it took, for sale at a villager for an emerald, once (ABI 31). */
	private static JsonObject offerTrade(ServerLevel level, JsonObject command, long id, Context context, WorldData data) {
		if (!(level.getEntity(command.get("entity").getAsInt()) instanceof net.minecraft.world.entity.npc.villager.AbstractVillager villager)) {
			return report(id, false, "no villager with that id here");
		}
		ItemStack stack = data.giveBack(command.get("kept").getAsString());
		if (stack == null || stack.isEmpty()) {
			return report(id, false, "nothing kept under that name");
		}
		context.gatherings().sell(villager, stack);

		return report(id, true, "");
	}

	/** An animal runs at a player and hits (ABI 27); with spare, never below half a heart. */
	private static JsonObject charge(ServerLevel level, JsonObject command, long id, Context context) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		if (!(entity instanceof net.minecraft.world.entity.animal.Animal animal)) {
			return report(id, false, "no animal with that id here");
		}
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		context.charges().add(animal, player, command.get("damage").getAsFloat(), command.get("hits").getAsInt(),
				command.get("seconds").getAsFloat(), command.has("spare") && command.get("spare").getAsBoolean());

		return report(id, true, "");
	}

	/** An animal warped for a while and put back (ABI 25): only an animal, never a held mob or one already warped. */
	private static JsonObject warpMob(ServerLevel level, JsonObject command, long id, Context context) {
		Entity entity = level.getEntity(command.get("entity").getAsInt());
		if (!(entity instanceof net.minecraft.world.entity.animal.Animal animal)) {
			return report(id, false, "no animal with that id here");
		}
		ServerPlayer player = context.senses().player(level, command.get("player").getAsInt());
		if (player == null) {
			return report(id, false, "the player is not here");
		}
		Vec3 to = null;
		if (command.has("to") && command.get("to").isJsonArray()) {
			JsonArray p = command.getAsJsonArray("to");
			to = new Vec3(p.get(0).getAsDouble(), p.get(1).getAsDouble(), p.get(2).getAsDouble());
		}
		boolean warped = context.warps().add(animal, player, command.get("seconds").getAsFloat(),
				command.has("scale") ? command.get("scale").getAsDouble() : 1.0,
				command.has("flip") && command.get("flip").getAsBoolean(),
				command.has("creep") && command.get("creep").getAsBoolean(), to);

		return warped ? report(id, true, "") : report(id, false, "it is held or warped already");
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
