package craftkit.host.mixin;

import craftkit.host.world.Returns;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.projectile.throwableitemprojectile.Snowball;
import net.minecraft.world.phys.HitResult;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** `throw_back`: where a snowball of theirs lands, noted, so one comes back at them ({@link Returns}). */
@Mixin(Snowball.class)
public abstract class SnowballMixin {
	@Inject(method = "onHit", at = @At("HEAD"))
	private void craftkit$back(HitResult hit, CallbackInfo ci) {
		Snowball ball = (Snowball) (Object) this;
		if (ball.level() instanceof ServerLevel level && ball.getOwner() instanceof net.minecraft.server.level.ServerPlayer owner) {
			Returns.landed(level, owner.getUUID(), Returns.Kind.SNOWBALL, hit.getLocation(), null);
		}
	}
}
