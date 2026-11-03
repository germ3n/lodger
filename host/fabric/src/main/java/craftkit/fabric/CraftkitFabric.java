package craftkit.fabric;

import craftkit.host.Host;
import craftkit.host.net.HintPayload;
import craftkit.host.senses.Senses;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.entity.event.v1.EntitySleepEvents;
import net.fabricmc.fabric.api.entity.event.v1.ServerEntityLevelChangeEvents;
import net.fabricmc.fabric.api.entity.event.v1.ServerLivingEntityEvents;
import net.fabricmc.fabric.api.entity.event.v1.ServerPlayerEvents;
import net.fabricmc.fabric.api.event.player.AttackEntityCallback;
import net.fabricmc.fabric.api.event.player.PlayerBlockBreakEvents;
import net.fabricmc.fabric.api.event.player.AttackBlockCallback;
import net.fabricmc.fabric.api.event.player.UseBlockCallback;
import net.fabricmc.fabric.api.message.v1.ServerMessageEvents;
import net.fabricmc.fabric.api.networking.v1.ServerPlayConnectionEvents;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.player.Player;
import net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLevelEvents;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.fabricmc.fabric.api.networking.v1.PayloadTypeRegistry;
import net.fabricmc.fabric.api.networking.v1.ServerConfigurationConnectionEvents;
import net.fabricmc.fabric.api.networking.v1.ServerConfigurationNetworking;
import net.minecraft.network.chat.Component;

/** The host on Fabric: load the mods, refuse clients without the host, register the command. */
public final class CraftkitFabric implements ModInitializer {
	private static Host host;

	@Override
	public void onInitialize() {
		host = Host.start(new FabricPlatform());

		PayloadTypeRegistry.clientboundConfiguration().register(Hello.TYPE, Hello.CODEC);
		PayloadTypeRegistry.clientboundPlay().register(HintPayload.TYPE, HintPayload.CODEC);
		PayloadTypeRegistry.clientboundPlay().register(craftkit.host.net.DebugTellPayload.TYPE, craftkit.host.net.DebugTellPayload.CODEC);
		PayloadTypeRegistry.serverboundPlay().register(craftkit.host.net.DebugAskPayload.TYPE, craftkit.host.net.DebugAskPayload.CODEC);
		net.fabricmc.fabric.api.networking.v1.ServerPlayNetworking.registerGlobalReceiver(craftkit.host.net.DebugAskPayload.TYPE,
				(payload, context) -> context.server().execute(() -> host.debugAsked(context.player(), payload)));
		host.worlds().senses().mods(new FabricPlatform().loadedMods());
		hookSenses(host.worlds().senses());
		ServerConfigurationConnectionEvents.CONFIGURE.register((handler, server) -> {
			if (!ServerConfigurationNetworking.canSend(handler, Hello.TYPE)) {
				handler.disconnect(Component.literal("This server runs craftkit, which your client needs too."));
			}
		});

		CommandRegistrationCallback.EVENT.register((dispatcher, registry, environment) -> host.registerCommands(dispatcher));
		// A dedicated server fetches and loads before the world opens; the client did it at the
		// title screen, so for an integrated server this finds nothing to do.
		ServerLifecycleEvents.SERVER_STARTING.register(server -> host.prepareBlocking());
		ServerLevelEvents.LOAD.register((server, level) -> host.worlds().levelLoaded(server, level));
		ServerLevelEvents.UNLOAD.register((server, level) -> host.worlds().levelUnloaded(level));
		ServerTickEvents.END_SERVER_TICK.register(server -> host.worlds().tick(server));
		ServerLifecycleEvents.SERVER_STOPPING.register(server -> host.worlds().closeAll());
	}

	/** The game's events into the senses. Fabric's side of the Minecraft-facing edge. */
	private static void hookSenses(Senses senses) {
		ServerPlayConnectionEvents.JOIN.register((handler, sender, server) -> senses.noticedJoined(handler.player));
		ServerPlayConnectionEvents.DISCONNECT.register((handler, server) -> senses.noticedLeft(handler.player));
		UseBlockCallback.EVENT.register((player, level, hand, hit) -> {
			if (player instanceof ServerPlayer serverPlayer) {
				senses.noticedUsedBlock(serverPlayer, hit.getBlockPos());
			}
			return InteractionResult.PASS;
		});
		AttackBlockCallback.EVENT.register((player, level, hand, pos, direction) -> {
			if (player instanceof ServerPlayer serverPlayer) {
				senses.noticedStruckBlock(serverPlayer, pos);
			}
			return InteractionResult.PASS;
		});
		PlayerBlockBreakEvents.AFTER.register((level, player, pos, state, entity) -> {
			if (player instanceof ServerPlayer serverPlayer) {
				senses.noticedBrokeBlock(serverPlayer, pos, state);
			}
		});
		EntitySleepEvents.ALLOW_SLEEPING.register((player, pos) -> {
			if (player instanceof ServerPlayer serverPlayer && senses.shouldRefuseSleep(serverPlayer)) {
				senses.noticedSleepRefused(serverPlayer, pos, "not_safe");
				return Player.BedSleepingProblem.NOT_SAFE;
			}
			return null;
		});
		ServerLivingEntityEvents.AFTER_DEATH.register((entity, source) -> {
			if (entity instanceof ServerPlayer serverPlayer) {
				senses.noticedDied(serverPlayer);
			} else if (source.getEntity() instanceof ServerPlayer killer) {
				senses.noticedKilled(killer, entity);
			}
		});
		ServerPlayerEvents.AFTER_RESPAWN.register((oldPlayer, newPlayer, alive) -> senses.noticedRespawned(newPlayer));
		ServerEntityLevelChangeEvents.AFTER_PLAYER_CHANGE_LEVEL.register((player, from, to) ->
				senses.noticedChangedDimension(player, from.dimension().identifier().toString(), to.dimension().identifier().toString()));
		AttackEntityCallback.EVENT.register((player, level, hand, target, hit) -> {
			if (player instanceof ServerPlayer serverPlayer) {
				senses.noticedHit(serverPlayer, target);
			}
			return InteractionResult.PASS;
		});
		ServerMessageEvents.CHAT_MESSAGE.register((message, player, params) -> senses.noticedChat(player, message.signedContent()));
	}

	/** Called by the block-placement mixin. */
	public static void placedBlock(ServerPlayer player, net.minecraft.core.BlockPos pos, net.minecraft.world.level.block.state.BlockState state) {
		if (host != null) {
			host.worlds().senses().noticedPlacedBlock(player, pos, state);
		}
	}

	public static Host host() {
		return host;
	}
}
