package craftkit.neoforge;

import com.mojang.blaze3d.platform.InputConstants;
import craftkit.host.Host;
import craftkit.host.LoadedMod;
import craftkit.host.client.Glitches;
import craftkit.host.client.debug.DebugHud;
import craftkit.host.client.debug.DebugScreen;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.resources.Identifier;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.client.event.RegisterGuiLayersEvent;
import net.neoforged.neoforge.client.event.RegisterKeyMappingsEvent;
import net.neoforged.neoforge.client.gui.VanillaGuiLayers;
import net.neoforged.neoforge.common.NeoForge;

/**
 * The client half on NeoForge: the debug key and screen, the glitch renderer over the HUD. Its
 * own class so the server never loads client classes; {@link CraftkitNeoForge} installs it on
 * the client dist alone.
 */
final class CraftkitNeoForgeClient {
	private static KeyMapping debugKey;
	private static KeyMapping hudKey;

	private CraftkitNeoForgeClient() {
	}

	static void install(IEventBus modBus) {
		DebugScreen.sendWith(ask -> {
			if (Minecraft.getInstance().getConnection() != null) {
				Minecraft.getInstance().getConnection().send(new net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket(ask));
			}
		});
		modBus.addListener((RegisterKeyMappingsEvent event) -> {
			debugKey = new KeyMapping("key.craftkit.debug", InputConstants.Type.KEYBOARD, InputConstants.KEY_F8, KeyMapping.Category.MISC);
			hudKey = new KeyMapping("key.craftkit.hud", InputConstants.Type.KEYBOARD, InputConstants.KEY_F9, KeyMapping.Category.MISC);
			event.register(debugKey);
			event.register(hudKey);
		});
		modBus.addListener((RegisterGuiLayersEvent event) -> {
			event.registerAboveAll(Identifier.fromNamespaceAndPath(Host.MOD_ID, "glitches"), (graphics, tickCounter) -> {
				Glitches.render(graphics, graphics.guiWidth(), graphics.guiHeight());
				DebugHud.render(graphics, Minecraft.getInstance().font, graphics.guiWidth(), graphics.guiHeight());
			});
			event.wrapLayer(VanillaGuiLayers.CROSSHAIR, crosshair -> (graphics, tickCounter) -> {
				if (!Glitches.hideCrosshair()) {
					crosshair.render(graphics, tickCounter);
				}
			});
		});
		NeoForge.EVENT_BUS.addListener((ClientTickEvent.Post event) -> {
			Glitches.tick(Minecraft.getInstance());
			DebugHud.tick(Minecraft.getInstance());
			while (debugKey != null && debugKey.consumeClick()) {
				DebugScreen.open(firstMod());
			}
			while (hudKey != null && hudKey.consumeClick()) {
				DebugHud.toggle(firstMod());
			}
		});
	}

	private static String firstMod() {
		Host host = CraftkitNeoForge.host();
		if (host == null) {
			return "";
		}

		return host.mods().stream().filter(LoadedMod::loaded).map(LoadedMod::id).findFirst().orElse("");
	}
}
