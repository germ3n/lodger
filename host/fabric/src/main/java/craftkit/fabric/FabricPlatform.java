package craftkit.fabric;

import craftkit.host.Platform;
import net.fabricmc.loader.api.FabricLoader;
import net.fabricmc.loader.api.ModContainer;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

/** Fabric's answers to what the host asks: the config directory and every loaded mod's files. */
public final class FabricPlatform implements Platform {
	@Override
	public String name() {
		return "fabric";
	}

	@Override
	public Path gameDir() {
		return FabricLoader.getInstance().getGameDir();
	}

	@Override
	public boolean dedicatedServer() {
		return FabricLoader.getInstance().getEnvironmentType() == net.fabricmc.api.EnvType.SERVER;
	}

	@Override
	public void sendToPlayer(net.minecraft.server.level.ServerPlayer player, net.minecraft.network.protocol.common.custom.CustomPacketPayload payload) {
		net.fabricmc.fabric.api.networking.v1.ServerPlayNetworking.send(player, payload);
	}

	@Override
	public List<String[]> loadedMods() {
		// The mods installed in their own right: not Minecraft or Java themselves ("builtin"), nor a
		// module inside another's jar (Fabric API's fifty-odd made the list).
		return FabricLoader.getInstance().getAllMods().stream()
				.filter(mod -> mod.getContainingMod().isEmpty() && !"builtin".equals(mod.getMetadata().getType()))
				.map(mod -> new String[] {mod.getMetadata().getId(), mod.getMetadata().getVersion().getFriendlyString()})
				.toList();
	}

	@Override
	public Path configDir() {
		return FabricLoader.getInstance().getConfigDir();
	}

	@Override
	public List<ContentJar> jars() {
		return FabricLoader.getInstance().getAllMods().stream()
				.<ContentJar>map(FabricJar::new)
				.toList();
	}

	private record FabricJar(ModContainer container) implements ContentJar {
		@Override
		public String id() {
			return container.getMetadata().getId();
		}

		@Override
		public Optional<byte[]> read(String resource) throws IOException {
			Optional<Path> path = container.findPath(resource);
			if (path.isEmpty()) {
				return Optional.empty();
			}

			return Optional.of(Files.readAllBytes(path.get()));
		}
	}
}
