package craftkit.host.client;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import craftkit.host.LoadedMod;
import craftkit.host.Preparation;
import craftkit.host.Setting;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * A mod's settings, from the title screen (owner, 2026-11-09): the few keys it offers, grouped,
 * each a button that changes it (a press for the next value, or - and + for a number). Done
 * writes the changes into the mod's config file and initialises the mod again; when that leaves
 * something to fetch or load (a new model size), the preparing screen comes up at once.
 */
public final class SettingsScreen extends Screen {
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFF9A9A9A;
	private static final int ROW = 22;

	private final Screen parent;
	private final LoadedMod mod;
	private final Map<String, JsonElement> values = new LinkedHashMap<>();
	private String error = "";

	public SettingsScreen(Screen parent, LoadedMod mod) {
		super(Component.translatable("craftkit.settings.title", mod.name()));
		this.parent = parent;
		this.mod = mod;
		for (Setting setting : mod.settings()) {
			values.put(setting.key(), setting.value());
		}
	}

	@Override
	protected void init() {
		int left = this.width / 2 - 150;
		int y = 36;
		String group = null;
		for (Setting setting : mod.settings()) {
			if (!setting.group().equals(group)) {
				group = setting.group();
				y += 12;
			}
			int rowY = y;
			if (setting.kind().equals("number")) {
				addRenderableWidget(Button.builder(Component.literal("-"), b -> step(setting, false))
						.bounds(left + 160, rowY, 20, 20).build());
				addRenderableWidget(Button.builder(Component.literal("+"), b -> step(setting, true))
						.bounds(left + 280, rowY, 20, 20).build());
			} else {
				addRenderableWidget(Button.builder(Component.literal(Setting.show(values.get(setting.key()))), b -> {
					step(setting, true);
					b.setMessage(Component.literal(Setting.show(values.get(setting.key()))));
				}).bounds(left + 160, rowY, 140, 20).build());
			}
			y += ROW;
		}
		int bottom = Math.min(this.height - 28, y + 12);
		addRenderableWidget(Button.builder(Component.translatable("gui.done"), b -> done())
				.bounds(this.width / 2 - 154, bottom, 150, 20).build());
		addRenderableWidget(Button.builder(Component.translatable("gui.cancel"), b -> onClose())
				.bounds(this.width / 2 + 4, bottom, 150, 20).build());
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
		super.extractRenderState(graphics, mouseX, mouseY, partialTick);
		graphics.centeredText(this.font, this.title, this.width / 2, 12, WHITE);
		int left = this.width / 2 - 150;
		int y = 36;
		String group = null;
		for (Setting setting : mod.settings()) {
			if (!setting.group().equals(group)) {
				group = setting.group();
				graphics.text(this.font, group, left, y, GREY);
				y += 12;
			}
			graphics.text(this.font, setting.label(), left, y + 6, WHITE);
			if (setting.kind().equals("number")) {
				String shown = Setting.show(values.get(setting.key()));
				graphics.centeredText(this.font, Component.literal(shown), left + 230, y + 6, WHITE);
			}
			y += ROW;
		}
		if (!error.isEmpty()) {
			graphics.centeredText(this.font, Component.literal(error), this.width / 2, this.height - 44, 0xFFE06060);
		}
	}
}
