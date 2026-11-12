package craftkit.host.client;

import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

import java.util.HashSet;
import java.util.IdentityHashMap;
import java.util.Map;
import java.util.Set;

/**
 * The pause menu going wrong while a `menu_freaks` hint is armed ({@link MenuFreak}): the first
 * time it opens, every button shakes in place and settles, the first one's words read wrong a
 * moment; then all as they were, and that hint spent.
 */
public final class MenuFreaks {
	private MenuFreaks() {
	}

	private static final Set<Long> SPENT = new HashSet<>();
	private static Screen screen;
	private static long since;
	private static long hint;
	private static final Map<AbstractWidget, int[]> AT = new IdentityHashMap<>();
	private static Component firstWords;
	private static AbstractWidget first;

	/** Each frame of the pause menu, before it is drawn. */
	public static void frame(Screen pause) {
		long now = System.currentTimeMillis();
		if (pause != screen) {
			screen = pause;
			AT.clear();
			first = null;
			hint = -1;
			for (Hints.Active active : Hints.active()) {
				if (active.hint().hint().equals("menu_freaks") && !SPENT.contains(active.id())) {
					hint = active.id();
				}
			}
			if (hint < 0) {
				return;
			}
			SPENT.add(hint);
			since = now;
		}
		if (hint < 0) {
			return;
		}
		long t = now - since;
		int i = 0;
		for (var child : pause.children()) {
			if (!(child instanceof AbstractWidget widget)) {
				continue;
			}
			int[] was = AT.computeIfAbsent(widget, w -> new int[] {w.getX(), w.getY()});
			int[] shake = MenuFreak.shake(t, i);
			widget.setX(was[0] + shake[0]);
			widget.setY(was[1] + shake[1]);
			if (i == 0) {
				if (first != widget) {
					first = widget;
					firstWords = widget.getMessage();
				}
				widget.setMessage(MenuFreak.wrong(t) ? Component.literal(MenuFreak.WRONG) : firstWords);
			}
			i++;
		}
		if (MenuFreak.over(t)) {
			hint = -1;
		}
	}
}
