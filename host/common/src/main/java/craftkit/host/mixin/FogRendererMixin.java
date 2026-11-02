package craftkit.host.mixin;

import craftkit.host.client.Atmospheres;
import net.minecraft.client.Camera;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.fog.FogData;
import net.minecraft.client.renderer.fog.FogRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** The atmospheres' real fog: the game's fog, as it leaves setupFog, before it is uploaded. */
@Mixin(FogRenderer.class)
public abstract class FogRendererMixin {
	@Inject(method = "setupFog", at = @At("RETURN"))
	private void craftkit$atmospheres(Camera camera, int renderDistance, DeltaTracker delta, float darkening, ClientLevel level, CallbackInfoReturnable<FogData> cir) {
		Atmospheres.fog(cir.getReturnValue());
	}
}
