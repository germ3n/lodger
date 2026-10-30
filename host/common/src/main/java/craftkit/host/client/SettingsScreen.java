package craftkit.host.client;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import craftkit.host.LoadedMod;
import craftkit.host.Preparation;
import craftkit.host.Setting;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * A mod's settings, from the title screen (owner, 2026-11-09): the few keys it offers, grouped,
 * in a list that scrolls between a fixed title and a fixed footer of Done and Cancel; each row a
 * label and a button that changes it (a press for the next value, or - and + for a number).
 * Done writes the changes into the mod's config file and initialises the mod again; when that
 * leaves something to fetch or load (a new model size), the preparing screen comes up at once.
 */
public final class SettingsScreen extends Screen {
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFF9A9A9A;
	private static final int DIM = 0xFF6A6A6A;
	private static final int PANEL = 0xA0000000;
	private static final int LINE = 0xFF3A3A3A;
	private static final int WIDTH = 320;
	private static final int TOP = 32;
	private static final int FOOTER = 40;

	private final Screen parent;
	private final LoadedMod mod;
	private final Map<String, JsonElement> values = new LinkedHashMap<>();
	private final List<List<AbstractWidget>> rowWidgets = new ArrayList<>();
	private List<ListLayout.Place> places = List.of();
	private int scroll;
	private String error = "";

	public SettingsScreen(Screen parent, LoadedMod mod) {
		super(Component.translatable("craftkit.settings.title", mod.name()));
		this.parent = parent;
		this.mod = mod;
		for (Setting setting : mod.settings()) {
			values.put(setting.key(), setting.value());
		}
	}

	private int left() {
		return this.width / 2 - WIDTH / 2;
	}

	private int bottom() {
		return this.height - FOOTER;
	}

	@Override
	protected void init() {
		rowWidgets.clear();
		places = ListLayout.places(mod.settings().stream().map(Setting::group).toList());
		int control = left() + WIDTH - 150;
		for (Setting setting : mod.settings()) {
			List<AbstractWidget> widgets = new ArrayList<>();
			if (setting.kind().equals("number")) {
				widgets.add(addRenderableWidget(Button.builder(Component.literal("-"), b -> step(setting, false)).bounds(control, 0, 20, 20).build()));
				widgets.add(addRenderableWidget(Button.builder(Component.literal("+"), b -> step(setting, true)).bounds(control + 130, 0, 20, 20).build()));
			} else {
				widgets.add(addRenderableWidget(Button.builder(Component.literal(Setting.show(values.get(setting.key()))), b -> {
					step(setting, true);
					b.setMessage(Component.literal(Setting.show(values.get(setting.key()))));
				}).bounds(control, 0, 150, 20).build()));
			}
			rowWidgets.add(widgets);
		}
		int footer = this.height - 30;
		addRenderableWidget(Button.builder(Component.translatable("gui.done"), b -> done()).bounds(this.width / 2 - 154, footer, 150, 20).build());
		addRenderableWidget(Button.builder(Component.translatable("gui.cancel"), b -> onClose()).bounds(this.width / 2 + 4, footer, 150, 20).build());
		scroll = ListLayout.clamp(scroll, ListLayout.height(places), bottom() - TOP);
		place();
	}

	/** Moves each row's buttons to where the scroll puts them, hiding the ones out of the list. */
	private void place() {
		for (int i = 0; i < rowWidgets.size(); i++) {
			int y = TOP + places.get(i).top() - scroll;
			for (AbstractWidget widget : rowWidgets.get(i)) {
				widget.setY(y);
				widget.visible = ListLayout.shows(y, 20, TOP, bottom());
			}
		}
	}

	@Override
	public boolean mouseScrolled(double x, double y, double dx, double dy) {
		scroll = ListLayout.clamp(scroll - (int) (dy * ListLayout.ROW), ListLayout.height(places), bottom() - TOP);
		place();

		return true;
	}

	private void step(Setting setting, boolean up) {
		values.put(setting.key(), setting.next(values.get(setting.key()), up));
	}

	/** The changed keys written and the mod initialised again with them. */
	private void done() {
		JsonObject change = new JsonObject();
		for (Setting setting : mod.settings()) {
			JsonElement now = values.get(setting.key());
			if (!now.equals(setting.value())) {
				craftkit.host.ConfigFile.mergeInto(change, setting.change(now));
			}
		}
		if (change.size() > 0) {
			String failed = mod.reconfigure(change);
			if (!failed.isEmpty()) {
				error = failed;
				return;
			}
		}
		Minecraft minecraft = Minecraft.getInstance();
		Preparation.current().filter(Preparation::needed)
				.ifPresentOrElse(p -> minecraft.gui.setScreen(new PreparingScreen(parent, p)), () -> minecraft.gui.setScreen(parent));
	}

	@Override
	public void onClose() {
		Minecraft.getInstance().gui.setScreen(parent);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
		int left = left();
		graphics.fill(left - 8, TOP - 4, left + WIDTH + 8, bottom() + 2, PANEL);
		graphics.fill(left - 8, bottom() + 2, left + WIDTH + 8, bottom() + 3, LINE);
		super.extractRenderState(graphics, mouseX, mouseY, partialTick);
		graphics.centeredText(this.font, this.title, this.width / 2, 12, WHITE);

		graphics.enableScissor(left - 8, TOP - 4, left + WIDTH + 8, bottom() + 2);
		List<Setting> settings = mod.settings();
		for (int i = 0; i < settings.size(); i++) {
			Setting setting = settings.get(i);
			int y = TOP + places.get(i).top() - scroll;
			if (places.get(i).heading()) {
				graphics.text(this.font, setting.group().toUpperCase(java.util.Locale.ROOT), left, y - 12, DIM);
			}
			graphics.text(this.font, setting.label(), left, y + 6, WHITE);
			if (setting.kind().equals("number")) {
				int control = left + WIDTH - 150;
				graphics.centeredText(this.font, Component.literal(Setting.show(values.get(setting.key()))), control + 75, y + 6, WHITE);
			}
		}
		graphics.disableScissor();

		int height = ListLayout.height(places);
		int view = bottom() - TOP;
		if (height > view) {
			int thumb = Math.max(12, view * view / height);
			int at = TOP + (view - thumb) * scroll / Math.max(1, height - view);
			graphics.fill(left + WIDTH + 4, TOP, left + WIDTH + 6, bottom(), LINE);
			graphics.fill(left + WIDTH + 4, at, left + WIDTH + 6, at + thumb, GREY);
		}
		if (!error.isEmpty()) {
			graphics.centeredText(this.font, Component.literal(error), this.width / 2, this.height - 44, 0xFFE06060);
		}
	}
}
