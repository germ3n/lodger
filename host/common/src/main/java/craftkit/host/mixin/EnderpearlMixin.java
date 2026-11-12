package craftkit.host.mixin;

import craftkit.host.world.Returns;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownEnderpearl;
import net.minecraft.world.phys.HitResult;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** `throw_back`'s `pearl`: a pearl of theirs landed, noted, so they are turned round where it sets them ({@link Returns}). */
@Mixin(ThrownEnderpearl.class)
public abstract class EnderpearlMixin {
	@Inject(method = "onHit", at = @At("HEAD"))
	private void craftkit$turn(HitResult hit, CallbackInfo ci) {
		ThrownEnderpearl pearl = (ThrownEnderpearl) (Object) this;
		if (pearl.level() instanceof ServerLevel level && pearl.getOwner() instanceof net.minecraft.server.level.ServerPlayer owner) {
			Returns.landed(level, owner.getUUID(), Returns.Kind.PEARL, hit.getLocation(), null);
		}
	}
}
