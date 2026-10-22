// Micro-interactions for the Glass style. Each helper is used as a ref callback, so it attaches
// to the element and cleans up with the component. Pointer work is batched into one animation
// frame and written as CSS variables; the stylesheet decides what moves. Everything is a no-op
// when motion is reduced.
import { onCleanup } from "solid-js";
import { isMotionReduced } from "~/studio/spring";

function onFrame(): (work: () => void) => void {
  let frame = 0;
  onCleanup(() => cancelAnimationFrame(frame));
  return (work) => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(work);
  };
}

/**
 * Pointer position as `--mx`/`--my` (0-100 %, for a glare), `--px`/`--py` (-0.5..0.5, for
 * parallax offsets) and a tilt as `--rx`/`--ry` in degrees, all easing back to centre on leave.
 * `tilt` is the largest tilt.
 */
export function pointerGlow(element: HTMLElement, tilt = 0): void {
  const schedule = onFrame();
  const move = (event: PointerEvent) => {
    if (isMotionReduced()) return;
    schedule(() => {
      const box = element.getBoundingClientRect();
      const x = (event.clientX - box.left) / box.width;
      const y = (event.clientY - box.top) / box.height;
      element.style.setProperty("--mx", `${(x * 100).toFixed(1)}%`);
      element.style.setProperty("--my", `${(y * 100).toFixed(1)}%`);
      element.style.setProperty("--px", (x - 0.5).toFixed(3));
      element.style.setProperty("--py", (y - 0.5).toFixed(3));
      element.style.setProperty("--rx", `${((0.5 - y) * tilt).toFixed(2)}deg`);
      element.style.setProperty("--ry", `${((x - 0.5) * tilt).toFixed(2)}deg`);
      element.dataset["hover"] = "";
    });
  };
  const leave = () =>
    schedule(() => {
      element.style.setProperty("--rx", "0deg");
      element.style.setProperty("--ry", "0deg");
      element.style.setProperty("--px", "0");
      element.style.setProperty("--py", "0");
      delete element.dataset["hover"];
    });
  element.addEventListener("pointermove", move);
  element.addEventListener("pointerleave", leave);
  onCleanup(() => {
    element.removeEventListener("pointermove", move);
    element.removeEventListener("pointerleave", leave);
  });
}

/** Pulls the element a little toward the pointer (`--gx`/`--gy` in px) and lets go on leave. */
export function magnetic(element: HTMLElement, strength = 0.18): void {
  const schedule = onFrame();
  const move = (event: PointerEvent) => {
    if (isMotionReduced()) return;
    schedule(() => {
      const box = element.getBoundingClientRect();
      const dx = event.clientX - (box.left + box.width / 2);
      const dy = event.clientY - (box.top + box.height / 2);
      element.style.setProperty("--gx", `${(dx * strength).toFixed(1)}px`);
      element.style.setProperty("--gy", `${(dy * strength).toFixed(1)}px`);
    });
  };
  const leave = () =>
    schedule(() => {
      element.style.setProperty("--gx", "0px");
      element.style.setProperty("--gy", "0px");
    });
  element.addEventListener("pointermove", move);
  element.addEventListener("pointerleave", leave);
  onCleanup(() => {
    element.removeEventListener("pointermove", move);
    element.removeEventListener("pointerleave", leave);
  });
}

/** A soft ring that grows from the press point. The element needs `overflow: hidden`. */
export function ripple(element: HTMLElement, className: string): void {
  const press = (event: PointerEvent) => {
    if (isMotionReduced() || event.button !== 0) return;
    const box = element.getBoundingClientRect();
    const size = Math.hypot(box.width, box.height) * 2;
    const ring = document.createElement("span");
    ring.className = className;
    ring.style.width = ring.style.height = `${size}px`;
    ring.style.left = `${event.clientX - box.left - size / 2}px`;
    ring.style.top = `${event.clientY - box.top - size / 2}px`;
    element.append(ring);
    ring
      .animate(
        [
          { transform: "scale(0)", opacity: 0.45 },
          { transform: "scale(1)", opacity: 0 },
        ],
        { duration: 650, easing: "cubic-bezier(0.2, 0.8, 0.3, 1)" },
      )
      .finished.then(
        () => ring.remove(),
        () => ring.remove(),
      );
  };
  element.addEventListener("pointerdown", press);
  onCleanup(() => element.removeEventListener("pointerdown", press));
}

/** A quick spring "pop" (scale overshoot), for a value that just changed. */
export function pop(element: HTMLElement | undefined, amount = 1.18): void {
  if (!element || isMotionReduced()) return;
  element.animate([{ scale: "1" }, { scale: String(amount) }, { scale: "0.96" }, { scale: "1" }], {
    duration: 520,
    easing: "cubic-bezier(0.2, 0.9, 0.3, 1)",
  });
}

/** Stretches a sliding thumb sideways while it travels, like a drop of liquid. */
export function stretch(element: HTMLElement | undefined, distance: number): void {
  if (!element || isMotionReduced() || distance === 0) return;
  const amount = Math.min(1.45, 1 + Math.abs(distance) * 0.12);
  element.animate([{ scale: "1 1" }, { scale: `${amount} 0.9`, offset: 0.35 }, { scale: "1 1" }], {
    duration: 520,
    easing: "cubic-bezier(0.3, 0.7, 0.3, 1)",
  });
}

/** Rolls new text in from below (old digits leave upward); used for counters and clocks. */
export function roll(element: HTMLElement | undefined, direction: 1 | -1 = 1): void {
  if (!element || isMotionReduced()) return;
  element.animate(
    [
      { translate: `0 ${direction * 60}%`, opacity: 0, filter: "blur(3px)" },
      { translate: "0 0", opacity: 1, filter: "blur(0)" },
    ],
    { duration: 380, easing: "cubic-bezier(0.2, 1, 0.3, 1)" },
  );
}
