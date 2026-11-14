package craftkit.host.mixin;

import craftkit.host.senses.UsedItems;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.InstrumentItem;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** A goat horn blown, on the server: told to the senses as `used_item` (`horn_answered`, ABI 57). */
@Mixin(InstrumentItem.class)
public abstract class InstrumentItemMixin {
	@Inject(method = "use", at = @At("RETURN"))
	private void craftkit$blown(Level level, Player player, InteractionHand hand, CallbackInfoReturnable<InteractionResult> cir) {
		if (!level.isClientSide() && player instanceof ServerPlayer server && cir.getReturnValue().consumesAction()) {
			UsedItems.noticed(server, net.minecraft.core.registries.BuiltInRegistries.ITEM.getKey(player.getItemInHand(hand).getItem()).toString());
		}
	}
}
