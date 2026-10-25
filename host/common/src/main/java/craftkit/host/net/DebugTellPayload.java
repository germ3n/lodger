package craftkit.host.net;

import craftkit.host.Host;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.ByteBufCodecs;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;

/**
 * The server's answer to a {@link DebugAskPayload}, or its order to open the screen (from
 * {@code /craftkit <mod> debug}): JSON with {@code host} (the senses' own panel), {@code mod}
 * (the mod's {@code DebugReply}), {@code mods} (the ids a client may ask about), {@code open}
 * or {@code error}. See {@code craftkit.host.client.debug.DebugModel} for the shape.
 */
public record DebugTellPayload(String mod, String json) implements CustomPacketPayload {
	public static final CustomPacketPayload.Type<DebugTellPayload> TYPE = new CustomPacketPayload.Type<>(Identifier.fromNamespaceAndPath(Host.MOD_ID, "debug_tell"));
	public static final StreamCodec<RegistryFriendlyByteBuf, DebugTellPayload> CODEC = StreamCodec.composite(
			ByteBufCodecs.STRING_UTF8, DebugTellPayload::mod,
			ByteBufCodecs.stringUtf8(1 << 20), DebugTellPayload::json,
			DebugTellPayload::new);

	@Override
	public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
		return TYPE;
	}
}
