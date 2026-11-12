package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.Minecraft;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** `false_glint`: the tool in their hand drawn shimmering as if enchanted; it is not. Client only. */
@Mixin(ItemStack.class)
public abstract class ItemStackFoilMixin {
	@Inject(method = "hasFoil", at = @At("HEAD"), cancellable = true)
	private void craftkit$glint(CallbackInfoReturnable<Boolean> cir) {
		var player = Minecraft.getInstance().player;
		if (player != null && (Object) this == player.getMainHandItem() && Glitches.falseGlint()) {
			cir.setReturnValue(true);
		}
	}
}
