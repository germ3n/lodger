package craftkit.fabric;

import craftkit.host.Host;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;

/**
 * An empty configuration-phase payload whose only job is to be a channel the client registers,
 * so the server can tell a client with the host from one without and send the latter away:
 * Rust mods' client-side effects need the host on both sides.
 */
public record Hello() implements CustomPacketPayload {
	public static final CustomPacketPayload.Type<Hello> TYPE = new CustomPacketPayload.Type<>(Identifier.fromNamespaceAndPath(Host.MOD_ID, "hello"));
	public static final StreamCodec<FriendlyByteBuf, Hello> CODEC = StreamCodec.unit(new Hello());

	@Override
	public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
		return TYPE;
	}
}
