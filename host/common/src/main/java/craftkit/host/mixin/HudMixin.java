package craftkit.host.mixin;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import craftkit.host.client.Glitches;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.Hud;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

/** The hotbar drawn wrong a moment, on their screen alone: every stack's count a 0 (`count_shows_zero`). */
@Mixin(Hud.class)
public abstract class HudMixin {
	@WrapOperation(method = "extractSlot", at = @At(value = "INVOKE",
			target = "Lnet/minecraft/client/gui/GuiGraphicsExtractor;itemDecorations(Lnet/minecraft/client/gui/Font;Lnet/minecraft/world/item/ItemStack;II)V"))
	private void craftkit$countZero(GuiGraphicsExtractor graphics, Font font, ItemStack stack, int x, int y, Operation<Void> original) {
		if (Glitches.countsZero() && !stack.isEmpty()) {
			graphics.itemDecorations(font, stack, x, y, "0");
			return;
		}
		original.call(graphics, font, stack, x, y);
	}
}
