package craftkit.host.mixin;

import craftkit.host.world.Returns;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.projectile.arrow.AbstractArrow;
import net.minecraft.world.phys.BlockHitResult;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * `throw_back`: an arrow of theirs that sticks, one they could pick up, taken out of the world and
 * dropped at their feet a moment on ({@link Returns}): the very arrow, nothing had from nothing.
 */
@Mixin(AbstractArrow.class)
public abstract class AbstractArrowMixin {
	@Inject(method = "onHitBlock", at = @At("TAIL"))
	private void craftkit$back(BlockHitResult hit, CallbackInfo ci) {
		AbstractArrow arrow = (AbstractArrow) (Object) this;
		if (arrow.level() instanceof ServerLevel level && arrow.getOwner() instanceof net.minecraft.server.level.ServerPlayer owner
				&& arrow.pickup == AbstractArrow.Pickup.ALLOWED
				&& Returns.landed(level, owner.getUUID(), Returns.Kind.ARROW, hit.getLocation(), arrow.getPickupItemStackOrigin())) {
			arrow.discard();
		}
	}
}
