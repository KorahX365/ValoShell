import { describe, expect, it } from "vitest";
import { clampRect, collectGuides, resizeFromEdge, snapMove, snapResize, toFraction, toPx, touchingGuides } from "./geometry";

const stage = { w: 1000, h: 500 };

describe("geometry", () => {
  it("round-trips fractions and pixels", () => {
    const px = toPx({ x: 0.1, y: 0.2, w: 0.3, h: 0.4 }, stage);
    expect(px).toEqual({ x: 100, y: 100, w: 300, h: 200 });
    expect(toFraction(px, stage)).toEqual({ x: 0.1, y: 0.2, w: 0.3, h: 0.4 });
  });

  it("resizes from the west edge keeping the east edge fixed", () => {
    const next = resizeFromEdge({ x: 100, y: 100, w: 200, h: 100 }, "w", 50, 0, false);
    expect(next).toEqual({ x: 150, y: 100, w: 150, h: 100 });
  });

  it("keeps the aspect ratio on a corner when asked", () => {
    const next = resizeFromEdge({ x: 0, y: 0, w: 200, h: 100 }, "se", 100, 0, true);
    expect(next.w / next.h).toBeCloseTo(2);
  });

  it("clamps into the stage and to the minimum size", () => {
    const next = clampRect({ x: -40, y: 480, w: 10, h: 10 }, stage, { w: 50, h: 40 });
    expect(next).toEqual({ x: 0, y: 460, w: 50, h: 40 });
  });

  it("snaps the nearest edge or centre to a guide within the threshold", () => {
    const guides = collectGuides(stage, [{ x: 300, y: 0, w: 100, h: 100 }]);
    const snapped = snapMove({ x: 296, y: 203, w: 50, h: 50 }, guides, 10);
    expect(snapped.x).toBe(300);
    // The bottom edge (253) is the closest point to a guide (the stage centre line, 250).
    expect(snapped.y + snapped.h).toBe(250);
  });

  it("does not snap beyond the threshold", () => {
    const guides = collectGuides(stage, []);
    expect(snapMove({ x: 120, y: 120, w: 50, h: 50 }, guides, 10)).toEqual({ x: 120, y: 120, w: 50, h: 50 });
  });

  it("snaps only the dragged edges while resizing", () => {
    const guides = collectGuides(stage, []);
    const next = snapResize({ x: 100, y: 100, w: 393, h: 100 }, "e", guides, 10);
    expect(next).toEqual({ x: 100, y: 100, w: 400, h: 100 });
  });

  it("reports which guides a rect touches", () => {
    const guides = collectGuides(stage, []);
    const touching = touchingGuides({ x: 475, y: 0, w: 50, h: 40 }, guides, 1);
    expect(touching.xs).toEqual([500]);
    expect(touching.ys).toEqual([0]);
  });
});
