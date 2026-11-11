package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.inventory.InventoryScreen;
import net.minecraft.world.entity.LivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * `inventory_stare`: their own figure in the inventory drawn as if the mouse were at its face,
 * so it stares straight out at them instead of following the mouse.
 */
@Mixin(InventoryScreen.class)
public abstract class InventoryScreenMixin {
	/** Drawing it again here: the second draw is the game's own. */
	@Unique
	private static boolean craftkit$staring;

	@Inject(method = "extractEntityInInventoryFollowsMouse", at = @At("HEAD"), cancellable = true)
	private static void craftkit$stare(GuiGraphicsExtractor graphics, int x0, int y0, int x1, int y1, int size, float yOffset,
			float mouseX, float mouseY, LivingEntity entity, CallbackInfo ci) {
		if (craftkit$staring || entity != Minecraft.getInstance().player || !Glitches.inventoryStare()) {
			return;
		}
		craftkit$staring = true;
		try {
			InventoryScreen.extractEntityInInventoryFollowsMouse(graphics, x0, y0, x1, y1, size, yOffset, (x0 + x1) / 2f, (y0 + y1) / 2f, entity);
		} finally {
			craftkit$staring = false;
		}
		ci.cancel();
	}
}
