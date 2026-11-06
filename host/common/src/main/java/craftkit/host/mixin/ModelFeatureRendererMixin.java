package craftkit.host.mixin;

import craftkit.host.client.WarpedModels;
import net.minecraft.client.renderer.feature.ModelFeatureRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * The fight's warped animals: their models bent as they are posed for drawing ({@link
 * WarpedModels}). Injected after the pose rather than redirecting it, so another mod's hook on
 * the same call does not clash.
 */
@Mixin(ModelFeatureRenderer.class)
public abstract class ModelFeatureRendererMixin {
	@Inject(method = "prepareModel", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/model/Model;setupAnim(Ljava/lang/Object;)V", shift = At.Shift.AFTER))
	private void craftkit$warp(ModelFeatureRenderer.Submit<?> submit, CallbackInfo ci) {
		WarpedModels.warp(submit.model(), submit.state());
	}
}
