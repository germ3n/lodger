package craftkit.host.client;

import org.lwjgl.openal.AL10;
import org.lwjgl.openal.AL11;
import org.lwjgl.openal.EXTEfx;

/**
 * The `sound_warp` hint carried out on every sound as it starts ({@code ChannelMixin}): its pitch
 * lowered, or its source sent through OpenAL's effects (EFX): a reverb for the hall, an echo and a
 * distortion for the broken, a low-pass filter for the muffled. Each source is set or cleared as it
 * plays, since the game uses its sources again. A device without EFX keeps the slowed way alone.
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
			if (way == null && !ready) {
				return;
			}
			setUp();
			if (!ready) {
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
