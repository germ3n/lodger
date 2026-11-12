package craftkit.host.world;

import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerBossEvent;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.BossEvent;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

/**
 * Boss bars shown to one player for a while (`boss_bar`, ABI 55): named as the mod asks, filling
 * from empty over their time, and gone at its end (`boss_bar_it`). Server-wide, ticked with the
 * host's other timers; the fill is {@link BarFill}'s.
 */
public final class BossBars {
	private record Shown(ServerBossEvent bar, ServerPlayer player, long from, long until) {
	}

	private static final List<Shown> SHOWN = new ArrayList<>();
	private static long tick;

	private BossBars() {
	}

	/** A bar named {@code name} shown to {@code player} for {@code seconds} (a minute and a half at most), filling. */
	public static void show(ServerPlayer player, String name, float seconds) {
		ServerBossEvent bar = new ServerBossEvent(UUID.randomUUID(), Component.literal(name), BossEvent.BossBarColor.WHITE, BossEvent.BossBarOverlay.PROGRESS);
		bar.setProgress(0);
		bar.addPlayer(player);
		long ticks = (long) (Math.max(1, Math.min(90, seconds)) * 20);
		SHOWN.add(new Shown(bar, player, tick, tick + ticks));
	}

	/** Once a server tick: each bar filled to where its time is, and gone once it is up. */
	public static void tick() {
		tick++;
		Iterator<Shown> it = SHOWN.iterator();
		while (it.hasNext()) {
			Shown shown = it.next();
			if (tick >= shown.until() || shown.player().isRemoved()) {
				shown.bar().removeAllPlayers();
				it.remove();
			} else {
				shown.bar().setProgress(BarFill.of(tick - shown.from(), shown.until() - shown.from()));
			}
		}
	}
}
