package craftkit.host.world;

import com.google.gson.JsonObject;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.Container;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.function.Function;

/**
 * Offerings a mod arms for a player's next fresh loot ({@code arm_loot}): when they open a
 * chest whose loot is being made, the host unpacks it as the game would and, by the offering's
 * chance, adds one item to it or swaps one out, before the chest is seen. A swapped-out item is
 * kept with the world under {@code loot_taken}, so nothing vanishes. One offering a player; a
 * new one replaces it. Not saved: a mod arms again after a restart.
 */
public final class LootOffers {
	private record Offer(String mod, JsonObject offer) {
	}

	private final Map<UUID, Offer> armed = new HashMap<>();

	public void arm(ServerPlayer player, String mod, JsonObject offer) {
		armed.put(player.getUUID(), new Offer(mod, offer));
	}

	/**
	 * A chest's fresh loot, just unpacked: the offering applied by its chance, if one is armed.
	 * Answers the item placed, "" for an offering that could not be given (and is spent), or
	 * null when nothing was armed or the chance said no.
	 */
	public String apply(ServerPlayer player, Container chest, Function<String, WorldData> dataOf) {
		Offer offer = armed.get(player.getUUID());
		if (offer == null) {
			return null;
		}
		JsonObject o = offer.offer();
		var random = player.level().getRandom();
		if (random.nextFloat() >= (o.has("chance") ? o.get("chance").getAsFloat() : 0)) {
			return null;
		}
		armed.remove(player.getUUID());

		WorldData data = dataOf.apply(offer.mod());
		ItemStack gift = gift(player, o, data);
		if (gift == null || gift.isEmpty()) {
			return "";
		}

		List<Integer> full = new ArrayList<>();
		List<Integer> empty = new ArrayList<>();
		for (int slot = 0; slot < chest.getContainerSize(); slot++) {
			(chest.getItem(slot).isEmpty() ? empty : full).add(slot);
		}
		boolean swap = !full.isEmpty() && (empty.isEmpty() || random.nextFloat() < (o.has("replace") ? o.get("replace").getAsFloat() : 0));
		int slot;
		// A swap only where what it swaps out can be kept (until 2026-11-10 a loot item could
		// be overwritten with nowhere to keep it).
		if (swap && data == null) {
			swap = false;
		}
		if (swap) {
			slot = full.get(random.nextInt(full.size()));
			// Each under its own key: under one, a second swap of another item replaced the first.
			String key = "loot_taken";
			ItemStack out = chest.getItem(slot);
			for (int n = 1; data.has(key) && !data.holdsLike(key, out); n++) {
				key = "loot_taken_" + n;
			}
			data.keep(key, out);
		} else if (!empty.isEmpty()) {
			slot = empty.get(random.nextInt(empty.size()));
		} else {
			return "";
		}
		chest.setItem(slot, gift);
		chest.setChanged();

		return BuiltInRegistries.ITEM.getKey(gift.getItem()).toString();
	}

	/** The item offered: the stack kept under its name, the player's own head, or one made. */
	private static ItemStack gift(ServerPlayer player, JsonObject o, WorldData data) {
		if (o.has("kept") && !o.get("kept").getAsString().isEmpty()) {
			return data == null ? null : data.giveBack(o.get("kept").getAsString());
		}
		JsonObject item = o.getAsJsonObject("item");
		String id = item.get("id").getAsString();
		if (id.equals("@head")) {
			ItemStack head = new ItemStack(Items.PLAYER_HEAD);
			head.set(net.minecraft.core.component.DataComponents.PROFILE, net.minecraft.world.item.component.ResolvableProfile.createResolved(player.getGameProfile()));
			return head;
		}
		Optional<Holder.Reference<Item>> kind = BuiltInRegistries.ITEM.get(Identifier.parse(id));

		return kind.map(k -> new ItemStack(k.value(), Math.max(1, item.get("count").getAsInt()))).orElse(null);
	}

	/** Everything for every player, forgotten: the world is closing, and none of it is the next one's. */
	public void clear() {
		armed.clear();
	}
}
