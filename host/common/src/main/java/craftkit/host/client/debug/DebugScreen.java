package craftkit.host.client.debug;

import craftkit.host.net.DebugAskPayload;
import craftkit.host.net.DebugTellPayload;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

/**
 * The debug screen: four tabs over one {@link DebugModel} the server refreshes every half
 * second while the screen is open. <b>Senses</b> is the host's reading of this player, as
 * meters. <b>Mind</b> is the mod's gauges, its sections and its buttons. <b>Events</b> is every
 * event the mod can do, with whether it is offered right now and why not, a filter, and a Fire
 * button on each row that forces it. <b>Log</b> is the last events the senses took in and the
 * last commands the host carried out. Opened with the debug key (F8 unless rebound) or
 * {@code /craftkit <mod> debug}; operators and the owner of a singleplayer world alone.
 *
 * <p>Everything is drawn by hand from the model, with hit testing on the rows, so the screen
 * never rebuilds widgets as the model changes; the one widget is the filter box.
 */
public final class DebugScreen extends Screen {
	private static final int REFRESH_TICKS = 10;
	private static final int ROW = 12;
	private static final int PAD = 8;
	private static final int BAR_WIDTH = 120;
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFF9A9A9A;
	private static final int DIM = 0xFF5A5A5A;
	private static final int GREEN = 0xFF7FD17F;
	private static final int RED = 0xFFE06060;
	private static final int YELLOW = 0xFFE0C860;
	private static final int BACK = 0xC0101014;
	private static final int PANEL = 0xFF1C1C22;
	private static final int BAR_BACK = 0xFF2A2A2A;
	private static final int BAR_FRONT = 0xFFDADADA;

	enum Tab {
		SENSES("Senses"), MIND("Mind"), EVENTS("Events"), LOG("Log");

		final String label;

		Tab(String label) {
			this.label = label;
		}
	}

	/** How the client sends an ask; the loader sets it once at startup. */
	private static Consumer<DebugAskPayload> sender = ask -> {
	};
	private static DebugScreen open;

	private final String mod;
	private DebugModel model = new DebugModel();
	private Tab tab = Tab.SENSES;
	private int scroll;
	private int contentHeight;
	private int ticks;
	private EditBox filter;
	private String filterText = "";
	private String flash = "";
	private long flashUntil;
	/** The clickable rectangles drawn this frame: x, y, w, h and what a click does. */
	private final List<Hit> hits = new ArrayList<>();

	private record Hit(int x, int y, int w, int h, Runnable action) {
		boolean contains(double px, double py) {
			return px >= x && px < x + w && py >= y && py < y + h;
		}
	}

	private DebugScreen(String mod) {
		super(Component.translatable("craftkit.debug.title", mod));
		this.mod = mod;
	}

	/** The loader tells the screen (and the HUD overlay) how to reach the server. */
	public static void sendWith(Consumer<DebugAskPayload> send) {
		sender = send;
		DebugHud.sendWith(send);
	}

	/** Opens the screen for a mod; an empty id asks the server which mods there are and takes the first. */
	public static void open(String mod) {
		Minecraft minecraft = Minecraft.getInstance();
		if (minecraft.level == null) {
			return;
		}
		DebugScreen screen = new DebugScreen(mod);
		open = screen;
		minecraft.gui.setScreen(screen);
		screen.ask("{\"kind\":\"snapshot\"}");
	}

	/** A server answer: an order to open, or a refresh of the open screen. */
	public static void receive(DebugTellPayload tell) {
		if (tell.json().contains("\"open\":true")) {
			open(tell.mod());
			return;
		}

		DebugModel parsed = DebugModel.parse(tell.json());
		DebugHud.latest(tell.mod(), parsed);
		DebugScreen screen = open;
		if (screen == null || Minecraft.getInstance().gui.screen() != screen) {
			return;
		}

		if (!parsed.message.isEmpty()) {
			screen.flash = parsed.message;
			screen.flashUntil = System.currentTimeMillis() + 4000;
		}
		if (!parsed.error.isEmpty() && parsed.mods.isEmpty() && screen.model.events.isEmpty()) {
			screen.model = parsed;
			return;
		}
		// A reply to a button keeps the panel; a snapshot replaces it.
		if (!parsed.gauges.isEmpty() || !parsed.senses.isEmpty()) {
			screen.model = parsed;
		} else if (!parsed.error.isEmpty()) {
			screen.model.error = parsed.error;
		}
		if (screen.mod.isEmpty() && !parsed.mods.isEmpty()) {
			open(parsed.mods.get(0));
		}
	}

	private void ask(String json) {
		sender.accept(new DebugAskPayload(mod, json));
	}

	@Override
	protected void init() {
		filter = new EditBox(this.font, this.width - PAD - 160, PAD + 4, 160, 14, Component.translatable("craftkit.debug.filter"));
		filter.setMaxLength(40);
		filter.setValue(filterText);
		filter.setResponder(text -> {
			filterText = text;
			scroll = 0;
		});
		filter.setHint(Component.translatable("craftkit.debug.filter"));
		addRenderableWidget(filter);
		filter.visible = tab == Tab.EVENTS;
	}

	@Override
	public boolean isPauseScreen() {
		return false;
	}

	@Override
	public void removed() {
		if (open == this) {
			open = null;
		}
	}

	@Override
	public void tick() {
		super.tick();
		ticks++;
		if (ticks % REFRESH_TICKS == 0) {
			ask("{\"kind\":\"snapshot\"}");
		}
		filter.visible = tab == Tab.EVENTS;
	}

	@Override
	public boolean mouseScrolled(double x, double y, double dx, double dy) {
		scroll = Math.max(0, Math.min(Math.max(0, contentHeight - (this.height - 60)), scroll - (int) (dy * ROW * 3)));

		return true;
	}

	@Override
	public boolean mouseClicked(MouseButtonEvent event, boolean doubled) {
		for (Hit hit : hits) {
			if (hit.contains(event.x(), event.y())) {
				hit.action().run();
				return true;
			}
		}

		return super.mouseClicked(event, doubled);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
		graphics.fill(0, 0, this.width, this.height, BACK);
		hits.clear();

		// The header: the title, the mod picker, the tabs.
		graphics.text(this.font, this.title, PAD, PAD, WHITE);
		int x = PAD;
		int tabY = PAD + 14;
		for (Tab t : Tab.values()) {
			int w = this.font.width(t.label) + 12;
			boolean on = t == tab;
			graphics.fill(x, tabY, x + w, tabY + 14, on ? PANEL : BAR_BACK);
			graphics.text(this.font, t.label, x + 6, tabY + 3, on ? WHITE : GREY);
			Tab chosen = t;
			hits.add(new Hit(x, tabY, w, 14, () -> {
				tab = chosen;
				scroll = 0;
			}));
			x += w + 4;
		}
		{
			String hud = DebugHud.enabled() ? "HUD: on" : "HUD: off";
			int w = this.font.width(hud) + 12;
			graphics.fill(x, tabY, x + w, tabY + 14, DebugHud.enabled() ? PANEL : BAR_BACK);
			graphics.text(this.font, hud, x + 6, tabY + 3, DebugHud.enabled() ? YELLOW : GREY);
			hits.add(new Hit(x, tabY, w, 14, () -> DebugHud.toggle(mod)));
			x += w + 4;
		}
		if (model.mods.size() > 1) {
			String picker = "mod: " + mod + " (click to switch)";
			int w = this.font.width(picker) + 8;
			graphics.text(this.font, picker, x + 8, tabY + 3, GREY);
			hits.add(new Hit(x + 8, tabY, w, 14, () -> {
				int index = (model.mods.indexOf(mod) + 1) % model.mods.size();
				open(model.mods.get(index));
			}));
		}
		if (!model.error.isEmpty()) {
			graphics.text(this.font, model.error, this.width - PAD - this.font.width(model.error), PAD, RED);
		} else if (!flash.isEmpty() && System.currentTimeMillis() < flashUntil && tab != Tab.EVENTS) {
			graphics.text(this.font, flash, this.width - PAD - this.font.width(flash), PAD, YELLOW);
		}

		// The body scrolls under a clip.
		int top = tabY + 20;
		int bottom = this.height - PAD;
		graphics.fill(PAD, top - 2, this.width - PAD, bottom, PANEL);
		graphics.enableScissor(PAD, top, this.width - PAD, bottom);
		int y = top + 2 - scroll;
		int start = y;
		int headerHits = hits.size();
		switch (tab) {
			case SENSES -> y = senses(graphics, y);
			case MIND -> y = mind(graphics, y);
			case EVENTS -> y = events(graphics, y, top, bottom);
			case LOG -> y = log(graphics, y);
		}
		contentHeight = y - start + 4;
		graphics.disableScissor();
		// Body rows scrolled out of the clip take no clicks; the header's hits stay.
		List<Hit> body = new ArrayList<>(hits.subList(headerHits, hits.size()));
		hits.subList(headerHits, hits.size()).clear();
		for (Hit hit : body) {
			if (hit.y() + hit.h() >= top && hit.y() <= bottom) {
				hits.add(hit);
			}
		}

		if (contentHeight > bottom - top) {
			// A scroll bar.
			int track = bottom - top;
			int thumb = Math.max(10, track * track / contentHeight);
			int at = top + (track - thumb) * scroll / Math.max(1, contentHeight - track);
			graphics.fill(this.width - PAD - 3, top, this.width - PAD, bottom, BAR_BACK);
			graphics.fill(this.width - PAD - 3, at, this.width - PAD, at + thumb, GREY);
		}

		super.extractRenderState(graphics, mouseX, mouseY, partialTick);
	}

	private int senses(GuiGraphicsExtractor graphics, int y) {
		y = heading(graphics, y, "This player, as the host reads them");
		for (DebugModel.Gauge gauge : model.senses) {
			y = gauge(graphics, y, gauge);
		}
		y += 4;
		y = heading(graphics, y, "Where and what");
		for (String line : model.senseLines) {
			y = line(graphics, y, line, GREY);
		}
		y += 4;
		y = heading(graphics, y, "Entities within 64 blocks");
		if (model.entities.isEmpty()) {
			y = line(graphics, y, "none", DIM);
		}
		for (String line : model.entities) {
			y = line(graphics, y, line, GREY);
		}

		return y;
	}

	private int mind(GuiGraphicsExtractor graphics, int y) {
		if (model.gauges.isEmpty() && model.sections.isEmpty()) {
			return line(graphics, y, model.error.isEmpty() ? "waiting for the mod" : model.error, DIM);
		}
		for (DebugModel.Gauge gauge : model.gauges) {
			y = gauge(graphics, y, gauge);
		}
		y += 4;
		if (!model.actions.isEmpty()) {
			int x = PAD + 6;
			for (DebugModel.ActionRow action : model.actions) {
				int w = this.font.width(action.label()) + 10;
				if (x + w > this.width - PAD - 6) {
					x = PAD + 6;
					y += 16;
				}
				graphics.fill(x, y, x + w, y + 13, BAR_BACK);
				graphics.text(this.font, action.label(), x + 5, y + 3, WHITE);
				String id = action.id();
				hits.add(new Hit(x, y, w, 13, () -> ask("{\"kind\":\"action\",\"id\":\"" + id + "\"}")));
				x += w + 4;
			}
			y += 20;
		}
		for (DebugModel.Section section : model.sections) {
			y = heading(graphics, y, section.title());
			if (section.lines().isEmpty()) {
				y = line(graphics, y, "nothing", DIM);
			}
			for (String text : section.lines()) {
				for (net.minecraft.util.FormattedCharSequence part : this.font.split(Component.literal(text), this.width - 2 * PAD - 20)) {
					graphics.text(this.font, part, PAD + 6, y, GREY);
					y += 10;
				}
			}
			y += 4;
		}

		return y;
	}

	private int events(GuiGraphicsExtractor graphics, int y, int top, int bottom) {
		List<DebugModel.EventRow> rows = model.eventsMatching(filterText);
		long offered = rows.stream().filter(DebugModel.EventRow::offered).count();
		y = heading(graphics, y, rows.size() + " events, " + offered + " offered right now" + (flash.isEmpty() || System.currentTimeMillis() > flashUntil ? "" : "  |  " + flash));
		int fireX = this.width - PAD - 40;
		for (DebugModel.EventRow row : rows) {
			if (y + ROW < top || y > bottom) {
				y += ROW;
				continue;
			}
			graphics.text(this.font, row.id(), PAD + 6, y + 1, row.offered() ? GREEN : WHITE);
			String tail = row.offered() ? "offered" : row.why();
			if (row.times() > 0) {
				tail += "  x" + row.times();
			}
			graphics.text(this.font, trim(row.words(), 150), PAD + 120, y + 1, GREY);
			graphics.text(this.font, trim(tail, fireX - (PAD + 280) - 6), PAD + 280, y + 1, row.offered() ? GREEN : DIM);
			graphics.fill(fireX, y, fireX + 34, y + 11, BAR_BACK);
			graphics.text(this.font, "Fire", fireX + 7, y + 1, WHITE);
			String id = row.id();
			hits.add(new Hit(fireX, y, 34, 11, () -> ask("{\"kind\":\"force\",\"event\":\"" + id + "\",\"player\":0}")));
			y += ROW;
		}

		return y;
	}

	private int log(GuiGraphicsExtractor graphics, int y) {
		y = heading(graphics, y, "Sensed lately, newest first");
		if (model.sensedEvents.isEmpty()) {
			y = line(graphics, y, "nothing yet", DIM);
		}
		for (String line : model.sensedEvents) {
			y = line(graphics, y, line, GREY);
		}
		y += 4;
		y = heading(graphics, y, "Carried out lately, newest first");
		if (model.commands.isEmpty()) {
			y = line(graphics, y, "nothing yet", DIM);
		}
		for (String line : model.commands) {
			y = line(graphics, y, line, line.contains("not done") ? RED : GREY);
		}

		return y;
	}

	private int heading(GuiGraphicsExtractor graphics, int y, String text) {
		graphics.text(this.font, text, PAD + 4, y, WHITE);

		return y + ROW + 2;
	}

	private int line(GuiGraphicsExtractor graphics, int y, String text, int colour) {
		graphics.text(this.font, trim(text, this.width - 2 * PAD - 16), PAD + 6, y, colour);

		return y + 10;
	}

	private int gauge(GuiGraphicsExtractor graphics, int y, DebugModel.Gauge gauge) {
		int labelWidth = 90;
		graphics.text(this.font, gauge.label(), PAD + 6, y + 1, GREY);
		int left = PAD + 6 + labelWidth;
		graphics.fill(left, y + 2, left + BAR_WIDTH, y + 9, BAR_BACK);
		int filled = (int) (BAR_WIDTH * gauge.fraction());
		graphics.fill(left, y + 2, left + filled, y + 9, gauge.fraction() > 0.85f ? YELLOW : BAR_FRONT);
		graphics.text(this.font, trim(gauge.text(), this.width - (left + BAR_WIDTH + 8) - PAD - 8), left + BAR_WIDTH + 8, y + 1, WHITE);

		return y + ROW;
	}

	private String trim(String text, int width) {
		if (width <= 10 || this.font.width(text) <= width) {
			return text;
		}
		String cut = text;
		while (cut.length() > 1 && this.font.width(cut + "…") > width) {
			cut = cut.substring(0, cut.length() - 1);
		}

		return cut + "…";
	}
}
