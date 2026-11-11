package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.gui.components.debug.DebugEntryPosition;
import net.minecraft.client.gui.components.debug.DebugScreenDisplayer;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.chunk.LevelChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** `f3_line`: one more line among the debug screen's own, after where they stand. */
@Mixin(DebugEntryPosition.class)
public abstract class DebugEntryPositionMixin {
	@Inject(method = "display", at = @At("TAIL"))
	private void craftkit$watching(DebugScreenDisplayer displayer, Level level, LevelChunk client, LevelChunk server, CallbackInfo ci) {
		if (Glitches.f3Line()) {
			displayer.addLine("Lodger: watching");
		}
	}
}
