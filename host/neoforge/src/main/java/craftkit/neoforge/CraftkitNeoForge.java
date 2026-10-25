package craftkit.neoforge;

import craftkit.host.Host;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.RegisterCommandsEvent;
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
