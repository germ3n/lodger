package craftkit.host.mixin;

import craftkit.host.world.DeadRedstone;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** No power from it while its redstone is held dead (`dead_redstone`, {@link DeadRedstone}): it still presses, clicks and flips. */
@Mixin(net.minecraft.world.level.block.ButtonBlock.class)
public abstract class ButtonSignalMixin {
	@Inject(method = "ownSignal", at = @At("HEAD"), cancellable = true)
	private void craftkit$dead(BlockState state, BlockGetter level, BlockPos pos, CallbackInfoReturnable<Integer> cir) {
		if (DeadRedstone.dead(pos)) {
			cir.setReturnValue(0);
		}
	}

	@Inject(method = "getDirectSignal", at = @At("HEAD"), cancellable = true)
	private void craftkit$deadDirect(BlockState state, BlockGetter level, BlockPos pos, Direction direction, CallbackInfoReturnable<Integer> cir) {
		if (DeadRedstone.dead(pos)) {
			cir.setReturnValue(0);
		}
	}
}
