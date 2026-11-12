package craftkit.host.mixin;

import craftkit.host.world.LateDrops;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Block;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** `late_drop`: what a broken block drops held back a moment ({@link LateDrops}), then let fall. */
@Mixin(Block.class)
public abstract class PopResourceMixin {
	@Inject(method = "popResource(Lnet/minecraft/world/level/Level;Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/item/ItemStack;)V", at = @At("HEAD"), cancellable = true)
	private static void craftkit$late(Level level, BlockPos pos, ItemStack stack, CallbackInfo ci) {
		if (level instanceof ServerLevel server && !stack.isEmpty() && LateDrops.hold(server, pos, stack)) {
			ci.cancel();
		}
	}
}
