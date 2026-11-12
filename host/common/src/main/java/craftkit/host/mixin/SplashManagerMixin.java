package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.gui.components.SplashRenderer;
import net.minecraft.client.resources.SplashManager;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** `title_splash`: the title screen's next yellow splash a line about them, once, when armed. */
@Mixin(SplashManager.class)
public abstract class SplashManagerMixin {
	@Inject(method = "getSplash", at = @At("HEAD"), cancellable = true)
	private void craftkit$splash(CallbackInfoReturnable<SplashRenderer> cir) {
		String line = Glitches.takeSplash();
		if (line != null) {
			cir.setReturnValue(new SplashRenderer(Component.literal(line)));
		}
	}
}
