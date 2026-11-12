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
	/** The screen's left and top, read once: the game's own fields, not shadowed through the hierarchy. */
	@Unique
	private static java.lang.reflect.Field craftkit$left;
	@Unique
	private static java.lang.reflect.Field craftkit$top;

	/** `phantom_item`: drawn over the slots once the screen is drawn. */
	@Inject(method = "extractRenderState", at = @At("TAIL"))
	private void craftkit$phantom(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		Glitches.inventoryOpen();
		var screen = (net.minecraft.client.gui.screens.inventory.AbstractContainerScreen<?>) (Object) this;
		try {
			if (craftkit$left == null) {
				craftkit$left = net.minecraft.client.gui.screens.inventory.AbstractContainerScreen.class.getDeclaredField("leftPos");
				craftkit$top = net.minecraft.client.gui.screens.inventory.AbstractContainerScreen.class.getDeclaredField("topPos");
				craftkit$left.setAccessible(true);
				craftkit$top.setAccessible(true);
			}
			Glitches.phantomItem(graphics, screen.getMenu(), craftkit$left.getInt(screen), craftkit$top.getInt(screen), mouseX, mouseY);
		} catch (ReflectiveOperationException ignored) {
			// The screen moved its fields: no paper, nothing else lost.
		}
	}

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
