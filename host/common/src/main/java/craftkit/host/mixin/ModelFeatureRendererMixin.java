package craftkit.host.mixin;

import craftkit.host.client.WarpedModels;
import net.minecraft.client.model.Model;
import net.minecraft.client.renderer.feature.ModelFeatureRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/** The fight's warped animals: their models bent as they are posed for drawing ({@link WarpedModels}). */
@Mixin(ModelFeatureRenderer.class)
public abstract class ModelFeatureRendererMixin {
	@Redirect(method = "prepareModel", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/model/Model;setupAnim(Ljava/lang/Object;)V"))
	private void craftkit$warp(Model<Object> model, Object state) {
		model.setupAnim(state);
		WarpedModels.warp(model, state);
	}
}
