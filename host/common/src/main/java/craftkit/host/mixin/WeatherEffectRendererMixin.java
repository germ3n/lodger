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
	private int craftkit$redRain(float alpha) {
		return Atmospheres.rain(alpha);
	}
}
