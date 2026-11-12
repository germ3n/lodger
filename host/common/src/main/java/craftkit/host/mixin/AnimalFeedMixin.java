package craftkit.host.mixin;

import craftkit.host.world.Refusals;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.animal.Animal;
import net.minecraft.world.entity.player.Player;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** `refuse_food`: the one feeding refused, the animal turning its head away, nothing eaten. */
@Mixin(Animal.class)
public abstract class AnimalFeedMixin {
	@Inject(method = "mobInteract", at = @At("HEAD"), cancellable = true)
	private void craftkit$refuse(Player player, InteractionHand hand, CallbackInfoReturnable<InteractionResult> cir) {
		Animal animal = (Animal) (Object) this;
		if (animal.level().isClientSide() || !animal.isFood(player.getItemInHand(hand))
				|| !Refusals.take(player.getUUID(), animal.level().getGameTime())) {
			return;
		}
		// Its head turned from them, a quarter round, and the food still in their hand.
		float away = player.getYRot() + 90f + (animal.getRandom().nextBoolean() ? 0 : 180f);
		animal.setYHeadRot(away);
		animal.setYBodyRot(away);
		animal.setYRot(away);
		cir.setReturnValue(InteractionResult.FAIL);
	}
}
