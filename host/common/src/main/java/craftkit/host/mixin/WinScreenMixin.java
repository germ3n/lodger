package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.WinScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * `the_poem_knows`: when armed, the end poem's last lines followed by three of its own, in the
 * poem's two voices, about them by name and their house. Once.
 */
@Mixin(WinScreen.class)
public abstract class WinScreenMixin {
	@Shadow
	private void addPoemLines(String line) {
		throw new AssertionError();
	}

	@Shadow
	private void addEmptyLine() {
		throw new AssertionError();
	}

	@Inject(method = "addPoemFile", at = @At("TAIL"))
	private void craftkit$knows(java.io.Reader reader, CallbackInfo ci) {
		if (!Glitches.takePoem()) {
			return;
		}
		String name = Minecraft.getInstance().getUser().getName();
		addEmptyLine();
		addPoemLines("§3and there was another in the dream, one the player did not dream.");
		addPoemLines("§2it learned the player's name. " + name + ".");
		addPoemLines("§3it is in the house now, by the bed, waiting for the player to wake.");
	}
}
