package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.sounds.SoundManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/** `sound_swap`: a sound the hint names played as another while it is on (`wrong_footing`'s steps). */
@Mixin(SoundManager.class)
public abstract class SoundManagerMixin {
	@ModifyVariable(method = "play(Lnet/minecraft/client/resources/sounds/SoundInstance;)Lnet/minecraft/client/sounds/SoundEngine$PlayResult;",
			at = @At("HEAD"), argsOnly = true)
	private SoundInstance craftkit$swap(SoundInstance sound) {
		return Glitches.swapSound(sound);
	}
}
