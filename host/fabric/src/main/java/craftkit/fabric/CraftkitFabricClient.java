package craftkit.fabric;

import com.mojang.blaze3d.platform.InputConstants;
import craftkit.host.Host;
import craftkit.host.client.Glitches;
import craftkit.host.client.Hints;
import craftkit.host.client.TitleGate;
import craftkit.host.client.debug.DebugHud;
import craftkit.host.client.debug.DebugScreen;
import craftkit.host.net.DebugAskPayload;
import craftkit.host.net.DebugTellPayload;
import craftkit.host.net.HintPayload;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.fabric.api.client.networking.v1.ClientConfigurationNetworking;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.fabric.api.client.rendering.v1.hud.VanillaHudElements;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.minecraft.client.KeyMapping;
import net.minecraft.resources.Identifier;

/**
 * The client half on Fabric: the hello channel so a server with the host lets this client in,
 * the title gate, the hint and debug channels, the glitch renderer over the HUD, and the debug
 * key (F8 unless rebound).
 */
public final class CraftkitFabricClient implements ClientModInitializer {
	private static KeyMapping debugKey;
	private static KeyMapping hudKey;

	@Override
	public void onInitializeClient() {
		ClientConfigurationNetworking.registerGlobalReceiver(Hello.TYPE, (payload, context) -> {
			// Being registered is the whole message.
		});
		ScreenEvents.AFTER_INIT.register((client, screen, width, height) -> {
			net.fabricmc.fabric.api.client.screen.v1.Screens.getWidgets(screen).addAll(TitleGate.settingsButtons(screen));
			TitleGate.onScreen(client, screen);
		});
		ClientPlayNetworking.registerGlobalReceiver(HintPayload.TYPE, (payload, context) -> Hints.show(payload));
		ClientPlayNetworking.registerGlobalReceiver(DebugTellPayload.TYPE, (payload, context) -> DebugScreen.receive(payload));
		DebugScreen.sendWith(ClientPlayNetworking::send);

		debugKey = KeyMappingHelper.registerKeyMapping(new KeyMapping("key.craftkit.debug", InputConstants.Type.KEYBOARD, InputConstants.KEY_F8, KeyMapping.Category.MISC));
		hudKey = KeyMappingHelper.registerKeyMapping(new KeyMapping("key.craftkit.hud", InputConstants.Type.KEYBOARD, InputConstants.KEY_F9, KeyMapping.Category.MISC));
		ClientTickEvents.END_CLIENT_TICK.register(client -> {
			Glitches.tick(client);
			DebugHud.tick(client);
			while (debugKey.consumeClick()) {
				DebugScreen.open(firstMod());
			}
			while (hudKey.consumeClick()) {
				DebugHud.toggle(firstMod());
			}
		});

		HudElementRegistry.addLast(Identifier.fromNamespaceAndPath(Host.MOD_ID, "glitches"), (graphics, tickCounter) -> {
			Glitches.render(graphics, graphics.guiWidth(), graphics.guiHeight());
			DebugHud.render(graphics, net.minecraft.client.Minecraft.getInstance().font, graphics.guiWidth(), graphics.guiHeight());
		});
		HudElementRegistry.replaceElement(VanillaHudElements.CROSSHAIR, crosshair -> (graphics, tickCounter) -> {
			if (!Glitches.hideCrosshair()) {
				crosshair.extractRenderState(graphics, tickCounter);
			}
		});
	}

	/** The first loaded Rust mod's id, for the key; the screen's picker switches among several. */
	static String firstMod() {
		Host host = CraftkitFabric.host();
		if (host == null) {
			return "";
		}

		return host.mods().stream().filter(craftkit.host.LoadedMod::loaded).map(craftkit.host.LoadedMod::id).findFirst().orElse("");
	}
}
