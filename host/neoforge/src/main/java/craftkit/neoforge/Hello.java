package craftkit.neoforge;

import craftkit.host.Host;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;

/**
 * An empty configuration-phase payload registered as required, so NeoForge's own handshake
 * turns away a client without the host.
 */
public record Hello() implements CustomPacketPayload {
	public static final CustomPacketPayload.Type<Hello> TYPE = new CustomPacketPayload.Type<>(Identifier.fromNamespaceAndPath(Host.MOD_ID, "hello"));
	public static final StreamCodec<FriendlyByteBuf, Hello> CODEC = StreamCodec.unit(new Hello());

	@Override
	public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
		return TYPE;
	}
}
