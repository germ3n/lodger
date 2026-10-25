package craftkit.host.net;

import craftkit.host.Host;
import net.minecraft.core.BlockPos;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.ByteBufCodecs;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;

import java.util.Optional;

/**
 * A client-side hint: a kind the client knows (`flicker`, `lights_fail`, `sun_blink`, ...), a
 * place if it has one, how long, and a detail string. The client renders it and then stops; the
 * server's world never changed. The lodger's glitches ({@code docs/entity.md}) are these.
 */
public record HintPayload(String hint, Optional<BlockPos> pos, float seconds, String detail) implements CustomPacketPayload {
	public static final CustomPacketPayload.Type<HintPayload> TYPE = new CustomPacketPayload.Type<>(Identifier.fromNamespaceAndPath(Host.MOD_ID, "hint"));
	public static final StreamCodec<RegistryFriendlyByteBuf, HintPayload> CODEC = StreamCodec.composite(
			ByteBufCodecs.STRING_UTF8, HintPayload::hint,
			ByteBufCodecs.optional(BlockPos.STREAM_CODEC), HintPayload::pos,
			ByteBufCodecs.FLOAT, HintPayload::seconds,
			ByteBufCodecs.STRING_UTF8, HintPayload::detail,
			HintPayload::new);

	@Override
	public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
		return TYPE;
	}
}
