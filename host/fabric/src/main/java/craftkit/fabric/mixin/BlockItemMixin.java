package craftkit.fabric.mixin;

import craftkit.fabric.CraftkitFabric;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.context.BlockPlaceContext;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Fabric has no event for a block placed, so the senses learn it here, at the end of
 * {@code BlockItem.place}, when the placement went through. NeoForge has an event and needs no
 * mixin.
 */
@Mixin(BlockItem.class)
public abstract class BlockItemMixin {
	@Inject(method = "place", at = @At("RETURN"))
	private void craftkit$placed(BlockPlaceContext context, CallbackInfoReturnable<InteractionResult> cir) {
		if (cir.getReturnValue().consumesAction() && context.getPlayer() instanceof ServerPlayer player && !context.getLevel().isClientSide()) {
			CraftkitFabric.placedBlock(player, context.getClickedPos(), context.getLevel().getBlockState(context.getClickedPos()));
		}
	}
}
