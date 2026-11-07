package craftkit.host.mixin;

import craftkit.host.client.Atmospheres;
import net.minecraft.client.renderer.WeatherEffectRenderer;
import net.minecraft.util.ARGB;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/** Red rain: each drop's colour, white in the game, red while `red_rain` is on. */
@Mixin(WeatherEffectRenderer.class)
public abstract class WeatherEffectRendererMixin {
	@Redirect(method = "prepareInstances", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/ARGB;white(F)I"))
	private int craftkit$redRain(float alpha, com.mojang.blaze3d.vertex.VertexConsumer builder, java.util.List<?> columns,
			net.minecraft.world.phys.Vec3 cameraPos, float maxAlpha, int radius, float intensity) {
		// The rain's call is made with a full alpha, the snow's with 0.8: the snow stays white
		// (until now it turned red too).
		return maxAlpha >= 0.9f ? Atmospheres.rain(alpha) : net.minecraft.util.ARGB.white(alpha);
	}
}
