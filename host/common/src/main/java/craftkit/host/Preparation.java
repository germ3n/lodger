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
	private volatile Thread thread;

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

	/** The mods it prepares: the title screen's settings button lists those with settings. */
	public List<LoadedMod> mods() {
		return mods;
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
	 * A resource with choices, with its mod: the one on screen, or after a failure (nothing on
	 * screen then) the first needed one, so its size can still be changed (the button vanished).
	 */
	public Optional<PendingLicense> choosable() {
		Optional<PendingLicense> now = shown().filter(s -> !s.resource().choices().isEmpty());
		if (now.isPresent()) {
			return now;
		}
		for (LoadedMod mod : mods) {
			if (!mod.loaded()) {
				continue;
			}
			for (Resource resource : mod.resources()) {
				if (!resource.choices().isEmpty() && resource.required()) {
					return Optional.of(new PendingLicense(mod, resource));
				}
			}
		}

		return Optional.empty();
	}

	/**
	 * A resource with a way to play without it, needed or not, with its mod: the screen's Play
	 * without it, which must be there when the download or the load failed too (it was not, and a
	 * failing download left only Quit).
	 */
	public Optional<PendingLicense> playableWithout() {
		Optional<PendingLicense> now = shown().filter(s -> s.resource().without() != null);
		if (now.isPresent()) {
			return now;
		}
		for (LoadedMod mod : mods) {
			if (!mod.loaded()) {
				continue;
			}
			for (Resource resource : mod.resources()) {
				if (resource.without() != null && resource.required()) {
					return Optional.of(new PendingLicense(mod, resource));
				}
			}
		}

		return Optional.empty();
	}

	/**
	 * The screen's model choice: the running work stopped, the change written to the mod's
	 * config and the mod initialised again with it, and the work started over for what it now
	 * needs. Off the render thread.
	 */
	/** A size being switched: the cancelled download is no failure to show or try again. */
	private final java.util.concurrent.atomic.AtomicInteger switching = new java.util.concurrent.atomic.AtomicInteger();

	/** Whether a switch is waiting or under way: a count, so the first one done does not clear a second still queued. */
	public boolean switching() {
		return switching.get() > 0;
	}

	/**
	 * One switch after another, in the order clicked: two at once (a size, then Play without
	 * it) both read the old work's thread, and the second's start found the first's download
	 * running and let it run.
	 */
	private final java.util.concurrent.ExecutorService switches = java.util.concurrent.Executors.newSingleThreadExecutor(r -> {
		Thread switcher = new Thread(r, "craftkit-choose");
		switcher.setDaemon(true);
		return switcher;
	});

	public void choose(LoadedMod mod, Resource.Choice choice) {
		switching.incrementAndGet();
		switches.execute(() -> {
			try {
				cancel();
				Thread running = thread;
				if (running != null) {
					running.interrupt();
					// Until it has stopped, however long: a load or a check that ignores the
					// interrupt ran on past three seconds, and its "ready" or its failure landed
					// over the new size.
					try {
						running.join();
					} catch (InterruptedException e) {
						Thread.currentThread().interrupt();
						return;
					}
				}
				pendingLicense.set(null);
				String error;
				try {
					error = mod.reconfigure(choice.config());
				} catch (RuntimeException e) {
					// A reply it could not read: said, never a switch stuck with its buttons hidden.
					error = String.valueOf(e.getMessage());
				}
				if (!error.isEmpty()) {
					progress.set(new Progress(Phase.FAILED, mod.name(), 0, "", mod.id() + ": " + error));
					return;
				}
				start();
			} finally {
				switching.decrementAndGet();
			}
		});
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
				// Joined, so a last mirror of the download never lands after what comes next (a
				// stale "downloading" over the failure left the screen with no Try again).
				try {
					watcher.join(1000);
				} catch (InterruptedException e) {
					Thread.currentThread().interrupt();
				}
				current.set(null);
				fetching.set(null);

				if (downloader.state() != Downloader.State.DONE) {
					if (switching()) {
						// Stopped for another size: the switch starts again, nothing failed.
						return;
					}
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
