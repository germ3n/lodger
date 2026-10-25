package craftkit.neoforge;

import craftkit.host.Host;
import craftkit.host.client.Hints;
import craftkit.host.net.HintPayload;
import craftkit.host.senses.Senses;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Player;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.neoforge.event.ServerChatEvent;
import net.neoforged.neoforge.event.entity.living.LivingDeathEvent;
import net.neoforged.neoforge.event.entity.player.AttackEntityEvent;
import net.neoforged.neoforge.event.entity.player.CanPlayerSleepEvent;
import net.neoforged.neoforge.event.entity.player.PlayerEvent;
import net.neoforged.neoforge.event.entity.player.PlayerInteractEvent;
import net.neoforged.neoforge.event.level.BlockEvent;
import net.neoforged.neoforge.event.level.block.BreakBlockEvent;
import net.neoforged.fml.common.Mod;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.fml.loading.FMLEnvironment;
import net.neoforged.neoforge.client.event.ScreenEvent;
import net.neoforged.neoforge.event.RegisterCommandsEvent;
import net.neoforged.neoforge.event.level.LevelEvent;
import net.neoforged.neoforge.event.server.ServerStartingEvent;
import net.neoforged.neoforge.event.server.ServerStoppingEvent;
import net.neoforged.neoforge.event.tick.ServerTickEvent;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;
import net.neoforged.neoforge.network.registration.PayloadRegistrar;

/** The host on NeoForge: load the mods, require the host on clients, register the command. */
@Mod(Host.MOD_ID)
public final class CraftkitNeoForge {
	private static Host host;

	public CraftkitNeoForge(IEventBus modBus) {
		host = Host.start(new NeoForgePlatform());

		modBus.addListener(this::registerPayloads);
		NeoForge.EVENT_BUS.addListener(this::registerCommands);
		NeoForge.EVENT_BUS.addListener(this::serverStarting);
		NeoForge.EVENT_BUS.addListener(this::levelLoad);
		NeoForge.EVENT_BUS.addListener(this::levelUnload);
		NeoForge.EVENT_BUS.addListener(this::serverTick);
		NeoForge.EVENT_BUS.addListener(this::serverStopping);

		if (FMLEnvironment.getDist() == Dist.CLIENT) {
			NeoForge.EVENT_BUS.addListener(this::screenInit);
		}

		host.worlds().senses().mods(new NeoForgePlatform().loadedMods());
		hookSenses(host.worlds().senses());
	}

	private void serverStarting(ServerStartingEvent event) {
		host.prepareBlocking();
	}

	private void levelLoad(LevelEvent.Load event) {
		if (event.getLevel() instanceof ServerLevel level) {
			host.worlds().levelLoaded(level.getServer(), level);
		}
	}

	private void levelUnload(LevelEvent.Unload event) {
		if (event.getLevel() instanceof ServerLevel level) {
			host.worlds().levelUnloaded(level);
		}
	}

	private void serverTick(ServerTickEvent.Post event) {
		host.worlds().tick(event.getServer());
	}

	private void serverStopping(ServerStoppingEvent event) {
		host.worlds().closeAll();
	}

	private void screenInit(ScreenEvent.Init.Post event) {
		craftkit.host.client.TitleGate.onScreen(net.minecraft.client.Minecraft.getInstance(), event.getScreen());
	}

	private void registerPayloads(RegisterPayloadHandlersEvent event) {
		// Not optional, so a client without the host fails NeoForge's own channel check.
		PayloadRegistrar registrar = event.registrar("1");
		registrar.configurationToClient(Hello.TYPE, Hello.CODEC, (payload, context) -> {
			// Being registered is the whole message.
		});
		registrar.playToClient(HintPayload.TYPE, HintPayload.CODEC, (payload, context) -> Hints.show(payload));
	}

	/** The game's events into the senses. NeoForge's side of the Minecraft-facing edge. */
	private void hookSenses(Senses senses) {
		NeoForge.EVENT_BUS.addListener((PlayerEvent.PlayerLoggedInEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedJoined(player);
			}
		});
		NeoForge.EVENT_BUS.addListener((PlayerEvent.PlayerLoggedOutEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedLeft(player);
			}
		});
		NeoForge.EVENT_BUS.addListener((PlayerInteractEvent.RightClickBlock event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedUsedBlock(player, event.getPos());
			}
		});
		NeoForge.EVENT_BUS.addListener((BlockEvent.EntityPlaceEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedPlacedBlock(player, event.getPos(), event.getPlacedBlock());
			}
		});
		NeoForge.EVENT_BUS.addListener((BreakBlockEvent event) -> {
			if (event.getPlayer() instanceof ServerPlayer player) {
				senses.noticedBrokeBlock(player, event.getPos(), event.getState());
			}
		});
		NeoForge.EVENT_BUS.addListener((CanPlayerSleepEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player && senses.shouldRefuseSleep(player)) {
				senses.noticedSleepRefused(player, event.getPos(), "not_safe");
				event.setProblem(Player.BedSleepingProblem.NOT_SAFE);
			}
		});
		NeoForge.EVENT_BUS.addListener((LivingDeathEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedDied(player);
			}
		});
		NeoForge.EVENT_BUS.addListener((PlayerEvent.PlayerRespawnEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedRespawned(player);
			}
		});
		NeoForge.EVENT_BUS.addListener((PlayerEvent.PlayerChangedDimensionEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedChangedDimension(player, event.getFrom().identifier().toString(), event.getTo().identifier().toString());
			}
		});
		NeoForge.EVENT_BUS.addListener((AttackEntityEvent event) -> {
			if (event.getEntity() instanceof ServerPlayer player) {
				senses.noticedHit(player, event.getTarget());
			}
		});
		NeoForge.EVENT_BUS.addListener((ServerChatEvent event) -> senses.noticedChat(event.getPlayer(), event.getRawText()));
	}

	private void registerCommands(RegisterCommandsEvent event) {
		host.registerCommands(event.getDispatcher());
	}

	public static Host host() {
		return host;
	}
}
