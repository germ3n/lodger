package craftkit.host.client;

import craftkit.host.LoadedMod;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import net.minecraft.util.FormattedCharSequence;

import java.util.List;

/**
 * Shown once a launch, over the title screen, when a Rust mod could not load: each one and why
 * (most often a jar with no native library for this computer, which names the platforms it does
 * carry). Continue goes on to the title screen without it; Quit quits.
 */
public final class FailedScreen extends Screen {
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFF9A9A9A;
	private static final int RED = 0xFFE06060;

	private final Screen parent;
	private final List<LoadedMod> failed;

	public FailedScreen(Screen parent, List<LoadedMod> failed) {
		super(Component.translatable("craftkit.failed.title"));
		this.parent = parent;
		this.failed = failed;
	}

	@Override
	protected void init() {
		int y = this.height - 40;
		addRenderableWidget(Button.builder(Component.translatable("craftkit.failed.continue"), button -> Minecraft.getInstance().gui.setScreen(parent))
				.bounds(this.width / 2 - 154, y, 150, 20).build());
		addRenderableWidget(Button.builder(Component.translatable("craftkit.preparing.quit"), button -> Minecraft.getInstance().stop())
				.bounds(this.width / 2 + 4, y, 150, 20).build());
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
		super.extractRenderState(graphics, mouseX, mouseY, partialTick);
		int centerX = this.width / 2;
		graphics.centeredText(this.font, this.title, centerX, 30, RED);
		int width = Math.min(this.width - 40, 420);
		int y = 56;
		for (LoadedMod mod : failed) {
			graphics.centeredText(this.font, Component.literal(mod.name().isEmpty() ? mod.id() : mod.name()), centerX, y, WHITE);
			y += 14;
			for (FormattedCharSequence line : this.font.split(Component.literal(mod.failure()), width)) {
				graphics.text(this.font, line, centerX - width / 2, y, GREY);
				y += 10;
			}
			y += 10;
		}
		graphics.centeredText(this.font, Component.translatable("craftkit.failed.hint"), centerX, this.height - 60, GREY);
	}

	@Override
	public boolean shouldCloseOnEsc() {
		return true;
	}

	@Override
	public void onClose() {
		Minecraft.getInstance().gui.setScreen(parent);
	}
}
