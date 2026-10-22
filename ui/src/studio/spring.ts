// Physics for the editor: a damped spring integrator (several values at once), Apple-style
// momentum projection and rubber-banding. Fixed sub-steps keep it stable at any frame rate;
// nothing runs once every value is at rest.

export interface SpringParams {
  /** Seconds for one oscillation period of the undamped spring (feel: "response"). */
  response: number;
  /** 1 = critically damped, < 1 bounces. */
  dampingRatio: number;
}

/** Snappy with a hint of overshoot: drops, snaps, resizes. */
export const SNAPPY: SpringParams = { response: 0.32, dampingRatio: 0.78 };
/** Softer settle for panels and pop-ins. */
export const GENTLE: SpringParams = { response: 0.45, dampingRatio: 0.86 };

const STEP_S = 1 / 240;
const MAX_FRAME_S = 1 / 30;
const REST_DISTANCE = 0.25;
const REST_SPEED = 2;

/** The Esc-menu motion setting, falling back to the OS preference for "System". */
export function isMotionReduced(): boolean {
  if (typeof document === "undefined") return false;
  const setting = document.documentElement.dataset["motion"];
  if (setting === "reduced") return true;
  if (setting === "full") return false;
  return typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export interface SpringHandle {
  /** Stop now, leaving the values where they are. */
  stop(): void;
}

/**
 * Animate `from` to `to` starting with `velocity` (units per second). `onFrame` receives the
 * current values every frame; `onRest` runs once when settled (values snapped to `to`).
 */
export function runSpring(
  from: readonly number[],
  to: readonly number[],
  velocity: readonly number[],
  params: SpringParams,
  onFrame: (values: number[]) => void,
  onRest?: () => void,
): SpringHandle {
  if (isMotionReduced()) {
    onFrame([...to]);
    onRest?.();
    return { stop() {} };
  }
  const values = [...from];
  const speeds = to.map((_, index) => velocity[index] ?? 0);
  const stiffness = (2 * Math.PI / params.response) ** 2;
  const damping = (4 * Math.PI * params.dampingRatio) / params.response;
  let frame = 0;
  let last = performance.now();
  let isStopped = false;

  const isAtRest = () =>
    values.every((value, index) => Math.abs((to[index] ?? value) - value) < REST_DISTANCE && Math.abs(speeds[index] ?? 0) < REST_SPEED);

  const tick = (now: number) => {
    if (isStopped) return;
    let remaining = Math.min((now - last) / 1000, MAX_FRAME_S);
    last = now;
    while (remaining > 0) {
      const dt = Math.min(STEP_S, remaining);
      remaining -= dt;
      for (let index = 0; index < values.length; index += 1) {
        const target = to[index] ?? 0;
        const displacement = (values[index] ?? 0) - target;
        const speed = (speeds[index] ?? 0) + (-stiffness * displacement - damping * (speeds[index] ?? 0)) * dt;
        speeds[index] = speed;
        values[index] = (values[index] ?? 0) + speed * dt;
      }
    }
    if (isAtRest()) {
      onFrame([...to]);
      onRest?.();
      return;
    }
    onFrame([...values]);
    frame = requestAnimationFrame(tick);
  };

  frame = requestAnimationFrame(tick);
  return {
    stop() {
      isStopped = true;
      cancelAnimationFrame(frame);
    },
  };
}

/** Where a flick would come to rest (UIScrollView deceleration, rate 0.998 per ms). */
export function project(velocity: number, decelerationRate = 0.998): number {
  return ((velocity / 1000) * decelerationRate) / (1 - decelerationRate);
}

/** Resistance past a limit: the further you pull, the less it moves (iOS rubber band). */
export function rubberBand(overshoot: number, dimension: number, constant = 0.55): number {
  if (dimension <= 0) return 0;
  const sign = Math.sign(overshoot);
  const magnitude = Math.abs(overshoot);
  return sign * (1 - 1 / ((magnitude * constant) / dimension + 1)) * dimension;
}

/** Clamp `value` into [min, max] with rubber-banding outside. */
export function rubberClamp(value: number, min: number, max: number, dimension: number): number {
  if (value < min) return min + rubberBand(value - min, dimension);
  if (value > max) return max + rubberBand(value - max, dimension);
  return value;
}

/** Recent pointer samples -> velocity in px/s (last ~100 ms, robust to a final stall). */
export class VelocityTracker {
  private samples: { t: number; x: number; y: number }[] = [];

  add(x: number, y: number, t = performance.now()): void {
    this.samples.push({ t, x, y });
    while (this.samples.length > 2 && t - (this.samples[0]?.t ?? t) > 100) this.samples.shift();
  }

  velocity(now = performance.now()): { x: number; y: number } {
    const first = this.samples[0];
    const last = this.samples[this.samples.length - 1];
    if (!first || !last || last.t === first.t || now - last.t > 80) return { x: 0, y: 0 };
    const dt = (last.t - first.t) / 1000;
    return { x: (last.x - first.x) / dt, y: (last.y - first.y) / dt };
  }

  reset(): void {
    this.samples = [];
  }
}
