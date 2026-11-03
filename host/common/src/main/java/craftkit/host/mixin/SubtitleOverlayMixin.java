package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.gui.components.SubtitleOverlay;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.sounds.WeighedSoundEvents;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/** `ghast_name`: a sound's subtitle, another line while a hint says so (the ghasts' cries their name). */
@Mixin(SubtitleOverlay.class)
public abstract class SubtitleOverlayMixin {
	@Redirect(method = "onPlaySound", at = @At(value = "INVOKE",
			target = "Lnet/minecraft/client/sounds/WeighedSoundEvents;getSubtitle()Lnet/minecraft/network/chat/Component;"))
	private Component craftkit$subtitle(WeighedSoundEvents events, SoundInstance sound, WeighedSoundEvents same, float range) {
		return Glitches.subtitleOf(sound.getIdentifier().getPath(), events.getSubtitle());
	}
}
