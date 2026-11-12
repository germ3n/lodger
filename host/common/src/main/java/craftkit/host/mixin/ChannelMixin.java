package craftkit.host.mixin;

import com.mojang.blaze3d.audio.Channel;
import craftkit.host.client.SoundWarps;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Every sound as it starts, through the `sound_warp` hint's way ({@link SoundWarps}): its pitch, and its source's effects. */
@Mixin(Channel.class)
public abstract class ChannelMixin {
	@Shadow
	@Final
	private int source;

	@Inject(method = "play", at = @At("HEAD"))
	private void craftkit$warp(CallbackInfo ci) {
		SoundWarps.apply(source);
	}

	@ModifyVariable(method = "setPitch", at = @At("HEAD"), argsOnly = true)
	private float craftkit$pitch(float pitch) {
		return SoundWarps.pitch(pitch);
	}
}
