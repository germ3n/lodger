package craftkit.host.client;

import org.lwjgl.openal.AL10;
import org.lwjgl.openal.AL11;
import org.lwjgl.openal.EXTEfx;

/**
 * The `sound_warp` hint carried out on every sound as it starts ({@code ChannelMixin}): its pitch
 * lowered, or its source sent through OpenAL's effects (EFX): a reverb for the hall, an echo and a
 * distortion for the broken, a low-pass filter for the muffled. Each source is set or cleared as it
 * plays, since the game uses its sources again. A device without EFX keeps the slowed way alone.
 *
 * <p>With Sound Physics Remastered (owner: "apply it on top more carefully"), which sets four sends
 * and a direct filter on every source as it plays and as it moves, the warp is set after it, and
 * takes no more than it must: the muffled way's filter over its own while the warp lasts; the hall
 * by lengthening its four reverbs into a hall far too big, put back as they were after; the broken
 * way's distortion on its first send alone, its other three kept.
 */
public final class SoundWarps {
	private SoundWarps() {
	}

	private static SoundWarp.Way cached;
	private static long cachedAt;
	private static boolean ready;
	private static boolean broken;
	private static int slot;
	private static int reverb;
	private static int distortion;
	private static int lowpass;
	private static SoundWarp.Way loaded;
	/** Sound Physics Remastered's reverbs, when it is there, and what they were before the hall. */
	private static Boolean physics;
	private static int[] physicsReverbs;
	private static float[][] physicsWas;
	private static boolean physicsHall;

	/** Whether Sound Physics Remastered is loaded. */
	private static boolean physics() {
		if (physics == null) {
			try {
				Class.forName("com.sonicether.soundphysics.SoundPhysics", false, SoundWarps.class.getClassLoader());
				physics = true;
			} catch (ClassNotFoundException absent) {
				physics = false;
			}
		}
		return physics;
	}

	/** The way the active `sound_warp` hint names, read again a few times a second; null for none. */
	public static synchronized SoundWarp.Way current() {
		long now = System.currentTimeMillis();
		if (now - cachedAt < 200) {
			return cached;
		}
		cachedAt = now;
		SoundWarp.Way way = null;
		for (Hints.Active active : Hints.active()) {
			if (active.hint().hint().equals("sound_warp")) {
				way = SoundWarp.of(active.hint().detail());
			}
		}
		cached = way;
		return way;
	}

	/** A sound's pitch, under the warp now. */
	public static float pitch(float pitch) {
		return SoundWarp.pitch(current(), pitch);
	}

	/** On the sound engine's thread, as a source starts: its effects set for the warp now, or cleared. */
	public static void apply(int source) {
		if (broken) {
			return;
		}
		try {
			SoundWarp.Way way = current();
			if (way == null && !ready && !physicsHall) {
				return;
			}
			setUp();
			if (!ready) {
				return;
			}
			if (physics()) {
				onPhysics(source, way);
				return;
			}
			int send = EXTEfx.AL_EFFECTSLOT_NULL;
			int direct = EXTEfx.AL_FILTER_NULL;
			if (way == SoundWarp.Way.HALL || way == SoundWarp.Way.BROKEN) {
				load(way);
				send = slot;
			}
			if (way == SoundWarp.Way.MUFFLED) {
				direct = lowpass;
			}
			AL11.alSource3i(source, EXTEfx.AL_AUXILIARY_SEND_FILTER, send, 0, EXTEfx.AL_FILTER_NULL);
			AL10.alSourcei(source, EXTEfx.AL_DIRECT_FILTER, direct);
		} catch (Throwable unsupported) {
			// No EFX on this device: the slowed way alone, and nothing tried again.
			broken = true;
		}
	}

	/**
	 * With Sound Physics Remastered: set after its own (the mixin runs later), touching only what
	 * the way needs, and its own left to it otherwise.
	 */
	private static void onPhysics(int source, SoundWarp.Way way) throws ReflectiveOperationException {
		hall(way == SoundWarp.Way.HALL);
		if (way == SoundWarp.Way.MUFFLED) {
			AL10.alSourcei(source, EXTEfx.AL_DIRECT_FILTER, lowpass);
		}
		if (way == SoundWarp.Way.BROKEN) {
			load(way);
			AL11.alSource3i(source, EXTEfx.AL_AUXILIARY_SEND_FILTER, slot, 0, EXTEfx.AL_FILTER_NULL);
		}
	}

	/** A reverb's decay time and late gain keys, by its kind: the EAX reverb's or the plain one's. */
	private static int[] reverbKeys(int reverb) {
		return EXTEfx.alGetEffecti(reverb, EXTEfx.AL_EFFECT_TYPE) == EXTEfx.AL_EFFECT_EAXREVERB
				? new int[] {EXTEfx.AL_EAXREVERB_DECAY_TIME, EXTEfx.AL_EAXREVERB_LATE_REVERB_GAIN}
				: new int[] {EXTEfx.AL_REVERB_DECAY_TIME, EXTEfx.AL_REVERB_LATE_REVERB_GAIN};
	}

	/** Sound Physics' four reverbs lengthened into a hall far too big, or put back as they were. */
	private static void hall(boolean on) throws ReflectiveOperationException {
		if (on == physicsHall) {
			return;
		}
		if (physicsReverbs == null) {
			Class<?> sp = Class.forName("com.sonicether.soundphysics.SoundPhysics", false, SoundWarps.class.getClassLoader());
			physicsReverbs = new int[4];
			for (int i = 0; i < 4; i++) {
				var field = sp.getDeclaredField("reverb" + i);
				field.setAccessible(true);
				physicsReverbs[i] = field.getInt(null);
			}
		}
		if (on) {
			physicsWas = new float[4][];
			for (int i = 0; i < 4; i++) {
				int r = physicsReverbs[i];
				int[] keys = reverbKeys(r);
				physicsWas[i] = new float[] {EXTEfx.alGetEffectf(r, keys[0]), EXTEfx.alGetEffectf(r, keys[1])};
				EXTEfx.alEffectf(r, keys[0], 9f + i);
				EXTEfx.alEffectf(r, keys[1], 2.5f);
			}
		} else if (physicsWas != null) {
			for (int i = 0; i < 4; i++) {
				int[] keys = reverbKeys(physicsReverbs[i]);
				EXTEfx.alEffectf(physicsReverbs[i], keys[0], physicsWas[i][0]);
				EXTEfx.alEffectf(physicsReverbs[i], keys[1], physicsWas[i][1]);
			}
		}
		physicsHall = on;
	}

	/**
	 * As a source moves: Sound Physics sets its own again then, so the warp is set again over it;
	 * without it, nothing (the source was set as it played).
	 */
	public static void applyMoved(int source) {
		if (physics() && (current() != null || physicsHall)) {
			apply(source);
		}
	}

	private static void setUp() {
		if (ready) {
			return;
		}
		slot = EXTEfx.alGenAuxiliaryEffectSlots();
		reverb = EXTEfx.alGenEffects();
		EXTEfx.alEffecti(reverb, EXTEfx.AL_EFFECT_TYPE, EXTEfx.AL_EFFECT_REVERB);
		EXTEfx.alEffectf(reverb, EXTEfx.AL_REVERB_DECAY_TIME, 9f);
		EXTEfx.alEffectf(reverb, EXTEfx.AL_REVERB_GAIN, 0.9f);
		EXTEfx.alEffectf(reverb, EXTEfx.AL_REVERB_DIFFUSION, 1f);
		EXTEfx.alEffectf(reverb, EXTEfx.AL_REVERB_LATE_REVERB_GAIN, 2.5f);
		distortion = EXTEfx.alGenEffects();
		EXTEfx.alEffecti(distortion, EXTEfx.AL_EFFECT_TYPE, EXTEfx.AL_EFFECT_DISTORTION);
		EXTEfx.alEffectf(distortion, EXTEfx.AL_DISTORTION_EDGE, 0.95f);
		EXTEfx.alEffectf(distortion, EXTEfx.AL_DISTORTION_GAIN, 0.6f);
		lowpass = EXTEfx.alGenFilters();
		EXTEfx.alFilteri(lowpass, EXTEfx.AL_FILTER_TYPE, EXTEfx.AL_FILTER_LOWPASS);
		EXTEfx.alFilterf(lowpass, EXTEfx.AL_LOWPASS_GAIN, 0.8f);
		EXTEfx.alFilterf(lowpass, EXTEfx.AL_LOWPASS_GAINHF, 0.05f);
		ready = true;
	}

	/** The slot's effect for the way, loaded when it changes. */
	private static void load(SoundWarp.Way way) {
		if (loaded == way) {
			return;
		}
		EXTEfx.alAuxiliaryEffectSloti(slot, EXTEfx.AL_EFFECTSLOT_EFFECT, way == SoundWarp.Way.HALL ? reverb : distortion);
		loaded = way;
	}
}
