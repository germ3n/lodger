package craftkit.host.mixin;

import craftkit.host.client.Atmospheres;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.item.properties.numeric.CompassAngleState;
import net.minecraft.core.GlobalPos;
import net.minecraft.world.entity.ItemOwner;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/** `compass_points`: a compass's target, its place while the hint is on; `recovery_compass_lies`: a recovery compass's. */
@Mixin(CompassAngleState.class)
public abstract class CompassAngleStateMixin {
	@Redirect(method = "calculate", at = @At(value = "INVOKE",
			target = "Lnet/minecraft/client/renderer/item/properties/numeric/CompassAngleState$CompassTarget;get(Lnet/minecraft/client/multiplayer/ClientLevel;Lnet/minecraft/world/item/ItemStack;Lnet/minecraft/world/entity/ItemOwner;)Lnet/minecraft/core/GlobalPos;"))
	private GlobalPos craftkit$compass(CompassAngleState.CompassTarget target, ClientLevel level, ItemStack stack, ItemOwner owner) {
		return Atmospheres.compass(target == CompassAngleState.CompassTarget.RECOVERY, target.get(level, stack, owner));
	}
}
