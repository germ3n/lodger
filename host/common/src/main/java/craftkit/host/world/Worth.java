package craftkit.host.world;

/**
 * What a stack is worth to its player, for `take_up` (ABI 54), apart from the game so JUnit
 * checks it: by its item's id and whether it is enchanted, the rare and hard-won first.
 * Below {@link #LEAST} it is not worth taking (a stack of dirt).
 */
public final class Worth {
	public static final int LEAST = 10;

	private Worth() {
	}

	/** The worth of an item by its id ({@code minecraft:diamond_sword}), more for one enchanted. */
	public static int of(String id, boolean enchanted) {
		String name = id.substring(id.indexOf(':') + 1);
		int worth = base(name);

		return worth + (enchanted ? 15 : 0);
	}

	private static int base(String name) {
		String[][] tiers = {
			{"100", "netherite"},
			{"98", "elytra"},
			{"96", "totem_of_undying"},
			{"95", "nether_star", "beacon", "dragon_egg"},
			{"92", "enchanted_golden_apple", "heavy_core", "mace"},
			{"90", "trident", "ancient_debris"},
			{"88", "shulker_box"},
			{"80", "diamond"},
			{"70", "echo_shard", "heart_of_the_sea", "conduit"},
			{"65", "enchanted_book"},
			{"60", "emerald"},
			{"58", "golden_apple"},
			{"45", "music_disc", "trial_key"},
			{"40", "gold", "golden"},
			{"35", "ender_pearl", "ender_eye", "blaze_rod"},
			{"30", "iron", "experience_bottle"},
			{"28", "name_tag", "saddle", "spyglass", "compass", "clock"},
			{"20", "lapis", "amethyst", "quartz"},
			{"15", "redstone", "copper", "bow", "crossbow"},
		};
		for (String[] tier : tiers) {
			for (int i = 1; i < tier.length; i++) {
				if (name.contains(tier[i])) {
					return Integer.parseInt(tier[0]);
				}
			}
		}

		return 1;
	}
}
