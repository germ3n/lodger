package craftkit.neoforge;

import craftkit.host.Host;
import net.neoforged.bus.api.IEventBus;
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
	}

	private void registerCommands(RegisterCommandsEvent event) {
		host.registerCommands(event.getDispatcher());
	}

	public static Host host() {
		return host;
	}
}
