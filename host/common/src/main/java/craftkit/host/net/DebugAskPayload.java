package craftkit.host.net;

import craftkit.host.Host;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.ByteBufCodecs;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;

/**
 * The debug screen asking the server about one mod: a {@code DebugRequest} as JSON
 * ({@code sdk/src/debug.rs}). The server checks the player may ask (an operator, or the owner of
 * a singleplayer world), passes it to the mod's world the player is in, and answers with a
 * {@link DebugTellPayload}. An empty {@code mod} asks for the list of mods instead.
 */
public record DebugAskPayload(String mod, String json) implements CustomPacketPayload {
	public static final CustomPacketPayload.Type<DebugAskPayload> TYPE = new CustomPacketPayload.Type<>(Identifier.fromNamespaceAndPath(Host.MOD_ID, "debug_ask"));
	public static final StreamCodec<RegistryFriendlyByteBuf, DebugAskPayload> CODEC = StreamCodec.composite(
			ByteBufCodecs.STRING_UTF8, DebugAskPayload::mod,
			ByteBufCodecs.stringUtf8(65536), DebugAskPayload::json,
			DebugAskPayload::new);

	@Override
	public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
		return TYPE;
	}
}
