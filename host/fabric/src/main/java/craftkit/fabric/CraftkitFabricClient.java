package craftkit.fabric;

import craftkit.host.client.TitleGate;
import net.fabricmc.api.ClientModInitializer;
import craftkit.host.client.Hints;
import craftkit.host.net.HintPayload;
import net.fabricmc.fabric.api.client.networking.v1.ClientConfigurationNetworking;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;

/** The client half on Fabric: register the hello channel so a server with the host lets this client in. */
public final class CraftkitFabricClient implements ClientModInitializer {
	@Override
	public void onInitializeClient() {
		ClientConfigurationNetworking.registerGlobalReceiver(Hello.TYPE, (payload, context) -> {
			// Being registered is the whole message.
		});
		ScreenEvents.AFTER_INIT.register((client, screen, width, height) -> TitleGate.onScreen(client, screen));
		ClientPlayNetworking.registerGlobalReceiver(HintPayload.TYPE, (payload, context) -> Hints.show(payload));
	}
}
