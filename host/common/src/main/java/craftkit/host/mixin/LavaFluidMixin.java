package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.core.BlockPos;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.LavaFluid;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** `lava_still`: the lava round them gone still on their screen, no pop, no spark, no sound. Client only. */
@Mixin(LavaFluid.class)
public abstract class LavaFluidMixin {
	@Inject(method = "animateTick", at = @At("HEAD"), cancellable = true)
	private void craftkit$still(Level level, BlockPos pos, FluidState state, RandomSource random, CallbackInfo ci) {
		if (level.isClientSide() && Glitches.lavaStill()) {
			ci.cancel();
		}
	}
}
