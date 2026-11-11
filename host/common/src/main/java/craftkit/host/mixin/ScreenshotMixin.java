package craftkit.host.mixin;

import com.mojang.blaze3d.pipeline.RenderTarget;
import craftkit.host.client.ScreenshotFigure;
import net.minecraft.client.Screenshot;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.File;
import java.util.function.Consumer;

/** `in_the_screenshot`: the screenshot held back a tick for one with the figure in it. */
@Mixin(Screenshot.class)
public abstract class ScreenshotMixin {
	@Inject(method = "grab(Ljava/io/File;Lcom/mojang/blaze3d/pipeline/RenderTarget;Ljava/util/function/Consumer;)V", at = @At("HEAD"), cancellable = true)
	private static void craftkit$hold(File dir, RenderTarget target, Consumer<Component> message, CallbackInfo ci) {
		if (ScreenshotFigure.hold(dir, target, message)) {
			ci.cancel();
		}
	}
}
