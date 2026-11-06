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

	/** The mods that failed to load were shown once this launch. */
	private static boolean failuresShown;

	public static void onScreen(Minecraft minecraft, Screen screen) {
		if (!(screen instanceof TitleScreen)) {
			return;
		}

		// A mod that could not load says why, once, before anything else (owner, 2026-11-10:
		// "else it should show an error message on boot").
		if (!failuresShown) {
			java.util.List<craftkit.host.LoadedMod> failed = Preparation.current()
					.map(p -> p.mods().stream().filter(m -> !m.loaded()).toList())
					.orElse(java.util.List.of());
			if (!failed.isEmpty()) {
				failuresShown = true;
				minecraft.gui.setScreen(new FailedScreen(screen, failed));
				return;
			}
		}

		Preparation.current().ifPresent(preparation -> {
			if (preparation.needed()) {
				minecraft.gui.setScreen(new PreparingScreen(screen, preparation));
			}
		});
	}
}
