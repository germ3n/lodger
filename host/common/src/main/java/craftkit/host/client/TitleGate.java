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
