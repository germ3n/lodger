package craftkit.host.client;

import craftkit.host.Preparation;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

/**
 * The screen over the title screen while the mods fetch and load what they need: a title, a
 * progress bar, one line of status, the model's sizes to pick with a note on each, and no way
 * past it but Quit, or Play without it where the mod says how (owner). On failure, Retry and
 * Quit. Shown by the loaders' title-screen hooks whenever {@link Preparation#needed()}
 * says so, so a player with the model already there never sees it.
 */
public final class PreparingScreen extends Screen {
	private static final int BAR_WIDTH = 300;
	private static final int BAR_HEIGHT = 8;
	private static final int WHITE = 0xFFFFFFFF;
	private static final int GREY = 0xFF9A9A9A;
	private static final int RED = 0xFFE06060;
	private static final int BAR_BACK = 0xFF2A2A2A;
	private static final int BAR_FRONT = 0xFFDADADA;

	private final Screen parent;
	private final Preparation preparation;
	private Button retry;
	private Button quit;
	private Button accept;
	private Button readTerms;
	/** The resource's choices, when it has any: the next one on each press. */
	private Button choice;
	/** Past it without the resource, where the mod names the change that does it. */
	private Button without;

	public PreparingScreen(Screen parent, Preparation preparation) {
		super(Component.translatable("craftkit.preparing.title"));
		this.parent = parent;
		this.preparation = preparation;
	}

	@Override
	protected void init() {
		preparation.start();
		int y = top() + 116;
		retry = addRenderableWidget(Button.builder(Component.translatable("craftkit.preparing.retry"), button -> preparation.start())
				.bounds(this.width / 2 - 154, y, 150, 20).build());
		quit = addRenderableWidget(Button.builder(Component.translatable("craftkit.preparing.quit"), button -> {
			preparation.cancel();
			Minecraft.getInstance().stop();
		}).bounds(this.width / 2 + 4, y, 150, 20).build());
		retry.visible = false;
		accept = addRenderableWidget(Button.builder(Component.translatable("craftkit.preparing.accept"), button -> preparation.pendingLicense().ifPresent(pending -> {
			try {
				preparation.accept(pending);
			} catch (java.io.IOException e) {
				craftkit.host.Host.LOGGER.warn("could not record the acceptance: {}", e.getMessage());
			}
		})).bounds(this.width / 2 - 154, y, 150, 20).build());
		readTerms = addRenderableWidget(Button.builder(Component.translatable("craftkit.preparing.read"), button -> preparation.pendingLicense().ifPresent(pending ->
				com.mojang.blaze3d.Blaze3D.openUri(java.net.URI.create(pending.resource().license().url()))))
				.bounds(this.width / 2 - 154, y + 24, 308, 20).build());
		accept.visible = false;
		readTerms.visible = false;
		choice = addRenderableWidget(Button.builder(Component.empty(), button -> preparation.shown().ifPresent(shown -> {
			java.util.List<craftkit.host.Resource.Choice> choices = shown.resource().choices();
			int now = 0;
			for (int i = 0; i < choices.size(); i++) {
				if (choices.get(i).chosen()) {
					now = i;
				}
			}
			preparation.choose(shown.mod(), choices.get((now + 1) % choices.size()));
		})).bounds(this.width / 2 - 154, y - 24, 308, 20).build());
		choice.visible = false;
		without = addRenderableWidget(Button.builder(Component.translatable("craftkit.preparing.without"), button -> preparation.shown().ifPresent(shown -> {
			com.google.gson.JsonObject change = shown.resource().without();
			if (change != null) {
				// The download stopped, its part file kept; the mod initialised again without it,
				// so nothing is needed and the screen closes.
				preparation.choose(shown.mod(), new craftkit.host.Resource.Choice("", change, false, ""));
			}
		}))
				.tooltip(net.minecraft.client.gui.components.Tooltip.create(Component.translatable("craftkit.preparing.without.tip")))
				.bounds(this.width / 2 - 154, y + 48, 308, 20).build());
		without.visible = false;
	}

	/** The top of the screen's middle block: the title, the bar, the lines, then the buttons. */
	private int top() {
		return this.height / 2 - 70;
	}

	@Override
	public boolean shouldCloseOnEsc() {
		return false;
	}

	@Override
	public void tick() {
		super.tick();
		Preparation.Progress progress = preparation.progress();
		retry.visible = progress.failed();
		boolean license = progress.phase() == Preparation.Phase.LICENSE;
		accept.visible = license;
		readTerms.visible = license;
		retry.active = !license;
		java.util.Optional<craftkit.host.Resource.Choice> chosen = preparation.shown()
				.flatMap(shown -> shown.resource().choices().stream().filter(craftkit.host.Resource.Choice::chosen).findFirst());
		choice.visible = chosen.isPresent();
		chosen.ifPresent(c -> choice.setMessage(Component.translatable("craftkit.preparing.choice", c.label())));
		without.visible = progress.phase() != Preparation.Phase.LOADING
				&& preparation.shown().map(shown -> shown.resource().without() != null).orElse(false);

		if (progress.done()) {
			Minecraft.getInstance().gui.setScreen(parent);
		}
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
		super.extractRenderState(graphics, mouseX, mouseY, partialTick);
		Preparation.Progress progress = preparation.progress();
		int centerX = this.width / 2;
		int top = top();

		if (progress.phase() == Preparation.Phase.LICENSE) {
			preparation.pendingLicense().ifPresent(pending -> {
				graphics.centeredText(this.font, Component.translatable("craftkit.preparing.terms", pending.resource().title()), centerX, 30, WHITE);
				graphics.centeredText(this.font, Component.literal(pending.resource().license().name()), centerX, 44, GREY);
				int textWidth = Math.min(this.width - 40, 400);
				int lineY = 64;
				for (net.minecraft.util.FormattedCharSequence line : this.font.split(Component.literal(pending.resource().license().text()), textWidth)) {
					graphics.text(this.font, line, centerX - textWidth / 2, lineY, WHITE);
					lineY += 10;
				}
			});

			return;
		}

		graphics.centeredText(this.font, this.title, centerX, top, WHITE);

		if (progress.failed()) {
			graphics.centeredText(this.font, Component.translatable("craftkit.preparing.failed"), centerX, top + 20, RED);
			graphics.centeredText(this.font, Component.literal(progress.error()), centerX, top + 34, GREY);

			return;
		}

		String heading = progress.phase() == Preparation.Phase.LOADING
				? Component.translatable("craftkit.preparing.loading", progress.title()).getString()
				: progress.title();
		graphics.centeredText(this.font, Component.literal(heading), centerX, top + 20, WHITE);

		int left = centerX - BAR_WIDTH / 2;
		int barTop = top + 40;
		graphics.fill(left, barTop, left + BAR_WIDTH, barTop + BAR_HEIGHT, BAR_BACK);

		if (progress.phase() == Preparation.Phase.LOADING) {
			// No fraction while loading: a marker sweeping back and forth.
			int span = BAR_WIDTH - 40;
			int offset = (int) ((System.currentTimeMillis() / 8) % (span * 2));
			int x = left + (offset < span ? offset : span * 2 - offset);
			graphics.fill(x, barTop, x + 40, barTop + BAR_HEIGHT, BAR_FRONT);
		} else {
			int filled = (int) (BAR_WIDTH * progress.fraction());
			graphics.fill(left, barTop, left + filled, barTop + BAR_HEIGHT, BAR_FRONT);
		}

		graphics.centeredText(this.font, Component.literal(progress.line()), centerX, barTop + 16, GREY);

		// The chosen size's note: how it differs, and the VRAM it wants.
		preparation.shown()
				.flatMap(shown -> shown.resource().choices().stream().filter(craftkit.host.Resource.Choice::chosen).findFirst())
				.filter(c -> !c.note().isEmpty())
				.ifPresent(c -> {
					int noteY = barTop + 30;
					for (net.minecraft.util.FormattedCharSequence line : this.font.split(Component.literal(c.note()), Math.min(this.width - 40, 308))) {
						graphics.centeredText(this.font, line, centerX, noteY, GREY);
						noteY += 10;
					}
				});
	}
}
