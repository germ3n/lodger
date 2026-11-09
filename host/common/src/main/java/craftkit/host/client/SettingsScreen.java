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
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * A mod's settings, from the title screen (owner), in two tabs. **Settings**: the few keys it
 * offers, grouped, each row a label and a button that changes it (a press for the next value, or
 * - and + for a number). **All keys** (owner: "a full config editor... every key, just in a
 * separate tab"): every key of the mod's config file as it is, grouped by its section, a toggle
 * for a yes or no and a field for the rest (a list as its JSON). The list scrolls between a fixed
 * title and a fixed footer of Done and Cancel. Done writes the changes of both tabs into the mod's
 * config file and initialises the mod again, which checks them: a value it refuses is said, and
 * the file is put back. When that leaves something to fetch or load (a new model size), the
 * preparing screen comes up at once.
 */
public final class SettingsScreen extends Screen {
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFF9A9A9A;
	private static final int DIM = 0xFF6A6A6A;
	private static final int RED = 0xFFE06060;
	private static final int PANEL = 0xA0000000;
	private static final int LINE = 0xFF3A3A3A;
	private static final int WIDTH = 320;
	private static final int TOP = 52;
	private static final int FOOTER = 40;

	/** One key of the config file: its path, its section and its value as the file has it. */
	private record Leaf(List<String> path, String group, JsonElement value) {
		String key() {
			return String.join(".", path);
		}

		String label() {
			return path.size() > 1 ? String.join(".", path.subList(1, path.size())) : path.get(0);
		}
	}

	private final Screen parent;
	private final LoadedMod mod;
	private final Map<String, JsonElement> values = new LinkedHashMap<>();
	private final List<Leaf> leaves = new ArrayList<>();
	/** All keys' rows as edited: a field's text, or a toggle's value, by key. */
	private final Map<String, String> edits = new LinkedHashMap<>();
	/** All keys' rows sent back to their preset: taken out of the file, which the mod fills in again. */
	private final java.util.Set<String> toPreset = new java.util.HashSet<>();
	private final List<List<AbstractWidget>> rowWidgets = new ArrayList<>();
	private List<ListLayout.Place> places = List.of();
	private boolean all;
	private int scroll;
	private String error = "";

	public SettingsScreen(Screen parent, LoadedMod mod) {
		super(Component.translatable("craftkit.settings.title", mod.name()));
		this.parent = parent;
		this.mod = mod;
		load();
	}

	/** Both tabs from the mod as it is now: its settings, and its config file key by key. */
	private void load() {
		values.clear();
		leaves.clear();
		edits.clear();
		toPreset.clear();
		for (Setting setting : mod.settings()) {
			values.put(setting.key(), setting.value());
		}
		mod.configFile().ifPresent(file -> flatten(file, new ArrayList<>()));
		for (Leaf leaf : leaves) {
			edits.put(leaf.key(), ConfigText.text(leaf.value()));
		}
	}

	/** Every key of an object, depth first, in the file's own order. */
	private void flatten(JsonObject object, List<String> path) {
		for (Map.Entry<String, JsonElement> entry : object.entrySet()) {
			List<String> at = new ArrayList<>(path);
			at.add(entry.getKey());
			if (entry.getValue().isJsonObject()) {
				flatten(entry.getValue().getAsJsonObject(), at);
			} else {
				leaves.add(new Leaf(at, at.size() > 1 ? at.get(0) : "general", entry.getValue()));
			}
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
		int half = WIDTH / 2 - 2;
		addRenderableWidget(Button.builder(Component.translatable("craftkit.settings.tab.settings"), b -> tab(false))
				.bounds(left(), 26, half, 20).build()).active = all;
		addRenderableWidget(Button.builder(Component.translatable("craftkit.settings.tab.all"), b -> tab(true))
				.bounds(left() + WIDTH - half, 26, half, 20).build()).active = !all && !leaves.isEmpty();

		int control = left() + WIDTH - 150;
		if (all) {
			places = ListLayout.places(leaves.stream().map(Leaf::group).toList());
			for (Leaf leaf : leaves) {
				List<AbstractWidget> widgets = new ArrayList<>();
				JsonElement was = leaf.value();
				// Back to its preset: the key taken out of the file as Done or a tab change
				// applies, and the preset's value written in its place.
				Button reset = Button.builder(Component.literal("↺"), b -> {
					toPreset.add(leaf.key());
					edits.put(leaf.key(), ConfigText.text(leaf.value()));
					rebuildWidgets();
				}).bounds(control, 0, 20, 20).tooltip(net.minecraft.client.gui.components.Tooltip.create(Component.translatable("craftkit.settings.to_preset"))).build();
				widgets.add(addRenderableWidget(reset));
				if (was.isJsonPrimitive() && was.getAsJsonPrimitive().isBoolean()) {
					widgets.add(addRenderableWidget(Button.builder(Component.literal(edits.get(leaf.key())), b -> {
						String flipped = edits.get(leaf.key()).equals("true") ? "false" : "true";
						edits.put(leaf.key(), flipped);
						toPreset.remove(leaf.key());
						b.setMessage(Component.literal(flipped));
					}).bounds(control + 24, 0, 126, 20).build()));
				} else {
					EditBox field = new EditBox(this.font, control + 24, 0, 126, 20, Component.literal(leaf.key()));
					field.setMaxLength(4096);
					field.setValue(edits.get(leaf.key()));
					field.setResponder(text -> {
						edits.put(leaf.key(), text);
						if (!text.equals(ConfigText.text(leaf.value()))) {
							toPreset.remove(leaf.key());
						}
					});
					widgets.add(addRenderableWidget(field));
				}
				rowWidgets.add(widgets);
			}
		} else {
			places = ListLayout.places(mod.settings().stream().map(Setting::group).toList());
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
		}
		int footer = this.height - 30;
		addRenderableWidget(Button.builder(Component.translatable("gui.done"), b -> done()).bounds(this.width / 2 - 154, footer, 150, 20).build());
		addRenderableWidget(Button.builder(Component.translatable("gui.cancel"), b -> onClose()).bounds(this.width / 2 + 4, footer, 150, 20).build());
		scroll = ListLayout.clamp(scroll, ListLayout.height(places), bottom() - TOP);
		place();
	}

	/**
	 * The other tab, after what was changed in this one is applied and both read again: All keys
	 * shows the file as it is (a preset picked a moment ago, its values filled in), never as it was
	 * when the screen opened.
	 */
	private void tab(boolean toAll) {
		if (!apply()) {
			return;
		}
		load();
		all = toAll;
		scroll = 0;
		error = "";
		rebuildWidgets();
	}

	/** Moves each row's widgets to where the scroll puts them, hiding the ones out of the list. */
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

	/** The changed keys of both tabs applied, then back to where the screen came from. */
	private void done() {
		if (!apply()) {
			return;
		}
		Minecraft minecraft = Minecraft.getInstance();
		Preparation.current().filter(Preparation::needed)
				.ifPresentOrElse(p -> minecraft.gui.setScreen(new PreparingScreen(parent, p)), () -> minecraft.gui.setScreen(parent));
	}

	/** The changed keys of both tabs written and the mod initialised again with them; false, and said, when refused. */
	private boolean apply() {
		JsonObject change = new JsonObject();
		for (Setting setting : mod.settings()) {
			JsonElement now = values.get(setting.key());
			if (!now.equals(setting.value())) {
				craftkit.host.ConfigFile.mergeInto(change, setting.change(now));
			}
		}
		// All keys after: a key changed in both is the one typed in full.
		for (Leaf leaf : leaves) {
			String typed = edits.get(leaf.key());
			if (typed.equals(ConfigText.text(leaf.value()))) {
				continue;
			}
			JsonElement now = ConfigText.parse(leaf.value(), typed);
			if (now == null) {
				error = leaf.key() + ": not " + kind(leaf.value());
				return false;
			}
			if (now.equals(leaf.value())) {
				continue;
			}
			JsonObject at = change;
			for (String part : leaf.path().subList(0, leaf.path().size() - 1)) {
				if (!at.has(part) || !at.get(part).isJsonObject()) {
					at.add(part, new JsonObject());
				}
				at = at.getAsJsonObject(part);
			}
			at.add(leaf.path().get(leaf.path().size() - 1), now);
		}
		List<List<String>> removed = leaves.stream().filter(l -> toPreset.contains(l.key())).map(Leaf::path).toList();
		if (change.size() > 0 || !removed.isEmpty()) {
			String failed = mod.reconfigure(change, removed);
			if (!failed.isEmpty()) {
				error = failed;
				return false;
			}
		}
		return true;
	}

	private static String kind(JsonElement value) {
		if (value.isJsonArray()) {
			return "a list (as [a, b])";
		}
		if (value.isJsonPrimitive() && value.getAsJsonPrimitive().isNumber()) {
			return value.toString().contains(".") ? "a number" : "a whole number";
		}
		return "a value of its kind";
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
		graphics.centeredText(this.font, this.title, this.width / 2, 10, WHITE);

		graphics.enableScissor(left - 8, TOP - 4, left + WIDTH + 8, bottom() + 2);
		int rows = all ? leaves.size() : mod.settings().size();
		for (int i = 0; i < rows; i++) {
			int y = TOP + places.get(i).top() - scroll;
			String group = all ? leaves.get(i).group() : mod.settings().get(i).group();
			if (places.get(i).heading()) {
				graphics.text(this.font, group.toUpperCase(java.util.Locale.ROOT), left, y - 12, DIM);
			}
			if (all) {
				Leaf leaf = leaves.get(i);
				boolean changed = !edits.get(leaf.key()).equals(ConfigText.text(leaf.value()));
				String label = toPreset.contains(leaf.key()) ? leaf.label() + " → preset" : leaf.label();
				graphics.text(this.font, trim(label, WIDTH - 156), left, y + 6, changed || toPreset.contains(leaf.key()) ? 0xFFE8C860 : WHITE);
			} else {
				Setting setting = mod.settings().get(i);
				graphics.text(this.font, setting.label(), left, y + 6, WHITE);
				if (setting.kind().equals("number")) {
					int control = left + WIDTH - 150;
					graphics.centeredText(this.font, Component.literal(Setting.show(values.get(setting.key()))), control + 75, y + 6, WHITE);
				}
			}
		}
		graphics.disableScissor();

		// What the key under the mouse does, from the mod's own words for it.
		String about = "";
		if (mouseY >= TOP && mouseY < bottom() && mouseX >= left && mouseX < left + WIDTH - 152) {
			for (int i = 0; i < rows; i++) {
				int y = TOP + places.get(i).top() - scroll;
				if (mouseY >= y && mouseY < y + 20) {
					String key = all ? leaves.get(i).key() : mod.settings().get(i).key();
					about = mod.configDoc(key);
					if (!about.isEmpty()) {
						about = key + "\n" + about;
					}
				}
			}
		}

		int height = ListLayout.height(places);
		int view = bottom() - TOP;
		if (height > view) {
			int thumb = Math.max(12, view * view / height);
			int at = TOP + (view - thumb) * scroll / Math.max(1, height - view);
			graphics.fill(left + WIDTH + 4, TOP, left + WIDTH + 6, bottom(), LINE);
			graphics.fill(left + WIDTH + 4, at, left + WIDTH + 6, at + thumb, GREY);
		}
		if (!error.isEmpty()) {
			graphics.centeredText(this.font, Component.literal(trim(error, this.width - 16)), this.width / 2, this.height - 44, RED);
		}
		if (!about.isEmpty()) {
			tooltip(graphics, about, mouseX, mouseY);
		}
	}

	/** A dark box of wrapped lines by the mouse, kept on the screen. */
	private void tooltip(GuiGraphicsExtractor graphics, String text, int mouseX, int mouseY) {
		int width = Math.min(this.width - 16, 260);
		List<net.minecraft.util.FormattedCharSequence> lines = new ArrayList<>();
		for (String part : text.split("\n")) {
			lines.addAll(this.font.split(Component.literal(part), width - 8));
		}
		int height = lines.size() * 10 + 6;
		int x = Math.max(4, Math.min(mouseX + 10, this.width - width - 4));
		int y = Math.max(4, Math.min(mouseY + 12, this.height - height - 4));
		graphics.fill(x - 1, y - 1, x + width + 1, y + height + 1, GREY);
		graphics.fill(x, y, x + width, y + height, 0xF4101016);
		int ty = y + 3;
		for (net.minecraft.util.FormattedCharSequence line : lines) {
			graphics.text(this.font, line, x + 4, ty, WHITE);
			ty += 10;
		}
	}

	private String trim(String text, int width) {
		if (this.font.width(text) <= width) {
			return text;
		}
		String cut = text;
		while (cut.length() > 1 && this.font.width(cut + "…") > width) {
			cut = cut.substring(0, cut.length() - 1);
		}

		return cut + "…";
	}
}
