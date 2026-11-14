package craftkit.host.mixin;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.mojang.blaze3d.vertex.VertexConsumer;
import craftkit.host.client.Atmospheres;
import net.minecraft.client.renderer.WeatherEffectRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Red rain: each drop's colour as it is set, red while `red_rain` is on. A wrap of the colour
 * call itself, at a priority over the rest, so it is the last word: Subtle Effects' biome-coloured
 * rain sets the drops' colour there too, and it left them water-blue under red rain (owner: "without
 * a shader the rain drops are still blue, even when triggering red_rain"). Otherwise the call goes
 * on as it was, theirs included.
 */
@Mixin(value = WeatherEffectRenderer.class, priority = 1500)
public abstract class WeatherEffectRendererMixin {
	@WrapOperation(method = "prepareInstances", at = @At(value = "INVOKE",
			target = "Lcom/mojang/blaze3d/vertex/VertexConsumer;setColor(I)Lcom/mojang/blaze3d/vertex/VertexConsumer;"))
	private VertexConsumer craftkit$redRain(VertexConsumer vertex, int colour, Operation<VertexConsumer> original,
			VertexConsumer builder, java.util.List<?> columns, net.minecraft.world.phys.Vec3 cameraPos, float maxAlpha, int radius, float intensity) {
		// The rain's columns are drawn with a full alpha, the snow's with 0.8: the snow stays white.
		if (maxAlpha >= 0.9f && Atmospheres.redRainOn()) {
			return vertex.setColor(Atmospheres.rain(((colour >>> 24) & 0xFF) / 255f));
		}
		return original.call(vertex, colour);
	}

	/** The columns of rain and snow as gathered, changed for `dry_beside_you` and `snow_hangs`. */
	@Inject(method = "extractRenderState", at = @At("TAIL"))
	private void craftkit$columns(net.minecraft.client.multiplayer.ClientLevel level, float partialTick, net.minecraft.world.phys.Vec3 camera,
			net.minecraft.client.renderer.state.level.WeatherRenderState state, CallbackInfo info) {
		Atmospheres.weather(state);
	}
}
