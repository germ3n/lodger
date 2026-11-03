package craftkit.host.client;

/** Which subtitles a hint renames, apart from the game so JUnit checks it (`ghast_name`, batch 16). */
public final class Subtitles {
	private Subtitles() {
	}

	/** Whether the hint `hint` renames the subtitle of the sound at `soundPath` (`entity.ghast.ambient`). */
	public static boolean renames(String hint, String soundPath) {
		return hint.equals("ghast_name") && soundPath.startsWith("entity.ghast.");
	}
}
