package craftkit.host.senses;

import net.minecraft.server.level.ServerPlayer;

import java.util.ArrayList;
import java.util.List;

/**
 * Items used that the senses report (`used_item`, ABI 57): a goat horn blown. Noticed by a hook
 * on the item where it is used, queued, and reported on the next world tick, where the senses
 * are at hand (`horn_answered`).
 */
public final class UsedItems {
	private record Used(ServerPlayer player, String item) {
	}

	private static final List<Used> QUEUED = new ArrayList<>();

	private UsedItems() {
	}

	public static void noticed(ServerPlayer player, String item) {
		synchronized (QUEUED) {
			if (QUEUED.size() < 64) {
				QUEUED.add(new Used(player, item));
			}
		}
	}

	/** Every use queued since the last tick, reported. */
	public static void report(Senses senses) {
		List<Used> taken;
		synchronized (QUEUED) {
			taken = new ArrayList<>(QUEUED);
			QUEUED.clear();
		}
		for (Used used : taken) {
			if (!used.player().isRemoved()) {
				senses.noticedUsedItem(used.player(), used.item());
			}
		}
	}
}
