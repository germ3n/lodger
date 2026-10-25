package craftkit.neoforge;

import craftkit.host.Platform;
import net.neoforged.fml.ModList;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforgespi.language.IModInfo;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

/** NeoForge's answers to what the host asks: the config directory and every loaded mod's files. */
public final class NeoForgePlatform implements Platform {
	@Override
	public String name() {
		return "neoforge";
	}

	@Override
	public Path gameDir() {
		return FMLPaths.GAMEDIR.get();
	}

	@Override
	public boolean dedicatedServer() {
		return net.neoforged.fml.loading.FMLEnvironment.getDist().isDedicatedServer();
	}

	@Override
	public Path configDir() {
		return FMLPaths.CONFIGDIR.get();
	}

	@Override
	public List<ContentJar> jars() {
		return ModList.get().getMods().stream()
				.<ContentJar>map(NeoForgeJar::new)
				.toList();
	}

	private record NeoForgeJar(IModInfo info) implements ContentJar {
		@Override
		public String id() {
			return info.getModId();
		}

		@Override
		public Optional<byte[]> read(String resource) throws IOException {
			var contents = info.getOwningFile().getFile().getContents();
			if (!contents.containsFile(resource)) {
				return Optional.empty();
			}

			return Optional.of(contents.readFile(resource));
		}
	}
}
