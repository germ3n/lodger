package craftkit.host.mixin;

import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.math.Axis;
import craftkit.host.client.Atmospheres;
import net.minecraft.client.renderer.SubmitNodeCollector;
import net.minecraft.client.renderer.entity.PaintingRenderer;
import net.minecraft.client.renderer.entity.state.PaintingRenderState;
import net.minecraft.client.renderer.state.level.CameraRenderState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** `painting_flip`: the painting turned half round about the wall's normal, so it hangs upside down. */
@Mixin(PaintingRenderer.class)
public abstract class PaintingRendererMixin {
	@Inject(method = "submit", at = @At("HEAD"))
	private void craftkit$flip(PaintingRenderState state, PoseStack pose, SubmitNodeCollector collector, CameraRenderState camera, CallbackInfo ci) {
		if (state.direction != null && Atmospheres.flipped(state.x, state.y, state.z)) {
			var normal = state.direction.step();
			pose.rotate(Axis.of(normal).rotationDegrees(180));
		}
	}
}
