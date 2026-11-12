package craftkit.host.mixin;

import craftkit.host.client.Glitches;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.item.properties.numeric.Time;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.ItemOwner;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/** `clock_wrong`: a clock's day time half a day off while the hint is on, night at noon. */
@Mixin(Time.class)
public abstract class ClockTimeMixin {
	@Redirect(method = "calculate", at = @At(value = "INVOKE",
			target = "Lnet/minecraft/client/renderer/item/properties/numeric/Time$TimeSource;get(Lnet/minecraft/client/multiplayer/ClientLevel;Lnet/minecraft/world/item/ItemStack;Lnet/minecraft/world/entity/ItemOwner;Lnet/minecraft/util/RandomSource;)F"))
	private float craftkit$clock(Time.TimeSource source, ClientLevel level, ItemStack stack, ItemOwner owner, RandomSource random) {
		float time = source.get(level, stack, owner, random);
		return source == Time.TimeSource.DAYTIME && Glitches.clockWrong() ? (time + 0.5f) % 1f : time;
	}
}
