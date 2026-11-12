package craftkit.host.mixin;

import craftkit.host.client.MenuFreaks;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.PauseScreen;
import net.minecraft.client.gui.screens.Screen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** The pause menu, each frame before it is drawn, for `menu_freaks` ({@link MenuFreaks}). */
@Mixin(PauseScreen.class)
public abstract class PauseScreenMixin {
	@Inject(method = "extractRenderState", at = @At("HEAD"))
	private void craftkit$freak(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		MenuFreaks.frame((Screen) (Object) this);
	}
}
