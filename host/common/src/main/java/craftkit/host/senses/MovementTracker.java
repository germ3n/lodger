package craftkit.host.senses;

import java.util.ArrayDeque;
import java.util.Deque;

/**
 * How one player moves, read off their position and facing once a tick, so a mod can read fear
 * without being told ({@code docs/entity.md}, Senses): pace, stops, turns, looks back, backing
 * away, sprint bursts, standing still. Pure arithmetic over the last minute of samples; the
 * JUnit tests feed it synthetic walks.
 */
public final class MovementTracker {
	/** Ticks a second. */
	public static final int TPS = 20;
	private static final int MINUTE = 60 * TPS;
	private static final int FIVE_SECONDS = 5 * TPS;
	/** A turn of more than this many degrees within a second counts as a turn. */
	private static final double TURN_DEGREES = 120;
	/** A turn and back within this many ticks counts as a look back. */
	private static final int LOOK_BACK_TICKS = 2 * TPS;

	private record Sample(double x, double y, double z, float yaw, boolean sprinting) {
	}

	private final Deque<Sample> samples = new ArrayDeque<>(MINUTE + 1);
	private int stops;
	private int turns;
	private int lookBacks;
	private double backingAwayTicks;
	private int sprintBursts;
	private int stillTicks;
	private final Deque<Integer> stopTicks = new ArrayDeque<>();
	private final Deque<Integer> turnTicks = new ArrayDeque<>();
	private final Deque<Integer> lookBackTicks = new ArrayDeque<>();
	private final Deque<Integer> backingTicks = new ArrayDeque<>();
	private final Deque<Integer> sprintTicks = new ArrayDeque<>();
	private int tick;
	private boolean wasStill;
	private float turnStartYaw;
	private int turnStartTick = -1;
	private int lastTurnTick = -1;
	private float yawBeforeLastTurn;

	/** One sample a tick. {@code yaw} in degrees, as the game gives it. */
	public void sample(double x, double y, double z, float yaw, boolean sprinting) {
		tick++;
		Sample previous = samples.peekLast();
		samples.addLast(new Sample(x, y, z, yaw, sprinting));
		while (samples.size() > MINUTE + 1) {
			samples.removeFirst();
		}

		if (previous == null) {
			return;
		}

		double moved = Math.hypot(x - previous.x, z - previous.z);
		boolean still = moved < 0.01;
		if (still) {
			stillTicks++;
			if (!wasStill) {
				stopTicks.addLast(tick);
			}
		} else {
			stillTicks = 0;
		}
		wasStill = still;

		float delta = yawDelta(yaw, previous.yaw);
		if (Math.abs(delta) > 1) {
			if (turnStartTick < 0) {
				turnStartTick = tick;
				turnStartYaw = previous.yaw;
			} else if (tick - turnStartTick > TPS) {
				turnStartTick = tick;
				turnStartYaw = previous.yaw;
			}

			if (Math.abs(yawDelta(yaw, turnStartYaw)) > TURN_DEGREES) {
				turnTicks.addLast(tick);
				if (lastTurnTick >= 0 && tick - lastTurnTick <= LOOK_BACK_TICKS && Math.abs(yawDelta(yaw, yawBeforeLastTurn)) < 45) {
					lookBackTicks.addLast(tick);
				}
				yawBeforeLastTurn = turnStartYaw;
				lastTurnTick = tick;
				turnStartTick = -1;
			}
		} else if (turnStartTick >= 0 && tick - turnStartTick > TPS) {
			turnStartTick = -1;
		}

		if (!still) {
			double facingX = -Math.sin(Math.toRadians(yaw));
			double facingZ = Math.cos(Math.toRadians(yaw));
			double dot = (x - previous.x) * facingX + (z - previous.z) * facingZ;
			if (dot < -0.005) {
				backingTicks.addLast(tick);
			}
		}

		if (sprinting && !previous.sprinting) {
			sprintTicks.addLast(tick);
		}

		expire(stopTicks);
		expire(turnTicks);
		expire(lookBackTicks);
		expire(backingTicks);
		expire(sprintTicks);
		stops = stopTicks.size();
		turns = turnTicks.size();
		lookBacks = lookBackTicks.size();
		backingAwayTicks = backingTicks.size();
		sprintBursts = sprintTicks.size();
	}

	private void expire(Deque<Integer> ticks) {
		while (!ticks.isEmpty() && tick - ticks.peekFirst() > MINUTE) {
			ticks.removeFirst();
		}
	}

	static float yawDelta(float a, float b) {
		float delta = (a - b) % 360;
		if (delta > 180) {
			delta -= 360;
		}
		if (delta < -180) {
			delta += 360;
		}
		return delta;
	}

	/** Blocks a second over the last five seconds. */
	public float pace() {
		if (samples.size() < 2) {
			return 0;
		}

		Sample[] all = samples.toArray(new Sample[0]);
		int from = Math.max(0, all.length - 1 - FIVE_SECONDS);
		double distance = 0;
		for (int i = from + 1; i < all.length; i++) {
			distance += Math.hypot(all[i].x - all[i - 1].x, all[i].z - all[i - 1].z);
		}

		return (float) (distance / ((all.length - 1 - from) / (double) TPS));
	}

	public int stops() {
		return stops;
	}

	public int turns() {
		return turns;
	}

	public int lookBacks() {
		return lookBacks;
	}

	/** Seconds moving against the facing in the last minute. */
	public float backingAway() {
		return (float) (backingAwayTicks / TPS);
	}

	public int sprintBursts() {
		return sprintBursts;
	}

	/** Seconds standing still right now. */
	public float stillSeconds() {
		return stillTicks / (float) TPS;
	}
}
