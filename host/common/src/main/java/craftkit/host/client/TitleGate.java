package craftkit.host.client;

import craftkit.host.Preparation;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;

/**
 * Puts the {@link PreparingScreen} in front of the title screen while anything is left to fetch
 * or load. Each loader calls {@link #onScreen} from its screen-init event; the check is cheap and
 * the screen is only ever opened over a title screen, so a player already in a world is never
 * interrupted.
 */
public final class TitleGate {
	private TitleGate() {
	}

	/**
	 * The title screen's settings buttons, one for each mod that offers settings, top left
	 * (owner, 2026-11-09: a title screen button). The loader adds them to the screen.
	 */
	public static java.util.List<net.minecraft.client.gui.components.Button> settingsButtons(Screen screen) {
		java.util.List<net.minecraft.client.gui.components.Button> buttons = new java.util.ArrayList<>();
		if (!(screen instanceof TitleScreen)) {
			return buttons;
		}
		Preparation.current().ifPresent(preparation -> {
			int y = 4;
			for (craftkit.host.LoadedMod mod : preparation.mods()) {
				if (!mod.loaded() || mod.settings().isEmpty()) {
					continue;
				}
				buttons.add(net.minecraft.client.gui.components.Button.builder(
								net.minecraft.network.chat.Component.translatable("craftkit.settings.button", mod.name()),
								b -> Minecraft.getInstance().gui.setScreen(new SettingsScreen(screen, mod)))
						.bounds(4, y, 110, 20).build());
				y += 24;
			}
		});

		return buttons;
	}

	public static void onScreen(Minecraft minecraft, Screen screen) {
		if (!(screen instanceof TitleScreen)) {
			return;
		}

		Preparation.current().ifPresent(preparation -> {
			if (preparation.needed()) {
				minecraft.gui.setScreen(new PreparingScreen(screen, preparation));
			}
		});
	}
}
