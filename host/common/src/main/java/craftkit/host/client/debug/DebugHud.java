package craftkit.host.client.debug;

import craftkit.host.net.DebugAskPayload;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

/**
 * The debug screen's essentials over the game while playing (owner, 2026-11-09: "make it so
 * you can play with debug ui, like make it hud elements"): the mod's gauges as small bars, the
 * story's NEXT line, the last few things that happened, and a few of the senses' meters, in the
 * top right. Toggled with the HUD key (F9 unless rebound) or the screen's HUD button. While on,
 * the client asks the server for a snapshot once a second; the same answers feed the screen.
 */
public final class DebugHud {
	private static final int ASK_EVERY_TICKS = 20;
	private static final int WIDTH = 230;
	private static final int BAR = 70;
	private static final int BACK = 0x90101014;
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFFB0B0B0;
	private static final int DIM = 0xFF707070;
	private static final int BAR_BACK = 0xC02A2A2A;
	private static final int BAR_FRONT = 0xFFDADADA;
	private static final int YELLOW = 0xFFE0C860;
	private static final int RED = 0xFFE06060;

	private static boolean enabled;
	private static String mod = "";
	private static DebugModel model = new DebugModel();
	private static int ticks;
	private static Consumer<DebugAskPayload> sender = ask -> {
	};

	private DebugHud() {
	}

	public static void sendWith(Consumer<DebugAskPayload> send) {
		sender = send;
	}

	public static boolean enabled() {
		return enabled;
	}

	/** Turns the overlay on for a mod (or off); an empty id lets the first answer name the mod. */
	public static void toggle(String modId) {
		enabled = !enabled;
		if (enabled) {
			mod = modId;
			ticks = ASK_EVERY_TICKS - 1;
		}
	}

	/** Out of a world: the overlay off and the last world's picture gone, so the next server (where they may not be an operator) never shows it nor is asked every second. */
	public static void leave() {
		enabled = false;
		mod = "";
		model = new DebugModel();
		ticks = 0;
	}

	/** Every snapshot the client gets, from the screen's asks or the overlay's. */
	static void latest(String modId, DebugModel parsed) {
		if (!parsed.gauges.isEmpty() || !parsed.senses.isEmpty()) {
			model = parsed;
			if (mod.isEmpty() && !modId.isEmpty()) {
				mod = modId;
			}
			if (mod.isEmpty() && !parsed.mods.isEmpty()) {
				mod = parsed.mods.get(0);
			}
		}
	}

	/** Once a client tick. */
	public static void tick(Minecraft minecraft) {
		if (!enabled || minecraft.level == null || minecraft.player == null) {
			return;
		}
		// The open screen asks on its own; the overlay asks only when it is alone.
		if (minecraft.gui.screen() instanceof DebugScreen) {
			return;
		}
		if (++ticks >= ASK_EVERY_TICKS) {
			ticks = 0;
			sender.accept(new DebugAskPayload(mod, "{\"kind\":\"snapshot\"}"));
		}
	}

	/** Once a frame over the HUD, when on and no screen covers the game. */
	public static void render(GuiGraphicsExtractor graphics, Font font, int width, int height) {
		Minecraft minecraft = Minecraft.getInstance();
		if (!enabled || minecraft.level == null || minecraft.gui.screen() != null) {
			return;
		}

		List<Runnable> lines = new ArrayList<>();
		int x = width - WIDTH - 4;
		int[] y = {4};
		int lineHeight = 10;

		// A measured pass first, so the backing fits.
		int count = 1 + model.gauges.size() + 2 + Math.min(4, senseLines()) + 1 + Math.min(5, model.commands.size());
		graphics.fill(x - 4, 2, width - 2, 2 + count * lineHeight + 8, BACK);

		String title = (mod.isEmpty() ? "craftkit" : mod) + " (F9 hides, F8 opens)";
		graphics.text(font, title, x, y[0], WHITE);
		y[0] += lineHeight;

		for (DebugModel.Gauge gauge : model.gauges) {
			gauge(graphics, font, x, y[0], gauge);
			y[0] += lineHeight;
		}

		String next = model.sections.stream()
				.filter(s -> s.title().equals("Story"))
				.flatMap(s -> s.lines().stream())
				.filter(l -> l.startsWith("NEXT:"))
				.findFirst()
				.orElse("NEXT: ?");
		graphics.text(font, trim(font, next, WIDTH), x, y[0], YELLOW);
		y[0] += lineHeight;

		graphics.text(font, "you, as sensed:", x, y[0], DIM);
		y[0] += lineHeight;
		int shown = 0;
		for (DebugModel.Gauge gauge : model.senses) {
			if (!isShownSense(gauge.label())) {
				continue;
			}
			gauge(graphics, font, x, y[0], gauge);
			y[0] += lineHeight;
			if (++shown >= 4) {
				break;
			}
		}

		graphics.text(font, "lately:", x, y[0], DIM);
		y[0] += lineHeight;
		for (String line : model.commands.stream().limit(5).toList()) {
			graphics.text(font, trim(font, line, WIDTH), x, y[0], line.contains("not done") ? RED : GREY);
			y[0] += lineHeight;
		}
		if (model.commands.isEmpty()) {
			graphics.text(font, "nothing yet", x, y[0], DIM);
		}
		lines.forEach(Runnable::run);
	}

	private static int senseLines() {
		int n = 0;
		for (DebugModel.Gauge gauge : model.senses) {
			if (isShownSense(gauge.label())) {
				n++;
			}
		}

		return n;
	}

	private static boolean isShownSense(String label) {
		return label.equals("fear") || label.equals("pace") || label.equals("look backs") || label.equals("light");
	}

	private static void gauge(GuiGraphicsExtractor graphics, Font font, int x, int y, DebugModel.Gauge gauge) {
		graphics.text(font, trim(font, gauge.label(), 60), x, y, GREY);
		int left = x + 64;
		graphics.fill(left, y + 2, left + BAR, y + 8, BAR_BACK);
		graphics.fill(left, y + 2, left + (int) (BAR * gauge.fraction()), y + 8, gauge.fraction() > 0.85f ? YELLOW : BAR_FRONT);
		graphics.text(font, trim(font, gauge.text(), WIDTH - 64 - BAR - 6), left + BAR + 4, y, WHITE);
	}

	private static String trim(Font font, String text, int width) {
		if (font.width(text) <= width) {
			return text;
		}
		String cut = text;
		while (cut.length() > 1 && font.width(cut + "…") > width) {
			cut = cut.substring(0, cut.length() - 1);
		}

		return cut + "…";
	}
}
