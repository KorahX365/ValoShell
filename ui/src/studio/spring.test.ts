import { describe, expect, it } from "vitest";
import { VelocityTracker, project, rubberBand, rubberClamp } from "./spring";

describe("spring helpers", () => {
  it("projects a flick in its direction and not at all at rest", () => {
    expect(project(1000, 0.998)).toBeCloseTo(499, 0);
    expect(project(-1000, 0.993)).toBeLessThan(0);
    expect(project(0)).toBe(0);
  });

  it("rubber-bands with diminishing returns and keeps the sign", () => {
    const small = rubberBand(50, 200);
    const large = rubberBand(500, 200);
    expect(small).toBeGreaterThan(0);
    expect(small).toBeLessThan(50);
    expect(large).toBeLessThan(200);
    expect(rubberBand(-50, 200)).toBeCloseTo(-small);
  });

  it("only resists outside the limits", () => {
    expect(rubberClamp(100, 0, 500, 200)).toBe(100);
    expect(rubberClamp(-100, 0, 500, 200)).toBeGreaterThan(-100);
    expect(rubberClamp(600, 0, 500, 200)).toBeLessThan(600);
  });

  it("measures pointer velocity from recent samples", () => {
    const tracker = new VelocityTracker();
    tracker.add(0, 0, 1000);
    tracker.add(100, 50, 1050);
    const velocity = tracker.velocity(1060);
    expect(velocity.x).toBeCloseTo(2000);
    expect(velocity.y).toBeCloseTo(1000);
  });

  it("reports no velocity after the pointer stopped", () => {
    const tracker = new VelocityTracker();
    tracker.add(0, 0, 1000);
    tracker.add(100, 0, 1050);
    expect(tracker.velocity(1300)).toEqual({ x: 0, y: 0 });
  });
});
