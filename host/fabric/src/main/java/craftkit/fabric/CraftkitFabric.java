package craftkit.fabric;

import craftkit.host.Host;
import net.fabricmc.api.ModInitializer;
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

	public static Host host() {
		return host;
	}
}
