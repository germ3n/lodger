package craftkit.host.client.debug;

import craftkit.host.net.DebugAskPayload;
import craftkit.host.net.DebugTellPayload;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.network.chat.Component;
import net.minecraft.util.FormattedCharSequence;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

/**
 * The debug panel: a strip down the right of the screen, a third of it wide, over the world,
 * which stays in view and keeps running beside it (owner, 2026-11-09: the full screen hid the
 * events it was there to watch). Four tabs over one {@link DebugModel} the server refreshes
 * every half second while it is open. <b>Senses</b> is the host's reading of this player, as
 * meters. <b>Mind</b> is the mod's gauges, its buttons, and its sections, whose items (each plan)
 * carry buttons of their own. <b>Events</b> is every event the mod can do, with whether it is
 * offered right now and why not, a filter, and a Fire button on each. <b>Log</b> is the last
 * events the senses took in and the last commands carried out. Opened with the debug key (F8
 * unless rebound; fn+F8 on a Mac keyboard) or {@code /craftkit <mod> debug}; operators and the
 * owner of a singleplayer world alone.
 *
 * <p>Everything in the panel is drawn by hand at {@link PanelLayout#SCALE} into the panel's own
 * coordinates ({@link #pw} by {@link #ph} units), with hit testing on the rows in those units,
 * so the screen never rebuilds widgets as the model changes; the one widget is the filter box,
 * drawn at the game's own scale.
 */
public final class DebugScreen extends Screen {
	private static final int REFRESH_TICKS = 10;
	private static final float SCALE = PanelLayout.SCALE;
	private static final int ROW = 11;
	private static final int PAD = 6;
	private static final int BAR_WIDTH = 70;
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFFA8A8A8;
	private static final int DIM = 0xFF666666;
	private static final int GREEN = 0xFF7FD17F;
	private static final int RED = 0xFFE06060;
	private static final int YELLOW = 0xFFE0C860;
	/** The panel: dark, and see-through enough that the world still shows behind it. */
	private static final int PANEL = 0xB8101014;
	private static final int TAB_ON = 0xE02A2A34;
	private static final int BUTTON = 0xE0303038;
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
	private Tab tab = Tab.MIND;
	private int scroll;
	private int contentHeight;
	private int ticks;
	private EditBox filter;
	private String filterText = "";
	private String flash = "";
	private long flashUntil;
	/** The panel's left edge in GUI pixels, and its size in its own units. */
	private int left;
	private int pw;
	private int ph;
	/** The mouse in the panel's units, this frame. */
	private double mx;
	private double my;
	/** What to show by the mouse after everything else is drawn, if anything. */
	private String tooltip = "";
	/** The clickable rectangles drawn this frame, in the panel's units. */
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

	/** Opens the panel for a mod; an empty id asks the server which mods there are and takes the first. */
	public static void open(String mod) {
		Minecraft minecraft = Minecraft.getInstance();
		if (minecraft.level == null) {
			return;
		}
		DebugScreen screen = new DebugScreen(mod);
		if (open != null) {
			screen.tab = open.tab;
		}
		open = screen;
		minecraft.gui.setScreen(screen);
		screen.ask("{\"kind\":\"snapshot\"}");
	}

	/** A server answer: an order to open, or a refresh of the open panel. */
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

	private void action(String id) {
		ask("{\"kind\":\"action\",\"id\":\"" + id.replace("\"", "") + "\"}");
	}

	@Override
	protected void init() {
		left = PanelLayout.left(this.width);
		pw = PanelLayout.units(this.width);
		ph = (int) (this.height / SCALE);
		int boxWidth = Math.min(110, this.width - left - 2 * PAD);
		filter = new EditBox(this.font, this.width - PAD - boxWidth, this.height - PAD - 14, boxWidth, 14, Component.translatable("craftkit.debug.filter"));
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

	/** No blur and no darkening: the world beside the panel is what is being watched. */
	@Override
	public void extractBackground(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
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
		if (x < left) {
			return false;
		}
		int view = ph - bodyTop() - PAD;
		scroll = Math.max(0, Math.min(Math.max(0, contentHeight - view), scroll - (int) (dy * ROW * 3)));

		return true;
	}

	@Override
	public boolean mouseClicked(MouseButtonEvent event, boolean doubled) {
		double px = PanelLayout.toPanel(event.x(), left);
		double py = PanelLayout.toPanel(event.y(), 0);
		for (Hit hit : hits) {
			if (hit.contains(px, py)) {
				hit.action().run();
				return true;
			}
		}

		return super.mouseClicked(event, doubled);
	}

	private int bodyTop() {
		return PAD + 2 * ROW + 6;
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
		hits.clear();
		tooltip = "";
		mx = PanelLayout.toPanel(mouseX, left);
		my = PanelLayout.toPanel(mouseY, 0);
		graphics.fill(left, 0, this.width, this.height, PANEL);

		graphics.pose().pushMatrix();
		graphics.pose().translate(left, 0);
		graphics.pose().scale(SCALE, SCALE);

		// The header: the title or what just happened, then the tabs.
		String head = !model.error.isEmpty() ? model.error
				: !flash.isEmpty() && System.currentTimeMillis() < flashUntil ? flash
				: this.title.getString() + (model.mods.size() > 1 ? "  (" + mod + ", click to switch)" : "");
		int headColour = !model.error.isEmpty() ? RED : !flash.isEmpty() && System.currentTimeMillis() < flashUntil ? YELLOW : WHITE;
		graphics.text(this.font, trim(head, pw - 2 * PAD), PAD, PAD, headColour);
		if (model.mods.size() > 1) {
			hits.add(new Hit(PAD, PAD, pw - 2 * PAD, ROW, () -> {
				int index = (model.mods.indexOf(mod) + 1) % model.mods.size();
				open(model.mods.get(index));
			}));
		}
		int x = PAD;
		int tabY = PAD + ROW + 2;
		for (Tab t : Tab.values()) {
			x = button(graphics, x, tabY, t.label, t == tab ? WHITE : GREY, t == tab ? TAB_ON : BUTTON, () -> {
				tab = t;
				scroll = 0;
			});
		}
		button(graphics, x, tabY, DebugHud.enabled() ? "HUD on" : "HUD off", DebugHud.enabled() ? YELLOW : GREY, BUTTON, () -> DebugHud.toggle(mod));

		// The body scrolls under a clip, set in GUI pixels while the pose is still plain.
		int top = bodyTop();
		int bottom = ph - (tab == Tab.EVENTS ? (int) (22 / SCALE) : PAD);
		graphics.pose().popMatrix();
		graphics.enableScissor(left, (int) (top * SCALE), this.width, (int) (bottom * SCALE));
		graphics.pose().pushMatrix();
		graphics.pose().translate(left, 0);
		graphics.pose().scale(SCALE, SCALE);
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
		// Body rows scrolled out of the clip take no clicks; the header's hits stay.
		List<Hit> body = new ArrayList<>(hits.subList(headerHits, hits.size()));
		hits.subList(headerHits, hits.size()).clear();
		for (Hit hit : body) {
			if (hit.y() + hit.h() >= top && hit.y() <= bottom) {
				hits.add(hit);
			}
		}
		if (contentHeight > bottom - top) {
			int track = bottom - top;
			int thumb = Math.max(10, track * track / contentHeight);
			int at = top + (track - thumb) * scroll / Math.max(1, contentHeight - track);
			graphics.fill(pw - 3, top, pw, bottom, BAR_BACK);
			graphics.fill(pw - 3, at, pw, at + thumb, GREY);
		}
		graphics.pose().popMatrix();
		graphics.disableScissor();
		if (!tooltip.isEmpty()) {
			graphics.pose().pushMatrix();
			graphics.pose().translate(left, 0);
			graphics.pose().scale(SCALE, SCALE);
			tooltip(graphics);
			graphics.pose().popMatrix();
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
			y = wrapped(graphics, y, line, GREY);
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
			int x = PAD;
			for (DebugModel.ActionRow action : model.actions) {
				int w = this.font.width(action.label()) + 8;
				if (x + w > pw - PAD) {
					x = PAD;
					y += ROW + 3;
				}
				String id = action.id();
				x = button(graphics, x, y, action.label(), WHITE, BUTTON, () -> action(id));
			}
			y += ROW + 6;
		}
		for (DebugModel.Section section : model.sections) {
			y = heading(graphics, y, section.title());
			if (section.lines().isEmpty() && section.items().isEmpty()) {
				y = line(graphics, y, "nothing", DIM);
			}
			for (String text : section.lines()) {
				y = wrapped(graphics, y, text, GREY);
			}
			for (DebugModel.Item item : section.items()) {
				y = item(graphics, y, item);
			}
			y += 4;
		}

		return y;
	}

	/** A section's row: its text with its buttons on the right, and the quieter line under it. */
	private int item(GuiGraphicsExtractor graphics, int y, DebugModel.Item item) {
		int buttons = 0;
		for (DebugModel.ActionRow action : item.actions()) {
			buttons += this.font.width(action.label()) + 12;
		}
		graphics.text(this.font, trim(item.text(), pw - 2 * PAD - buttons - 4), PAD, y + 1, WHITE);
		goTo(graphics, item.text(), PAD, y, pw - 2 * PAD - buttons - 4, ROW);
		int x = pw - PAD - buttons;
		for (DebugModel.ActionRow action : item.actions()) {
			String id = action.id();
			x = button(graphics, x, y, action.label(), WHITE, BUTTON, () -> action(id));
		}
		y += ROW;
		if (!item.detail().isEmpty()) {
			y = line(graphics, y, "  " + item.detail(), item.detail().startsWith("in view") || item.detail().startsWith("waiting") ? YELLOW : DIM);
		}

		return y + 2;
	}

	private int events(GuiGraphicsExtractor graphics, int y, int top, int bottom) {
		List<DebugModel.EventRow> rows = model.eventsMatching(filterText);
		long offered = rows.stream().filter(DebugModel.EventRow::offered).count();
		y = heading(graphics, y, rows.size() + " events, " + offered + " offered right now");
		int fireX = pw - PAD - 26;
		int idWidth = Math.min(110, (pw - 2 * PAD) / 3);
		for (DebugModel.EventRow row : rows) {
			if (y + ROW < top || y > bottom) {
				y += ROW;
				continue;
			}
			graphics.text(this.font, trim(row.id(), idWidth - 4), PAD, y + 1, row.offered() ? GREEN : WHITE);
			String tail = row.offered() ? "offered" : row.why();
			if (row.times() > 0) {
				tail += "  x" + row.times();
			}
			graphics.text(this.font, trim(tail, fireX - (PAD + idWidth) - 4), PAD + idWidth, y + 1, row.offered() ? GREEN : DIM);
			String id = row.id();
			button(graphics, fireX, y, "Fire", WHITE, BUTTON, () -> ask("{\"kind\":\"force\",\"event\":\"" + id + "\",\"player\":0}"));
			if (mx >= PAD && mx < fireX && my >= y && my < y + ROW && my >= top && my <= bottom) {
				graphics.fill(PAD, y + ROW - 1, fireX - 4, y + ROW, DIM);
				tooltip = row.id() + " (" + row.words() + ")\n" + (row.about().isEmpty() ? (row.offered() ? "offered" : row.why()) : row.about());
			}
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
			y = wrapped(graphics, y, line, GREY);
		}
		y += 4;
		y = heading(graphics, y, "Carried out lately, newest first");
		if (model.commands.isEmpty()) {
			y = line(graphics, y, "nothing yet", DIM);
		}
		for (String line : model.commands) {
			y = wrapped(graphics, y, line, line.contains("not done") ? RED : GREY);
		}

		return y;
	}

	/** A button at x, y in the panel's units; answers the x after it. */
	private int button(GuiGraphicsExtractor graphics, int x, int y, String label, int colour, int back, Runnable action) {
		int w = this.font.width(label) + 8;
		graphics.fill(x, y, x + w, y + ROW, back);
		graphics.text(this.font, label, x + 4, y + 2, colour);
		hits.add(new Hit(x, y, w, ROW, action));

		return x + w + 3;
	}

	private int heading(GuiGraphicsExtractor graphics, int y, String text) {
		graphics.text(this.font, trim(text, pw - 2 * PAD), PAD, y, WHITE);

		return y + ROW + 1;
	}

	private int line(GuiGraphicsExtractor graphics, int y, String text, int colour) {
		graphics.text(this.font, trim(text, pw - 2 * PAD - 4), PAD, y, colour);

		return y + 10;
	}

	/** A long line over as many lines as it takes, rather than cut: the panel is narrow. */
	private int wrapped(GuiGraphicsExtractor graphics, int y, String text, int colour) {
		int start = y;
		for (FormattedCharSequence part : this.font.split(Component.literal(text), pw - 2 * PAD - 4)) {
			graphics.text(this.font, part, PAD, y, colour);
			y += 10;
		}
		goTo(graphics, text, PAD, start, pw - 2 * PAD, y - start);

		return y;
	}

	/**
	 * A line with a spot in it is a link: underlined under the mouse, Shift+click goes there. A
	 * plain click does nothing (owner, 2026-11-10: a stray click put them deep in a cave).
	 */
	private void goTo(GuiGraphicsExtractor graphics, String text, int x, int y, int w, int h) {
		Coordinates.find(text).ifPresent(spot -> {
			hits.add(new Hit(x, y, w, h, () -> {
				if (net.minecraft.client.Minecraft.getInstance().hasShiftDown()) {
					ask("{\"kind\":\"teleport\",\"pos\":" + spot.json() + "}");
				}
			}));
			if (mx >= x && mx < x + w && my >= y && my < y + h) {
				graphics.fill(x, y + h - 1, x + w, y + h, YELLOW);
				tooltip = "shift+click: go to " + spot.x() + " " + spot.y().map(String::valueOf).orElse("(the ground)") + " " + spot.z();
			}
		});
	}

	/** The tooltip, by the mouse and inside the panel: a dark box of wrapped lines. */
	private void tooltip(GuiGraphicsExtractor graphics) {
		int width = Math.min(pw - 2 * PAD, 280);
		List<FormattedCharSequence> lines = new ArrayList<>();
		for (String text : tooltip.split("\n")) {
			lines.addAll(this.font.split(Component.literal(text), width - 8));
		}
		int height = lines.size() * 10 + 6;
		int x = (int) Math.max(PAD, Math.min(mx + 8, pw - PAD - width));
		int y = (int) Math.min(my + 10, ph - PAD - height);
		if (y < PAD) {
			y = PAD;
		}
		graphics.fill(x - 1, y - 1, x + width + 1, y + height + 1, GREY);
		graphics.fill(x, y, x + width, y + height, 0xF4101016);
		int ty = y + 3;
		for (FormattedCharSequence line : lines) {
			graphics.text(this.font, line, x + 4, ty, WHITE);
			ty += 10;
		}
	}

	private int gauge(GuiGraphicsExtractor graphics, int y, DebugModel.Gauge gauge) {
		int labelWidth = 72;
		graphics.text(this.font, trim(gauge.label(), labelWidth - 4), PAD, y + 1, GREY);
		int bar = PAD + labelWidth;
		graphics.fill(bar, y + 2, bar + BAR_WIDTH, y + 8, BAR_BACK);
		int filled = (int) (BAR_WIDTH * gauge.fraction());
		graphics.fill(bar, y + 2, bar + filled, y + 8, gauge.fraction() > 0.85f ? YELLOW : BAR_FRONT);
		int textX = bar + BAR_WIDTH + 6;
		List<FormattedCharSequence> parts = this.font.split(Component.literal(gauge.text()), pw - textX - PAD);
		for (FormattedCharSequence part : parts) {
			graphics.text(this.font, part, textX, y + 1, WHITE);
			y += 10;
		}

		return y + (parts.isEmpty() ? ROW : 2);
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
