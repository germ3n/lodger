package craftkit.host;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

/**
 * Getting every loaded mod ready to play: fetch the required resources that are missing, one at
 * a time, then ask each mod to load what it loads ({@code warm}), and say how far along it all
 * is. The client shows it as a screen over the title screen; a dedicated server runs it at
 * startup and logs it. Both read {@link #progress()} from their own thread; the work runs on
 * one thread of its own.
 */
public final class Preparation {
	/** What the screen or the log shows right now. */
	public record Progress(Phase phase, String title, double fraction, String line, String error) {
		public boolean done() {
			return phase == Phase.READY;
		}

		public boolean failed() {
			return phase == Phase.FAILED;
		}
	}

	public enum Phase {
		/** A resource's terms wait to be accepted; {@link #pendingLicense()} says which. */
		LICENSE, DOWNLOADING, LOADING, READY, FAILED
	}

	/** A resource whose terms must be accepted before it is fetched. */
	public record PendingLicense(LoadedMod mod, Resource resource) {
	}

	private final List<LoadedMod> mods;
	private final Acceptance acceptance;
	private final boolean interactive;
	private final AtomicReference<PendingLicense> pendingLicense = new AtomicReference<>();
	private final Object licenseWait = new Object();
	private final AtomicReference<Progress> progress = new AtomicReference<>(new Progress(Phase.READY, "", 1, "", ""));
	private final AtomicReference<Downloader> current = new AtomicReference<>();
	/** The resource being fetched, and whose: for the screen's choice while it downloads. */
	private final AtomicReference<PendingLicense> fetching = new AtomicReference<>();
	private Thread thread;

	/**
	 * @param interactive whether someone can accept terms (a client with a screen). A dedicated
	 *                    server is not: it logs the terms and skips the resource until the
	 *                    operator accepts in {@code config/craftkit/accepted.json}.
	 */
	public Preparation(List<LoadedMod> mods, Acceptance acceptance, boolean interactive) {
		this.mods = mods;
		this.acceptance = acceptance;
		this.interactive = interactive;
	}

	/** The terms waiting to be accepted, while {@link Progress#phase()} is LICENSE. */
	public Optional<PendingLicense> pendingLicense() {
		return Optional.ofNullable(pendingLicense.get());
	}

	/** The screen's Accept: records the acceptance and lets the download start. */
	public void accept(PendingLicense pending) throws java.io.IOException {
		acceptance.accept(pending.mod().id(), pending.resource());

		synchronized (licenseWait) {
			licenseWait.notifyAll();
		}
	}

	/** Whether any resource needs the terms shown before it can be fetched. */
	public boolean licensesPending() {
		for (LoadedMod mod : mods) {
			if (!mod.loaded()) {
				continue;
			}

			for (Resource resource : mod.resources()) {
				if (resource.required() && !resource.present() && !acceptance.accepted(mod.id(), resource)) {
					return true;
				}
			}
		}

		return false;
	}

	/** Whether anything at all needs doing: a required resource missing or a mod not ready. */
	public boolean needed() {
		for (LoadedMod mod : mods) {
			if (!mod.loaded()) {
				continue;
			}

			for (Resource resource : mod.resources()) {
				if (resource.required() && !resource.present()) {
					return true;
				}
			}

			if (!mod.status().ready()) {
				return true;
			}
		}

		return false;
	}

	public Progress progress() {
		return progress.get();
	}

	/** Starts the work on a thread, once. Harmless to call again while it runs. */
	public synchronized void start() {
		if (thread != null && thread.isAlive()) {
			return;
		}

		thread = new Thread(this::run, "craftkit-prepare");
		thread.setDaemon(true);
		thread.start();
	}

	/** The resource on screen, with its mod: the terms waiting, or the one being fetched. */
	public Optional<PendingLicense> shown() {
		PendingLicense pending = pendingLicense.get();

		return Optional.ofNullable(pending != null ? pending : fetching.get());
	}

	/**
	 * The screen's model choice: the running work stopped, the change written to the mod's
	 * config and the mod initialised again with it, and the work started over for what it now
	 * needs. Off the render thread.
	 */
	public void choose(LoadedMod mod, Resource.Choice choice) {
		Thread switching = new Thread(() -> {
			cancel();
			Thread running = thread;
			if (running != null) {
				running.interrupt();
				try {
					running.join(3000);
				} catch (InterruptedException e) {
					Thread.currentThread().interrupt();
					return;
				}
			}
			pendingLicense.set(null);
			String error = mod.reconfigure(choice.config());
			if (!error.isEmpty()) {
				progress.set(new Progress(Phase.FAILED, mod.name(), 0, "", mod.id() + ": " + error));
				return;
			}
			start();
		}, "craftkit-choose");
		switching.setDaemon(true);
		switching.start();
	}

	/** Stops a running download; the screen's Quit button and the server's shutdown use it. */
	public void cancel() {
		Downloader downloader = current.get();

		if (downloader != null) {
			downloader.cancel();
		}
	}

	/** Runs the whole thing on the calling thread. The server uses this to block its startup. */
	public void run() {
		for (LoadedMod mod : mods) {
			if (!mod.loaded()) {
				continue;
			}

			for (Resource resource : mod.resources()) {
				if (!resource.required() || resource.present()) {
					continue;
				}

				if (!acceptance.accepted(mod.id(), resource)) {
					if (!interactive) {
						Log.LOGGER.warn("{}: {} comes under {} ({}). Read it, then add \"{}\" to \"accepted\" in {} and restart; until then the mod goes without it.\n{}",
								mod.id(), resource.title(), resource.license().name(), resource.license().url(), Acceptance.key(mod.id(), resource), acceptance.file(), resource.license().text());
						progress.set(new Progress(Phase.FAILED, resource.title(), 0, "", mod.id() + ": " + resource.id() + ": terms not accepted; see the log"));

						return;
					}

					// Wait for the screen's Accept (or Quit, which ends the process).
					pendingLicense.set(new PendingLicense(mod, resource));
					progress.set(new Progress(Phase.LICENSE, resource.title(), 0, resource.license().name(), ""));

					synchronized (licenseWait) {
						while (!acceptance.accepted(mod.id(), resource)) {
							try {
								licenseWait.wait(500);
							} catch (InterruptedException e) {
								Thread.currentThread().interrupt();

								return;
							}
						}
					}

					pendingLicense.set(null);
				}

				Downloader downloader = new Downloader(resource);
				current.set(downloader);
				fetching.set(new PendingLicense(mod, resource));
				Thread watcher = watch(downloader);
				downloader.run();
				watcher.interrupt();
				current.set(null);
				fetching.set(null);

				if (downloader.state() != Downloader.State.DONE) {
					progress.set(new Progress(Phase.FAILED, resource.title(), downloader.fraction(), "", mod.id() + ": " + resource.id() + ": " + downloader.error()));
					Log.LOGGER.warn("{}: could not fetch {}: {}", mod.id(), resource.id(), downloader.error());

					return;
				}

				Log.LOGGER.info("{}: fetched {} to {}", mod.id(), resource.id(), resource.path());
			}
		}

		for (LoadedMod mod : mods) {
			if (!mod.loaded() || mod.status().ready()) {
				continue;
			}

			progress.set(new Progress(Phase.LOADING, mod.name(), 0, mod.status().line(), ""));
			String error = mod.warm();

			if (!error.isEmpty()) {
				progress.set(new Progress(Phase.FAILED, mod.name(), 0, "", mod.id() + ": " + error));
				Log.LOGGER.warn("{}: did not load: {}", mod.id(), error);

				return;
			}

			Log.LOGGER.info("{}: {}", mod.id(), mod.status().line());
		}

		progress.set(new Progress(Phase.READY, "", 1, "", ""));
	}

	/** Mirrors a running download into {@link #progress()} ten times a second, for the screen. */
	private Thread watch(Downloader downloader) {
		Thread watcher = new Thread(() -> {
			int lastTenth = -1;

			while (!Thread.currentThread().isInterrupted()) {
				double fraction = downloader.fraction();
				progress.set(new Progress(Phase.DOWNLOADING, downloader.resource().title(), fraction, megabytes(downloader.bytes()) + (downloader.total() > 0 ? " / " + megabytes(downloader.total()) : ""), ""));

				int tenth = (int) (fraction * 10);
				if (tenth != lastTenth) {
					lastTenth = tenth;
					Log.LOGGER.info("{}: {}%", downloader.resource().title(), tenth * 10);
				}

				try {
					Thread.sleep(100);
				} catch (InterruptedException e) {
					return;
				}
			}
		}, "craftkit-prepare-watch");
		watcher.setDaemon(true);
		watcher.start();

		return watcher;
	}

	static String megabytes(long bytes) {
		return (bytes / (1024 * 1024)) + " MB";
	}

	/** The one preparation of the process, made by the host. */
	private static final AtomicReference<Preparation> INSTANCE = new AtomicReference<>();

	static void install(Preparation preparation) {
		INSTANCE.set(preparation);
	}

	public static Optional<Preparation> current() {
		return Optional.ofNullable(INSTANCE.get());
	}

	/** For tests: a preparation over no mods is ready at once. */
	static Preparation none(Acceptance acceptance) {
		return new Preparation(new ArrayList<>(), acceptance, true);
	}
}
